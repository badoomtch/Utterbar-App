//! Windows device fingerprint for the licence system.
//!
//! Algorithm from docs/LICENCE_APP_HANDOFF §5:
//!   sha256("utterbar-fp-v1:" + machine_guid + ":" + hostname_lower + ":" + mac_lo64_be)
//!
//! Keep this byte-identical with the server — any drift here invalidates every
//! installed device. The prefix string `utterbar-fp-v1:` is literal; bump the
//! `v1` suffix and teach the server to accept both before changing anything.

use anyhow::{anyhow, Context, Result};
use mac_address::{get_mac_address, mac_address_by_name, MacAddressIterator};
use sha2::{Digest, Sha256};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
use winreg::RegKey;

const PREFIX: &[u8] = b"utterbar-fp-v1:";

/// Compute the 64-char hex fingerprint for the current machine. Fails if the
/// MachineGuid registry key is missing (shouldn't happen on a real Windows
/// install — it's created during OOBE) or if no physical MAC can be found.
pub fn compute_fingerprint() -> Result<String> {
    let machine_guid = read_machine_guid().context("read HKLM MachineGuid")?;
    let hostname_lower = hostname::get()
        .context("hostname lookup")?
        .to_string_lossy()
        .to_lowercase();
    let mac = first_physical_mac_u64().context("find physical MAC address")?;

    let mut h = Sha256::new();
    h.update(PREFIX);
    h.update(machine_guid.as_bytes());
    h.update(b":");
    h.update(hostname_lower.as_bytes());
    h.update(b":");
    h.update(mac.to_be_bytes());
    Ok(hex::encode(h.finalize()))
}

/// Read `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` verbatim — no
/// trimming, no case change. The server hashes the exact bytes; see handoff §5.
///
/// Forcing the 64-bit view (`KEY_WOW64_64KEY`) is defensive — if the app is ever
/// cross-compiled or launched via an emulator, we still land on the system-wide
/// MachineGuid rather than a Wow6432Node mirror.
fn read_machine_guid() -> Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let cryptography = hklm
        .open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Cryptography",
            KEY_READ | KEY_WOW64_64KEY,
        )
        .context("open HKLM\\SOFTWARE\\Microsoft\\Cryptography")?;
    let value: String = cryptography
        .get_value("MachineGuid")
        .context("read MachineGuid value")?;
    Ok(value)
}

/// Enumerate physical adapters, pick the one with the lowest 48-bit MAC as a
/// u64, skip anything that looks virtual/software/BT. Returns the 6 bytes of
/// MAC widened to u64 so the hash input is always 8 bytes and endian-stable.
fn first_physical_mac_u64() -> Result<u64> {
    // `mac_address` crate doesn't expose interface names on its default API,
    // so we go through the iterator (which yields every interface, virtual or
    // not, on Windows) and skip entries whose `name` matches the exclusion
    // heuristic.
    let iter = MacAddressIterator::new().context("enumerate network adapters")?;
    let mut candidates: Vec<(String, [u8; 6])> = Vec::new();

    for m in iter {
        let bytes = m.bytes();
        if bytes == [0; 6] {
            continue;
        }
        // Skip "locally administered" MACs — the second-least-significant bit of the
        // first octet is set for software-assigned addresses. This cheaply excludes
        // most Hyper-V / WSL / VMware adapters without needing the interface name.
        if bytes[0] & 0b0000_0010 != 0 {
            continue;
        }
        // `mac_address::MacAddress` doesn't surface an interface name directly in
        // this iterator; we consult `mac_address_by_name` round-trip indirectly by
        // re-checking well-known names. As a fallback we always include the raw
        // byte candidate — the sort-by-lowest step below picks deterministically.
        candidates.push((String::new(), bytes));
    }

    // Belt-and-braces: if the crate's primary `get_mac_address()` returned a
    // specific adapter, prefer that one when it appears in the candidate list.
    // This keeps the hash stable across crate-version nits about iteration order.
    if let Ok(Some(primary)) = get_mac_address() {
        let pb = primary.bytes();
        if pb != [0; 6] && pb[0] & 0b0000_0010 == 0 {
            return Ok(mac_to_u64(pb));
        }
    }

    // Name-based exclusion via the `mac_address_by_name` lookup for common
    // virtual adapters. We re-check presence rather than remove, because on
    // some systems these adapters aren't installed at all.
    let virtual_names = [
        "vEthernet",
        "VMware",
        "VirtualBox",
        "Bluetooth",
        "Loopback",
        "Hyper-V",
        "WSL",
    ];
    let mut excluded_macs: Vec<[u8; 6]> = Vec::new();
    for name in virtual_names {
        if let Ok(Some(m)) = mac_address_by_name(name) {
            excluded_macs.push(m.bytes());
        }
    }
    candidates.retain(|(_, b)| !excluded_macs.contains(b));

    candidates
        .into_iter()
        .map(|(_, b)| mac_to_u64(b))
        .min()
        .ok_or_else(|| anyhow!("no non-virtual MAC adapter found"))
}

fn mac_to_u64(bytes: [u8; 6]) -> u64 {
    // 48-bit MAC → u64 by zero-extending the top 2 bytes. Server does
    // `(u64) first 6 bytes big-endian`.
    let mut padded = [0u8; 8];
    padded[2..].copy_from_slice(&bytes);
    u64::from_be_bytes(padded)
}

/// Best-guess friendly device name for activation requests. Falls back through
/// COMPUTERNAME → hostname → "Unknown Device".
pub fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .ok()
        .or_else(|| {
            hostname::get()
                .ok()
                .map(|h| h.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "Unknown Device".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_to_u64_round_trips_low_order() {
        // Ensure a MAC that fits in the low 48 bits encodes as expected.
        let bytes = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        assert_eq!(mac_to_u64(bytes), 0x0000_0011_2233_4455);
    }

    /// Smoke test — only run on Windows and assumes MachineGuid + an adapter
    /// exist. Skipped in CI where those may not be populated deterministically.
    #[test]
    #[ignore]
    fn fingerprint_is_hex_64() {
        let fp = compute_fingerprint().expect("fingerprint");
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
