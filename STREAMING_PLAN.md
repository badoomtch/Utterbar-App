# Streaming + long-form transcription plan

> Engineering brief for the work that turns Utterbar into a real long-form
> dictation tool. Picked up across multiple sessions; this doc is the
> contract between them.

**Status:** safety patches landed in v0.12.1-beta (capture cap lifted to
8 hours, transcribe-input guard added, silent truncation fixed). Phase 1A
(streaming wrapper) and Phase 1B.1 (trait scaffolding) landed
in commits 9820575 and cc42b64. **Phase 1B.2 is the next session's
opener** — see "Where to pick up" at the bottom. The 5-minute transcribe
guard remains in place until 1B-1E lands and a smoke test on a 40-min
recording confirms the streaming path is live.

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

**STATUS UPDATE 2026-04-27 (post-recon):** the FFI we worried about
already exists. `sherpa-rs-sys-0.6.8` (transitive dep of `sherpa-rs`)
generates bindings against the *full* `c-api.h` via bindgen. Every
streaming symbol — `SherpaOnnxCreateOnlineRecognizer`,
`SherpaOnnxOnlineStreamAcceptWaveform`, `SherpaOnnxIsOnlineStreamReady`,
`SherpaOnnxDecodeOnlineStream`, `SherpaOnnxGetOnlineStreamResult`,
`SherpaOnnxOnlineStreamReset`, `SherpaOnnxOnlineStreamInputFinished`,
`SherpaOnnxDestroyOnlineRecognizer`, `SherpaOnnxDestroyOnlineStream` —
is reachable from `sherpa_rs_sys::`.

What's missing is only the **safe Rust wrapper**: an `OnlineRecognizer`
struct that mirrors the existing `TransducerRecognizer` (in `sherpa-rs`'s
[`transducer.rs`](https://docs.rs/sherpa-rs/latest/sherpa_rs/transducer/index.html))
but talks to the online recogniser. Single file, ~200 lines, lives
inside our own crate at
[services/inference/streaming/online_recognizer.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/streaming/online_recognizer.rs).

**Implementation plan:**

1. Add `sherpa-rs-sys = "0.6"` as a direct dep in our Cargo.toml (it's
   already a transitive dep, this just makes the path public to our
   code).
2. Mirror `sherpa-rs`'s `transducer.rs` shape:
   - `OnlineRecognizer { recognizer: *const sys::SherpaOnnxOnlineRecognizer }`
   - `OnlineRecognizerConfig { encoder, decoder, joiner, tokens, num_threads, sample_rate, feature_dim, decoding_method, provider, ... }`
   - `OnlineStream { stream: *const sys::SherpaOnnxOnlineStream }` —
     dropped via `SherpaOnnxDestroyOnlineStream`.
   - `Drop`, `Send`, `Sync` impls following the same pattern as the
     offline wrapper.
3. Public API:
   ```rust
   impl OnlineRecognizer {
       pub fn new(cfg: OnlineRecognizerConfig) -> Result<Self>;
       pub fn create_stream(&self) -> Result<OnlineStream>;
       pub fn is_ready(&self, stream: &OnlineStream) -> bool;
       pub fn decode(&self, stream: &mut OnlineStream);
       pub fn result(&self, stream: &OnlineStream) -> String;
       pub fn reset(&self, stream: &mut OnlineStream);
   }
   impl OnlineStream {
       pub fn accept_waveform(&mut self, sample_rate: u32, samples: &[f32]);
       pub fn input_finished(&mut self);
   }
   ```
4. Mark `OnlineRecognizer: Send + Sync` (the C++ object is internally
   thread-safe per sherpa-onnx docs; the *stream* is `Send` but not
   `Sync` — only one thread feeds a stream at a time).

**The model:** streaming Zipformer English (smaller, real-time on CPU,
well-tested) — `sherpa-onnx-streaming-zipformer-en-2023-06-26`. Add to
the model catalogue alongside the existing offline entries; the `kind`
discriminant on the catalogue entry tells the inference layer to use
the streaming or offline path. Streaming Parakeet exists too but the
Zipformer streaming export is more mature and the size is friendlier
(~70 MB vs ~600 MB).

**Test bar before merge:**
- Unit test: synth 60 s of sine + silence, push through, get a
  non-empty result, no segfaults / no double-frees.
- Mini-bench: 40 min of synthesised audio runs at <100 MB peak RSS.
- Cargo test passes for the existing offline path (sanity that we
  haven't broken it).

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

## Where to pick up (next session)

**Phase 1A is complete.** The safe Rust wrapper for sherpa's
`OnlineRecognizer` lives at
[src-tauri/src/services/inference/streaming/online_recognizer.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/streaming/online_recognizer.rs).
Cargo check + clippy are clean. The wrapper is dead-code from the
binary's perspective until 1B wires it in.

**Phase 1B.1 is complete.** `StreamingAsrProvider` + `StreamingSession`
traits are in
[provider.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/provider.rs)
alongside the existing `AsrProvider`. Marked `#[allow(dead_code)]`
because no impl exists yet.

**Phase 1B.2 is the next ticket.** Steps in order:

1. **Download the streaming Zipformer model and pin its SHA-256.**
   The model is at
   `https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-streaming-zipformer-en-2023-06-26.tar.bz2`.
   Untar; inspect the file layout (the offline Zipformer has names
   like `encoder-epoch-99-avg-1.onnx`; the streaming variant has its
   own naming convention — get the actual filenames from the tarball
   before writing the provider).

2. **Add the streaming Zipformer to the model catalogue** in
   [services/models.rs](Utterbar%20Windows%20App/src-tauri/src/services/models.rs).
   New entry with `engine: ModelEngine::ZipformerStreaming` (add this
   variant to the enum if it doesn't exist), the SHA from step 1, and
   the URL above.

3. **Write `services/inference/sherpa_zipformer_streaming.rs`** —
   the `StreamingAsrProvider` impl. Pattern: holds the
   `OnlineRecognizer` from Phase 1A, `start_session()` calls
   `recognizer.create_stream()` and wraps the resulting
   `OnlineStream` in a `StreamingSession` impl that converts i16 →
   f32, pushes via `accept_waveform`, drives the decoder with
   `is_ready` / `decode`, and reads via `result()`.

4. **Plumb the new provider into `build_provider`** in
   [services/inference/mod.rs](Utterbar%20Windows%20App/src-tauri/src/services/inference/mod.rs).
   Map `ModelEngine::ZipformerStreaming` → the new provider.

5. **Add `EVT_STREAMING_TEXT`** to
   [domain/events.rs](Utterbar%20Windows%20App/src-tauri/src/domain/events.rs).
   Payload: `{ text: String, is_final: bool }`.

6. **Wire `start_recording` in
   [commands/dictation.rs](Utterbar%20Windows%20App/src-tauri/src/commands/dictation.rs)** —
   if the loaded provider implements `StreamingAsrProvider`, spawn a
   task that drains `capture::frames_rx` (already exists, marked
   `#[allow(dead_code)]` with a comment "kept for the streaming ASR
   path we expect to wire up next" — this is that path). Each
   ~100-500 ms chunk → `session.accept_chunk()` → emit
   `EVT_STREAMING_TEXT` if the result grew. On `stop_recording`,
   `session.finish()`, write to history.

7. **Phase 1C** — UI work. Floating bar gets a small text region that
   fills with the live transcript; History writes the streaming text
   as the body and marks the entry `pending_refinement: true` so
   Phase 2 has something to flip.

8. **Phase 1D** — once 1B-1E pass an end-to-end 40-min recording,
   remove the 5-minute guard at the top of
   [services/inference/mod.rs `transcribe()`](Utterbar%20Windows%20App/src-tauri/src/services/inference/mod.rs).

9. **Phase 1E** — the smoke test. Record 40 min mic+loopback, expect
   memory under 500 MB throughout, transcript appearing live, no
   crash, history entry complete with `pending_refinement: true`.

The riskiest piece is step 6 — the async task lifecycle. The capture
side already has `frames_rx` and the streaming session is just an
async consumer of that channel. The hard parts are: (a) what happens
when the user cancels, (b) what happens when an `accept_chunk`
errors mid-recording, (c) ensuring the streaming task is properly
joined before `start_recording` returns success.

Use the existing tokio `select!` pattern that the rest of the
dictation flow uses; don't invent new task plumbing.

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
