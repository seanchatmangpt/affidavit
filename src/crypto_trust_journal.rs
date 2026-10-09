//! Append-only, hash-chained journal of cryptographic standing receipts for the
//! affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//! Consumed query columns (journal.rq): journal_format, genesis, domain_tag.
//!
//! Closes the wave-2 review gap "journal lacks rotation/key IDs": every entry
//! carries the `key_id` that verified the receipt plus the rotation context it
//! was recorded under (`policy_epoch`, `revocation_epoch` — envelope fields 5
//! and 6), so a journal answers WHO was standing WHEN under WHICH key without
//! unsealing anything.
//!
//! CHAIN LAW: `entry_hash` is the lowercase-hex BLAKE3 of
//! `digest(JOURNAL_DOMAIN, &[jcs(entry-minus-entry_hash)])` — the canonical
//! (JCS, RFC 8785) JSON of every field except the hash itself, domain-separated
//! by [`JOURNAL_DOMAIN`] through `crypto_trust_canonical::digest_hex`. Each
//! entry's `prev` binds its predecessor's `entry_hash`; entry 0 binds
//! [`JOURNAL_GENESIS`] (64 hex zeros). Any bit flipped anywhere in the chain —
//! payload, link, or hash — is refused on load ([`JournalError::ChainBroken`]);
//! any sequence discontinuity is refused as [`JournalError::SeqGap`]. The load
//! path (`from_jsonl`) re-verifies every byte it reads: the wire form is never
//! trusted without recomputation.

use crate::crypto_trust_canonical::{digest_hex, jcs, CanonicalError};

/// Journal wire-format identifier (ctp:journal-v1 ctp:journalFormat).
pub const JOURNAL_FORMAT: &str = "CTP-JOURNAL-v1";

/// Genesis anchor for entry 0's `prev` link
/// (ctp:journal-v1 ctp:journalGenesis): 64 lowercase-hex zeros.
pub const JOURNAL_GENESIS: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Domain-separation tag mixed into every entry digest
/// (ctp:policy-v1 ctp:domainTag); identical to the plane-wide domain tag.
pub const JOURNAL_DOMAIN: &str = "affidavit.crypto-trust-plane.v1";

/// One append-only journal entry: a sealed
/// `crate::crypto_trust_verify::CryptoStandingReceipt` projected into its
/// rotation and key context.
///
/// `entry_hash` is computed over the canonical JSON (JCS) of every other
/// field — see the chain law in the module header.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalEntry {
    /// Sequence number: 0 for the first entry, `last.seq + 1` thereafter.
    pub seq: u64,
    /// Hex `entry_hash` of the predecessor, or [`JOURNAL_GENESIS`] for seq 0.
    pub prev: String,
    /// The registry key whose verification the recorded receipt certifies.
    pub key_id: String,
    /// Policy epoch (envelope field 5) in force when the receipt was recorded.
    pub policy_epoch: u64,
    /// Revocation epoch (envelope field 6) the recorded signature proved
    /// awareness of.
    pub revocation_epoch: u64,
    /// The decided standing's SCREAMING_SNAKE_CASE name (e.g. `VALID`).
    pub standing: String,
    /// Lowercase-hex BLAKE3 commitment over the envelope's signed bytes,
    /// copied from the receipt.
    pub envelope_commitment: String,
    /// The receipt's own canonical hash, copied verbatim: the journal chains
    /// over receipts it does not reinterpret.
    pub receipt_hash: String,
    /// Lowercase-hex chain hash over every field above (see module header).
    pub entry_hash: String,
}

/// The hash material: every [`JournalEntry`] field except `entry_hash` itself.
#[derive(serde::Serialize)]
struct JournalEntryMaterial<'a> {
    seq: u64,
    prev: &'a str,
    key_id: &'a str,
    policy_epoch: u64,
    revocation_epoch: u64,
    standing: &'a str,
    envelope_commitment: &'a str,
    receipt_hash: &'a str,
}

/// Chain hash over the canonical JSON of `material`, domain-separated by
/// [`JOURNAL_DOMAIN`]: `hex(BLAKE3(JOURNAL_DOMAIN || 0x00 || JOURNAL_DOMAIN
/// || 0x00 || len(jcs) || jcs))` via `crypto_trust_canonical::digest_hex`.
/// Refusals are typed [`JournalError::Serialization`] — the JCS layer refuses
/// non-I-JSON integers (any field beyond 2^53) rather than coercing them.
fn journal_entry_hash(material: &JournalEntryMaterial<'_>) -> Result<String, JournalError> {
    let value = serde_json::to_value(material)?;
    let canonical = jcs(&value)?;
    Ok(digest_hex(JOURNAL_DOMAIN, &[canonical.as_bytes()]))
}

impl JournalEntry {
    /// Recompute this entry's chain hash over its own material (every field
    /// except `entry_hash`); the recompute-and-compare law `verify_chain`
    /// enforces entry-wise.
    fn recomputed_hash(&self) -> Result<String, JournalError> {
        let material = JournalEntryMaterial {
            seq: self.seq,
            prev: &self.prev,
            key_id: &self.key_id,
            policy_epoch: self.policy_epoch,
            revocation_epoch: self.revocation_epoch,
            standing: &self.standing,
            envelope_commitment: &self.envelope_commitment,
            receipt_hash: &self.receipt_hash,
        };
        journal_entry_hash(&material)
    }
}

/// Typed refusal of a journal transition. Refusals are data: every variant
/// names the exact state that broke the append-only law.
#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    /// Sequence continuity broke: the position demanded `expected` but the
    /// entry carried `got`. Fires on load (a tampered or truncated line) and
    /// defensively on append past `u64::MAX`, where no successor exists.
    #[error("sequence gap: expected {expected}, got {got}")]
    SeqGap {
        /// The sequence number the chain position demands.
        expected: u64,
        /// The sequence number the offending entry actually carries.
        got: u64,
    },
    /// The chain does not reproduce: an entry's `prev` link does not name its
    /// predecessor's `entry_hash`, or its own `entry_hash` does not match the
    /// recomputed hash of its material — the journal was tampered with at
    /// entry `at`.
    #[error("chain broken at entry {at}")]
    ChainBroken {
        /// Sequence number of the first entry that failed verification.
        at: u64,
    },
    /// Canonical serialization refused the material (non-I-JSON integer beyond
    /// double precision), or a wire line was not parseable JSON.
    #[error("serialization: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for JournalError {
    fn from(err: serde_json::Error) -> Self {
        JournalError::Serialization(err.to_string())
    }
}

impl From<CanonicalError> for JournalError {
    fn from(err: CanonicalError) -> Self {
        JournalError::Serialization(err.to_string())
    }
}

/// The append-side input: everything a new entry carries except the chain
/// facts (`seq`, `prev`, `entry_hash`) the journal itself computes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntryDraft {
    /// The registry key whose verification the receipt certifies.
    pub key_id: String,
    /// Policy epoch in force when the receipt was recorded.
    pub policy_epoch: u64,
    /// Revocation epoch the recorded signature proved awareness of.
    pub revocation_epoch: u64,
    /// The decided standing's SCREAMING_SNAKE_CASE name.
    pub standing: String,
    /// Lowercase-hex envelope commitment copied from the receipt.
    pub envelope_commitment: String,
    /// The receipt's own canonical hash, copied verbatim.
    pub receipt_hash: String,
}

/// The append-only standing journal: a strict sequence of
/// [`JournalEntry`]s whose `prev` links and `entry_hash`es form a hash chain
/// anchored at [`JOURNAL_GENESIS`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StandingJournal {
    entries: Vec<JournalEntry>,
}

impl StandingJournal {
    /// The empty journal: no entries, chain anchored at genesis.
    pub fn new() -> Self {
        StandingJournal {
            entries: Vec::new(),
        }
    }

    /// Append `draft` as the next entry. Sequence continuity is structural:
    /// `seq` is `last.seq + 1` (0 for the first entry) and `prev` is the last
    /// entry's `entry_hash` (genesis for the first); the chain hash is
    /// recomputed here, never accepted from the caller. The only refusal is
    /// [`JournalError::SeqGap`] when the journal is exhausted
    /// (`u64::MAX` appended) and no successor exists, or
    /// [`JournalError::Serialization`] when the material cannot canonicalize.
    pub fn append(&mut self, draft: JournalEntryDraft) -> Result<JournalEntry, JournalError> {
        let (seq, prev) = match self.entries.last() {
            Some(last) => {
                let seq = last.seq.checked_add(1).ok_or(JournalError::SeqGap {
                    expected: last.seq,
                    got: u64::MAX,
                })?;
                (seq, last.entry_hash.clone())
            }
            None => (0, JOURNAL_GENESIS.to_string()),
        };
        let material = JournalEntryMaterial {
            seq,
            prev: &prev,
            key_id: &draft.key_id,
            policy_epoch: draft.policy_epoch,
            revocation_epoch: draft.revocation_epoch,
            standing: &draft.standing,
            envelope_commitment: &draft.envelope_commitment,
            receipt_hash: &draft.receipt_hash,
        };
        let entry_hash = journal_entry_hash(&material)?;
        let entry = JournalEntry {
            seq,
            prev,
            key_id: draft.key_id,
            policy_epoch: draft.policy_epoch,
            revocation_epoch: draft.revocation_epoch,
            standing: draft.standing,
            envelope_commitment: draft.envelope_commitment,
            receipt_hash: draft.receipt_hash,
            entry_hash,
        };
        self.entries.push(entry.clone());
        Ok(entry)
    }

    /// Re-verify the whole chain from genesis: sequence continuity (position
    /// `i` must carry seq `i`), every `prev` link, and every `entry_hash`
    /// recomputed over its material. The sequence check runs FIRST, so a
    /// tampered line is refused as [`JournalError::SeqGap`] even when its
    /// stolen hash would also fail; a hash or link mismatch is
    /// [`JournalError::ChainBroken`] at the first offending entry.
    pub fn verify_chain(&self) -> Result<(), JournalError> {
        for (index, entry) in self.entries.iter().enumerate() {
            let expected_seq = index as u64;
            if entry.seq != expected_seq {
                return Err(JournalError::SeqGap {
                    expected: expected_seq,
                    got: entry.seq,
                });
            }
            let expected_prev = if index == 0 {
                JOURNAL_GENESIS
            } else {
                self.entries[index - 1].entry_hash.as_str()
            };
            if entry.prev != expected_prev || entry.recomputed_hash()? != entry.entry_hash {
                return Err(JournalError::ChainBroken { at: entry.seq });
            }
        }
        Ok(())
    }

    /// Every entry recorded under `kid`, in journal order.
    pub fn by_key(&self, kid: &str) -> Vec<&JournalEntry> {
        self.entries.iter().filter(|e| e.key_id == kid).collect()
    }

    /// Every entry in `epoch`, in journal order. An entry is IN an epoch when
    /// it was recorded under that policy epoch OR proves awareness of that
    /// revocation epoch — rotation context is the pair, and either half makes
    /// the entry evidence about that epoch.
    pub fn by_epoch(&self, epoch: u64) -> Vec<&JournalEntry> {
        self.entries
            .iter()
            .filter(|e| e.policy_epoch == epoch || e.revocation_epoch == epoch)
            .collect()
    }

    /// Number of entries in the journal.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the journal holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries, in journal order (read-only access).
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// The JSONL wire form: one [`JournalEntry`] per line, no trailing
    /// newline, empty string for the empty journal. Serialization of
    /// `JournalEntry` is total over its closed field domain (`String` +
    /// `u64`), so a skipped line cannot lose a lawful entry; the load path
    /// re-verifies every byte it reads regardless.
    pub fn to_jsonl(&self) -> String {
        self.entries
            .iter()
            .filter_map(|entry| serde_json::to_string(entry).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Load from the [`StandingJournal::to_jsonl`] wire form and re-verify the
    /// chain from genesis. A line that does not parse is
    /// [`JournalError::Serialization`]; a parsed journal that does not
    /// reproduce — tampered payload, broken link, wrong hash, gap — is refused
    /// with the exact [`JournalError`] variant. Empty (or whitespace-only)
    /// input loads the empty journal.
    pub fn from_jsonl(s: &str) -> Result<Self, JournalError> {
        let mut entries = Vec::new();
        for line in s.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let entry: JournalEntry = serde_json::from_str(line)?;
            entries.push(entry);
        }
        let journal = StandingJournal { entries };
        journal.verify_chain()?;
        Ok(journal)
    }
}

/// Record a sealed [`crate::crypto_trust_verify::CryptoStandingReceipt`] into
/// `journal` under the given rotation context. The receipt's identity fields
/// (`key_id`, `standing` in its SCREAMING_SNAKE_CASE wire name,
/// `envelope_commitment`, `receipt_hash`) are copied verbatim — the journal is
/// evidence about receipts, never a reinterpretation of them.
pub fn record_receipt(
    journal: &mut StandingJournal,
    receipt: &crate::crypto_trust_verify::CryptoStandingReceipt,
    policy_epoch: u64,
    revocation_epoch: u64,
) -> Result<JournalEntry, JournalError> {
    journal.append(JournalEntryDraft {
        key_id: receipt.key_id.clone(),
        policy_epoch,
        revocation_epoch,
        standing: receipt.standing.as_str().to_string(),
        envelope_commitment: receipt.envelope_commitment.clone(),
        receipt_hash: receipt.receipt_hash.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_envelope::{NonceJournal, SignatureEnvelope, ENVELOPE_VERSION};
    use crate::crypto_trust_es256::Es256SigningKey;
    use crate::crypto_trust_keys::{
        fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
        KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
    };
    use crate::crypto_trust_lifecycle::RevocationList;
    use crate::crypto_trust_verify::{CryptographicStanding, TrustPolicy, VerificationEngine};

    /// The verifier clock used across the court (mirrors crypto_trust_verify).
    const NOW: u64 = 1_700_000_500;
    const KID_A: &str = "afk1_aaaaaaaaaaaaaaaa";
    const KID_B: &str = "afk1_bbbbbbbbbbbbbbbb";
    const COMMITMENT: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const RECEIPT_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";

    fn draft(key_id: &str, policy_epoch: u64, revocation_epoch: u64) -> JournalEntryDraft {
        JournalEntryDraft {
            key_id: key_id.to_string(),
            policy_epoch,
            revocation_epoch,
            standing: "VALID".to_string(),
            envelope_commitment: COMMITMENT.to_string(),
            receipt_hash: RECEIPT_HASH.to_string(),
        }
    }

    // -- teeth first: append N -> verify_chain Ok, genesis anchor, links -------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(JOURNAL_FORMAT, "CTP-JOURNAL-v1");
        assert_eq!(JOURNAL_GENESIS, "0".repeat(64));
        assert_eq!(JOURNAL_DOMAIN, "affidavit.crypto-trust-plane.v1");
        // Cross-module graph consistency: every lane rendered the same facts.
        assert_eq!(JOURNAL_DOMAIN, crate::crypto_trust_verify::DOMAIN_TAG);
        assert_eq!(JOURNAL_DOMAIN, crate::crypto_trust_canonical::DOMAIN_TAG);
    }

    #[test]
    fn append_chain_of_n_verifies_from_genesis() {
        let mut journal = StandingJournal::new();
        assert!(journal.is_empty());
        assert_eq!(journal.len(), 0);
        journal.verify_chain().expect("the empty chain verifies");

        for i in 0..5u64 {
            let entry = journal
                .append(draft(KID_A, i, i + 10))
                .expect("append is lawful");
            assert_eq!(entry.seq, i);
            let expected_prev = if i == 0 {
                JOURNAL_GENESIS.to_string()
            } else {
                journal.entries()[i as usize - 1].entry_hash.clone()
            };
            assert_eq!(
                entry.prev, expected_prev,
                "entry {i} must bind its predecessor"
            );
        }
        assert_eq!(journal.len(), 5);
        assert!(!journal.is_empty());
        journal
            .verify_chain()
            .expect("the chained journal verifies");
    }

    #[test]
    fn genesis_anchor_and_entry_hash_law() {
        let mut journal = StandingJournal::new();
        let entry = journal.append(draft(KID_A, 1, 0)).expect("append");
        assert_eq!(entry.prev, JOURNAL_GENESIS);
        // The stored hash is exactly the documented pre-image: JCS of the
        // entry minus entry_hash, domain-separated, BLAKE3, lowercase hex.
        let material = JournalEntryMaterial {
            seq: entry.seq,
            prev: &entry.prev,
            key_id: &entry.key_id,
            policy_epoch: entry.policy_epoch,
            revocation_epoch: entry.revocation_epoch,
            standing: &entry.standing,
            envelope_commitment: &entry.envelope_commitment,
            receipt_hash: &entry.receipt_hash,
        };
        let value = serde_json::to_value(&material).expect("material serializes");
        let canonical = jcs(&value).expect("material canonicalizes");
        assert_eq!(
            entry.entry_hash,
            digest_hex(JOURNAL_DOMAIN, &[canonical.as_bytes()])
        );
        assert_eq!(entry.entry_hash.len(), 64);
        assert_eq!(
            entry.recomputed_hash().expect("recomputes"),
            entry.entry_hash
        );
    }

    #[test]
    fn same_appends_produce_identical_bytes() {
        let mut a = StandingJournal::new();
        let mut b = StandingJournal::new();
        for i in 0..4u64 {
            let d = draft(KID_B, i * 2, i * 2 + 1);
            assert_eq!(
                a.append(d.clone()).expect("append a"),
                b.append(d).expect("append b"),
                "deterministic appends produce identical entries"
            );
        }
        assert_eq!(a.to_jsonl(), b.to_jsonl());
        assert_eq!(a.entries(), b.entries());
    }

    // -- filters -----------------------------------------------------------------

    #[test]
    fn by_key_returns_only_that_keys_entries_in_order() {
        let mut journal = StandingJournal::new();
        journal.append(draft(KID_A, 1, 0)).expect("append");
        journal.append(draft(KID_B, 1, 0)).expect("append");
        journal.append(draft(KID_A, 2, 0)).expect("append");
        let of_a = journal.by_key(KID_A);
        assert_eq!(of_a.len(), 2);
        assert_eq!(of_a[0].seq, 0);
        assert_eq!(of_a[1].seq, 2);
        assert!(of_a.iter().all(|e| e.key_id == KID_A));
        assert_eq!(journal.by_key("afk1_never_seen").len(), 0);
    }

    #[test]
    fn by_epoch_matches_policy_and_revocation_context() {
        let mut journal = StandingJournal::new();
        // Entry 0: policy epoch 7, revocation epoch 100.
        journal.append(draft(KID_A, 7, 100)).expect("append");
        // Entry 1: policy epoch 8, revocation epoch 7 — epoch 7 on BOTH sides.
        journal.append(draft(KID_B, 8, 7)).expect("append");
        // Entry 2: neither side touches 7.
        journal.append(draft(KID_A, 9, 101)).expect("append");

        let in_seven = journal.by_epoch(7);
        assert_eq!(in_seven.len(), 2, "policy OR revocation context matches");
        assert_eq!(in_seven[0].seq, 0);
        assert_eq!(in_seven[1].seq, 1);
        let in_nine = journal.by_epoch(9);
        assert_eq!(in_nine.len(), 1);
        assert_eq!(in_nine[0].seq, 2);
        assert!(journal.by_epoch(u64::MAX).is_empty());
    }

    // -- JSONL wire form ------------------------------------------------------------

    #[test]
    fn jsonl_round_trips_and_reverifies() {
        let mut journal = StandingJournal::new();
        for i in 0..6u64 {
            journal.append(draft(KID_A, i, 50 + i)).expect("append");
        }
        let jsonl = journal.to_jsonl();
        assert_eq!(jsonl.lines().count(), 6);
        let loaded = StandingJournal::from_jsonl(&jsonl).expect("honest wire form loads");
        assert_eq!(loaded.entries(), journal.entries());
        assert_eq!(loaded.to_jsonl(), jsonl);
        loaded.verify_chain().expect("loaded journal verifies");
    }

    #[test]
    fn empty_journal_round_trips_through_empty_wire() {
        let journal = StandingJournal::new();
        assert_eq!(journal.to_jsonl(), "");
        let loaded = StandingJournal::from_jsonl("").expect("empty wire loads empty");
        assert!(loaded.is_empty());
        loaded.verify_chain().expect("empty chain verifies");
    }

    // -- every JournalError variant witnessed by exact variant ----------------------

    #[test]
    fn seq_gap_refuses_on_load() {
        let mut journal = StandingJournal::new();
        for i in 0..3u64 {
            journal.append(draft(KID_A, i, 0)).expect("append");
        }
        // Craft a wire form whose second line carries a wrong seq: the stolen
        // entry_hash no longer matches anything, but the SEQUENCE check runs
        // first and refuses with the exact gap.
        let mut lines: Vec<String> = journal
            .to_jsonl()
            .lines()
            .map(|line| line.to_string())
            .collect();
        let mut tampered: serde_json::Value = serde_json::from_str(&lines[1]).expect("line parses");
        tampered["seq"] = serde_json::Value::from(42u64);
        lines[1] = serde_json::to_string(&tampered).expect("tampered line serializes");
        let wire = lines.join("\n");
        match StandingJournal::from_jsonl(&wire) {
            Err(JournalError::SeqGap { expected, got }) => {
                assert_eq!(expected, 1);
                assert_eq!(got, 42);
            }
            other => panic!("expected SeqGap, got {other:?}"),
        }
    }

    #[test]
    fn flipped_entry_hash_refuses_on_load() {
        let mut journal = StandingJournal::new();
        for i in 0..3u64 {
            journal.append(draft(KID_A, i, 0)).expect("append");
        }
        let mut lines: Vec<String> = journal
            .to_jsonl()
            .lines()
            .map(|line| line.to_string())
            .collect();
        let mut tampered: serde_json::Value = serde_json::from_str(&lines[1]).expect("line parses");
        let claimed = tampered["entry_hash"]
            .as_str()
            .expect("hex field")
            .to_string();
        let mut chars: Vec<char> = claimed.chars().collect();
        chars[0] = if chars[0] == '0' { '1' } else { '0' };
        let flipped: String = chars.into_iter().collect();
        assert_ne!(flipped, claimed, "the flip must change the hash");
        tampered["entry_hash"] = serde_json::Value::from(flipped);
        lines[1] = serde_json::to_string(&tampered).expect("tampered line serializes");
        match StandingJournal::from_jsonl(&lines.join("\n")) {
            Err(JournalError::ChainBroken { at }) => assert_eq!(at, 1),
            other => panic!("expected ChainBroken, got {other:?}"),
        }
    }

    #[test]
    fn tampered_payload_refuses_on_load() {
        let mut journal = StandingJournal::new();
        journal.append(draft(KID_A, 1, 0)).expect("append");
        journal.append(draft(KID_B, 1, 0)).expect("append");
        let mut lines: Vec<String> = journal
            .to_jsonl()
            .lines()
            .map(|line| line.to_string())
            .collect();
        let mut tampered: serde_json::Value = serde_json::from_str(&lines[0]).expect("line parses");
        // Rewrite history: entry 0's standing flips VALID -> INVALID. Its
        // stored entry_hash no longer reproduces over the new material.
        tampered["standing"] = serde_json::Value::from("INVALID");
        lines[0] = serde_json::to_string(&tampered).expect("tampered line serializes");
        match StandingJournal::from_jsonl(&lines.join("\n")) {
            Err(JournalError::ChainBroken { at }) => assert_eq!(at, 0),
            other => panic!("expected ChainBroken at 0, got {other:?}"),
        }
    }

    #[test]
    fn unparseable_line_refuses_as_serialization() {
        match StandingJournal::from_jsonl("this is not json") {
            Err(JournalError::Serialization(_)) => {}
            other => panic!("expected Serialization, got {other:?}"),
        }
    }

    #[test]
    fn epoch_beyond_double_precision_refuses_as_serialization() {
        // JCS refuses non-I-JSON integers beyond 2^53: an epoch that cannot
        // canonicalize cannot enter the chain — refusal, never coercion (the
        // same law the envelope module enforces on policy_epoch).
        let mut journal = StandingJournal::new();
        let draft = JournalEntryDraft {
            key_id: KID_A.to_string(),
            policy_epoch: 9_007_199_254_740_993,
            revocation_epoch: 0,
            standing: "VALID".to_string(),
            envelope_commitment: COMMITMENT.to_string(),
            receipt_hash: RECEIPT_HASH.to_string(),
        };
        match journal.append(draft) {
            Err(JournalError::Serialization(_)) => {}
            other => panic!("expected Serialization, got {other:?}"),
        }
        assert!(journal.is_empty(), "a refused append must not mutate");
    }

    // -- real receipts through the trust plane ----------------------------------------

    /// Builds a real ES256 signing key and its registry record (mirrors the
    /// rendered crypto_trust_verify court fixture — real bodies throughout).
    fn es256_fixture(tag: u8) -> (Es256SigningKey, KeyRecord) {
        let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid scalar seed");
        let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
        let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
        let record = KeyRecord {
            id: KeyId::from_fingerprint(&fingerprint),
            algorithm: AlgorithmId::Es256,
            fingerprint,
            custodian: CustodianIdentity {
                subject: "subject-a".to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key: public,
            created_epoch: NOW - 1_000,
        };
        (signing, record)
    }

    /// The domain-separated digest of a subject string — the exact law
    /// `certify_signed` enforces against `env.subject_digest`.
    fn subject_bound(subject: &str) -> [u8; 32] {
        crate::crypto_trust_canonical::digest(
            crate::crypto_trust_verify::DOMAIN_TAG,
            &[subject.as_bytes()],
        )
    }

    /// A fresh, in-window envelope naming `kid`.
    fn envelope(kid: &KeyId, nonce: [u8; 16]) -> SignatureEnvelope {
        SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm: AlgorithmId::Es256,
            key_id: kid.clone(),
            profile: CryptoProfile::Classical,
            policy_epoch: 1,
            revocation_epoch: 0,
            generation: 1,
            nonce,
            not_before: NOW - 1_000,
            expires_at: NOW + 1_000,
            subject_digest: subject_bound("subject-a"),
            audience: "affidavit.seal".to_string(),
        }
    }

    /// An engine holding `record` under the graph-default policy, clock at NOW.
    fn engine_with(record: &KeyRecord) -> VerificationEngine {
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            TrustPolicy::from_graph_defaults().with_now(NOW),
        )
    }

    #[test]
    fn record_receipt_binds_the_sealed_receipt_verbatim() {
        let (signing, record) = es256_fixture(31);
        let env = envelope(&record.id, [31u8; 16]);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine_with(&record)
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("VALID verdict mints a receipt");

        let mut journal = StandingJournal::new();
        let entry = record_receipt(&mut journal, &receipt, 4, 9).expect("record");
        assert_eq!(entry.seq, 0);
        assert_eq!(entry.prev, JOURNAL_GENESIS);
        assert_eq!(entry.key_id, receipt.key_id);
        assert_eq!(entry.standing, "VALID");
        assert_eq!(entry.standing, CryptographicStanding::Valid.as_str());
        assert_eq!(entry.envelope_commitment, receipt.envelope_commitment);
        assert_eq!(entry.receipt_hash, receipt.receipt_hash);
        assert_eq!(entry.policy_epoch, 4);
        assert_eq!(entry.revocation_epoch, 9);
        journal.verify_chain().expect("receipt chain verifies");

        // The projection is faithful: a journal reader can recover the exact
        // receipt identity without unsealing anything.
        assert_eq!(journal.by_key(&receipt.key_id).len(), 1);
        assert_eq!(journal.by_epoch(4).len(), 1);
        assert_eq!(journal.by_epoch(9).len(), 1);
        assert_eq!(journal.by_epoch(5).len(), 0);
    }

    #[test]
    fn receipt_journal_is_deterministic_and_survives_a_round_trip() {
        let (_, record_first) = es256_fixture(32);
        let mut journal_a = StandingJournal::new();
        let mut journal_b = StandingJournal::new();
        for (tag, nonce) in [(32u8, [32u8; 16]), (33, [33u8; 16])] {
            let (signing, record) = es256_fixture(tag);
            let env = envelope(&record.id, nonce);
            let signature =
                signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
            let receipt = engine_with(&record)
                .certify_signed(&env, &signature, "subject-a", &signing)
                .expect("mint");
            record_receipt(&mut journal_a, &receipt, 2, 0).expect("record a");
            record_receipt(&mut journal_b, &receipt, 2, 0).expect("record b");
        }
        assert_eq!(journal_a.to_jsonl(), journal_b.to_jsonl());
        let loaded = StandingJournal::from_jsonl(&journal_a.to_jsonl()).expect("honest wire loads");
        assert_eq!(loaded.entries(), journal_a.entries());
        assert_eq!(journal_a.by_key(&record_first.id.to_string()).len(), 1);
        assert_eq!(journal_a.len(), 2);
    }
}
