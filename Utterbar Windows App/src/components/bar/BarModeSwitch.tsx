import { useEffect } from 'react';

import { ModeIcon } from '@/components/modes/ModeIcon';
import { useModes, type Mode } from '@/state/modes';
import { cn } from '@/lib/cn';

/**
 * Floating-bar mode quick switcher.
 *
 * Renders up to {@link MAX_VISIBLE} icon-only chips, one per mode the
 * user is most likely to want one tap away. Selection rule:
 *
 *   1. Favourites first (sorted by sort_order)
 *   2. Built-ins next, in sort_order
 *   3. Anything else, in sort_order
 *   4. Truncate to MAX_VISIBLE
 *
 * The cap exists because the floating pill is ~460 px wide expanded and
 * already carries the wordmark + state + record + utility cluster.
 * Showing every mode would either overflow the pill or shrink the
 * existing controls below their hit-target floor.
 *
 * Visual:
 *   - 28×28 chip, rounded-[8px], same surface family as BarButton's
 *     neutral tone.
 *   - Active mode: accent gradient, same treatment as BarButton tone
 *     "accent". Only one chip is ever active at a time, which keeps
 *     the "cyan budget" intact.
 *   - Hover: scale 1.08 with the same overshoot cubic-bezier.
 *
 * Switching mid-recording is intentionally not supported; the bar
 * caller only renders this component in the non-listening expanded
 * state. Switching mid-record would be ambiguous (Meeting mode forces
 * a different audio source than dictation modes) and not something
 * the bar can resolve cleanly.
 */
const MAX_VISIBLE = 5;

export function BarModeSwitch({ compact = false }: { compact?: boolean }) {
  const list = useModes((s) => s.list);
  const activeId = useModes((s) => s.activeId);
  const load = useModes((s) => s.load);
  const setActive = useModes((s) => s.setActive);

  // Lazy-load on first mount. Cheap (one IPC each for list + active),
  // and the floating bar mounts once per session.
  useEffect(() => {
    if (list.length === 0) {
      void load();
    }
  }, [list.length, load]);

  const visible = pickVisibleModes(list, MAX_VISIBLE);
  if (visible.length === 0) return null;

  const sizePx = compact ? 24 : 28;
  const radiusClass = compact ? 'rounded-[7px]' : 'rounded-[8px]';
  const iconPx = compact ? 13 : 15;

  return (
    <div className="flex items-center gap-1" aria-label="Mode quick switch">
      {visible.map((m) => {
        const active = m.id === activeId;
        return (
          <button
            key={m.id}
            onClick={() => {
              if (m.id !== activeId) void setActive(m.id);
            }}
            aria-label={`Switch to ${m.name} mode`}
            aria-pressed={active}
            title={m.name}
            style={{
              width: sizePx,
              height: sizePx,
              ...(active
                ? {
                    background:
                      'linear-gradient(180deg, var(--accent-2), var(--accent))',
                    color: '#06121a',
                    boxShadow:
                      '0 2px 10px color-mix(in oklab, var(--accent) 40%, transparent)',
                  }
                : {
                    background: 'var(--pill-surface)',
                    color: 'var(--pill-fg-muted)',
                  }),
            }}
            className={cn(
              'no-drag flex items-center justify-center transition-[transform,box-shadow,background,color] duration-150',
              '[transition-timing-function:cubic-bezier(0.34,1.56,0.64,1)]',
              radiusClass,
              'hover:scale-[1.08] active:scale-[0.86]',
              !active && 'hover:bg-[var(--pill-surface-hi)] hover:text-[var(--pill-ink-on-hi)]',
              active && 'hover:brightness-110',
            )}
          >
            <ModeIcon name={m.icon} size={iconPx} />
          </button>
        );
      })}
    </div>
  );
}

function pickVisibleModes(list: Mode[], cap: number): Mode[] {
  const sorted = [...list].sort((a, b) => a.sort_order - b.sort_order);
  const favourites = sorted.filter((m) => m.is_favourite);
  const builtins = sorted.filter((m) => !m.is_favourite && m.is_builtin);
  const custom = sorted.filter((m) => !m.is_favourite && !m.is_builtin);
  return [...favourites, ...builtins, ...custom].slice(0, cap);
}
