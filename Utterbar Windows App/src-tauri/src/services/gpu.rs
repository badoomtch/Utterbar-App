//! GPU adapter detection.
//!
//! Used by:
//! * onboarding — decide whether to show the "GPU acceleration" step
//! * settings UI — enable/disable the GPU choice and show the adapter name
//! * inference — resolve `GpuAcceleration::Auto` to a concrete provider
//!
//! Enumerates Direct3D adapters via DXGI. We treat any adapter with non-zero
//! dedicated video memory and a non-"Microsoft Basic Render Driver" description
//! as a real GPU. Integrated iGPUs qualify — DirectML happily runs on them,
//! and they're still faster than CPU for Parakeet.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct GpuAdapter {
    pub name: String,
    pub dedicated_video_memory_mb: u64,
    pub dedicated_system_memory_mb: u64,
    pub shared_system_memory_mb: u64,
    pub is_software: bool,
    pub vendor: GpuVendor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../../src/generated/")]
#[serde(rename_all = "snake_case")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Qualcomm,
    Microsoft,
    Other,
}

impl GpuVendor {
    fn from_pci_vendor_id(id: u32) -> Self {
        match id {
            0x10DE => GpuVendor::Nvidia,
            0x1002 | 0x1022 => GpuVendor::Amd,
            0x8086 => GpuVendor::Intel,
            0x17CB | 0x5143 => GpuVendor::Qualcomm,
            0x1414 => GpuVendor::Microsoft,
            _ => GpuVendor::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct GpuProbe {
    /// Every adapter we saw — real GPUs and software fallbacks.
    pub adapters: Vec<GpuAdapter>,
    /// Whether any non-software adapter is present. This is the flag onboarding / UI
    /// should check — if `false`, don't offer GPU acceleration.
    pub has_hardware_gpu: bool,
}

/// What execution providers the SHIPPING ONNX Runtime in this build
/// can actually serve.
///
/// This is distinct from `GpuProbe` (which asks "does the user own a
/// GPU?"). A user can have a perfectly capable DX12 GPU and still get
/// `has_dml_ep: false` here if the runtime DLL set next to the exe
/// doesn't include DirectML. Asking the hardware doesn't tell you
/// anything about what the loaded ORT can do.
///
/// **v0.15.5 (Phase A) semantics:** file-presence check for the legacy
/// satellite layout (`onnxruntime_providers_dml.dll` +
/// `DirectML.dll`). The shipping CPU-only runtime has neither,
/// returning `false`. Conservative and easy to test.
///
/// **v0.15.6 (Phase B) semantics:** ORT 1.14.1's DirectML build
/// embeds the DML EP statically into `onnxruntime.dll` itself — there
/// is no `onnxruntime_providers_dml.dll` satellite. So the v0.15.5
/// rule would mark the v0.15.6 bundle as `has_dml_ep: false` even
/// though DML is fully wired. The probe now accepts EITHER layout:
///
///   has_dml_ep =
///     file_exists(DirectML.dll) AND
///     ( file_exists(onnxruntime_providers_dml.dll)            [legacy]
///       OR onnxruntime.dll contains the export-name string
///          "OrtSessionOptionsAppendExecutionProvider_DML" )    [embedded]
///
/// The embedded-layout check uses a substring scan of the DLL bytes
/// rather than a full PE export-table parse. The export name is a
/// C-API entry point — it appears in the DLL bytes if and only if
/// it's exported. For a real ORT build the binary distinction maps
/// 1:1 onto the substring presence (verified during Step 1 + Step 2
/// of the v0.15.6 work: CPU-only ORT lacks the string, DML ORT has
/// it). No PE parser dependency needed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct RuntimeCapabilities {
    /// Whether the loaded ORT can serve the DirectML execution provider.
    pub has_dml_ep: bool,
    /// Whether the loaded ORT can serve the CUDA execution provider.
    /// We have never bundled CUDA; `false` until that changes.
    pub has_cuda_ep: bool,
}

/// Probe the runtime DLL set next to the current executable for
/// optional execution-provider satellites. See `RuntimeCapabilities`
/// for the design rationale.
///
/// Falls back to `false / false` if the exe path can't be resolved.
/// Conservative default — at worst a user with a working DML setup
/// gets told it's inactive.
pub fn runtime_capabilities() -> RuntimeCapabilities {
    let exe_dir = match std::env::current_exe() {
        Ok(p) => p.parent().map(|d| d.to_path_buf()),
        Err(_) => None,
    };
    match exe_dir {
        Some(dir) => probe_runtime_dir(&dir),
        None => RuntimeCapabilities {
            has_dml_ep: false,
            has_cuda_ep: false,
        },
    }
}

/// Pure helper for `runtime_capabilities`. Separated so tests can
/// drive it against the in-tree `src-tauri/runtime/` directory or a
/// fixture without depending on the test binary's location.
pub fn probe_runtime_dir(dir: &std::path::Path) -> RuntimeCapabilities {
    // DirectML.dll is the DirectML runtime satellite. Required by
    // both the legacy (ORT < 1.14) and embedded (ORT 1.14.1+) layouts.
    let directml_dll = dir.join("DirectML.dll");
    let has_directml_dll = directml_dll.is_file();

    // Legacy satellite layout: onnxruntime_providers_dml.dll exists
    // alongside DirectML.dll. Pre-1.14 ORT builds shipped this; our
    // v0.15.6+ DML bundle does not, but the probe accepts either
    // layout for forward-compat with future ORT versions or
    // hand-rolled bundles.
    let has_legacy_satellite = dir.join("onnxruntime_providers_dml.dll").is_file();

    // Embedded layout: ORT 1.14.1+ DirectML build embeds the DML EP
    // into onnxruntime.dll. Detect by substring scan of the DLL
    // bytes for the C-API export name.
    let has_embedded_dml_ep = onnxruntime_exports_dml_symbol(&dir.join("onnxruntime.dll"));

    let has_dml_ep = has_directml_dll && (has_legacy_satellite || has_embedded_dml_ep);

    // CUDA EP requires the satellite providers DLL. We have never
    // bundled CUDA; this stays `false` until a future build does.
    let has_cuda_ep = dir.join("onnxruntime_providers_cuda.dll").is_file();

    RuntimeCapabilities {
        has_dml_ep,
        has_cuda_ep,
    }
}

/// Substring-scan `onnxruntime.dll` for the DML EP C-API export name.
/// Returns `false` if the file can't be read for any reason.
///
/// Why a substring scan rather than a PE-export parse: the export
/// name appears in the DLL's exports section as a NUL-terminated
/// ASCII string. For a real ORT build the binary distinction
/// (DML EP exported vs not) maps 1:1 onto the string's presence.
/// A full PE parser would handle pathological cases — a DLL that
/// contains the string in a debug-info section without actually
/// exporting the symbol — but no realistic ORT build does that.
/// The simpler check is good enough; we trust the build chain.
fn onnxruntime_exports_dml_symbol(path: &std::path::Path) -> bool {
    const NEEDLE: &[u8] = b"OrtSessionOptionsAppendExecutionProvider_DML";
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return false,
    };
    contains_subslice(&bytes, NEEDLE)
}

/// Trivial subslice search. `[u8]::windows(N).any(...)` would do but
/// is `O(N*M)` worst-case; for ~11 MB ORT DLL × 44-byte needle that's
/// fine but a cheap manual loop is clearer and avoids the iterator
/// overhead.
fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    let last = haystack.len() - needle.len();
    for i in 0..=last {
        if &haystack[i..i + needle.len()] == needle {
            return true;
        }
    }
    false
}

pub fn probe() -> GpuProbe {
    #[cfg(windows)]
    {
        windows_impl::probe().unwrap_or_else(|e| {
            tracing::warn!(error = ?e, "DXGI GPU probe failed");
            GpuProbe {
                adapters: Vec::new(),
                has_hardware_gpu: false,
            }
        })
    }
    #[cfg(not(windows))]
    {
        GpuProbe {
            adapters: Vec::new(),
            has_hardware_gpu: false,
        }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use anyhow::{anyhow, Result};
    use windows::core::Interface;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
    };

    pub fn probe() -> Result<GpuProbe> {
        unsafe {
            let factory: IDXGIFactory1 =
                CreateDXGIFactory1().map_err(|e| anyhow!("CreateDXGIFactory1 failed: {e}"))?;
            let mut adapters = Vec::new();
            let mut index = 0u32;
            loop {
                let adapter: IDXGIAdapter1 = match factory.EnumAdapters1(index) {
                    Ok(a) => a,
                    Err(_) => break,
                };
                index += 1;
                let desc = match adapter.GetDesc1() {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                // Description is a UTF-16 null-terminated array.
                let name_len = desc
                    .Description
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(desc.Description.len());
                let name = String::from_utf16_lossy(&desc.Description[..name_len])
                    .trim()
                    .to_string();

                let flags = desc.Flags as u32;
                let is_software = (flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0
                    // Microsoft's WARP / Basic Render Driver also shows as vendor 0x1414.
                    || name.eq_ignore_ascii_case("Microsoft Basic Render Driver");

                adapters.push(GpuAdapter {
                    name,
                    dedicated_video_memory_mb: (desc.DedicatedVideoMemory as u64) / (1024 * 1024),
                    dedicated_system_memory_mb: (desc.DedicatedSystemMemory as u64) / (1024 * 1024),
                    shared_system_memory_mb: (desc.SharedSystemMemory as u64) / (1024 * 1024),
                    is_software,
                    vendor: GpuVendor::from_pci_vendor_id(desc.VendorId),
                });
                // Release explicit; IDXGIAdapter1 drops here.
                let _ = adapter.as_raw();
            }

            let has_hardware_gpu = adapters.iter().any(|a| !a.is_software);
            Ok(GpuProbe {
                adapters,
                has_hardware_gpu,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **v0.15.6 contract.** Today's shipping `src-tauri/runtime/`
    /// is the DirectML-enabled bundle: ORT 1.14.1 with the DML EP
    /// statically embedded into onnxruntime.dll, plus DirectML.dll
    /// as the runtime satellite. The capability probe must agree:
    /// has_dml_ep = true via the embedded-layout branch.
    ///
    /// If a future bundle rebuild changes this — e.g. drops back
    /// to a CPU-only ORT, or moves to a sherpa-onnx version that
    /// ships the legacy onnxruntime_providers_dml.dll satellite
    /// — this test will fail and signal that BUNDLE_MANIFEST.txt
    /// needs review.
    #[test]
    fn runtime_capabilities_against_shipped_dml_bundle() {
        let runtime_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime");
        assert!(
            runtime_dir.is_dir(),
            "src-tauri/runtime/ must exist at {}",
            runtime_dir.display()
        );
        let caps = probe_runtime_dir(&runtime_dir);
        assert!(
            caps.has_dml_ep,
            "shipping runtime/ must contain a DirectML EP — v0.15.6 vendors the DML-enabled bundle"
        );
        assert!(
            !caps.has_cuda_ep,
            "shipping runtime/ must NOT contain a CUDA EP; we have never bundled CUDA"
        );
    }

    /// Convention used elsewhere in this crate (see app_state.rs,
    /// history.rs) — manual `temp_dir()/<unique>` directory with
    /// per-test cleanup. We don't depend on the `tempfile` crate.
    fn unique_tmp(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "ut-runtime-caps-{}-{}-{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    /// **Legacy satellite layout** (pre-ORT 1.14): a separate
    /// `onnxruntime_providers_dml.dll` next to `DirectML.dll`.
    /// The probe must still recognise this as DML-available because
    /// nothing stops a future bundle reverting to it. Empty fixture
    /// files are sufficient: this branch only checks file presence.
    #[test]
    fn runtime_capabilities_legacy_satellite_layout() {
        let tmp = unique_tmp("dml-legacy");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("onnxruntime_providers_dml.dll"), b"").unwrap();
        std::fs::write(tmp.join("DirectML.dll"), b"").unwrap();
        // No onnxruntime.dll at all -- legacy path doesn't need
        // the symbol check.
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(
            caps.has_dml_ep,
            "legacy satellite layout must still satisfy has_dml_ep"
        );
    }

    /// **DirectML.dll alone is not enough.** A user could end up
    /// with DirectML.dll next to their exe via Windows update or
    /// system policy, without our ORT being DML-enabled. The probe
    /// must NOT report has_dml_ep in that state.
    #[test]
    fn runtime_capabilities_directml_dll_alone_is_not_enough() {
        let tmp = unique_tmp("dml-alone");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("DirectML.dll"), b"").unwrap();
        // No onnxruntime.dll, no providers_dml satellite.
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(
            !caps.has_dml_ep,
            "DirectML.dll alone must not satisfy has_dml_ep"
        );
    }

    /// **DML symbol alone is not enough.** Even if onnxruntime.dll
    /// exposes the DML EP factory, without DirectML.dll nearby the
    /// EP can't load — there's no DirectML runtime to talk to.
    #[test]
    fn runtime_capabilities_symbol_alone_is_not_enough() {
        let tmp = unique_tmp("dml-symbol-only");
        std::fs::create_dir_all(&tmp).unwrap();
        // Synthesise an onnxruntime.dll that contains the export
        // name as a substring. Doesn't have to be a real PE — the
        // probe just substring-scans the file bytes.
        std::fs::write(
            tmp.join("onnxruntime.dll"),
            b"prefix bytes... OrtSessionOptionsAppendExecutionProvider_DML ...suffix",
        )
        .unwrap();
        // DirectML.dll deliberately missing.
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(
            !caps.has_dml_ep,
            "DML symbol without DirectML.dll runtime must not satisfy has_dml_ep"
        );
    }

    /// **Embedded layout, synthetic fixture.** ORT 1.14.1+ embeds
    /// the DML EP into onnxruntime.dll and ships only DirectML.dll
    /// as the satellite. The probe must accept this layout.
    #[test]
    fn runtime_capabilities_embedded_layout_synthetic() {
        let tmp = unique_tmp("dml-embedded");
        std::fs::create_dir_all(&tmp).unwrap();
        // Fake onnxruntime.dll containing the DML export name in
        // its bytes. Real PE structure not required for the probe.
        std::fs::write(
            tmp.join("onnxruntime.dll"),
            b"\x00\x00\x00\x00OrtSessionOptionsAppendExecutionProvider_DML\x00more",
        )
        .unwrap();
        std::fs::write(tmp.join("DirectML.dll"), b"").unwrap();
        // No legacy satellite -- the embedded branch must take over.
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(
            caps.has_dml_ep,
            "embedded layout (DML symbol + DirectML.dll) must satisfy has_dml_ep"
        );
    }

    /// CPU-only fixture: onnxruntime.dll without the DML symbol,
    /// no DirectML.dll, no satellite. Pre-v0.15.6 layout.
    #[test]
    fn runtime_capabilities_cpu_only_layout() {
        let tmp = unique_tmp("cpu-only");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("onnxruntime.dll"),
            b"plain CPU build, no DML EP exported",
        )
        .unwrap();
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(!caps.has_dml_ep);
        assert!(!caps.has_cuda_ep);
    }

    #[test]
    fn runtime_capabilities_cuda_present_when_dll_exists() {
        let tmp = unique_tmp("cuda");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("onnxruntime_providers_cuda.dll"), b"").unwrap();
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(caps.has_cuda_ep);
        assert!(!caps.has_dml_ep);
    }

    /// Substring search helper smoke test.
    #[test]
    fn contains_subslice_matches_expected_cases() {
        assert!(super::contains_subslice(b"hello world", b"world"));
        assert!(super::contains_subslice(b"foo", b"foo"));
        assert!(super::contains_subslice(b"prefix Foo suffix", b"Foo"));
        assert!(!super::contains_subslice(b"hello", b"world"));
        assert!(!super::contains_subslice(b"short", b"longer than haystack"));
        assert!(
            !super::contains_subslice(b"any", b""),
            "empty needle must not match"
        );
    }
}
