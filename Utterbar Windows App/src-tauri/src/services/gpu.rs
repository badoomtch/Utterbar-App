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
/// `has_dml_ep: false` here, because today's vendored runtime in
/// `src-tauri/runtime/` ships the CPU provider only — no
/// `onnxruntime_providers_dml.dll`, no `DirectML.dll`. Asking the
/// hardware doesn't tell you anything about what the loaded ORT can do.
///
/// **Phase A semantics (v0.15.5):** file-presence check next to the
/// running exe. Cheap, deterministic, no native API calls. The two
/// satellite DLLs are required by ORT's DML path; their absence is
/// definitive proof the EP is unreachable. A future build that ships
/// the DLLs will flip these to `true` automatically.
///
/// **Why not parse PE exports?** A previous design considered
/// introspecting `onnxruntime.dll` for the DML EP factory symbol. It's
/// more work, more failure modes, and produces the same answer for
/// every realistic ORT build — if the satellite DLLs are missing, the
/// EP is unusable regardless of what symbols `onnxruntime.dll`
/// advertises.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct RuntimeCapabilities {
    /// Whether the loaded ORT can serve the DirectML execution provider.
    pub has_dml_ep: bool,
    /// Whether the loaded ORT can serve the CUDA execution provider.
    /// Always `false` on Phase A — we have never bundled CUDA.
    pub has_cuda_ep: bool,
}

/// Probe the runtime DLL set next to the current executable for
/// optional execution-provider satellites. See `RuntimeCapabilities`
/// for the design rationale.
///
/// Falls back to `false / false` if the exe path can't be resolved
/// (e.g. an unusual launch context). That's the safe default — the
/// fallback to CPU is correct, the UI banner is correct, the only
/// loss is a user with a working DML setup is told it's inactive
/// when it could have worked. Phase A is paper-honest about today's
/// shipping reality, so this conservative answer is fine.
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
    // DirectML EP requires both files. Either missing means the EP
    // can't load. Names are pinned by the microsoft/onnxruntime DML
    // build artefact.
    let has_dml_ep =
        dir.join("onnxruntime_providers_dml.dll").is_file() && dir.join("DirectML.dll").is_file();

    // CUDA EP requires the satellite providers DLL. We've never
    // shipped the CUDA runtime; this is `false` today and stays
    // `false` until a CUDA build is explicitly bundled.
    let has_cuda_ep = dir.join("onnxruntime_providers_cuda.dll").is_file();

    RuntimeCapabilities {
        has_dml_ep,
        has_cuda_ep,
    }
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

    /// **v0.15.5 contract.** Today's shipping `src-tauri/runtime/`
    /// contains only `onnxruntime.dll`, `onnxruntime_providers_shared.dll`,
    /// and the two sherpa-onnx API DLLs. No DirectML satellite, no
    /// CUDA satellite. The capability probe must agree.
    ///
    /// Phase B (v0.15.6 DirectML) will swap the runtime DLL set and
    /// flip these expectations. When that change ships, this test
    /// must be updated alongside the new DLLs — that's the deliberate
    /// signal.
    #[test]
    fn runtime_capabilities_against_shipped_runtime_are_cpu_only() {
        let runtime_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime");
        assert!(
            runtime_dir.is_dir(),
            "src-tauri/runtime/ must exist at {}",
            runtime_dir.display()
        );
        let caps = probe_runtime_dir(&runtime_dir);
        assert!(
            !caps.has_dml_ep,
            "shipping runtime/ must NOT contain a DirectML EP; if v0.15.6 has landed, update this test"
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

    /// Confirm a fixture directory containing the DML satellite DLLs
    /// flips `has_dml_ep` to true. Uses an empty file as the test
    /// fixture — `probe_runtime_dir` only checks file presence (not
    /// PE structure), so an empty file is sufficient and keeps the
    /// test free of binary fixtures.
    #[test]
    fn runtime_capabilities_dml_present_when_both_dlls_exist() {
        let tmp = unique_tmp("dml-both");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("onnxruntime_providers_dml.dll"), b"").unwrap();
        std::fs::write(tmp.join("DirectML.dll"), b"").unwrap();
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(caps.has_dml_ep);
        assert!(!caps.has_cuda_ep);
    }

    /// One of the two DML DLLs is not enough — both must be present
    /// for the EP to load.
    #[test]
    fn runtime_capabilities_dml_absent_when_only_satellite_present() {
        let tmp = unique_tmp("dml-satellite-only");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("onnxruntime_providers_dml.dll"), b"").unwrap();
        // DirectML.dll deliberately missing.
        let caps = probe_runtime_dir(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        assert!(!caps.has_dml_ep);
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
}
