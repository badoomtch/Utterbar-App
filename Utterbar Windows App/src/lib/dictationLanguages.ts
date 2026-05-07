/**
 * Dictation language catalogue.
 *
 * The list of languages the user can pick as their *dictation* language —
 * i.e. the language Utterbar listens for. This is deliberately distinct
 * from the *app interface* language (`general.app_language`); see
 * MONOREPO.md §10 + the App-language picker copy in
 * settings.general.app_language for the contract.
 *
 * Codes here are written verbatim into `models.whisper_language` on the
 * Settings store. `'auto'` means "let Whisper detect the language per
 * utterance" and is the safe default.
 *
 * The shortlist mirrors the one already in
 * `routes/settings/ModelsSettings.tsx` (Whisper Language picker, ~26
 * codes) so the two surfaces stay aligned. Whisper itself supports
 * ~99 languages; advanced users can edit `whisper_language` in
 * settings.json directly.
 *
 * Visible labels resolve through i18next at render time:
 *   - `auto` uses `common.language.auto_detect`
 *   - `en` reuses the existing `common.language.english`
 *   - all others use `common.language.<code>` keys added alongside this
 *     module.
 */

export type DictationLanguageCode =
  | 'auto'
  | 'en'
  | 'es'
  | 'fr'
  | 'de'
  | 'it'
  | 'pt'
  | 'nl'
  | 'pl'
  | 'tr'
  | 'ru'
  | 'ar'
  | 'hi'
  | 'ja'
  | 'ko'
  | 'zh'
  | 'id'
  | 'th'
  | 'vi'
  | 'sv'
  | 'no'
  | 'da'
  | 'fi'
  | 'cs'
  | 'el'
  | 'he';

export type DictationLanguageEntry = {
  /** ISO-639-1 code (or `auto`) written into models.whisper_language. */
  readonly code: DictationLanguageCode;
  /** i18next key (in the `common` namespace) for the visible label. */
  readonly labelKey: string;
};

/**
 * Ordered for the picker — `auto` first, then English, then a rough
 * popularity / Latin-vs-non-Latin grouping that matches the existing
 * Whisper Language Select in ModelsSettings.tsx. Order is part of the
 * UI; do not re-sort alphabetically without also updating the picker.
 */
export const DICTATION_LANGUAGES: readonly DictationLanguageEntry[] = [
  { code: 'auto', labelKey: 'common.language.auto_detect' },
  { code: 'en', labelKey: 'common.language.english' },
  { code: 'es', labelKey: 'common.language.spanish' },
  { code: 'fr', labelKey: 'common.language.french' },
  { code: 'de', labelKey: 'common.language.german' },
  { code: 'it', labelKey: 'common.language.italian' },
  { code: 'pt', labelKey: 'common.language.portuguese' },
  { code: 'nl', labelKey: 'common.language.dutch' },
  { code: 'pl', labelKey: 'common.language.polish' },
  { code: 'tr', labelKey: 'common.language.turkish' },
  { code: 'ru', labelKey: 'common.language.russian' },
  { code: 'ar', labelKey: 'common.language.arabic' },
  { code: 'hi', labelKey: 'common.language.hindi' },
  { code: 'ja', labelKey: 'common.language.japanese' },
  { code: 'ko', labelKey: 'common.language.korean' },
  { code: 'zh', labelKey: 'common.language.chinese' },
  { code: 'id', labelKey: 'common.language.indonesian' },
  { code: 'th', labelKey: 'common.language.thai' },
  { code: 'vi', labelKey: 'common.language.vietnamese' },
  { code: 'sv', labelKey: 'common.language.swedish' },
  { code: 'no', labelKey: 'common.language.norwegian' },
  { code: 'da', labelKey: 'common.language.danish' },
  { code: 'fi', labelKey: 'common.language.finnish' },
  { code: 'cs', labelKey: 'common.language.czech' },
  { code: 'el', labelKey: 'common.language.greek' },
  { code: 'he', labelKey: 'common.language.hebrew' },
] as const;

/**
 * `auto` and `en` are the languages every catalogue model supports
 * (Parakeet, Whisper, Zipformer); anything else needs a multilingual
 * model. Used by the recommendation helper to decide whether the
 * user's dictation choice constrains model selection.
 */
export function isMultilingualOnly(code: string): boolean {
  return code !== 'auto' && code !== 'en';
}
