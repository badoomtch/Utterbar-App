# Utterbar — Product Requirements Document

**Version:** 1.0 (v1 scope)
**Owner:** Product
**Status:** Approved for build
**Platform:** Windows 11 (Windows 10 22H2 minimum)

---

## 1. Vision

Utterbar is a premium, local-first dictation utility for Windows that converts speech into polished text in any application. It lives in a signature movable floating bar, processes audio entirely on-device using NVIDIA Parakeet, and offers a mode-based workflow system (Voice → Text, Message, Email, Note, Meeting, Super, Custom) with per-app auto-activation, vocabulary, text replacements, and local analytics.

It exists because professional dictation users want the workflow depth of modern cloud dictation tools **without** their voice or text ever leaving the machine.

## 2. Goals and non-goals

### v1 goals

- Ship a polished Windows app with a floating bar, onboarding, settings, history, stats, and file transcription.
- Integrate NVIDIA Parakeet (English + multilingual) via a provider abstraction.
- Deliver reliable text insertion via paste and keystroke-simulation fallback.
- Support toggle / hold-to-talk / push-to-talk, cancel, and mode-switch global hotkeys.
- Implement modes, vocabulary, text replacements, per-app auto-activation, and context capture.
- Provide local-only analytics, history with reprocessing, and file transcription.
- Ship a signed-build-ready installer and an updater plumbing layer.

### v1 non-goals

- No macOS or Linux ports.
- No cloud ASR or cloud LLM fallback. Ever.
- No collaborative / multi-user features.
- No mobile companion app.
- No voice commands that execute OS actions (dictation only in v1).

## 3. Target user

A knowledge worker on Windows who writes 5k+ words/day across email, chat, docs, and code — and wants to dictate most of it. Secondary users: accessibility-driven users who rely on dictation for physical reasons, and privacy-conscious professionals (legal, medical, research).

## 4. Principles

1. **Local-only is a product, not a feature.** Every decision defaults to on-device.
2. **Honesty over hype.** Language support, accuracy, and "minutes saved" are grounded in measurable reality.
3. **Feel premium.** Motion, sound, typography, and depth are polish items, not optional.
4. **Keyboard-first.** Every primary action has a shortcut. The mouse is a nice-to-have.
5. **Fail gracefully.** If the local LLM is absent, ASR still works. If a model is downloading, the UI is still responsive.

## 5. User stories (v1)

| # | As a… | I want… | so that… |
|---|-------|--------|---------|
| U1 | new user | to be walked through setup with model download, mic selection, and a test recording | I feel confident it works before I commit |
| U2 | dictating user | a global hotkey that starts recording no matter which app is focused | I never break flow |
| U3 | email writer | a Message mode that cleans up my speech into polite prose | I can fire off replies in seconds |
| U4 | note-taker | a Note mode that produces bullet structure | I can dictate into Obsidian or Notion |
| U5 | meeting attendee | a Meeting mode that handles long-form transcription | I can capture a whole call |
| U6 | privacy-sensitive user | proof that nothing leaves my machine, with a visible data folder | I trust the tool with confidential work |
| U7 | power user | per-app auto-activation of modes | the right mode kicks in when I click Outlook vs Slack |
| U8 | domain specialist | a vocabulary list with custom words | model stops mangling "Kubernetes" and "Azure AD" |
| U9 | file-wrangler | to drag an MP3 into Utterbar and get a transcript | I can transcribe recorded meetings or voice notes |
| U10 | analytics lover | a simple dashboard of my words/day and minutes saved | I see my productivity trend |

## 6. Feature scope (v1)

### 6.1 Onboarding (§ matches build spec)
Welcome → local-only pitch → permission checks → model pick (Parakeet EN / Parakeet Multi) → download with progress → mic pick → live meter → test recording → result confirmation → typing-speed baseline → trigger style (toggle / hold / push) → hotkey binding → theme pick → "ready to dictate" screen.

### 6.2 Floating bar
- Movable horizontally; snaps to screen top in thirds.
- Compact idle state (height ≈ 34 px, width ≈ 140 px): brand mark, mode indicator, small VU dot.
- Expanded active state: waveform, timer, mode name, model chip, state pill (loading / recording / processing / success / error), inline controls (stop, cancel, mode picker, history, settings).
- Hover reveals expanded; click pins expanded.
- Smooth animated transitions (Framer Motion spring, mass 1, stiffness 240, damping 22).

### 6.3 Modes
Built-in: Voice to Text, Message, Email, Note, Meeting, Custom, Super (context-aware).
Each mode stores: name, icon, ASR preference, post-processing rules, formatting, target language, app auto-activation rules, shortcut overrides, context toggles (clipboard / selection / active-app).
Duplicate, rename, favourite, reorder.

Post-processing priority:
1. If local LLM provider is enabled → transform with mode prompt.
2. Else → deterministic fallback (punctuation repair, casing, filler-word removal, template formatting).
ASR pipeline must never block on the LLM.

### 6.4 Context-aware ("Super") mode
Reads: focused window class + title, active app executable name, selected text (via UIA / clipboard copy trick when allowed), recent clipboard.
Each source is opt-in per mode and globally in Privacy settings.
"Context Inspection" panel in History shows exactly what was captured and sent into processing for each dictation.

### 6.5 Shortcuts
Toggle record, Push-to-talk, Hold-to-talk, Cancel, Cycle mode, Open settings, Open history, Per-mode shortcuts.
Conflict detection + warnings.
Mouse-button binding (extra mouse buttons) if the device exposes them.

### 6.6 Text output
Default: paste via `Ctrl+V`, optional restore-previous-clipboard.
Fallback: simulated keystrokes for apps that reject paste.
Options: hold `Shift` while ending record to auto-press Enter, auto-capitalize-sentence, trim-trailing-space.

### 6.7 Vocabulary & replacements
Custom words with optional IPA-style hint (if the model accepts it; otherwise stored as a prompt hint for post-processing).
Text replacements applied post-ASR, pre-LLM.
Import/export JSON.
Guardrail: warn if hints >500 entries or if hint-length ratio risks accuracy degradation.

### 6.8 History
SQLite-backed. Each entry: timestamp, mode, audio duration, raw transcript, processed text, app context, model, processing time, status, context payload (inspectable).
Search, filter by mode/app/date, replay audio (if saved), reprocess with another mode, copy raw, copy processed, export.

### 6.9 Analytics
Local-only. Words today/week/month, minutes saved (based on user-set typing-speed baseline), avg speaking WPM, per-mode usage, correction rate (edits applied to processed text within 5 min of insertion).

### 6.10 File transcription
Drag-drop `.mp3/.m4a/.wav/.flac/.ogg/.mp4/.mov/.mkv`. Queue UI. Per-item progress. Apply selected mode post-ASR. Export to `.txt/.md/.json`. Preserve word timestamps when the model supports them.

### 6.11 Settings
See [§7 IA](#7-information-architecture).

### 6.12 Tray & window behaviour
Tray icon with: Open Utterbar, Start Recording, Switch Mode ▸, Pause Shortcuts, Quit.
Close-to-tray, launch-on-login, start-minimised.

### 6.13 Updater
Tauri updater plumbing wired; endpoint TBD for production. Staged rollout UI: "Update available → Download in background → Restart to install."

## 7. Information architecture

```
Utterbar/
├── Floating Bar (always reachable)
├── Onboarding (first run)
├── Main window (tab-style navigation)
│   ├── Dashboard       — stats + recent activity
│   ├── History         — past dictations
│   ├── Modes           — mode list + editor
│   ├── Vocabulary      — words + replacements
│   ├── Files           — drag-drop transcription
│   └── Settings
│       ├── General
│       ├── Audio
│       ├── Models
│       ├── Shortcuts
│       ├── Text Output
│       ├── Modes (cross-link)
│       ├── Privacy
│       ├── Advanced
│       └── About
└── Tray menu
```

## 8. UX states every surface must handle

- Empty (no data yet)
- Loading (model warming, audio device enumeration)
- Active (recording, processing)
- Success (transcript delivered)
- Error (mic missing, model failed, paste blocked)
- Offline/degraded (no local LLM, or GPU unavailable — must still work)

## 9. Performance targets

| Metric | Target |
|--------|-------|
| Cold launch to floating-bar visible | < 1.5 s on mid-range laptop |
| Idle RAM (bar only, no model warm) | < 120 MB |
| Idle RAM (bar + warm Parakeet) | < 900 MB (CPU), < 1.4 GB (GPU) |
| Record-start latency (hotkey → listening) | < 120 ms |
| Transcription for 10 s clip (RTX 3060) | < 2 s end-to-end |
| Transcription for 10 s clip (CPU, i7) | < 8 s end-to-end |
| Bar animation | 60 fps sustained, no jank during record |

## 10. Accessibility

- Full keyboard navigation in every window.
- `aria-live` announcements for state changes.
- High-contrast theme variant.
- Reduce-motion preference honoured.
- Minimum 4.5:1 contrast on all text.

## 11. Privacy (hard rules)

- Outbound network requests are restricted to: updater endpoint, optional model-registry metadata fetch (user-initiated), documentation links (user-initiated).
- All dictation, processing, vocabulary, history, and analytics are local.
- Telemetry is **off** by default, and when off, no analytics pings leave the device — ever.
- Settings surface an "Open data folder" and a one-click "Delete all data" action.

## 12. Success metrics (post-v1)

- Onboarding completion rate ≥ 85%.
- 7-day retention ≥ 55%.
- Median dictation session length ≥ 60 s.
- Crash-free sessions ≥ 99.5%.

## 13. Risks & mitigations

| Risk | Mitigation |
|------|-----------|
| Parakeet ONNX runtime complexity on heterogeneous hardware | Provider interface; ship CPU-safe fallback; surface hardware diagnostics in Settings → Advanced |
| Text insertion blocked by some apps (UAC, secure desktops) | Detect failure and auto-fall-back to keystroke simulation; surface error with copy-to-clipboard action |
| Global hotkey conflicts with user apps | Conflict detection on bind; Pause Shortcuts toggle |
| Model download size (~1–3 GB) deters onboarding | Clear size disclosure before download, resumable downloads, optional "Skip for now" path |
| Users expecting cloud-grade LLM cleanup | Be explicit in onboarding: post-processing quality scales with local LLM presence |

## 14. Out of scope (explicit)

Voice commands that drive the OS; shared team vocabulary; non-Windows platforms; cloud backup; speaker diarization (deferred to v1.1).

## 15. Release gates

- All §9 performance targets met on the reference rig.
- Onboarding passes in < 3 minutes including model download on 50 Mbps.
- No P0/P1 bugs in core pipeline.
- Installer passes signed-build dry run.
- Privacy copy reviewed by legal.
