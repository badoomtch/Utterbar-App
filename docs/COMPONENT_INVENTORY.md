# Utterbar — UI Component Inventory

Legend: **P** = primitive (shadcn-wrapped), **A** = atom, **M** = molecule, **O** = organism.

## Design-system primitives (P)

Button, IconButton, Input, Textarea, Select, Switch, Slider, Checkbox, RadioGroup, Tabs, Dialog, Drawer, Popover, Tooltip, Toast, DropdownMenu, ContextMenu, Separator, ScrollArea, Progress, Badge, Kbd, Card, Sheet, Command (palette).

All wrap shadcn/ui with Utterbar's token overrides.

## Atoms (A)

- `UtterbarLogoMark` — the compact glyph used in the bar.
- `UtterbarWordmark` — full wordmark, onboarding + about.
- `ModeGlyph` — 16 px lucide glyph with mode colour.
- `VUDot` — single-dot audio-active indicator.
- `StatePill` — loading / recording / processing / success / error.
- `ModelChip` — model id + language + backend (CPU/GPU) badge.
- `Timer` — monospace MM:SS, respects `prefers-reduced-motion`.
- `Hotkey` — renders a keybinding with `Kbd` primitives.
- `Waveform` — canvas waveform (see Organisms for the full bar variant).
- `SoundToggle` — icon + on/off sound-cue toggle.

## Molecules (M)

- `MetricCard` — value + label + delta + sparkline.
- `ModeRow` — icon · name · favourite star · context badges · shortcut.
- `HistoryRow` — timestamp · mode · app · snippet · duration · status.
- `DeviceRow` — device name · default badge · test button.
- `ModelRow` — name · size · languages · actions (install/remove/set-default).
- `ShortcutBinder` — label · current binding · "Change" action · conflict warning.
- `ContextBadge` — "Clipboard" / "Selection" / "App" indicator.
- `DropZone` — file-drag target with animated border.
- `SnippetDiff` — raw vs processed side-by-side or unified diff.
- `OnboardingStepHeader` — step indicator + title + subtitle.

## Organisms (O)

- `FloatingBar` — the signature bar. Composes `VUDot`, `Waveform`, `StatePill`, `ModeChip`, `IconButton`s. Owns its own Zustand slice.
- `FloatingBarMenu` — popover with mode list + quick actions.
- `OnboardingShell` — stepper wrapper with motion transitions.
- `SettingsShell` — left rail + routed panel.
- `ModeEditor` — name · icon · ASR · post-processing · context · formatting · auto-activation · shortcut.
- `HistoryDetail` — full history entry inspector with context payload.
- `StatsDashboard` — MetricCards + charts (Words/day, Minutes saved, Per-mode).
- `FileQueue` — list of `file_jobs` with live progress.
- `ModelManager` — installed + available models, download manager.
- `Diagnostics` — hardware, backend EP, log tail, reset actions.

## Charts

Recharts. Palette pulled from design tokens.

- `BarSparkline` — tiny 7-day bar chart used inside `MetricCard`.
- `ChartWordsPerDay` — bar + trend line, 30-day window.
- `ChartMinutesSaved` — stacked bar (per-mode).
- `ChartSpeakingSpeed` — line, 30-day window.

## Tokens

See `src/design/tokens.css`. Accent defaults to **Obsidian Teal** `#5FE3D6` with a graphite-to-obsidian surface ramp. User-selectable accents: Teal, Amber, Violet, Rose, Cyan, Lime.

## Motion

Spring presets defined in `src/design/motion.ts`:

- `pop` — mass 1, stiffness 420, damping 26
- `glide` — mass 1, stiffness 240, damping 22
- `settle` — mass 1, stiffness 140, damping 20

Bar transitions use `glide`; state-pill changes use `pop`; modal opens use `settle`.

## Sound cues

Files under `resources/sounds/`:
- `cue-start.ogg` — soft tick on record start
- `cue-stop.ogg` — low chime on record stop
- `cue-success.ogg` — brief confirmation blip
- `cue-error.ogg` — low thunk

All off by default; enabled from Settings → General.
