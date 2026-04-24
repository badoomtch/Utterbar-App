# Utterbar — Technical Architecture

**Version:** 1.0
**Scope:** v1 Windows build

---

## 1. High-level shape

```
┌──────────────────────────────────────────────────────────────────┐
│                         Utterbar.exe (Tauri)                        │
│                                                                  │
│  ┌──────────────────────────┐   ┌────────────────────────────┐   │
│  │  WebView (React / TS)    │◄─►│  Rust core (tauri::State)  │   │
│  │  - Floating bar window   │   │  - audio, hotkeys, tray    │   │
│  │  - Main window (tabs)    │   │  - inference orchestrator  │   │
│  │  - Onboarding window     │   │  - db, settings, analytics │   │
│  └──────────────────────────┘   └─────────┬──────────────────┘   │
│                                           │                      │
│                                           ▼                      │
│                          ┌──────────────────────────────┐        │
│                          │  ASR provider interface      │        │
│                          │  ┌────────┐  ┌────────────┐  │        │
│                          │  │ ONNX   │  │ Py sidecar │  │        │
│                          │  │ backend│  │ (fallback) │  │        │
│                          │  └────────┘  └────────────┘  │        │
│                          └──────────────────────────────┘        │
└──────────────────────────────────────────────────────────────────┘

         SQLite (rusqlite) — %APPDATA%/Utterbar/data/orliq.db
         Models  — %APPDATA%/Utterbar/models/
         Audio   — %APPDATA%/Utterbar/recordings/    (opt-in retention)
         Logs    — %APPDATA%/Utterbar/logs/
         Config  — %APPDATA%/Utterbar/config/settings.json
```

## 2. Process topology

**Single process, multi-window, multi-worker.** All production functionality runs inside `Utterbar.exe`. Long-running jobs (model load, transcription, LLM post-processing) run on dedicated Tokio tasks and communicate with the UI via Tauri events.

Tauri windows:

| Window | URL | Role |
|--------|-----|-----|
| `floating` | `/floating` | Always-on-top movable bar |
| `main` | `/` | Settings, history, stats, modes, files |
| `onboarding` | `/onboarding` | First-run flow |

The main window is created lazy-on-demand; the floating bar is always present (hidden when paused).

## 3. Rust module map

```
src-tauri/src/
├── main.rs                  # Tauri builder, window setup, command registry
├── app_state.rs             # Global AppState + DI container
├── commands/                # tauri::command fns, one file per domain
│   ├── audio.rs
│   ├── dictation.rs
│   ├── history.rs
│   ├── modes.rs
│   ├── models.rs
│   ├── shortcuts.rs
│   ├── settings.rs
│   ├── vocabulary.rs
│   ├── files.rs
│   ├── analytics.rs
│   └── window.rs
├── services/
│   ├── audio/               # cpal-based capture, VAD, resample
│   │   ├── mod.rs
│   │   ├── capture.rs
│   │   ├── meter.rs
│   │   └── preprocess.rs
│   ├── hotkeys/             # global-hotkey integration
│   ├── tray/                # system tray
│   ├── inference/           # ASR provider trait + impls
│   │   ├── mod.rs
│   │   ├── provider.rs      # AsrProvider trait
│   │   ├── onnx_parakeet.rs # ort-backed ONNX provider
│   │   └── py_sidecar.rs    # python stdio fallback provider
│   ├── llm/                 # post-processing provider trait + impls
│   │   ├── mod.rs
│   │   ├── provider.rs      # LlmProvider trait
│   │   ├── llama_cpp.rs     # llama.cpp via `llama-cpp-2`
│   │   └── deterministic.rs # fallback pipeline
│   ├── context/             # active window, selection, clipboard
│   ├── text_inject/         # paste + keystroke simulation
│   ├── history/             # SQLite history queries
│   ├── modes/               # mode CRUD + auto-activation matcher
│   ├── vocabulary/          # words + replacements
│   ├── models/              # model registry, downloads, versioning
│   ├── settings/            # settings store + migration
│   ├── analytics/           # rollups + queries
│   ├── files/               # file-transcription queue
│   └── updater/             # update check + apply
├── db/
│   ├── mod.rs               # pool, migrations
│   └── migrations/          # 0001_init.sql, …
├── domain/                  # shared types mirrored to TS
│   ├── mod.rs
│   ├── mode.rs
│   ├── history_entry.rs
│   ├── settings.rs
│   └── events.rs            # event payloads
├── platform/
│   └── windows/             # Win32-specific helpers (UIA, SendInput, etc.)
└── util/
    ├── paths.rs
    ├── logging.rs
    └── result.rs
```

## 4. Frontend module map

```
src/
├── main.tsx                 # Vite entry — dispatches by route
├── routes/
│   ├── FloatingBar.tsx
│   ├── Onboarding.tsx
│   ├── Dashboard.tsx
│   ├── History.tsx
│   ├── Modes.tsx
│   ├── Vocabulary.tsx
│   ├── Files.tsx
│   └── settings/
│       ├── General.tsx
│       ├── Audio.tsx
│       ├── Models.tsx
│       ├── Shortcuts.tsx
│       ├── TextOutput.tsx
│       ├── Privacy.tsx
│       ├── Advanced.tsx
│       └── About.tsx
├── components/              # shadcn-wrapped primitives + domain atoms
│   ├── ui/                  # Button, Dialog, Tabs, etc. (shadcn)
│   ├── bar/                 # FloatingBar atoms
│   ├── modes/
│   ├── history/
│   ├── charts/              # recharts wrappers
│   └── common/
├── hooks/
│   ├── useIpc.ts            # typed invoke + event bridge
│   ├── useDictationState.ts
│   ├── useSettings.ts
│   └── useMeter.ts
├── lib/
│   ├── ipc.ts               # typed wrapper around @tauri-apps/api
│   ├── events.ts            # event name constants
│   └── format.ts
├── state/                   # Zustand stores
│   ├── dictation.ts
│   ├── settings.ts
│   ├── modes.ts
│   └── models.ts
├── design/
│   ├── tokens.css           # CSS vars (colour, radius, motion)
│   ├── motion.ts            # Framer spring presets
│   └── sounds.ts            # Audio cue registry
└── styles/
    └── globals.css
```

## 5. Data flow — a dictation

```
[User presses hotkey]
       │
       ▼
HotkeyService (Rust)  ──emits──►  AppState.start_record()
       │
       ▼
AudioService starts cpal stream → ring buffer (int16 mono 16 kHz)
       │         │
       │         └──emits waveform frames──► FloatingBar
       ▼
[User stops]
       │
       ▼
AppState.stop_record() → finalise PCM buffer
       │
       ▼
ASR provider.transcribe(pcm) → raw transcript
       │
       ▼
Vocabulary + Replacements applied (deterministic)
       │
       ▼
If mode.postProcess != none:
   LLM provider.rewrite(raw, mode.prompt) → processed text
Else:
   Deterministic cleanup
       │
       ▼
TextInjectService.deliver(processed, strategy)
   1. copy to clipboard
   2. SendInput Ctrl+V
   3. (if flagged) keystroke fallback
   4. (if flagged) restore previous clipboard
       │
       ▼
HistoryService.insert(entry)  +  AnalyticsService.record(entry)
       │
       ▼
Emit `dictation:complete` → FloatingBar success state
```

Every step is cancellable via the Cancel hotkey; cancellation unwinds the pipeline and frees audio/model resources cleanly.

## 6. ASR provider abstraction

```rust
#[async_trait]
pub trait AsrProvider: Send + Sync {
    fn id(&self) -> &str;                               // "parakeet-en", "parakeet-multi", ...
    fn supported_languages(&self) -> &[&'static str];
    async fn warm(&self) -> Result<()>;
    async fn transcribe(&self, audio: AudioBuffer, opts: AsrOptions)
        -> Result<AsrResult>;
    async fn unload(&self) -> Result<()>;
}

pub struct AsrResult {
    pub text: String,
    pub words: Option<Vec<WordTiming>>,
    pub language: Option<String>,
    pub confidence: Option<f32>,
    pub model_version: String,
    pub duration_ms: u32,
}
```

Two v1 implementations:

1. **`onnx_parakeet`** — primary. Loads a Parakeet ONNX export via `ort` (ONNX Runtime). CUDA EP preferred, DirectML second, CPU third. Auto-detects best EP at warm time.
2. **`py_sidecar`** — fallback for users whose hardware/model combo doesn't work in pure Rust. Spawns a Python child process with NeMo installed, communicates over stdin/stdout length-prefixed JSON. Managed lifecycle; auto-restart on crash.

The Model Manager knows which backends a given model file can run with and picks the right provider at load time.

## 7. LLM provider abstraction

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn is_available(&self) -> bool;
    async fn rewrite(&self, input: &str, system: &str, ctx: &ProcContext)
        -> Result<String>;
}
```

Implementations:

- **`llama_cpp`** — `llama-cpp-2` crate, GGUF models, user-installed. Optional.
- **`deterministic`** — always available. Punctuation via rule-based post-edit; casing via sentence detection; filler-word removal via token list; mode-specific templates (e.g., Email → greeting/closing injection only if absent).

## 8. Context service

Reads (Windows-only, best-effort):

- Active window (GetForegroundWindow → GetWindowTextW + GetWindowThreadProcessId → QueryFullProcessImageNameW).
- Window class name.
- Selected text via UIAutomation `TextPattern` when available; falls back to `Ctrl+C` snapshot-then-restore when the user has explicitly enabled it.
- Clipboard contents (length-capped, image-skipped).

All reads are gated by per-mode and global privacy toggles. The captured payload is stored alongside the history entry under `context_payload` for user inspection.

## 9. Text injection

Primary path: `SetClipboardData` → `SendInput(Ctrl+V)`.
Fallback: `SendInput` with synthesized scancodes character-by-character (slower, but works in hostile apps).
Auto-detection of failure: if clipboard content isn't consumed within 300 ms AND focus hasn't changed, fall back.

Optional: restore previous clipboard after a 1.5 s delay.

## 10. Global hotkeys

Uses the `global-hotkey` crate. Bindings are stored in settings and rebuilt on change. Mouse-button bindings use `rdev` for low-level hooks when the user opts in.

## 11. Persistence

- **Settings:** JSON file (`settings.json`) with a versioned schema and a migration runner. Atomic writes (temp-file + rename).
- **SQLite:** history, modes, vocabulary, replacements, analytics rollups, model registry cache. Migrations run at startup via `refinery`. See [SCHEMA.md](SCHEMA.md).
- **Audio:** opt-in; stored as 16 kHz mono `.wav` or `.ogg`; retention policy enforced by a daily sweep.

## 12. Windows integration

- **WebView2** via Tauri.
- **DPI:** per-monitor v2; React honours `devicePixelRatio` for waveform canvas.
- **Startup:** registered via `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **Tray:** `tauri-plugin-tray` with dynamic submenu for modes.
- **Single-instance:** `tauri-plugin-single-instance` — second launch focuses existing.
- **Updater:** `tauri-plugin-updater` v2.

## 13. Configuration and data paths

| Purpose | Path |
|---------|------|
| Config | `%APPDATA%/Utterbar/config/settings.json` |
| DB | `%APPDATA%/Utterbar/data/orliq.db` |
| Models | `%APPDATA%/Utterbar/models/<id>/…` |
| Recordings | `%APPDATA%/Utterbar/recordings/YYYY/MM/…` |
| Logs | `%APPDATA%/Utterbar/logs/utterbar.log` |
| Exports | `%USERPROFILE%/Documents/Utterbar Exports/` |

## 14. State management (frontend)

**Zustand** for local UI state. **Tauri events** are the source of truth for backend state (recording phase, model state, hotkey state). Each Zustand store subscribes to its events on mount.

Pattern:

```ts
// src/state/dictation.ts
export const useDictation = create<DictationState>((set) => ({
  phase: 'idle',
  waveform: new Float32Array(0),
  startedAt: null,
  elapsedMs: 0,
  // … reducers driven by events
}));

// subscribe once at app root:
listen<DictationPhase>('dictation:phase', (e) =>
  useDictation.setState({ phase: e.payload }),
);
```

No React Query — backend is in-process and events are push-based, so pulling is unnecessary.

## 15. IPC contract

All `invoke` commands are typed via a generated `src/lib/ipc.ts`. A small build script emits TypeScript types from Rust `domain/` structs using `ts-rs`. Event payloads likewise.

## 16. Error handling

- Rust errors use `anyhow::Result` internally; commands return `Result<T, String>` with user-facing messages. A typed `OrliqError` enum distinguishes user-facing from diagnostic errors.
- Every service exposes a diagnostic snapshot consumed by **Settings → Advanced → Diagnostics**.
- Unhandled panics are caught, logged, and surfaced as a non-modal "something went wrong, logs saved" toast.

## 17. Testing strategy

- **Rust unit tests:** per-service, prefer pure-function extraction.
- **Rust integration:** a headless `AppState` with in-memory SQLite exercises full dictation flow with a stub ASR provider.
- **Frontend:** Vitest for stores, Playwright (Tauri) for windowed smoke tests.
- **Manual QA matrix:** see `docs/QA_MATRIX.md` (built during Phase 4).

## 18. Packaging

- `pnpm tauri build` produces `Utterbar_<version>_x64_en-US.msi` and a Setup.exe via NSIS.
- Signing handled by CI job reading cert from secret store (not committed).
- Models are **not** bundled in the installer — downloaded post-install via Model Manager to keep the installer small.

## 19. Threading model

- **Main thread:** Tauri event loop, window management.
- **Tokio runtime (multi-thread):** all async services.
- **Audio thread:** cpal owns its own realtime thread; pushes frames to a `tokio::sync::mpsc` bounded channel.
- **Inference worker:** dedicated long-running Tokio task with a `mpsc` command queue; serialises GPU access.

## 20. Observability

Local-only. Structured logs via `tracing` → rolling file in `%APPDATA%/Utterbar/logs/`. No network sinks. Log-level configurable in Settings → Advanced.
