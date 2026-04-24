# /docs audit against Utterbar app v0.6.0 → v0.6.1

Generated 2026-04-23. All line references below are to the docs Blade files
under `resources/views/pages/docs/`. App references below use paths that
predate the monorepo consolidation — where an old path says `Orliq/` or
"sibling repo", read it as `Utterbar Windows App/` in the current layout.

## `index.blade.php`

OK — no changes needed.

## `getting-started.blade.php`

OK — no changes needed.

## `shortcuts.blade.php`

**Issue — default bindings are all marked "unbound", but the app ships real defaults.** Source of truth: `src-tauri/src/domain/settings.rs`, `impl Default for ShortcutSettings` at lines 484–536.

The actual shipped defaults are:

| Action              | Default         | Trigger style |
| ------------------- | --------------- | ------------- |
| `ToggleRecording`   | `Ctrl+Space`    | Toggle        |
| `PushToTalk`        | `Ctrl+Alt+Space`| PushToTalk    |
| `HoldToTalk`        | `F6`            | HoldToTalk    |
| `CancelRecording`   | `Ctrl+Alt+X`    | Toggle        |
| `CycleMode`         | `Ctrl+Alt+M`    | Toggle        |
| `OpenSettings`      | `Ctrl+Alt+,`    | Toggle        |
| `OpenHistory`       | `Ctrl+Alt+H`    | Toggle        |

Diffs:

1. **LINE ~17 (PushToTalk row):** currently claims "unbound". Change to display `Ctrl+Alt+Space`.
2. **LINE ~18 (HoldToTalk row):** currently claims "unbound". Change to `F6`.
3. **LINE ~19 (CancelRecording row):** "unbound" → `Ctrl+Alt+X`.
4. **LINE ~20 (CycleMode row):** "unbound" → `Ctrl+Alt+M`.
5. **LINE ~21 (OpenSettings row):** "unbound" → `Ctrl+Alt+,`.
6. **LINE ~22 (OpenHistory row):** "unbound" → `Ctrl+Alt+H`.
7. **Missing row:** `ToggleRecording` with default `Ctrl+Space` is not listed at all. Add it as the first row — it's the most-used shortcut and matches the "Ctrl+Space to dictate" convention shared with Wispr Flow / Dragon, which helps muscle-memory migrators.

Narrative beneath the table should also be softened: the current "nothing is bound by default — you choose" pitch is no longer true. Replace with "Utterbar ships sensible defaults you can rebind any time from Settings → Shortcuts."

## `modes.blade.php`

**Issue — docs list six built-in modes but the app ships three.** Source of truth: `src-tauri/src/services/modes.rs` (`builtin_modes()` function), which currently seeds only `voice-to-text`, `message`, `meeting`.

Diffs:

1. **LINE ~7 ("six built-in modes"):** change to "three built-in modes."
2. **LINES ~18–31 (modes table):** remove the `Email`, `Note`, and `Super` rows. Keep `Voice-to-text`, `Message`, `Meeting`.
3. **LLM post-processing paragraph:** keep as-is if it's framed as "coming in 0.7"; otherwise soften to future tense. The llama-cpp dependency is behind an optional feature flag (`Cargo.toml` `[features] llama = ["dep:llama-cpp-2"]`) and is not default-enabled, so any UI copy implying it's live today is wrong.

## `vocabulary.blade.php`

Mostly OK — the vocabulary feature exists and the Free/Pro limits match. One issue:

1. **LINE ~19 ("Team: shared lists"):** there is no Team tier in the app. Pricing is Free vs Pro one-time. Either remove the row or prefix it with "Coming later:". Leaving it reads as a phantom tier users can't find.

## `privacy-controls.blade.php`

OK — the three toggles described (clipboard capture, active-window context, selection capture) match `PrivacySettings` in `src-tauri/src/domain/settings.rs`. No changes needed.

## `troubleshooting.blade.php`

1. **LINE ~11 (pill visibility toggle hotkey):** copy claims `Ctrl+Alt+\` toggles pill visibility. No such binding exists in `ShortcutSettings` defaults or the hotkeys service. Either remove the claim or replace with the correct instruction (Settings → Floating Bar → Visibility, which is a preference, not a hotkey). Likely safer to remove — users were searching for a hotkey that doesn't exist.

2. **Add a v0.6.1 section — "Update didn't install."** New in 0.6.1: the in-app updater (see `Utterbar Windows App/src-tauri/src/commands/updater.rs`). Suggested copy:

   > **The update didn't install.** The updater downloads the new installer to `%TEMP%\Utterbar-Setup-<version>.exe`, verifies its SHA-256, and silently replaces the app. If it fails: (1) Try **Settings → General → Check now** to re-fetch the update. (2) If that keeps failing, grab the installer directly from the Downloads page — we use the same binary. (3) SmartScreen may have quarantined the downloaded `.exe`; right-click → Properties → Unblock.

## `faq.blade.php`

OK — the privacy, platform, offline, export-history, and commercial-use answers all match reality.

1. **Optional addition:** add a "How do I get updates?" entry mentioning the new in-app auto-update path (v0.6.1+): toast on launch when an update is available, plus Settings → General → "Check now" for manual checks. Currently the docs don't mention the updater at all and users might assume they need to re-download manually every time.

## Cross-cutting

- **Version-sensitive copy:** no explicit "v0.5" / "v0.6.0" strings found in the docs directory, so nothing to age-adjust.
- **Screenshots:** not audited here (I only scanned text). Worth a manual pass when the redesigned History page (0.6.1) and new Custom-accent swatch ship.
- **Pricing:** $29 one-time for Pro. The docs don't hardcode a price, which is correct — leave it to `pages/pricing.blade.php`.
- **Platform:** Windows-only is correct in the FAQ. No macOS/Linux promises to delete.

## Priority

Land in this order if you're time-boxing:

1. `shortcuts.blade.php` — 100% wrong today, biggest user-facing hit.
2. `modes.blade.php` — claims three features that don't exist, which creates "is this thing broken?" support tickets.
3. `vocabulary.blade.php` — one phantom tier row.
4. `troubleshooting.blade.php` — add the updater section, drop the nonexistent pill-visibility hotkey.
5. `faq.blade.php` — optional updater entry.
