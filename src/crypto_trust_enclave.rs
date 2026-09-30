//! Secure Enclave signing adapter (macOS Security.framework; non-exportable P-256) for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Custody law: the affidavit owns only the KEY REFERENCE — `KeyId`, label,
//! and the public point. The private key is generated inside the Secure
//! Enclave (`kSecAttrTokenIDSecureEnclave`), is non-exportable
//! (`ctp:keyMaterialExportable false`), and never enters process memory, so
//! this module cannot and does not hand out key bytes. Signing is delegated
//! (`SecKey::create_signature` over `ECDSASignatureMessageX962SHA256` — the
//! enclave hashes SHA-256 internally and emits an X9.62 DER signature);
//! verification is certificate-independent and runs on the software side via
//! `crate::crypto_trust_es256::verify_es256`.
//!
//! Access control: keys are created with
//! `AccessibleWhenUnlockedThisDeviceOnly` + `DevicePasscode` +
//! `PrivateKeyUsage`. The constraint is the DEVICE PASSCODE, deliberately NOT
//! biometry — unattended runs never block on a Touch ID / Face ID prompt; the
//! unlocked-with-passcode login keychain satisfies the constraint.
//!
//! Persistence law: creating a key PERSISTS it in the keychain. Every creator
//! is paired with [`delete_enclave_key`], and every test here deletes what it
//! creates (drop guard + explicit deletion + post-delete refusal assert).
//!
//! Target safety: the module compiles on every target (the consumer declares
//! it under the `secure-enclave` feature on all platforms). On non-macOS the
//! public functions exist and refuse with
//! [`EnclaveError::UnsupportedPlatform`] — an honest typed refusal, never a
//! fake success.
//!
//! Keychain honesty: the security-framework crate exposes the data-protection
//! keychain location only behind its `OSX_10_15` feature (not enabled by the
//! consumer's default-feature dependency), so this adapter admits keys into
//! `Location::DefaultFileKeychain` (the login keychain), which is where
//! Secure Enclave tokens are persisted on macOS.
//!
//! Standing (witnessed 2026-09-28, darwin 25.2.0 arm64): PARTIAL_ALIVE.
//! Keygen inside the Secure Enclave is real — every refused persistence
//! attempt returned a fresh P-256 public key in the error payload — but the
//! keychain ADD from a non-Apple-signed binary is refused by the OS: plain
//! ad-hoc (file and data-protection keychain, with and without the
//! access-control object) fails with `-34018`
//! `errSecMissingEntitlement`; ad-hoc + `com.apple.security.application-groups`
//! fails with `-34018`; ad-hoc + `keychain-access-groups` is killed by the
//! platform (SIGKILL). The live tests below stay `#[ignore]`-gated honestly,
//! and the typed refusal carries the machine-readable unblock path in
//! [`ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE`].

// Consumed query columns (enclave.rq): provider_kind, target_os, exportable, alg_name.

use crate::crypto_trust_keys::KeyId;

/// Provider identity (`ctp:provider-SECURE-ENCLAVE` `ctp:providerKind`).
pub const PROVIDER_KIND: &str = "SECURE_ENCLAVE";
/// Target operating system admitted by the graph (`ctp:targetOs`).
pub const TARGET_OS: &str = "macos";
/// Whether key material may leave the provider (`ctp:keyMaterialExportable`):
/// `false` — the private key never leaves the enclave, and this module never
/// touches private bytes.
pub const KEY_MATERIAL_EXPORTABLE: bool = false;
/// Algorithm the enclave signs with (`ctp:supportedAlgorithm`
/// `ctp:algorithmName`).
pub const ALGORITHM_NAME: &str = "ES256";

/// Unblock recipe for Secure Enclave keychain persistence, carried verbatim
/// by the typed refusal: [`generate_enclave_key`] appends this text to any
/// `-34018` `errSecMissingEntitlement` failure. Witnessed 2026-09-28
/// (darwin 25.2.0 arm64): keygen succeeds inside the Secure Enclave, but the
/// OS refuses the keychain add from every ad-hoc-signed binary — plain ad-hoc
/// over the file keychain and the data-protection keychain (with and without
/// the access-control object), and ad-hoc +
/// `com.apple.security.application-groups`, all with `-34018`; ad-hoc +
/// `keychain-access-groups` is killed by the platform (SIGKILL, exit 137).
/// The only observed unblock is a real Apple Development signing identity
/// (`security find-identity -v -p codesigning` reported 0 valid identities on
/// the witnessing host, so identity provisioning is an operator step).
pub const ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE: &str = "\
Unblock for errSecMissingEntitlement (-34018): Secure Enclave keygen succeeds \
but keychain persistence requires a REAL Apple Development code signature. \
(1) Write ent.plist: <?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE plist \
PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"\
><plist version=\"1.0\"><dict><key>keychain-access-groups</key><array><string>\
<TEAMID>.affidavit.trust-plane</string></array></dict></plist> \
(2) codesign --force --sign \"Apple Development\" --entitlements ent.plist \
<test-binary> (3) cargo test --features secure-enclave live_ -- --ignored \
(or AFFI_ENCLAVE_KAT=1). Ad-hoc signatures are refused: plain ad-hoc and \
ad-hoc + com.apple.security.application-groups give -34018; ad-hoc + \
keychain-access-groups is SIGKILLed by the platform.\
";

/// The affidavit's custody handle for one enclave key: identity, label, and
/// the public point. Never carries private bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnclaveKeyRef {
    /// Trust-plane key id derived from the public-key fingerprint.
    pub key_id: KeyId,
    /// The keychain label the private key lives under.
    pub label: String,
    /// Public key in X9.63 uncompressed SEC1 encoding (`0x04 || X || Y`, 65
    /// bytes for P-256).
    pub public_key_sec1: Vec<u8>,
}

/// Typed refusals of the enclave adapter. Values, never panics.
#[derive(Debug, thiserror::Error)]
pub enum EnclaveError {
    /// The Security.framework (or, for verification, the software verifier)
    /// refused an operation; carries the typed message.
    #[error("security framework: {0}")]
    SecurityFramework(String),
    /// No enclave key exists under the requested label.
    #[error("enclave key not found: {0}")]
    KeyNotFound(String),
    /// The Secure Enclave provider requires macOS
    /// (`ctp:targetOs "macos"`); other targets are refused, never faked.
    #[error("unsupported platform: secure enclave requires macOS")]
    UnsupportedPlatform,
}

#[cfg(target_os = "macos")]
mod imp {
    use security_framework::{
        access_control::{ProtectionMode, SecAccessControl},
        base::Error as SfError,
        item::{ItemClass, ItemSearchOptions, KeyClass, Location, Reference, SearchResult},
        key::{Algorithm, GenerateKeyOptions, KeyType, SecKey, Token},
        passwords::AccessControlOptions,
    };

    use super::{EnclaveError, EnclaveKeyRef, ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE};
    use crate::crypto_trust_keys::{fingerprint_public_key, AlgorithmId, KeyId, PublicKeyMaterial};

    /// `errSecItemNotFound` (Security.framework, SecBase.h): the keychain
    /// search matched nothing. Restated locally because the consumer depends
    /// on `security-framework`, not `security-framework-sys`, and the
    /// high-level crate does not re-export the status code.
    const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

    /// `errSecMissingEntitlement` (Security.framework, SecBase.h): the host
    /// binary lacks the entitlement the keychain demands. Witnessed
    /// 2026-09-28 as the universal ad-hoc-signature refusal for Secure
    /// Enclave key persistence; the typed refusal carries
    /// [`ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE`] when this code is seen. `isize`
    /// because `SecKey::new` fails with `CFError`, whose `code()` is
    /// `CFIndex` (`isize`) — unlike the `base::Error::code()` (`OSStatus`,
    /// `i32`) the item-search paths use.
    const ERR_SEC_MISSING_ENTITLEMENT: isize = -34018;

    /// Maps a search failure: a missing key is the typed
    /// [`EnclaveError::KeyNotFound`], everything else stays a framework
    /// refusal with its message.
    fn map_search_error(label: &str, error: SfError) -> EnclaveError {
        if error.code() == ERR_SEC_ITEM_NOT_FOUND {
            EnclaveError::KeyNotFound(label.to_string())
        } else {
            EnclaveError::SecurityFramework(error.to_string())
        }
    }

    /// Finds the private enclave key stored under `label`.
    fn find_private_key(label: &str) -> Result<SecKey, EnclaveError> {
        let results = ItemSearchOptions::new()
            .load_refs(true)
            .class(ItemClass::key())
            .key_class(KeyClass::private())
            .label(label)
            .search()
            .map_err(|error| map_search_error(label, error))?;
        for result in results {
            if let SearchResult::Ref(Reference::Key(key)) = result {
                return Ok(key);
            }
        }
        Err(EnclaveError::KeyNotFound(label.to_string()))
    }

    /// Builds the custody handle the affidavit owns for a fresh key.
    fn custody_ref(label: &str, public_key_sec1: Vec<u8>) -> EnclaveKeyRef {
        let fingerprint = fingerprint_public_key(
            AlgorithmId::Es256,
            &PublicKeyMaterial::Es256Sec1(public_key_sec1.clone()),
        );
        EnclaveKeyRef {
            key_id: KeyId::from_fingerprint(&fingerprint),
            label: label.to_string(),
            public_key_sec1,
        }
    }

    /// Generates a P-256 key pair inside the Secure Enclave and returns the
    /// custody handle. The private key is non-exportable and persists in the
    /// keychain under `label` until [`delete_enclave_key`] removes it.
    pub fn generate_enclave_key(label: &str) -> Result<EnclaveKeyRef, EnclaveError> {
        let access_control = SecAccessControl::create_with_protection(
            Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly),
            (AccessControlOptions::PRIVATE_KEY_USAGE | AccessControlOptions::DEVICE_PASSCODE)
                .bits(),
        )
        .map_err(|error| EnclaveError::SecurityFramework(error.to_string()))?;

        let mut options = GenerateKeyOptions::default();
        options
            .set_key_type(KeyType::ec())
            .set_size_in_bits(256)
            .set_label(label)
            .set_token(Token::SecureEnclave)
            .set_location(Location::DefaultFileKeychain)
            .set_access_control(access_control);

        let private_key = SecKey::new(&options).map_err(|error| {
            if error.code() == ERR_SEC_MISSING_ENTITLEMENT {
                // The honest refusal carries its own unblock path: the exact
                // codesign command and entitlements that un-gate persistence
                // (see the standing note in the module header).
                EnclaveError::SecurityFramework(format!(
                    "{error}; {ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE}"
                ))
            } else {
                EnclaveError::SecurityFramework(error.to_string())
            }
        })?;
        let public_key = private_key.public_key().ok_or_else(|| {
            EnclaveError::SecurityFramework(String::from("enclave key has no public half"))
        })?;
        let external = public_key.external_representation().ok_or_else(|| {
            EnclaveError::SecurityFramework(String::from(
                "enclave public key has no external representation",
            ))
        })?;
        let public_key_sec1 = external.bytes().to_vec();
        Ok(custody_ref(label, public_key_sec1))
    }

    /// Signs `msg` with the enclave key stored under `label`. The enclave
    /// hashes SHA-256 internally (`ECDSASignatureMessageX962SHA256`) and the
    /// returned bytes are the X9.62 DER signature. The private key never
    /// leaves the enclave.
    pub fn enclave_sign(label: &str, msg: &[u8]) -> Result<Vec<u8>, EnclaveError> {
        let key = find_private_key(label)?;
        key.create_signature(Algorithm::ECDSASignatureMessageX962SHA256, msg)
            .map_err(|error| EnclaveError::SecurityFramework(error.to_string()))
    }

    /// Software-side verification (certificate-independent): delegates to
    /// `crate::crypto_trust_es256::verify_es256` against the public point
    /// recorded in the custody handle. Malformed inputs are refused as
    /// [`EnclaveError::SecurityFramework`] carrying the typed verifier
    /// message; a well-formed signature that does not verify is `Ok(false)`.
    pub fn enclave_verify(
        public_key_sec1: &[u8],
        msg: &[u8],
        sig_der: &[u8],
    ) -> Result<bool, EnclaveError> {
        crate::crypto_trust_es256::verify_es256(public_key_sec1, msg, sig_der)
            .map_err(|error| EnclaveError::SecurityFramework(error.to_string()))
    }

    /// Deletes every keychain item stored under `label` (both key halves).
    /// Refuses with [`EnclaveError::KeyNotFound`] when nothing matched.
    /// Persistence law: every test that creates a key deletes it through this
    /// function (plus a drop guard against panics).
    pub fn delete_enclave_key(label: &str) -> Result<(), EnclaveError> {
        let results = ItemSearchOptions::new()
            .load_refs(true)
            .class(ItemClass::key())
            .label(label)
            .search()
            .map_err(|error| map_search_error(label, error))?;
        let mut deleted_any = false;
        for result in results {
            if let SearchResult::Ref(Reference::Key(key)) = result {
                key.delete()
                    .map_err(|error| EnclaveError::SecurityFramework(error.to_string()))?;
                deleted_any = true;
            }
        }
        if deleted_any {
            Ok(())
        } else {
            Err(EnclaveError::KeyNotFound(label.to_string()))
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::{EnclaveError, EnclaveKeyRef};

    /// Honest typed refusal: this provider is macOS-only
    /// (`ctp:targetOs "macos"`), and no other platform is faked.
    pub fn generate_enclave_key(_label: &str) -> Result<EnclaveKeyRef, EnclaveError> {
        Err(EnclaveError::UnsupportedPlatform)
    }

    /// Honest typed refusal: this provider is macOS-only.
    pub fn enclave_sign(_label: &str, _msg: &[u8]) -> Result<Vec<u8>, EnclaveError> {
        Err(EnclaveError::UnsupportedPlatform)
    }

    /// Honest typed refusal: this provider is macOS-only.
    pub fn enclave_verify(
        _public_key_sec1: &[u8],
        _msg: &[u8],
        _sig_der: &[u8],
    ) -> Result<bool, EnclaveError> {
        Err(EnclaveError::UnsupportedPlatform)
    }

    /// Honest typed refusal: this provider is macOS-only.
    pub fn delete_enclave_key(_label: &str) -> Result<(), EnclaveError> {
        Err(EnclaveError::UnsupportedPlatform)
    }
}

pub use imp::{delete_enclave_key, enclave_sign, enclave_verify, generate_enclave_key};

#[cfg(test)]
mod graph_tests {
    use super::*;

    #[test]
    fn graph_facts_are_rendered() {
        assert_eq!(PROVIDER_KIND, "SECURE_ENCLAVE");
        assert_eq!(TARGET_OS, "macos");
        assert_eq!(KEY_MATERIAL_EXPORTABLE.to_string(), "false");
        assert_eq!(ALGORITHM_NAME, "ES256");
    }

    /// Compile-time proof the unblock recipe renders actionable: the const
    /// must carry the exact codesign command, the entitlements key, the
    /// OSStatus it unblocks, and the identity class. Non-enclave: touches no
    /// keychain, runs on every target.
    #[test]
    fn unblock_recipe_renders_actionable_text() {
        assert!(
            ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE.contains("codesign --force --sign"),
            "recipe must carry the exact codesign command"
        );
        assert!(
            ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE.contains("keychain-access-groups"),
            "recipe must carry the entitlements key"
        );
        assert!(
            ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE.contains("-34018"),
            "recipe must name the OSStatus it unblocks"
        );
        assert!(
            ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE.contains("Apple Development"),
            "recipe must name the required identity class"
        );
        assert!(
            ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE.contains("ent.plist"),
            "recipe must point at the entitlements file"
        );
    }
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static LABEL_COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Collision-free keychain labels per test process.
    fn unique_label(tag: &str) -> String {
        format!(
            "ctp-lane7-{}-{}-{}",
            tag,
            std::process::id(),
            LABEL_COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    }

    /// Deletes the key on drop so a failing assertion never leaks keychain
    /// state; tests still delete explicitly and assert the refusal after.
    struct KeyGuard(String);

    impl KeyGuard {
        fn new(label: &str) -> KeyGuard {
            KeyGuard(label.to_string())
        }
    }

    impl Drop for KeyGuard {
        fn drop(&mut self) {
            let _ = delete_enclave_key(&self.0);
        }
    }

    /// Splits a DER ECDSA-Sig-Value into its (r, s) INTEGER contents
    /// (short-form lengths, as the enclave emits).
    fn split_rs(der: &[u8]) -> (&[u8], &[u8]) {
        assert_eq!(der[0], 0x30, "DER must open with SEQUENCE");
        let seq_len = usize::from(der[1]);
        assert_eq!(
            der.len(),
            2 + seq_len,
            "SEQUENCE length must cover the rest"
        );
        assert_eq!(der[2], 0x02, "r must be an INTEGER");
        let r_len = usize::from(der[3]);
        let r = &der[4..4 + r_len];
        let after_r = 4 + r_len;
        assert_eq!(der[after_r], 0x02, "s must be an INTEGER");
        let s_len = usize::from(der[after_r + 1]);
        let s = &der[after_r + 2..after_r + 2 + s_len];
        assert_eq!(after_r + 2 + s_len, der.len(), "no trailing bytes");
        (r, s)
    }

    /// The software-side verification delegation is real on any macOS host:
    /// an ES256 software key signs, and `enclave_verify` verifies it. This is
    /// the exact path `enclave_verify` uses for enclave signatures too
    /// (verification is certificate-independent and software-side).
    #[test]
    fn software_verification_path_delegates_to_es256() {
        let seed = {
            let mut seed = [0x37u8; 32];
            seed[0] = 0xC9;
            seed
        };
        let key = crate::crypto_trust_es256::Es256SigningKey::from_seed(&seed)
            .expect("software seed key");
        let pk = key.public_key_sec1();
        let msg = b"software-side delegation witness";
        let sig = key.sign(msg);
        assert!(
            matches!(enclave_verify(&pk, msg, &sig), Ok(true)),
            "enclave_verify must verify a well-formed ES256 signature"
        );
        let mut tampered = sig.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0x01;
        assert!(
            matches!(enclave_verify(&pk, msg, &tampered), Ok(false)),
            "tampered signature must not verify"
        );
        assert!(matches!(
            enclave_verify(&pk, msg, &[]),
            Err(EnclaveError::SecurityFramework(_))
        ));
    }

    /// Real cycle on the real enclave: generate, sign, software-verify,
    /// tamper-teeth, delete — and the deleted key refuses to sign again.
    ///
    /// IGNORED HONESTLY: persisting a Secure Enclave key from a
    /// non-Apple-signed binary is refused by the OS on this host. Witnessed
    /// 2026-09-28 (ad-hoc probe replicating this exact generator): plain
    /// ad-hoc, ad-hoc + `com.apple.security.application-groups`, and plain
    /// ad-hoc over the data-protection keychain all fail with `-34018`
    /// `errSecMissingEntitlement` (an earlier witness also recorded `-25308`
    /// with the access-control object); ad-hoc + `keychain-access-groups` is
    /// SIGKILLed by the platform. Keygen itself is real — a fresh P-256
    /// public key comes back inside every refusal. The test is NOT faked; the
    /// unblock is [`ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE`] (real Apple Development
    /// identity), then:
    ///
    /// ```text
    /// cargo test --features secure-enclave secure_enclave_full_cycle -- --ignored
    /// ```
    #[test]
    #[ignore = "Secure Enclave keychain persistence needs a real Apple Development-signed binary (ad-hoc refused, -34018; see ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE)"]
    fn secure_enclave_full_cycle() {
        let label = unique_label("cycle");
        let guard = KeyGuard::new(&label);

        let key_ref = generate_enclave_key(&label).expect("enclave key generation");
        assert_eq!(key_ref.label, label);
        assert_eq!(
            key_ref.public_key_sec1.len(),
            crate::crypto_trust_es256::PUBLIC_KEY_LENGTH
        );
        assert_eq!(key_ref.public_key_sec1[0], 0x04, "X9.63 uncompressed point");
        assert!(key_ref.key_id.to_string().starts_with("afk1_"));

        let msg = b"affidavit enclave full cycle";
        let sig = enclave_sign(&label, msg).expect("enclave signing");
        assert_eq!(sig[0], 0x30, "X9.62 DER signature");

        // Teeth before tamper: valid verifies true, tampered message lies.
        assert!(
            matches!(
                enclave_verify(&key_ref.public_key_sec1, msg, &sig),
                Ok(true)
            ),
            "untampered enclave signature must verify (software-side)"
        );
        let mut tampered = msg.to_vec();
        tampered[0] ^= 0x01;
        assert!(
            matches!(
                enclave_verify(&key_ref.public_key_sec1, &tampered, &sig),
                Ok(false)
            ),
            "flipped message byte must not verify"
        );

        // Explicit delete (the guard covers panics only).
        delete_enclave_key(&label).expect("delete created key");
        drop(guard);
        let err = enclave_sign(&label, msg).expect_err("deleted key must not sign");
        assert!(matches!(err, EnclaveError::KeyNotFound(_)));
    }

    #[test]
    fn sign_for_unknown_label_is_typed_refusal() {
        let label = unique_label("ghost");
        let err = enclave_sign(&label, b"m").expect_err("no such key");
        assert!(matches!(err, EnclaveError::KeyNotFound(_)));
    }

    #[test]
    fn delete_of_unknown_label_is_typed_refusal() {
        let label = unique_label("unknown");
        let err = delete_enclave_key(&label).expect_err("nothing to delete");
        assert!(matches!(err, EnclaveError::KeyNotFound(_)));
    }

    /// LIVE known-answer structure check against the real Secure Enclave:
    /// the signature's DER decodes to two 32-byte big-endian scalars (the
    /// P-256 r/s range) and the software verifier cross-checks it. Requires a
    /// device passcode (keys are passcode-constrained, deliberately not
    /// biometry, so unattended runs never prompt) AND a real Apple
    /// Development-signed host binary — every ad-hoc signature is refused
    /// (`errSecMissingEntitlement`; unblock:
    /// [`ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE`]); never faked. Run manually:
    ///
    /// ```text
    /// cargo test --features secure-enclave live_ -- --ignored
    /// ```
    #[test]
    #[ignore = "live Secure Enclave cycle; requires macOS keychain, device passcode, and a real Apple Development-signed binary (see ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE)"]
    fn live_secure_enclave_known_answer_vector() {
        let label = unique_label("live");
        let guard = KeyGuard::new(&label);

        let key_ref = generate_enclave_key(&label).expect("enclave key generation");
        let msg = b"live secure enclave known answer vector";
        let sig = enclave_sign(&label, msg).expect("enclave signing");

        // Structure: SEQUENCE { INTEGER r, INTEGER s }, each exactly the
        // 32-byte big-endian P-256 scalar, non-zero.
        assert_eq!(sig[0], 0x30, "X9.62 DER signature");
        let (r, s) = split_rs(&sig);
        assert_eq!(r.len(), 32, "r must be a full 32-byte scalar");
        assert_eq!(s.len(), 32, "s must be a full 32-byte scalar");
        assert!(r.iter().any(|byte| *byte != 0), "r must be non-zero");
        assert!(s.iter().any(|byte| *byte != 0), "s must be non-zero");

        // Cross-verification by the independent software verifier.
        assert!(
            matches!(
                enclave_verify(&key_ref.public_key_sec1, msg, &sig),
                Ok(true)
            ),
            "enclave signature must verify software-side"
        );

        // Enclave nonces are not RFC 6979, so a second signature may differ;
        // both must verify — determinism is not the enclave's law, VALIDITY
        // is.
        let sig2 = enclave_sign(&label, msg).expect("enclave signing again");
        assert!(
            matches!(
                enclave_verify(&key_ref.public_key_sec1, msg, &sig2),
                Ok(true)
            ),
            "second enclave signature must also verify"
        );

        delete_enclave_key(&label).expect("delete created key");
        drop(guard);
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod non_macos_tests {
    use super::*;

    #[test]
    fn generate_refuses_by_variant() {
        let err = generate_enclave_key("ctp-lane7").expect_err("non-macOS must refuse");
        assert!(matches!(err, EnclaveError::UnsupportedPlatform));
    }

    #[test]
    fn sign_refuses_by_variant() {
        let err = enclave_sign("ctp-lane7", b"m").expect_err("non-macOS must refuse");
        assert!(matches!(err, EnclaveError::UnsupportedPlatform));
    }

    #[test]
    fn verify_refuses_by_variant() {
        let err = enclave_verify(&[0x04], b"m", &[0x30, 0x00]).expect_err("non-macOS must refuse");
        assert!(matches!(err, EnclaveError::UnsupportedPlatform));
    }

    #[test]
    fn delete_refuses_by_variant() {
        let err = delete_enclave_key("ctp-lane7").expect_err("non-macOS must refuse");
        assert!(matches!(err, EnclaveError::UnsupportedPlatform));
    }
}
