import { useEffect, useState } from 'react';
import { Copy, Pencil, Plus, Star, Trash2 } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { PageHeader } from '@/components/common/PageHeader';
import { ModeEditor, type EditableMode } from '@/components/modes/ModeEditor';
import { ModeIcon } from '@/components/modes/ModeIcon';
import { invoke } from '@/lib/ipc';
import { cn } from '@/lib/cn';

type Mode = EditableMode;
// Local narrow shape over the Rust ModelInfo. Only the fields the
// editor actually consumes; `installed` lets the meeting-mode copy
// tell the user whether Whisper Large is ready to auto-route to.
type ModelInfo = { id: string; display_name: string; installed: boolean };

/**
 * Modes, output-shaping presets.
 *
 * Per DESIGN_SYSTEM.md §8.2 the active mode row wears the canonical
 * ub-item-active treatment (2px cyan rail + bg-2 lift). The "Set active"
 * button on the active row collapses to an .ub-chip--live pill so we
 * don't have a full-cyan button sitting next to a full-cyan rail.
 */
export function ModesRoute() {
  const [modes, setModes] = useState<Mode[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [editing, setEditing] = useState<Mode | null>(null);
  const reload = async () => {
    const [list, active, m] = await Promise.all([
      invoke<Mode[]>('list_modes'),
      invoke<Mode | null>('get_active_mode').catch(() => null),
      invoke<ModelInfo[]>('list_available_models').catch(() => []),
    ]);
    setModes(list);
    setActiveId(active?.id ?? null);
    setModels(m);
  };

  useEffect(() => {
    void reload();
  }, []);

  const setActive = async (id: string) => {
    await invoke('set_active_mode', { id });
    setActiveId(id);
  };

  const duplicate = async (id: string) => {
    await invoke('duplicate_mode', { id });
    await reload();
  };

  const remove = async (id: string) => {
    if (!confirm('Delete this mode? Built-in modes cannot be deleted.')) return;
    await invoke('delete_mode', { id });
    await reload();
  };

  const createNew = () => {
    const now = Date.now();
    setEditing({
      id: crypto.randomUUID().replace(/-/g, '').slice(0, 26),
      name: 'New mode',
      icon: 'sparkles',
      is_builtin: false,
      is_favourite: false,
      sort_order: modes.length,
      asr_model_id: null,
      target_language: null,
      post_processing: { kind: 'none' },
      context_rules: { use_clipboard: false, use_selection: false, use_active_app: true },
      auto_activation: [],
      shortcut_override: null,
      formatting: {
        capitalisation: 'sentence',
        strip_fillers: true,
        auto_punctuation: true,
        bullet_style: 'none',
        preserve_linebreaks: false,
      },
      created_at: now,
      updated_at: now,
    });
  };

  const toggleFavourite = async (m: Mode) => {
    await invoke('upsert_mode', { mode: { ...m, is_favourite: !m.is_favourite } });
    await reload();
  };

  return (
    <div className="mx-auto max-w-5xl p-8">
      <PageHeader
        title="Modes"
        subtitle="Shape output for different writing contexts. Each mode owns its own ASR, formatting, and post-processing."
        actions={
          <Button variant="cta" size="sm" onClick={createNew}>
            <Plus className="h-3.5 w-3.5" /> New mode
          </Button>
        }
      />

      {/* Hint about the favourite control. The star next to each mode
          is what decides whether the mode appears as a quick-switch
          chip in the floating bar. New installs get every built-in
          favourited so the bar isn't empty out of the box; from there
          the user prunes or extends as they like. Surfacing this
          here means "where do I add/remove modes from the bar?" has
          a visible answer next to the action it refers to. */}
      <p className="mt-5 text-[12px] leading-relaxed text-muted">
        Tap the star next to a mode to show or hide it in the floating bar's quick
        switcher. Favourited modes appear as one-tap chips next to the Record button.
      </p>

      <ul className="mt-3 overflow-hidden rounded-card bg-bg-1">
        {modes.map((m) => {
          const active = m.id === activeId;
          return (
            <li
              key={m.id}
              className={cn(
                'group relative flex items-center gap-4 px-5 py-4',
                'border-t border-border-soft/50 first:border-t-0',
                active ? 'bg-bg-2' : '',
              )}
            >
              <span
                className={cn(
                  'absolute left-0 top-3 bottom-3 w-[2px] rounded-r-full transition-opacity',
                  active ? 'bg-accent opacity-100' : 'opacity-0',
                )}
                aria-hidden
              />
              <span className="flex h-8 w-8 items-center justify-center rounded-input bg-bg-2 text-fg-hi">
                <ModeIcon name={m.icon} />
              </span>
              <div className="flex-1 min-w-0">
                {/* Weight 700. font-semibold (600) is forbidden. */}
                <div className="text-[14px] font-bold text-fg-hi truncate">{m.name}</div>
                <div className="font-mono text-[11px] text-fg-dim truncate">
                  {m.is_builtin ? 'Built-in' : 'Custom'}
                  {m.asr_model_id ? ` · ${m.asr_model_id}` : ''}
                  {` · ${outputStyleLabel(m.post_processing.kind)}`}
                </div>
              </div>

              {active ? (
                /* Live chip per §6.3 .ub-chip--live, cyan LED + accent text
                   on a cyan-tinted background. No button semantics; clicking
                   the row while it's already active is a no-op. */
                <span className="ub-chip ub-chip--live">
                  <span className="ub-led" aria-hidden />
                  Active
                </span>
              ) : (
                <Button variant="ghost" size="sm" noLed onClick={() => void setActive(m.id)}>
                  Set active
                </Button>
              )}

              <IconBtn
                label={
                  m.is_favourite
                    ? 'Remove from floating bar'
                    : 'Show in floating bar'
                }
                onClick={() => void toggleFavourite(m)}
              >
                <Star className={cn('h-4 w-4', m.is_favourite && 'fill-accent text-accent')} />
              </IconBtn>
              <IconBtn label="Edit" onClick={() => setEditing(m)}>
                <Pencil className="h-4 w-4" />
              </IconBtn>
              {/* Duplicate is shown for Free users too; clicking opens the
                  upgrade modal (handled in the `duplicate` function) rather
                  than disabling a visible action. This follows the same
                  "click-to-learn" pattern as the New mode button. */}
              <IconBtn label="Duplicate" onClick={() => void duplicate(m.id)}>
                <Copy className="h-4 w-4" />
              </IconBtn>
              {!m.is_builtin && (
                <IconBtn label="Delete" onClick={() => void remove(m.id)}>
                  <Trash2 className="h-4 w-4" />
                </IconBtn>
              )}
            </li>
          );
        })}
      </ul>

      {editing && (
        <ModeEditor
          mode={editing}
          models={models}
          onClose={() => setEditing(null)}
          onSaved={() => void reload()}
        />
      )}
    </div>
  );
}

function IconBtn({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      title={label}
      aria-label={label}
      className="text-fg-dim transition-colors hover:text-fg-hi"
    >
      {children}
    </button>
  );
}

/**
 * Friendly label for a mode's post-processing kind, shown in the
 * metadata strip on each row. Avoids leaking internal vocabulary
 * ('none', 'deterministic') into the user-facing list.
 *
 * Kept in sync with OUTPUT_STYLES in components/modes/ModeEditor.tsx —
 * if you add a third style there, add the label here too.
 */
function outputStyleLabel(kind: 'none' | 'deterministic' | 'llm'): string {
  switch (kind) {
    case 'none':
      return 'Plain';
    case 'deterministic':
      return 'Templated';
    case 'llm':
      return 'AI rewrite';
  }
}

// ModeIcon now lives in components/modes/ModeIcon.tsx — a real lucide
// renderer instead of the emoji-fallback lookup that used to live here.
// Imported at the top of this file.
