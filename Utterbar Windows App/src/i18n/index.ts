import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import barEn from './locales/en/bar.json';
import commonEn from './locales/en/common.json';
import settingsEn from './locales/en/settings.json';

import type { AppLanguage } from '@/generated/AppLanguage';

// Resources are bundled directly. No backend loader, no async fetch — every
// English string ships with the binary, so first paint never has to wait on
// IO. Adding a new locale means importing its JSON files here and adding a
// branch to `resolveLng()`. Keep one namespace per UI region; this file is
// the single source of truth for the namespace registry.
const resources = {
  en: {
    bar: barEn,
    common: commonEn,
    settings: settingsEn,
  },
} as const;

// Maps the Rust-side `AppLanguage` enum (BCP-47 short codes) to the i18next
// language tag. `AppLanguage::En` -> `"en"`. Keep this aligned with the
// `serde(rename_all = "lowercase")` attribute on the Rust enum.
function resolveLng(lang: AppLanguage | undefined): string {
  switch (lang) {
    case 'en':
    default:
      return 'en';
  }
}

void i18n.use(initReactI18next).init({
  resources,
  lng: 'en',
  fallbackLng: 'en',
  defaultNS: 'common',
  ns: ['bar', 'common', 'settings'],
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
