# True Per-App Microphone Mute Decision

**Status: APPROVED for experimental beta. Decided 2026-05-03 by Jay.**

**Approved scope (locked at this commit):**

1. **Q1, doing it at all:** Yes, as **experimental** behind an explicit warning UX. The v0.16.5 honesty-pass framing stands: this is a known-fragile feature with a real silent-failure mode, and it ships only as opt-in beta in Settings → Advanced → Experimental, never bundled into stable without a full beta soak.
2. **Q2, which apps in v1: Teams AND Discord.** Revised 2026-05-03 by Jay (was Teams-only). Both ship together. Per-app toggles in Settings, each with its own user-configurable mute-hotkey field (defaults match each app's out-of-the-box default; user can rebind to match what they have set in the call app). Both apps treated symmetrically: Teams' hotkey is also user-configurable in Microsoft Teams settings (Settings, Keyboard shortcuts, Toggle mute), so making Utterbar's input field configurable on both apps avoids the asymmetry of "Teams hardcoded, Discord configurable" and futureproofs against either app changing its default. Zoom and Webex remain on the staged-iteration path (separate v2/v3, each with its own beta soak).
3. **Q3, experimental risk acceptable:** Yes, IF the warning UX is exhaustive (Q4) and verification-failure path is loud (Q5/Q6).
4. **Q4, warning copy:** TBD in Branch 1 (copy/spec). Toggle eyebrow `EXPERIMENTAL`. Toggle description begins **"Experimental. This may fail silently. We recommend leaving it off unless you're actively testing it."** Banner copy and release-notes paragraph drafted in Branch 1, approved by Jay before Branch 2 starts.
5. **Q5, verification UX:** Start with **NONE; always-on banner**. After every mic-mute dictation, a banner across the floating bar reads "Tried to unmute you in Teams, please confirm in Teams." Auto-dismisses after 5 seconds for the success path; sticks until clicked if restore failed (process gone, focus lost, keystroke delivery returned an error). Honest-can't-verify beats UIA-confidence-we-shouldn't-have. UIA-based verification is a separate v2 decision after the always-on banner soaks; do NOT add UIA mute-state scraping in v1.
6. **Q6, restore-failure UX:** Persistent banner that does NOT auto-dismiss until clicked, on the floating bar AND a system-tray balloon notification, both saying "Utterbar may have left you muted in Teams. Click to confirm." Plus: persist mute-was-applied state to disk between toggle-on and stop, so a crashed-and-relaunched Utterbar attempts restore on next launch and surfaces the banner if it can't.
7. **Q7, mechanism:** **Hotkey synthesis only**, no UI Automation. Per-app: send the user-configured mute combo (default `Ctrl+Shift+M` for both Teams and Discord) via `SendInput` targeted at the relevant app window (resolved by process name; window class as a secondary disambiguator). UIA verification is explicitly excluded from v1 per Q5. The `SendInput` keystroke needs a window-targeting wrapper; today's `text_inject.rs` is focused-window only. Build that as part of v1 inside `platform/windows/`, not a new module. The wrapper accepts a target HWND so the same code path serves Teams and Discord. Reference to "we deliberately background-target so the user keeps focus" stands as an aspiration; the actual focus behaviour during keystroke send is a Branch 2 finding to record.
8. **Q8, Settings placement:** **Settings → Advanced → Experimental** subsection. Distinct toggle from the v0.17.0 output-mute (which stays in Privacy). The Advanced placement is load-bearing: it sets the "this is experimental" framing before the user sees the toggle.

**Required safety constraints (unchanged from below, restated for clarity):** off by default (per-app); never claim guaranteed mute (always say "tried to mute"); restore from every dictation-end path including SEH crash handler; persisted mute-state to disk per app; release notes that explicitly say "may fail with app updates"; Teams-and-Discord support matrix in UI with per-app version pinning; hotkey input must require at least one modifier (reject bare-letter binds like `M`); hard timeout on every keystroke-send call; failure logging without secrets (no transcripts, no window titles beyond app name); no global OS mic mute (`IAudioEndpointVolume::SetMute` forbidden, unit-tested).

**Recorded:** 2026-05-03 against `0b1986d` on `main`. Approved (experimental scope): 2026-05-03 by Jay.

**Owner of the decision:** Jay.

The body of this document below is the original investigation. The "Approval decisions required before code" section is now answered (above). The "Recommended branches" section IS the implementation roadmap. Branch 1 (copy/spec pin) is the next step.

**v1 spec drafted at [true-per-app-microphone-mute-v1-spec.md](true-per-app-microphone-mute-v1-spec.md), Jay approves the literal copy in that file before Branch 2 starts.**

---

## What this is about

A future feature that mutes the **microphone feed** Teams / Slack / Discord / Zoom / Webex are sending into a call, while Utterbar is recording — so the user dictating into another window doesn't bleed their voice into a live meeting. The opposite of the v0.16.5 output-audio mute, which silences what you HEAR from those apps.

This is a fundamentally different feature from output-audio mute and carries a fundamentally worse failure mode: **the user could be left muted in a real call after a crash or restore failure**, which is the exact failure mode the v0.16.5 honesty pass warned against.

---

## What already exists

- **Output-audio ducking**, shipped in v0.16.5 beta. [services/audio_ducking.rs](../../Utterbar%20Windows%20App/src-tauri/src/services/audio_ducking.rs) mutes the **render** sessions of conferencing apps via Windows audio session APIs (`IAudioSessionControl2` + `ISimpleAudioVolume::SetMute`), and restores them on dictation stop. Only flips sessions that weren't already muted, so a pre-existing manual mute survives.
- **Editable conferencing app list**, also shipped in v0.16.5. `Settings.privacy.conferencing_apps_override: Vec<String>` ([domain/settings.rs](../../Utterbar%20Windows%20App/src-tauri/src/domain/settings.rs)) plus the editor UI in [PrivacySettings.tsx](../../Utterbar%20Windows%20App/src/routes/settings/PrivacySettings.tsx). Empty override means "use built-in defaults".
- **Explicit honest copy** that the v0.16.5 toggle does NOT mute the microphone: see the toggle description in [PrivacySettings.tsx](../../Utterbar%20Windows%20App/src/routes/settings/PrivacySettings.tsx) and the load-bearing release-notes lines at [releases/0.16.5.md](../../Utterbar%20Windows%20App/releases/0.16.5.md) lines 10, 16, 22. The release notes say literally: *"True per-app microphone mute would require simulating each app's in-call mute shortcut, which is brittle and out of scope."* and *"Your microphone feed into Teams, Discord, Zoom, or Webex is unchanged."*
- **Existing UI Automation infrastructure** at [platform/windows/uia_thread.rs](../../Utterbar%20Windows%20App/src-tauri/src/platform/windows/uia_thread.rs). Today it's used **read-only** for selection-text snapshots, on a dedicated STA OS thread because UIA is fundamentally STA-COM. Not a green light for click-driving the mute button — see "Approach C" below for why reusing this is not free.
- **Existing `SendInput` text injection** at [platform/windows/text_inject.rs](../../Utterbar%20Windows%20App/src-tauri/src/platform/windows/text_inject.rs). Today it injects Unicode keystrokes and `Ctrl+V` into the **focused** window only. Synthesising a hotkey to a different (background) Teams/Discord/Zoom window is not what `SendInput` does — see "Approach A" below.
- **No mic-mute implementation anywhere.** Greps confirm: no `mute_microphone`, `mic_mute`, `capture_endpoint_mute`, no audio-input session muting, no per-app-mic anything. The v0.16.5 release notes are accurate.

---

## Why true microphone mute is a different feature

Three architectural facts make this fundamentally harder than output-audio mute:

1. **Utterbar needs the microphone to dictate.** Any approach that silences the OS-level mic capture path (the input endpoint) would also silence Utterbar's own ASR. We can't mute the OS capture endpoint while we're recording — we'd be muting ourselves.

2. **Muting the OS capture endpoint would also mute Utterbar.** Same fact, different framing. Windows audio capture is an endpoint property — `IAudioEndpointVolume::SetMute` on the default capture device — and an app cannot scope that mute to "everyone except me". The endpoint is shared.

3. **Windows output session APIs do not control another app's microphone send state.** The render-side `ISimpleAudioVolume::SetMute` we use today silences what the user hears from a session. There is no equivalent input-session API for "stop sending this app's microphone capture to its remote peer". The conferencing app itself decides what to send to its server; Windows has no privileged hook into that decision.

The only ways to influence what Teams/Discord/Zoom/Webex send to a call are **outside the audio stack**: simulate the app's in-call mute hotkey, drive its mute button via UI Automation, or play games with virtual microphone devices. Each is brittle in its own way.

---

## Approach comparison

### Approach A — Per-app hotkey synthesis (`SendInput` to a target window)

Send the app's documented in-call mute hotkey via Windows input synthesis when Utterbar starts recording, and the same hotkey again to un-mute on stop.

| Property | Reality |
| --- | --- |
| Reliability | Low. `SendInput` always delivers to the **foreground** window. Sending Ctrl+Shift+M to a backgrounded Teams does nothing — Teams isn't focused. To work it would have to focus-switch (visible to user, kills dictation flow), or rely on Teams' global hotkey (which is opt-in and many users don't enable). |
| Failure mode | **Silent.** No way to know whether the keystroke "took". The hotkey synthesis succeeds in the OS sense even when the target app didn't react. |
| Implementation cost | Low to write, **high to make trustworthy.** Per-app hotkey lookup table, per-version drift, per-user customisation drift, focus-management, foreground-restoration after sending. |
| User-risk | Severe. Could leave the user muted in a real call after Utterbar crashes between mute and restore. Could mute the wrong app (whichever is foreground). Could synthesise the hotkey into the user's text editor mid-typing. |
| Verification possible? | No. The app's mute state is opaque to us. |

### Approach B — UI Automation targeting mute buttons

Walk each conferencing app's UI tree, find the "Mute" / "Unmute" button by name and class, and invoke it via `IUIAutomationInvokePattern::Invoke`.

| Property | Reality |
| --- | --- |
| Reliability | Medium-low and deteriorates over time. Teams renames its mute button between Classic / New / Web shells; Discord re-themes regularly; Zoom A/B-tests its meeting toolbar. Selectors break. |
| Failure mode | Fails closed (button not found → did nothing → user unmuted) or fails open (button found but already in the desired state → toggle goes the wrong way → muted when we wanted to unmute). Both are **silent**. |
| Implementation cost | **High.** Inherits the apartment-mode pain documented in [uia_thread.rs](../../Utterbar%20Windows%20App/src-tauri/src/platform/windows/uia_thread.rs): UIA is STA-COM, must run on the dedicated STA worker thread. Adding a second category of UIA call (mute-button-click vs current selection-text-read) is not free. |
| User-risk | Severe. Same "left muted after crash" failure mode as Approach A, plus the new failure mode of "we found a button labelled Mute but it was Discord's notifications-mute, not call-mute". |
| Verification possible? | Partial — we can read the button's `Toggle.ToggleState` after invoking, IF the app exposes it as a togglepattern, IF the property reflects the actual call-mute state. Many apps do not surface this. |

### Approach C — OS capture endpoint mute

Call `IAudioEndpointVolume::SetMute(TRUE)` on the default capture device for the duration of dictation. Restore on stop.

| Property | Reality |
| --- | --- |
| Reliability | High in the narrow sense (the API call works). |
| Failure mode | **Doesn't actually do what the user wants.** Muting the capture endpoint also mutes Utterbar's own ASR input — we'd be silencing ourselves. Unmute-on-restore only works if our process is alive. |
| Implementation cost | Trivial. |
| User-risk | Catastrophic. Every dictation = mic killed everywhere. We've now broken the conferencing app AND Utterbar in one move. |
| Verification possible? | We'd see our own ASR drop; that's the verification, and that's why we can't ship this. |

**Approach C is forbidden by the strict-action list above and would be forbidden anyway by the architecture.**

### Approach D — Virtual microphone / device routing

Install a virtual audio device that sits between the OS and the conferencing app. Utterbar controls the virtual device; muting it stops audio reaching Teams/Discord/etc.

| Property | Reality |
| --- | --- |
| Reliability | High once installed. |
| Failure mode | Setup-time. Virtual driver requires admin-level installer steps, signed driver, Windows kernel-mode security review. SmartScreen warnings get worse, not better. |
| Implementation cost | **Huge.** Different product. Driver authoring, signing, distribution, support burden across Windows builds. |
| User-risk | Moderate at runtime (well-defined surface), severe at install-time (kernel-mode driver = bricked-machine class of failures). |
| Verification possible? | Yes — we own the device, we know its mute state. |

This is a different product. Out of scope. Mentioned only for completeness.

### Approach E — "Do nothing beyond output mute"

Continue to ship the output-audio mute, continue to be honest about scope, and let users hit their app's own mute button. Recommend this approach in any UI surface that asks about mic mute ("looking for mic mute? See your call app's mute button").

| Property | Reality |
| --- | --- |
| Reliability | Trivially perfect. We don't promise something we can't deliver. |
| Failure mode | None — the user is on the existing manual contract. |
| Implementation cost | Zero. |
| User-risk | Zero. |
| Verification possible? | N/A. |

This is the recommended default.

---

## Recommended default

**Do not build true mic mute.** Keep output-audio mute as the honest supported feature. The v0.16.5 release notes are accurate; let them stay accurate.

If Jay later insists, only ship as **experimental beta** with:

- An unmissable warning at toggle-on time.
- A persistent banner during dictation: "Utterbar may have left you muted in &lt;app&gt; — click to confirm."
- A verification check on dictation stop that reads back the app's mute-toggle state via UIA where possible, and surfaces a status pill if verification fails.
- Off by default.
- Settings → Advanced (not Settings → Privacy) so the framing matches the risk.
- A release note that explicitly says "may fail with app updates; this is experimental".

---

## Approval decisions required before code

These must be answered on paper before any branch starts. Default of "no decision" = "do not implement". Any future implementation prompt must cite which of these Jay has answered.

1. **Are we doing this at all?** The v0.16.5 release notes set an explicit "out of scope" expectation. Reversing that is a brand decision: we shipped a careful honest feature; building this on top breaks the implicit contract. Confirm yes.
2. **Which apps to support first.** Recommend ONE app for v1 (probably Teams or Discord — Teams has a configurable global hotkey, Discord has a documented push-to-talk system). Adding a second app is a separate decision after the first soaks. Do not promise a four-app matrix on day one.
3. **Whether experimental risk is acceptable.** Are we comfortable with the failure mode "user is muted in their boss's call after Utterbar crashed"? If not, do not build. If yes, the warning UX must be exhaustive.
4. **Exact warning copy.** Onboarding-style copy for the toggle, the persistent banner, and the verification-failure pill. Jay must approve every line.
5. **Verification UX.** What signals to the user "we believe you are muted right now" vs "we tried but couldn't verify". Where does the banner live (floating bar, system tray, settings)? Click-to-confirm restore on dictation stop?
6. **Restore-failure UX.** What happens if Utterbar can't put the mute back the way it found it? Banner that doesn't dismiss until the user clicks? System notification? Tray icon flash?
7. **Hotkeys, UI Automation, or neither.** Recommend UIA for verification + hotkey-or-UIA for invocation, picked per-app. Jay must confirm the failure modes of the chosen mechanism are acceptable.
8. **Whether this belongs in Advanced rather than Privacy.** Recommend **Settings → Advanced** under an "Experimental" subsection, with a banner that says "This feature is experimental and can fail silently. We recommend leaving it off." The current "Privacy" placement of the v0.16.5 toggle would imply the same level of trust we have for output-audio mute — which we don't.

Until ALL of these are answered (and approved by Jay in writing), do not start any branch.

---

## Required safety constraints if ever implemented

These are non-negotiable and must be encoded as tests + UX wiring:

1. **Off by default.** Settings toggle ships `false`. No silent migration, no opt-in via the existing output-mute toggle.
2. **Marked Experimental.** Settings card eyebrow `EXPERIMENTAL`. Toggle description begins "**Experimental.** This may fail silently."
3. **App-specific support matrix.** A static list of supported apps with per-app version tested. UI shows the matrix so the user knows whether their build is covered. Adding an app requires a code change, not a settings field.
4. **Never claim guaranteed mute unless app state can be verified.** If we can't read back the app's mute state via UIA toggle-pattern, we may say "tried to mute" but never "you are muted". The status pill copy reflects this.
5. **Visible "check your call app" banner after restore.** On dictation stop, surface a banner that doesn't auto-dismiss saying "We tried to restore your mute state in &lt;app&gt;. Please confirm in the app." User clicks to acknowledge. The banner is independent of whether verification succeeded — even when we think it worked, we ask the user to look.
6. **Restore attempt on stop / cancel / error / panic path.** The restore must run from every code path that ends a dictation, including the SEH crash handler. The mute set must be persisted to disk between toggle-on and stop, so a crashed-and-relaunched Utterbar can attempt restore on next launch.
7. **Timeout and failure logging without secrets.** Every UIA selector lookup has a hard timeout (e.g. 500 ms). Failures log the app name + selector + timeout, never user transcripts, never window titles beyond the app name.
8. **No global OS mic mute while Utterbar is recording.** Approach C is forbidden. Add a unit test that asserts no `IAudioEndpointVolume::SetMute` call appears anywhere in the codebase.
9. **No silent success state.** "Toggle on, dictation runs, no banner" is not allowed. Every toggle-on dictation surfaces a status notice on stop, even if it's "Mute restored successfully."
10. **Clear release notes saying it may fail with app updates.** The release note for the version that ships this must include a paragraph explaining: "Teams / Discord / Zoom / Webex update their UI regularly. This feature may stop working when those apps update before we do. If it stops working, turn it off until the next Utterbar release. We will not silently break or claim success when the underlying app changed."

---

## Recommended branch breakdown if approved later

Strict ordering. Branches 1-2 are decision/research and ship no runtime mute code.

1. **Copy / spec branch** (`feature/mic-mute-spec-pin`)
   - Add the warning copy, banner copy, status-pill copy, release-notes paragraph as draft text in this decision doc (or a sibling in `docs/decisions/`).
   - **Reversible.** Stops here if the warnings sound un-Utterbar.

2. **Prototype branch behind compile-time or advanced flag** (`research/mic-mute-prototype`)
   - Cargo feature `experimental_mic_mute` (off by default in release builds).
   - One-app prototype (Teams or Discord) as a Rust unit test that exercises the UIA + verification path against a real running instance. Manual run only.
   - **No release ships this branch.** Findings recorded in this document or a sibling memory note.

3. **App-specific implementation branch for ONE app only** (`feature/mic-mute-teams` or `feature/mic-mute-discord`)
   - First app gets the full UX wiring: toggle, banner, restore-on-crash persistence, status pill.
   - **Beta only first.** Soak on the beta channel before any stable promotion — judgement-based, not a fixed duration. Given the silent-failure risk class, lean towards the cautious end of the soak spectrum.

4. **Beta release branch** (`chore(release): prepare 0.x.y beta` for the patch that introduces it)
   - Beta-only. No `--announce`. Release notes carry the "may fail with app updates" warning paragraph in full.
   - Promote to a mainline minor only after a clean beta soak and explicit Jay approval. No fixed minimum duration; given the failure class, this feature warrants extra caution before a stable cut bundles it.

Adding a second app (e.g. Zoom after Teams) is a separate iteration of branches 3 + 4, with its own beta soak.

---

## Tests required if later implemented

- **No-capture-mute invariant** — codebase-wide grep test that `IAudioEndpointVolume::SetMute` is not called from anywhere. Pinned because Approach C is the easiest "fix" a future developer might reach for.
- **Stop / cancel / error restore tests** — for every dictation-end path (normal stop, hotkey cancel, error, panic), the restore code is exercised. Pin the four call sites.
- **Crash-restore-on-relaunch test** — simulated mid-dictation crash leaves a `mute_state.json` on disk; next launch reads it and attempts restore + surfaces a banner.
- **Hotkey target / window-focus tests where possible** — given a synthetic Teams-like target window, the hotkey reaches the right window without changing foreground.
- **UI Automation selector tests where possible** — for each supported app, a CI-skipped integration test that walks the UI tree against a known-version build of that app (probably a manual matrix, not CI).
- **Manual matrix for Teams / Discord / Zoom / Webex versions** — quarterly re-check that the selector still finds the mute button on the latest stable build of each app. If not, the support matrix entry flips to "broken on version X" and the app is disabled in Settings UI for that user.

---

## Explicit non-goals

These are out of scope even if Jay approves the rest:

- **No guaranteed mute without verification.** If we can't verify, we don't claim it.
- **No browser-tab Google Meet support.** Meet runs inside `chrome.exe` / `msedge.exe`; muting those would silence every browser tab. Out of scope unless the user explicitly adds a browser EXE to the conferencing list (per the v0.16.5 honesty statement).
- **No muting the whole microphone endpoint during dictation.** Approach C is forbidden by both the architecture and this document.
- **No pretending output mute equals microphone mute.** The v0.16.5 toggle's scope and copy stay exactly as they are. This is a separate feature.
- **No shipping hidden behind the existing output-mute toggle.** The user must opt in to true mic mute as a separate, clearly-labelled experimental control.
- **No analytics tied to this feature** (see [docs/decisions/opt-in-product-analytics.md](opt-in-product-analytics.md)).
- **No reuse of the v0.16.5 output-mute "Apps to silence" list.** The mic mute matrix is a separate per-app static list because it requires per-app hotkey/UIA selectors, not just an EXE basename.

---

## What this document is NOT

- Not implementation code. No Rust services, no Tauri commands, no React components, no settings fields, no policies.
- Not a copy / legal / marketing edit. Those branches happen later, conditionally.
- Not a green light. The default for every approval question above is "no".

If a future session decides to implement this, the corresponding implementation prompt should reference this document by path (`docs/decisions/true-per-app-microphone-mute.md`) and explicitly cite which approval decisions Jay has made and which app is the first target. A prompt that does not cite these is incomplete.
