// Separate Remotion entrypoint for the social-ad pack. The marketing
// landscape compositions still live in ../index.ts and ../Root.tsx —
// the ads pack is strictly additive and shares only the design tokens,
// font registration, and the BarDriver / Cursor / chrome primitives.
//
// Render via:  pnpm remotion render src/ads/index.ts <ad-id> out/<file>.mp4
// Studio via:  pnpm remotion studio src/ads/index.ts
import { registerRoot } from 'remotion';
import { AdsRoot } from './AdsRoot';

registerRoot(AdsRoot);
