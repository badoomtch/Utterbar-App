import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { ChevronDown, Check } from 'lucide-react';

import { cn } from '@/lib/cn';

/**
 * Themed Select, replacing the native `<select>` so the list doesn't open
 * as an unthemed white Windows listbox against Utterbar's palette. Keyboard
 * behaviour mirrors native: Enter/Space opens, arrows move, Escape closes.
 *
 * Per DESIGN_SYSTEM.md:
 *   • Trigger sits on bg-2 (§6.1-adjacent — the spec's form controls use
 *     bg-0 for text inputs; dropdowns read better elevated, so bg-2).
 *   • No border by default — a hairline appears on hover to hint clickability.
 *   • Menu floats on bg-2 with a very quiet border; no drop shadow.
 *   • Option highlights are TONAL (bg-2 → slight lift), never cyan fills.
 *     The cyan Check icon is the only accent moment inside the menu.
 */
export type SelectOption<T extends string> = {
  value: T;
  label: string;
  caption?: string;
  disabled?: boolean;
};

export function Select<T extends string>({
  value,
  options,
  onChange,
  placeholder,
  className,
  buttonClassName,
  disabled,
  minWidth,
}: {
  value: T | null | undefined;
  options: SelectOption<T>[];
  onChange: (next: T) => void;
  placeholder?: string;
  className?: string;
  buttonClassName?: string;
  disabled?: boolean;
  /** Tailwind min-width class for the button + menu, e.g. `min-w-[18rem]`. */
  minWidth?: string;
}) {
  const [open, setOpen] = useState(false);
  const [activeIdx, setActiveIdx] = useState<number>(() => {
    const i = options.findIndex((o) => o.value === value);
    return i >= 0 ? i : 0;
  });
  const btnRef = useRef<HTMLButtonElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);

  const selected = options.find((o) => o.value === value);

  // Close on outside click.
  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      const t = e.target as Node;
      if (btnRef.current?.contains(t) || menuRef.current?.contains(t)) return;
      setOpen(false);
    };
    window.addEventListener('mousedown', onDown);
    return () => window.removeEventListener('mousedown', onDown);
  }, [open]);

  // Reset active row to the current selection whenever the menu opens,
  // so arrow-down starts from where the user is, not from the top.
  useLayoutEffect(() => {
    if (!open) return;
    const i = options.findIndex((o) => o.value === value);
    if (i >= 0) setActiveIdx(i);
  }, [open, options, value]);

  const moveActive = (delta: number) => {
    if (options.length === 0) return;
    let i = activeIdx;
    for (let step = 0; step < options.length; step++) {
      i = (i + delta + options.length) % options.length;
      if (!options[i].disabled) break;
    }
    setActiveIdx(i);
  };

  const pick = (idx: number) => {
    const o = options[idx];
    if (!o || o.disabled) return;
    onChange(o.value);
    setOpen(false);
    btnRef.current?.focus();
  };

  return (
    <div className={cn('relative inline-block', minWidth, className)}>
      <button
        ref={btnRef}
        type="button"
        disabled={disabled}
        onClick={() => !disabled && setOpen((v) => !v)}
        onKeyDown={(e) => {
          if (disabled) return;
          if (e.key === 'ArrowDown' || e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            setOpen(true);
          } else if (e.key === 'Escape') {
            setOpen(false);
          }
        }}
        aria-haspopup="listbox"
        aria-expanded={open}
        className={cn(
          'flex w-full items-center justify-between gap-2 rounded-input border px-3 py-1.5 text-[13px]',
          'bg-bg-2 text-fg-hi transition-colors duration-180',
          open
            ? 'border-accent/60'
            : 'border-transparent hover:border-border-strong',
          disabled && 'cursor-not-allowed opacity-60',
          buttonClassName,
        )}
      >
        <span className="truncate text-left">
          {selected ? (
            selected.label
          ) : (
            <span className="text-fg-dim">{placeholder ?? 'Select…'}</span>
          )}
        </span>
        <ChevronDown
          className={cn(
            'h-3.5 w-3.5 shrink-0 text-muted transition-transform duration-180',
            open && 'rotate-180',
          )}
        />
      </button>

      {open && (
        <div
          ref={menuRef}
          role="listbox"
          tabIndex={-1}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown') { e.preventDefault(); moveActive(1); }
            else if (e.key === 'ArrowUp') { e.preventDefault(); moveActive(-1); }
            else if (e.key === 'Enter') { e.preventDefault(); pick(activeIdx); }
            else if (e.key === 'Escape') { e.preventDefault(); setOpen(false); btnRef.current?.focus(); }
          }}
          className={cn(
            // Menu sits on bg-2 with a very quiet border. No shadow; depth
            // comes from the tonal lift, not from a drop-shadow. Radius
            // matches card-minor (14px lg alias) so it reads as "the same
            // family as a card, smaller".
            //
            // Width: at LEAST as wide as the trigger button (so it never
            // looks detached), but allowed to grow to fit the longest
            // option label up to the viewport width. The previous version
            // pinned to `w-full` of the trigger, which clipped option
            // labels to single characters when the trigger lived inside
            // a tight filter chip (History route's Mode / App pills).
            // `whitespace-nowrap` on the wrapper means each option lays
            // out at its natural width, the menu grows to enclose them.
            'absolute z-50 mt-1.5 max-h-72 overflow-auto rounded-lg',
            'min-w-full max-w-[min(28rem,calc(100vw-2rem))] whitespace-nowrap',
            'border border-border-soft bg-bg-2 p-1',
          )}
        >
          {options.length === 0 && (
            <div className="px-3 py-2 text-xs text-fg-dim">No options</div>
          )}
          {options.map((o, i) => {
            const isActive = i === activeIdx;
            const isSelected = o.value === value;
            return (
              <button
                key={o.value}
                type="button"
                role="option"
                aria-selected={isSelected}
                disabled={o.disabled}
                onMouseEnter={() => setActiveIdx(i)}
                onClick={() => pick(i)}
                className={cn(
                  'flex w-full items-start gap-2 rounded-md px-2.5 py-1.5 text-left text-[13px] transition-colors duration-150',
                  o.disabled && 'cursor-not-allowed opacity-50',
                  isActive && !o.disabled
                    ? 'bg-bg-1 text-fg-hi'
                    : 'text-fg',
                )}
              >
                <Check
                  className={cn(
                    'mt-0.5 h-3.5 w-3.5 shrink-0 text-accent',
                    isSelected ? 'opacity-100' : 'opacity-0',
                  )}
                />
                <div className="min-w-0 flex-1">
                  <div>{o.label}</div>
                  {o.caption && (
                    <div className="text-[11px] text-fg-dim">{o.caption}</div>
                  )}
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
