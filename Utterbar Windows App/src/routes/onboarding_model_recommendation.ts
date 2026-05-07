/**
 * Pure helper that picks the best onboarding-time model recommendation
 * for the user's chosen *dictation* language. Lives separately from
 * `Onboarding.tsx` so the rules stay testable in pure Node.
 *
 * Inputs come from the same `list_available_models` IPC the
 * onboarding model-step already calls. The helper does not invoke any
 * Tauri command or read from any store — callers wire it up.
 *
 * Model language metadata contract (see services/models.rs):
 *   - English-only models advertise `languages: ["en"]`
 *     (Parakeet-TDT, English-only Whisper variants, Zipformer).
 *   - Multilingual Whisper models advertise `languages: ["multi"]`.
 *
 * Decision rules:
 *   - `auto` and `en` dictation: keep the existing onboarding default
 *     (installed > recommended > first), no compatibility filtering.
 *   - any other code (`fr`, `es`, `de`, …): prefer a multilingual
 *     model. Existing English-only recommendations are flagged
 *     `incompatible` so the UI can warn and one-click switch.
 *   - if no multilingual model exists, return a fallback warning so
 *     the UI prompts the user to install Whisper from Settings.
 */

import { isMultilingualOnly } from '@/lib/dictationLanguages';

export type RecommendationModel = {
  /** Catalogue id (matches `services/models.rs`). */
  readonly id: string;
  /** From the model catalogue: `["en"]` or `["multi"]`. */
  readonly languages: readonly string[];
  /** Whether the model is already on disk. */
  readonly installed: boolean;
  /** Backend-marked default recommendation. */
  readonly recommended: boolean;
};

export type Reason =
  /** Default branch. The recommendation is the existing
   *  installed/recommended/first pick; current selection is
   *  compatible. */
  | 'default'
  /** Non-English dictation language; the helper picked a
   *  multilingual model and the current selection is compatible
   *  (or matches the new recommendation). */
  | 'multilingual_match'
  /** Non-English dictation language but the currently-selected
   *  model is English-only. The UI should warn and offer the
   *  multilingual recommendation as a one-click switch. */
  | 'selection_english_only'
  /** Non-English dictation language and the catalogue contains no
   *  multilingual model at all. The UI should explain and route
   *  the user to Settings → Models to install Whisper. */
  | 'no_multilingual_available';

export type Recommendation = {
  /** Catalogue id of the recommended model, or `null` when the
   *  catalogue is empty or no multilingual fallback exists. */
  readonly recommendedId: string | null;
  /** Whether the caller-supplied `currentSelection` is compatible
   *  with the dictation language. `true` when current selection
   *  is the recommendedId, or when it's a multilingual model
   *  even if not the one the helper would pick first. */
  readonly compatible: boolean;
  /** Why the helper made this call. The UI keys its warning copy
   *  off this so the catalogue (`onboarding.language.warning.*`)
   *  stays in lock-step with the rule set. */
  readonly reason: Reason;
};

/**
 * `'auto'` and `'en'` are compatible with every catalogue model;
 * anything else requires `languages` to include `'multi'`. Pure check —
 * no side effects.
 */
export function modelSupportsLanguage(
  model: Pick<RecommendationModel, 'languages'>,
  language: string,
): boolean {
  if (!isMultilingualOnly(language)) return true;
  return model.languages.includes('multi');
}

/**
 * Reproduce the existing onboarding default: installed first,
 * else recommended, else the first available model. Used as the
 * fallback when no language constraint is in effect.
 */
function defaultPick(models: readonly RecommendationModel[]): string | null {
  const installed = models.find((m) => m.installed);
  const recommended = models.find((m) => m.recommended);
  return installed?.id ?? recommended?.id ?? models[0]?.id ?? null;
}

/**
 * Pick a multilingual model. Same priority as the default branch
 * (installed > recommended > first) but filtered to multi-language
 * models only. Returns `null` when the catalogue has no multilingual
 * entry — caller surfaces the `no_multilingual_available` fallback.
 */
function multilingualPick(models: readonly RecommendationModel[]): string | null {
  const multi = models.filter((m) => m.languages.includes('multi'));
  return defaultPick(multi);
}

/**
 * Main entry point. Given the onboarding model list, the user's
 * current pick, and the chosen dictation language, returns the
 * model id to recommend plus a reason code the UI keys its copy off.
 *
 * Pure. Stable order: language check first, then catalogue
 * filtering, then current-selection compatibility check. Caller
 * decides what to do with the result (auto-select the recommendation,
 * or warn-and-offer a one-click switch).
 */
export function recommendModelForLanguage(args: {
  readonly models: readonly RecommendationModel[];
  readonly currentSelection: string | null;
  readonly dictationLanguage: string;
}): Recommendation {
  const { models, currentSelection, dictationLanguage } = args;

  // English / auto: existing behaviour, no filtering.
  if (!isMultilingualOnly(dictationLanguage)) {
    const recommendedId = defaultPick(models);
    return {
      recommendedId,
      compatible: true,
      reason: 'default',
    };
  }

  // Non-English: must be a multilingual model.
  const multilingualId = multilingualPick(models);
  if (multilingualId === null) {
    return {
      recommendedId: null,
      compatible: false,
      reason: 'no_multilingual_available',
    };
  }

  // A multilingual recommendation exists. Check whether the user's
  // current selection is already compatible — i.e. either matches
  // the recommendation or is itself a multilingual model.
  const currentModel = currentSelection
    ? models.find((m) => m.id === currentSelection) ?? null
    : null;
  const currentIsCompatible =
    currentModel !== null && currentModel.languages.includes('multi');

  return {
    recommendedId: multilingualId,
    compatible: currentIsCompatible,
    reason: currentIsCompatible ? 'multilingual_match' : 'selection_english_only',
  };
}
