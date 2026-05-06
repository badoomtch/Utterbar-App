import { useEffect, useState } from 'react';
import { AlertTriangle } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Kbd } from '@/components/common/Kbd';
import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { invoke } from '@/lib/ipc';
import { cn } from '@/lib/cn';

type ShortcutIdEnum =
  | 'toggle_recording'
  | 'push_to_talk'
  | 'hold_to_talk'
  | 'cancel_recording'
  | 'cycle_mode'
  | 'open_settings'
  | 'open_history';

type Slot = {
  id: ShortcutIdEnum;
  label: string;
  description: string;
  current: { accelerator: string; label: string; trigger_style: string } | null;
};

type Conflict = {
  slot_a: ShortcutIdEnum;
  slot_b: ShortcutIdEnum;
  accelerator: string;
};

export function ShortcutsSettings() {
  const { t } = useTranslation('settings');
  const [slots, setSlots] = useState<Slot[]>([]);
  const [conflicts, setConflicts] = useState<Conflict[]>([]);
  const [capturing, setCapturing] = useState<ShortcutIdEnum | null>(null);

  const reload = async () => {
    const [s, c] = await Promise.all([
      invoke<Slot[]>('list_shortcuts'),
      invoke<Conflict[]>('detect_conflicts'),
    ]);
    setSlots(s);
    setConflicts(c);
  };

  useEffect(() => {
    void reload();
  }, []);

  const startCapture = (id: ShortcutIdEnum) => {
    setCapturing(id);
  };

  const onKeyDown = async (ev: React.KeyboardEvent, id: ShortcutIdEnum) => {
    if (capturing !== id) return;
    ev.preventDefault();
    ev.stopPropagation();
    if (ev.key === 'Escape') {
      setCapturing(null);
      return;
    }
    const parts: string[] = [];
    if (ev.ctrlKey) parts.push('Ctrl');
    if (ev.altKey) parts.push('Alt');
    if (ev.shiftKey) parts.push('Shift');
    if (ev.metaKey) parts.push('Super');
    const key = ev.key;
    // Require a non-modifier key before accepting.
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(key)) return;
    const display =
      key.length === 1 ? key.toUpperCase() : key === ' ' ? 'Space' : key;
    parts.push(display);
    const accelerator = parts.join('+');
    const trigger_style =
      id === 'push_to_talk'
        ? 'push_to_talk'
        : id === 'hold_to_talk'
          ? 'hold_to_talk'
          : 'toggle';
    await invoke('bind_shortcut', {
      id,
      shortcut: { accelerator, label: accelerator, trigger_style },
    });
    setCapturing(null);
    await reload();
  };

  const clear = async (id: ShortcutIdEnum) => {
    await invoke('unbind_shortcut', { id });
    await reload();
  };

  const conflictFor = (id: ShortcutIdEnum) =>
    conflicts.find((c) => c.slot_a === id || c.slot_b === id);

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader
        title={t('shortcuts.title')}
        subtitle={t('shortcuts.subtitle')}
      />
      <Card className="mt-6">
        {slots.map((slot) => {
          const conflict = conflictFor(slot.id);
          const isCapturing = capturing === slot.id;
          return (
            // slot.label and slot.description are emitted by the Rust
            // `list_shortcuts` command and stay verbatim. Backend-owned
            // prose is deferred to a future error/event-codes branch
            // and not catalogued here.
            <Row key={slot.id} label={slot.label} description={slot.description}>
              <div className="flex items-center gap-2">
                {conflict && (
                  <span
                    title={t('shortcuts.conflict.tooltip', {
                      accelerator: conflict.accelerator,
                    })}
                    className="text-amber"
                  >
                    <AlertTriangle className="h-4 w-4" />
                  </span>
                )}
                {/* The capture target is a .ub-kbd styled button so the
                    accelerator reads with the same chip geometry as any other
                    keybind display in the app (§6.2). Captures pulse the
                    accent colour instead of glowing the border. */}
                <button
                  onKeyDown={(e) => void onKeyDown(e, slot.id)}
                  onBlur={() => setCapturing(null)}
                  onClick={() => startCapture(slot.id)}
                  className={cn(
                    'ub-kbd outline-none transition-colors',
                    isCapturing && 'text-accent',
                  )}
                >
                  {isCapturing
                    ? t('shortcuts.capture.press_keys')
                    : slot.current?.accelerator ?? t('shortcuts.capture.unbound')}
                </button>
                {slot.current && !isCapturing && (
                  <button
                    onClick={() => void clear(slot.id)}
                    className="text-[11px] text-fg-dim transition-colors hover:text-rose"
                  >
                    {t('shortcuts.buttons.clear')}
                  </button>
                )}
              </div>
            </Row>
          );
        })}
      </Card>
      <p className="mt-3 text-[11px] text-muted inline-flex items-center gap-1">
        {t('shortcuts.tip.prefix')} <Kbd>Esc</Kbd> {t('shortcuts.tip.suffix')}
      </p>
    </div>
  );
}
