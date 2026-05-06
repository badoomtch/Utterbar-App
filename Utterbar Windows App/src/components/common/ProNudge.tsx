import { Sparkles } from 'lucide-react';
import { open as openShell } from '@tauri-apps/plugin-shell';
import { useTranslation } from 'react-i18next';

import { cn } from '@/lib/cn';
import { useLicence } from '@/state/licence';

const PRICING_URL = 'https://utterbar.com/pricing';

/**
 * ProNudge — inline "this is a Pro feature" banner.
 *
 * The contextual, non-popup form factor the product brief calls for:
 * when a user lands on a Pro-gated surface we render this banner inline
 * above/near the gated control instead of blocking the page with a modal
 * or toast. Users on Pro never see it (returns null), so the call sites
 * don't need to guard — drop one in wherever a Pro-only feature lives.
 *
 * Visual grammar:
 *   • bg-2 surface with the 2px cyan left-rail (.ub-card-rail) — this IS
 *     the "featured / selected / important" card treatment per §4.5.
 *   • Sparkles icon in accent-cyan + one beat of cyan text for "Pro".
 *     Keeps us inside the `<3 cyan moments per viewport` budget because
 *     the icon+word read as one object.
 *   • "Learn more →" button opens utterbar.com/pricing in the system
 *     browser (shell plugin, same pattern as the Licence page).
 *
 * Props:
 *   • `feature` — short noun phrase naming the gated capability, used
 *     verbatim in the headline ("Custom vocabulary is a Pro feature.").
 *   • `description` — one sentence explaining what the Pro tier unlocks
 *     here specifically. Keep it editorial, not feature-listy.
 *   • `dense` — tighter padding/text for in-list nudges (e.g. inside a
 *     Card row). Default reads comfortably at the page top.
 */
export function ProNudge({
  feature,
  description,
  dense = false,
  className,
}: {
  feature: string;
  description?: string;
  dense?: boolean;
  className?: string;
}) {
  const { t } = useTranslation('common');
  const status = useLicence((s) => s.status);
  const loaded = useLicence((s) => s.loaded);
  // Don't render until the licence store has resolved — avoids flashing the
  // nudge for one frame at launch on Pro devices.
  if (!loaded) return null;
  if (status?.tier === 'pro') return null;

  return (
    <div
      className={cn(
        'ub-card-rail rounded-card bg-bg-2',
        dense ? 'px-4 py-3' : 'px-5 py-4',
        'flex items-start gap-3',
        className,
      )}
      role="note"
    >
      <Sparkles
        className={cn('shrink-0 text-accent', dense ? 'mt-0.5 h-4 w-4' : 'mt-0.5 h-[18px] w-[18px]')}
        aria-hidden
      />
      <div className="flex-1 min-w-0">
        {/*
          Same prefix/pro/suffix shape as UpgradeModal: the framing wraps
          "Pro" in an accent span, so we split the surrounding sentence
          rather than restructuring the JSX. See UpgradeModal.tsx for the
          deferral note on locales with different word order.
        */}
        <div
          className={cn(
            'font-medium text-fg-hi',
            dense ? 'text-[12px]' : 'text-[13px]',
          )}
        >
          {feature} {t('chrome.pro_nudge.feature_prefix')}{' '}
          <span className="text-accent">{t('chrome.pro_nudge.pro_word')}</span>{' '}
          {t('chrome.pro_nudge.feature_suffix')}
        </div>
        {description && (
          <p
            className={cn(
              'mt-1 leading-relaxed text-muted',
              dense ? 'text-[11px]' : 'text-[12px]',
            )}
          >
            {description}
          </p>
        )}
      </div>
      <button
        type="button"
        onClick={() => void openShell(PRICING_URL).catch(() => undefined)}
        className={cn(
          'ub-btn ub-btn-plain shrink-0',
          dense ? '!py-[0.3rem] !text-[11px]' : '!py-[0.4rem] !text-[12px]',
        )}
      >
        {t('chrome.pro_nudge.learn_more')}
      </button>
    </div>
  );
}
