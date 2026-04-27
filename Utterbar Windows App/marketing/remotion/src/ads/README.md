# Social ads pack

Vertical 1080×1920 social ads (TikTok / Reels / Shorts), built on the
existing Remotion bundle's primitives. **Strictly additive** — this folder
adds files; it touches nothing under `src/`, `src/compositions/`, or
`Root.tsx`.

## What's here

```
ads/
  index.ts              separate Remotion entrypoint (registerRoot)
  AdsRoot.tsx           composition registry (12 compositions, all 1080×1920)
  render-ads.mjs        batch render to out/ads/<id>.mp4
  frame/                vertical-only framing primitives
    SocialFrame.tsx     bg + faux taskbar wrapper
    Captions.tsx        burned-in caption strip (essential for muted social)
    EndCard.tsx         wordmark + tagline + URL outro
    humanType.ts        burst-pause-burst typewriter (deterministic per seed)
  scenes/               vertical mock-app surfaces, brand-agnostic
    SlackVerticalThread.tsx
    AssistantChat.tsx
    NetworkMonitor.tsx
    LanguageBadge.tsx
    HotkeyBadge.tsx
    PriceLine.tsx
    SignInWall.tsx
  compositions/         the 12 ads
    Ad01StopTyping.tsx
    Ad02VoiceIsFaster.tsx
    Ad03LocalOnly.tsx
    Ad04TalksToAnyAI.tsx
    Ad05ThirtyLanguages.tsx
    Ad06HoldHotkey.tsx
    Ad07TeachVocabulary.tsx
    Ad08WindowsFirst.tsx
    Ad09Manifesto.tsx
    Ad10PriceOnce.tsx
    Ad11NoAccount.tsx
    Ad12FoundersNote.tsx
```

## The 12 ads

Each is a fresh concept — none of them mirror the existing landscape
`Feature*` compositions in `../compositions/`. They share only the design
tokens, font registration, and the `BarDriver` / `WindowsDesktop` /
`HostWindow` primitives.

| id | length | concept |
| --- | --- | --- |
| `ad-01-stop-typing` | 12s | rage-typing then deletion (the empathy hook nobody else does), then dictation |
| `ad-02-voice-is-faster` | 11s | split-vertical race against a wall-clock — voice finishes first, sits idle while typing grinds on |
| `ad-03-local-only` | 10s | live network-monitor widget pinned to **0 KB** while a sensitive sentence is dictated |
| `ad-04-talks-to-any-ai` | 13s | one chat surface, assistant cycles ChatGPT → Claude → Gemini → Copilot → "Your local model" |
| `ad-05-thirty-languages` | 12s | speed-cycle 6 languages in 10s, ends on an "and 24 more" chip strip |
| `ad-06-hold-hotkey` | 8s | the hotkey itself as protagonist — three press/release reps, third one transcribes |
| `ad-07-teach-vocabulary` | 12s | take 1 mishears "Anthropic" → teach chip → take 2 nails it (BEFORE/AFTER) |
| `ad-08-windows-first` | 10s | half-frame Mac (no app), half-frame Windows (the pill) — explicit positioning vs Superwhisper |
| `ad-09-manifesto` | 30s | brand film, kinetic typography over the slow-pulsing pill |
| `ad-10-price-once` | 9s | crossed-out £39.99/month → £39.99/year → **£39.99 once** in cyan |
| `ad-11-no-account` | 11s | three competitor sign-in walls flash by, then the pill with no friction |
| `ad-12-founders-note` | 20s | typewritten note from the maker, mono, founder voice register |

## Render

From `marketing/remotion/`:

```bash
# all 12
node src/ads/render-ads.mjs

# one
node src/ads/render-ads.mjs ad-03-local-only
```

Outputs go to `marketing/remotion/out/ads/<id>.mp4`.

## Studio (preview interactively)

```bash
npx remotion studio src/ads/index.ts
```

## Why this lives in a separate entrypoint

The existing `src/index.ts` → `src/Root.tsx` registers the landscape
marketing pack. Touching that to add 12 vertical compositions would:
1. Force a re-render of the parity stills (per CLAUDE.md §5.5).
2. Mix two unrelated render targets into one Remotion bundle.
3. Risk drift in any subsequent refactor of the existing comps.

Two entrypoints, two roots, zero shared mutable state. The shared
primitives (`tokens`, `BarDriver`, `WindowsDesktop`, `Cursor`,
`anim.ts`) stay imported from their original locations under `../`.

## Brand discipline

All 12 ads obey the BRIEF.md rules:
- British English throughout (vocab, no Americanisms in copy)
- £39.99 one-time, never £40 / $49 / $29
- No banned words (synergy, empower, revolutionise, AI-powered as noun, etc.)
- ≤3 cyan moments per frame (mostly: pill, end-card URL, one accent)
- Lockup never redrawn — `EndCard` uses `BarDriver` directly

## Adding a new ad

1. Add `compositions/AdNN<Slug>.tsx` exporting a default function.
2. Import it in `AdsRoot.tsx` and add a `<Composition id="ad-NN-<slug>" …/>`.
3. Add the id to `ALL_IDS` in `render-ads.mjs`.
4. Document it in this README's table.

## What this pack deliberately does NOT do

- **Does not include Cursor-driven shots.** The existing `util/Cursor.tsx`
  works great for the landscape pack, but every ad here is composed
  around the pill + a host surface, not a moving cursor. If we want a
  cursor-led ad later, it slots in as Ad13.
- **Does not bundle audio.** Remotion supports inlined audio tracks but
  voiceover, music, and SFX get added in DaVinci/CapCut at the edit
  stage. Each MP4 ships clean.
- **Does not rotate existing landscape comps.** Per the brief — these
  are 12 fresh concepts, not vertical re-renders of `FeatureTalkToAI`
  / `FeatureMultilingual` / `FeatureLocalPrivacy` / etc.
