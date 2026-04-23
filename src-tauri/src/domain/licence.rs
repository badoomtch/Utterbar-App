//! Licence domain types — exported to TypeScript via ts-rs.
//!
//! Wire shape mirrors the server contract at docs/LICENCE_APP_HANDOFF.md. The
//! Rust side always projects through `LicenceStatus` — the raw `signed_blob`
//! and `fingerprint` are internals, never surfaced to the UI (rule §11).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Current activation state. Drives whether Pro features are unlocked.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "tier", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/generated/")]
pub enum LicenceStatus {
    /// No valid licence cached. Every Pro feature falls back to free behaviour.
    Free,
    /// Cached signed blob verifies against the bundled public key. Server
    /// heartbeat may or may not have happened recently — we don't require it.
    Pro {
        /// Display form of the licence key, e.g. `UB-7QX4-K2YM-...`.
        display_key: String,
        /// Server row id, from the signed payload.
        licence_id: String,
        /// SKU slug, e.g. `utterbar-pro-lifetime`.
        sku: String,
        /// Epoch ms when the licence was issued. RFC 3339 rendering is a UI concern.
        issued_at_ms: String,
        /// Cap on simultaneous devices per the server (0 means unlimited — we still
        /// display it so the UI can show the cap).
        max_devices: u16,
        /// Known devices for this licence, including this one. Server-authoritative —
        /// populated on activate, refreshed on deactivate.
        devices: Vec<LicenceDevice>,
    },
}

/// One registered device. `this_device` marks the current install so the UI can
/// highlight it without recomputing the fingerprint client-side.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/")]
pub struct LicenceDevice {
    /// Hex SHA-256 fingerprint. Never shown to the user (rule §11) — sent to the
    /// frontend so the UI can identify "this device" across the list.
    pub fingerprint: String,
    pub device_name: String,
    /// ISO 8601 / RFC 3339 string, server-rendered.
    pub first_seen: String,
    pub this_device: bool,
}

/// Parsed payload from the 96-byte signed blob. Kept server-facing only — the UI
/// receives `LicenceStatus::Pro` instead. Lives in `domain` because the verify
/// module and the cache module both reconstruct it from bytes.
///
/// Fields are `#[allow(dead_code)]` because today only the fact of a successful
/// verification is consumed — the parsed values matter once we start surfacing
/// "issued on …" or gating by `version` during a key rotation, which is §12 of
/// the handoff doc rather than current work.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LicencePayload {
    pub version: u8,
    pub sku: u8,
    /// Unix epoch milliseconds. The byte layout uses u48 big-endian — we widen
    /// to u64 for Rust ergonomics (u48 has no direct type).
    pub issued_at_ms: u64,
    pub nonce: u16,
    pub key_id: u64,
}
