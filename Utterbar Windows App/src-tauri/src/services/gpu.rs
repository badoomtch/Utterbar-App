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
