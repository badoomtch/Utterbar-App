import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import barEn from './locales/en/bar.json';
import commonEn from './locales/en/common.json';
import onboardingEn from './locales/en/onboarding.json';
import settingsEn from './locales/en/settings.json';

import barEs from './locales/es/bar.json';
import commonEs from './locales/es/common.json';
import onboardingEs from './locales/es/onboarding.json';
import settingsEs from './locales/es/settings.json';

import barFr from './locales/fr/bar.json';
import commonFr from './locales/fr/common.json';
import onboardingFr from './locales/fr/onboarding.json';
import settingsFr from './locales/fr/settings.json';

import barDe from './locales/de/bar.json';
import commonDe from './locales/de/common.json';
import onboardingDe from './locales/de/onboarding.json';
import settingsDe from './locales/de/settings.json';

import barIt from './locales/it/bar.json';
import commonIt from './locales/it/common.json';
import onboardingIt from './locales/it/onboarding.json';
import settingsIt from './locales/it/settings.json';

import barPt from './locales/pt/bar.json';
import commonPt from './locales/pt/common.json';
import onboardingPt from './locales/pt/onboarding.json';
import settingsPt from './locales/pt/settings.json';

import barNl from './locales/nl/bar.json';
import commonNl from './locales/nl/common.json';
import onboardingNl from './locales/nl/onboarding.json';
import settingsNl from './locales/nl/settings.json';

import type { AppLanguage } from '@/generated/AppLanguage';

// Resources are bundled directly. No backend loader, no async fetch — every
// string ships with the binary, so first paint never has to wait on IO.
// Adding a new locale means importing its JSON files here and adding a
// branch to `resolveLng()`. Keep one namespace per UI region; this file is
// the single source of truth for the namespace registry.
const resources = {
  en: {
    bar: barEn,
    common: commonEn,
    onboarding: onboardingEn,
    settings: settingsEn,
  },
  es: {
    bar: barEs,
    common: commonEs,
    onboarding: onboardingEs,
    settings: settingsEs,
  },
  fr: {
    bar: barFr,
    common: commonFr,
    onboarding: onboardingFr,
    settings: settingsFr,
  },
  de: {
    bar: barDe,
    common: commonDe,
    onboarding: onboardingDe,
    settings: settingsDe,
  },
  it: {
    bar: barIt,
    common: commonIt,
    onboarding: onboardingIt,
    settings: settingsIt,
  },
  pt: {
    bar: barPt,
    common: commonPt,
    onboarding: onboardingPt,
    settings: settingsPt,
  },
  nl: {
    bar: barNl,
    common: commonNl,
    onboarding: onboardingNl,
    settings: settingsNl,
  },
} as const;

// Maps the Rust-side `AppLanguage` enum (BCP-47 short codes) to the i18next
// language tag. Keep this aligned with the `serde(rename_all = "lowercase")`
// attribute on the Rust enum.
function resolveLng(lang: AppLanguage | undefined): string {
  switch (lang) {
    case 'es':
      return 'es';
    case 'fr':
      return 'fr';
    case 'de':
      return 'de';
    case 'it':
      return 'it';
    case 'pt':
      return 'pt';
    case 'nl':
      return 'nl';
    case 'en':
    default:
      return 'en';
  }
}

void i18n.use(initReactI18next).init({
  resources,
  lng: 'en',
  fallbackLng: 'en',
  supportedLngs: ['en', 'es', 'fr', 'de', 'it', 'pt', 'nl'],
  defaultNS: 'common',
  ns: ['bar', 'common', 'onboarding', 'settings'],
  interpolation: {
    // React already escapes everything it renders, so the i18next-side
    // escape would just double-encode `&` and friends. Disable it here.
    escapeValue: false,
  },
  returnEmptyString: false,
});

/**
 * Set the active language to match the user's `general.app_language` setting.
 * Safe to call repeatedly — i18next short-circuits when the language hasn't
 * changed. Called from `AppBootstrap` once settings hydrate, and again on
 * every subsequent `app_language` change.
 */
export function applyAppLanguage(lang: AppLanguage | undefined): void {
  const tag = resolveLng(lang);
  if (i18n.language !== tag) {
    void i18n.changeLanguage(tag);
  }
}

export default i18n;
