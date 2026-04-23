import { useEffect, useState } from 'react';
import { Copy, Pencil, Plus, Star, Trash2 } from 'lucide-react';

import { Button } from '@/components/common/Button';
import { PageHeader } from '@/components/common/PageHeader';
import { ProNudge } from '@/components/common/ProNudge';
import { ModeEditor, type EditableMode } from '@/components/modes/ModeEditor';
import { invoke } from '@/lib/ipc';
import { cn } from '@/lib/cn';
import { useLicence, isPro } from '@/state/licence';

type Mode = EditableMode;
type ModelInfo = { id: string; display_name: string };

/**
 * Modes — output-shaping presets.
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

  const licenceStatus = useLicence((s) => s.status);
  const pro = isPro(licenceStatus);

  return (
    <div className="mx-auto max-w-5xl p-8">
      <PageHeader
        title="Modes"
        subtitle="Shape output for different writing contexts. Each mode owns its own ASR, formatting, and post-processing."
        actions={
          <Button
            variant="cta"
            size="sm"
            onClick={createNew}
            disabled={!pro}
            title={pro ? undefined : 'Authoring custom modes is a Pro feature.'}
          >
            <Plus className="h-3.5 w-3.5" /> New mode
          </Button>
        }
      />

      {!pro && (
        <ProNudge
          feature="Custom modes"
          description="The built-ins are yours to use — Pro unlocks authoring your own: per-app auto-activation, your own templates, and custom post-processing rules."
          className="mt-5"
        />
      )}

      <ul className="mt-6 overflow-hidden rounded-card bg-bg-1">
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
              <span className="flex h-8 w-8 items-center justify-center rounded-input bg-bg-2 text-[13px] text-fg-hi">
                <ModeIcon id={m.id} />
              </span>
              <div className="flex-1 min-w-0">
                {/* Weight 700. font-semibold (600) is forbidden. */}
                <div className="text-[14px] font-bold text-fg-hi truncate">{m.name}</div>
                <div className="font-mono text-[11px] text-fg-dim truncate">
                  {m.is_builtin ? 'Built-in' : 'Custom'}
                  {m.asr_model_id ? ` · ${m.asr_model_id}` : ''}
                  {` · ${m.post_processing.kind}`}
                </div>
              </div>

              {active ? (
                /* Live chip per §6.3 .ub-chip--live — cyan LED + accent text
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

              <IconBtn label="Favourite" onClick={() => void toggleFavourite(m)}>
                <Star className={cn('h-4 w-4', m.is_favourite && 'fill-accent text-accent')} />
              </IconBtn>
              <IconBtn label="Edit" onClick={() => setEditing(m)}>
                <Pencil className="h-4 w-4" />
              </IconBtn>
              {pro && (
                <IconBtn label="Duplicate" onClick={() => void duplicate(m.id)}>
                  <Copy className="h-4 w-4" />
                </IconBtn>
              )}
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

function ModeIcon({ id }: { id: string }) {
  const g: Record<string, string> = {
    'voice-to-text': '🎙',
    message: '✉︎',
    email: '✦',
    note: '✎',
    meeting: '☰',
    super: '✴︎',
  };
  return <span aria-hidden>{g[id] ?? '·'}</span>;
}
