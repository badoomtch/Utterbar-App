# Third-party notices

Utterbar is distributed under the MIT licence (see [LICENSE](LICENSE)). It
bundles, links, or downloads at runtime a number of third-party components,
each under its own licence. This file lists the significant ones and the
attributions that must be preserved when you redistribute Utterbar or a build
of it.

If you fork Utterbar, keep this file (and the model attribution below) intact.

---

## Speech models

### NVIDIA Parakeet-TDT / NeMo — CC-BY-4.0 (attribution required)

The recommended English speech models (`Parakeet v3` and `Parakeet v2`) are
NVIDIA NeMo **Parakeet-TDT** checkpoints, downloaded on demand from the
[k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) model releases.

These model weights are licensed under
**[Creative Commons Attribution 4.0 International (CC-BY-4.0)](https://creativecommons.org/licenses/by/4.0/)**.
CC-BY-4.0 **requires attribution**. Utterbar credits NVIDIA as the model author
in this file and in the in-app model catalogue. If you redistribute these
weights or a build that includes them, you must preserve that attribution.

- Model family: NVIDIA Parakeet-TDT (NeMo)
- Author: NVIDIA Corporation
- Licence: CC-BY-4.0

### OpenAI Whisper — MIT

The multilingual models (`Whisper base`, `Whisper small`, and larger variants)
are derived from **[OpenAI Whisper](https://github.com/openai/whisper)**,
exported to ONNX and downloaded on demand from the k2-fsa/sherpa-onnx model
releases.

- Author: OpenAI
- Licence: MIT

---

## Runtime and libraries

| Component | Role | Licence |
| --- | --- | --- |
| [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) (k2-fsa) | On-device ASR runtime + model exports (vendored under `src-tauri/vendor/sherpa-onnx`) | Apache-2.0 |
| [ONNX Runtime](https://github.com/microsoft/onnxruntime) (Microsoft) | Neural-network inference engine used by sherpa-onnx | MIT |
| [Silero VAD](https://github.com/snakers4/silero-vad) | Voice-activity detection (`silero_vad.onnx`, vendored) | MIT |
| [DirectML](https://learn.microsoft.com/windows/ai/directml/dml) (Microsoft) | Optional GPU acceleration on DirectX 12 hardware | Microsoft redistributable terms |
| [Tauri](https://github.com/tauri-apps/tauri) | Desktop application shell (Rust + WebView) | MIT / Apache-2.0 |
| [React](https://github.com/facebook/react) | Front-end UI framework | MIT |

Utterbar vendors a patched copy of `tauri-runtime-wry` at
`src-tauri/vendor/tauri-runtime-wry-2.10.1/` to work around an upstream IPC
crash; it is licensed under the same MIT / Apache-2.0 terms as Tauri.

---

## Everything else

The desktop app depends on many additional Rust crates and npm packages, and
the website on PHP (Composer) packages. These are overwhelmingly permissive
(MIT or Apache-2.0). The authoritative, exhaustive list — with exact versions
and licences — is the lockfiles in the repository:

- Rust: `Utterbar Windows App/src-tauri/Cargo.toml` and `Cargo.lock`
- Front-end: `Utterbar Windows App/package.json` and `pnpm-lock.yaml`
- Website: `Utterbar Website/composer.json` and `composer.lock`

To regenerate a full machine-readable licence report for the Rust
dependencies, install [`cargo-about`](https://github.com/EmbarkStudios/cargo-about)
and run it against `src-tauri/`.
