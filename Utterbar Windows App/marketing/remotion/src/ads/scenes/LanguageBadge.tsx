/**
 * Cycling language badge — v2 hero-sized. Native-script sample text
 * is now the visual hero (90 px, was 44). Badge chip bigger too.
 */
import React from 'react';
import { tokens, fonts } from '../../design/tokens';

export type LangSpec = {
  code: string;
  native: string;
  sample: string;
  hue: number;
};

export const LANGS: LangSpec[] = [
  { code: 'EN', native: 'English',  sample: 'Send the deck before five.',     hue: 210 },
  { code: 'FR', native: 'Français', sample: 'Envoie le dossier avant cinq heures.', hue:   0 },
  { code: 'DE', native: 'Deutsch',  sample: 'Schick die Folien vor fünf.',    hue:  45 },
  { code: 'ES', native: 'Español',  sample: 'Envía la presentación antes de las cinco.', hue:  20 },
  { code: 'JA', native: '日本語',     sample: '5時前に資料を送って。',             hue: 350 },
  { code: 'AR', native: 'العربية',    sample: 'أرسل العرض قبل الخامسة.',          hue: 130 },
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
        gap: 50,
        opacity,
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 24,
          padding: '20px 32px',
          background: tokens.bg2,
          border: `1px solid ${tokens.border}`,
          borderRadius: 999,
        }}
      >
        <div
          style={{
            width: 40,
            height: 40,
            borderRadius: 999,
            background: `hsl(${lang.hue} 70% 55%)`,
            boxShadow: `0 0 20px hsl(${lang.hue} 70% 55% / 0.65)`,
          }}
        />
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: 28,
            color: tokens.muted,
            letterSpacing: '0.12em',
          }}
        >
          {lang.code}
        </div>
        <div style={{ width: 1, height: 32, background: tokens.border }} />
        <div style={{ color: tokens.fgHi, fontSize: 44, fontWeight: 600 }}>{lang.native}</div>
      </div>

      <div
        style={{
          fontSize: 90,
          color: tokens.fgHi,
          textAlign: 'center',
          maxWidth: 1000,
          lineHeight: 1.2,
          padding: '0 60px',
          letterSpacing: '-0.025em',
          fontWeight: 600,
          direction: lang.code === 'AR' ? 'rtl' : 'ltr',
        }}
      >
        {lang.sample}
      </div>
    </div>
  );
}
