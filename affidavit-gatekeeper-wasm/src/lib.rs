//! Gatekeeper WASM: named `verify_witness` over dalek `verify_strict`.
//!
//! This crate deliberately does **not** link the fat provenance module
//! (`affidavit-wasm`). Surface is Ed25519 verify only — offline, zero PII,
//! no assemble/mine/PQC/JSON ABI.
//!
//! Law parity: byte-identical check to
//! `affidavit::ed25519_witness::verify_witness` (host) and
//! `affidavit-wasm::signature::ed25519` (`verify_strict`).

// no_std only on the deployable wasm cdylib. Host rlib / unit tests keep std
// so integration courts (size_court) link without a custom panic handler.
#![cfg_attr(all(target_arch = "wasm32", not(test)), no_std)]

use ed25519_dalek::{Signature, VerifyingKey};

#[cfg(all(target_arch = "wasm32", not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

/// Verification succeeded.
pub const VERIFY_OK: i32 = 0;
/// Public key bytes are not a valid curve point encoding.
pub const MALFORMED_PUBLIC_KEY: i32 = -1;
/// Reserved (dalek `Signature::from_bytes` on `[u8; 64]` is infallible).
pub const MALFORMED_SIGNATURE: i32 = -2;
/// Signature does not verify under `verify_strict`.
pub const VERIFICATION_FAILED: i32 = -3;
/// Null pointer passed for a required buffer.
pub const NULL_PTR: i32 = -4;

/// Strict Ed25519 witness check (native twin of host `verify_witness`).
///
/// Uses dalek [`VerifyingKey::verify_strict`] — rejects non-canonical /
/// malleated signatures that lenient `verify` would accept.
pub fn verify_witness_strict(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), i32> {
    let verifying = VerifyingKey::from_bytes(public_key).map_err(|_| MALFORMED_PUBLIC_KEY)?;
    let signature = Signature::from_bytes(signature);
    verifying
        .verify_strict(message, &signature)
        .map_err(|_| VERIFICATION_FAILED)
}

/// C / WASM export named `verify_witness`.
///
/// # Safety
///
/// `public_key` must point to 32 readable bytes; `signature` to 64 readable
/// bytes; `message` to `message_len` readable bytes (or null when
/// `message_len == 0`). Null required pointers return [`NULL_PTR`].
#[no_mangle]
pub unsafe extern "C" fn verify_witness(
    public_key: *const u8,
    message: *const u8,
    message_len: usize,
    signature: *const u8,
) -> i32 {
    if public_key.is_null() || signature.is_null() {
        return NULL_PTR;
    }
    if message_len > 0 && message.is_null() {
        return NULL_PTR;
    }
    // SAFETY: caller guarantees the pointed-to slices are valid for the
    // stated lengths for the duration of this call.
    let pk: &[u8; 32] = &*public_key.cast::<[u8; 32]>();
    let sig: &[u8; 64] = &*signature.cast::<[u8; 64]>();
    let msg = if message_len == 0 {
        &[][..]
    } else {
        core::slice::from_raw_parts(message, message_len)
    };
    match verify_witness_strict(pk, msg, sig) {
        Ok(()) => VERIFY_OK,
        Err(code) => code,
    }
}

/// ABI revision for host capability negotiation (starts at 1).
#[no_mangle]
pub extern "C" fn gatekeeper_abi_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn keypair() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    #[test]
    fn roundtrip_accepts_honest_sig() {
        let sk = keypair();
        let msg = b"gatekeeper witness";
        let sig = sk.sign(msg).to_bytes();
        assert_eq!(
            verify_witness_strict(&sk.verifying_key().to_bytes(), msg, &sig),
            Ok(())
        );
    }

    #[test]
    fn tamper_is_refused() {
        let sk = keypair();
        let sig = sk.sign(b"honest").to_bytes();
        assert_eq!(
            verify_witness_strict(&sk.verifying_key().to_bytes(), b"tampered", &sig),
            Err(VERIFICATION_FAILED)
        );
    }

    #[test]
    fn garbage_key_is_typed_refusal() {
        // Parity with tests/ed25519_strict_vectors.rs: [0xff; 32] is a
        // decompressible encoding under dalek, but verify_strict refuses it
        // as VerificationFailed (not MalformedPublicKey).
        let sk = keypair();
        let sig = sk.sign(b"m").to_bytes();
        assert_eq!(
            verify_witness_strict(&[0xffu8; 32], b"m", &sig),
            Err(VERIFICATION_FAILED)
        );
    }

    #[test]
    fn short_order_identity_key_is_malformed_or_failed() {
        // All-zero is the classic small-order / identity encoding class.
        let sk = keypair();
        let sig = sk.sign(b"m").to_bytes();
        let rc = verify_witness_strict(&[0u8; 32], b"m", &sig);
        assert!(
            rc == Err(MALFORMED_PUBLIC_KEY) || rc == Err(VERIFICATION_FAILED),
            "zero key must refuse, got {rc:?}"
        );
    }

    #[test]
    fn export_null_is_refused() {
        unsafe {
            assert_eq!(
                verify_witness(core::ptr::null(), b"".as_ptr(), 0, [0u8; 64].as_ptr()),
                NULL_PTR
            );
        }
    }
}
