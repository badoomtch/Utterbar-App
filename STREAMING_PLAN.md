# Streaming + long-form transcription plan

> Engineering brief for the work that turns Utterbar into a real long-form
> dictation tool. Picked up across multiple sessions; this doc is the
> contract between them.

**Status:** safety patches landed in v0.12.1-beta (capture cap lifted to
8 hours, transcribe-input guard added, silent truncation fixed). Phase 1
work has not started. The 5-minute transcribe guard exists *until*
Phase 1 ships; once streaming works the guard is removed.

**Trigger:** v0.12.0 user reproduced a hard crash on a 40-minute meeting
(mic + system audio mixed). Process ballooned to 12 GB RAM, processed for
5 minutes, then died. Crash dump (`utterbar.exe.73872.dmp`, 2026-04-27)
confirmed `FAIL_FAST_FATAL_APP_EXIT_c0000409` inside
`SherpaOnnxDecodeOfflineStream` — the offline transducer decoder hit
`abort()` after running out of memory or hitting an internal assertion
on a too-long input.

**Diagnosis:**
- Capture and loopback both buffer the entire recording in a single
  `Vec<i16>` until `stop()` is called. That part is fine — 40 min × 16 kHz
  × 2 bytes ≈ 76 MB per stream.
- The whole buffer is then handed to sherpa's
  `TransducerRecognizer::transcribe(sample_rate, &samples)` in **one
  batch call**.
- Sherpa-onnx's offline decoder allocates O(duration) memory inside
  ONNX Runtime for encoder hidden states + intermediate decoder states.
  At ~30 minutes the allocator hits a ceiling and the C runtime aborts.
- This kills any "just call transcribe()" approach, regardless of how
  much RAM is available. **The fix is structural: never feed the offline
  recogniser a long buffer in one call.**

**The wrong fix:** allocate more memory, set `ORT_TENSORRT_MAX_WORKSPACE_SIZE`
higher, switch to a smaller model. None of these address the
length-proportional growth.

**The right fix:** two parallel paths.

1. **Live (streaming) recognition during recording.** Sherpa-onnx ships
   `OnlineTransducerRecognizer` for Parakeet, which processes 320 ms
   chunks and emits tokens incrementally. Constant ~50 MB memory. The
   transcript appears as the user speaks, no end-of-recording delay.
2. **Offline refinement pass for higher accuracy.** After the recording
   stops, a background worker re-runs the *offline* model on
   VAD-segmented chunks (Silero VAD finds silence boundaries; each chunk
   is independently bounded). The transcript silently upgrades to the
   higher-quality version when the pass completes.

This puts Utterbar ahead of every consumer competitor:

| | Wispr Flow | MacWhisper | Superwhisper | Otter | **Utterbar** |
|---|---|---|---|---|---|
| Long-form works | ✓ | ✓ | ✓ | ✓ | ✓ |
| Live transcript during recording | — | — | — | ✓ | ✓ |
| Higher-accuracy refinement pass | — | — | — | — | ✓ |
| Fully on-device | ✓ | ✓ | ✓ | — | ✓ |
| Works on 4-hour recordings | (cap) | (memory) | (cap) | ✓ | ✓ |

---

## Phase 1 — Streaming recognition (the urgent path)

**Goal:** memory bounded, no crash on long recordings, live transcript
appears as the user speaks. Ships as v0.13.0-beta.

**Estimate:** 5–7 focused days.

### 1A — sherpa-rs streaming binding

The crate we use ([`sherpa-rs`](https://github.com/thewh1teagle/sherpa-rs))
exposes the offline `TransducerRecognizer` only. Sherpa-onnx's C++ API
has the matching online recogniser; we need a Rust binding for it.

**Decision: fork `sherpa-rs`, vendor at `src-tauri/vendor/sherpa-rs-streaming/`.**
Rationale:
- Same pattern as the existing `tauri-runtime-wry-2.10.1` vendor.
- Upstream PR is the right end-state but takes weeks; vendoring now
  unblocks us.
- Diff is small enough that re-syncing on a future upstream release is
  manageable.

**The C++ symbols we need to bind** (all in `sherpa-onnx-c-api.h`):
- `SherpaOnnxCreateOnlineRecognizer(config) -> *OnlineRecognizer`
- `SherpaOnnxCreateOnlineStream(recognizer) -> *OnlineStream`
- `SherpaOnnxOnlineStreamAcceptWaveform(stream, sample_rate, samples, n)`
- `SherpaOnnxIsOnlineStreamReady(recognizer, stream) -> int`
- `SherpaOnnxDecodeOnlineStream(recognizer, stream)`
- `SherpaOnnxGetOnlineStreamResult(recognizer, stream) -> *OnlineRecognizerResult`
- `SherpaOnnxOnlineStreamReset(recognizer, stream)`
- `SherpaOnnxDestroyOnlineRecognizer/Stream/...Result`

**The model we'll use:** Parakeet-TDT-0.6b ships a streaming variant in
the same sherpa-onnx model catalogue. Same accuracy as offline, packaged
for online inference. Add to the model catalogue in
[catalogue.rs](Utterbar%20Windows%20App/src-tauri/src/services/models/catalogue.rs)
as a new entry; existing offline-only catalogue entries stay.

**Open questions to resolve in the binding work:**
1. Is the C `*OnlineStream` Send + Sync? (Read sherpa-onnx source; if it's
   internally `std::shared_ptr` we can probably mark `Send` with care.)
2. Does sherpa intern the recogniser config strings, or do we need to
   keep our `CString` allocations alive for the recogniser's lifetime?
3. What's the chunk size the recogniser actually consumes? (320 ms or
   user-configurable?)

**Test bar before merge:**
- Unit test: feed 60 seconds of synthesised sine + silence, get a
  non-empty result, no leaks under `cargo test --release` with
  `MIRIFLAGS=-Zmiri-strict-provenance`.
- Bench: peak RSS during a 40-minute synthesised speech stays under
  200 MB.
- The `sherpa-rs-streaming` vendor compiles standalone with
  `cargo build` from inside its directory.

### 1B — Provider trait + streaming session

`AsrProvider` (the trait in
[services/inference/provider.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/provider.rs))
gains:

```rust
trait AsrProvider {
    // ...existing offline transcribe...

    /// Begin a streaming session. Returns an opaque handle the caller
    /// feeds audio chunks into. None means this provider doesn't
    /// support streaming (Whisper, Zipformer at the time of writing).
    async fn streaming_session(&self) -> Result<Option<Box<dyn StreamingSession>>>;
}

trait StreamingSession {
    /// Feed a 16 kHz mono i16 chunk. Returns any newly-emitted token
    /// text since the last call, or None if no progress.
    async fn push(&mut self, samples: &[i16]) -> Result<Option<String>>;

    /// Mark end of audio, drain remaining tokens. Returns the final
    /// segment text.
    async fn finish(&mut self) -> Result<String>;
}
```

**Implementations:**
- `SherpaParakeet` returns `Some(StreamingSession)` once the streaming
  model is loaded.
- `SherpaWhisper`, `SherpaZipformer` return `None` — they fall back to
  offline-with-VAD (see Phase 2). The dictation orchestrator handles
  the None case by chunking before calling the offline `transcribe()`.

**Live token feed wiring:**
- `commands::dictation::start_recording` opens a streaming session
  alongside the existing capture. The capture's `frames_rx` already
  exists ([capture.rs L33](Utterbar%20Windows%20App/src-tauri/src/services/audio/capture.rs#L33),
  marked `#[allow(dead_code)]` with a comment "kept for the streaming
  ASR path we expect to wire up next" — this is that path).
- A new event `EVT_STREAMING_TEXT { text, is_final }` fires for every
  push() call that returns Some.
- On stop, finish() drains, the final result is written to history as
  the *current* transcript (Phase 2 will overwrite this with the
  refined version).

### 1C — UI: live transcript

- **Floating bar:** the listening state's display gets a small text
  region that fills with tokens as they arrive. Ellipsises when long.
  Clears on stop. Done in
  [FloatingBar.tsx](Utterbar%20Windows%20App/src/routes/FloatingBar.tsx).
- **History entry: `pending_refinement` state.** The streaming text
  lands as the entry's body, with a small "Refining…" badge that
  Phase 2 lights up; for now (Phase 1 only) it just says "Live".
- **No text injection** during streaming — that's still done at stop,
  using the streaming-final result. (Streaming injection is a separate
  feature requiring incremental keystroke insertion; out of scope.)

### 1D — Lift the 30-min cap properly

**Already done in v0.12.1-beta.** Capture + loopback now permit 8 hours.
The transcribe-side guard (5 min hard ceiling) goes away once Phase 1
streaming lands — the streaming path has no length limit.

### 1E — End-to-end smoke test

**The repro:** record 40 minutes of mixed mic + system audio, stop, expect:
- Memory peak under 500 MB throughout.
- Transcript appears live during recording (small lag is OK, ≤2 s).
- On stop, history entry is complete with the streaming result and a
  "Refining…" badge (Phase 2 not yet lit; badge just sits there
  pre-Phase 2).
- No crash, no truncation.

**Cancel test:** stop mid-recording at 20 min. Expect: transcript
finalises in <1 s, no leak.

---

## Phase 2 — Offline refinement pass

**Goal:** the streaming transcript gets silently upgraded to the offline
model's higher-accuracy output. Ships as v0.14.0-beta.

**Estimate:** 3–4 days.

### 2A — Silero VAD bundled model

- Add `silero-vad-v5.onnx` (~2 MB) to the model catalogue alongside
  Parakeet/Whisper.
- Build a thin Rust wrapper that takes 16 kHz f32 frames and returns
  `Vec<(start_ms, end_ms)>` speech intervals.
- VAD runs on the same `ContextService` thread layout as the existing
  ASR providers — no cross-runtime concerns.

### 2B — Refinement worker

- New service `services::refinement` that owns a job queue.
- Each completed `history` row gets a refinement job enqueued at stop.
- Worker pops a job, loads the offline Parakeet model, VAD-segments the
  audio, runs `transcribe()` on each segment, joins the segments with
  spaces.
- Worker is **persistent**: `refinement_jobs` table mirrors `file_jobs`
  ([V0001__init.sql](Utterbar%20Windows%20App/src-tauri/src/db/migrations/V0001__init.sql)
  L95-106). Jobs survive app restart; on boot, requeue any
  `status='running'` rows as `'queued'`.
- Each refinement updates the history row's `processed_text` (the user-
  visible field) and stores the streaming version in
  `streaming_text_snapshot` (new column) for audit/diff.
- Concurrency: 1 worker. Refinement is CPU-heavy and shouldn't compete
  with active recording.

### 2C — UI for the two-pass flow

- **History row badge.** Three states:
  1. `Live` while streaming
  2. `Refining…` while the worker is running on this entry
  3. (no badge) when refinement complete
- **Diff display.** Optional, deferred — could ship without and just
  silently upgrade. If we ship the diff, it's a "refined" pill on the
  entry that, when clicked, shows old vs. new in a small modal.
- **Settings → Privacy: "Refine transcripts in background"** toggle.
  Default on. Off means the streaming output is the final transcript
  with no refinement.

### 2D — File transcription

The Files route ([services/files.rs](Utterbar%20Windows%20App/src-tauri/src/services/files.rs))
currently calls the offline transcribe directly. Replace with the same
VAD-chunked pipeline used by the refinement worker. This is essentially
"refinement worker, but the input is a file instead of a recording".

Same code path, same behaviour, same memory profile. A 4-hour podcast
becomes a queue of ~200 chunks; each chunk's memory is bounded; the
overall job's progress is reportable.

### 2E — Smoke test

- Record 40 min with refinement enabled. After stop, the entry shows
  `Refining…` for ~5–10 minutes (pure CPU on the offline model), then
  the badge clears and the transcript is the higher-accuracy version.
- Force-quit the app mid-refinement. On restart, the worker resumes from
  the persisted queue. Job completes.
- Drop a 2-hour MP4 into Files. Job runs without crashing.

---

## Phase 3 — Meeting-grade structure

**Goal:** speaker labels, structured output, optional summaries. The
"first product that does on-device meeting transcription, properly"
moment. Ships as v0.15.0-beta or later — only after Phase 2 is solid.

**Estimate:** 1–2 weeks. Don't start until Phase 2 has shipped and soaked.

### 3A — Diarisation

- Add `pyannote-segmentation-3.0.onnx` (~17 MB) to the model catalogue.
- Run it after VAD chunking but before transcription. Each VAD segment
  gets a speaker cluster ID.
- Speaker clustering: simple agglomerative clustering on per-segment
  embeddings. Number of speakers is auto-detected with a max of 6
  (configurable in Settings → Privacy → "Diarisation" section).

### 3B — Speaker-labelled history rendering

- History row body becomes a list of `{ speaker, text, start_ms }` rather
  than a single string.
- Default render: `Speaker 1: …\nSpeaker 2: …`.
- User can rename speakers per-meeting (right-click a speaker label →
  "Rename to…").

### 3C — Local LLM summary

- `llama.cpp` integration is already feature-flagged in the codebase
  (commented as "deliberately absent in 0.6, returns in 0.7" — bring it
  back).
- Settings → Privacy → "Generate meeting summaries" toggle, default
  off.
- After refinement completes, optionally pipe the transcript to a local
  Llama-3-8B-Instruct (or similar) with a prompt like "Summarise this
  meeting: action items, key decisions, open questions."
- Summary appears as a separate field on the history entry.
- All on-device. No cloud LLM ever — that would break the brand
  promise.

---

## Risks worth flagging upfront

1. **`OnlineTransducerRecognizer` accuracy may be lower than offline.**
   The streaming Parakeet model is trained slightly differently from the
   offline one. Phase 2's refinement pass is the safety net — the user
   sees the streaming transcript first (less accurate, immediate) and
   the refined version replaces it (more accurate, async). If the
   streaming model is *too* much worse, we adjust the wording in the UI
   ("Live preview…") to set expectations.

2. **VAD may miss speakers under heavy background noise.** Silero is
   trained for clean speech; in a noisy meeting room speech may bleed
   into "silence" segments. Test on a real noisy recording before
   shipping Phase 2. Fallback: cap chunks at 30 s regardless of VAD —
   the 30-second window is a hard upper bound.

3. **Refinement worker on a low-end CPU is slow.** A 40-minute recording
   may take 15+ minutes to refine on a 2-core 8 GB laptop. The badge UI
   needs to be honest ("~12 minutes remaining") so users don't think
   the app is broken.

4. **Diarisation on a 4-hour meeting can take hours of CPU.** Speaker
   clustering scales with the number of segments. Phase 3 needs a
   per-recording opt-in (not auto-on), so a user who just wants a quick
   transcript doesn't pay the diarisation cost.

5. **The vendored `sherpa-rs-streaming` will need re-syncing** on every
   upstream sherpa-onnx update. Same maintenance burden as
   `tauri-runtime-wry-2.10.1`. Document the rebase procedure inline in
   the vendor directory.

---

## What to read before starting Phase 1A

- [services/audio/capture.rs](Utterbar%20Windows%20App/src-tauri/src/services/audio/capture.rs)
  — note the `frames_rx` field already exists for streaming.
- [services/inference/provider.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/provider.rs)
  — the trait we extend.
- [services/inference/mod.rs L141-L185](Utterbar%20Windows%20App/src-tauri/src/services/inference/mod.rs)
  — the orchestrator and the long-input guard.
- [services/inference/sherpa_parakeet.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/sherpa_parakeet.rs)
  — the existing offline provider, model for the new streaming sibling.
- [src-tauri/vendor/tauri-runtime-wry-2.10.1/](Utterbar%20Windows%20App/src-tauri/vendor/tauri-runtime-wry-2.10.1/)
  — read the README to understand how the existing vendor pattern works.
- Sherpa-onnx C++ API:
  https://github.com/k2-fsa/sherpa-onnx/blob/master/sherpa-onnx/c-api/c-api.h
- Sherpa-onnx streaming Parakeet model card:
  https://k2-fsa.github.io/sherpa/onnx/pretrained_models/online-transducer/zipformer-transducer.html

---

## Phase release plan

| Version | Channel | Phase contents |
|---|---|---|
| 0.12.1 | beta | safety patches (this work) + settings clarity |
| 0.13.0 | mainline | Phase 1 streaming (after 0.12.x soak) |
| 0.13.x | beta | bug-fix soak on streaming |
| 0.14.0 | mainline | Phase 2 refinement |
| 0.14.x | beta | refinement + file-transcription soak |
| 0.15.0 | mainline | Phase 3 diarisation + summary |

Per [CLAUDE.md §5.7](CLAUDE.md) versioning convention: minor bumps are
mainline / stable, patch bumps are beta soak. No major changes to
versioning here.
