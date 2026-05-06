import 'i18next';

import type barEn from './i18n/locales/en/bar.json';
import type commonEn from './i18n/locales/en/common.json';
import type onboardingEn from './i18n/locales/en/onboarding.json';
import type settingsEn from './i18n/locales/en/settings.json';

// Augments i18next so `t('bar:phase.ready')` is type-checked end-to-end:
// keys are autocompleted, misspellings break `tsc --noEmit`, and pruning
// a key from the JSON catalogue forces every call site to react.
//
// The English JSON files are the source of truth for the typed schema —
// new locales must mirror their shape exactly (the missing-keys vitest
// test enforces this at build time).
declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common';
    resources: {
      bar: typeof barEn;
      common: typeof commonEn;
      onboarding: typeof onboardingEn;
      settings: typeof settingsEn;
    };
  }
}
