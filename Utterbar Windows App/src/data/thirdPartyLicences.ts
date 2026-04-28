/**
 * Third-party attribution surface — bundled copy of
 * /THIRD_PARTY_LICENCES.md at repo root.
 *
 * Kept as a TypeScript string constant (rather than a bundler
 * `?raw` import of the .md) so Vite doesn't need fs-allow-list
 * gymnastics to read above the project root. The trade-off is
 * that this file MUST be kept in sync with the canonical .md;
 * a CI lint step could enforce that later.
 *
 * Rendered at Settings → About → Third-party licences as
 * preformatted plain text — no Markdown renderer, no syntax
 * highlighter, no XSS surface to worry about.
 *
 * Last synced from THIRD_PARTY_LICENCES.md on 2026-04-28.
 */
export const THIRD_PARTY_LICENCES_MD = `# Third-Party Licences

Utterbar is built on a stack of open-source libraries and pre-trained machine-learning models. This document records each component, its licence, and the attribution requirement (if any), as verified from the upstream source files.

The same content lives at /THIRD_PARTY_LICENCES.md in the repo. This bundled copy is what Settings → About → Third-party licences renders inside the app.

---

## Pre-trained models

### NVIDIA Parakeet TDT v3 (English ASR)

- Used for: primary speech-to-text recogniser on the dictation path.
- Source: https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3
- Licence: CC-BY-4.0
- Attribution: NVIDIA Corporation. Required by CC-BY-4.0.

### OpenAI Whisper Large v3 (multilingual ASR)

- Used for: Meeting Mode transcription + multilingual fallback.
- Source: https://huggingface.co/openai/whisper-large-v3
- Licence: Apache 2.0
- Attribution: OpenAI. Apache 2.0 requires preservation of LICENSE / NOTICE files; these travel with the upstream tarball.

### Pyannote Speaker Segmentation 3.0 (v0.15.0+)

- Used for: Meeting Mode speaker diarisation — voice-activity + per-frame segment boundary detection.
- Source: https://huggingface.co/pyannote/segmentation-3.0
- Licence: MIT
- Copyright: © 2022 CNRS
- Attribution: the upstream LICENSE file is preserved alongside the runtime asset on disk at %APPDATA%\\Utterbar\\models\\diariser\\segmentation.LICENSE.

### NVIDIA NeMo TitaNet Small (v0.15.0+)

- Used for: Meeting Mode speaker diarisation — per-segment voice embedding for clustering.
- Source: https://huggingface.co/nvidia/speakerverification_en_titanet_large
- Licence: CC-BY-4.0
- Attribution: NVIDIA Corporation. Required by CC-BY-4.0.

---

## Native libraries

### sherpa-onnx (k2-fsa)

- Used for: the C++ runtime layer over ONNX Runtime that ASR and diarisation sit on top of.
- Source: https://github.com/k2-fsa/sherpa-onnx
- Licence: Apache 2.0

### sherpa-rs (thewh1teagle)

- Used for: Rust bindings over sherpa-onnx.
- Source: https://github.com/thewh1teagle/sherpa-rs
- Licence: MIT

### ONNX Runtime (Microsoft)

- Used for: the underlying neural-network inference engine.
- Source: https://github.com/microsoft/onnxruntime
- Licence: MIT

---

## Rust crate dependencies

The full dependency tree is recorded in src-tauri/Cargo.lock. Major direct dependencies:

- tauri (Apache-2.0 / MIT dual)
- tokio (MIT)
- rusqlite (MIT) + refinery (MIT)
- cpal (Apache-2.0 / MIT dual) — audio capture
- rubato (MIT) — sample-rate resampling
- symphonia (MPL-2.0) — audio decoding for file transcription
- reqwest (Apache-2.0 / MIT dual) + rustls (Apache-2.0 / ISC / MIT triple)
- serde + serde_json (Apache-2.0 / MIT dual)
- anyhow + thiserror (Apache-2.0 / MIT dual)
- windows + winreg (MIT) — Win32 bindings
- ed25519-dalek (BSD-3-Clause) — licence + updater signature verification
- keyring (Apache-2.0 / MIT dual) — DPAPI-backed credential storage
- tracing + tracing-subscriber (MIT)
- bzip2 (MIT) + tar (MIT) — model archive extraction
- sha2 + hex (MIT)
- hound (Apache-2.0) — WAV I/O

---

## Frontend dependencies

- React (MIT)
- Vite (MIT)
- Tailwind CSS (MIT)
- Radix UI primitives (MIT)
- Zustand (MIT)
- Framer Motion (MIT)
- Lucide React icons (ISC)
- Tauri JS API + plugins (Apache-2.0 / MIT dual)

---

Last verified: 2026-04-28 against upstream source files / Hugging Face model cards / GitHub release pages. Hashes for the model assets are pinned in src-tauri/src/services/inference/diarize_assets.rs and src-tauri/src/services/models.rs.
`;
