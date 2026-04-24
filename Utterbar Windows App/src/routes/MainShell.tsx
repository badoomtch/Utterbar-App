import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { NavLink, Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom';
import { getVersion } from '@tauri-apps/api/app';
import { open as openShell } from '@tauri-apps/plugin-shell';

import { listen } from '@/lib/ipc';
import { EVT } from '@/lib/events';
import { Led } from '@/components/common/Led';
import { useLicence } from '@/state/licence';
import {
  Accessibility,
  BarChart3,
  FileAudio,
  History as HistoryIcon,
  KeyRound,
  Keyboard,
  Layers,
  Mic,
  Radio,
  Settings2,
  Shapes,
  ShieldCheck,
  SlidersHorizontal,
  Info,
  Minimize2,
} from 'lucide-react';

import { UtterbarLogoMark } from '@/components/bar/UtterbarLogoMark';
import { UtterbarWordmark } from '@/components/bar/UtterbarWordmark';
import { UpdateToast } from '@/components/common/UpdateToast';
import { cn } from '@/lib/cn';
import { Dashboard } from '@/routes/Dashboard';
import { HistoryRoute } from '@/routes/HistoryRoute';
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

const NAV_PRIMARY = [
  { to: '/dashboard', label: 'Dashboard', icon: BarChart3 },
  { to: '/history', label: 'History', icon: HistoryIcon },
  { to: '/modes', label: 'Modes', icon: Shapes },
  { to: '/vocabulary', label: 'Vocabulary', icon: Layers },
  { to: '/files', label: 'Files', icon: FileAudio },
];

const NAV_SETTINGS = [
  { to: '/settings/general', label: 'General', icon: Settings2 },
  { to: '/settings/bar', label: 'Utterbar', icon: Minimize2 },
  { to: '/settings/audio', label: 'Audio', icon: Mic },
  { to: '/settings/models', label: 'Models', icon: Radio },
  { to: '/settings/shortcuts', label: 'Shortcuts', icon: Keyboard },
  { to: '/settings/text-output', label: 'Text Output', icon: SlidersHorizontal },
  { to: '/settings/accessibility', label: 'Accessibility', icon: Accessibility },
  { to: '/settings/privacy', label: 'Privacy', icon: ShieldCheck },
  { to: '/settings/advanced', label: 'Advanced', icon: SlidersHorizontal },
  { to: '/settings/licence', label: 'Licence', icon: KeyRound },
  { to: '/settings/about', label: 'About', icon: Info },
];

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
  const [appVersion, setAppVersion] = useState<string>('');
  const navigate = useNavigate();
  const location = useLocation();
  const mainRef = useRef<HTMLElement>(null);

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
    <div className="flex h-screen overflow-hidden bg-bg-0 text-fg">
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
          <NavGroup label="Workspace" items={NAV_PRIMARY} />
          <NavGroup label="Settings" items={NAV_SETTINGS} />
        </nav>

        <div className="px-5 py-3 space-y-2">
          <TierLine />
          <div className="ub-eyebrow font-mono">
            {appVersion ? `v${appVersion}` : 'v…'} · local only
          </div>
        </div>
      </aside>

      <main ref={mainRef} className="flex-1 min-h-0 overflow-y-auto bg-bg-0">
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/history" element={<HistoryRoute />} />
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
          <Route path="/settings/about" element={<AboutSettings />} />
        </Routes>
      </main>
      <UpdateToast />
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
        Pro
      </NavLink>
    );
  }
  return (
    <button
      onClick={() => void openShell('https://utterbar.com/pricing').catch(() => undefined)}
      className="text-[12px] font-medium text-muted transition-colors hover:text-fg-hi"
    >
      Upgrade to Pro →
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
}: {
  label: string;
  items: { to: string; label: string; icon: React.ComponentType<{ className?: string }> }[];
}) {
  return (
    <div className="mb-5">
      <div className="ub-eyebrow px-3 pb-2 pt-3">{label}</div>
      <div className="flex flex-col gap-0.5">
        {items.map(({ to, label: l, icon: Icon }) => (
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
                <span className="truncate">{l}</span>
              </>
            )}
          </NavLink>
        ))}
      </div>
    </div>
  );
}
