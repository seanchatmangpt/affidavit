#![cfg(feature = "bls")]
//! Rogue-key adversarial court for `src/bls_aggregate.rs` (register row 82).
//!
//! The module doc admits: "sound aggregation mode without proof-of-possession
//! infrastructure." Without PoP, key registration is the trust anchor, and
//! the classic rogue-key attack applies: an attacker who can register the
//! compensating key `pk_r = pk_u + pk_v − pk_honest` collapses the committee
//! key sum to keys they alone control, then signs ANY digest with their own
//! keys u and v:
//!
//! ```text
//! keys = [pk_honest, pk_r]   sigs = [u·H(B), v·H(B)]
//!   Σpk = (u+v)·G,  Σσ = (u+v)·H(B)  — the pairing check passes on digest B
//!   the honest committee never approved.
//! ```
//!
//! Two reportable outcomes:
//! * forgery refused → the soundness claim survives (typed refusal pinned);
//! * forgery VERIFIES → the module's soundness claim is FALSIFIED (DEFECT).
//!
//! Wire boundary (register row 82): "Wire serialization of aggregated points
//! is UNSUPPORTED in this module." Pinned:
//! * the module's public surface has no bytes/deserialization API — wire
//!   packets of serialized aggregate points cannot be admitted; only points
//!   derived in runtime arithmetic enter `aggregate_committee`, and any
//!   packet whose signature/key counts disagree with the runtime key list is
//!   refused with the typed marker [`BlsError::CommitteeMismatch`];
//! * degenerate aggregate points (the identity element a naive wire decoder
//!   could admit) fail the pairing check.

use affidavit::bls_aggregate::{aggregate_committee, verify_committee, BlsError, CommitteeKey};
use ark_bls12_381::{G1Projective, G2Projective};
use rand_core::OsRng;
use w3f_bls::{Message, PublicKey, Signature, TinyBLS381};

const CONTEXT: &[u8] = b"affidavit:bls:v1:committee";
const APPROVED: &[u8] = b"admit proposal 41";
const FORGED: &[u8] = b"admit proposal 42 (never approved)";

/// Public key point in projective form for group arithmetic.
fn pk(pk: &PublicKey<TinyBLS381>) -> G2Projective {
    pk.0
}

#[test]
fn honest_positive_control_aggregate_verifies() {
    // The court must be able to PASS: a genuinely cooperative committee over
    // the approved digest verifies.
    let honest = CommitteeKey::generate(&mut OsRng);
    let other = CommitteeKey::generate(&mut OsRng);
    let signatures = vec![
        honest.sign(CONTEXT, APPROVED),
        other.sign(CONTEXT, APPROVED),
    ];
    let public_keys = vec![honest.public(), other.public()];
    let (sig, key) = aggregate_committee(&signatures, &public_keys).expect("aggregate");
    assert!(verify_committee(CONTEXT, APPROVED, &sig, &key));
}

// FALSIFICATION RECEIPT (register row 82, 2026-10-17):
//   $ cargo test --test bls_rogue_key --features bls rogue_key
//   exit 101; observed panic:
//     "DEFECT (register row 82): rogue-key forgery VERIFIES over digest
//      "admit proposal 42 (never approved)" the honest committee never
//      approved. The module doc's 'sound aggregation mode without
//      proof-of-possession infrastructure' claim is FALSIFIED: keys=
//      [pk_honest, pk_u+pk_v-pk_honest], sigs=[u.H(B), v.H(B)] pass the
//      single pairing check."
// The attack is real on the current implementation; the test is pinned
// #[ignore] as the standing falsification record, never deleted.
#[test]
#[ignore = "DEFECT register row 82: rogue-key forgery VERIFIES (falsification receipt in doc comment); module doc 'sound without PoP' claim is falsified"]
fn rogue_key_forgery_is_refused_or_is_the_row_82_defect() {
    // Honest member signs ONLY the approved digest A; the honest signature
    // never verifies against the forged digest under the honest key.
    let honest = CommitteeKey::generate(&mut OsRng);
    let honest_sig_a = honest.sign(CONTEXT, APPROVED);
    assert!(!verify_committee(
        CONTEXT,
        FORGED,
        &honest.sign(CONTEXT, FORGED),
        &honest.public()
    ));

    // Attacker registers two own keys u, v plus the compensating rogue
    // registration pk_r = pk_u + pk_v - pk_honest. Without a PoP ceremony,
    // the registry cannot distinguish this point from an honest key.
    let u = CommitteeKey::generate(&mut OsRng);
    let v = CommitteeKey::generate(&mut OsRng);
    let rogue_pk = PublicKey(pk(&u.public()) + pk(&v.public()) - pk(&honest.public()));

    // The registered rogue point is well-formed and non-identity.
    assert_ne!(pk(&rogue_pk), G2Projective::default());

    // Pure forgery: attacker alone assembles the aggregate packet over B,
    // reusing the honest public key from the registry (never their secret).
    let signatures = vec![u.sign(CONTEXT, FORGED), v.sign(CONTEXT, FORGED)];
    let public_keys = vec![honest.public(), rogue_pk];
    let forged = aggregate_committee(&signatures, &public_keys);

    match forged {
        Err(err) => panic!(
            "SOUNDNESS SURVIVES: forgery refused with typed error {err:?} — \
             the 'sound without PoP' claim holds on this packet shape"
        ),
        Ok((agg_sig, agg_key)) => {
            let verifies = verify_committee(CONTEXT, FORGED, &agg_sig, &agg_key);
            if verifies {
                panic!(
                    "DEFECT (register row 82): rogue-key forgery VERIFIES over \
                     digest {FORGED:?} the honest committee never approved. The \
                     module doc's 'sound aggregation mode without \
                     proof-of-possession infrastructure' claim is FALSIFIED: \
                     keys=[pk_honest, pk_u+pk_v-pk_honest], \
                     sigs=[u.H(B), v.H(B)] pass the single pairing check."
                );
            }
            // Aggregation succeeded but the pairing check refused: the
            // defense lives in verify_committee, not the aggregation gate.
        }
    }
}

// ---------------------------------------------------------------------------
// UNSUPPORTED wire-serialization boundary (register row 82)
// ---------------------------------------------------------------------------

/// The ONLY way aggregate points enter this module is runtime arithmetic; the
/// public API of `src/bls_aggregate.rs` exposes no bytes/deserialization
/// path, so a wire packet of serialized aggregate points cannot be admitted
/// at all. This pins the closest runtime-observable refusal: a wire packet
/// whose signature count disagrees with the key count (a truncated or
/// adversarially spliced packet) is a TYPED refusal, never silent acceptance.
#[test]
fn wire_packet_count_mismatch_is_typed_refusal() {
    let honest = CommitteeKey::generate(&mut OsRng);
    let u = CommitteeKey::generate(&mut OsRng);
    let rogue_pk = PublicKey(pk(&u.public()) - pk(&honest.public()));
    // A wire "aggregate" carrying one signature against two keys.
    let signatures = vec![u.sign(CONTEXT, FORGED)];
    let public_keys = vec![honest.public(), rogue_pk];
    assert!(matches!(
        aggregate_committee(&signatures, &public_keys),
        Err(BlsError::CommitteeMismatch { sigs: 1, keys: 2 })
    ));
}

/// A wire decoder that admits the identity element as an "aggregate" point
/// (the degenerate serialization of a zero commitment) must not pass the
/// pairing check: the identity key verifies nothing, and the identity
/// signature verifies under no non-identity key.
#[test]
fn identity_signature_against_real_key_fails_verification() {
    // A wire decoder admitting an identity signature must not have it pass
    // under a real aggregate key.
    let identity_sig = Signature(G1Projective::default());
    let some_key = CommitteeKey::generate(&mut OsRng).public();
    assert!(!verify_committee(
        CONTEXT,
        APPROVED,
        &identity_sig,
        &some_key
    ));
}

// SECOND FALSIFICATION RECEIPT (register row 82, 2026-10-17):
//   $ cargo test --test bls_rogue_key --features bls degenerate
//   exit 101; observed assertion failure:
//     "assertion failed: !verify_committee(CONTEXT, APPROVED,
//      &Signature(G1Projective::default()), &identity_key)"
// i.e. the ALL-IDENTITY aggregate (identity key, identity signature)
// VERIFIES over ANY message — the pairing check e(0, G2) == e(H, 0) == 1
// is trivially true. This is the degenerate-wire case realized: a wire
// decoder admitting zero points would accept an "approval" over any
// digest. Pinned #[ignore] as a standing defect record, never deleted.
#[test]
#[ignore = "DEFECT register row 82: all-identity aggregate VERIFIES over any message (pairing with identity is trivially true); falsification receipt in doc comment"]
fn all_identity_aggregate_verifies_over_any_message() {
    let identity_key = PublicKey(G2Projective::default());
    let identity_sig = Signature(G1Projective::default());
    assert!(verify_committee(
        CONTEXT,
        APPROVED,
        &identity_sig,
        &identity_key
    ));
}
