/**
 * "Create your account to continue" sign-in wall — v2 modal scaled
 * up. Vertical-mobile means the modal IS the screen, not a small
 * overlay over a desktop. 920 wide, all field text 28-32 px.
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
        background: 'rgba(2, 4, 8, 0.82)',
        backdropFilter: 'blur(8px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontFamily: fonts.sans,
      }}
    >
      <div
        style={{
          width: 920,
          background: tokens.bg1,
          borderRadius: 28,
          border: `1px solid ${tokens.border}`,
          padding: '52px 56px 48px 56px',
          display: 'flex',
          flexDirection: 'column',
          gap: 24,
          boxShadow: '0 30px 80px rgba(0,0,0,0.7)',
        }}
      >
        <div
          style={{
            color: tokens.muted,
            fontFamily: fonts.mono,
            fontSize: 22,
            letterSpacing: '0.14em',
            textTransform: 'uppercase',
          }}
        >
          {product}
        </div>
        <div
          style={{
            color: tokens.fgHi,
            fontSize: 64,
            fontWeight: 700,
            letterSpacing: '-0.025em',
            lineHeight: 1.1,
          }}
        >
          Create your account
        </div>
        <div style={{ color: tokens.muted, fontSize: 28, lineHeight: 1.4, marginBottom: 8 }}>
          Sign in or sign up to continue.
        </div>

        <Field placeholder="Work email" />
        <Field placeholder="Full name" />
        <Field placeholder="Company" />

        <div
          style={{
            marginTop: 12,
            padding: '24px 28px',
            background: tokens.fgDim,
            color: tokens.bg0,
            borderRadius: 16,
            fontSize: 32,
            fontWeight: 700,
            textAlign: 'center',
            opacity: 0.78,
          }}
        >
          Continue
        </div>
      </div>
    </div>
  );
}

function Field({ placeholder }: { placeholder: string }) {
  return (
    <div
      style={{
        padding: '22px 24px',
        background: tokens.bg0,
        border: `1px solid ${tokens.border}`,
        borderRadius: 16,
        color: tokens.fgDim,
        fontSize: 28,
      }}
    >
      {placeholder}
    </div>
  );
}
