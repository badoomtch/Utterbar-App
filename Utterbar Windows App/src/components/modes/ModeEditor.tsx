import { useEffect, useState } from 'react';
import { X } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { Input, Textarea } from '@/components/common/Input';
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
            <Field label="Icon (lucide key)">
              <Input
                value={draft.icon}
                onChange={(e) => patch({ icon: e.target.value })}
                className="flex-1"
              />
            </Field>
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

          <Section title="Post-processing">
            {/* `llm` is deliberately absent in 0.6, the llama.cpp provider
                isn't wired yet, so the option would silently fall back to
                deterministic. Returns in 0.7 with the real LLM backend. */}
            <div className="flex flex-wrap gap-2">
              {(['none', 'deterministic'] as const).map((k) => (
                <button
                  key={k}
                  onClick={() =>
                    patchPost(
                      k === 'none'
                        ? { kind: 'none' }
                        : { kind: 'deterministic', template: '{text}' },
                    )
                  }
                  className={cn(
                    'ub-chip capitalize',
                    draft.post_processing.kind === k && 'ub-chip--active',
                  )}
                >
                  {k}
                </button>
              ))}
            </div>
            {draft.post_processing.kind === 'deterministic' && (
              <Field label="Template">
                <Input
                  value={draft.post_processing.template}
                  onChange={(e) =>
                    patchPost({ kind: 'deterministic', template: e.target.value })
                  }
                  className="flex-1"
                />
              </Field>
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
            description="Rules that make this mode active when a matching app is focused. Use * as a wildcard."
          >
            {draft.auto_activation.map((rule, idx) => (
              <div key={idx} className="flex gap-2">
                <select
                  value={rule.match}
                  onChange={(e) => {
                    const next = [...draft.auto_activation];
                    next[idx] = { ...rule, match: e.target.value as typeof rule.match };
                    patch({ auto_activation: next });
                  }}
                  className="ub-input !py-1 !text-[12px]"
                >
                  <option value="by_exe">exe</option>
                  <option value="by_title">title</option>
                  <option value="by_class">class</option>
                </select>
                <Input
                  value={rule.pattern}
                  onChange={(e) => {
                    const next = [...draft.auto_activation];
                    next[idx] = { ...rule, pattern: e.target.value };
                    patch({ auto_activation: next });
                  }}
                  placeholder="e.g. *outlook.exe"
                  className="flex-1 !py-1 !text-[12px]"
                />
                <button
                  onClick={() => {
                    const next = draft.auto_activation.filter((_, i) => i !== idx);
                    patch({ auto_activation: next });
                  }}
                  className="text-[11px] text-fg-dim transition-colors hover:text-rose"
                >
                  Remove
                </button>
              </div>
            ))}
            <Button
              variant="plain"
              size="sm"
              noLed
              onClick={() =>
                patch({
                  auto_activation: [
                    ...draft.auto_activation,
                    { match: 'by_exe', pattern: '' },
                  ],
                })
              }
            >
              + Add rule
            </Button>
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
