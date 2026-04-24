# Utterbar — marketing asset pack (Remotion)

Twelve short videos and five press stills for the Utterbar marketing site, rendered deterministically from React + Remotion. Lives in its own npm workspace under `marketing/remotion/` so the shipping Tauri build is never touched by the asset pipeline.

## What's in the box

**Videos** (1920 × 1080, 30 fps, H.264 / CRF 18 / yuv420p, silent):

| id | seconds | purpose |
| -- | ------- | ------- |
| `hero-pill-loop` | 6 | homepage hero — the pill cycling through idle → listening → processing, loops |
| `how-it-works-walkthrough` | 12 | three-beat press/speak/land-in-app demo |
| `feature-press-speak-done` | 5 | tight state cycle under the "Press. Speak. Done." tagline |
| `feature-works-anywhere` | 10 | Gmail → Slack → Obsidian — same pill, three host apps |
| `feature-modes` | 8 | raw transcript + three mode outputs staggering in |
| `feature-system-audio` | 7 | meeting-call window alongside live transcript from system audio |
| `feature-vocabulary` | 8 | vocabulary + replacements grid with before-and-after sentence |
| `feature-local-privacy` | 8 | mic → Parakeet → app diagram with cloud path crossed out |
| `feature-file-transcription` | 8 | Files route dropzone + queue progressing to done |
| `feature-history-retranscribe` | 9 | reprocess an entry through Meeting mode; transcript rewrites |
| `feature-customise-bar` | 8 | Obsidian → Graphite → Paper theme flip with accent change |
| `onboarding-walkthrough` | 12 | welcome / privacy / first-dictation payoff inside the onboarding window |

Each video also renders a first-frame PNG poster at the same resolution for `<video poster="…">` markup.

**Stills** (2560 × 1600 PNG):

- `press-01-floating-bar-idle`
- `press-02-floating-bar-listening`
- `press-03-main-shell-dashboard`
- `press-04-vocabulary`
- `press-05-file-transcription`

All assets are catalogued with alt text, licence, and intended placement in [`asset-manifest.json`](./asset-manifest.json).

## Running it

```bash
# from marketing/remotion/
npm install          # resolves Remotion + self-hosted Instrument Sans / Geist Mono
npm start            # Remotion Studio, for live editing
npm run render       # batch-render every video + still into public/out/
```

Targeted renders (useful when iterating on a single composition):

```bash
node render.mjs --only=hero-pill-loop          # one asset
node render.mjs --kind=stills                  # only press stills
node render.mjs --force                        # re-render even if output exists
```

Outputs land in `public/out/`:

```
public/out/
  videos/<id>.mp4
  posters/<id>-poster.png
  stills/<id>.png
```

## Project layout

```
src/
  design/
    tokens.ts        # full Obsidian palette port — bg-0/1/2, fg-hi/fg/muted,
                     # accent/2/3, gradient wordmark, pill colours, radii,
                     # EASE curve, duration presets. Matches DESIGN_SYSTEM.md.
    fonts.ts         # side-effect imports for the @fontsource-variable packs
  brand/
    UtterbarLogoMark.tsx   # the pill logo — viewBox 120×60 pad, rx 30, 7-bar
                           # wave (x=44,54,64,84,94,104) + central tail
    UtterbarMiniMark.tsx   # wave-only variant for in-pill contexts
    UtterbarWordmark.tsx   # "Utter·bar" with gradient on the second word
    Waveform.tsx           # deterministic prn(i, t) → frame-identical reruns
    StatePill.tsx          # idle/warming/listening/processing/success/error
    FloatingBar.tsx        # the pill composite — matches src/routes/FloatingBar.tsx
  chrome/
    WindowsDesktop.tsx     # radial-gradient stand-in for the Windows wallpaper
                           # (no photo — avoids licensing)
    HostWindow.tsx         # browser / chat / editor / email / plain frames
    AppShell.tsx           # 248 px sidebar + nav-item rail, like the real app
  util/
    anim.ts                # tween / typewriter / pulse / bezierEasing helpers
    Scene.tsx              # AbsoluteFill wrapper + Callout primitive
  compositions/
    <one .tsx per asset>   # see asset-manifest.json for the full list
  Root.tsx                 # <Composition/> registry — one entry per asset
  index.ts                 # registerRoot(Root)
remotion.config.ts   # codec, crf, pixel format, image format defaults
render.mjs           # batch renderer — reads asset-manifest.json
asset-manifest.json  # per-asset metadata (duration, dims, alt, placement)
```

## Design rules baked in

- **Typography**: Instrument Sans (body) + Geist Mono (timers, chips). No other faces, no weights other than 300/400/500/700/800 in Sans and 400/500 in Mono.
- **Tokens**: everything routes through `design/tokens.ts`. Don't hardcode hex — if it's not a token yet, add it there and use the named export.
- **Cyan budget**: at most three accent-coloured elements per viewport (including the pill's gradient wordmark and any lit action tile). Violating this is how the bar stops feeling premium.
- **Pill geometry is canon**: viewBox 120 × 60 with a 2 px pad, rounded radius 30, bars at x=44, 54, 64, 84, 94, 104, central tail `M 74 10 A 2 2 0 0 1 78 10 V 50 L 74 44 Z`, 2 px white ring, hole-punch cx 20 cy 30 r 8. Don't tweak any of those numbers — every marketing surface is meant to read as "the same pill" as the app.
- **British English copy**: *customise*, *optimise*, *licence*, *colour*. Check anything new against this before committing.
- **No invented features**: every beat, every label, every route mirrors the real app. If a composition would need a feature that doesn't exist yet, open a separate task before shipping that frame.

## Renderer notes

- **Deterministic motion.** All animation is frame-driven via `util/anim.ts` — no `Math.random()`, no real-time clocks. Re-renders produce byte-identical MP4s.
- **Silent.** Every composition is visual-only. No audio track means the MP4s are also safe for autoplay embeds.
- **Self-hosted fonts.** `@fontsource-variable/instrument-sans` + `@fontsource-variable/geist-mono` ship the woff2 files inside `node_modules`; the bundler serves them, so rendering works with no network.
- **Windows-first chrome.** The `WindowsDesktop` + `HostWindow` helpers use Windows-style min/max/close glyphs + taskbar; macOS traffic lights are deliberately not available.

## Troubleshooting

- *"Cannot find composition id X"* — either the id is wrong or the composition isn't registered in `src/Root.tsx`. Every asset in `asset-manifest.json` must have a matching `<Composition id="…">` entry.
- *Fonts fall back to Arial in the output* — `src/design/fonts.ts` must be imported eagerly. It already is from `Root.tsx`; keep it that way.
- *Posters look different from the MP4 first frame* — check that the composition's frame 0 isn't behind an opacity tween. If the intended poster needs a later frame, tweak `frame` in `render.mjs` for that specific id.

## Licence

Utterbar internal. The stills themselves ship under CC BY with attribution — see `asset-manifest.json` per-still.
