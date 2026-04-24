import { Sparkles, Check, ExternalLink } from 'lucide-react';
import { open as openShell } from '@tauri-apps/plugin-shell';

import { Modal } from '@/components/common/Modal';
import { Button } from '@/components/common/Button';
import { useProPrice } from '@/hooks/useProPrice';
import { cn } from '@/lib/cn';

const PRICING_URL = 'https://utterbar.com/pricing';

/**
 * UpgradeModal — the in-app Pro CTA.
 *
 * Fires when a Free user clicks a Pro-gated action (e.g. "New mode",
 * "Duplicate mode"). Goals:
 *
 *   1. Explain what Pro is, specifically in the context the user triggered
 *      it from (passed in via `feature` + `description` — same props shape
 *      as `ProNudge` so they share copy patterns).
 *   2. Show price up front so the cost isn't a surprise later in checkout.
 *      Prices come from `useProPrice` which uses the Rust `get_pro_price`
 *      command; if that command can't reach the server it returns an
 *      offline fallback so the modal still shows a number.
 *   3. Drive a conversion with a single CTA that opens `utterbar.com/
 *      pricing` in the default browser via Tauri's shell plugin — we don't
 *      embed checkout here because Stripe's WebView2 coverage is uneven on
 *      older Windows 10 builds and the browser is the safer target.
 *
 * We intentionally use the same visual grammar as `ProNudge` (Sparkles icon,
 * one beat of cyan on "Pro") so the two upgrade surfaces feel like one
 * family of copy at different densities — a quiet nudge inline on the page
 * vs a full modal on intent.
 */
export function UpgradeModal({
  open,
  onClose,
  feature,
  description,
  bullets,
}: {
  open: boolean;
  onClose: () => void;
  /** Noun-phrase naming the blocked action, used in the headline. */
  feature: string;
  /** One sentence saying what the user was trying to do and why it's Pro. */
  description?: string;
  /** Optional short list of Pro benefits to reinforce the pitch. */
  bullets?: string[];
}) {
  const { price, loading } = useProPrice();

  return (
    <Modal open={open} onClose={onClose} widthClass="w-[min(460px,95vw)]">
      <div className="flex flex-col gap-5 px-7 pb-6 pt-7">
        <div className="flex items-start gap-3">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[10px] bg-[color-mix(in_oklab,var(--accent)_18%,var(--bg-2))]">
            <Sparkles className="h-[18px] w-[18px] text-accent" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="text-[17px] font-bold leading-tight text-fg-hi">
              {feature} is a <span className="text-accent">Pro</span> feature
            </div>
            {description && (
              <p className="mt-1.5 text-[13px] leading-relaxed text-muted">
                {description}
              </p>
            )}
          </div>
        </div>

        {bullets && bullets.length > 0 && (
          <ul className="flex flex-col gap-2 rounded-card bg-bg-2 px-4 py-3.5">
            {bullets.map((b) => (
              <li key={b} className="flex items-start gap-2.5 text-[13px] text-fg">
                <Check className="mt-[3px] h-3.5 w-3.5 shrink-0 text-accent" />
                <span>{b}</span>
              </li>
            ))}
          </ul>
        )}

        <div className="flex items-center justify-between gap-3 border-t border-border-soft/60 pt-4">
          <div className="flex items-baseline gap-2">
            <span
              className={cn(
                'font-bold text-fg-hi tabular-nums transition-opacity',
                loading && 'opacity-60',
              )}
              style={{ fontSize: '22px' }}
            >
              {price.display}
            </span>
            {price.period === 'one_time' && (
              <span className="text-[12px] font-medium uppercase tracking-[0.08em] text-muted">
                one-time
              </span>
            )}
            {price.period === 'monthly' && (
              <span className="text-[12px] font-medium uppercase tracking-[0.08em] text-muted">
                per month
              </span>
            )}
            {price.period === 'yearly' && (
              <span className="text-[12px] font-medium uppercase tracking-[0.08em] text-muted">
                per year
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" onClick={onClose}>
              Not now
            </Button>
            <Button
              variant="cta"
              size="sm"
              onClick={() => {
                void openShell(PRICING_URL).catch(() => undefined);
                // Close after opening — the user's attention is on the
                // browser now, and leaving the modal up means they come
                // back to a dead CTA after buying.
                onClose();
              }}
            >
              Upgrade
              <ExternalLink className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  );
}
