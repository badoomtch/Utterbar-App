# Third-Party Licences

Utterbar is built on a stack of open-source libraries and pre-trained machine-learning models. This document records each component, its licence, and the attribution requirement (if any), as verified from the upstream source files.

The same content is rendered inside the desktop app at **Settings → About → Third-party licences** so it ships with every install.

---

## Pre-trained models

### NVIDIA Parakeet TDT v3 (English ASR)

- **Used for:** primary speech-to-text recogniser on the dictation path.
- **Source:** <https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3>
- **Hosted at:** k2-fsa/sherpa-onnx releases (`sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8.tar.bz2`)
- **Licence:** CC-BY-4.0
- **Attribution:** NVIDIA Corporation. Required by CC-BY-4.0; this entry plus the rendered in-app surface satisfy that requirement.

### OpenAI Whisper Large v3 (multilingual ASR)

- **Used for:** Meeting Mode transcription + multilingual fallback.
- **Source:** <https://huggingface.co/openai/whisper-large-v3>
- **Hosted at:** k2-fsa/sherpa-onnx releases (`sherpa-onnx-whisper-large-v3.tar.bz2`)
- **Licence:** Apache 2.0
- **Attribution:** OpenAI. Apache 2.0 requires preservation of the LICENSE / NOTICE files and copyright notices, which travel with the upstream tarball Utterbar downloads.

### Pyannote Speaker Segmentation 3.0 (v0.15.0+)

- **Used for:** Meeting Mode speaker diarisation — voice-activity + per-frame segment boundary detection.
- **Source:** <https://huggingface.co/pyannote/segmentation-3.0>
- **Hosted at:** k2-fsa/sherpa-onnx releases (`sherpa-onnx-pyannote-segmentation-3-0.tar.bz2`)
- **Licence:** MIT
- **Copyright:** © 2022 CNRS
- **Attribution:** the upstream `LICENSE` file (MIT text + CNRS copyright) is preserved alongside the runtime asset on disk at `%APPDATA%\Utterbar\models\diariser\segmentation.LICENSE` so an end-user inspecting the install can see it directly.

### NVIDIA NeMo TitaNet Small (v0.15.0+)

- **Used for:** Meeting Mode speaker diarisation — per-segment voice embedding for clustering.
- **Source:** <https://huggingface.co/nvidia/speakerverification_en_titanet_large> (TitaNet small is the smaller variant of the same architecture, same licence)
- **Hosted at:** k2-fsa/sherpa-onnx releases (`nemo_en_titanet_small.onnx`)
- **Licence:** CC-BY-4.0
- **Attribution:** NVIDIA Corporation. Required by CC-BY-4.0; this entry plus the rendered in-app surface satisfy that requirement.

---

## Native libraries

### sherpa-onnx (k2-fsa)

- **Used for:** the C++ runtime layer over ONNX Runtime that ASR (Parakeet, Whisper) and diarisation (pyannote, TitaNet) sit on top of.
- **Source:** <https://github.com/k2-fsa/sherpa-onnx>
- **Licence:** Apache 2.0
- **Attribution:** preserved via the upstream LICENSE file inside the vendored sherpa-onnx bundle (`src-tauri/vendor/sherpa-onnx/`).

### sherpa-rs (thewh1teagle)

- **Used for:** Rust bindings over sherpa-onnx — the API surface Utterbar's inference services call.
- **Source:** <https://github.com/thewh1teagle/sherpa-rs>
- **Licence:** MIT
- **Attribution:** preserved via the published Cargo metadata + crate's own LICENSE file.

### ONNX Runtime (Microsoft)

- **Used for:** the underlying neural-network inference engine; loaded as a DLL alongside sherpa-onnx.
- **Source:** <https://github.com/microsoft/onnxruntime>
- **Licence:** MIT
- **Attribution:** preserved via the DLL metadata + Microsoft copyright notice in the upstream LICENSE.

---

## Rust crate dependencies

The full dependency tree is recorded in `src-tauri/Cargo.lock`. The major direct dependencies are:

- **tauri** (Apache-2.0 / MIT dual)
- **tokio** (MIT)
- **rusqlite** (MIT) + **refinery** (MIT)
- **cpal** (Apache-2.0 / MIT dual) — audio capture
- **rubato** (MIT) — sample-rate resampling
- **symphonia** (MPL-2.0) — audio decoding for file transcription
- **reqwest** (Apache-2.0 / MIT dual) + **rustls** (Apache-2.0 / ISC / MIT triple)
- **serde** + **serde_json** (Apache-2.0 / MIT dual)
- **anyhow** + **thiserror** (Apache-2.0 / MIT dual)
- **windows** + **winreg** (MIT) — Win32 bindings
- **ed25519-dalek** (BSD-3-Clause) — licence + updater signature verification
- **keyring** (Apache-2.0 / MIT dual) — DPAPI-backed credential storage
- **tracing** + **tracing-subscriber** (MIT)
- **bzip2** (MIT) + **tar** (MIT) — model archive extraction
- **sha2** + **hex** (MIT)
- **hound** (Apache-2.0) — WAV I/O

Each of these carries its own LICENSE file inside the published crate; running `cargo tree` against `Cargo.toml` enumerates the complete transitive list.

---

## Frontend dependencies

The desktop app's frontend ships React + Vite + Tailwind + Radix UI. The full list is recorded in `package.json`. Major direct dependencies:

- **React** (MIT)
- **Vite** (MIT)
- **Tailwind CSS** (MIT)
- **Radix UI primitives** (MIT)
- **Zustand** (MIT)
- **Framer Motion** (MIT)
- **Lucide React** icons (ISC)
- **Tauri JS API + plugins** (Apache-2.0 / MIT dual)

---

## How attribution is preserved at runtime

Utterbar's installer ships:

- This file (`THIRD_PARTY_LICENCES.md`) bundled into the install.
- The in-app **Settings → About → Third-party licences** surface, which renders this file's contents.
- The pyannote segmentation `LICENSE` file alongside the runtime asset at `%APPDATA%\Utterbar\models\diariser\segmentation.LICENSE` (specific MIT requirement that the licence text accompanies the model).
- The vendored `sherpa-onnx/include/...` LICENSE files inside the application bundle.

Removing or replacing any of these files violates the relevant upstream licence; please keep them intact.

---

*Last verified: 2026-04-28 against upstream source files / Hugging Face model cards / GitHub release pages. Hashes for the model assets are pinned in `src-tauri/src/services/inference/diarize_assets.rs` and `src-tauri/src/services/models.rs`.*
