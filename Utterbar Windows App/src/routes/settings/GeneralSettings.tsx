import { useRef, useState } from 'react';
import { Palette, Sparkles } from 'lucide-react';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { UpgradeModal } from '@/components/upgrade/UpgradeModal';
import { useSettings } from '@/state/settings';
import { useLicence, isPro } from '@/state/licence';
import { cn } from '@/lib/cn';
import { previewProfile, SOUND_PROFILES, type SoundProfileId } from '@/design/sounds';

const ACCENTS = ['teal', 'amber', 'violet', 'rose', 'cyan', 'lime'] as const;
const THEMES = ['obsidian', 'graphite', 'paper'] as const;

/**
 * General settings.
 *
 * Per DESIGN_SYSTEM.md §6.3 chip-style pickers use the .ub-chip class.
 * Active chips use .ub-chip--active (cyan fill, dark text), never a
 * bordered "selected" ring. This keeps the active-state vocabulary
 * consistent with list selection surfaces (rail) and navigation (rail).
 *
 * Channel selector + auto-check + check-now controls live on the
 * dedicated Updates page (`UpdatesSettings.tsx`). This page covers
 * theme, accent, sound profile, notifications, and the release-email
 * subscription only.
 */
export function GeneralSettings() {
  const s = useSettings((x) => x.data);
  const patch = useSettings((x) => x.patch);
  const licenceStatus = useLicence((x) => x.status);
  const pro = isPro(licenceStatus);
  const [showAccentUpgrade, setShowAccentUpgrade] = useState(false);
  // Hidden <input type="color">, clicking the Custom swatch chip opens the
  // OS colour picker through this ref, so we don't have to build a palette
  // UI ourselves and the user gets their native picker with recent colours,
  // hex input, and eyedropper (where Win32 supports it).
  const colorPickerRef = useRef<HTMLInputElement>(null);
  if (!s) return null;

  const update = (key: string, value: unknown) =>
    patch({ general: { ...s.general, [key]: value } });

  const g = s.general as Record<string, unknown>;
  const customHex =
    typeof g.custom_accent_hex === 'string' && /^#[0-9a-fA-F]{6}$/.test(g.custom_accent_hex)
      ? g.custom_accent_hex
      : '#22D3EE';

  return (
    <div className="mx-auto max-w-3xl p-8 pb-16">
      <PageHeader title="General" />
      <div className="mt-6 space-y-4">
        <Card title="Startup">
          <Row
            label="Launch on login"
            description="Start Utterbar automatically when you sign in to Windows, so the floating bar is always there without opening it manually."
          >
            <Toggle
              checked={Boolean(g.launch_on_login)}
              onChange={(v) => void update('launch_on_login', v)}
            />
          </Row>
          <Row
            label="Start minimised"
            description="When on, Utterbar launches straight to the tray and the floating bar, no main window until you ask for it. When off, the Dashboard opens on launch so you see your stats immediately."
          >
            <Toggle
              checked={Boolean(g.start_minimised)}
              onChange={(v) => void update('start_minimised', v)}
            />
          </Row>
          <Row
            label="Close to tray"
            description="Pressing the X on any window hides it to the tray instead of quitting."
          >
            <Toggle
              checked={Boolean(g.close_to_tray)}
              onChange={(v) => void update('close_to_tray', v)}
            />
          </Row>
        </Card>

        <Card title="Appearance">
          <Row label="Theme" description="Background palette for the main window and onboarding.">
            <div className="flex gap-2">
              {THEMES.map((t) => (
                <button
                  key={t}
                  onClick={() => void update('theme', t)}
                  className={cn(
                    'ub-chip capitalize',
                    s.general.theme === t && 'ub-chip--active',
                  )}
                >
                  {t}
                </button>
              ))}
            </div>
          </Row>
          <Row
            label="Accent"
            description="The colour used for the recording button, the pill glyph, highlights, and the floating bar's waveform."
          >
            <div className="flex items-center gap-2">
              {ACCENTS.map((a) => (
                <button
                  key={a}
                  onClick={() => void update('accent', a)}
                  className={cn(
                    // Raw swatch picker, NOT a ub-chip because the swatch
                    // IS the label. Ring on the active one so the currently
                    // chosen colour is obvious without colouring the border.
                    `h-6 w-6 rounded-full accent-${a} bg-accent transition-shadow duration-180`,
                    s.general.accent === a
                      ? 'ring-2 ring-fg-hi ring-offset-2 ring-offset-bg-1'
                      : 'opacity-70 hover:opacity-100',
                  )}
                  title={a}
                  aria-label={a}
                />
              ))}
              {/* Custom accent (Pro). For Free users, clicking this swatch
                  opens the upgrade modal instead of the colour picker, same
                  click-to-learn pattern used on the Modes page. Pro users
                  get the native OS colour picker via a hidden input[type=
                  color]. The swatch itself always previews the user's last
                  pick (or cyan if they've never picked), giving a hint at
                  what Pro unlocks without showing a dead grey tile. */}
              <button
                type="button"
                onClick={() => {
                  if (!pro) {
                    setShowAccentUpgrade(true);
                    return;
                  }
                  colorPickerRef.current?.click();
                }}
                title={
                  pro
                    ? 'Pick any colour (Pro)'
                    : 'Custom accent is a Pro feature'
                }
                aria-label="Custom accent colour"
                className={cn(
                  'relative flex h-6 w-6 items-center justify-center rounded-full transition-shadow duration-180',
                  s.general.accent === 'custom'
                    ? 'ring-2 ring-fg-hi ring-offset-2 ring-offset-bg-1'
                    : 'opacity-80 hover:opacity-100',
                )}
                style={{ background: pro ? customHex : 'var(--bg-2)' }}
              >
                {pro ? (
                  <Palette className="h-3 w-3 text-white mix-blend-difference" />
                ) : (
                  <Sparkles className="h-3 w-3 text-accent" />
                )}
              </button>
              {pro && (
                <input
                  ref={colorPickerRef}
                  type="color"
                  // Hidden but focusable, we drive the click from the
                  // swatch above. `aria-hidden` because the swatch is the
                  // accessible affordance.
                  aria-hidden
                  tabIndex={-1}
                  className="pointer-events-none absolute h-0 w-0 opacity-0"
                  value={customHex}
                  onChange={(e) => {
                    // Patch both fields in one go so the backend sees a
                    // coherent "custom + hex" update. A two-step patch
                    // (accent=custom, then hex) briefly flashes the last-
                    // known hex before the new one lands.
                    void patch({
                      general: {
                        ...s.general,
                        accent: 'custom',
                        custom_accent_hex: e.target.value.toUpperCase(),
                      },
                    });
                  }}
                />
              )}
            </div>
          </Row>
        </Card>
        <UpgradeModal
          open={showAccentUpgrade}
          onClose={() => setShowAccentUpgrade(false)}
          feature="Custom accent colours"
          description="Pick any colour for the accent family, the recording button, the floating bar's waveform, the pill glyph, and every highlight. The six presets are free; bring-your-own is part of Pro."
          bullets={[
            'Pick any hex colour from the native Windows colour picker',
            'Auto-derived tint and shade so gradients still have depth',
            'All six preset accents stay free for everyone',
          ]}
        />

        <Card title="Feedback">
          <Row
            label="Sound cues"
            description="Play a short synthesised tone when recording starts, stops, succeeds, or errors. The tones are generated in-app via WebAudio so there are no sample files, low-volume by default."
          >
            <Toggle
              checked={Boolean(g.sound_cues_enabled)}
              onChange={(v) => void update('sound_cues_enabled', v)}
            />
          </Row>
          {Boolean(g.sound_cues_enabled) && (
            <Row
              label="Sound profile"
              description="Which family of cue tones to use. Click a profile to hear its success cue."
            >
              <div className="flex flex-wrap items-center gap-2">
                {SOUND_PROFILES.map((p) => {
                  const current =
                    ((g.sound_profile as SoundProfileId | undefined) ?? 'default') === p.id;
                  return (
                    <button
                      key={p.id}
                      onClick={() => {
                        void update('sound_profile', p.id);
                        previewProfile(p.id);
                      }}
                      title={p.description}
                      className={cn('ub-chip', current && 'ub-chip--active')}
                    >
                      {p.label}
                    </button>
                  );
                })}
              </div>
            </Row>
          )}
          <Row
            label="System notifications"
            description="Raise a Windows toast when something actually needs your attention, dictation errors, model-install failures, successful model installs."
          >
            <Toggle
              checked={Boolean(g.notifications_enabled)}
              onChange={(v) => void update('notifications_enabled', v)}
            />
          </Row>
        </Card>

        {/* Channel selector, check-for-updates controls, and the
            release-email subscription all live on the dedicated
            /settings/updates page. General Settings is purely about
            theme, accent, sound profile, and notifications. */}
      </div>
    </div>
  );
}

