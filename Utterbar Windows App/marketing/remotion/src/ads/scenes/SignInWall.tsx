/**
 * "Create your account to continue" sign-in wall — the friction every
 * other category app forces. Used in Ad11NoAccount as the contrast
 * surface against Utterbar's no-account-needed flow.
 *
 * Generic chrome — modal centred on a dim background. Three input
 * fields, a primary CTA, a "Continue with Google" row. None of it
 * functional; it's a still frame that the parent fades through.
 */
import React from 'react';
import { tokens, fonts } from '../../design/tokens';

type Props = {
  product: string;
};

export function SignInWall({ product }: Props) {
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        background: 'rgba(2, 4, 8, 0.78)',
        backdropFilter: 'blur(8px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          width: 720,
          background: tokens.bg1,
          borderRadius: 22,
          border: `1px solid ${tokens.border}`,
          padding: '40px 44px 36px 44px',
          display: 'flex',
          flexDirection: 'column',
          gap: 18,
          boxShadow: '0 30px 80px rgba(0,0,0,0.65)',
        }}
      >
        <div
          style={{
            color: tokens.muted,
            fontFamily: fonts.mono,
            fontSize: 14,
            letterSpacing: '0.12em',
            textTransform: 'uppercase',
          }}
        >
          {product}
        </div>
        <div style={{ color: tokens.fgHi, fontSize: 38, fontWeight: 600, letterSpacing: '-0.02em' }}>
          Create your account
        </div>
        <div style={{ color: tokens.muted, fontSize: 18, lineHeight: 1.4, marginBottom: 8 }}>
          Sign in or sign up to continue. We'll send a verification link to your email.
        </div>

        <Field placeholder="Work email" />
        <Field placeholder="Full name" />
        <Field placeholder="Company" />

        <div
          style={{
            marginTop: 8,
            padding: '18px 22px',
            background: tokens.fgDim,
            color: tokens.bg0,
            borderRadius: 12,
            fontSize: 22,
            fontWeight: 600,
            textAlign: 'center',
            opacity: 0.75,
          }}
        >
          Continue
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12, color: tokens.fgDim, fontSize: 14, margin: '8px 0' }}>
          <div style={{ flex: 1, height: 1, background: tokens.border }} />
          <div>or</div>
          <div style={{ flex: 1, height: 1, background: tokens.border }} />
        </div>

        <div
          style={{
            padding: '16px 22px',
            background: tokens.bg2,
            border: `1px solid ${tokens.border}`,
            borderRadius: 12,
            color: tokens.fg,
            fontSize: 18,
            textAlign: 'center',
          }}
        >
          Continue with Google
        </div>

        <div style={{ color: tokens.fgDim, fontSize: 13, lineHeight: 1.5, marginTop: 6 }}>
          By continuing you agree to the Terms and acknowledge the Privacy Policy.
        </div>
      </div>
    </div>
  );
}

function Field({ placeholder }: { placeholder: string }) {
  return (
    <div
      style={{
        padding: '16px 18px',
        background: tokens.bg0,
        border: `1px solid ${tokens.border}`,
        borderRadius: 12,
        color: tokens.fgDim,
        fontSize: 18,
      }}
    >
      {placeholder}
    </div>
  );
}
