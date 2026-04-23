# Utterbar — Screen Map

Numbered surfaces, their entry points, and the primary interactions on each.

## F — Floating Bar (`/floating`)
Always-on-top, top-of-screen, movable horizontally.

| State | Content |
|-------|---------|
| F.0 Hidden | Off-screen / paused |
| F.1 Idle compact | Logo glyph · active mode glyph · tiny VU dot |
| F.2 Idle expanded (hover) | Logo · mode label · VU meter · quick menu (•••) |
| F.3 Loading | Spinner + "Warming Parakeet EN…" |
| F.4 Recording | Live waveform · timer · mode chip · Stop / Cancel |
| F.5 Processing | Progress shimmer · "Processing…" |
| F.6 Success | Check · "Inserted 84 words" (fades to idle after 1.2 s) |
| F.7 Error | Alert · terse reason · "View logs" |
| F.8 Mode picker | Popover list of modes, keyboard navigable |

## O — Onboarding (`/onboarding`)
Windowed, 880 × 620, non-resizable. Linear but skippable past step 2.

| Step | Screen |
|------|--------|
| O.1 | Welcome / brand reveal |
| O.2 | "Everything local, always" — privacy pitch |
| O.3 | Permission checks (mic permission dialog) |
| O.4 | Pick ASR model (Parakeet EN / Multi) + size disclosure |
| O.5 | Model download with progress (resumable) |
| O.6 | Pick microphone + live meter |
| O.7 | Record test clip ("Say something like…") + transcription result |
| O.8 | Typing-speed baseline (type a 30-word paragraph) |
| O.9 | Trigger style (Toggle / Hold / Push) |
| O.10 | Bind recording shortcut (live capture) |
| O.11 | Theme (Obsidian / Graphite / Paper) |
| O.12 | "Ready to dictate" — launch bar |

## D — Main window (`/`)
Left rail navigation. 1180 × 780 default, min 960 × 600.

| Route | Screen |
|-------|--------|
| D.1 `/dashboard` | Stats + recent activity + quick-start |
| D.2 `/history` | List · filters · detail drawer · Reprocess · Export |
| D.3 `/modes` | Mode list · editor · auto-activation table · context toggles |
| D.4 `/vocabulary` | Words table · replacements table · import/export |
| D.5 `/files` | Drop zone · queue list · per-item progress · export |
| D.6 `/settings/general` | Startup, theme, accent, motion, sound, language |
| D.7 `/settings/audio` | Input device, meter, preprocessing, test |
| D.8 `/settings/models` | Installed models · available · default-per-mode |
| D.9 `/settings/shortcuts` | Bind table · conflict view · mouse buttons |
| D.10 `/settings/text-output` | Paste vs keystrokes · clipboard restore · casing |
| D.11 `/settings/privacy` | Data paths · retention · clear-all · LLM toggle |
| D.12 `/settings/advanced` | Diagnostics · logs · experimental toggles · reset |
| D.13 `/settings/about` | Version · links · credits |

## T — Tray menu
- Open Utterbar
- Start Recording (⌥O or configured)
- Switch Mode ▸ (submenu built from enabled modes)
- Pause Shortcuts (checkbox)
- ──
- Quit

## Modals / drawers
- M.1 History detail drawer (right side, 520 px)
- M.2 Mode editor (modal, 720 × 640)
- M.3 Hotkey capture (inline popover during bind)
- M.4 Model download progress (inline on Models page; toast on floating bar)
- M.5 "Update available" toast (bottom-right of main window)

## Empty states
Each D-screen has a considered empty state. Examples:
- Dashboard empty: hero showing how to dictate + "Do a 10-second test" CTA.
- History empty: illustration + "Your dictations will appear here."
- Files empty: drop-zone with example file types.
