import { describe, expect, it } from 'vitest';

import {
  modelSupportsLanguage,
  recommendModelForLanguage,
  type RecommendationModel,
} from './onboarding_model_recommendation';

// Fixtures mirror the real catalogue shape (services/models.rs):
// Parakeet advertises ['en']; Whisper variants advertise ['multi'].
const PARAKEET: RecommendationModel = {
  id: 'sherpa-parakeet-tdt-0.6b-v3-int8',
  languages: ['en'],
  installed: false,
  recommended: true,
};
const WHISPER_SMALL: RecommendationModel = {
  id: 'sherpa-whisper-small',
  languages: ['multi'],
  installed: false,
  recommended: false,
};
const WHISPER_MEDIUM: RecommendationModel = {
  id: 'sherpa-whisper-medium',
  languages: ['multi'],
  installed: false,
  recommended: false,
};

describe('modelSupportsLanguage', () => {
  it('lets every model serve auto-detect dictation', () => {
    expect(modelSupportsLanguage(PARAKEET, 'auto')).toBe(true);
    expect(modelSupportsLanguage(WHISPER_SMALL, 'auto')).toBe(true);
  });

  it('lets every model serve English dictation', () => {
    expect(modelSupportsLanguage(PARAKEET, 'en')).toBe(true);
    expect(modelSupportsLanguage(WHISPER_SMALL, 'en')).toBe(true);
  });

  it('rejects an English-only model for non-English dictation', () => {
    expect(modelSupportsLanguage(PARAKEET, 'fr')).toBe(false);
    expect(modelSupportsLanguage(PARAKEET, 'es')).toBe(false);
    expect(modelSupportsLanguage(PARAKEET, 'ja')).toBe(false);
  });

  it('accepts a multilingual model for non-English dictation', () => {
    expect(modelSupportsLanguage(WHISPER_SMALL, 'fr')).toBe(true);
    expect(modelSupportsLanguage(WHISPER_MEDIUM, 'es')).toBe(true);
  });
});

describe('recommendModelForLanguage', () => {
  describe("dictation language 'en' (existing behaviour)", () => {
    it('uses the existing default-pick (installed > recommended > first) and reports default reason', () => {
      const out = recommendModelForLanguage({
        models: [PARAKEET, WHISPER_SMALL],
        currentSelection: null,
        dictationLanguage: 'en',
      });
      expect(out.recommendedId).toBe(PARAKEET.id);
      expect(out.compatible).toBe(true);
      expect(out.reason).toBe('default');
    });

    it('prefers an installed model over the catalogue-recommended one', () => {
      const out = recommendModelForLanguage({
        models: [
          { ...PARAKEET, installed: false, recommended: true },
          { ...WHISPER_SMALL, installed: true, recommended: false },
        ],
        currentSelection: null,
        dictationLanguage: 'en',
      });
      expect(out.recommendedId).toBe(WHISPER_SMALL.id);
      expect(out.reason).toBe('default');
    });
  });

  describe("dictation language 'auto' (existing behaviour)", () => {
    it('uses the existing default-pick and reports default reason', () => {
      const out = recommendModelForLanguage({
        models: [PARAKEET, WHISPER_SMALL],
        currentSelection: null,
        dictationLanguage: 'auto',
      });
      expect(out.recommendedId).toBe(PARAKEET.id);
      expect(out.reason).toBe('default');
    });
  });

  describe('non-English dictation languages', () => {
    it('recommends a multilingual model for French', () => {
      const out = recommendModelForLanguage({
        models: [PARAKEET, WHISPER_SMALL, WHISPER_MEDIUM],
        currentSelection: null,
        dictationLanguage: 'fr',
      });
      expect(out.recommendedId).toBe(WHISPER_SMALL.id);
      expect(out.reason).toBe('selection_english_only');
    });

    it('skips English-only models even when they are catalogue-recommended', () => {
      const out = recommendModelForLanguage({
        models: [
          { ...PARAKEET, recommended: true },
          { ...WHISPER_SMALL, recommended: false },
        ],
        currentSelection: null,
        dictationLanguage: 'es',
      });
      expect(out.recommendedId).toBe(WHISPER_SMALL.id);
      expect(out.recommendedId).not.toBe(PARAKEET.id);
    });

    it('marks the current selection compatible when it is already multilingual', () => {
      const out = recommendModelForLanguage({
        models: [PARAKEET, WHISPER_SMALL, WHISPER_MEDIUM],
        currentSelection: WHISPER_MEDIUM.id,
        dictationLanguage: 'de',
      });
      expect(out.compatible).toBe(true);
      expect(out.reason).toBe('multilingual_match');
    });

    it('marks the current selection incompatible when it is English-only', () => {
      const out = recommendModelForLanguage({
        models: [PARAKEET, WHISPER_SMALL],
        currentSelection: PARAKEET.id,
        dictationLanguage: 'ja',
      });
      expect(out.compatible).toBe(false);
      expect(out.reason).toBe('selection_english_only');
      // The recommendation still names the multilingual fallback so
      // the UI can offer a one-click switch.
      expect(out.recommendedId).toBe(WHISPER_SMALL.id);
    });

    it('returns the no-multilingual fallback when no multi model is in the catalogue', () => {
      const out = recommendModelForLanguage({
        models: [PARAKEET],
        currentSelection: PARAKEET.id,
        dictationLanguage: 'fr',
      });
      expect(out.recommendedId).toBeNull();
      expect(out.compatible).toBe(false);
      expect(out.reason).toBe('no_multilingual_available');
    });

    it('prefers an installed multilingual model over a not-installed one', () => {
      const out = recommendModelForLanguage({
        models: [
          { ...WHISPER_SMALL, installed: false, recommended: true },
          { ...WHISPER_MEDIUM, installed: true, recommended: false },
        ],
        currentSelection: null,
        dictationLanguage: 'pl',
      });
      // Same installed > recommended > first priority as the default
      // branch, just filtered to multilingual models.
      expect(out.recommendedId).toBe(WHISPER_MEDIUM.id);
    });
  });

  it('handles an empty catalogue without throwing', () => {
    const out = recommendModelForLanguage({
      models: [],
      currentSelection: null,
      dictationLanguage: 'en',
    });
    expect(out.recommendedId).toBeNull();
    expect(out.reason).toBe('default');
  });

  it('handles an empty catalogue with non-English dictation', () => {
    const out = recommendModelForLanguage({
      models: [],
      currentSelection: null,
      dictationLanguage: 'fr',
    });
    expect(out.recommendedId).toBeNull();
    expect(out.reason).toBe('no_multilingual_available');
  });
});
