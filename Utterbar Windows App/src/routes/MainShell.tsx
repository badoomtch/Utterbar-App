import { forwardRef, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { NavLink, Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom';
import { getVersion } from '@tauri-apps/api/app';
import { open as openShell } from '@tauri-apps/plugin-shell';
import { useTranslation } from 'react-i18next';

import { listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { Led } from '@/components/common/Led';
import { useLicence } from '@/state/licence';
import {
  BarChart3,
  FileAudio,
  History as HistoryIcon,
  Layers,
  Search,
  Shapes,
  Users,
  X,
} from 'lucide-react';

import { UtterbarLogoMark } from '@/components/bar/UtterbarLogoMark';
import { UtterbarWordmark } from '@/components/bar/UtterbarWordmark';
import { PostUpdateToast, UpdateToast } from '@/components/common/UpdateToast';
import { cn } from '@/lib/cn';
import { filterSettingsByQuery, settingsSearchMatchSummary } from '@/routes/settings_search';
import { buildSettingsNav } from '@/routes/settings_nav';
import { Dashboard } from '@/routes/Dashboard';
import { HistoryRoute } from '@/routes/HistoryRoute';
import { MeetingsRoute } from '@/routes/MeetingsRoute';
import { ModesRoute } from '@/routes/ModesRoute';
import { VocabularyRoute } from '@/routes/VocabularyRoute';
import { FilesRoute } from '@/routes/FilesRoute';
import { AccessibilitySettings } from '@/routes/settings/AccessibilitySettings';
import { GeneralSettings } from '@/routes/settings/GeneralSettings';
import { AudioSettings } from '@/routes/settings/AudioSettings';
import { ModelsSettings } from '@/routes/settings/ModelsSettings';
import { ShortcutsSettings } from '@/routes/settings/ShortcutsSettings';
import { TextOutputSettings } from '@/routes/settings/TextOutputSettings';
import { PrivacySettings } from '@/routes/settings/PrivacySettings';
import { AdvancedSettings } from '@/routes/settings/AdvancedSettings';
import { AboutSettings } from '@/routes/settings/AboutSettings';
import { BarSettings } from '@/routes/settings/BarSettings';
import { LicenceSettings } from '@/routes/settings/LicenceSettings';
import { UpdatesSettings } from '@/routes/settings/UpdatesSettings';

// Workspace nav structural data — route paths, icons, and the stable
// `labelKey` under `common.chrome.nav.*`. The visible label is resolved
// at render time inside `MainShell` so a future locale flip swaps the
// copy without restart. `to`, `labelKey`, and `icon` MUST NOT change
// without updating the translation catalogue + tests.
const NAV_PRIMARY = [
  { to: '/dashboard', labelKey: 'chrome.nav.dashboard', icon: BarChart3 },
  { to: '/history', labelKey: 'chrome.nav.history', icon: HistoryIcon },
  { to: '/meetings', labelKey: 'chrome.nav.meetings', icon: Users },
  { to: '/modes', labelKey: 'chrome.nav.modes', icon: Shapes },
  { to: '/vocabulary', labelKey: 'chrome.nav.vocabulary', icon: Layers },
  { to: '/files', labelKey: 'chrome.nav.files', icon: FileAudio },
] as const;

// Settings nav data lives in `@/routes/settings_nav.ts` so the
// per-section / per-match display labels can flip with the active
// app language. Search keywords + per-control aliases stay English on
// purpose: they're search-haystack terms, not display strings, and
// keeping them static preserves `settings_search.ts` behaviour
// without architectural change.


/**
 * MainShell — the app's root chrome.
 *
 * Per DESIGN_SYSTEM.md §11.1 the shell is a two-column layout:
 *   • 240px sidebar on bg-1 (the "frame")
 *   • Content column on bg-0 (the "stage")
 *
 * Elevation comes from the surface tone step alone — no border, no shadow
 * on the divider. The sidebar is darker than the stage because cards on
 * the stage lift to bg-1 themselves; keeping the rail one step cooler
 * preserves the ladder bg-0 → bg-1 → bg-2 everywhere.
 */
export function MainShell() {
  const { t } = useTranslation('settings');
  const { t: tCommon } = useTranslation('common');
  const [appVersion, setAppVersion] = useState<string>('');
  // Settings search query. Empty string = no filter, full list shown.
  // Lives in shell state because the input sits in the sidebar (above
  // the Settings group label) and the filter result must drive the
  // NavGroup render in the same render pass.
  const [settingsQuery, setSettingsQuery] = useState('');
  const settingsSearchRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();
  const location = useLocation();
  const mainRef = useRef<HTMLElement>(null);

  // Build the Settings nav registry against the active language. The
  // memo keeps the array reference stable across re-renders that don't
  // change the language, so `filterSettingsByQuery` doesn't fire on
  // every keystroke against a freshly-allocated array.
  const navSettings = useMemo(() => buildSettingsNav(t), [t]);

  // Workspace nav resolved against the active common-namespace t().
  // Same memo discipline as `navSettings`: a fresh array on every
  // render would defeat NavLink memoisation downstream.
  const navPrimary = useMemo(
    () => NAV_PRIMARY.map((item) => ({ ...item, label: tCommon(item.labelKey) })),
    [tCommon],
  );

  // Reset scroll to the top whenever the route changes. Without this, paging
  // through Settings ("General" → "Accessibility" → "Privacy") keeps the scroll
  // position from the previous page, which reads as "the app didn't load the
  // new page" because the header is scrolled off. useLayoutEffect keeps the
  // reset synchronous — no one-frame flash of the old scroll position.
  useLayoutEffect(() => {
    mainRef.current?.scrollTo({ top: 0, left: 0, behavior: 'auto' });
  }, [location.pathname]);
  useEffect(() => {
    getVersion()
      .then(setAppVersion)
      .catch(() => undefined);
  }, []);

  // Cross-window navigation intent. The floating bar's History / Settings
  // buttons call `show_main_at` which emits `main:navigate` with the target
  // route — we honour it here so clicking History always lands on /history,
  // not "whatever you had open last time". `replace: true` keeps the router
  // history clean so repeated bar clicks don't pollute the back stack.
  useEffect(() => {
    const p = listen<string>(EVT.mainNavigate, (e) => {
      if (typeof e.payload === 'string' && e.payload.startsWith('/')) {
        navigate(e.payload, { replace: true });
      }
    });
    return () => {
      void p.then((u) => u()).catch(() => undefined);
    };
  }, [navigate]);

  return (
    <div className="flex h-full min-h-0 overflow-hidden bg-bg-0 text-fg">
      <aside className="flex h-full w-[248px] min-h-0 flex-col overflow-hidden bg-bg-1">
        {/* Sidebar lockup. Bumped from 28/18 → 36/24 per the brief: the old
            sizing read as "small utility chrome" and made the wordmark's
            "bar" half (in accent gradient) too tight to register as a
            colour moment. At 36/24 the mark reads as the app's identity
            from across a monitor, and the accent-coloured "bar" becomes a
            real brand beat the user notices on every open. Gap bumped in
            lockstep so the lockup rhythm is preserved. */}
        <div className="flex items-center gap-3 px-5 pt-7 pb-6">
          <UtterbarLogoMark size={36} />
          <UtterbarWordmark size={24} />
        </div>

        <nav className="flex-1 min-h-0 overflow-y-auto px-3 pb-4">
          <NavGroup label={t('nav.group.workspace')} items={navPrimary} />
          <SettingsSearchInput
            ref={settingsSearchRef}
            value={settingsQuery}
            onChange={setSettingsQuery}
            placeholder={t('nav.search.placeholder')}
            clearLabel={t('nav.search.clear')}
          />
          <NavGroup
            label={t('nav.group.settings')}
            items={filterSettingsByQuery(navSettings, settingsQuery).map((item) => {
              // While searching: every filtered item gets a short hint
              // showing the matched control / feature. ("filler words"
              // → Text Output, hint "Smart rewrite".) The hint replaces
              // the version line on the About row during search, which
              // is the right tradeoff: a stale "v0.18.0" under About
              // when the user is looking for a specific control is
              // distracting; the matched-feature hint is what actually
              // helps them navigate.
              if (settingsQuery.trim().length > 0) {
                const summary = settingsSearchMatchSummary(item, settingsQuery);
                return summary ? { ...item, subLabel: summary } : item;
              }
              // Otherwise: attach the live app version directly beneath
              // the `About` row. This is in addition to the footer strap
              // and the version shown inside the About page itself.
              // Users reported wanting the version glanceable from the
              // nav without opening About or scanning to the bottom.
              return item.to === '/settings/about'
                ? { ...item, subLabel: appVersion ? `v${appVersion}` : undefined }
                : item;
            })}
            emptyMessage={
              settingsQuery.trim().length > 0
                ? t('nav.search.no_match', { query: settingsQuery.trim() })
                : undefined
            }
          />
        </nav>

        {/* Footer — TierLine + version + "local only" strap.
            `shrink-0` is load-bearing: without it, flexbox can shrink
            this block to zero on shorter windows now that the aside is
            `overflow-hidden` (v0.6.2 triple-scrollbar fix). The version
            number went missing on shorter viewports because of that
            interaction. Pinning the footer to its intrinsic height
            keeps it visible regardless of nav overflow.
            Bottom padding is generous (pb-5) so the line sits a little
            off the window edge at any size — on shorter viewports the
            nav-scroll ends right up against the footer and pb-3 read
            as the version was glued to the edge. */}
        <div className="shrink-0 px-5 pt-3 pb-5 space-y-2">
          <TierLine />
          <div className="ub-eyebrow font-mono">
            {appVersion ? `v${appVersion}` : 'v…'} · {tCommon('chrome.footer.strap')}
          </div>
        </div>
      </aside>

      {/* Scroll container.
          Parent `<div>` is `flex h-screen overflow-hidden`, so this `<main>`
          must derive its height from the flex parent. The trio is:
          - `flex-1`         — fill the remaining width/height of the row
          - `min-h-0`        — allow this flex child to shrink below content
                               height (default `min-height: auto` makes the
                               child refuse to shrink, which kills overflow)
          - `overflow-y-auto`— produce the scrollbar when content > height
          - `h-full`         — belt-and-braces: even if `flex-1`'s height
                               resolution lags a layout pass, h-full pins us
                               to the parent's `h-screen` so we never overrun
          The previous v0.7.0 fix used `h-screen` here, which decoupled the
          height from the flex parent. That worked when the window was at
          exactly 100vh but left content clipped on shorter windows where
          the OS chrome / DPI rounding meant `100vh` didn't equal the true
          inner viewport. Going back to flex-derived height with the proper
          `min-h-0` is the textbook scroll container in a flex column. */}
      <main
        ref={mainRef}
        className="flex-1 min-h-0 overflow-y-auto bg-bg-0"
      >
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/history" element={<HistoryRoute />} />
          <Route path="/meetings" element={<MeetingsRoute />} />
          <Route path="/modes" element={<ModesRoute />} />
          <Route path="/vocabulary" element={<VocabularyRoute />} />
          <Route path="/files" element={<FilesRoute />} />
          <Route path="/settings/general" element={<GeneralSettings />} />
          <Route path="/settings/bar" element={<BarSettings />} />
          <Route path="/settings/audio" element={<AudioSettings />} />
          <Route path="/settings/models" element={<ModelsSettings />} />
          <Route path="/settings/shortcuts" element={<ShortcutsSettings />} />
          <Route path="/settings/text-output" element={<TextOutputSettings />} />
          <Route path="/settings/accessibility" element={<AccessibilitySettings />} />
          <Route path="/settings/privacy" element={<PrivacySettings />} />
          <Route path="/settings/advanced" element={<AdvancedSettings />} />
          <Route path="/settings/licence" element={<LicenceSettings />} />
          <Route path="/settings/updates" element={<UpdatesSettings />} />
          <Route path="/settings/about" element={<AboutSettings />} />
        </Routes>
      </main>
      <UpdateToast />
      <PostUpdateToast />
    </div>
  );
}

/**
 * Sidebar tier indicator. Visible on every page so the user always knows
 * whether the install is Pro — two forms:
 *
 *   • Free — muted "Upgrade to Pro →" link. Opens utterbar.com/pricing in
 *     the default browser. Not a CTA button: the spec allows one or two
 *     deliberate cyan moments per viewport, and an always-visible CTA in
 *     the sidebar would burn the budget before the user has even looked
 *     at the stage.
 *   • Pro  — cyan LED + "Pro" label as a link to Settings → Licence. Uses
 *     the LED primitive directly so the glow language stays consistent
 *     with the recording-state LED on the floating pill.
 *
 * Hidden until the store has hydrated to avoid flashing "Free" for one
 * frame on launch before `licence_status` resolves.
 */
function TierLine() {
  const { t } = useTranslation('common');
  const status = useLicence((s) => s.status);
  const loaded = useLicence((s) => s.loaded);
  if (!loaded || !status) return null;

  if (status.tier === 'pro') {
    return (
      <NavLink
        to="/settings/licence"
        className="inline-flex items-center gap-2 text-[12px] font-medium text-fg-hi transition-colors hover:text-accent"
      >
        <Led size={6} />
        {t('chrome.tier.pro')}
      </NavLink>
    );
  }
  return (
    <button
      onClick={() => void openShell('https://utterbar.com/pricing').catch(() => undefined)}
      className="text-[12px] font-medium text-muted transition-colors hover:text-fg-hi"
    >
      {t('chrome.tier.free_cta')}
    </button>
  );
}

/**
 * Sidebar nav group.
 *
 * Per DESIGN_SYSTEM.md §7 and §8.2, the active state is the canonical
 * ".ub-item-active" treatment: a 2px cyan rail on the left edge plus a
 * lift to bg-2. No tint wash, no cyan text, no glow. The rail is the
 * signal; the surface step is the confirmation.
 *
 * Hover on a non-selected row is a tonal shift to bg-2/60 — colour is
 * reserved for the active row and status signals (§1.6 cyan budget).
 *
 * Group headers use the .ub-eyebrow class (Instrument Sans 500, not mono).
 */
function NavGroup({
  label,
  items,
  emptyMessage,
}: {
  label: string;
  items: {
    to: string;
    label: string;
    icon: React.ComponentType<{ className?: string }>;
    /** Optional secondary line rendered directly under the main label.
     *  Currently used only for the live app version under the `About` row. */
    subLabel?: string;
  }[];
  /** Shown in place of the items list when `items` is empty. Used by
   *  the Settings group during a settings-search miss so the user
   *  sees "No settings match …" instead of a silently empty section. */
  emptyMessage?: string;
}) {
  return (
    <div className="mb-5">
      <div className="ub-eyebrow px-3 pb-2 pt-3">{label}</div>
      {items.length === 0 && emptyMessage ? (
        <div className="px-3 py-1 text-[12px] text-muted">{emptyMessage}</div>
      ) : null}
      <div className="flex flex-col gap-0.5">
        {items.map(({ to, label: l, icon: Icon, subLabel }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              cn(
                // Bumped from 13→14px and widened the row for legibility. The
                // old text-[13px] on `text-muted` (≈60% luminance) fell below
                // WCAG AA contrast on every theme — worst on Graphite where
                // muted vs bg-1 was nearly pastel. We now use `text-fg` for
                // the idle state which is full-strength foreground, and keep
                // `text-fg-hi` for the active row so the step from idle →
                // active is still visible.
                'group relative flex items-center gap-3 rounded-[10px] px-3 py-[8px] text-[14px] font-medium transition-colors',
                isActive
                  ? 'bg-bg-2 text-fg-hi'
                  : 'text-fg hover:bg-bg-2/60 hover:text-fg-hi',
              )
            }
          >
            {({ isActive }) => (
              <>
                {/* Active-state 2px cyan rail — positioned absolutely so
                    label alignment is identical active vs. idle. */}
                <span
                  className={cn(
                    'absolute left-0 top-1.5 bottom-1.5 w-[2px] rounded-r-full transition-opacity',
                    isActive ? 'bg-accent opacity-100' : 'opacity-0',
                  )}
                  aria-hidden="true"
                />
                <Icon
                  className={cn(
                    'h-[16px] w-[16px] shrink-0 transition-colors',
                    // Icon stays monochrome — spec forbids stacking a
                    // coloured icon on top of the already-coloured rail.
                    // Idle now rides on `muted` (not `fg-dim`) so icons
                    // don't disappear into the sidebar on Graphite.
                    isActive ? 'text-fg-hi' : 'text-muted group-hover:text-fg-hi',
                  )}
                />
                {subLabel ? (
                  // When the row carries a secondary line, stack the label
                  // + sub-label so the version sits directly beneath the
                  // `About` text (not beneath the icon). `leading-tight`
                  // keeps vertical rhythm close to the single-line rows.
                  <span className="flex min-w-0 flex-col leading-tight">
                    <span className="truncate">{l}</span>
                    <span className="truncate text-[11px] font-mono text-muted">
                      {subLabel}
                    </span>
                  </span>
                ) : (
                  <span className="truncate">{l}</span>
                )}
              </>
            )}
          </NavLink>
        ))}
      </div>
    </div>
  );
}

/**
 * Settings search input. Sits in the sidebar between the Workspace
 * group and the Settings group so the input is co-located with the
 * nav it filters. Empty value renders the same as no-search; users
 * who never engage the input never see the filter logic at all.
 *
 * Keyboard:
 *   • Esc clears the input. Focus stays in the input so the user can
 *     keep typing without re-clicking.
 *   • The input is plain — no global Ctrl+K shortcut. Adding a global
 *     hotkey would conflict with the floating bar's hotkey scheme and
 *     is out of scope for the search feature itself.
 */
const SettingsSearchInput = forwardRef<
  HTMLInputElement,
  {
    value: string;
    onChange: (next: string) => void;
    /** Translated placeholder + sr-only label. Same string for both
     *  surfaces so the screen-reader announcement matches the visual
     *  hint a sighted user sees. */
    placeholder: string;
    /** Translated aria-label for the clear button. */
    clearLabel: string;
  }
>(function SettingsSearchInput({ value, onChange, placeholder, clearLabel }, ref) {
  const hasValue = value.length > 0;
  return (
    <div className="px-3 pb-2 pt-1">
      <label className="relative block">
        <span className="sr-only">{placeholder}</span>
        <Search
          aria-hidden
          className="pointer-events-none absolute left-2 top-1/2 h-[14px] w-[14px] -translate-y-1/2 text-muted"
        />
        <input
          ref={ref}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Escape' && hasValue) {
              e.preventDefault();
              onChange('');
            }
          }}
          placeholder={placeholder}
          spellCheck={false}
          autoComplete="off"
          className={cn(
            'w-full rounded-[8px] bg-bg-0 py-1.5 pl-7 pr-7 text-[13px]',
            'text-fg placeholder:text-muted',
            'border border-transparent transition-colors',
            'hover:border-border-soft/70 focus:border-accent/60 focus:outline-none',
          )}
        />
        {hasValue ? (
          <button
            type="button"
            onClick={() => onChange('')}
            aria-label={clearLabel}
            className="absolute right-1 top-1/2 -translate-y-1/2 rounded p-1 text-muted transition-colors hover:text-fg-hi"
          >
            <X className="h-[12px] w-[12px]" />
          </button>
        ) : null}
      </label>
    </div>
  );
});
