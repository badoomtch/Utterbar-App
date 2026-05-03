# Smart Rewrite Local LLM Decision

**Status:** Decision-gated. **Not approved for implementation.** Investigation only.

**Recorded:** 2026-05-03 against `0153398` on `main`.

**Owner of the decision:** Jay.

This document persists the conclusion of the 2026-05-03 investigation so future Claude sessions do not re-investigate or accidentally start coding `llama.cpp` integration, model download paths, or prompt templates without Jay's explicit decisions in writing. If you are reading this and considering implementing local LLM rewrite, **stop and re-read the "Approval decisions required before code" section at the bottom**. None of those approvals exist yet.

---

## What this is about

A second pass of the existing **Smart rewrite** toggle (shipped in v0.16.4 beta) that runs a small local LLM via [llama.cpp](https://github.com/ggerganov/llama.cpp) to do "fix obvious wording / punctuation / ASR weirdness" beyond what the deterministic rules can handle.

The user-facing description today is *"Deterministic only — no AI model or cloud service"*. A local LLM v2 changes that to *"Optional local LLM rewrite, opt-in, off by default. No cloud."* The trade-off is a real model running on the user's machine: latency, RAM, disk space, and a non-trivial change to the marketing position.

---

## What already exists

Already built and shipped:

- **Deterministic Smart rewrite v1** is live in v0.16.4 beta (and merged to main; will ride to stable in v0.17.0). The toggle is `Settings.text_output.smart_rewrite: bool`, default `false` ([Utterbar Windows App/src-tauri/src/domain/settings.rs:422](../../Utterbar%20Windows%20App/src-tauri/src/domain/settings.rs#L422)). On regular dictation only — Meeting Mode is gated out by an outer `if mode.is_meeting_mode` branch and a source-grep guard test.
- **Provider trait** at [services/llm/provider.rs](../../Utterbar%20Windows%20App/src-tauri/src/services/llm/provider.rs): `LlmProvider` with `id() -> &str`, `is_available() -> bool`, and async `rewrite(input, system, ctx) -> Result<String>`. Stable for ≥1 year — designed exactly for a second implementor.
- **Deterministic provider** at [services/llm/deterministic.rs](../../Utterbar%20Windows%20App/src-tauri/src/services/llm/deterministic.rs): the rule-based filler-removal / whitespace-normalisation / sentence-case / terminal-punctuation pipeline. ASCII-fast, UTF-8-safe (multibyte regression tests pinned 2026-04-29). Pure string-in / string-out via `DeterministicProvider::rewrite_sync` (the sync wrapper lives at [commands/dictation.rs:3317](../../Utterbar%20Windows%20App/src-tauri/src/commands/dictation.rs#L3317)).
- **`LlamaCppProvider` stub** at [services/llm/llama_cpp.rs](../../Utterbar%20Windows%20App/src-tauri/src/services/llm/llama_cpp.rs): gated behind the `llama` Cargo feature; `try_new()` always bails with `"llama_cpp provider pending implementation"`. The `llama-cpp-2` crate is declared in [Cargo.toml](../../Utterbar%20Windows%20App/src-tauri/Cargo.toml) as `optional = true`. **Not** linked into release builds today.
- **Selector wiring** at [commands/dictation.rs:2842-2858](../../Utterbar%20Windows%20App/src-tauri/src/commands/dictation.rs#L2842): the post-processing block already routes `PostProcessing::Llm { prompt, .. }` through the async `LlmProvider::rewrite` path with a silent `unwrap_or(raw.clone())` fallback. The `Llm` variant exists in `domain::PostProcessing` ([domain/mode.rs:67](../../Utterbar%20Windows%20App/src-tauri/src/domain/mode.rs#L67)). Today no shipped mode uses it; modes use `None` or `Deterministic`.
- **History semantics**: `raw_text` (what ASR produced) and `processed_text` (what got typed/pasted) are persisted as separate columns. Users can already see the diff per row in History detail. This contract must survive any v2.

---

## Why this is a separate feature from deterministic v1

The deterministic toggle is a one-pass O(n) string transform with no model, no IO, no async surface beyond the call site. A local LLM rewrite is fundamentally different:

| Concern | Deterministic v1 | Local LLM v2 |
| --- | --- | --- |
| Latency | Sub-millisecond | Hundreds of ms to seconds, depending on model + GPU |
| Memory | Negligible | 1-3 GB RAM per inference |
| Disk | None | 1-3 GB GGUF on disk |
| Non-determinism | None | Sampling-temperature-dependent; can hallucinate or change meaning |
| Edit risk | Bounded by 4 hand-written rules | Free-form generation; one bad prompt loses the user's actual words |
| Cancellation | Not needed | Required (long-running token stream) |
| Marketing surface | "Deterministic only — no AI model or cloud service" | "Optional local LLM rewrite, opt-in, off by default" |
| Failure mode | Cannot fail | Network-free but: OOM, timeout, model missing, hallucination, drift from user intent |

Crucially, deterministic v1 stays available as the safe / default rewrite even if v2 ships. Users who don't want any LLM-anything keep using v1.

---

## Current public copy that would need softening

Less severe than the analytics decision (which had absolute "no telemetry" claims), but real. None of these are *absolute* "no LLM ever" claims; they read as "no cloud LLM, no surprise rewrites" today. An opt-in local LLM doesn't strictly violate them, but several would still need wording care:

| Surface | Current claim | Needs change for local LLM v2? |
| --- | --- | --- |
| `Utterbar Website/resources/views/pages/compare/superwhisper.blade.php:74` | `'Local, deterministic. No LLM rewrites of your speech.'` | **Yes — direct conflict.** Becomes "Local. Optional opt-in local LLM rewrite (off by default). No cloud LLM rewrites." |
| `Utterbar Website/resources/views/pages/compare/superwhisper.blade.php:164` | `'No LLM rewrites of your speech'` (bullet on "Choose Utterbar if…" list) | **Yes** — soften to "No LLM rewrites by default; opt-in only" or remove the bullet. |
| `Utterbar Website/resources/views/pages/compare/superwhisper.blade.php:230` | "You don't want LLM rewrites... Our default is honest transcription only." | Acceptable as-is *if* v2 is genuinely off by default and clearly opt-in. The "default" framing already permits this. |
| `Utterbar Website/resources/views/pages/compare/wispr-flow.blade.php:42` | Frames cloud-LLM rewriting as the competitor's bad behaviour | No change required: v2 is local, not cloud. |
| `Utterbar Website/resources/views/pages/docs/modes.blade.php:54` | "It runs entirely on rules, no LLM, no external service" | **Yes** — clarify this is about the deterministic rewrite specifically; add a separate paragraph for the optional local LLM if approved. |
| `Utterbar Website/resources/views/pages/docs/modes.blade.php:110` | **"Why no LLM-driven modes?"** with rationale that local LLMs take "seconds per dictation, not milliseconds" | **Yes** — heading and answer both need a rewrite. The rationale is honest but contradicts shipping v2. |
| `Utterbar Website/resources/blog/2026-04-26-why-local-dictation.md:34` | "If you want an AI ghost-writer, Utterbar isn't the answer." | Acceptable as-is — "ghost-writer" implies free-form prose, which v2 explicitly is not (constrained to "fix wording, do not change meaning"). |
| `Utterbar Windows App/src/routes/Onboarding.tsx:214` | "No cloud transcription. No cloud post-processing. No telemetry by default." | No change required — "no cloud" survives a local LLM. |
| `Utterbar Windows App/src/routes/settings/TextOutputSettings.tsx` (toggle copy on Smart rewrite v1) | "Deterministic only — no AI model or cloud service. Adds a small amount of processing time. Off by default." | **Yes** — this becomes the v1 sub-option's copy. The toggle structure changes from boolean to two-mode (v1 Deterministic vs v2 Local LLM), unless v2 is gated as a separate toggle. |

Short version: **at most ~6-8 surfaces** need copy edits, and most are softening rather than retraction. Order of magnitude smaller than the analytics rewrite.

---

## Recommended default

**Do not build local LLM rewrite until after v0.17.0 stable.** Specifically:

1. v0.17.0 stable bundles the soaked v0.16.x line (per [docs/decisions/opt-in-product-analytics.md](opt-in-product-analytics.md) and the existing TODO note from `bfa71c2`).
2. After v0.17.0 ships, soak it for at least one beta cycle (~7-10 days).
3. Only then consider local LLM v2 work, and only as a beta-channel feature first (probably v0.17.1 or v0.18.x beta, never straight to stable).
4. Deterministic v1 remains the default rewrite path. v2 is a parallel opt-in, not a replacement.

The single most important constraint: **deterministic v1 must remain available and selectable** even after v2 ships. Users who opt out of v2 keep the v1 behaviour they're already used to.

---

## Recommended branch breakdown if Jay approves later

Strict ordering. Branches 1-2 are decision/research only and ship no runtime code.

1. **Copy / privacy / spec confirmation** (`feature/smart-rewrite-llm-spec-pin`)
   - Soften the website copy surfaces flagged above.
   - Update the privacy.blade.php "What the desktop app sends" list to confirm: still nothing. Local LLM is local.
   - Update modes.blade.php "Why no LLM-driven modes?" to explain the v2 trade-off honestly.
   - **Reversible.** Stops here if the rewrites don't sound like Utterbar.

2. **Model packaging / download investigation** (`research/smart-rewrite-llm-model-pack`)
   - **No code commits.** Jay-driven research, results recorded in this document or a sibling memory note.
   - Test 2-3 candidate GGUF models on Jay's hardware: latency, RAM, edit-quality, hallucination rate.
   - Decide bundle vs first-run download. Bundling adds ~1-3 GB to the installer; downloading needs a model catalogue / manifest, but the desktop already has a `services/models.rs` ASR-model catalogue pattern that could be extended.
   - Decide CPU vs GPU provider expectations. The DirectML stack is already vendored; llama.cpp has its own DirectML / Vulkan / CUDA backends. **Do not change the existing ASR DirectML provider policy as part of this work.**

3. **Desktop provider implementation** (`feature/smart-rewrite-llm-provider`)
   - Flip the `llama` Cargo feature on for release builds.
   - Implement `LlamaCppProvider::try_new` to locate a GGUF under `%APPDATA%\Utterbar\models\llm\*` and load it via `llama-cpp-2`.
   - Implement `LlamaCppProvider::rewrite` with hard timeout, cancellation, and the safety constraints below.
   - **Behind a Cargo feature flag for the first beta** so a Free / non-opt-in user's binary doesn't even link the runtime.

4. **UI toggle / preview** (`feature/smart-rewrite-llm-ui`)
   - Either a second toggle row in Settings → Text Output ("Use local LLM rewrite (Pro)") or a select between "Off / Deterministic / Local LLM".
   - Onboarding step **only if** Jay decides v2 is on the default discovery path. Otherwise: Settings only, like every other Pro-gated feature.
   - "Preview" affordance: paste / type a sample, click "rewrite", see before/after side-by-side. Lets the user audit the model's behaviour before turning it on for real dictation.

5. **Beta release** (`chore(release): prepare 0.x.y beta` with v2 disabled by default)
   - Beta-only first. No `--announce` (per the v0.16.x cadence). Soak ≥7 days before any minor cut considers it.

---

## Recommended model criteria

The repo does NOT yet have a chosen model. **Do not pick one in this task.** When Jay gets to Branch 2 above, the candidate must satisfy:

- **Local-only.** GGUF format, runnable through llama.cpp / `llama-cpp-2`. No remote inference.
- **Small enough for normal Windows machines.** Recommend a ≤2 GB RAM working-set ceiling on a 1-3B parameter class. Larger models break the "instant dictation" promise.
- **Permissive licence for commercial desktop use.** Apache 2.0, MIT, or equivalent. Llama-family licences are commercial-friendly above certain MAU thresholds; the licence terms must be re-read at the time of selection. **Requires later verification** — model licences change.
- **Predictable latency.** P50 ≤500 ms on Jay's hardware for a typical 30-word dictation; P95 ≤2 s. If we can't hit this, the model is too large or the prompt is wrong.
- **Acceptable hallucination / edit-risk profile.** Tested against a fixed corpus of ~50 real Parakeet outputs; the v2 must (a) never invent words not in the input, (b) never delete substantive content, (c) never change meaning. Quantified as a per-100-rewrite "regret rate" — needs a number Jay accepts.
- **Decent instruction-following.** Must reliably honour "fix wording, do not change meaning". Smaller models often drift toward elaboration; the prompt + model combination must be tested for this specifically.

Specific model names (Phi-3-mini, Llama-3.2-1B, Qwen2.5-1.5B, etc.) are not pinned here on purpose. **Requires later verification** of current versions, licences, and benchmarks.

---

## Required safety constraints for any future implementation

These are non-negotiable and must be encoded as tests:

1. **Off by default.** Settings toggle ships `false`. No silent migration that flips it to true.
2. **Local-only.** No cloud fallback. The provider must hard-refuse if a network call would be needed.
3. **No transcript upload.** The `LlmProvider::rewrite` signature is in-process; do not add any path that exfiltrates the input or output.
4. **No prompt logging.** System prompts are private to the binary; user transcripts must never appear in logs (`tracing::*` calls, panic messages, crash dumps). Pin with a snapshot test: redact known transcript markers from any log line.
5. **No raw transcript in logs.** Stronger statement of the above. If a developer adds `tracing::info!(text = %input)` they lose log review.
6. **Hard timeout.** Configurable but capped (e.g. 10 s). Beyond the cap, fall back to the raw transcript silently with a status pill notice ("Smart rewrite timed out — using raw").
7. **Cancellation support.** A new dictation while a rewrite is in flight cancels the previous rewrite's token stream. The token loop must check a cancel signal each iteration.
8. **Before/after history preserved.** `raw_text` and `processed_text` continue to be persisted distinctly. The History UI continues to show the diff. The Settings → History detail view should label the row's rewrite path ("v1 deterministic" / "v2 local LLM").
9. **Clear UI that it may change wording.** The toggle / preview text must not understate the edit-risk. Suggested copy: "Local LLM rewrite (beta). Runs a small language model on your PC to fix wording, spacing, and obvious punctuation. No cloud. May occasionally rephrase or shorten — preview a few dictations before keeping it on."
10. **Never enabled for Meeting Mode by default.** The existing source-grep guard for v1 extends to v2 — meeting transcripts stay verbatim regardless of this toggle. Add a second guard test for v2 specifically.
11. **Deterministic v1 remains available.** Whatever the v2 toggle's shape, `Deterministic` must be a selectable option distinct from `LocalLlm`.
12. **Fallback on error.** Model missing → fall back to deterministic v1 (if user has it on) or raw (if not). Never block dictation; never error out to the user; surface a one-line status notice.

---

## Tests required if later implemented

Strict requirement before any v2 build ships beta:

- **Prompt contract tests** — given a fixed input, model output is bounded (length cap, no banned tokens, no XSS-like artefacts). Run against the chosen model in CI on a single fixed seed. Pin the canonical brief example ("um this is a test" → "This is a test.") with explicit allowed deviations.
- **No-network tests** — a mocked / stubbed `reqwest::Client` proves the provider never makes a remote call. Either a feature-flagged build that omits `reqwest` from the LLM crate, or a process-level network-deny in the test runner.
- **Timeout / cancellation tests** — synthetic slow model returns; provider hits the timeout cap and falls back. Synthetic in-flight cancel; provider stops generating tokens within ≤200 ms of cancel signal.
- **Fallback tests** — model file missing on disk → deterministic v1 (if on) or raw passthrough. Model file corrupt → same. Both surface a status pill notice without errors to the user.
- **Raw vs processed history tests** — for a v2-rewritten dictation, the History row shows both columns and the diff is non-empty.
- **Meeting Mode exclusion tests** — source-grep test for `if mode.is_meeting_mode` gating the LLM path. Plus an integration test that a meeting transcript is byte-identical to ASR output regardless of the toggle.
- **Benchmark / performance matrix** — a one-shot harness on Jay's hardware that records P50 / P95 latency, peak RAM, and edit-quality per candidate model. Results pinned in [docs/decisions/smart-rewrite-local-llm.md](smart-rewrite-local-llm.md) (this file) before approval.

---

## Approval decisions required before code

These must be answered on paper before Branch 1 starts. Default of "no decision" = "do not implement". Cross-reference: any future implementation prompt must cite which of these Jay has answered.

1. **Are we doing this at all?** Deterministic v1 already covers the chat / email use case. The product positioning today is "honest transcription, no AI ghost-writing". Local LLM v2 changes that subtly. Confirm yes.
2. **Model choice.** Specific GGUF, version, quantisation level, licence verified at the time of decision. Defer to Branch 2 research. Do not pick in this task.
3. **Bundle vs first-run download.** Bundling: ~+1-3 GB installer, no first-run dependency, model frozen to release. Download: smaller installer, needs a model manifest / catalogue endpoint, susceptible to host outage. Recommend first-run download to keep the installer light, model fetched from R2 the same way ASR models already are. Confirm.
4. **Model size budget.** Hard cap on disk + RAM. Recommend ≤2 GB RAM peak, ≤2 GB disk. Confirm.
5. **Latency budget.** P50 ≤500 ms, P95 ≤2 s on Jay's hardware. If a candidate model can't hit this, it's the wrong model. Confirm thresholds.
6. **CPU vs GPU provider expectations.** Llama.cpp ships several backends. The desktop already has DirectML for ASR; reusing that for LLM is plausible but **must not entangle with the existing ASR DirectML provider policy**. Recommend CPU-first for v2 with GPU as a stretch goal.
7. **Pro gating or all users?** Recommend Pro-only — gives the feature a clear value lane and limits the support-burden surface.
8. **Prompt strictness.** "Fix wording, do not change meaning" is the brief; the actual system prompt needs to be written, tested against the regret-rate corpus, and pinned in source with a contract test that asserts the literal prompt text.
9. **Fallback behaviour when model is missing / slow / fails.** Recommend silent fallback to deterministic v1 if the user has it on; otherwise raw passthrough. A status pill notice ("Smart rewrite v2 unavailable — using deterministic") on first occurrence per session.
10. **Privacy / marketing wording change.** Specifically: the modes.blade.php "Why no LLM-driven modes?" rewrite, the superwhisper.blade.php compare-row softening, the TextOutputSettings.tsx toggle copy. Jay must approve every line.
11. **Meeting Mode exclusion.** Recommend yes (matches v1) and pinned with a guard test. Any change to Meeting Mode behaviour is a separate decision.

Until ALL of these are answered (and approved by Jay in writing), do not start Branch 1.

---

## Explicit non-goals

These are out of scope for this feature even if Jay approves the rest:

- **No cloud LLM.** Ever. If Jay later decides to ship a cloud-rewrite option, it's a different feature with a different decision document.
- **No analytics tied to this feature.** v2's behaviour is opaque to us; we do not gain telemetry on what gets rewritten. (See [docs/decisions/opt-in-product-analytics.md](opt-in-product-analytics.md) — opt-in analytics is a separate decision.)
- **No meeting summarisation.** Even if the underlying model is capable, summarising meeting transcripts is a different product surface (meetings stay verbatim).
- **No diarisation.** Speaker labels remain off in shipping builds; this work doesn't change that.
- **No automatic rewrite without explicit opt-in.** No "we noticed you typed X, want us to rewrite future dictations?" promotion. Off by default means off.
- **No replacing deterministic v1.** v1 stays. v2 is a parallel opt-in.

---

## What this document is NOT

- Not implementation code. No JSON schema, no model manifest, no Rust services, no React components.
- Not a privacy / legal / marketing edit. Those branches happen later, conditionally.
- Not a green light. The default for every approval question above is "no".

If a future session decides to implement this, the corresponding implementation prompt should reference this document by path (`docs/decisions/smart-rewrite-local-llm.md`) and explicitly cite which approval decisions Jay has made. A prompt that does not cite these is incomplete.
