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
//! (c) The same nonces sign two different packages and both transcripts
//!     aggregate to verifying group signatures ⇒ the seam has no
//!     nonce-reuse guard and the transcript leaks the shares ⇒ this is
//!     the DEFECT verdict the court reports (pinned as observed behavior,
//!     not papered over — see test (c)'s doc comment for the disposition).

use affidavit::threshold_quorum::{
    aggregate_signature, generate_quorum, round1_commit, round2_sign, signing_package, verify_quorum, DealtQuorum, ParticipantId, QuorumError,
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

/// (c) THE PINNED FAULT: one participant reuses the SAME nonces across two
/// different packages (both consistent with their single round-1 commit).
/// This is the exact catastrophic class of register row 81.
///
/// Observed FROST behavior being pinned: frost round 2 has no nonce-reuse
/// guard ( SigningNonces are moved by value, once), so both signings
/// succeed and both transcripts aggregate. Two group signatures over two
/// different messages under the same nonces = the participant secret share
/// is algebraically extractable from the transcript.
///
/// DEFECT VERDICT (court result, not papered over): the
/// `threshold_quorum` seam exposes `round2_sign(secret_share, nonces,
/// package)` with NO state to detect reuse and NO typed refusal for it.
/// Defense currently lives entirely in caller discipline ("discard nonces
/// after one use"), which the seam does not enforce. A guard (e.g. a
/// `SingleUseNonces` newtype consumed by `round2_sign`, emitting
/// `QuorumError::NonceReuse`) is the repair lane; this court will flip to
/// refusal-class asserts when it lands.
#[test]
fn c_nonce_reuse_across_two_packages_is_accepted_and_leaks() {
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

    // Round 2 with the SAME nonces for both packages.
    let sign_a: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| round2_sign(&quorum.shares[p], &nonces[i].1, &pkg_a))
        .collect();
    let sign_b: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| round2_sign(&quorum.shares[p], &nonces[i].1, &pkg_b))
        .collect();

    // Named-reason pin: the seam accepted the reuse. If a future repair adds
    // the guard, THIS assert is what must flip — update it alongside the
    // QuorumError::NonceReuse arm.
    for (i, p) in chosen.iter().enumerate() {
        assert!(
            sign_a[i].is_ok() && sign_b[i].is_ok(),
            "behavior changed (FM-row81c): round2_sign refused nonce reuse for \
             participant {:?} — a nonce-reuse guard has landed; update this court \
             to assert the typed QuorumError::NonceReuse refusal",
            p
        );
    }

    let shares_a: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, sign_a[i].as_ref().expect("pinned ok above").clone()))
        .collect();
    let shares_b: Vec<_> = chosen
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, sign_b[i].as_ref().expect("pinned ok above").clone()))
        .collect();

    let agg_a = aggregate_signature(&pkg_a, &shares_a, &quorum.public_key_package, quorum.threshold)
        .expect("DEFECT(FM-row81c): reused-nonce transcript A failed to aggregate");
    let agg_b = aggregate_signature(&pkg_b, &shares_b, &quorum.public_key_package, quorum.threshold)
        .expect("DEFECT(FM-row81c): reused-nonce transcript B failed to aggregate");

    // The leak, demonstrated end to end: BOTH group signatures verify, over
    // two different messages, from ONE round-1 nonce draw. This transcript
    // is a key-extraction oracle for every participant's secret share.
    assert!(
        verify_quorum(&quorum.group_key, b"tx-A", &agg_a).is_ok(),
        "DEFECT(FM-row81c): transcript A signature does not verify"
    );
    assert!(
        verify_quorum(&quorum.group_key, b"tx-B", &agg_b).is_ok(),
        "DEFECT(FM-row81c): transcript B signature does not verify"
    );
    // The transcripts are distinguishable (different messages ⇒ different
    // signatures under the same nonces) — which is precisely what makes the
    // pair a solvable linear system for the shares.
    assert_ne!(
        agg_a, agg_b,
        "DEFECT(FM-row81c): signatures under identical nonces unexpectedly equal"
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
