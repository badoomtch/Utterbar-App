import { ShieldCheck } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { cn } from '@/lib/cn';

/**
 * ManagedHint — inline "Disabled by your administrator" badge.
 *
 * Rendered next to (or above) a control that has been pinned by group
 * policy. The control itself is disabled and unchanged in shape; this
 * badge just explains why the user can't move it. Backend enforcement
 * is the load-bearing part — see `services/policy.rs` and the gates
 * in `commands/history.rs`, `commands/dictation.rs` etc. — but
 * showing the hint up-front beats letting a click bounce back as a
 * permission error.
 *
 * Variants:
 *   • `inline` — small one-liner suited to sit in a Card row alongside
 *     a disabled toggle.
 *   • `block` — slightly larger, used at the top of a settings page
 *     when several controls are pinned at once.
 */
export function ManagedHint({
  children,
  variant = 'inline',
  className,
}: {
  children?: React.ReactNode;
  variant?: 'inline' | 'block';
  className?: string;
}) {
  const { t } = useTranslation('common');
  // Caller-provided `children` (a JSX node) wins so callers that need
  // a fully custom message keep their existing shape. The default
  // fallback is the only frontend-owned literal here, and that's what
  // the localisation routes through.
  const text = children ?? t('chrome.managed_hint.default');
  if (variant === 'block') {
    return (
      <div
        className={cn(
          'flex items-center gap-2 rounded-md border border-bg-3 bg-bg-1 px-3 py-2 text-[12px] text-muted',
          className,
        )}
        role="note"
      >
        <ShieldCheck className="h-3.5 w-3.5 shrink-0" aria-hidden />
        <span>{text}</span>
      </div>
    );
  }
  return (
    <span
      className={cn('inline-flex items-center gap-1 text-[11px] text-muted', className)}
      role="note"
    >
      <ShieldCheck className="h-3 w-3 shrink-0" aria-hidden />
      {text}
    </span>
  );
}
