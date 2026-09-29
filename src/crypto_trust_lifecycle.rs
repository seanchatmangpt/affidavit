//! Key epochs, rotation policy, and revocation for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Revocation-epoch law (RFC-SA2A-007-errata, envelope field 6): every
//! revocation advances the global revocation epoch to `max(revoked_at) + 1`,
//! and each signature stamps the revocation epoch it was made under. A
//! signature made under an epoch older than the current one is refused as
//! revoked once the key's own revocation sits more than
//! [`MAX_REVOCATION_STALENESS_SECONDS`] in the past — inside that grace the
//! revocation may simply not have propagated to the verifier yet.

use std::collections::BTreeMap;

/// Maximum seconds a signature's revocation epoch may lag the current
/// revocation epoch before the signature is refused as revoked
/// (ctp:policy-v1 ctp:maxRevocationStalenessSeconds).
pub const MAX_REVOCATION_STALENESS_SECONDS: u64 = 300;

/// Replay-protection acceptance window in seconds
/// (ctp:nonce-policy-v1 ctp:windowSeconds).
pub const NONCE_WINDOW_SECONDS: u64 = 300;

/// Envelope fields composing the replay-detection key
/// (ctp:nonce-policy-v1 ctp:replayKey).
pub const REPLAY_KEY: &str = "kid,nonce";

/// One numbered slice of a key's lifetime: activation instant plus optional
/// retirement instant (seconds since the trust-plane epoch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyEpoch {
    pub index: u64,
    pub activated_at: u64,
    pub retired_at: Option<u64>,
}

impl KeyEpoch {
    /// True while `now` lies inside the activation span: from `activated_at`
    /// (inclusive) until `retired_at` (exclusive), or indefinitely when the
    /// epoch has not been retired.
    pub fn active_at(&self, now: u64) -> bool {
        if now < self.activated_at {
            return false;
        }
        match self.retired_at {
            Some(retired) => now < retired,
            None => true,
        }
    }
}

/// Rotation limits applied when epochs are opened and validated.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RotationPolicy {
    pub max_epochs_in_flight: usize,
    pub max_age_seconds: u64,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            max_epochs_in_flight: 2,
            max_age_seconds: 90 * 86400,
        }
    }
}

impl RotationPolicy {
    /// Validate one epoch of `kid` for use at `now` against this policy: an
    /// already retired epoch is refused as [`LifecycleRefusal::AlreadyRetired`],
    /// and an epoch whose age exceeds `max_age_seconds` is refused as
    /// [`LifecycleRefusal::EpochExpired`].
    pub fn validate_epoch(
        &self,
        kid: &str,
        epoch: &KeyEpoch,
        now: u64,
    ) -> Result<(), LifecycleRefusal> {
        if epoch.retired_at.is_some() {
            return Err(LifecycleRefusal::AlreadyRetired(epoch.index));
        }
        if now.saturating_sub(epoch.activated_at) > self.max_age_seconds {
            return Err(LifecycleRefusal::EpochExpired {
                kid: kid.to_string(),
                index: epoch.index,
                max: self.max_age_seconds,
            });
        }
        Ok(())
    }

    /// Refuse opening one more epoch for `kid` when `history` already holds
    /// `max_epochs_in_flight` unretired epochs
    /// ([`LifecycleRefusal::EpochsExhausted`]).
    pub fn admit_opening(&self, kid: &str, history: &[KeyEpoch]) -> Result<(), LifecycleRefusal> {
        let in_flight = history
            .iter()
            .filter(|epoch| epoch.retired_at.is_none())
            .count();
        if in_flight >= self.max_epochs_in_flight {
            return Err(LifecycleRefusal::EpochsExhausted(kid.to_string()));
        }
        Ok(())
    }
}

/// Typed refusal of a lifecycle transition. Refusals are data: every variant
/// carries the evidence a verifier needs to act on the denial.
#[derive(Debug, thiserror::Error)]
pub enum LifecycleRefusal {
    /// The key was revoked at the given instant for the given reason and the
    /// staleness grace has elapsed.
    #[error("key {0} revoked at {1}: {2}")]
    Revoked(String, u64, String),
    /// No epoch of the key is active at the questioned instant.
    #[error("no active epoch for {0}")]
    NoActiveEpoch(String),
    /// The epoch exceeded its maximum age in seconds.
    #[error("epoch {index} for {kid} expired after {max}s")]
    EpochExpired { kid: String, index: u64, max: u64 },
    /// Opening another epoch would exceed the rotation policy's in-flight cap.
    #[error("too many epochs in flight for {0}")]
    EpochsExhausted(String),
    /// The epoch has already been retired; retirement is first-write-wins.
    #[error("epoch {0} already retired")]
    AlreadyRetired(u64),
    /// Epoch activation times must never move backwards.
    #[error("epoch index not monotonic: {0} after {1}")]
    NonMonotonic(u64, u64),
}

/// One revocation event: when the key died and why.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RevocationRecord {
    pub revoked_at: u64,
    pub reason: String,
}

/// The trust plane's revocation list, keyed by `kid`.
///
/// The list doubles as the revocation-epoch clock: every revocation advances
/// [`RevocationList::current_epoch`] to `max(revoked_at) + 1`, giving each
/// signature a comparable stamp (envelope field 6, `revocation_epoch`).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RevocationList {
    records: BTreeMap<String, RevocationRecord>,
}

impl RevocationList {
    /// Record the revocation of `kid` at `at` for `reason`. Re-revoking the
    /// same key overwrites the record: the latest revocation governs both the
    /// epoch clock and the grace window.
    pub fn revoke(&mut self, kid: &str, at: u64, reason: String) {
        self.records.insert(
            kid.to_string(),
            RevocationRecord {
                revoked_at: at,
                reason,
            },
        );
    }

    /// True when `kid` carries a revocation record.
    pub fn is_revoked(&self, kid: &str) -> bool {
        self.records.contains_key(kid)
    }

    /// The instant `kid` was revoked, if it was.
    pub fn revoked_at(&self, kid: &str) -> Option<u64> {
        self.records.get(kid).map(|record| record.revoked_at)
    }

    /// Current revocation epoch: `max(revoked_at) + 1` over all records, or 0
    /// while nothing has been revoked.
    pub fn current_epoch(&self) -> u64 {
        self.records
            .values()
            .map(|record| record.revoked_at)
            .max()
            .map_or(0, |latest| latest.saturating_add(1))
    }

    /// Revocation-epoch admission for a signature by `kid` naming
    /// `sig_revocation_epoch` (envelope field 6) questioned at `now`.
    ///
    /// A signature naming an epoch at or beyond [`RevocationList::current_epoch`]
    /// is live: the signer demonstrably knew the current revocation state. A
    /// signature naming an older epoch is still live while the key's own
    /// revocation is within [`MAX_REVOCATION_STALENESS_SECONDS`] of `now`
    /// (the revocation may not have propagated to the verifier yet); past that
    /// grace it is refused as [`LifecycleRefusal::Revoked`] carrying the
    /// revocation instant and reason. A key with no revocation record is
    /// always live.
    pub fn signature_epoch_live(
        &self,
        kid: &str,
        sig_revocation_epoch: u64,
        now: u64,
    ) -> Result<(), LifecycleRefusal> {
        let Some(record) = self.records.get(kid) else {
            return Ok(());
        };
        if sig_revocation_epoch >= self.current_epoch() {
            return Ok(());
        }
        if now.saturating_sub(record.revoked_at) <= MAX_REVOCATION_STALENESS_SECONDS {
            return Ok(());
        }
        Err(LifecycleRefusal::Revoked(
            kid.to_string(),
            record.revoked_at,
            record.reason.clone(),
        ))
    }
}

/// Per-`kid` epoch history with rotation enforcement.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LifecycleLedger {
    epochs: BTreeMap<String, Vec<KeyEpoch>>,
    policy: RotationPolicy,
}

impl LifecycleLedger {
    /// A ledger governed by `policy`.
    pub fn new(policy: RotationPolicy) -> Self {
        Self {
            epochs: BTreeMap::new(),
            policy,
        }
    }

    /// The rotation policy in force.
    pub fn policy(&self) -> &RotationPolicy {
        &self.policy
    }

    /// Open a new epoch for `kid` activating at `at` and return its sequential
    /// index. Refusals: [`LifecycleRefusal::NonMonotonic`] when `at` moves
    /// backwards against the existing history, [`LifecycleRefusal::EpochsExhausted`]
    /// when the rotation policy's in-flight cap is already reached.
    pub fn open_epoch(&mut self, kid: &str, at: u64) -> Result<u64, LifecycleRefusal> {
        let history = self.epochs.entry(kid.to_string()).or_default();
        if let Some(last) = history.last() {
            if at < last.activated_at {
                return Err(LifecycleRefusal::NonMonotonic(
                    history.len() as u64,
                    last.index,
                ));
            }
        }
        self.policy.admit_opening(kid, history)?;
        let index = history.len() as u64;
        history.push(KeyEpoch {
            index,
            activated_at: at,
            retired_at: None,
        });
        Ok(index)
    }

    /// Retire the epoch with `index` for `kid` at `at`. Idempotent with
    /// first-write-wins: a second retirement, or an unknown index, leaves the
    /// history unchanged so the first retirement instant survives.
    pub fn retire_epoch(&mut self, kid: &str, index: u64, at: u64) {
        if let Some(history) = self.epochs.get_mut(kid) {
            if let Some(epoch) = history.iter_mut().find(|epoch| epoch.index == index) {
                if epoch.retired_at.is_none() {
                    epoch.retired_at = Some(at);
                }
            }
        }
    }

    /// The latest unretired epoch for `kid`, if any.
    pub fn active_epoch(&self, kid: &str) -> Option<&KeyEpoch> {
        self.epochs
            .get(kid)?
            .iter()
            .rev()
            .find(|epoch| epoch.retired_at.is_none())
    }

    /// The epoch that must back a signature by `kid` at `now`; refused as
    /// [`LifecycleRefusal::NoActiveEpoch`] when none is open and live.
    pub fn require_active_epoch(&self, kid: &str, now: u64) -> Result<&KeyEpoch, LifecycleRefusal> {
        self.active_epoch(kid)
            .filter(|epoch| epoch.active_at(now))
            .ok_or_else(|| LifecycleRefusal::NoActiveEpoch(kid.to_string()))
    }

    /// Full lifecycle admission for a signature by `kid` at `now`: an active
    /// epoch must exist and satisfy the rotation policy.
    pub fn validate_signature_key(&self, kid: &str, now: u64) -> Result<(), LifecycleRefusal> {
        let epoch = self.require_active_epoch(kid, now)?;
        self.policy.validate_epoch(kid, epoch, now)
    }

    /// Full epoch history for `kid` (empty when the key is unknown).
    pub fn epochs(&self, kid: &str) -> &[KeyEpoch] {
        self.epochs
            .get(kid)
            .map(|history| history.as_slice())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KID: &str = "k-1";
    const REASON: &str = "compromised";

    // -- teeth first: the revocation-epoch freshness law ---------------------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(MAX_REVOCATION_STALENESS_SECONDS, 300);
        assert_eq!(NONCE_WINDOW_SECONDS, 300);
        assert_eq!(REPLAY_KEY, "kid,nonce");
    }

    #[test]
    fn revocation_blocks_signature_past_grace() {
        let mut list = RevocationList::default();
        let now = 1_000_000;
        let revoked_at = now - MAX_REVOCATION_STALENESS_SECONDS - 100;
        list.revoke(KID, revoked_at, REASON.to_string());
        assert!(list.is_revoked(KID));
        assert_eq!(list.revoked_at(KID), Some(revoked_at));
        match list.signature_epoch_live(KID, 0, now) {
            Err(LifecycleRefusal::Revoked(kid, at, reason)) => {
                assert_eq!(kid, KID);
                assert_eq!(at, revoked_at);
                assert_eq!(reason, REASON);
            }
            outcome => panic!("expected Revoked, got {outcome:?}"),
        }
    }

    #[test]
    fn staleness_grace_at_exactly_the_bound_is_live() {
        let mut list = RevocationList::default();
        let now = 2_000_000;
        list.revoke(
            KID,
            now - MAX_REVOCATION_STALENESS_SECONDS,
            REASON.to_string(),
        );
        assert!(list.signature_epoch_live(KID, 0, now).is_ok());
    }

    #[test]
    fn staleness_grace_one_second_past_the_bound_refused() {
        let mut list = RevocationList::default();
        let now = 2_000_000;
        list.revoke(
            KID,
            now - MAX_REVOCATION_STALENESS_SECONDS - 1,
            REASON.to_string(),
        );
        match list.signature_epoch_live(KID, 0, now) {
            Err(LifecycleRefusal::Revoked(kid, at, reason)) => {
                assert_eq!(kid, KID);
                assert_eq!(at, now - MAX_REVOCATION_STALENESS_SECONDS - 1);
                assert_eq!(reason, REASON);
            }
            outcome => panic!("expected Revoked, got {outcome:?}"),
        }
    }

    #[test]
    fn signature_naming_current_epoch_is_live_despite_revocation() {
        let mut list = RevocationList::default();
        list.revoke(KID, 500, REASON.to_string());
        let current = list.current_epoch();
        // Far past any grace, yet the stamp proves awareness of the revocation.
        assert!(list.signature_epoch_live(KID, current, 5_000_000).is_ok());
        // One epoch behind current falls back to the grace window and refuses.
        assert!(matches!(
            list.signature_epoch_live(KID, current - 1, 5_000_000),
            Err(LifecycleRefusal::Revoked(..))
        ));
    }

    #[test]
    fn unrevoked_key_is_always_live() {
        let list = RevocationList::default();
        assert!(!list.is_revoked("never-revoked"));
        assert!(list
            .signature_epoch_live("never-revoked", 0, 999_999)
            .is_ok());
    }

    #[test]
    fn current_epoch_is_max_revocation_plus_one() {
        let mut list = RevocationList::default();
        assert_eq!(list.current_epoch(), 0);
        list.revoke("a", 41, "rotated".to_string());
        assert_eq!(list.current_epoch(), 42);
        list.revoke("b", 100, "rotated".to_string());
        assert_eq!(list.current_epoch(), 101);
    }

    // -- epoch activation window ---------------------------------------------

    #[test]
    fn epoch_active_window_bounds() {
        let mut epoch = KeyEpoch {
            index: 0,
            activated_at: 10,
            retired_at: None,
        };
        assert!(!epoch.active_at(9));
        assert!(epoch.active_at(10));
        assert!(epoch.active_at(19));
        epoch.retired_at = Some(20);
        assert!(epoch.active_at(19));
        assert!(!epoch.active_at(20));
    }

    // -- rotation policy: expiry, retirement, capacity, monotonicity ----------

    #[test]
    fn rotation_refuses_expired_epoch() {
        let policy = RotationPolicy::default();
        let epoch = KeyEpoch {
            index: 3,
            activated_at: 0,
            retired_at: None,
        };
        assert!(policy
            .validate_epoch(KID, &epoch, policy.max_age_seconds)
            .is_ok());
        match policy.validate_epoch(KID, &epoch, policy.max_age_seconds + 1) {
            Err(LifecycleRefusal::EpochExpired { kid, index, max }) => {
                assert_eq!(kid, KID);
                assert_eq!(index, 3);
                assert_eq!(max, policy.max_age_seconds);
            }
            outcome => panic!("expected EpochExpired, got {outcome:?}"),
        }
    }

    #[test]
    fn retire_then_validate_is_refused() {
        let mut ledger = LifecycleLedger::new(RotationPolicy::default());
        let index = ledger.open_epoch(KID, 0).expect("first epoch opens");
        ledger.retire_epoch(KID, index, 50);
        let epoch = ledger.epochs(KID)[0];
        assert_eq!(epoch.retired_at, Some(50));
        match ledger.policy().validate_epoch(KID, &epoch, 60) {
            Err(LifecycleRefusal::AlreadyRetired(retired_index)) => {
                assert_eq!(retired_index, index);
            }
            outcome => panic!("expected AlreadyRetired, got {outcome:?}"),
        }
        // Retirement is first-write-wins: a later retirement cannot move it.
        ledger.retire_epoch(KID, index, 999);
        assert_eq!(ledger.epochs(KID)[0].retired_at, Some(50));
    }

    #[test]
    fn rotation_policy_caps_epochs_in_flight() {
        let mut ledger = LifecycleLedger::new(RotationPolicy::default());
        let first = ledger.open_epoch(KID, 0).expect("epoch 0 opens");
        ledger.open_epoch(KID, 10).expect("epoch 1 opens");
        match ledger.open_epoch(KID, 20) {
            Err(LifecycleRefusal::EpochsExhausted(kid)) => assert_eq!(kid, KID),
            outcome => panic!("expected EpochsExhausted, got {outcome:?}"),
        }
        // Retiring one epoch frees an in-flight slot; opening resumes at the
        // next sequential index.
        ledger.retire_epoch(KID, first, 15);
        assert_eq!(
            ledger.open_epoch(KID, 20).expect("opens after retirement"),
            2
        );
    }

    #[test]
    fn open_epoch_rejects_backwards_clock() {
        let mut ledger = LifecycleLedger::new(RotationPolicy::default());
        ledger.open_epoch(KID, 100).expect("first epoch opens");
        match ledger.open_epoch(KID, 50) {
            Err(LifecycleRefusal::NonMonotonic(new_index, last_index)) => {
                assert_eq!(new_index, 1);
                assert_eq!(last_index, 0);
            }
            outcome => panic!("expected NonMonotonic, got {outcome:?}"),
        }
        // The same instant is monotonic (the clock must not move backwards).
        assert_eq!(ledger.open_epoch(KID, 100).expect("same instant opens"), 1);
    }

    #[test]
    fn require_active_epoch_refuses_unknown_and_retired_keys() {
        let mut ledger = LifecycleLedger::new(RotationPolicy::default());
        match ledger.require_active_epoch("unknown", 0) {
            Err(LifecycleRefusal::NoActiveEpoch(kid)) => assert_eq!(kid, "unknown"),
            outcome => panic!("expected NoActiveEpoch, got {outcome:?}"),
        }
        let index = ledger.open_epoch(KID, 0).expect("epoch opens");
        ledger.retire_epoch(KID, index, 10);
        assert!(ledger.active_epoch(KID).is_none());
        match ledger.require_active_epoch(KID, 20) {
            Err(LifecycleRefusal::NoActiveEpoch(kid)) => assert_eq!(kid, KID),
            outcome => panic!("expected NoActiveEpoch, got {outcome:?}"),
        }
    }

    #[test]
    fn validate_signature_key_admits_live_epoch() {
        let mut ledger = LifecycleLedger::new(RotationPolicy::default());
        ledger.open_epoch(KID, 0).expect("epoch opens");
        assert!(ledger.validate_signature_key(KID, 1_000).is_ok());
    }

    #[test]
    fn epochs_slice_is_scoped_per_kid() {
        let mut ledger = LifecycleLedger::new(RotationPolicy::default());
        ledger.open_epoch(KID, 0).expect("epoch opens");
        ledger.open_epoch("k-2", 5).expect("epoch opens");
        assert_eq!(ledger.epochs(KID).len(), 1);
        assert_eq!(ledger.epochs("k-2").len(), 1);
        assert!(ledger.epochs("k-3").is_empty());
    }

    // -- serialization round-trips --------------------------------------------

    #[test]
    fn key_epoch_round_trips() {
        let epoch = KeyEpoch {
            index: 7,
            activated_at: 100,
            retired_at: Some(200),
        };
        let encoded = serde_json::to_string(&epoch).expect("serializes");
        let decoded: KeyEpoch = serde_json::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded, epoch);
    }

    #[test]
    fn rotation_policy_round_trips_with_defaults() {
        let policy = RotationPolicy::default();
        let encoded = serde_json::to_string(&policy).expect("serializes");
        let decoded: RotationPolicy = serde_json::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded.max_epochs_in_flight, 2);
        assert_eq!(decoded.max_age_seconds, 90 * 86400);
    }

    #[test]
    fn revocation_list_round_trips() {
        let mut list = RevocationList::default();
        list.revoke(KID, 300, REASON.to_string());
        list.revoke("k-2", 400, "superseded".to_string());
        let encoded = serde_json::to_string(&list).expect("serializes");
        let decoded: RevocationList = serde_json::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded, list);
        assert_eq!(decoded.current_epoch(), 401);
    }

    #[test]
    fn lifecycle_ledger_round_trips() {
        let mut ledger = LifecycleLedger::new(RotationPolicy::default());
        let index = ledger.open_epoch(KID, 0).expect("epoch opens");
        ledger.retire_epoch(KID, index, 10);
        ledger.open_epoch(KID, 10).expect("second epoch opens");
        let encoded = serde_json::to_string(&ledger).expect("serializes");
        let decoded: LifecycleLedger = serde_json::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded.epochs(KID), ledger.epochs(KID));
        assert_eq!(
            decoded.policy().max_epochs_in_flight,
            ledger.policy().max_epochs_in_flight
        );
        assert!(decoded.active_epoch(KID).is_some());
    }
}
