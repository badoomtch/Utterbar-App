# Teams Mic-Mute v1 Spec (experimental)

**Branch 1 of the four-branch plan in [true-per-app-microphone-mute.md](true-per-app-microphone-mute.md).**

**Status: DRAFT, awaiting Jay sign-off on every literal string below.** Branch 2 (Cargo feature + Rust prototype) does NOT start until Jay has read this file end-to-end and confirmed the copy lands as written. Reversibility is the point of this branch: if any string reads wrong, edit it here and re-push before any code work begins.

---

## Scope reminder

In v1: Teams only. Mechanism: send `Ctrl+Shift+M` to the Teams window via a new background-window-targeting `SendInput` wrapper. No UI Automation. Verification: NONE; every dictation that engaged the toggle surfaces an always-on banner saying we tried to unmute Teams and asking the user to confirm. Restore-failure path: persistent banner on the floating bar plus a Windows tray balloon, both load-bearing because the failure mode is silent. State persisted to disk under `%APPDATA%\Utterbar\` so a crashed-and-relaunched Utterbar can attempt restore on next launch. Toggle ships off by default in Settings, Advanced, Experimental.

NOT in v1: Discord, Zoom, Webex (each is a separate iteration with its own beta soak); UI Automation read-back of the mute button state (deliberately deferred so we don't ship false confidence); any change to the OS-level capture endpoint mute (forbidden by the parent decision doc, would also mute Utterbar's own ASR); auto-on by default; bundling into a stable release without an explicit Jay-confirmed soak window.

The parent decision doc [true-per-app-microphone-mute.md](true-per-app-microphone-mute.md) carries the why for each of those constraints.

---

## Settings, Advanced, Experimental: literal copy

The toggle lives in a new `Experimental` card on the existing `Settings, Advanced` page. Same `Card` + `Row` + `Toggle` primitives the rest of Settings uses; the eyebrow is the new bit. Tone matches the v0.17.0 conferencing-output-mute description on `Settings, Privacy, Media`: second person, plain English, honest about scope, points at the call app's own button as the trustworthy fallback.

**Card eyebrow:** `EXPERIMENTAL`

**Card title:** `Mute your microphone in Teams while recording`

**Toggle label:** `Mute Teams microphone while you dictate`

**Toggle description:**

> Experimental. This may fail silently. When you start a dictation, Utterbar sends `Ctrl+Shift+M` to your Teams window to mute the microphone there, then sends it again when you stop to unmute. After every dictation, a banner asks you to confirm the mute state in Teams, because Utterbar cannot read Teams' actual mute state and shouldn't pretend it can. Teams only today; Discord, Zoom, and Webex are not supported. If this ever misbehaves, turn it off and use the Teams mute button directly.

**Supported apps subtitle:** `Supported apps`

**Matrix one-row entry for Teams:**

| App | Status | Tested against |
|---|---|---|
| Microsoft Teams (classic and new Teams) | `Experimental` | Last verified 2026-05-03 against Teams classic 1.7.x and new Teams 24.x |

(The `Last verified` date gets bumped on every quarterly manual re-check per the testing rules in the parent decision doc. The "broken on version X" disabled state shape is documented in section "What Branch 2 will do" below; v1 does not implement detection, only the matrix UI.)

**Empty-state copy when Teams is not detected on this machine:**

> Teams is not running or installed on this machine. Install Teams or open it once before this toggle becomes available.

(Detection is a process-list scan for `Teams.exe` or `ms-teams.exe`; happens at Settings page open, not in real time.)

---

## Floating-bar banner: literal copy

Banners render inline at the top of the floating bar, above the existing recording chrome, in the same width as the bar itself. Two variants share a layout but differ in colour, dwell, and dismiss affordance.

**Active-recording sub-line during dictation (only when the toggle is on):**

> Tried to mute Teams.

This sits in the same line as the existing mode chip / timer; small, neutral colour, no dismiss affordance. Disappears when dictation ends and is replaced by one of the two banners below.

**Success banner shown after dictation stop (auto-dismisses after 5 seconds):**

> Tried to unmute you in Teams. Please confirm in Teams.

Subdued colour, no border emphasis, dismiss affordance is `Got it`. The banner says "tried to" rather than "did" because we genuinely cannot verify the keystroke landed; "tried" is honest and the user is the verifier.

**Restore-failure banner that does NOT auto-dismiss (sticks until clicked):**

> Utterbar may have left you muted in Teams. Open Teams and check the mic icon.

Loud colour (uses the same accent the existing recording-state pill uses for attention but should not feel celebratory; the brief is "noticeable, not alarming"). Dismiss affordance is `Confirmed in Teams`, deliberately a verb the user has to mentally enact, not a passive `OK`. Clicking `Confirmed in Teams` clears the banner and writes nothing to disk; this is a UI ack only.

**Why two affordances rather than one shared `OK`:** the verb on the failure case forces the user to perform the check before dismissing. Reading `OK` and clicking it is autopilot. Reading `Confirmed in Teams` and clicking it is at least one cognitive beat where the user looks at the Teams window. That beat is the entire value of the always-on banner approach over a silent failure.

---

## System-tray balloon notification: literal copy

Fires alongside the restore-failure banner only, never on the success path (a tray balloon every dictation would be noise; the failure case is the one we want to escape the floating bar). Standard Windows toast.

**Title (under 64 chars):** `Utterbar may have left you muted in Teams`

**Body (under 256 chars):** `Open Teams and check the mic icon. Click here to bring Utterbar to the front.`

**Click action:** brings the floating bar to the foreground so the user sees the persistent banner. Does not dismiss the banner; only navigates to it.

---

## Status pill copy on the bar

The existing recording-stop status pill (the small pill that briefly shows things like "Inserted") gets two new states for mic-mute toggle-on dictations. Brief, single-line, render in the existing pill style.

| State | Pill copy | Dwell |
|---|---|---|
| Restore reported success | `Mute restored` | Existing default (a few seconds) |
| Restore reported failure | `Mute restore failed` | Persists until the floating-bar banner is dismissed |

`Mute restored` is honest because the success case is "we sent the unmute keystroke and `SendInput` returned no error" — that's what we know, no more, and the always-on banner already softens this from a guarantee to a request to confirm. `Mute restore failed` is shown when `SendInput` returned an error, the Teams process is gone, or the targeting wrapper couldn't find a Teams window.

---

## Persistence schema for crash-restore

When the toggle is on and a dictation starts, Utterbar writes `mic_mute_state.json` under `%APPDATA%\Utterbar\` recording the fact that we sent a mute keystroke. The file is deleted when we successfully send the unmute keystroke and `SendInput` returns no error. If Utterbar crashes between mute and unmute, the file persists; on next launch, Utterbar reads it, sends an unmute keystroke to any running Teams window, and surfaces the restore-failure banner regardless of the keystroke result (because the user has been muted unknowingly for some interval and deserves to be told).

**Filename:** `%APPDATA%\Utterbar\mic_mute_state.json`

**Schema:**

```json
{
  "schema": 1,
  "app": "teams",
  "applied_at_utc": "2026-05-03T22:00:00Z",
  "previous_mute_state": "unknown",
  "session_id": "01HXYZ12345ABCDEFGHIJKLMNO"
}
```

**Field rules:**

- `schema`: integer. Currently `1`. Bumped if the file shape ever changes; older schemas are deleted-and-ignored on next launch (do not attempt restore against an unknown schema).
- `app`: string. Currently always `"teams"`. Reserved for the per-app expansion in later iterations of Branches 3 and 4. Closed enum.
- `applied_at_utc`: ISO 8601 UTC string with seconds precision. Used only for log lines and the "if this is older than 24h, delete and ignore" sanity gate (a stale file from a long-ago crash should not silently mute Teams next launch).
- `previous_mute_state`: closed enum, one of `"unknown"`, `"muted"`, `"unmuted"`. v1 always writes `"unknown"` because we do not read Teams' state. Reserved for a future v2 that uses UIA to read mute state at apply time; the field exists now so a v2 desktop reading a v1 file gets a defined value rather than an absent one.
- `session_id`: 26-character ULID held in memory for the dictation session. Used to correlate log lines without storing any user-identifying info. Local-only; never sent anywhere; the REJECTED-analytics decision in CLAUDE.md §5.1 forbids any wire transport of this or any other field.

**Lifecycle:**

1. Toggle is on, user starts a dictation: write the file with current UTC, then send mute keystroke.
2. Dictation ends normally: send unmute keystroke; if `SendInput` returned no error, delete the file. If it errored, keep the file and surface the restore-failure banner.
3. Utterbar crashes between (1) and (2): file persists.
4. Utterbar launches: if `mic_mute_state.json` exists and `applied_at_utc` is within the last 24 hours, attempt restore (send unmute keystroke to any running Teams window) and surface the restore-failure banner regardless of keystroke outcome. Delete the file. If the file is older than 24 hours, delete and ignore (do not assume a day-old crash; the user has long since dealt with it manually).
5. The toggle is turned off while a dictation is in flight: the in-flight dictation finishes its normal lifecycle (the unmute keystroke still sends, the file still gets cleaned up). Subsequent dictations skip the mute path entirely.

The "older than 24 hours" sanity gate is deliberate. The whole feature is about catching short-window crashes; a stale file from last week is not the failure mode the persistence layer is for.

---

## Onboarding behaviour

This feature is NOT on the onboarding path. New users do not see the toggle during first-run, and the onboarding script does not mention Teams mute or experimental features. Discovery is by Settings search (typing `teams mute`, `mic mute`, `microphone`, `experimental`, all of which should resolve to this Settings row via the Settings search v2 alias map shipped in v0.18.1) or by browsing Settings, Advanced manually.

This is deliberate. Onboarding sets first-impression promises; an experimental feature with a known silent-failure mode is not the right shape for that surface. A future session adding an onboarding step for this is explicitly out of scope. Settings -> Advanced -> Experimental is also the placement in the parent decision doc's Q8 sign-off; do not reuse the v0.17.0 output-mute Privacy card for this toggle.

---

## Release-notes paragraph (DRAFT for Branch 4)

Pre-written so Jay can read the user-facing copy now rather than at release time. Will be lifted into the v0.18.2 (or whichever) beta release notes verbatim once Branch 3 ships. Voice rules per CLAUDE.md §5.8: second person, British English, no source-code acronyms, no em dashes, no internal-mechanism jargon.

> ### Mute your microphone in Teams while you dictate (experimental, beta only)
>
> A new toggle in Settings, Advanced, Experimental, off by default. When you turn it on, Utterbar will try to mute your microphone inside Microsoft Teams whenever you start a dictation, then unmute it when you stop. The point is to stop your dictation bleeding into a live Teams call when you're talking to another window.
>
> This is experimental. Utterbar cannot read Teams' actual mute state, so after every dictation a banner appears on the floating bar asking you to confirm the mute state in Teams. The banner is not a glitch; it's the honest version of "we tried to unmute you, please look at the mic icon in Teams to make sure". If something goes wrong, a more visible banner and a Windows tray notification will tell you that you may have been left muted, and Utterbar will try to unmute on its next launch if it crashes mid-dictation.
>
> Teams only in this release. Discord, Zoom, and Webex are not supported. Microsoft updates Teams regularly; this feature may stop working when Teams changes before Utterbar does. If it stops working, turn it off and use the Teams mute button directly until the next Utterbar release.
>
> What this does not change: the existing v0.17.0 setting that mutes what you HEAR from conferencing apps (Settings, Privacy, Media) is unchanged and continues to work as before. Microphone capture for your own dictation, audio routing, the speech models, and everything else are unaffected.

---

## Telemetry and logging stance

Failures are logged locally to the existing app log under `%APPDATA%\Utterbar\logs\`. Log lines record the app name (`teams`), the action attempted (`mute_apply` / `mute_restore`), the result code from `SendInput`, and the timeout if any. They do NOT record user transcripts, focused-window titles, clipboard contents, vocabulary entries, or any other secret listed in the parent decision doc's "Banned fields" guidance. The log lines are local-only; no remote shipping, no analytics call, no opt-in fallback.

This stance is pinned by the REJECTED opt-in analytics decision in CLAUDE.md §5.1. Future sessions: do not propose adding remote shipping of mic-mute logs, even as a debugging aid, even opt-in. If a debugging aid is genuinely needed, the existing `Settings, Advanced, Diagnostics, Export` flow already produces a local zip the user can attach to an email; that is the supported path.

---

## What Branch 2 will do (not in this branch)

Brief summary so the next prompt has a head start; expand into a real implementation prompt when Branch 1 merges.

- Add Cargo feature `experimental_mic_mute` to `Utterbar Windows App/src-tauri/Cargo.toml`. Off by default in release builds.
- Add a new module at `Utterbar Windows App/src-tauri/src/platform/windows/window_send_input.rs` that wraps `SendInput` to target a background window resolved by process name + window class, without changing the foreground. The existing `text_inject.rs` is focused-window only; this is a deliberately separate module so the existing text-injection path is not destabilised.
- Add a Rust unit test (manual-run only, gated by `cfg(feature = "experimental_mic_mute")` plus `#[ignore]` so it does not run on `cargo test`) that exercises the new wrapper against a real running Teams instance on Jay's machine and prints the result.
- No Settings UI, no toggle field, no banner, no persistence file in this branch. Pure runtime prototype.
- Do not flip the `experimental_mic_mute` feature on for any release build in Branch 2.
- Findings (P50 latency of the keystroke send, observed Teams classic vs new Teams behaviour, any background-targeting quirks) get appended to this spec doc as a new section before Branch 3 starts.

Branch 3 then wires the Settings UI, the banner, the persistence file, and the tray notification, copying the literal strings from the sections above. Branch 4 is the v0.18.x beta release that flips the Cargo feature on for that build only and ships the toggle to beta-channel users.

---

## What this document is NOT

- Not Rust or TypeScript code. No structs, no React components, no commands.
- Not a Cargo feature flip. Branch 2 does that, not this.
- Not a release notes file. The release-notes block above is a draft; it goes into a real `releases/0.x.y.md` file in Branch 4, not here.
- Not a green light to start coding. Jay reads this spec, edits any string that reads wrong, then approves on the PR before Branch 2 starts.
