//! Offline signature verification for the 96-byte licence blob.
//!
//! See docs/LICENCE_APP_HANDOFF §3. Verification runs on every launch against
//! the cached blob, with no network call — the signature is over
//! `sha256(payload)`, not the raw payload, matching the server's
//! `sodium_crypto_sign_detached($digest, $sk)`.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::domain::licence::LicencePayload;
use crate::util::result::OrliqError;

/// Ed25519 sandbox public key from the handoff doc §1. **Rotate before live
/// launch** — ship v1.1 with both v1 and v2 keys, pick by payload `version`.
pub const PUBLIC_KEY_BYTES: [u8; 32] = [
    0xff, 0x56, 0x07, 0x91, 0xc9, 0x5c, 0x7c, 0xb5, 0x62, 0xe0, 0x75, 0xad, 0x64, 0x06, 0xb7, 0x6f,
    0xa5, 0xdd, 0xf0, 0xcc, 0x77, 0xb4, 0x3b, 0x47, 0x8e, 0x57, 0x1f, 0xda, 0x01, 0x4e, 0x24, 0x42,
];

/// Verify a signed blob and return the parsed payload. Fails closed — every
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

    let vk = VerifyingKey::from_bytes(&PUBLIC_KEY_BYTES).map_err(|e| OrliqError::Licence {
        code: "internal".to_string(),
        message: format!("bundled public key rejected: {e}"),
    })?;
    let sig = Signature::from_slice(sig_bytes).map_err(|e| OrliqError::Licence {
        code: "malformed".to_string(),
        message: format!("signature decode: {e}"),
    })?;

    let digest = Sha256::digest(payload_bytes);
    vk.verify(digest.as_slice(), &sig)
        .map_err(|_| OrliqError::Licence {
            code: "verification_failed".to_string(),
            message: "signature did not verify against the bundled public key".to_string(),
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

    // Offsets match docs/LICENCE_APP_HANDOFF §3.1.
    // u48 BE at offset 2..8 → widen to u64 by reading the top 2 bytes as u16 and
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
