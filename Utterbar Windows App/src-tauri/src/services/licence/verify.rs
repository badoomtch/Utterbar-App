//! Offline signature verification for the 96-byte licence blob.
//!
//! See docs/LICENCE_APP_HANDOFF Â§3. Verification runs on every launch against
//! the cached blob, with no network call â€” the signature is over
//! `sha256(payload)`, not the raw payload, matching the server's
//! `sodium_crypto_sign_detached($digest, $sk)`.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::domain::licence::LicencePayload;
use crate::util::result::OrliqError;

/// Ed25519 sandbox public key — payload version byte 1.
///
/// This is the SANDBOX key that shipped with v0.6.x–v0.8.x. We accept it
/// transitionally so existing installs keep working while the live key
/// (v2) rolls out via `licence:remint --all` on the server. After v0.9.0
/// saturates the install base (≥6 weeks), drop this constant and the
/// `try_with` fallback in `verify_signed_blob` so only v2 licences
/// verify.
pub const PUBLIC_KEY_BYTES_V1: [u8; 32] = [
    0xea, 0x5e, 0x9b, 0x4f, 0x93, 0x55, 0x33, 0x18, 0x89, 0xf8, 0x04, 0x10, 0xc0, 0xc2, 0x89, 0xa7,
    0xe1, 0x02, 0x32, 0xf0, 0x97, 0x76, 0xa2, 0x2b, 0xdb, 0x2b, 0x1d, 0x53, 0x27, 0x63, 0x2b, 0x04,
];

/// Ed25519 LIVE public key — payload version byte 2.
///
/// Generated for the live cut-over. The matching private key lives only
/// on the production licence-signing server (.env: LICENCE_PRIVATE_KEY_B64).
/// MUST be filled in before shipping the build that depends on it — a
/// build with all-zeros here will reject every v2 licence (no signature
/// can verify against the identity public key under `verify_strict`).
///
/// Generate via `php artisan licence:keygen` on a clean machine (NEVER
/// production) and paste the public-key bytes here AND
/// LICENCE_PUBLIC_KEY_B64 + LICENCE_PRIVATE_KEY_B64 into the server .env.
/// Also bump LICENCE_PAYLOAD_VERSION=2 server-side.
pub const PUBLIC_KEY_BYTES_V2: [u8; 32] = [
    0x10, 0x9c, 0x54, 0x05, 0x46, 0x44, 0x12, 0xf6, 0x16, 0xdc, 0x07, 0x6b, 0x50, 0xac, 0x94, 0x60,
    0x1c, 0x88, 0xc7, 0x50, 0xfc, 0x1f, 0x85, 0x78, 0x9a, 0x87, 0xec, 0x28, 0x62, 0x76, 0x3a, 0x57,
];


/// Verify a signed blob and return the parsed payload. Fails closed â€” every
/// branch that isn't "valid signature over the exact expected bytes" returns an
/// `OrliqError::Licence { kind: "verification_failed", ... }` or `"malformed"`.
///
/// The caller is responsible for feeding the raw 96 bytes; base64 decoding
/// happens in the cache/api layer.
pub fn verify_signed_blob(blob: &[u8]) -> Result<LicencePayload, OrliqError> {
    if blob.len() != 96 {
        return Err(OrliqError::Licence {
            code: "malformed".to_string(),
            message: format!("expected 96 byte blob, got {}", blob.len()),
        });
    }
    let (payload_bytes, sig_bytes) = blob.split_at(32);
    let sig = Signature::from_slice(sig_bytes).map_err(|e| OrliqError::Licence {
        code: "malformed".to_string(),
        message: format!("signature decode: {e}"),
    })?;
    let digest = Sha256::digest(payload_bytes);

    // Pick the verifying key by the payload's version byte. v1 is the
    // sandbox keypair, v2 is the live keypair. Anything else is treated
    // as "no key we trust signed this" — fail closed. We deliberately
    // do NOT fall back across versions: a v2 payload with a v1 signature
    // is malformed, not a valid licence, and accepting it would defeat
    // the rotation entirely.
    let version = payload_bytes[0];
    let pubkey = match version {
        1 => &PUBLIC_KEY_BYTES_V1,
        2 => &PUBLIC_KEY_BYTES_V2,
        other => {
            return Err(OrliqError::Licence {
                code: "verification_failed".to_string(),
                message: format!("payload advertises unknown key version {other}"),
            });
        }
    };

    let vk = VerifyingKey::from_bytes(pubkey).map_err(|e| OrliqError::Licence {
        code: "internal".to_string(),
        message: format!("bundled public key rejected: {e}"),
    })?;

    vk.verify(digest.as_slice(), &sig)
        .map_err(|_| OrliqError::Licence {
            code: "verification_failed".to_string(),
            message: format!("signature did not verify against bundled v{version} public key"),
        })?;

    parse_payload(payload_bytes)
}

fn parse_payload(p: &[u8]) -> Result<LicencePayload, OrliqError> {
    // Caller guarantees we already split off a 32-byte slice, but parse defensively
    // anyway so this function is reusable in isolation.
    if p.len() != 32 {
        return Err(OrliqError::Licence {
            code: "malformed".to_string(),
            message: format!("payload expected 32 bytes, got {}", p.len()),
        });
    }

    // Offsets match docs/LICENCE_APP_HANDOFF Â§3.1.
    // u48 BE at offset 2..8 â†’ widen to u64 by reading the top 2 bytes as u16 and
    // the bottom 4 bytes as u32.
    let issued_at_ms = (u64::from(u16::from_be_bytes([p[2], p[3]])) << 32)
        | u64::from(u32::from_be_bytes([p[4], p[5], p[6], p[7]]));

    Ok(LicencePayload {
        version: p[0],
        sku: p[1],
        issued_at_ms,
        nonce: u16::from_be_bytes([p[8], p[9]]),
        key_id: u64::from_be_bytes([p[10], p[11], p[12], p[13], p[14], p[15], p[16], p[17]]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_length() {
        let err = verify_signed_blob(&[0u8; 10]).unwrap_err();
        match err {
            OrliqError::Licence { code, .. } => assert_eq!(code, "malformed"),
            _ => panic!("wrong error kind"),
        }
    }

    #[test]
    fn rejects_bad_signature() {
        let blob = [0u8; 96];
        let err = verify_signed_blob(&blob).unwrap_err();
        match err {
            OrliqError::Licence { code, .. } => assert_eq!(code, "verification_failed"),
            _ => panic!("wrong error kind"),
        }
    }

    #[test]
    fn parse_payload_reads_fields() {
        // Hand-craft a payload with known offsets. Signature not exercised here.
        let mut p = [0u8; 32];
        p[0] = 1; // version
        p[1] = 1; // sku
                  // issued_at u48 BE: 0x00_00_01_23_45_67 = 4886374247
        p[2] = 0x00;
        p[3] = 0x00;
        p[4] = 0x01;
        p[5] = 0x23;
        p[6] = 0x45;
        p[7] = 0x67;
        // nonce
        p[8] = 0xBE;
        p[9] = 0xEF;
        // key_id BE
        p[10..18].copy_from_slice(&1234u64.to_be_bytes());

        let parsed = parse_payload(&p).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.sku, 1);
        assert_eq!(parsed.issued_at_ms, 0x0000_0123_4567);
        assert_eq!(parsed.nonce, 0xBEEF);
        assert_eq!(parsed.key_id, 1234);
    }

    /// End-to-end against a real sandbox blob fetched from `/api/licence/fetch`.
    /// Enable by dropping a base64 blob into the `UTTERBAR_TEST_BLOB_B64` env
    /// var and running `cargo test -- --ignored`. Left ignored so CI stays
    /// green without live sandbox credentials.
    #[test]
    #[ignore]
    fn e2e_sandbox_blob() {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let b64 = std::env::var("UTTERBAR_TEST_BLOB_B64")
            .expect("set UTTERBAR_TEST_BLOB_B64 to a base64 96-byte blob");
        let blob = STANDARD.decode(b64).expect("valid base64");
        let parsed = verify_signed_blob(&blob).expect("blob should verify");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.sku, 1);
    }
}
