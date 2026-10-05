//! Transparency-log operations: the standing journal and the RFC 9162 Merkle
//! log bound into ONE operational surface for the affidavit cryptographic
//! trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//! Consumed query columns (log.rq): ops_file, ops_format, domain_tag.
//!
//! BINDING LAW: every [`crate::crypto_trust_verify::CryptoStandingReceipt`] the
//! plane mints is recorded here exactly once, and the same act appends exactly
//! one Merkle leaf equal to BLAKE3 over the receipt's `envelope_commitment`
//! hex. The commitment check ([`LogOpsError::CommitmentMismatch`], refused
//! BEFORE any state changes) is the tooth that binds log leaf i to journal
//! entry i: an audit re-derives the entire leaf set from the journal entries
//! alone and refuses any divergence, so the log cannot claim a receipt the
//! journal does not hold, and the journal cannot hold a receipt the log cannot
//! prove.
//!
//! HEAD LAW: every record re-signs the current tree head
//! ([`SignedTreeHead`]: ES256 over the domain-separated pre-image
//! `digest(HEAD_SIGNING_DOMAIN, [kid, tree_size-le, head])`), and
//! [`publish_head`] mints a head for the current tree without mutating state.
//! Heads are self-contained — an old head keeps verifying after the log grows
//! (append-only law) and still commits exactly its prefix; a consistency proof
//! bridges any two published sizes.
//!
//! PERSISTENCE LAW: the ops state (journal + leaves + heads) persists at
//! [`OPS_FILE`] as [`OPS_FORMAT`]-stamped JSONL, one line per journal entry,
//! log leaf, and stored head. [`TrustLogOps::from_jsonl`] re-audits every byte
//! it reads — format stamp, journal chain, leaf-set binding — and refuses
//! tampered state with a typed [`LogOpsError`]; the durable wire form is never
//! trusted without recomputation.
//!
//! House law: refusal-as-value. Every failure is a typed [`LogOpsError`]
//! variant; nothing panics, coerces, or fakes. A refused [`TrustLogOps::record`]
//! leaves no partial state behind.

use crate::crypto_trust_canonical::digest;
use crate::crypto_trust_es256::{verify_es256, Es256SigningKey};
use crate::crypto_trust_journal::{record_receipt, JournalEntry, JournalError, StandingJournal};
use crate::crypto_trust_transparency::{InclusionProof, TransparencyError, TransparencyLog};
use crate::crypto_trust_verify::CryptoStandingReceipt;

/// Durable ops state file (ctp:log-ops-v1 ctp:opsFile), relative to the
/// consumer repository root.
pub const OPS_FILE: &str = ".affi/transparency.jsonl";

/// Ops state wire-format identity (ctp:log-ops-v1 ctp:opsFormat), stamped on
/// every persisted JSONL line; load refuses any other value.
pub const OPS_FORMAT: &str = "CTP-LOGOPS-v1";

/// Domain-separation tag for [`SignedTreeHead`] signature pre-images
/// (ctp:policy-v1 ctp:domainTag). The canonical module is the single owner of
/// the plane's DOMAIN_TAG, so this const asserts its own graph binding against
/// `crate::crypto_trust_canonical::DOMAIN_TAG` (a graph-side change breaks the
/// render loudly instead of drifting silently).
pub const HEAD_SIGNING_DOMAIN: &str = "affidavit.crypto-trust-plane.v1";

/// Journal sequence number of a recorded receipt (equal to the leaf index of
/// its commitment binding, by construction).
pub type JournalSeq = u64;

/// Merkle leaf index of a recorded receipt's commitment binding.
pub type LeafIndex = u64;

/// Typed refusal of a log-operations transition. Refusals are data: every
/// variant names the exact law that broke.
#[derive(Debug, thiserror::Error)]
pub enum LogOpsError {
    /// The caller's commitment is not BLAKE3 over the receipt's
    /// `envelope_commitment` hex: the leaf would not bind the receipt to the
    /// log. Refused before any state changes.
    #[error("commitment mismatch: leaf must be blake3 of the receipt envelope_commitment hex")]
    CommitmentMismatch,
    /// The standing journal refused or failed verification (chain broken,
    /// sequence gap, serialization); the message carries the underlying
    /// journal refusal.
    #[error("journal: {0}")]
    Journal(String),
    /// The transparency log refused an operation, or the leaf-set audit found
    /// the log and the journal diverged (size or content).
    #[error("transparency: {0}")]
    Transparency(String),
    /// The ES256 head-signature layer refused its inputs (malformed public
    /// key or signature bytes).
    #[error("signing: {0}")]
    Signing(String),
    /// A persisted ops line was unparseable, carried a foreign format stamp
    /// or unknown kind, lacked its payload, or a hex field did not decode.
    #[error("serialization: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for LogOpsError {
    fn from(err: serde_json::Error) -> Self {
        LogOpsError::Serialization(err.to_string())
    }
}

impl From<JournalError> for LogOpsError {
    fn from(err: JournalError) -> Self {
        LogOpsError::Journal(err.to_string())
    }
}

impl From<TransparencyError> for LogOpsError {
    fn from(err: TransparencyError) -> Self {
        LogOpsError::Transparency(err.to_string())
    }
}

/// A signed RFC 9162 tree head: the log's publishable, self-contained promise
/// that the first `tree_size` leaves hash to `head_hex` at `timestamp`.
///
/// `kid` names the signing key and is bound INTO the signature pre-image, so a
/// head cannot be re-attributed to another key by editing the field — the
/// signature would no longer reproduce. The signature is DER-encoded ES256
/// over `digest(HEAD_SIGNING_DOMAIN, [kid, tree_size.to_le_bytes(), head])`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignedTreeHead {
    /// Number of leaves the head commits to (prefix size of the log).
    pub tree_size: u64,
    /// Lowercase-hex Merkle Tree Hash of the first `tree_size` leaves.
    pub head_hex: String,
    /// Caller-clock second the head was signed at (the plane never reads
    /// wall-clock time).
    pub timestamp: u64,
    /// The signer's key id, bound into the signature pre-image.
    pub kid: String,
    /// DER-encoded ES256 signature over the domain-separated head pre-image.
    pub signature: Vec<u8>,
}

/// The domain-separated ES256 pre-image for a tree head:
/// `digest(HEAD_SIGNING_DOMAIN, [kid, tree_size-le, head])`. The little-endian
/// size and the length-prefixed parts of [`digest`] keep pre-images
/// unambiguous under concatenation.
fn head_preimage(kid: &str, tree_size: u64, head: &[u8; 32]) -> [u8; 32] {
    digest(
        HEAD_SIGNING_DOMAIN,
        &[kid.as_bytes(), &tree_size.to_le_bytes(), head],
    )
}

/// Lowercase-hex rendering (the wire form of every 32-byte field here).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Decodes exactly 64 hex characters into a 32-byte digest; anything else is a
/// typed [`LogOpsError::Serialization`] — never a coercion.
fn hex_decode_32(s: &str) -> Result<[u8; 32], LogOpsError> {
    let bytes = s.as_bytes();
    if bytes.len() != 64 {
        return Err(LogOpsError::Serialization(format!(
            "digest hex must be 64 characters, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    for (i, pair) in bytes.chunks_exact(2).enumerate() {
        let hi = (pair[0] as char)
            .to_digit(16)
            .ok_or_else(|| LogOpsError::Serialization(format!("invalid hex digit in {s:?}")))?;
        let lo = (pair[1] as char)
            .to_digit(16)
            .ok_or_else(|| LogOpsError::Serialization(format!("invalid hex digit in {s:?}")))?;
        out[i] = ((hi << 4) | lo) as u8;
    }
    Ok(out)
}

/// The lawful log leaf for an envelope commitment: BLAKE3 over the commitment
/// hex (the binding between a journal entry and its log leaf).
fn commitment_leaf(envelope_commitment: &str) -> [u8; 32] {
    blake3::hash(envelope_commitment.as_bytes()).into()
}

impl SignedTreeHead {
    /// Signs `tree_size`/`head` at `now` with `signer` under `kid`.
    /// Deterministic: RFC 6979 makes the same head bytes under the same key
    /// always yield the same signature, so published heads are reproducible.
    fn sign(
        signer: &Es256SigningKey,
        kid: &str,
        tree_size: u64,
        head: &[u8; 32],
        now: u64,
    ) -> Self {
        let preimage = head_preimage(kid, tree_size, head);
        SignedTreeHead {
            tree_size,
            head_hex: hex_encode(head),
            timestamp: now,
            kid: kid.to_string(),
            signature: signer.sign(&preimage),
        }
    }
}

/// Verifies a [`SignedTreeHead`] against a SEC1-uncompressed ES256 public key:
/// the pre-image is recomputed from the head's OWN fields (kid, tree_size,
/// head_hex) and the signature adjudicated over it. `Ok(false)` is a
/// well-formed head that does not verify (wrong key, tampered field);
/// malformed key, signature, or head-hex bytes are typed refusals.
pub fn verify_head(head: &SignedTreeHead, public_key_sec1: &[u8]) -> Result<bool, LogOpsError> {
    let head_bytes = hex_decode_32(&head.head_hex)?;
    let preimage = head_preimage(&head.kid, head.tree_size, &head_bytes);
    verify_es256(public_key_sec1, &preimage, &head.signature)
        .map_err(|err| LogOpsError::Signing(err.to_string()))
}

/// One persisted ops line: the [`OPS_FORMAT`] stamp plus exactly one payload
/// (`entry` for journal lines, `leaf_hex` for leaf lines, `head` for head
/// lines). Serialization of each payload is total over its closed field
/// domain (strings, integers, byte vectors), so a skipped line cannot lose a
/// lawful item; the load path re-verifies every byte it reads regardless.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct OpsWireRecord {
    format: String,
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    entry: Option<JournalEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    leaf_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    head: Option<SignedTreeHead>,
}

impl OpsWireRecord {
    fn journal_line(entry: &JournalEntry) -> Result<String, LogOpsError> {
        let line = OpsWireRecord {
            format: OPS_FORMAT.to_string(),
            kind: "journal".to_string(),
            entry: Some(entry.clone()),
            leaf_hex: None,
            head: None,
        };
        Ok(serde_json::to_string(&line)?)
    }

    fn leaf_line(leaf: &[u8; 32]) -> Result<String, LogOpsError> {
        let line = OpsWireRecord {
            format: OPS_FORMAT.to_string(),
            kind: "leaf".to_string(),
            entry: None,
            leaf_hex: Some(hex_encode(leaf)),
            head: None,
        };
        Ok(serde_json::to_string(&line)?)
    }

    fn head_line(head: &SignedTreeHead) -> Result<String, LogOpsError> {
        let line = OpsWireRecord {
            format: OPS_FORMAT.to_string(),
            kind: "head".to_string(),
            entry: None,
            leaf_hex: None,
            head: Some(head.clone()),
        };
        Ok(serde_json::to_string(&line)?)
    }
}

/// The audit verdict over one ops state: journal depth, log depth, and whether
/// the two bind each other — every journal entry's envelope-commitment binding
/// present as the log leaf at the same position, sizes equal. Total: an
/// inconsistent state is REPORTED (`consistent == false`), never silently
/// passed; the load boundary ([`TrustLogOps::from_jsonl`]) is where tampered
/// state is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuditReport {
    /// Entries in the standing journal.
    pub journal_entries: u64,
    /// Leaves in the transparency log.
    pub log_leaves: u64,
    /// True when the journal and the log bind each other exactly.
    pub consistent: bool,
}

/// The operational surface binding [`StandingJournal`] and [`TransparencyLog`]:
/// every recorded receipt is journaled AND committed to the log in one
/// transition, heads are re-signed as the log grows, and the durable state
/// persists as re-audited JSONL. Constructed empty ([`TrustLogOps::new`]),
/// grown only through [`TrustLogOps::record`], and reloaded only through
/// [`TrustLogOps::from_jsonl`] (which refuses tampered wire forms) — so every
/// reachable state satisfies the binding law.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustLogOps {
    journal: StandingJournal,
    log: TransparencyLog,
    heads: Vec<SignedTreeHead>,
}

impl TrustLogOps {
    /// The empty ops surface: no journal entries, no leaves, no heads.
    pub fn new() -> Self {
        Self::default()
    }

    /// The standing journal (read-only access for auditors).
    pub fn journal(&self) -> &StandingJournal {
        &self.journal
    }

    /// The transparency log (read-only access for auditors).
    pub fn log(&self) -> &TransparencyLog {
        &self.log
    }

    /// The re-signed heads, one per record, in record order (read-only
    /// access). Head `i` commits exactly the first `i + 1` leaves.
    pub fn heads(&self) -> &[SignedTreeHead] {
        &self.heads
    }

    /// The log leaf at `index`, when it exists.
    pub fn leaf(&self, index: u64) -> Option<&[u8; 32]> {
        self.log.leaf(index)
    }

    /// THE LAW — record one minted receipt in one transition:
    ///
    /// 0. the caller's `commitment` MUST equal BLAKE3 over the receipt's
    ///    `envelope_commitment` hex — otherwise
    ///    [`LogOpsError::CommitmentMismatch`] and NO state changes;
    /// 1. the receipt is journaled via
    ///    `crate::crypto_trust_journal::record_receipt` (identity fields —
    ///    `key_id`, `standing`, `envelope_commitment`, `receipt_hash` — copied
    ///    verbatim; the log-ops boundary carries no rotation context, so both
    ///    epochs record the reserved context 0: rotation-context journaling
    ///    stays at the certification call site);
    /// 2. the commitment binding is appended as the next log leaf;
    /// 3. the current tree head is re-signed with `signer` (`signer_kid` bound
    ///    into the pre-image) and stored — head `i` always commits leaves
    ///    `0..=i`;
    ///
    /// returning `(journal seq, leaf index)` — equal by construction.
    pub fn record(
        &mut self,
        receipt: &CryptoStandingReceipt,
        commitment: [u8; 32],
        signer: &Es256SigningKey,
        signer_kid: &str,
        now: u64,
    ) -> Result<(JournalSeq, LeafIndex), LogOpsError> {
        if commitment != commitment_leaf(&receipt.envelope_commitment) {
            return Err(LogOpsError::CommitmentMismatch);
        }
        let entry = record_receipt(&mut self.journal, receipt, 0, 0)?;
        let leaf = self.log.append(commitment);
        let head = SignedTreeHead::sign(signer, signer_kid, self.log.len(), &self.log.head(), now);
        self.heads.push(head);
        Ok((entry.seq, leaf))
    }

    /// Publishes a signed head for the CURRENT tree without mutating state:
    /// the publishable form handed to parties that hold only the signer's
    /// public key (see [`verify_head`]). Deterministic in `(log, signer, now)`.
    pub fn publish_head(
        &self,
        signer: &Es256SigningKey,
        kid: &str,
        now: u64,
    ) -> Result<SignedTreeHead, LogOpsError> {
        Ok(SignedTreeHead::sign(
            signer,
            kid,
            self.log.len(),
            &self.log.head(),
            now,
        ))
    }

    /// RFC 9162 inclusion proof for leaf `leaf` against the current tree,
    /// paired with the current head bytes it must be verified against. An
    /// out-of-range index is a typed [`LogOpsError::Transparency`].
    pub fn prove_inclusion(&self, leaf: u64) -> Result<(InclusionProof, [u8; 32]), LogOpsError> {
        let proof = self.log.inclusion_proof(leaf)?;
        Ok((proof, self.log.head()))
    }

    /// The consistency check behind [`TrustLogOps::audit`]: the journal chain
    /// reproduces from genesis AND the log's leaf set is exactly the journal
    /// entries' envelope-commitment bindings, position for position, sizes
    /// equal. Tampered journal -> [`LogOpsError::Journal`]; diverged leaf set
    /// -> [`LogOpsError::Transparency`].
    fn check_consistency(&self) -> Result<(), LogOpsError> {
        self.journal.verify_chain()?;
        let expected_len = self.journal.len() as u64;
        if self.log.len() != expected_len {
            return Err(LogOpsError::Transparency(format!(
                "log size {} does not match journal size {}",
                self.log.len(),
                expected_len
            )));
        }
        for (index, entry) in self.journal.entries().iter().enumerate() {
            let expected = commitment_leaf(&entry.envelope_commitment);
            match self.log.leaf(index as u64) {
                Some(leaf) if *leaf == expected => {}
                Some(_) => {
                    return Err(LogOpsError::Transparency(format!(
                        "leaf {index} does not bind journal entry {index}"
                    )));
                }
                None => {
                    return Err(LogOpsError::Transparency(format!(
                        "leaf {index} missing from log"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Audits the ops state — journal chain verification plus the full
    /// leaf-set binding recomputed from the journal entries alone — reported
    /// as an [`AuditReport`]. Total over reachable state: inconsistency is
    /// reported, not refused.
    pub fn audit(&self) -> Result<AuditReport, LogOpsError> {
        Ok(AuditReport {
            journal_entries: self.journal.len() as u64,
            log_leaves: self.log.len(),
            consistent: self.check_consistency().is_ok(),
        })
    }

    /// The JSONL wire form persisted at [`OPS_FILE`]: one [`OPS_FORMAT`]-stamped
    /// line per journal entry (journal order), then per log leaf (leaf order),
    /// then per stored head (record order). Empty state serializes to the
    /// empty string. Payload serialization is total over the closed field
    /// domain, so a skipped line cannot lose a lawful item.
    pub fn to_jsonl(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for entry in self.journal.entries() {
            if let Ok(line) = OpsWireRecord::journal_line(entry) {
                lines.push(line);
            }
        }
        for index in 0..self.log.len() {
            if let Some(leaf) = self.log.leaf(index) {
                if let Ok(line) = OpsWireRecord::leaf_line(leaf) {
                    lines.push(line);
                }
            }
        }
        for head in &self.heads {
            if let Ok(line) = OpsWireRecord::head_line(head) {
                lines.push(line);
            }
        }
        lines.join("\n")
    }

    /// Loads the [`TrustLogOps::to_jsonl`] wire form and re-audits every byte:
    /// each line must parse and carry the [`OPS_FORMAT`] stamp with a known
    /// kind and payload (else [`LogOpsError::Serialization`]), the journal
    /// chain must reproduce from genesis (else [`LogOpsError::Journal`]), and
    /// the leaf set must bind the journal entries exactly (else
    /// [`LogOpsError::Transparency`]) — tampered state is REFUSED, never
    /// trusted. Empty (or whitespace-only) input loads the empty state.
    pub fn from_jsonl(s: &str) -> Result<Self, LogOpsError> {
        let mut entries: Vec<JournalEntry> = Vec::new();
        let mut leaves: Vec<[u8; 32]> = Vec::new();
        let mut heads: Vec<SignedTreeHead> = Vec::new();
        for line in s.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let wire: OpsWireRecord = serde_json::from_str(line)?;
            if wire.format != OPS_FORMAT {
                return Err(LogOpsError::Serialization(format!(
                    "ops line format {:?} is not {:?}",
                    wire.format, OPS_FORMAT
                )));
            }
            match wire.kind.as_str() {
                "journal" => entries.push(wire.entry.ok_or_else(|| {
                    LogOpsError::Serialization("journal line without entry".to_string())
                })?),
                "leaf" => {
                    let leaf_hex = wire.leaf_hex.ok_or_else(|| {
                        LogOpsError::Serialization("leaf line without leaf_hex".to_string())
                    })?;
                    leaves.push(hex_decode_32(&leaf_hex)?);
                }
                "head" => heads.push(wire.head.ok_or_else(|| {
                    LogOpsError::Serialization("head line without head".to_string())
                })?),
                other => {
                    return Err(LogOpsError::Serialization(format!(
                        "unknown ops line kind {other:?}"
                    )));
                }
            }
        }
        // Re-verify the journal chain by loading through the journal module's
        // own re-auditing path — the wire form is never trusted.
        let journal_jsonl = entries
            .iter()
            .filter_map(|entry| serde_json::to_string(entry).ok())
            .collect::<Vec<_>>()
            .join("\n");
        let journal = StandingJournal::from_jsonl(&journal_jsonl)?;
        let mut log = TransparencyLog::new();
        for leaf in leaves {
            log.append(leaf);
        }
        let ops = TrustLogOps {
            journal,
            log,
            heads,
        };
        ops.check_consistency()?;
        Ok(ops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_envelope::{NonceJournal, SignatureEnvelope, ENVELOPE_VERSION};
    use crate::crypto_trust_keys::{
        fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
        KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
    };
    use crate::crypto_trust_lifecycle::RevocationList;
    use crate::crypto_trust_transparency::{verify_consistency, verify_inclusion};
    use crate::crypto_trust_verify::{TrustPolicy, VerificationEngine};

    /// The clock used across the court (mirrors crypto_trust_verify).
    const NOW: u64 = 1_700_000_500;
    /// The transparency-log signer's key id.
    const SIGNER_KID: &str = "afk1_transparency-log-ops";

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
            // Bound to the fixture subject under the same domain-separated
            // law `certify_signed` enforces (mirrors crypto_trust_verify).
            subject_digest: subject_bound("subject-a"),
            audience: "affidavit.cli".to_string(),
        }
    }

    /// The domain-separated digest of a subject string — the exact law
    /// `certify_signed` enforces against `env.subject_digest`.
    fn subject_bound(subject: &str) -> [u8; 32] {
        crate::crypto_trust_canonical::digest(
            crate::crypto_trust_verify::DOMAIN_TAG,
            &[subject.as_bytes()],
        )
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

    /// Mints a REAL valid receipt through the real verification engine.
    fn mint(tag: u8) -> CryptoStandingReceipt {
        let (signing, record) = es256_fixture(tag);
        let env = envelope(&record.id, [tag; 16]);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        engine_with(&record)
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("VALID verdict mints a receipt")
    }

    /// The head signer and its public key.
    fn head_signer() -> Es256SigningKey {
        Es256SigningKey::from_seed(&[98; 32]).expect("valid scalar seed")
    }

    fn signer_pk() -> Vec<u8> {
        head_signer().public_key_sec1()
    }

    /// The lawful leaf commitment for a receipt (the tooth's own law,
    /// recomputed independently in the court).
    fn leaf_commitment(receipt: &CryptoStandingReceipt) -> [u8; 32] {
        blake3::hash(receipt.envelope_commitment.as_bytes()).into()
    }

    /// Records real receipts for `tags` into a fresh ops surface.
    fn ops_with(tags: &[u8]) -> TrustLogOps {
        let mut ops = TrustLogOps::new();
        let signer = head_signer();
        for (i, tag) in tags.iter().enumerate() {
            let receipt = mint(*tag);
            ops.record(
                &receipt,
                leaf_commitment(&receipt),
                &signer,
                SIGNER_KID,
                NOW + i as u64,
            )
            .unwrap_or_else(|e| panic!("record {i} must be lawful: {e}"));
        }
        ops
    }

    // -- teeth first: N records -> audit, inclusion proofs, head verify -------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(OPS_FILE, ".affi/transparency.jsonl");
        assert_eq!(OPS_FORMAT, "CTP-LOGOPS-v1");
        assert_eq!(HEAD_SIGNING_DOMAIN, "affidavit.crypto-trust-plane.v1");
        // Single-owner law: the head-signing domain is the canonical module's
        // DOMAIN_TAG and the journal's — one tag plane-wide, no copy drifts.
        assert_eq!(
            HEAD_SIGNING_DOMAIN,
            crate::crypto_trust_canonical::DOMAIN_TAG
        );
        assert_eq!(
            HEAD_SIGNING_DOMAIN,
            crate::crypto_trust_journal::JOURNAL_DOMAIN
        );
    }

    #[test]
    fn record_n_receipts_audits_proves_and_publishes() {
        let ops = ops_with(&[41, 42, 43, 44, 45]);

        // The audit binds journal and log, position for position.
        let report = ops.audit().expect("audit is total");
        assert_eq!(
            report,
            AuditReport {
                journal_entries: 5,
                log_leaves: 5,
                consistent: true,
            }
        );
        ops.journal().verify_chain().expect("chain verifies");

        // Binding law: leaf i is BLAKE3 of journal entry i's envelope
        // commitment hex, and the journaled identity is the receipt's verbatim.
        for (i, entry) in ops.journal().entries().iter().enumerate() {
            assert_eq!(
                ops.leaf(i as u64),
                Some(&blake3::hash(entry.envelope_commitment.as_bytes()).into()),
                "leaf {i} must bind journal entry {i}"
            );
            assert_eq!(entry.standing, "VALID");
            assert_eq!(entry.seq, i as u64);
        }

        // Every inclusion proof verifies against the current head, and only
        // for its own leaf.
        let head = ops.log().head();
        for i in 0..5u64 {
            let (proof, proof_head) = ops.prove_inclusion(i).expect("leaf exists");
            assert_eq!(proof.leaf_index, i);
            assert_eq!(proof.tree_size, 5);
            assert_eq!(proof_head, head);
            let leaf = ops.leaf(i).expect("leaf");
            assert!(verify_inclusion(leaf, &proof, &head), "leaf {i} verifies");
            let other = ops.leaf((i + 1) % 5).expect("other leaf");
            assert!(
                !verify_inclusion(other, &proof, &head),
                "leaf {i}'s proof must refuse leaf {}",
                (i + 1) % 5
            );
        }

        // A published head carries the current tree and verifies under the
        // signer's key.
        let published = ops
            .publish_head(&head_signer(), SIGNER_KID, NOW + 100)
            .expect("publish is lawful");
        assert_eq!(published.tree_size, 5);
        assert_eq!(published.head_hex, hex_encode(&head));
        assert!(matches!(verify_head(&published, &signer_pk()), Ok(true)));
        // Stored heads: one per record, each committing its own prefix.
        assert_eq!(ops.heads().len(), 5);
        for (i, stored) in ops.heads().iter().enumerate() {
            assert_eq!(stored.tree_size, i as u64 + 1);
            assert!(matches!(verify_head(stored, &signer_pk()), Ok(true)));
        }
    }

    #[test]
    fn record_journals_the_receipt_identity_verbatim() {
        let receipt = mint(46);
        let mut ops = TrustLogOps::new();
        let (seq, leaf) = ops
            .record(
                &receipt,
                leaf_commitment(&receipt),
                &head_signer(),
                SIGNER_KID,
                NOW,
            )
            .expect("record is lawful");
        assert_eq!((seq, leaf), (0, 0));
        let entry = &ops.journal().entries()[0];
        assert_eq!(entry.key_id, receipt.key_id);
        assert_eq!(entry.standing, receipt.standing.as_str());
        assert_eq!(entry.envelope_commitment, receipt.envelope_commitment);
        assert_eq!(entry.receipt_hash, receipt.receipt_hash);
    }

    // -- CommitmentMismatch: the binding tooth, and no partial state ----------

    #[test]
    fn wrong_commitment_refused_without_mutation() {
        let receipt = mint(50);
        let mut ops = TrustLogOps::new();
        let mut bad = leaf_commitment(&receipt);
        bad[0] ^= 0x01;
        match ops.record(&receipt, bad, &head_signer(), SIGNER_KID, NOW) {
            Err(LogOpsError::CommitmentMismatch) => {}
            other => panic!("expected CommitmentMismatch, got {other:?}"),
        }
        // The refusal left no partial state: nothing journaled, nothing
        // logged, nothing signed.
        assert_eq!(
            ops.audit().expect("audit is total"),
            AuditReport {
                journal_entries: 0,
                log_leaves: 0,
                consistent: true,
            }
        );
        assert!(ops.heads().is_empty());
        // The lawful commitment then records as the first (seq, leaf).
        let (seq, leaf) = ops
            .record(
                &receipt,
                leaf_commitment(&receipt),
                &head_signer(),
                SIGNER_KID,
                NOW,
            )
            .expect("lawful record");
        assert_eq!((seq, leaf), (0, 0));
    }

    // -- Journal variant: tampered journal line in the persisted state --------

    #[test]
    fn tampered_journal_line_refuses_as_journal_on_load() {
        let ops = ops_with(&[41, 42, 43]);
        let mut lines: Vec<String> = ops.to_jsonl().lines().map(str::to_string).collect();
        // Line 1 is journal entry 1: flip its decided standing. The stored
        // entry_hash no longer reproduces over the new material, and the
        // load path refuses with the typed Journal refusal.
        let mut tampered: serde_json::Value = serde_json::from_str(&lines[1]).expect("line parses");
        tampered["entry"]["standing"] = serde_json::Value::from("INVALID");
        lines[1] = serde_json::to_string(&tampered).expect("tampered line serializes");
        match TrustLogOps::from_jsonl(&lines.join("\n")) {
            Err(LogOpsError::Journal(_)) => {}
            other => panic!("expected Journal, got {other:?}"),
        }
    }

    // -- Transparency variant: truncated / diverged leaf lines -----------------

    #[test]
    fn truncated_leaf_count_refuses_as_transparency_on_load() {
        let ops = ops_with(&[41, 42, 43]);
        let mut lines: Vec<String> = ops.to_jsonl().lines().map(str::to_string).collect();
        // Drop one leaf line: the log can no longer bind every journal entry
        // (sizes must match — the audit recomputes the leaf set from the
        // journal alone).
        let leaf_line = lines
            .iter()
            .position(|l| l.contains("\"kind\":\"leaf\""))
            .expect("a leaf line exists");
        lines.remove(leaf_line);
        match TrustLogOps::from_jsonl(&lines.join("\n")) {
            Err(LogOpsError::Transparency(msg)) => {
                assert!(
                    msg.contains("does not match journal size"),
                    "the refusal must name the divergence: {msg}"
                );
            }
            other => panic!("expected Transparency, got {other:?}"),
        }
    }

    #[test]
    fn flipped_leaf_value_refuses_as_transparency_on_load() {
        let ops = ops_with(&[41, 42]);
        let mut lines: Vec<String> = ops.to_jsonl().lines().map(str::to_string).collect();
        let idx = lines
            .iter()
            .position(|l| l.contains("\"kind\":\"leaf\""))
            .expect("a leaf line exists");
        let mut tampered: serde_json::Value =
            serde_json::from_str(&lines[idx]).expect("line parses");
        let claimed = tampered["leaf_hex"]
            .as_str()
            .expect("hex field")
            .to_string();
        let mut chars: Vec<char> = claimed.chars().collect();
        chars[0] = if chars[0] == '0' { '1' } else { '0' };
        let flipped: String = chars.into_iter().collect();
        assert_ne!(flipped, claimed, "the flip must change the leaf");
        tampered["leaf_hex"] = serde_json::Value::from(flipped);
        lines[idx] = serde_json::to_string(&tampered).expect("tampered line serializes");
        match TrustLogOps::from_jsonl(&lines.join("\n")) {
            Err(LogOpsError::Transparency(_)) => {}
            other => panic!("expected Transparency, got {other:?}"),
        }
    }

    // -- Signing/Serialization variants: heads under wrong and malformed keys --

    #[test]
    fn heads_refuse_wrong_keys_and_malformed_material() {
        let ops = ops_with(&[41]);
        let head = ops
            .publish_head(&head_signer(), SIGNER_KID, NOW)
            .expect("publish is lawful");

        // Right key verifies.
        assert!(matches!(verify_head(&head, &signer_pk()), Ok(true)));
        // A different well-formed key: decided false, not a refusal.
        let other = Es256SigningKey::from_seed(&[97; 32]).expect("valid scalar seed");
        assert!(matches!(
            verify_head(&head, &other.public_key_sec1()),
            Ok(false)
        ));
        // Malformed public-key bytes: the typed Signing refusal.
        assert!(matches!(
            verify_head(&head, &[0x04, 0x01]),
            Err(LogOpsError::Signing(_))
        ));

        // Tampered tree size: well-formed signature over a changed pre-image.
        let mut tampered = head.clone();
        tampered.tree_size += 1;
        assert!(matches!(verify_head(&tampered, &signer_pk()), Ok(false)));
        // Tampered head hex: decodes, commits a different tree.
        let mut tampered = head.clone();
        tampered.head_hex = hex_encode(&[0u8; 32]);
        assert!(matches!(verify_head(&tampered, &signer_pk()), Ok(false)));
        // Tampered kid: the signer identity is bound into the pre-image.
        let mut tampered = head.clone();
        tampered.kid = "afk1_someone_else".to_string();
        assert!(matches!(verify_head(&tampered, &signer_pk()), Ok(false)));
        // Corrupted DER structure: the typed Signing refusal.
        let mut broken_sig = head.clone();
        broken_sig.signature[0] ^= 0xFF;
        assert!(matches!(
            verify_head(&broken_sig, &signer_pk()),
            Err(LogOpsError::Signing(_))
        ));
        // Non-hex head hex: the typed Serialization refusal.
        let mut broken_hex = head.clone();
        broken_hex.head_hex = "zz".repeat(32);
        assert!(matches!(
            verify_head(&broken_hex, &signer_pk()),
            Err(LogOpsError::Serialization(_))
        ));
    }

    // -- Serialization variant: foreign and malformed wire forms ---------------

    #[test]
    fn foreign_and_malformed_wire_refuses_as_serialization() {
        // Not JSON at all.
        assert!(matches!(
            TrustLogOps::from_jsonl("this is not json"),
            Err(LogOpsError::Serialization(_))
        ));
        let ops = ops_with(&[41]);
        // A foreign format stamp on an otherwise honest line.
        let mut lines: Vec<String> = ops.to_jsonl().lines().map(str::to_string).collect();
        lines[0] = lines[0].replace("CTP-LOGOPS-v1", "CTP-LOGOPS-v2");
        match TrustLogOps::from_jsonl(&lines.join("\n")) {
            Err(LogOpsError::Serialization(_)) => {}
            other => panic!("expected Serialization, got {other:?}"),
        }
        // An unknown kind.
        match TrustLogOps::from_jsonl(r#"{"format":"CTP-LOGOPS-v1","kind":"mystery"}"#) {
            Err(LogOpsError::Serialization(_)) => {}
            other => panic!("expected Serialization, got {other:?}"),
        }
        // A journal line without its payload.
        match TrustLogOps::from_jsonl(r#"{"format":"CTP-LOGOPS-v1","kind":"journal"}"#) {
            Err(LogOpsError::Serialization(_)) => {}
            other => panic!("expected Serialization, got {other:?}"),
        }
        // A leaf line whose hex does not decode.
        match TrustLogOps::from_jsonl(
            r#"{"format":"CTP-LOGOPS-v1","kind":"leaf","leaf_hex":"nothex"}"#,
        ) {
            Err(LogOpsError::Serialization(_)) => {}
            other => panic!("expected Serialization, got {other:?}"),
        }
    }

    // -- determinism ------------------------------------------------------------

    #[test]
    fn identical_records_produce_identical_state() {
        let a = ops_with(&[41, 42]);
        let b = ops_with(&[41, 42]);
        assert_eq!(a, b, "same receipts, signer, and clock must agree");
        assert_eq!(a.to_jsonl(), b.to_jsonl());
        // Published heads are reproducible too (RFC 6979 signing).
        let head_a = a
            .publish_head(&head_signer(), SIGNER_KID, NOW)
            .expect("publish a");
        let head_b = b
            .publish_head(&head_signer(), SIGNER_KID, NOW)
            .expect("publish b");
        assert_eq!(head_a, head_b);
    }

    // -- JSONL wire form ------------------------------------------------------------

    #[test]
    fn jsonl_round_trips_and_reaudits() {
        let ops = ops_with(&[41, 42, 43, 44]);
        let jsonl = ops.to_jsonl();
        assert_eq!(jsonl.lines().count(), 12, "4 journal + 4 leaf + 4 head");
        assert!(
            jsonl.lines().all(|l| l.contains(OPS_FORMAT)),
            "every line carries the format stamp"
        );
        let loaded = TrustLogOps::from_jsonl(&jsonl).expect("honest wire form loads");
        assert_eq!(loaded, ops);
        assert_eq!(loaded.to_jsonl(), jsonl);
        assert!(loaded.audit().expect("audit is total").consistent);
    }

    #[test]
    fn empty_state_round_trips_through_empty_wire() {
        let ops = TrustLogOps::new();
        assert_eq!(ops.to_jsonl(), "");
        let loaded = TrustLogOps::from_jsonl("").expect("empty wire loads empty");
        assert_eq!(loaded, ops);
        assert_eq!(
            loaded.audit().expect("audit is total"),
            AuditReport {
                journal_entries: 0,
                log_leaves: 0,
                consistent: true,
            }
        );
    }

    #[test]
    fn inclusion_refuses_out_of_range_leaf_as_transparency() {
        let ops = ops_with(&[42]);
        match ops.prove_inclusion(1) {
            Err(LogOpsError::Transparency(_)) => {}
            other => panic!("expected Transparency, got {other:?}"),
        }
    }

    // -- growth law: old heads remain valid, consistency bridges ----------------

    #[test]
    fn growth_keeps_old_heads_valid_and_bridges_by_consistency() {
        let mut ops = ops_with(&[41, 42, 43]);
        let signer = head_signer();
        let head_at_3 = ops.log().head();
        let old_heads: Vec<SignedTreeHead> = ops.heads().to_vec();
        let proofs: Vec<InclusionProof> = (0..3)
            .map(|i| ops.log().inclusion_proof(i).expect("proof exists"))
            .collect();
        assert_eq!(old_heads.len(), 3);
        assert_eq!(old_heads[2].tree_size, 3);

        // Grow the log with two more real receipts.
        for tag in [44, 45] {
            let receipt = mint(tag);
            ops.record(
                &receipt,
                leaf_commitment(&receipt),
                &signer,
                SIGNER_KID,
                NOW,
            )
            .expect("growth is lawful");
        }
        assert_eq!(ops.log().len(), 5);
        assert_eq!(ops.heads().len(), 5);

        // Every old stored head still verifies, byte for byte, and still
        // commits exactly its own prefix of the grown log.
        for old in &old_heads {
            assert!(
                matches!(verify_head(old, &signer_pk()), Ok(true)),
                "head at size {} must remain valid after growth",
                old.tree_size
            );
            let mut prefix = TransparencyLog::new();
            for i in 0..old.tree_size {
                prefix.append(*ops.leaf(i).expect("leaf exists"));
            }
            assert_eq!(
                old.head_hex,
                hex_encode(&prefix.head()),
                "old head must commit exactly its prefix"
            );
        }

        // Every pre-growth inclusion proof still verifies against the
        // pre-growth head — history never moved.
        for (i, proof) in proofs.iter().enumerate() {
            assert!(
                verify_inclusion(ops.leaf(i as u64).expect("leaf"), proof, &head_at_3),
                "pre-growth proof for leaf {i} must survive growth"
            );
        }

        // And the grown tree binds the old head through a consistency proof:
        // append-only extension, not a rehash.
        let bridge = ops.log().consistency_proof(3).expect("proof exists");
        assert!(verify_consistency(&bridge, &head_at_3, &ops.log().head()));
    }

    // -- error display strings are wire stable ----------------------------------

    #[test]
    fn error_display_strings_are_wire_stable() {
        assert_eq!(
            LogOpsError::CommitmentMismatch.to_string(),
            "commitment mismatch: leaf must be blake3 of the receipt envelope_commitment hex"
        );
        assert_eq!(
            LogOpsError::Journal("chain".to_string()).to_string(),
            "journal: chain"
        );
        assert_eq!(
            LogOpsError::Transparency("size".to_string()).to_string(),
            "transparency: size"
        );
        assert_eq!(
            LogOpsError::Signing("pk".to_string()).to_string(),
            "signing: pk"
        );
        assert_eq!(
            LogOpsError::Serialization("line".to_string()).to_string(),
            "serialization: line"
        );
    }
}
