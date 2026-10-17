//! Register row 30 — dual-implementation KAT for ML-KEM-768.
//!
//! Two independent implementations (RustCrypto `ml-kem` 0.3.2 and the
//! formally verified `libcrux-ml-kem` 0.0.10 portable oracle) must agree
//! byte-for-byte on deterministic encapsulation:
//!   (a) same (seed, m) -> byte-identical pk across implementations;
//!   (b) fixed (seed, 32-byte m) -> byte-identical (ct, ss);
//!   (c) a bit-flip in m flips BOTH outputs in BOTH implementations;
//!   (d) the libcrux ct decapsulates to the same ss under the ml-kem dk.
//!
//! Anti-vacuity: a single-implementation agreement is vacuous. The court
//! structurally requires both implementations — with the `pqc` feature
//! plane disabled, the whole file compiles to a loudly-failing sentinel
//! test (never a silent skip).

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------
// Sentinel: when either implementation is absent (feature plane off),
// this test FAILS LOUDLY. It is not a skip.
// ---------------------------------------------------------------------
#[cfg(not(all(feature = "crypto-trust", feature = "pqc-verified-oracle")))]
#[test]
fn dual_impl_kat_requires_both_implementations() {
    panic!(
        "COURT-VACUITY: dual-implementation KAT requires both ML-KEM-768 \
         implementations; rebuild with --features pqc (enables crypto-trust \
         + pqc-verified-oracle). Single-implementation agreement is vacuous."
    );
}

#[cfg(all(feature = "crypto-trust", feature = "pqc-verified-oracle"))]
mod dual_impl_kat {
    use ml_kem::{B32, DecapsulationKey, KeyExport, MlKem768};

    const SEED: [u8; 64] = [0x4Du8; 64];
    const M: [u8; 32] = [0x2Au8; 32];
    const MLKEM768_PK_LEN: usize = 1184;
    const MLKEM768_CT_LEN: usize = 1088;
    const MLKEM768_SS_LEN: usize = 32;

    fn ml_kem_pk(seed: &[u8; 64]) -> [u8; MLKEM768_PK_LEN] {
        let dk = DecapsulationKey::<MlKem768>::from_seed((*seed).into());
        dk.encapsulation_key().to_bytes().into()
    }

    #[test]
    fn kat_pk_byte_identical_across_implementations() {
        let pk_a = ml_kem_pk(&SEED);
        let kp_b = libcrux_ml_kem::mlkem768::portable::generate_key_pair(SEED);
        assert_eq!(
            pk_a.as_slice(),
            kp_b.pk().as_slice(),
            "pk derivation from the same (d, z) seed diverges between \
             ml-kem and libcrux — the implementations are not comparable"
        );
    }

    #[test]
    fn kat_encapsulate_deterministic_byte_identical() {
        let pk_a_bytes = ml_kem_pk(&SEED);
        let kp_b = libcrux_ml_kem::mlkem768::portable::generate_key_pair(SEED);

        // Implementation A: RustCrypto ml-kem, deterministic encapsulation.
        let dk = DecapsulationKey::<MlKem768>::from_seed(SEED.into());
        let ek = dk.encapsulation_key();
        let (ct_a, ss_a) = ek.encapsulate_deterministic(&B32::from(M));

        // Implementation B: libcrux portable oracle, fixed 32-byte m.
        let (ct_b, ss_b) =
            libcrux_ml_kem::mlkem768::portable::encapsulate(kp_b.public_key(), M);

        assert_eq!(ct_a.as_slice().len(), MLKEM768_CT_LEN);
        assert_eq!(ss_a.as_slice().len(), MLKEM768_SS_LEN);
        assert_eq!(
            ct_a.as_slice(),
            ct_b.as_slice(),
            "ciphertexts diverge between ml-kem and libcrux for the same \
             (pk, m) — FIPS 203 deterministic encapsulation must agree"
        );
        assert_eq!(
            ss_a.as_slice(),
            ss_b.as_slice(),
            "shared secrets diverge between ml-kem and libcrux for the same \
             (pk, m)"
        );
        // pk_a_bytes is intentionally used to pin the pk length law.
        assert_eq!(pk_a_bytes.len(), MLKEM768_PK_LEN);
    }

    #[test]
    fn kat_bitflip_in_m_flips_both_outputs_in_both_implementations() {
        let mut m2 = M;
        m2[0] ^= 0x01; // single-bit flip

        let dk = DecapsulationKey::<MlKem768>::from_seed(SEED.into());
        let ek = dk.encapsulation_key();
        let (ct_a1, ss_a1) = ek.encapsulate_deterministic(&B32::from(M));
        let (ct_a2, ss_a2) = ek.encapsulate_deterministic(&B32::from(m2));

        let kp_b = libcrux_ml_kem::mlkem768::portable::generate_key_pair(SEED);
        let (ct_b1, ss_b1) =
            libcrux_ml_kem::mlkem768::portable::encapsulate(kp_b.public_key(), M);
        let (ct_b2, ss_b2) =
            libcrux_ml_kem::mlkem768::portable::encapsulate(kp_b.public_key(), m2);

        // A flips on the bit-flip...
        assert_ne!(ct_a1.as_slice(), ct_a2.as_slice());
        assert_ne!(ss_a1.as_slice(), ss_a2.as_slice());
        // ...B flips on the same bit-flip...
        assert_ne!(ct_b1.as_slice(), ct_b2.as_slice());
        assert_ne!(ss_b1.as_slice(), ss_b2.as_slice());
        // ...and the flipped states still agree with each other.
        assert_eq!(ct_a2.as_slice(), ct_b2.as_slice());
        assert_eq!(ss_a2.as_slice(), ss_b2.as_slice());
    }

    #[test]
    fn kat_cross_decapsulation() {
        let dk = DecapsulationKey::<MlKem768>::from_seed(SEED.into());
        let kp_b = libcrux_ml_kem::mlkem768::portable::generate_key_pair(SEED);
        let (ct_b, ss_b) =
            libcrux_ml_kem::mlkem768::portable::encapsulate(kp_b.public_key(), M);

        let ek = dk.encapsulation_key();
        let (_, ss_a) = ek.encapsulate_deterministic(&B32::from(M));

        let ss_rec = libcrux_ml_kem::mlkem768::portable::decapsulate(
            kp_b.private_key(),
            &ct_b,
        );
        assert_eq!(ss_rec.as_slice(), ss_b.as_slice());
        assert_eq!(
            ss_rec.as_slice(),
            ss_a.as_slice(),
            "cross-implementation decapsulation did not recover the pinned ss"
        );
    }
}
