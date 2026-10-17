#![cfg(feature = "threshold-quorum")]
//! FROST round-1 nonce-reuse court (register row 81).
//!
//! Class: FROST round-1 nonce reuse is the catastrophic secret-leak class —
//! two Schnorr responses under the same nonce reveal the participant's
//! secret share (and by polynomial reconstruction, the group key).
//!
//! Falsifiers (each test names the observation that would kill the claim):
//!
//! (a) `round1_commit` twice for the same participant yields equal nonce
//!     commitments ⇒ the RNG seam is broken and every session leaks the
//!     share ⇒ court fails with a named defect.
//! (b) A stale SigningPackage replayed with fresh round-1 nonces through
//!     `round2_sign` succeeds indistinguishably from an honest session ⇒
//!     no commitment binding at the seam ⇒ DEFECT, named-reason assert.
//! (c) The same nonces are presented for two different packages ⇒ the
//!     seam must refuse the second consumption typed
//!     (`QuorumError::NonceReuse`) and never emit a second Schnorr
//!     response. FLIPPED 2026-10-17: the `SingleUseNonces` consuming
//!     newtype + `round2_sign_once` landed; reuse is now a typed refusal
//!     and the court asserts it.

use affidavit::threshold_quorum::{
    aggregate_signature, generate_quorum, round1_commit, round2_sign, round2_sign_once,
    signing_package, verify_quorum, DealtQuorum, ParticipantId, QuorumError, SingleUseNonces,
};
use rand_core::OsRng;

/// Serialize round-1 commitments to comparable bytes via the frost-core
/// canonical commitment serialization (wire form, not Debug).
fn commitment_bytes(c: &frost_ed25519::round1::SigningCommitments) -> Vec<u8> {
    c.serialize()
        .expect("commitment serialization is infallible for well-formed commitments")
}

/// Full honest 3-of-5 signing session for `message`.
fn honest_session(message: &[u8]) -> (DealtQuorum, [u8; 64]) {
    let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
    let chosen: Vec<ParticipantId> = quorum.shares.keys().copied().take(3).collect();
    let mut nonces = Vec::new();
    let mut commitments = Vec::new();
    for p in &chosen {
        let (n, c) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
        nonces.push((*p, n));
        commitments.push((*p, c));
    }
    let package = signing_package(&commitments, message, quorum.threshold).expect("package");
    let mut shares = Vec::new();
    for (i, p) in chosen.iter().enumerate() {
        let share = round2_sign(&quorum.shares[p], &nonces[i].1, &package).expect("round2");
        shares.push((*p, share));
    }
    let signature = aggregate_signature(
        &package,
        &shares,
        &quorum.public_key_package,
        quorum.threshold,
    )
    .expect("aggregate");
    (quorum, signature)
}

/// (a) Statistical uniqueness of round-1 nonces over a small corpus.
///
/// Falsifier: any collision (equal commitment bytes) across independent
/// `round1_commit` calls ⇒ RNG seam broken ⇒ share leak. Bound: 256 draws
/// against 128-bit nonces; birthday odds ~2^-129, so one collision is the
/// signal, not noise.
#[test]
fn a_round1_nonce_commitments_are_unique_over_corpus() {
    let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
    let participant = *quorum.shares.keys().next().expect("non-empty quorum");
    let mut seen: Vec<Vec<u8>> = Vec::new();
    for _ in 0..256 {
        let (_, c) = round1_commit(&quorum.shares[&participant], &mut OsRng).expect("round1");
        let bytes = commitment_bytes(&c);
        assert!(
            !seen.contains(&bytes),
            "DEFECT(FM-row81a): round1_commit produced a repeated nonce commitment \
             for the same participant across independent calls — the RNG seam is \
             compromised and every signing session leaks the participant's secret \
             share (Schnorr nonce-reuse key-extraction class)."
        );
        seen.push(bytes);
    }
}

/// (b) A stale round-1 commitment set replayed with a NEW message is signed
/// only under the nonces that made those commitments; a participant holding
/// FRESH nonces (session 2) presented with the STALE package must be
/// refused, because frost round 2 re-derives commitments from the held
/// nonces and detects the mismatch.
///
/// Falsifier: `round2_sign` returns Ok ⇒ the participant signed a package
/// whose commitments are not theirs ⇒ coordinated forgery channel.
#[test]
fn b_stale_package_with_fresh_nonces_is_refused_typed() {
    let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
    let chosen: Vec<ParticipantId> = quorum.shares.keys().copied().take(3).collect();

    // Session 1 (recorded on the coordinator): round-1 commitments only.
    // The nonces are discarded — this is the transcript an attacker replays.
    let stale_commitments: Vec<_> = chosen
        .iter()
        .map(|p| {
            let (_, c) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
            (*p, c)
        })
        .collect();

    // Session 2: the participants hold FRESH nonces.
    let fresh_nonces: Vec<_> = chosen
        .iter()
        .map(|p| {
            let (n, _) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
            (*p, n)
        })
        .collect();

    // The replay: coordinator reuses session-1 commitments over a message
    // the participants never committed to.
    let replayed_package =
        signing_package(&stale_commitments, b"message-B-replayed", quorum.threshold)
            .expect("package");

    for (i, p) in chosen.iter().enumerate() {
        let result = round2_sign(&quorum.shares[p], &fresh_nonces[i].1, &replayed_package);
        match result {
            Ok(_) => panic!(
                "DEFECT(FM-row81b): round2_sign accepted a package whose round-1 \
                 commitments were not derived from the participant's held nonces \
                 (participant {:?}) — a replayed commitment set binds participants \
                 to a session they never joined, enabling signature forgery.",
                p
            ),
            Err(QuorumError::Protocol(reason)) => {
                // Typed refusal with a named protocol reason. Pin it.
                assert!(
                    !reason.is_empty(),
                    "DEFECT(FM-row81b-typing): refusal reason is empty — a refusal \
                     without a reason is not loggable at the federation seam"
                );
            }
            Err(other) => panic!(
                "DEFECT(FM-row81b-typing): replay refusal is not typed as \
                 QuorumError::Protocol, got {:?}",
                other
            ),
        }
    }
}

/// (c) THE FORMER PINNED FAULT, FLIPPED (FM-row81c): one participant
/// reuses the SAME round-1 nonces across two different packages (both
/// consistent with their single round-1 commit). This is the exact
/// catastrophic class of register row 81.
///
/// Repair landed: the `threshold_quorum` seam now exposes
/// `round2_sign_once(secret_share, &SingleUseNonces, package)` — the
/// consuming newtype marks the draw consumed on first use, and the second
/// consumption over ANY package (different message included) is a typed
/// `QuorumError::NonceReuse` refusal that never reaches the Schnorr
/// response computation. The legacy borrowing `round2_sign` remains only
/// for out-of-lane callers.
#[test]
fn c_nonce_reuse_across_two_packages_is_refused_typed() {
    let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
    let chosen: Vec<ParticipantId> = quorum.shares.keys().copied().take(3).collect();

    // Round 1 ONCE per participant.
    let mut commitments = Vec::new();
    let mut nonces = Vec::new();
    for p in &chosen {
        let (n, c) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
        nonces.push((*p, n));
        commitments.push((*p, c));
    }
    // TWO packages, same commitments, DIFFERENT messages.
    let pkg_a = signing_package(&commitments, b"tx-A", quorum.threshold).expect("pkgA");
    let pkg_b = signing_package(&commitments, b"tx-B", quorum.threshold).expect("pkgB");

    // Round 2 with the SAME nonces wrapped single-use for both packages.
    let singles: Vec<_> = nonces
        .iter()
        .map(|(_, n)| SingleUseNonces::new(n.clone()))
        .collect();

    // First consumption over package A: succeeds (honest single use).
    let sign_a: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| round2_sign_once(&quorum.shares[p], &singles[i], &pkg_a))
        .collect();

    // Second consumption of the SAME draw over package B: typed refusal.
    for (i, p) in chosen.iter().enumerate() {
        assert_eq!(
            round2_sign_once(&quorum.shares[p], &singles[i], &pkg_b),
            Err(QuorumError::NonceReuse),
            "DEFECT(FM-row81c): nonce reuse for participant {:?} was NOT refused \
             typed — the single-use guard regressed and the transcript again \
             leaks the participant's secret share",
            p
        );
    }

    // The refused transcript must not have leaked a usable signature for the
    // reused draw over message B: only transcript A's shares exist, and they
    // must aggregate and verify over exactly their committed message.
    let shares_a: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, sign_a[i].as_ref().expect("first use signs").clone()))
        .collect();
    let agg_a = aggregate_signature(
        &pkg_a,
        &shares_a,
        &quorum.public_key_package,
        quorum.threshold,
    )
    .expect("DEFECT(FM-row81c): single-use transcript A failed to aggregate");
    assert!(
        verify_quorum(&quorum.group_key, b"tx-A", &agg_a).is_ok(),
        "DEFECT(FM-row81c): transcript A signature does not verify"
    );

    // Package B still signs honestly under a FRESH round-1 draw (fresh
    // nonces + their commitments over the same message) — the guard refuses
    // reuse, not signing.
    let mut fresh_nonces = Vec::new();
    let mut fresh_commitments = Vec::new();
    for p in &chosen {
        let (n, c) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
        fresh_nonces.push(SingleUseNonces::new(n));
        fresh_commitments.push((*p, c));
    }
    let pkg_b_fresh =
        signing_package(&fresh_commitments, b"tx-B", quorum.threshold).expect("packageB-fresh");
    let sign_b: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| round2_sign_once(&quorum.shares[p], &fresh_nonces[i], &pkg_b_fresh))
        .collect();
    let shares_b: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, sign_b[i].as_ref().expect("fresh nonces sign").clone()))
        .collect();
    let agg_b = aggregate_signature(
        &pkg_b_fresh,
        &shares_b,
        &quorum.public_key_package,
        quorum.threshold,
    )
    .expect("DEFECT(FM-row81c): fresh-nonce transcript B failed to aggregate");
    assert!(
        verify_quorum(&quorum.group_key, b"tx-B", &agg_b).is_ok(),
        "DEFECT(FM-row81c): transcript B signature does not verify"
    );
}

/// (d) Positive control: the honest path still signs and verifies, so the
/// court above cannot pass vacuously (a court that cannot fail is not a
/// court — this test fails if the honest seam regresses).
#[test]
fn d_positive_control_honest_session_verifies() {
    let (quorum, signature) = honest_session(b"honest-message");
    assert!(
        verify_quorum(&quorum.group_key, b"honest-message", &signature).is_ok(),
        "DEFECT(row81-positive-control): honest 3-of-5 session failed to verify"
    );
    assert_eq!(
        verify_quorum(&quorum.group_key, b"tampered", &signature),
        Err(QuorumError::VerificationFailed),
        "DEFECT(row81-positive-control): tampered message verified"
    );
}
