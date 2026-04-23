import { useEffect, useRef, type ReactNode } from 'react';
import { X } from 'lucide-react';
import { cn } from '@/lib/cn';

/**
 * Modal — the app's single overlay primitive.
 *
 * Visual grammar matches DESIGN_SYSTEM §7.2 (overlays):
 *   • Fixed full-viewport backdrop at `bg-black/70` + 4px blur, clickable to
 *     dismiss. The blur makes the backdrop read as "paused reality" rather
 *     than "a dark rectangle sat on top of the page".
 *   • Dialog is bg-1, rounded-block, overflow-hidden. No shadow — the blur
 *     backdrop + surface lift is enough; extra shadow reads as dropshadow-
 *     on-flat which our theme doesn't do anywhere else.
 *   • Close button is the only chrome, top-right, small ghost icon button.
 *     Per §4.7, we don't echo the title in the close affordance.
 *
 * The primitive was extracted from `ModeEditor`'s inlined shell — every new
 * modal we add (UpgradeModal, ConfirmDelete, etc.) uses this and stays
 * visually identical instead of each place re-inventing backdrop/padding.
 *
 * Accessibility:
 *   • `role="dialog"`, `aria-modal="true"` so screen readers announce it.
 *   • Escape closes. (Click-outside also closes, via the backdrop handler.)
 *   • Autofocus lands on the first focusable child via the browser's default
 *     focus behaviour — we don't trap focus because the app is single-
 *     window and there's no background content that benefits from being
 *     unreachable-by-tab for 500ms. If that changes (e.g. we open a modal
 *     over a long scrolling page the user mustn't accidentally interact
 *     with), layer a `focus-trap-react` over this primitive then; don't
 *     build the trap in here.
 */
export function Modal({
  open,
  onClose,
  title,
  children,
  widthClass = 'w-[min(520px,95vw)]',
  maxHeightClass = 'max-h-[min(640px,90vh)]',
}: {
  open: boolean;
  onClose: () => void;
  title?: ReactNode;
  children: ReactNode;
  /** Override the default width when a modal wants to be narrower/wider. */
  widthClass?: string;
  /** Override the default max-height. */
  maxHeightClass?: string;
}) {
  const panelRef = useRef<HTMLDivElement>(null);

  // Escape-to-close. Scoped to window so we catch the keypress wherever
  // focus is — including the backdrop itself, which gets the initial
  // focus after mount.
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6 backdrop-blur-sm"
      // Click-outside closes. Guard against accidental close when the user
      // mousedowns inside the panel and drifts out during a drag-select —
      // only the backdrop itself counts as an outside click.
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={panelRef}
        className={cn(
          'flex flex-col overflow-hidden rounded-block bg-bg-1',
          widthClass,
          maxHeightClass,
        )}
      >
        {title != null && (
          <div className="flex items-center justify-between gap-3 border-b border-border-soft/60 px-6 py-4">
            <div className="text-[15px] font-bold text-fg-hi">{title}</div>
            <button
              type="button"
              aria-label="Close"
              onClick={onClose}
              className="-mr-2 rounded-[8px] p-1 text-fg-dim transition-colors hover:bg-bg-2 hover:text-fg-hi"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        )}
        {children}
      </div>
    </div>
  );
}
