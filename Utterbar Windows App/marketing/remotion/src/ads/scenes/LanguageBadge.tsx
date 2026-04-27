/**
 * Cycling language badge — used in Ad05ThirtyLanguages. Shows the
 * currently-active language's flag glyph (rendered as a coloured
 * circle to dodge any flag-rendering politics) and the language
 * name in its native script.
 *
 * Caller passes index + the LANGS array; this just renders the active
 * one with a smooth crossfade against whatever was on screen before.
 * Crossfade is handled by mounting one badge per language stacked,
 * each with its own opacity from the parent.
 */
import React from 'react';
import { tokens, fonts } from '../../design/tokens';

export type LangSpec = {
  code: string;       // ISO 639-1, used for the small chip
  native: string;     // language name in its own script
  sample: string;     // a short transcription example in that language
  hue: number;        // 0-360, for the dot-flag
};

export const LANGS: LangSpec[] = [
  { code: 'EN', native: 'English',  sample: 'Send the deck before five.',     hue: 210 },
  { code: 'FR', native: 'Français', sample: 'Envoie le dossier avant cinq heures.', hue:   0 },
  { code: 'DE', native: 'Deutsch',  sample: 'Schick die Folien vor fünf.',    hue:  45 },
  { code: 'ES', native: 'Español',  sample: 'Envía la presentación antes de las cinco.', hue:  20 },
  { code: 'JA', native: '日本語',     sample: '5時前に資料を送って。',             hue: 350 },
  { code: 'AR', native: 'العربية',    sample: 'أرسل العرض قبل الساعة الخامسة.',    hue: 130 },
];

type Props = {
  lang: LangSpec;
  opacity: number;
};

export function LanguageBadge({ lang, opacity }: Props) {
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 28,
        opacity,
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 18,
          padding: '14px 22px',
          background: tokens.bg2,
          border: `1px solid ${tokens.border}`,
          borderRadius: 999,
        }}
      >
        <div
          style={{
            width: 28,
            height: 28,
            borderRadius: 999,
            background: `hsl(${lang.hue} 70% 55%)`,
            boxShadow: `0 0 16px hsl(${lang.hue} 70% 55% / 0.6)`,
          }}
        />
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 18,
            color: tokens.muted,
            letterSpacing: '0.1em',
          }}
        >
          {lang.code}
        </div>
        <div style={{ width: 1, height: 22, background: tokens.border }} />
        <div style={{ color: tokens.fgHi, fontSize: 28, fontWeight: 600 }}>{lang.native}</div>
      </div>

      <div
        style={{
          fontSize: 44,
          color: tokens.fgHi,
          textAlign: 'center',
          maxWidth: 920,
          lineHeight: 1.3,
          padding: '0 60px',
          letterSpacing: '-0.01em',
          direction: lang.code === 'AR' ? 'rtl' : 'ltr',
        }}
      >
        {lang.sample}
      </div>
    </div>
  );
}
