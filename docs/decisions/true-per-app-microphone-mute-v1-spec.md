# Teams + Discord Mic-Mute v1 Spec (experimental)

**Branch 1 of the four-branch plan in [true-per-app-microphone-mute.md](true-per-app-microphone-mute.md).**

**Status: DRAFT, awaiting Jay sign-off on every literal string below.** Branch 2 (Cargo feature + Rust prototype) does NOT start until Jay has read this file end-to-end and confirmed the copy lands as written. Reversibility is the point of this branch: if any string reads wrong, edit it here and re-push before any code work begins.

**Revision note (2026-05-03):** the original v1 spec was Teams-only; Jay revised the scope on the same day to ship Teams AND Discord together in v1 with per-app toggles and per-app user-configurable mute hotkeys. The parent decision doc Q2 has been updated to match. This file replaces the Teams-only spec.

---

## Scope reminder

In v1: Teams AND Discord. Per-app toggles, per-app user-configurable mute hotkey field (each defaults to that app's stock default of `Ctrl+Shift+M` but the user can rebind to match what they have set in the call app's own settings). Mechanism: send the per-app mute combo to that app's window via a new background-window-targeting `SendInput` wrapper. No UI Automation. Verification: NONE; every dictation that engaged a mic-mute toggle surfaces an always-on banner naming the specific app and asking the user to confirm. Restore-failure path: persistent banner on the floating bar plus a Windows tray balloon, both load-bearing because the failure mode is silent. State persisted to disk under `%APPDATA%\Utterbar\` per app so a crashed-and-relaunched Utterbar can attempt restore on next launch. Both toggles ship off by default in Settings, Advanced, Experimental.

NOT in v1: Zoom, Webex (each is a separate iteration with its own beta soak); UI Automation read-back of the mute button state (deliberately deferred so we don't ship false confidence); any change to the OS-level capture endpoint mute (forbidden by the parent decision doc, would also mute Utterbar's own ASR); auto-on by default; bundling into a stable release without an explicit Jay-confirmed soak window; bare-letter hotkey binds (the input requires at least one modifier so a user cannot bind to `M` and accidentally mute every time they type).

The parent decision doc [true-per-app-microphone-mute.md](true-per-app-microphone-mute.md) carries the why for each of those constraints.

---

## Settings, Advanced, Experimental: literal copy

The toggles live in a new `Experimental` card on the existing `Settings, Advanced` page. Same `Card` + `Row` + `Toggle` primitives the rest of Settings uses; the eyebrow is the new bit. Tone matches the v0.17.0 conferencing-output-mute description on `Settings, Privacy, Media`: second person, plain English, honest about scope, points at the call app's own button as the trustworthy fallback.

**Card eyebrow:** `EXPERIMENTAL`

**Card title:** `Mute your microphone in conferencing apps while recording`

**Card description (single paragraph above the per-app rows):**

> Experimental. This may fail silently. When you start a dictation, Utterbar sends each enabled app's mute hotkey to its window so the microphone there is muted, then sends it again when you stop to unmute. Each app needs you to confirm what its mute hotkey is, because both Teams and Discord let you rebind it. The defaults below match each app's out-of-the-box shortcut. After every dictation, a banner asks you to confirm the mute state in the relevant app, because Utterbar cannot read either app's actual mute state and shouldn't pretend it can. If this ever misbehaves, turn off the relevant toggle and use the call app's own mute button.

**Per-app sub-row layout** (one block per supported app, both visible whether or not the toggle is on):

```
[Toggle] Microsoft Teams                                 [On / Off]
         Mute hotkey:  [ Ctrl + Shift + M ]   [ Re-record ]
         If you've changed this in Teams, click "Re-record"
         and press your current Toggle Mute shortcut.

[Toggle] Discord                                         [On / Off]
         Mute hotkey:  [ Ctrl + Shift + M ]   [ Re-record ]
         If you've changed this in Discord, click "Re-record"
         and press your current Toggle Mute keybind.
```

**Toggle label per row:** the literal app name (`Microsoft Teams`, `Discord`). No "Mute" prefix; the card title already says what the toggles do.

**Hotkey-capture input:** click-to-record affordance, same primitive used in `Settings, Shortcuts` for Utterbar's own hotkeys. Display shows the captured combo as `Ctrl + Shift + M` (plus-separated, lowercase modifiers capitalised). Default value: `Ctrl + Shift + M` for both apps. Re-record button label: `Re-record`. While recording, the field changes to read `Press your hotkey...` and waits for the next key combo. Empty / cleared state shows the default again with a small `Default` chip.

**Per-app helper line under each hotkey input:** the second sentence of each row in the layout above ("If you've changed this in Teams..." / "If you've changed this in Discord..."), pointing the user at the right Settings location inside that app to look up their actual binding (Teams: Settings, Keyboard shortcuts, Toggle mute. Discord: User Settings, Keybinds, Toggle Mute).

**Validation rules for the hotkey input:**

- Must include at least one modifier (`Ctrl`, `Shift`, `Alt`, or `Win`). A bare letter or function key is rejected with the inline error: `Pick a hotkey with at least one modifier (Ctrl, Shift, Alt, or Win) so you don't trigger it by accident while typing.`
- Reject combos that match Utterbar's own global hotkeys (the recording toggle, mode-cycle, etc., resolved at validation time from `Settings, Shortcuts`). Inline error: `That combo is already bound to Utterbar's <hotkey name>. Pick a different one.`
- Reject duplicates between the two app rows (Teams and Discord cannot share a hotkey, because we'd send the same keystroke to both windows). Inline error: `Teams and Discord cannot share the same hotkey. Pick a different one for one of them.`
- Otherwise: accept any combo the OS will deliver, including F-keys and exotic key letters. Exclude mouse buttons; `SendInput` for mouse buttons is a different code path and out of scope.

**Supported apps subtitle:** `Supported apps`

**Matrix entries (two rows for v1):**

| App | Status | Tested against |
|---|---|---|
| Microsoft Teams (classic and new Teams) | `Experimental` | Last verified 2026-05-03 against Teams classic 1.7.x and new Teams 24.x |
| Discord (stable, PTB, Canary) | `Experimental` | Last verified 2026-05-03 against Discord stable 1.0.90xx |

(The `Last verified` dates get bumped on every quarterly manual re-check per the testing rules in the parent decision doc. The "broken on version X" disabled state shape is documented in section "What Branch 2 will do" below; v1 does not implement detection, only the matrix UI.)

**Empty-state copy for each per-app row when that app is not detected on this machine:**

> Microsoft Teams is not running or installed on this machine. Install Teams or open it once before this toggle becomes available.

> Discord is not running or installed on this machine. Install Discord or open it once before this toggle becomes available.

(Detection is a process-list scan for `Teams.exe` / `ms-teams.exe` for the Teams row, and `Discord.exe` / `DiscordPTB.exe` / `DiscordCanary.exe` for the Discord row; happens at Settings page open, not in real time. If only one app is detected, the other row shows its empty state but stays in the card.)

---

## Floating-bar banner: literal copy

Banners render inline at the top of the floating bar, above the existing recording chrome, in the same width as the bar itself. Two variants share a layout but differ in colour, dwell, and dismiss affordance.

If both Teams and Discord were targeted in the same dictation (both toggles on, both apps running), the success banner names them with comma-and: `Tried to unmute you in Teams and Discord. Please confirm in both.` The failure banner names whichever specific apps had a restore problem: `Utterbar may have left you muted in Teams. Open Teams and check the mic icon.` If both failed: `Utterbar may have left you muted in Teams and Discord. Open both apps and check the mic icons.`

**Active-recording sub-line during dictation (only when at least one mic-mute toggle is on AND that app is running):**

> Tried to mute Teams.

> Tried to mute Discord.

> Tried to mute Teams and Discord.

These sit in the same line as the existing mode chip / timer; small, neutral colour, no dismiss affordance. Disappear when dictation ends and are replaced by one of the banners below. Pick whichever one matches the apps Utterbar actually attempted to mute (Utterbar may have skipped one because that app was not running at dictation start).

**Success banner shown after dictation stop (auto-dismisses after 5 seconds):**

> Tried to unmute you in Teams. Please confirm in Teams.

> Tried to unmute you in Discord. Please confirm in Discord.

> Tried to unmute you in Teams and Discord. Please confirm in both.

Subdued colour, no border emphasis, dismiss affordance is `Got it`. The banner says "tried to" rather than "did" because we genuinely cannot verify the keystroke landed; "tried" is honest and the user is the verifier.

**Restore-failure banner that does NOT auto-dismiss (sticks until clicked):**

> Utterbar may have left you muted in Teams. Open Teams and check the mic icon.

> Utterbar may have left you muted in Discord. Open Discord and check the mic icon.

> Utterbar may have left you muted in Teams and Discord. Open both apps and check the mic icons.

Loud colour (uses the same accent the existing recording-state pill uses for attention but should not feel celebratory; the brief is "noticeable, not alarming"). Dismiss affordance is `Confirmed`, deliberately a verb the user has to mentally enact, not a passive `OK`. Clicking `Confirmed` clears the banner and writes nothing to disk; this is a UI ack only.

**Why two affordances rather than one shared `OK`:** the verb on the failure case forces the user to perform the check before dismissing. Reading `OK` and clicking it is autopilot. Reading `Confirmed` and clicking it is at least one cognitive beat where the user looks at the relevant app's window. That beat is the entire value of the always-on banner approach over a silent failure.

---

## System-tray balloon notification: literal copy

Fires alongside the restore-failure banner only, never on the success path (a tray balloon every dictation would be noise; the failure case is the one we want to escape the floating bar). Standard Windows toast.

**Title (under 64 chars):**

> Utterbar may have left you muted in Teams

> Utterbar may have left you muted in Discord

> Utterbar may have left you muted in Teams and Discord

**Body (under 256 chars), per case:**

> Open Teams and check the mic icon. Click here to bring Utterbar to the front.

> Open Discord and check the mic icon. Click here to bring Utterbar to the front.

> Open Teams and Discord and check the mic icons. Click here to bring Utterbar to the front.

**Click action:** brings the floating bar to the foreground so the user sees the persistent banner. Does not dismiss the banner; only navigates to it.

---

## Status pill copy on the bar

The existing recording-stop status pill (the small pill that briefly shows things like "Inserted") gets two new states for mic-mute toggle-on dictations. Brief, single-line, render in the existing pill style. The pill names the apps to disambiguate the multi-app case from the single-app case at a glance.

| State | Pill copy |
|---|---|
| Restore reported success, single app | `Mute restored in Teams` (or `Mute restored in Discord`) |
| Restore reported success, both apps | `Mute restored in Teams and Discord` |
| Restore reported failure, single app | `Mute restore failed in Teams` (or `Mute restore failed in Discord`) |
| Restore reported failure, mixed (one succeeded, one failed) | `Mute restore failed in Discord` (always names the failure; success is implicit on the other app) |
| Restore reported failure, both | `Mute restore failed in Teams and Discord` |

Success pills use the existing default dwell (a few seconds). Failure pills persist until the floating-bar banner is dismissed.

`Mute restored` is honest because the success case is "we sent the unmute keystroke and `SendInput` returned no error", which is what we know, no more; the always-on banner already softens this from a guarantee to a request to confirm. `Mute restore failed` is shown when `SendInput` returned an error, the relevant app's process is gone, or the targeting wrapper couldn't find a window for that app.

---

## Persistence schema for crash-restore

When any mic-mute toggle is on AND that app is running AND a dictation starts, Utterbar writes per-app entries to `mic_mute_state.json` under `%APPDATA%\Utterbar\` recording the fact that we sent a mute keystroke to each targeted app. Per-app entries are deleted as each app's unmute keystroke succeeds; the file as a whole is deleted when both apps' entries are gone. If Utterbar crashes between mute and unmute, the file persists; on next launch, Utterbar reads it, sends an unmute keystroke to each running matching app, and surfaces the restore-failure banner naming whichever apps had outstanding entries (regardless of the keystroke result, because the user has been muted unknowingly for some interval and deserves to be told).

**Filename:** `%APPDATA%\Utterbar\mic_mute_state.json`

**Schema (v1):**

```json
{
  "schema": 1,
  "session_id": "01HXYZ12345ABCDEFGHIJKLMNO",
  "applied_at_utc": "2026-05-03T22:00:00Z",
  "apps": [
    {
      "app": "teams",
      "previous_mute_state": "unknown"
    },
    {
      "app": "discord",
      "previous_mute_state": "unknown"
    }
  ]
}
```

**Field rules:**

- `schema`: integer. Currently `1`. Bumped if the file shape ever changes; older schemas are deleted-and-ignored on next launch (do not attempt restore against an unknown schema).
- `session_id`: 26-character ULID held in memory for the dictation session. One ULID per dictation, not per app. Used to correlate log lines without storing any user-identifying info. Local-only; never sent anywhere; the REJECTED-analytics decision in CLAUDE.md §5.1 forbids any wire transport of this or any other field.
- `applied_at_utc`: ISO 8601 UTC string with seconds precision. One timestamp for the whole dictation, not per app (apps are muted within milliseconds of each other). Used for log lines and the "if this is older than 24h, delete and ignore" sanity gate.
- `apps`: array of per-app entries. Empty array is treated as no-state (same as a missing file). Each entry:
  - `app`: closed enum, currently `"teams"` or `"discord"`. Reserved values: `"zoom"`, `"webex"` for the staged-iteration future.
  - `previous_mute_state`: closed enum, one of `"unknown"`, `"muted"`, `"unmuted"`. v1 always writes `"unknown"` because we do not read any app's mute state. Reserved for a future v2 that uses UIA to read mute state at apply time; the field exists now so a v2 desktop reading a v1 file gets a defined value rather than an absent one.

**Lifecycle:**

1. Per-app toggle is on AND that app is running, user starts a dictation: write the file with current UTC plus an entry per targeted app, then send each app's mute keystroke (parallel or serial; Branch 2 finding).
2. Dictation ends normally: send unmute keystroke to each app whose entry is still present. For each app whose `SendInput` returned no error, remove that app's entry from the file. When the `apps` array becomes empty, delete the file. If any app's `SendInput` errored, keep that entry and surface the restore-failure banner naming those apps.
3. Utterbar crashes between (1) and (2): file persists with one or more entries.
4. Utterbar launches: if `mic_mute_state.json` exists and `applied_at_utc` is within the last 24 hours, attempt restore for each entry (send unmute keystroke to any running matching app process) and surface the restore-failure banner naming all entries regardless of keystroke outcome. Delete the file. If the file is older than 24 hours, delete and ignore.
5. Either toggle is turned off while a dictation is in flight: the in-flight dictation finishes its normal lifecycle (the unmute keystroke for that app still sends, the entry still gets cleaned up). Subsequent dictations skip the mute path for that app entirely.

The "older than 24 hours" sanity gate is deliberate. The whole feature is about catching short-window crashes; a stale file from last week is not the failure mode the persistence layer is for.

---

## Onboarding behaviour

This feature is NOT on the onboarding path. New users do not see either toggle during first-run, and the onboarding script does not mention Teams mute, Discord mute, or experimental features. Discovery is by Settings search (typing `teams mute`, `discord mute`, `mic mute`, `microphone`, `experimental`, all of which should resolve to this Settings card via the Settings search v2 alias map shipped in v0.18.1) or by browsing Settings, Advanced manually.

This is deliberate. Onboarding sets first-impression promises; an experimental feature with a known silent-failure mode is not the right shape for that surface. A future session adding an onboarding step for this is explicitly out of scope. Settings, Advanced, Experimental is also the placement in the parent decision doc's Q8 sign-off; do not reuse the v0.17.0 output-mute Privacy card for these toggles.

---

## Release-notes paragraph (DRAFT for Branch 4)

Pre-written so Jay can read the user-facing copy now rather than at release time. Will be lifted into the v0.18.2 (or whichever) beta release notes verbatim once Branch 3 ships. Voice rules per CLAUDE.md §5.8: second person, British English, no source-code acronyms, no em dashes, no internal-mechanism jargon.

> ### Mute your microphone in Teams and Discord while you dictate (experimental, beta only)
>
> Two new toggles in Settings, Advanced, Experimental, both off by default. When you turn one on, Utterbar will try to mute your microphone inside that app whenever you start a dictation, then unmute it when you stop. The point is to stop your dictation bleeding into a live call when you're talking to another window.
>
> This is experimental. Both Teams and Discord let you rebind the mute hotkey, so each toggle has a small input where you tell Utterbar what your current mute hotkey is for that app. The defaults match each app's out-of-the-box shortcut, but if you've changed yours, click "Re-record" and press your current shortcut so Utterbar sends the right keys.
>
> Utterbar cannot read either app's actual mute state, so after every dictation a banner appears on the floating bar asking you to confirm the mute state. The banner is not a glitch; it's the honest version of "we tried to unmute you, please look at the mic icon to make sure". If something goes wrong, a more visible banner and a Windows tray notification will tell you which app you may have been left muted in, and Utterbar will try to unmute on its next launch if it crashes mid-dictation.
>
> Teams and Discord only in this release. Zoom and Webex are not supported. Both apps update regularly; this feature may stop working when an app changes before Utterbar does. If it stops working, turn off the relevant toggle and use the call app's own mute button directly until the next Utterbar release.
>
> What this does not change: the existing v0.17.0 setting that mutes what you HEAR from conferencing apps (Settings, Privacy, Media) is unchanged and continues to work as before. Microphone capture for your own dictation, audio routing, the speech models, and everything else are unaffected.

---

## Telemetry and logging stance

Failures are logged locally to the existing app log under `%APPDATA%\Utterbar\logs\`. Log lines record the app name (`teams` or `discord`), the action attempted (`mute_apply` / `mute_restore`), the result code from `SendInput`, and the timeout if any. They do NOT record user transcripts, focused-window titles, clipboard contents, vocabulary entries, the user's actual hotkey combo (only that an apply/restore happened, not the literal modifiers), or any other secret listed in the parent decision doc's "Banned fields" guidance. The log lines are local-only; no remote shipping, no analytics call, no opt-in fallback.

This stance is pinned by the REJECTED opt-in analytics decision in CLAUDE.md §5.1. Future sessions: do not propose adding remote shipping of mic-mute logs, even as a debugging aid, even opt-in. If a debugging aid is genuinely needed, the existing Settings, Advanced, Diagnostics, Export flow already produces a local zip the user can attach to an email; that is the supported path.

---

## What Branch 2 will do (not in this branch)

Brief summary so the next prompt has a head start; expand into a real implementation prompt when Branch 1 amendment merges.

- Add Cargo feature `experimental_mic_mute` to `Utterbar Windows App/src-tauri/Cargo.toml`. Off by default in release builds.
- Add a new module at `Utterbar Windows App/src-tauri/src/platform/windows/window_send_input.rs` that wraps `SendInput` to target a window resolved by process name (Teams: `Teams.exe` / `ms-teams.exe`; Discord: `Discord.exe` / `DiscordPTB.exe` / `DiscordCanary.exe`), without changing the foreground IF that proves possible; if focus-targeting is required to deliver the keystroke reliably (a real Windows API constraint), record the focus-flicker behaviour as a Branch 2 finding and revisit the design before Branch 3. The existing `text_inject.rs` is focused-window only; this is a deliberately separate module so the existing text-injection path is not destabilised.
- Add Rust unit tests (manual-run only, gated by `cfg(feature = "experimental_mic_mute")` plus `#[ignore]` so they do not run on `cargo test`) that exercise the new wrapper against a real running Teams instance AND a real running Discord instance on Jay's machine and prints the result.
- The wrapper takes the target process-name list and the keystroke combo as parameters, so the same code path serves both apps with no per-app branching beyond which process names to look for.
- No Settings UI, no toggle field, no banner, no persistence file, no hotkey-capture input in this branch. Pure runtime prototype.
- Do not flip the `experimental_mic_mute` feature on for any release build in Branch 2.
- Findings (P50 latency of the keystroke send, observed Teams classic vs new Teams behaviour, Discord stable vs PTB vs Canary behaviour, any background-targeting quirks, focus-flicker observations) get appended to this spec doc as a new section before Branch 3 starts.

Branch 3 then wires the Settings UI (per-app toggle rows, hotkey-capture inputs with validation, supported-apps matrix), the banner, the persistence file, and the tray notification, copying the literal strings from the sections above. Branch 4 is the v0.18.x beta release that flips the Cargo feature on for that build only and ships the toggles to beta-channel users.

---

## What this document is NOT

- Not Rust or TypeScript code. No structs, no React components, no commands.
- Not a Cargo feature flip. Branch 2 does that, not this.
- Not a release notes file. The release-notes block above is a draft; it goes into a real `releases/0.x.y.md` file in Branch 4, not here.
- Not a green light to start coding Branch 3. Branch 2 must run end-to-end and its findings appended before any UI work begins.
