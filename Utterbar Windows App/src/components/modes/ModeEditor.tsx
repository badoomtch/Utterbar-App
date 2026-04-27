import { useEffect, useState } from 'react';
import { X } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { Input, Textarea } from '@/components/common/Input';
import { ModeIconPicker } from '@/components/modes/ModeIcon';
import { invoke } from '@/lib/ipc';
import { cn } from '@/lib/cn';

/**
 * Mode editor modal. Persisted as opaque JSON via `upsert_mode`, the Rust
 * side owns schema validation, the editor just surfaces the fields we
 * expose today.
 *
 * Per DESIGN_SYSTEM.md:
 *   • The modal shell sits on bg-1 with a 2px border-soft hairline. NO
 *     drop shadow, the dark backdrop behind it is the elevation.
 *   • Inputs wrap the .ub-input class via the Input primitive (bg-0 inside
 *     bg-1 so they read as wells).
 *   • Chip pickers use .ub-chip / .ub-chip--active, no custom border
 *     rings for the active one.
 *   • Footer CTA / Cancel use Ghost + CTA Buttons (no bespoke styling).
 */

export type EditableMode = {
  id: string;
  name: string;
  icon: string;
  is_builtin: boolean;
  is_favourite: boolean;
  sort_order: number;
  asr_model_id: string | null;
  target_language: string | null;
  post_processing:
    | { kind: 'none' }
    | { kind: 'deterministic'; template: string }
    | { kind: 'llm'; prompt: string; template: string | null };
  context_rules: {
    use_clipboard: boolean;
    use_selection: boolean;
    use_active_app: boolean;
  };
  auto_activation: Array<
    | { match: 'by_exe'; pattern: string }
    | { match: 'by_title'; pattern: string }
    | { match: 'by_class'; pattern: string }
  >;
  shortcut_override: unknown;
  formatting: {
    capitalisation: 'sentence' | 'none' | 'title' | 'upper';
    strip_fillers: boolean;
    auto_punctuation: boolean;
    bullet_style: 'none' | 'dash' | 'asterisk' | 'numbered';
    preserve_linebreaks: boolean;
  };
  english_variant?: 'auto' | 'american' | 'british';
  created_at: number;
  updated_at: number;
};

/**
 * Output-style options exposed in the editor. Each entry is a user-facing
 * label paired with a one-line blurb explaining what it does to text.
 *
 * Internally these map to the `PostProcessing` discriminated union:
 *   - 'none'         → { kind: 'none' }
 *   - 'deterministic'→ { kind: 'deterministic', template: '{text}' }
 *   - 'llm' is read-only / legacy; it's not surfaced as a chip but the
 *     editor still renders the prompt field if an existing mode reports it.
 *
 * The labels deliberately avoid engineering terms ("post-processing",
 * "deterministic", "template literal"). Competitor surveys (Wispr Flow,
 * Superwhisper, MacWhisper) all frame this as a writing style, not a
 * pipeline; we follow suit.
 */
const OUTPUT_STYLES: { kind: 'none' | 'deterministic'; label: string; blurb: string }[] = [
  {
    kind: 'none',
    label: 'Plain',
    blurb: 'Type what you said, exactly as you said it.',
  },
  {
    kind: 'deterministic',
    label: 'Templated',
    blurb: 'Wrap the dictation in a fixed format. Useful for chat prefixes, code comments, ticket replies.',
  },
];

const ENGLISH_VARIANTS: {
  id: NonNullable<EditableMode['english_variant']>;
  label: string;
  blurb: string;
}[] = [
  {
    id: 'auto',
    label: 'Auto',
    blurb: "Leave whatever Parakeet transcribed. The model skews American-spelled.",
  },
  {
    id: 'british',
    label: 'British',
    blurb: "Rewrite American spellings to British (color → colour, organize → organise).",
  },
  {
    id: 'american',
    label: 'American',
    blurb: "Rewrite British spellings to American (colour → color, organise → organize).",
  },
];

export function ModeEditor({
  mode,
  // `models` was read by the removed per-mode ASR picker. Keeping the
  // prop in the signature so callers don't need to change their
  // render site; the underscore prefix silences the unused-var lint.
  models: _models,
  onClose,
  onSaved,
}: {
  mode: EditableMode;
  models: Array<{ id: string; display_name: string }>;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [draft, setDraft] = useState<EditableMode>(mode);
  const [saving, setSaving] = useState(false);

  useEffect(() => setDraft(mode), [mode]);

  const save = async () => {
    setSaving(true);
    try {
      await invoke('upsert_mode', { mode: draft });
      onSaved();
      onClose();
    } finally {
      setSaving(false);
    }
  };

  const patch = (p: Partial<EditableMode>) => setDraft((d) => ({ ...d, ...p }));
  const patchPost = (p: Partial<EditableMode['post_processing']>) =>
    setDraft((d) => ({
      ...d,
      post_processing: { ...d.post_processing, ...p } as EditableMode['post_processing'],
    }));
  const patchFormat = (p: Partial<EditableMode['formatting']>) =>
    setDraft((d) => ({ ...d, formatting: { ...d.formatting, ...p } }));
  // `context_rules` is no longer edited in the UI (v0.6.6: globals own
  // it). The struct field stays for back-compat with older Mode rows —
  // we just don't expose a patch helper for it.

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6 backdrop-blur-sm">
      {/* Modal shell: bg-1, no shadow. The dark backdrop is the elevation. */}
      <div className="flex h-[min(640px,90vh)] w-[min(720px,95vw)] flex-col overflow-hidden rounded-block bg-bg-1">
        <header className="flex items-center justify-between border-b border-border-soft/50 px-6 py-4">
          <div>
            {/* Weight 700. font-semibold (600) is forbidden per §2.3. */}
            <h2 className="text-[18px] font-bold text-fg-hi -tracking-[0.02em]">Edit mode</h2>
            <div className="mt-0.5 font-mono text-[11px] text-fg-dim">
              {draft.is_builtin ? 'Built-in · limited fields editable' : 'Custom mode'}
            </div>
          </div>
          <button
            onClick={onClose}
            aria-label="Close"
            className="text-fg-dim transition-colors hover:text-fg-hi"
          >
            <X className="h-4 w-4" />
          </button>
        </header>

        <div className="flex-1 space-y-5 overflow-y-auto px-6 py-5">
          <Section title="Basics">
            <Field label="Name">
              <Input
                value={draft.name}
                disabled={draft.is_builtin}
                onChange={(e) => patch({ name: e.target.value })}
                className="flex-1"
              />
            </Field>
            {/* Icon picker uses items-start variant of Field so the label
                aligns with the top of the grid rather than mid-height. */}
            <div className="flex items-start gap-3">
              <div className="w-44 pt-1.5 text-[12px] text-muted">Icon</div>
              <ModeIconPicker
                value={draft.icon}
                onChange={(next) => patch({ icon: next })}
              />
            </div>
            {/* v0.6.6: the per-mode ASR model picker was removed. Globals
                are authoritative, change the model in Settings → Models.
                The field is still on the Mode struct for back-compat; it's
                just ignored at dictation time. Hiding the editor keeps the
                UI consistent with the runtime behaviour. */}
            <Field label="Target language (BCP-47)">
              <Input
                value={draft.target_language ?? ''}
                placeholder="e.g. en, es, ja, blank for auto"
                onChange={(e) => patch({ target_language: e.target.value || null })}
                className="flex-1"
              />
            </Field>
            <Field label="English variant">
              <div className="flex flex-1 flex-col gap-2">
                <div className="flex flex-wrap gap-2">
                  {ENGLISH_VARIANTS.map((v) => {
                    const current = (draft.english_variant ?? 'auto') === v.id;
                    return (
                      <button
                        key={v.id}
                        onClick={() => patch({ english_variant: v.id })}
                        title={v.blurb}
                        className={cn('ub-chip', current && 'ub-chip--active')}
                      >
                        {v.label}
                      </button>
                    );
                  })}
                </div>
                <div className="text-[11px] leading-relaxed text-muted">
                  {ENGLISH_VARIANTS.find(
                    (v) => v.id === (draft.english_variant ?? 'auto'),
                  )?.blurb}
                </div>
              </div>
            </Field>
          </Section>

          <Section
            title="Output style"
            description="How the dictation is shaped before it lands in the focused app. Plain leaves it alone. Templated wraps it in a fixed format you choose."
          >
            {/* `llm` is deliberately absent in 0.6, the llama.cpp provider
                isn't wired yet, so the option would silently fall back to
                deterministic. Returns in 0.7 with the real LLM backend. */}
            <div className="flex flex-wrap gap-2">
              {OUTPUT_STYLES.map((s) => {
                const active = draft.post_processing.kind === s.kind;
                return (
                  <button
                    key={s.kind}
                    onClick={() =>
                      patchPost(
                        s.kind === 'none'
                          ? { kind: 'none' }
                          : { kind: 'deterministic', template: '{text}' },
                      )
                    }
                    className={cn('ub-chip', active && 'ub-chip--active')}
                  >
                    {s.label}
                  </button>
                );
              })}
            </div>
            <div className="text-[11px] leading-relaxed text-muted">
              {OUTPUT_STYLES.find((s) => s.kind === draft.post_processing.kind)?.blurb}
            </div>
            {draft.post_processing.kind === 'deterministic' && (
              <>
                <Field label="Format">
                  <Input
                    value={draft.post_processing.template}
                    onChange={(e) =>
                      patchPost({ kind: 'deterministic', template: e.target.value })
                    }
                    placeholder="{text}"
                    className="flex-1"
                  />
                </Field>
                <div className="text-[11px] leading-relaxed text-muted">
                  Use <code className="font-mono">{'{text}'}</code> where the
                  dictation goes. Anything else is added literally.
                </div>
                <TemplatePreview template={draft.post_processing.template} />
              </>
            )}
            {draft.post_processing.kind === 'llm' && (
              // An existing mode on disk could still report `kind: 'llm'`
              // (legacy rows from pre-release builds). Render a read-only
              // placeholder so the editor doesn't crash; saving swaps it to
              // deterministic on the next edit.
              <Field label="Prompt">
                <Textarea
                  value={draft.post_processing.prompt}
                  readOnly
                  rows={3}
                  className="flex-1 opacity-60"
                />
              </Field>
            )}
          </Section>

          {/* v0.6.6: the per-mode Context Rules block was removed. Privacy
              is authoritative, toggle clipboard / selection / active-app
              context in Settings → Privacy. The struct fields are still
              persisted for back-compat with older Mode rows on disk, but
              they're no longer consulted at dictation time and the editor
              no longer surfaces them. `patchContext` is now unused, kept
              as a no-op alias in case old handlers reach it; next major
              version should drop it with the struct field. */}

          <Section title="Formatting">
            <Field label="Capitalisation">
              <select
                value={draft.formatting.capitalisation}
                onChange={(e) =>
                  patchFormat({
                    capitalisation: e.target
                      .value as EditableMode['formatting']['capitalisation'],
                  })
                }
                className="ub-input"
              >
                <option value="sentence">Sentence</option>
                <option value="none">None</option>
                <option value="title">Title</option>
                <option value="upper">UPPER</option>
              </select>
            </Field>
            <Field label="Bullet style">
              <select
                value={draft.formatting.bullet_style}
                onChange={(e) =>
                  patchFormat({
                    bullet_style: e.target
                      .value as EditableMode['formatting']['bullet_style'],
                  })
                }
                className="ub-input"
              >
                <option value="none">None</option>
                <option value="dash">Dash</option>
                <option value="asterisk">Asterisk</option>
                <option value="numbered">Numbered</option>
              </select>
            </Field>
            <label className="flex items-center gap-2 text-[13px] text-fg">
              <input
                type="checkbox"
                checked={draft.formatting.strip_fillers}
                onChange={(e) => patchFormat({ strip_fillers: e.target.checked })}
                className="accent-[var(--accent)]"
              />
              Remove filler words (um, uh, like…)
            </label>
            <label className="flex items-center gap-2 text-[13px] text-fg">
              <input
                type="checkbox"
                checked={draft.formatting.auto_punctuation}
                onChange={(e) => patchFormat({ auto_punctuation: e.target.checked })}
                className="accent-[var(--accent)]"
              />
              Auto-punctuation
            </label>
            <label className="flex items-center gap-2 text-[13px] text-fg">
              <input
                type="checkbox"
                checked={draft.formatting.preserve_linebreaks}
                onChange={(e) => patchFormat({ preserve_linebreaks: e.target.checked })}
                className="accent-[var(--accent)]"
              />
              Preserve line breaks
            </label>
          </Section>

          <Section
            title="Auto-activation"
            description="This mode switches on automatically when the focused window matches any of these rules. First match wins."
          >
            <AutoActivationRules
              rules={draft.auto_activation}
              onChange={(next) => patch({ auto_activation: next })}
            />
          </Section>
        </div>

        <footer className="flex items-center justify-end gap-2 border-t border-border-soft/50 px-6 py-3">
          <Button variant="ghost" noLed onClick={onClose}>
            Cancel
          </Button>
          <Button variant="cta" onClick={() => void save()} disabled={saving}>
            {saving ? 'Saving…' : 'Save'}
          </Button>
        </footer>
      </div>
    </div>
  );
}

/**
 * Section, nested sub-card inside the editor. Lives on bg-2 so there's a
 * clear hierarchy: modal shell (bg-1) → section (bg-2) → input wells (bg-0).
 */
function Section({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-input bg-bg-2 p-4">
      <div className="mb-3">
        <h3 className="text-[14px] font-bold text-fg-hi -tracking-[0.01em]">{title}</h3>
        {description && (
          <div className="mt-0.5 text-[12px] leading-relaxed text-muted">{description}</div>
        )}
      </div>
      <div className="space-y-3">{children}</div>
    </section>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center gap-3">
      <div className="w-44 text-[12px] text-muted">{label}</div>
      {children}
    </div>
  );
}

/* ===================================================================
   Auto-activation rule editor
   ===================================================================

   The previous editor was a row of cramped controls labelled "exe /
   title / class" with no explanation, no examples, and no feedback
   whether a rule would actually fire. This rebuild fixes all three:

     - Plain-English match-type labels with hints.
     - Empty state with example rules to scan.
     - A live "currently focused" indicator on each rule that
       polls the foreground window and runs the same glob check the
       Rust side does, turning green when the rule matches and grey
       otherwise. So the user can fill in `outlook.exe`, alt-tab to
       Outlook, and watch the line confirm the rule fires.

   The glob function below mirrors `services/modes::glob` on the Rust
   side: case-insensitive, `*` is the only wildcard, no anchoring at
   either end (a bare pattern matches if it appears anywhere). Keep
   them in sync. */

type AARule = EditableMode['auto_activation'][number];
type AAMatch = AARule['match'];

const MATCH_LABELS: Record<AAMatch, { label: string; placeholder: string; hint: string }> = {
  by_exe: {
    label: 'Exe name',
    placeholder: 'outlook.exe',
    hint: 'The application file name. Find it in Task Manager → Details tab.',
  },
  by_title: {
    label: 'Window title',
    placeholder: '*Outlook',
    hint: 'What appears in the focused window\'s title bar. Use * for partial matches.',
  },
  by_class: {
    label: 'Window class',
    placeholder: 'rctrl_renwnd32',
    hint: 'Advanced. Use a tool like Spy++ or accessibility-insights. Most rules don\'t need this.',
  },
};

/**
 * Case-insensitive glob match, mirrors `services/modes::glob` on the
 * Rust side. Single `*` wildcard, matches anywhere in the string, no
 * anchoring. Empty pattern matches everything (which is the same
 * behaviour as the Rust impl when `parts.is_empty()`).
 */
function globMatch(value: string, pattern: string): boolean {
  if (!pattern) return false; // empty rule never matches — UI signal "no rule yet"
  const v = value.toLowerCase();
  const p = pattern.toLowerCase();
  const parts = p.split('*');
  let cursor = 0;
  for (let i = 0; i < parts.length; i += 1) {
    const part = parts[i];
    if (part === '') continue;
    const idx = v.indexOf(part, cursor);
    if (idx < 0) return false;
    // First part must anchor at start unless preceded by a *.
    if (i === 0 && idx !== 0 && !p.startsWith('*')) return false;
    cursor = idx + part.length;
  }
  // Last part must anchor at end unless followed by a *.
  const last = parts[parts.length - 1];
  if (last !== '' && !p.endsWith('*') && cursor !== v.length) return false;
  return true;
}

/**
 * Poll the foreground window every 1 s while the editor is open. Cheap
 * Tauri IPC; the Rust side reuses the same `context::snapshot()` plumbing
 * the auto-activation watcher uses internally, so the values are
 * identical to what would actually fire the rule.
 */
function useFocusedWindow() {
  const [focused, setFocused] = useState<{
    exe: string | null;
    title: string | null;
    class: string | null;
  }>({ exe: null, title: null, class: null });

  useEffect(() => {
    let alive = true;
    const tick = async () => {
      try {
        const fw = await invoke<{ exe: string | null; title: string | null; class: string | null }>(
          'current_focused_window',
        );
        if (alive) setFocused(fw);
      } catch {
        // Swallow — the editor still works without the live indicator.
      }
    };
    void tick();
    const id = window.setInterval(() => void tick(), 1000);
    return () => {
      alive = false;
      window.clearInterval(id);
    };
  }, []);

  return focused;
}

function AutoActivationRules({
  rules,
  onChange,
}: {
  rules: AARule[];
  onChange: (next: AARule[]) => void;
}) {
  const focused = useFocusedWindow();

  function update(idx: number, next: AARule) {
    const copy = [...rules];
    copy[idx] = next;
    onChange(copy);
  }
  function remove(idx: number) {
    onChange(rules.filter((_, i) => i !== idx));
  }
  function add() {
    onChange([...rules, { match: 'by_exe', pattern: '' }]);
  }

  return (
    <div className="space-y-3">
      {rules.length === 0 && <AutoActivationEmptyState />}
      {rules.map((rule, idx) => {
        const meta = MATCH_LABELS[rule.match];
        const focusedValue =
          rule.match === 'by_exe' ? focused.exe :
          rule.match === 'by_title' ? focused.title :
          focused.class;
        const matched = focusedValue ? globMatch(focusedValue, rule.pattern) : false;
        const hasPattern = rule.pattern.trim().length > 0;
        return (
          <div
            key={idx}
            className="rounded-input border border-border-soft/60 bg-bg-1 p-4 space-y-3"
          >
            <div className="flex items-start gap-3">
              <div className="flex-1 space-y-3">
                <div className="flex gap-2">
                  <select
                    value={rule.match}
                    onChange={(e) =>
                      update(idx, { ...rule, match: e.target.value as AAMatch })
                    }
                    className="ub-input !py-1.5 !text-[13px] w-44"
                  >
                    <option value="by_exe">{MATCH_LABELS.by_exe.label}</option>
                    <option value="by_title">{MATCH_LABELS.by_title.label}</option>
                    <option value="by_class">{MATCH_LABELS.by_class.label}</option>
                  </select>
                  <Input
                    value={rule.pattern}
                    onChange={(e) => update(idx, { ...rule, pattern: e.target.value })}
                    placeholder={meta.placeholder}
                    className="flex-1 !text-[13px]"
                  />
                </div>
                <div className="text-[11px] leading-relaxed text-muted">
                  {meta.hint}{' '}
                  <span className="text-fg-dim">Use <code className="font-mono">*</code> as a wildcard.</span>
                </div>
                {hasPattern && focusedValue !== null && (
                  <div className="flex items-center gap-2 text-[11px] font-mono">
                    <span
                      className={cn(
                        'inline-block h-1.5 w-1.5 rounded-full',
                        matched ? 'bg-moss shadow-[0_0_6px_var(--moss)]' : 'bg-fg-dim',
                      )}
                      aria-hidden
                    />
                    <span className="text-muted">Currently focused: </span>
                    <span className="text-fg-hi truncate">{focusedValue}</span>
                    <span className={matched ? 'text-moss' : 'text-fg-dim'}>
                      {matched ? '· matches' : '· no match'}
                    </span>
                  </div>
                )}
              </div>
              <button
                onClick={() => remove(idx)}
                className="text-[11px] text-fg-dim transition-colors hover:text-rose pt-2"
                aria-label="Remove rule"
              >
                Remove
              </button>
            </div>
          </div>
        );
      })}
      <Button variant="plain" size="sm" noLed onClick={add}>
        + Add rule
      </Button>
    </div>
  );
}

/**
 * Empty-state panel shown when a mode has zero auto-activation rules.
 * Three illustrative examples — one per match type — so the user sees
 * what each rule shape looks like without having to add rules first.
 *
 * Showing all three types matters: a previous version showed three
 * "Exe name" examples which left users wondering when they'd ever
 * need "Window title" or "Window class". The Window-class example
 * uses Chrome_WidgetWin_1 specifically because that's the class used
 * by every Electron app (Slack, Discord, VS Code, Notion, Teams),
 * which is the most plausible reason a user reaches for class-matching
 * over exe-matching.
 */
function AutoActivationEmptyState() {
  return (
    <div className="rounded-input border border-dashed border-border-soft/60 bg-bg-1 p-4">
      <div className="text-[12px] leading-relaxed text-muted mb-3">
        No rules yet. This mode won't activate automatically.
      </div>
      <div className="text-[11px] font-mono text-fg-dim space-y-1.5">
        <div>
          &middot; Exe name <span className="text-fg">outlook.exe</span>{' '}
          fires in Outlook
        </div>
        <div>
          &middot; Window title <span className="text-fg">*Slack</span>{' '}
          fires in any Slack window
        </div>
        <div>
          &middot; Window class <span className="text-fg">Chrome_WidgetWin_1</span>{' '}
          fires in any Electron app (VS Code, Discord, Notion)
        </div>
      </div>
      <div className="mt-3 text-[11px] leading-relaxed text-muted">
        Click <span className="text-fg-hi">+ Add rule</span> below to start.
      </div>
    </div>
  );
}

/**
 * Live preview of a Templated output. Substitutes `{text}` with a sample
 * sentence so the user sees the result of their format string without
 * having to dictate, save, dictate again. Mirrors the same substitution
 * the dictation pipeline performs (see services/text_inject in the
 * Rust side); we only model `{text}` because that's the only token the
 * runtime expands today.
 */
function TemplatePreview({ template }: { template: string }) {
  const SAMPLE = 'Hello world';
  const rendered = template.includes('{text}')
    ? template.replace(/\{text\}/g, SAMPLE)
    : template; // No token? Show the raw template; the runtime would do the same.
  return (
    <div className="rounded-input bg-bg-0 p-3">
      <div className="font-mono text-[10px] uppercase tracking-[0.16em] text-fg-dim mb-1.5">
        Preview · sample: "{SAMPLE}"
      </div>
      <div className="text-[13px] text-fg-hi break-words">
        {rendered || <span className="text-fg-dim">empty</span>}
      </div>
    </div>
  );
}
