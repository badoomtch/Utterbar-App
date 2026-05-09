import { useRef, useState } from 'react';
import { Palette, Sparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { PageHeader } from '@/components/common/PageHeader';
import { Card, Row } from '@/components/common/Card';
import { Toggle } from '@/components/common/Toggle';
import { UpgradeModal } from '@/components/upgrade/UpgradeModal';
import { useSettings } from '@/state/settings';
import { useLicence, isPro } from '@/state/licence';
import { cn } from '@/lib/cn';
import { previewProfile, SOUND_PROFILES, type SoundProfileId } from '@/design/sounds';
import type { AppLanguage } from '@/generated/AppLanguage';

const ACCENTS = ['teal', 'amber', 'violet', 'rose', 'cyan', 'lime'] as const;
const THEMES = ['obsidian', 'graphite', 'paper'] as const;
// Languages currently available for the desktop UI. Adding a locale means
// (a) appending its `AppLanguage` variant here, (b) adding the matching
// JSON resources in `src/i18n/locales/`, and (c) ensuring `common.language.<code>`
// exists in EVERY locale's catalogue so the chip label renders in the
// language it represents (e.g. "Español" displays the same regardless of
// which UI language is currently active).
const APP_LANGUAGES: ReadonlyArray<AppLanguage> = [
  'en',
  'es',
  'fr',
  'de',
  'it',
  'pt',
  'nl',
] as const;

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
  const { t: tSettings } = useTranslation('settings');
  const { t: tCommon } = useTranslation('common');
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
      <PageHeader title={tSettings('general.title')} />
      <div className="mt-6 space-y-4">
        <Card title={tSettings('general.startup.title')}>
          <Row
            label={tSettings('general.startup.launch_on_login.label')}
            description={tSettings('general.startup.launch_on_login.description')}
          >
            <Toggle
              checked={Boolean(g.launch_on_login)}
              onChange={(v) => void update('launch_on_login', v)}
            />
          </Row>
          <Row
            label={tSettings('general.startup.start_minimised.label')}
            description={tSettings('general.startup.start_minimised.description')}
          >
            <Toggle
              checked={Boolean(g.start_minimised)}
              onChange={(v) => void update('start_minimised', v)}
            />
          </Row>
          <Row
            label={tSettings('general.startup.close_to_tray.label')}
            description={tSettings('general.startup.close_to_tray.description')}
          >
            <Toggle
              checked={Boolean(g.close_to_tray)}
              onChange={(v) => void update('close_to_tray', v)}
            />
          </Row>
        </Card>

        <Card title={tSettings('general.appearance.title')}>
          <Row
            label={tSettings('general.appearance.theme.label')}
            description={tSettings('general.appearance.theme.description')}
          >
            <div className="flex gap-2">
              {THEMES.map((themeId) => (
                <button
                  key={themeId}
                  onClick={() => void update('theme', themeId)}
                  className={cn(
                    'ub-chip',
                    s.general.theme === themeId && 'ub-chip--active',
                  )}
                >
                  {tSettings(`general.appearance.theme.options.${themeId}` as const)}
                </button>
              ))}
            </div>
          </Row>
          <Row
            label={tSettings('general.appearance.accent.label')}
            description={tSettings('general.appearance.accent.description')}
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
                  // Accent palette identifiers (`teal`, `amber`, ...) are
                  // design-system tokens, not user prose. Tooltip + aria
                  // surface the token name as-is so the active swatch is
                  // identifiable to assistive tech without each colour
                  // needing per-locale copy.
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
                    ? tSettings('general.appearance.custom_accent.title_pro')
                    : tSettings('general.appearance.custom_accent.title_free')
                }
                aria-label={tSettings('general.appearance.custom_accent.aria')}
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
          feature={tSettings('general.upgrade.custom_accent.feature')}
          description={tSettings('general.upgrade.custom_accent.description')}
          bullets={[
            tSettings('general.upgrade.custom_accent.bullets.any_hex'),
            tSettings('general.upgrade.custom_accent.bullets.auto_tint_shade'),
            tSettings('general.upgrade.custom_accent.bullets.presets_free'),
          ]}
        />

        <Card title={tSettings('general.app_language.label')}>
          {/* Stacked layout instead of Row's side-by-side. With seven
              chips in the picker the Row's label column collapses to
              single-word width (the chip container is shrink-0 so it
              wins the flex tug-of-war, leaving the label nowhere to
              go). Stacking puts the description above the chips so
              both can use the full Card width. */}
          <div>
            <div className="text-[13px] font-medium text-fg-hi">
              {tSettings('general.app_language.label')}
            </div>
            <div className="mt-1 text-[12px] leading-relaxed text-muted">
              {tSettings('general.app_language.description')}
            </div>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              {APP_LANGUAGES.map((code) => (
                <button
                  key={code}
                  onClick={() => void update('app_language', code)}
                  className={cn(
                    'ub-chip',
                    (g.app_language ?? 'en') === code && 'ub-chip--active',
                  )}
                >
                  {tCommon(`language.${code === 'en' ? 'english' : code}`)}
                </button>
              ))}
            </div>
          </div>
        </Card>

        <Card title={tSettings('general.feedback.title')}>
          <Row
            label={tSettings('general.feedback.sound_cues.label')}
            description={tSettings('general.feedback.sound_cues.description')}
          >
            <Toggle
              checked={Boolean(g.sound_cues_enabled)}
              onChange={(v) => void update('sound_cues_enabled', v)}
            />
          </Row>
          {Boolean(g.sound_cues_enabled) && (
            <Row
              label={tSettings('general.feedback.sound_profile.label')}
              description={tSettings('general.feedback.sound_profile.description')}
            >
              <div className="flex flex-wrap items-center gap-2">
                {SOUND_PROFILES.map((p) => {
                  const current =
                    ((g.sound_profile as SoundProfileId | undefined) ?? 'default') === p.id;
                  // Profile id (`default`/`classic`/`subtle`/`chime`) is the
                  // structural identifier from sounds.ts; user-facing copy
                  // (label + description) lives in settings.json so each
                  // profile gets one row in `feedback.sound_profile.options`.
                  const labelKey = `general.feedback.sound_profile.options.${p.id}.label` as const;
                  const descriptionKey = `general.feedback.sound_profile.options.${p.id}.description` as const;
                  return (
                    <button
                      key={p.id}
                      onClick={() => {
                        void update('sound_profile', p.id);
                        previewProfile(p.id);
                      }}
                      title={tSettings(descriptionKey)}
                      className={cn('ub-chip', current && 'ub-chip--active')}
                    >
                      {tSettings(labelKey)}
                    </button>
                  );
                })}
              </div>
            </Row>
          )}
          <Row
            label={tSettings('general.feedback.notifications.label')}
            description={tSettings('general.feedback.notifications.description')}
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

