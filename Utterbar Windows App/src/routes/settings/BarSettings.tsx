import { LocateFixed } from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { invoke } from '@/lib/ipc';
import { useSettings } from '@/state/settings';
import { cn } from '@/lib/cn';

const BAR_SIZES = ['compact', 'standard'] as const;

const VISIBILITY_OPTIONS: Array<{
  id: 'always' | 'while_recording' | 'hidden';
  label: string;
  desc: string;
}> = [
  {
    id: 'always',
    label: 'Always visible',
    desc: 'The Utterbar sits at the top of your screen whenever the app is running. Default behaviour — easiest to spot, easiest to click.',
  },
  {
    id: 'while_recording',
    label: 'Only while recording',
    desc: 'The bar stays hidden until you trigger dictation, then pops in so you can see the waveform and stop / cancel. Minimal on-screen footprint.',
  },
  {
    id: 'hidden',
    label: 'Never show',
    desc: 'The Utterbar never appears. Drive everything from your keyboard shortcut and the tray icon. The fastest, quietest setup.',
  },
];

export function BarSettings() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  if (!s) return null;

  const g = s.general as Record<string, unknown>;
  const update = (key: string, value: unknown) =>
    patch({ general: { ...s.general, [key]: value } });

  const visibility = (g.bar_visibility as string) ?? 'always';
  const size = (g.bar_size as string) ?? 'standard';
  const outline = (g.bar_outline as boolean | undefined) ?? true;

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader
        title="Utterbar"
        subtitle="The floating pill that lives on top of your desktop. Tune what it looks like and when it shows up."
      />
      <div className="mt-6 space-y-4">
        <Card title="Visibility">
          {/* Radio tiles — surface elevation + rail for the active one,
              matching the capture-source picker on Audio settings. */}
          <ul className="space-y-2">
            {VISIBILITY_OPTIONS.map((opt) => {
              const selected = visibility === opt.id;
              return (
                <li key={opt.id}>
                  <label
                    className={cn(
                      'group relative flex cursor-pointer items-start gap-3 rounded-input p-4 transition-colors',
                      selected ? 'bg-bg-2' : 'bg-bg-2/40 hover:bg-bg-2/70',
                    )}
                  >
                    <span
                      className={cn(
                        'absolute left-0 top-3 bottom-3 w-[2px] rounded-r-full transition-opacity',
                        selected ? 'bg-accent opacity-100' : 'opacity-0',
                      )}
                      aria-hidden
                    />
                    <input
                      type="radio"
                      checked={selected}
                      onChange={() => void update('bar_visibility', opt.id)}
                      className="mt-1 accent-[var(--accent)]"
                    />
                    <div className="pl-1">
                      <div className="text-[13px] font-bold text-fg-hi">{opt.label}</div>
                      <div className="mt-0.5 text-[12px] leading-relaxed text-muted">
                        {opt.desc}
                      </div>
                    </div>
                  </label>
                </li>
              );
            })}
          </ul>
        </Card>

        <Card title="Appearance">
          <Row
            label="Size"
            description="Compact shrinks the pill footprint. Standard gives its controls more breathing room."
          >
            <div className="flex gap-2">
              {BAR_SIZES.map((b) => (
                <button
                  key={b}
                  onClick={() => void update('bar_size', b)}
                  className={cn('ub-chip capitalize', size === b && 'ub-chip--active')}
                >
                  {b}
                </button>
              ))}
            </div>
          </Row>
          <Row
            label="Accent outline"
            description="Adds a thin accent-coloured border around the pill so it's easier to pick out against bright wallpapers and light backgrounds."
          >
            <Toggle checked={outline} onChange={(v) => void update('bar_outline', v)} />
          </Row>
        </Card>

        <Card title="Position">
          <Row
            label="Reset position"
            description="Snap the Utterbar back to the top-centre of your current monitor. Use this if the pill has ended up behind the taskbar, off-screen, or on a monitor you've since unplugged. Also available from the tray icon's right-click menu."
          >
            <button
              onClick={() => void invoke('reset_floating_position').catch(() => undefined)}
              className="ub-btn ub-btn-plain !py-[0.4rem] !text-[12px] inline-flex items-center gap-1.5"
            >
              <LocateFixed className="h-3.5 w-3.5" />
              Reset position
            </button>
          </Row>
        </Card>
      </div>
    </div>
  );
}
