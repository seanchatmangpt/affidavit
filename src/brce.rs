//! BRCE receipt ledger and zero-unreceipted-actuation court.
//!
//! Evidence-side implementation of `docs/integrations/BRCE_RECEIPT_ADAPTER_V0_1.md`
//! (profiles `affidavit/brce-actuation/v1`, `affidavit/brce-reconciliation/v1`,
//! `affidavit/brce-replay/v1`) against the BRCE Protocol RFC-0001 v0.1 pipeline:
//!
//! ```text
//! parse -> route -> admit/refuse -> construct -> DO -> receipt -> replay -> standing
//! ```
//!
//! Three pieces, deliberately separate:
//!
//! * [`BrceLedger`] — an append-only, BLAKE3 hash-chained log of pipeline records,
//!   optionally persisted as JSON lines (one record per line, fsync'd on append).
//!   The ledger accepts *any* record through [`BrceLedger::append`]; it is a log,
//!   not an authority.
//! * [`BrcePipeline`] — the lawful path. It refuses to construct un-admitted work,
//!   refuses DO whose authority grant does not bind the exact subject / operation /
//!   target / construct digest, writes a `PREPARED` actuation intent *before*
//!   calling the actuator, and emits a digest-identified receipt after it.
//! * [`court`] — judges an arbitrary ledger plus the observed external world and
//!   refuses any consequence without a receipt (`ZERO_UNRECEIPTED_ACTUATION`),
//!   together with the other BRCE Core safety rules listed in [`Rule`].
//!
//! Receipt validity never grants authority (`ReceiptValid ⇏ AuthorityGranted`) and
//! nothing in this module performs DO except through a caller-supplied [`Actuator`].

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Actuation receipt profile identifier.
pub const PROFILE_ACTUATION: &str = "affidavit/brce-actuation/v1";
/// Crash-window reconciliation profile identifier.
pub const PROFILE_RECONCILIATION: &str = "affidavit/brce-reconciliation/v1";
/// Consequence-free replay profile identifier.
pub const PROFILE_REPLAY: &str = "affidavit/brce-replay/v1";
/// Canonicalization used for every digest in this module.
pub const CANONICALIZATION: &str = "serde_json-struct-order+blake3";
/// Chain anchor for the first record.
pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Errors raised by the ledger or pipeline.
#[derive(Debug, thiserror::Error)]
pub enum BrceError {
    /// Filesystem failure while persisting or loading the ledger.
    #[error("ledger io: {0}")]
    Io(#[from] std::io::Error),
    /// A persisted line was not a ledger record.
    #[error("ledger line {line} is not a record: {reason}")]
    Malformed {
        /// 1-based line number.
        line: usize,
        /// Parser message.
        reason: String,
    },
    /// The pipeline refused a transition; the refusal is also recorded in the ledger.
    #[error("REFUSED({0})")]
    Refused(String),
    /// The pipeline cannot decide; the state stays typed as BLOCKED.
    #[error("BLOCKED({0})")]
    Blocked(String),
}

/// Hex BLAKE3 digest of the canonical JSON encoding of `value`.
///
/// Struct fields serialize in declaration order and maps are `BTreeMap`, so the
/// encoding is deterministic.
#[must_use]
pub fn digest<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    blake3::hash(&bytes).to_hex().to_string()
}

/// RFC-0001 §5 request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    /// Caller-assigned request id.
    pub request_id: String,
    /// Exact subject (e.g. a git SHA) the request is about.
    pub subject: String,
    /// Requested operation (e.g. `write`).
    pub operation: String,
    /// Requested target (e.g. a file name).
    pub target: String,
    /// Operation parameters.
    pub parameters: BTreeMap<String, String>,
    /// Who asked.
    pub requester: String,
}

/// RFC-0001 §6 routing decision. Routing never grants authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDecision {
    /// Capability that could handle the request.
    pub capability: String,
    /// Executor class.
    pub executor_class: String,
}

/// RFC-0001 §7 admission outcome; the four outcomes never collapse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", content = "reason")]
pub enum Admission {
    /// Known requirements satisfied.
    Admitted,
    /// Understood but prohibited.
    Refused(String),
    /// A required fact is unresolved.
    Blocked(String),
    /// The capability does not exist.
    Unsupported(String),
}

/// RFC-0001 §9 constructed action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructedAction {
    /// Digest of the request this construct realizes.
    pub request_digest: String,
    /// Exact subject.
    pub subject: String,
    /// Operation.
    pub operation: String,
    /// Target.
    pub target: String,
    /// Parameters (the payload for write-class operations).
    pub parameters: BTreeMap<String, String>,
    /// Scoped consequence identity (retries keep it).
    pub consequence_id: String,
    /// Whether re-execution of this consequence is idempotent.
    pub idempotent: bool,
}

impl ConstructedAction {
    /// `H(Canonical(construct))`.
    #[must_use]
    pub fn construct_digest(&self) -> String {
        digest(self)
    }
}

/// RFC-0001 §10 authority grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityGrant {
    /// Grant id.
    pub grant_id: String,
    /// Issuer (recorded, never verified here: `ValidReceipt ⇏ ValidIssuerAuthority`).
    pub issuer: String,
    /// Bound subject.
    pub subject: String,
    /// Bound operation.
    pub operation: String,
    /// Bound target.
    pub target: String,
    /// Bound construct digest.
    pub construct_digest: String,
    /// Logical expiry instant (exclusive).
    pub expires_at: u64,
    /// Maximum distinct consequences this grant may authorize.
    pub maximum_uses: u32,
}

impl AuthorityGrant {
    /// `H(Canonical(grant))`.
    #[must_use]
    pub fn grant_digest(&self) -> String {
        digest(self)
    }
}

/// What an actuator reports after DO.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActuationResult {
    /// `SUCCESS` / `FAILURE`.
    pub result_class: String,
    /// Digest of the actuator's result payload.
    pub result_digest: String,
    /// Whether external state changed (`executed ⇏ changed`).
    pub changed: bool,
}

/// What an observer sees of one consequence in the external world.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Observation {
    /// The effect exists; digest of the observed state.
    Effect(String),
    /// Positively observed that no effect exists.
    NoEffect,
    /// The observer cannot tell.
    Unknown,
}

/// The only component allowed to create a governed consequence.
pub trait Actuator {
    /// Executor identity bound into receipts.
    fn identity(&self) -> String;
    /// Perform the constructed action.
    ///
    /// # Errors
    /// Returns an I/O error when the consequence could not be attempted.
    fn execute(&mut self, action: &ConstructedAction) -> std::io::Result<ActuationResult>;
}

/// Read-only view of the external world, used for reconciliation and by the court.
pub trait Observer {
    /// Observe one consequence.
    fn observe(&self, consequence_id: &str) -> Observation;
    /// Every consequence id present in the world (for `ZERO_UNRECEIPTED_ACTUATION`).
    fn consequences(&self) -> Vec<String>;
}

/// Filesystem actuator: `write` puts `parameters["content"]` into `<root>/<consequence_id>`.
///
/// This is a real external effect (a file outside the ledger), not a simulation.
#[derive(Debug)]
pub struct FileActuator {
    root: PathBuf,
}

impl FileActuator {
    /// Actuate inside `root` (created if missing).
    ///
    /// # Errors
    /// Fails when `root` cannot be created.
    pub fn new(root: impl Into<PathBuf>) -> std::io::Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Directory the actuator writes into.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl Actuator for FileActuator {
    fn identity(&self) -> String {
        "affidavit::brce::FileActuator/v1".to_string()
    }

    fn execute(&mut self, action: &ConstructedAction) -> std::io::Result<ActuationResult> {
        let content = action
            .parameters
            .get("content")
            .cloned()
            .unwrap_or_default();
        let path = self.root.join(&action.consequence_id);
        let changed = std::fs::read(&path).ok().as_deref() != Some(content.as_bytes());
        if changed {
            std::fs::write(&path, content.as_bytes())?;
        }
        Ok(ActuationResult {
            result_class: "SUCCESS".to_string(),
            result_digest: blake3::hash(content.as_bytes()).to_hex().to_string(),
            changed,
        })
    }
}

impl Observer for FileActuator {
    fn observe(&self, consequence_id: &str) -> Observation {
        match std::fs::read(self.root.join(consequence_id)) {
            Ok(bytes) => Observation::Effect(blake3::hash(&bytes).to_hex().to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Observation::NoEffect,
            Err(_) => Observation::Unknown,
        }
    }

    fn consequences(&self) -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(&self.root)
            .map(|rd| {
                rd.filter_map(Result::ok)
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default();
        out.sort();
        out
    }
}

/// Actuation section of a receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActuationEvidence {
    /// Executor identity.
    pub executor_identity: String,
    /// Logical start instant.
    pub started_at: u64,
    /// Logical completion instant.
    pub completed_at: u64,
    /// Result class.
    pub result_class: String,
    /// Result digest.
    pub result_digest: String,
}

/// Effect section of a receipt (`executed`, `changed`, `verified` stay distinct).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectEvidence {
    /// DO ran.
    pub executed: bool,
    /// External state changed.
    pub changed: bool,
    /// Digest of the observed external state.
    pub effect_digest: String,
}

/// Verification section of a receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationEvidence {
    /// Verifier identity.
    pub verifier_identity: String,
    /// `EFFECT_VERIFIED` / `EFFECT_MISMATCH` / `EFFECT_CONFIRMED`.
    pub verdict: String,
    /// Digest of the verification evidence.
    pub evidence_digest: String,
}

/// Replay identity section of a receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayIdentity {
    /// Entrypoint that reconstructs the evidence path.
    pub command_or_entrypoint: String,
    /// Tool identity.
    pub tool_identity: String,
    /// Tool version.
    pub tool_version: String,
}

/// `affidavit/brce-actuation/v1` (or `brce-reconciliation/v1`) receipt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrceReceipt {
    /// Profile identifier.
    pub profile: String,
    /// Run id.
    pub run_id: String,
    /// Exact subject.
    pub subject: String,
    /// Request digest.
    pub request_digest: String,
    /// Route digest.
    pub route_digest: String,
    /// Admission digest.
    pub admission_digest: String,
    /// Construct digest.
    pub construct_digest: String,
    /// Authority grant digest presented to DO.
    pub authority_grant_digest: String,
    /// Consequence id.
    pub consequence_id: String,
    /// Attempt id.
    pub attempt_id: String,
    /// Actuation evidence.
    pub actuation: ActuationEvidence,
    /// Effect evidence.
    pub effect: EffectEvidence,
    /// Verification evidence.
    pub verification: VerificationEvidence,
    /// Replay identity.
    pub replay: ReplayIdentity,
    /// Canonicalization identifier.
    pub canonicalization: String,
    /// Previous receipt digest in this ledger (or [`GENESIS`]).
    pub previous_receipt: String,
    /// `H(receipt with receipt_digest = "")`; also the receipt id.
    pub receipt_digest: String,
}

impl BrceReceipt {
    /// Recompute the digest with `receipt_digest` blanked.
    #[must_use]
    pub fn compute_digest(&self) -> String {
        let mut body = self.clone();
        body.receipt_digest = String::new();
        digest(&body)
    }

    /// Fill `receipt_digest` from the body.
    #[must_use]
    pub fn seal(mut self) -> Self {
        self.receipt_digest = self.compute_digest();
        self
    }
}

/// Reconciliation verdicts (adapter §6). `ExecutionUnknown` is never promoted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationVerdict {
    /// Effect observed and matches the prepared construct.
    EffectConfirmed,
    /// Positively observed that no effect exists.
    NoEffectConfirmed,
    /// Cannot tell.
    ExecutionUnknown,
    /// Observed state contradicts the construct.
    BlockedReconciliation,
}

/// One pipeline entry.
///
/// Variant sizes differ because receipts are carried inline; entries are a
/// serialized, append-only log value (not a hot in-memory type), so the ledger keeps
/// the plain JSON shape instead of boxing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "kind")]
pub enum Entry {
    /// Parse.
    Parsed {
        /// Full request (kept so replay can recompute every digest).
        request: Request,
        /// `H(request)`.
        request_digest: String,
    },
    /// Route.
    Routed {
        /// Request digest.
        request_digest: String,
        /// Decision.
        route: RouteDecision,
        /// `H(route)`.
        route_digest: String,
    },
    /// Admission outcome.
    Admitted {
        /// Request digest.
        request_digest: String,
        /// Outcome.
        admission: Admission,
        /// `H(admission)`.
        admission_digest: String,
    },
    /// Construct.
    Constructed {
        /// Full construct.
        action: ConstructedAction,
        /// `H(action)`.
        construct_digest: String,
    },
    /// Recoverable pre-actuation intent (RFC-0001 §14 `PREPARED`).
    Prepared {
        /// Consequence id.
        consequence_id: String,
        /// Attempt id.
        attempt_id: String,
        /// Construct digest DO will execute.
        construct_digest: String,
        /// Grant presented.
        grant: AuthorityGrant,
        /// Logical instant of the authority check.
        at: u64,
    },
    /// DO happened (written after the actuator returns).
    Done {
        /// Consequence id.
        consequence_id: String,
        /// Attempt id.
        attempt_id: String,
        /// Construct digest executed.
        construct_digest: String,
        /// Actuator result.
        result: ActuationResult,
        /// Executor identity.
        executor_identity: String,
        /// Logical completion instant.
        at: u64,
    },
    /// Receipt persisted.
    Receipted {
        /// The receipt.
        receipt: BrceReceipt,
    },
    /// Crash-window reconciliation outcome.
    Reconciled {
        /// Consequence id.
        consequence_id: String,
        /// Prepared record digest reconciled.
        prepared_record_digest: String,
        /// Verdict.
        verdict: ReconciliationVerdict,
        /// Digest of the observation used.
        evidence_digest: String,
    },
    /// A typed refusal recorded by the pipeline.
    Refusal {
        /// Stage that refused.
        stage: String,
        /// Reason.
        reason: String,
    },
}

/// One chained ledger record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerRecord {
    /// 0-based sequence number.
    pub seq: u64,
    /// Digest of the previous record (or [`GENESIS`]).
    pub prev: String,
    /// Entry.
    pub entry: Entry,
    /// `H(seq, prev, entry)`.
    pub digest: String,
}

/// Digest of a record body.
#[must_use]
pub fn record_digest(seq: u64, prev: &str, entry: &Entry) -> String {
    digest(&(seq, prev, entry))
}

/// Append-only, hash-chained BRCE ledger.
#[derive(Debug, Default)]
pub struct BrceLedger {
    records: Vec<LedgerRecord>,
    path: Option<PathBuf>,
}

impl BrceLedger {
    /// In-memory ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ledger persisted to `path` (JSON lines). Existing records are loaded verbatim;
    /// chain validity is judged by the [`court`], not assumed here.
    ///
    /// # Errors
    /// I/O failure or a line that is not a record.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, BrceError> {
        let path = path.into();
        let mut records = Vec::new();
        if path.exists() {
            let reader = BufReader::new(File::open(&path)?);
            for (i, line) in reader.lines().enumerate() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let rec: LedgerRecord =
                    serde_json::from_str(&line).map_err(|e| BrceError::Malformed {
                        line: i + 1,
                        reason: e.to_string(),
                    })?;
                records.push(rec);
            }
        }
        Ok(Self {
            records,
            path: Some(path),
        })
    }

    /// Append an entry (chained to the current head) and persist it durably.
    ///
    /// # Errors
    /// I/O failure while persisting.
    pub fn append(&mut self, entry: Entry) -> Result<&LedgerRecord, BrceError> {
        let seq = self.records.len() as u64;
        let prev = self.head();
        let digest = record_digest(seq, &prev, &entry);
        let rec = LedgerRecord {
            seq,
            prev,
            entry,
            digest,
        };
        if let Some(path) = &self.path {
            let mut f = OpenOptions::new().create(true).append(true).open(path)?;
            let mut line = serde_json::to_vec(&rec).map_err(|e| BrceError::Malformed {
                line: 0,
                reason: e.to_string(),
            })?;
            line.push(b'\n');
            f.write_all(&line)?;
            f.sync_all()?;
        }
        self.records.push(rec);
        Ok(self.records.last().expect("just pushed"))
    }

    /// Records in append order.
    #[must_use]
    pub fn records(&self) -> &[LedgerRecord] {
        &self.records
    }

    /// Digest of the last record (or [`GENESIS`]).
    #[must_use]
    pub fn head(&self) -> String {
        self.records
            .last()
            .map_or_else(|| GENESIS.to_string(), |r| r.digest.clone())
    }

    fn last_receipt_digest(&self) -> String {
        self.records
            .iter()
            .rev()
            .find_map(|r| match &r.entry {
                Entry::Receipted { receipt } => Some(receipt.receipt_digest.clone()),
                _ => None,
            })
            .unwrap_or_else(|| GENESIS.to_string())
    }

    fn find<'a, T>(&'a self, f: impl Fn(&'a Entry) -> Option<T>) -> Option<T> {
        self.records.iter().rev().find_map(|r| f(&r.entry))
    }
}

/// Everything the pipeline carries from admission to DO for one request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admitted {
    /// Request.
    pub request: Request,
    /// Request digest.
    pub request_digest: String,
    /// Route digest.
    pub route_digest: String,
    /// Admission digest.
    pub admission_digest: String,
}

/// Lawful BRCE path over a ledger.
#[derive(Debug)]
pub struct BrcePipeline {
    /// The ledger this pipeline writes.
    pub ledger: BrceLedger,
    run_id: String,
    tool_version: String,
}

impl BrcePipeline {
    /// New pipeline writing `ledger` for `run_id`.
    #[must_use]
    pub fn new(ledger: BrceLedger, run_id: impl Into<String>) -> Self {
        Self {
            ledger,
            run_id: run_id.into(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn refuse(&mut self, stage: &str, reason: String) -> BrceError {
        let _ = self.ledger.append(Entry::Refusal {
            stage: stage.to_string(),
            reason: reason.clone(),
        });
        BrceError::Refused(reason)
    }

    /// parse → route → admit. `policy` decides admission (routing never does).
    ///
    /// # Errors
    /// `Refused`/`Blocked` when admission is anything but `Admitted`.
    pub fn admit(
        &mut self,
        request: Request,
        route: RouteDecision,
        policy: impl Fn(&Request, &RouteDecision) -> Admission,
    ) -> Result<Admitted, BrceError> {
        let request_digest = digest(&request);
        self.ledger.append(Entry::Parsed {
            request: request.clone(),
            request_digest: request_digest.clone(),
        })?;
        let route_digest = digest(&route);
        self.ledger.append(Entry::Routed {
            request_digest: request_digest.clone(),
            route: route.clone(),
            route_digest: route_digest.clone(),
        })?;
        let admission = policy(&request, &route);
        let admission_digest = digest(&admission);
        self.ledger.append(Entry::Admitted {
            request_digest: request_digest.clone(),
            admission: admission.clone(),
            admission_digest: admission_digest.clone(),
        })?;
        match admission {
            Admission::Admitted => Ok(Admitted {
                request,
                request_digest,
                route_digest,
                admission_digest,
            }),
            Admission::Refused(r) => Err(BrceError::Refused(r)),
            Admission::Blocked(r) => Err(BrceError::Blocked(r)),
            Admission::Unsupported(r) => Err(BrceError::Refused(format!("UNSUPPORTED:{r}"))),
        }
    }

    /// construct. Creates no consequence.
    ///
    /// # Errors
    /// I/O failure while appending.
    pub fn construct(
        &mut self,
        admitted: &Admitted,
        consequence_id: &str,
        idempotent: bool,
    ) -> Result<ConstructedAction, BrceError> {
        let r = &admitted.request;
        let action = ConstructedAction {
            request_digest: admitted.request_digest.clone(),
            subject: r.subject.clone(),
            operation: r.operation.clone(),
            target: r.target.clone(),
            parameters: r.parameters.clone(),
            consequence_id: consequence_id.to_string(),
            idempotent,
        };
        self.ledger.append(Entry::Constructed {
            construct_digest: action.construct_digest(),
            action: action.clone(),
        })?;
        Ok(action)
    }

    /// RFC-0001 §13 DO preconditions, evaluated against the ledger.
    fn authorize(
        &self,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        now: u64,
    ) -> Result<(), String> {
        authority_defect(action, &action.construct_digest(), grant, now).map_or(Ok(()), Err)?;
        let gd = grant.grant_digest();
        let used: BTreeSet<&str> = self
            .ledger
            .records()
            .iter()
            .filter_map(|r| match &r.entry {
                Entry::Prepared {
                    grant,
                    consequence_id,
                    ..
                } if grant.grant_digest() == gd => Some(consequence_id.as_str()),
                _ => None,
            })
            .collect();
        if !used.contains(action.consequence_id.as_str()) && used.len() as u32 >= grant.maximum_uses
        {
            return Err(format!("GRANT_EXHAUSTED:{}", grant.grant_id));
        }
        Ok(())
    }

    /// Write the `PREPARED` actuation intent (first half of DO).
    ///
    /// # Errors
    /// `Refused` on any authority defect; `Blocked` when the consequence has an
    /// unreconciled crash window and is not idempotent.
    pub fn prepare(
        &mut self,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        now: u64,
    ) -> Result<(), BrceError> {
        if let Err(reason) = self.authorize(action, grant, now) {
            return Err(self.refuse("DO", reason));
        }
        if self.receipted(&action.consequence_id) && !action.idempotent {
            return Err(self.refuse("DO", format!("AT_MOST_ONCE:{}", action.consequence_id)));
        }
        if !action.idempotent && self.pending(&action.consequence_id) {
            let reason = format!("RECONCILE_BEFORE_RETRY:{}", action.consequence_id);
            let _ = self.ledger.append(Entry::Refusal {
                stage: "DO".to_string(),
                reason: reason.clone(),
            });
            return Err(BrceError::Blocked(reason));
        }
        self.ledger.append(Entry::Prepared {
            consequence_id: action.consequence_id.clone(),
            attempt_id: attempt_id.to_string(),
            construct_digest: action.construct_digest(),
            grant: grant.clone(),
            at: now,
        })?;
        Ok(())
    }

    /// Call the actuator and record `Done` (second half of DO). A crash after this
    /// returns and before [`BrcePipeline::receipt`] is the reconciliation window.
    ///
    /// # Errors
    /// I/O failure from the actuator or ledger.
    pub fn execute(
        &mut self,
        action: &ConstructedAction,
        attempt_id: &str,
        actuator: &mut dyn Actuator,
        now: u64,
    ) -> Result<ActuationResult, BrceError> {
        let prepared = self.ledger.find(|e| match e {
            Entry::Prepared {
                consequence_id,
                attempt_id: a,
                construct_digest,
                ..
            } if consequence_id == &action.consequence_id && a == attempt_id => {
                Some(construct_digest.clone())
            }
            _ => None,
        });
        if prepared.as_deref() != Some(action.construct_digest().as_str()) {
            return Err(self.refuse("DO", format!("NOT_PREPARED:{}", action.consequence_id)));
        }
        let result = actuator.execute(action)?;
        self.ledger.append(Entry::Done {
            consequence_id: action.consequence_id.clone(),
            attempt_id: attempt_id.to_string(),
            construct_digest: action.construct_digest(),
            result: result.clone(),
            executor_identity: actuator.identity(),
            at: now,
        })?;
        Ok(result)
    }

    /// Verify the effect through `observer` and persist the actuation receipt.
    ///
    /// # Errors
    /// `Refused` when no `Done` exists for this attempt.
    pub fn receipt(
        &mut self,
        admitted: &Admitted,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        observer: &dyn Observer,
    ) -> Result<BrceReceipt, BrceError> {
        let done = self.ledger.find(|e| match e {
            Entry::Done {
                consequence_id,
                attempt_id: a,
                result,
                executor_identity,
                at,
                ..
            } if consequence_id == &action.consequence_id && a == attempt_id => {
                Some((result.clone(), executor_identity.clone(), *at))
            }
            _ => None,
        });
        let prepared_at = self.ledger.find(|e| match e {
            Entry::Prepared {
                consequence_id,
                attempt_id: a,
                at,
                ..
            } if consequence_id == &action.consequence_id && a == attempt_id => Some(*at),
            _ => None,
        });
        let (Some((result, executor, completed_at)), Some(started_at)) = (done, prepared_at) else {
            return Err(self.refuse("RECEIPT", format!("NO_DO:{}", action.consequence_id)));
        };
        let (verdict, effect_digest) = match observer.observe(&action.consequence_id) {
            Observation::Effect(d) if d == result.result_digest => ("EFFECT_VERIFIED", d),
            Observation::Effect(d) => ("EFFECT_MISMATCH", d),
            Observation::NoEffect => ("EFFECT_MISMATCH", GENESIS.to_string()),
            Observation::Unknown => ("EFFECT_UNOBSERVED", GENESIS.to_string()),
        };
        let receipt = BrceReceipt {
            profile: PROFILE_ACTUATION.to_string(),
            run_id: self.run_id.clone(),
            subject: action.subject.clone(),
            request_digest: admitted.request_digest.clone(),
            route_digest: admitted.route_digest.clone(),
            admission_digest: admitted.admission_digest.clone(),
            construct_digest: action.construct_digest(),
            authority_grant_digest: grant.grant_digest(),
            consequence_id: action.consequence_id.clone(),
            attempt_id: attempt_id.to_string(),
            actuation: ActuationEvidence {
                executor_identity: executor,
                started_at,
                completed_at,
                result_class: result.result_class.clone(),
                result_digest: result.result_digest.clone(),
            },
            effect: EffectEvidence {
                executed: true,
                changed: result.changed,
                effect_digest: effect_digest.clone(),
            },
            verification: VerificationEvidence {
                verifier_identity: "affidavit::brce::Observer".to_string(),
                verdict: verdict.to_string(),
                evidence_digest: digest(&(&action.consequence_id, &effect_digest)),
            },
            replay: self.replay_identity(),
            canonicalization: CANONICALIZATION.to_string(),
            previous_receipt: self.ledger.last_receipt_digest(),
            receipt_digest: String::new(),
        }
        .seal();
        self.ledger.append(Entry::Receipted {
            receipt: receipt.clone(),
        })?;
        Ok(receipt)
    }

    /// Full DO: prepare → execute → receipt.
    ///
    /// # Errors
    /// Any refusal from the three steps.
    #[allow(clippy::too_many_arguments)]
    pub fn actuate(
        &mut self,
        admitted: &Admitted,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        actuator: &mut (impl Actuator + Observer),
        now: u64,
    ) -> Result<BrceReceipt, BrceError> {
        self.prepare(action, grant, attempt_id, now)?;
        self.execute(action, attempt_id, actuator, now + 1)?;
        self.receipt(admitted, action, grant, attempt_id, actuator)
    }

    fn replay_identity(&self) -> ReplayIdentity {
        ReplayIdentity {
            command_or_entrypoint: "affidavit::brce::replay".to_string(),
            tool_identity: "affidavit".to_string(),
            tool_version: self.tool_version.clone(),
        }
    }

    fn receipted(&self, consequence_id: &str) -> bool {
        self.ledger.records().iter().any(|r| {
            matches!(&r.entry, Entry::Receipted { receipt } if receipt.consequence_id == consequence_id)
        })
    }

    fn pending(&self, consequence_id: &str) -> bool {
        pending_consequences(&self.ledger).contains(consequence_id)
    }

    /// Crash-window reconciliation for every `PREPARED` consequence that has no
    /// receipt and no reconciliation yet.
    ///
    /// * `Done` recorded and the effect observed with the recorded digest →
    ///   `EFFECT_CONFIRMED` and a `brce-reconciliation/v1` receipt is persisted.
    /// * No effect observed → `NO_EFFECT_CONFIRMED` (no receipt, nothing executed).
    /// * Effect observed but no `Done` (crash between actuator and ledger write)
    ///   whose digest matches the construct payload → `EFFECT_CONFIRMED` + receipt.
    /// * Anything else → `EXECUTION_UNKNOWN` / `BLOCKED_RECONCILIATION`; the court
    ///   refuses such a ledger.
    ///
    /// # Errors
    /// I/O failure while appending.
    pub fn reconcile(
        &mut self,
        observer: &dyn Observer,
    ) -> Result<Vec<(String, ReconciliationVerdict)>, BrceError> {
        let mut out = Vec::new();
        for cid in pending_consequences(&self.ledger) {
            let Some((prep_digest, attempt_id, construct_digest, grant, started_at)) = self
                .ledger
                .records()
                .iter()
                .rev()
                .find_map(|r| match &r.entry {
                    Entry::Prepared {
                        consequence_id,
                        attempt_id,
                        construct_digest,
                        grant,
                        at,
                    } if *consequence_id == cid => Some((
                        r.digest.clone(),
                        attempt_id.clone(),
                        construct_digest.clone(),
                        grant.clone(),
                        *at,
                    )),
                    _ => None,
                })
            else {
                continue;
            };
            let action = self.ledger.find(|e| match e {
                Entry::Constructed {
                    action,
                    construct_digest: d,
                } if *d == construct_digest => Some(action.clone()),
                _ => None,
            });
            let done = self.ledger.find(|e| match e {
                Entry::Done {
                    consequence_id,
                    attempt_id: a,
                    result,
                    executor_identity,
                    at,
                    ..
                } if *consequence_id == cid && *a == attempt_id => {
                    Some((result.clone(), executor_identity.clone(), *at))
                }
                _ => None,
            });
            let expected = match (&done, &action) {
                (Some((res, _, _)), _) => Some(res.result_digest.clone()),
                (None, Some(a)) => a
                    .parameters
                    .get("content")
                    .map(|c| blake3::hash(c.as_bytes()).to_hex().to_string()),
                (None, None) => None,
            };
            let observation = observer.observe(&cid);
            let verdict = match (&observation, &expected) {
                (Observation::Effect(d), Some(e)) if d == e => {
                    ReconciliationVerdict::EffectConfirmed
                }
                (Observation::Effect(_), _) => ReconciliationVerdict::BlockedReconciliation,
                (Observation::NoEffect, _) if done.is_none() => {
                    ReconciliationVerdict::NoEffectConfirmed
                }
                (Observation::NoEffect, _) => ReconciliationVerdict::BlockedReconciliation,
                (Observation::Unknown, _) => ReconciliationVerdict::ExecutionUnknown,
            };
            let evidence_digest = digest(&(&cid, &observation));
            self.ledger.append(Entry::Reconciled {
                consequence_id: cid.clone(),
                prepared_record_digest: prep_digest,
                verdict,
                evidence_digest: evidence_digest.clone(),
            })?;
            if verdict == ReconciliationVerdict::EffectConfirmed {
                if let (Some(action), Some(effect_digest)) = (&action, &expected) {
                    let request = self.ledger.find(|e| match e {
                        Entry::Parsed { request_digest, .. }
                            if *request_digest == action.request_digest =>
                        {
                            Some(request_digest.clone())
                        }
                        _ => None,
                    });
                    let route_digest = self.ledger.find(|e| match e {
                        Entry::Routed {
                            request_digest,
                            route_digest,
                            ..
                        } if *request_digest == action.request_digest => Some(route_digest.clone()),
                        _ => None,
                    });
                    let admission_digest = self.ledger.find(|e| match e {
                        Entry::Admitted {
                            request_digest,
                            admission_digest,
                            ..
                        } if *request_digest == action.request_digest => {
                            Some(admission_digest.clone())
                        }
                        _ => None,
                    });
                    let (executor, completed_at, changed, result_class) = match &done {
                        Some((res, ex, at)) => {
                            (ex.clone(), *at, res.changed, res.result_class.clone())
                        }
                        None => (
                            "UNRECORDED(crash-before-done)".to_string(),
                            started_at,
                            true,
                            "SUCCESS".to_string(),
                        ),
                    };
                    let receipt = BrceReceipt {
                        profile: PROFILE_RECONCILIATION.to_string(),
                        run_id: self.run_id.clone(),
                        subject: action.subject.clone(),
                        request_digest: request.unwrap_or_default(),
                        route_digest: route_digest.unwrap_or_default(),
                        admission_digest: admission_digest.unwrap_or_default(),
                        construct_digest: construct_digest.clone(),
                        authority_grant_digest: grant.grant_digest(),
                        consequence_id: cid.clone(),
                        attempt_id: attempt_id.clone(),
                        actuation: ActuationEvidence {
                            executor_identity: executor,
                            started_at,
                            completed_at,
                            result_class,
                            result_digest: effect_digest.clone(),
                        },
                        effect: EffectEvidence {
                            executed: true,
                            changed,
                            effect_digest: effect_digest.clone(),
                        },
                        verification: VerificationEvidence {
                            verifier_identity: "affidavit::brce::reconcile".to_string(),
                            verdict: "EFFECT_CONFIRMED".to_string(),
                            evidence_digest,
                        },
                        replay: self.replay_identity(),
                        canonicalization: CANONICALIZATION.to_string(),
                        previous_receipt: self.ledger.last_receipt_digest(),
                        receipt_digest: String::new(),
                    }
                    .seal();
                    self.ledger.append(Entry::Receipted { receipt })?;
                }
            }
            out.push((cid, verdict));
        }
        Ok(out)
    }
}

/// Returns `Some(reason)` when `grant` does not authorize `action` at `now`.
fn authority_defect(
    action: &ConstructedAction,
    construct_digest: &str,
    grant: &AuthorityGrant,
    now: u64,
) -> Option<String> {
    if grant.subject != action.subject {
        return Some(format!(
            "SUBJECT_MISMATCH:{}!={}",
            grant.subject, action.subject
        ));
    }
    if grant.operation != action.operation {
        return Some(format!("OPERATION_MISMATCH:{}", grant.operation));
    }
    if grant.target != action.target {
        return Some(format!("TARGET_MISMATCH:{}", grant.target));
    }
    if grant.construct_digest != construct_digest {
        return Some("CONSTRUCT_DIGEST_MISMATCH".to_string());
    }
    if now >= grant.expires_at {
        return Some(format!("GRANT_EXPIRED:{}", grant.grant_id));
    }
    if grant.maximum_uses == 0 {
        return Some(format!("GRANT_EXHAUSTED:{}", grant.grant_id));
    }
    None
}

/// Consequences with a `PREPARED` intent but neither a receipt nor a reconciliation.
#[must_use]
pub fn pending_consequences(ledger: &BrceLedger) -> BTreeSet<String> {
    let mut prepared = BTreeSet::new();
    let mut closed = BTreeSet::new();
    for r in ledger.records() {
        match &r.entry {
            Entry::Prepared { consequence_id, .. } => {
                prepared.insert(consequence_id.clone());
                closed.remove(consequence_id);
            }
            Entry::Receipted { receipt } => {
                closed.insert(receipt.consequence_id.clone());
            }
            Entry::Reconciled {
                consequence_id,
                verdict,
                ..
            } if *verdict == ReconciliationVerdict::NoEffectConfirmed => {
                closed.insert(consequence_id.clone());
            }
            _ => {}
        }
    }
    prepared.difference(&closed).cloned().collect()
}

/// Court rules; each has a refusing mutant in `tests/brce_ledger.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Rule {
    /// Every record's `prev`/`seq`/`digest` recomputes (append-only, untampered).
    ChainIntegrity,
    /// Every external consequence and every `Done` has a receipt.
    ZeroUnreceiptedActuation,
    /// Every `Done` follows a `PREPARED` whose grant binds subject/operation/target/
    /// construct digest, is unexpired at prepare time, and has uses left.
    DoRequiresAuthority,
    /// Every `Constructed` follows an `Admitted` outcome for its request.
    ConstructRequiresAdmission,
    /// Every receipt digest recomputes and binds the ledger's construct/grant/subject.
    ReceiptDigestValid,
    /// At most one changing effect per consequence id.
    AtMostOnceConsequence,
    /// No `PREPARED` consequence is left unreconciled (crash window closed).
    CrashWindowReconciled,
    /// `EXECUTION_UNKNOWN` is never followed by an executed receipt.
    UnknownNotPromoted,
}

impl Rule {
    /// All rules in evaluation order.
    pub const ALL: [Rule; 8] = [
        Rule::ChainIntegrity,
        Rule::ZeroUnreceiptedActuation,
        Rule::DoRequiresAuthority,
        Rule::ConstructRequiresAdmission,
        Rule::ReceiptDigestValid,
        Rule::AtMostOnceConsequence,
        Rule::CrashWindowReconciled,
        Rule::UnknownNotPromoted,
    ];
}

/// One court refusal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourtRefusal {
    /// Rule that refused.
    pub rule: Rule,
    /// Record sequence (if record-local).
    pub seq: Option<u64>,
    /// Detail.
    pub detail: String,
}

/// Court verdict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourtVerdict {
    /// `ADMITTED` when no refusals, else `REFUSED`.
    pub standing: String,
    /// Rules evaluated.
    pub rules: Vec<Rule>,
    /// Refusals.
    pub refusals: Vec<CourtRefusal>,
    /// Record count.
    pub records: usize,
    /// Receipt count.
    pub receipts: usize,
    /// External consequences observed.
    pub consequences_observed: usize,
    /// Ledger head digest.
    pub ledger_head: String,
    /// Consequence-free replay digest (see [`replay_digest`]).
    pub replay_digest: String,
}

impl CourtVerdict {
    /// True iff no rule refused.
    #[must_use]
    pub fn admitted(&self) -> bool {
        self.refusals.is_empty()
    }

    /// Rules that refused (deduplicated).
    #[must_use]
    pub fn refused_rules(&self) -> BTreeSet<Rule> {
        self.refusals.iter().map(|r| r.rule).collect()
    }
}

/// Consequence-free replay digest: recomputes every request/route/admission/
/// construct/receipt digest from the ledger content (never calling an actuator) and
/// hashes the recomputed evidence vector. Two ledgers with equal replay digests
/// carry the same evidence.
#[must_use]
pub fn replay_digest(ledger: &BrceLedger) -> String {
    let vector: Vec<(String, String)> = ledger
        .records()
        .iter()
        .map(|r| match &r.entry {
            Entry::Parsed { request, .. } => ("parse".to_string(), digest(request)),
            Entry::Routed { route, .. } => ("route".to_string(), digest(route)),
            Entry::Admitted { admission, .. } => ("admit".to_string(), digest(admission)),
            Entry::Constructed { action, .. } => {
                ("construct".to_string(), action.construct_digest())
            }
            Entry::Prepared { grant, .. } => ("prepare".to_string(), grant.grant_digest()),
            Entry::Done { result, .. } => ("do".to_string(), digest(result)),
            Entry::Receipted { receipt } => ("receipt".to_string(), receipt.compute_digest()),
            Entry::Reconciled { verdict, .. } => ("reconcile".to_string(), digest(verdict)),
            Entry::Refusal { reason, .. } => ("refuse".to_string(), digest(reason)),
        })
        .collect();
    digest(&(PROFILE_REPLAY, vector))
}

/// Judge `ledger` against the observed external `world`.
#[must_use]
pub fn court(ledger: &BrceLedger, world: &dyn Observer) -> CourtVerdict {
    let recs = ledger.records();
    let mut refusals = Vec::new();
    let mut refuse = |rule: Rule, seq: Option<u64>, detail: String| {
        refusals.push(CourtRefusal { rule, seq, detail });
    };

    // CHAIN_INTEGRITY
    let mut prev = GENESIS.to_string();
    for (i, r) in recs.iter().enumerate() {
        if r.seq != i as u64
            || r.prev != prev
            || r.digest != record_digest(r.seq, &r.prev, &r.entry)
        {
            refuse(
                Rule::ChainIntegrity,
                Some(r.seq),
                format!("record {i} does not recompute"),
            );
        }
        prev = r.digest.clone();
    }

    let mut admitted_requests = BTreeSet::new();
    let mut constructs: BTreeMap<String, ConstructedAction> = BTreeMap::new();
    let mut prepared: BTreeMap<(String, String), (String, AuthorityGrant, u64)> = BTreeMap::new();
    let mut grant_uses: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut done: BTreeMap<String, Vec<(u64, bool)>> = BTreeMap::new();
    let mut receipted: BTreeSet<String> = BTreeSet::new();
    let mut unknown: BTreeSet<String> = BTreeSet::new();
    let mut receipts = 0usize;
    let mut prev_receipt = GENESIS.to_string();

    for r in recs {
        match &r.entry {
            Entry::Admitted {
                request_digest,
                admission: Admission::Admitted,
                ..
            } => {
                admitted_requests.insert(request_digest.clone());
            }
            Entry::Constructed {
                action,
                construct_digest,
            } => {
                if !admitted_requests.contains(&action.request_digest) {
                    refuse(
                        Rule::ConstructRequiresAdmission,
                        Some(r.seq),
                        format!(
                            "construct for un-admitted request {}",
                            action.request_digest
                        ),
                    );
                }
                constructs.insert(construct_digest.clone(), action.clone());
            }
            Entry::Prepared {
                consequence_id,
                attempt_id,
                construct_digest,
                grant,
                at,
            } => {
                let uses = grant_uses.entry(grant.grant_digest()).or_default();
                uses.insert(consequence_id.clone());
                if uses.len() as u32 > grant.maximum_uses {
                    refuse(
                        Rule::DoRequiresAuthority,
                        Some(r.seq),
                        format!("GRANT_EXHAUSTED:{}", grant.grant_id),
                    );
                }
                match constructs.get(construct_digest) {
                    Some(action) => {
                        if let Some(defect) = authority_defect(action, construct_digest, grant, *at)
                        {
                            refuse(Rule::DoRequiresAuthority, Some(r.seq), defect);
                        }
                    }
                    None => refuse(
                        Rule::DoRequiresAuthority,
                        Some(r.seq),
                        format!("PREPARED for unknown construct {construct_digest}"),
                    ),
                }
                prepared.insert(
                    (consequence_id.clone(), attempt_id.clone()),
                    (construct_digest.clone(), grant.clone(), *at),
                );
            }
            Entry::Done {
                consequence_id,
                attempt_id,
                construct_digest,
                result,
                ..
            } => {
                match prepared.get(&(consequence_id.clone(), attempt_id.clone())) {
                    Some((d, _, _)) if d == construct_digest => {}
                    Some(_) => refuse(
                        Rule::DoRequiresAuthority,
                        Some(r.seq),
                        format!("DO executed a construct other than the prepared one for {consequence_id}"),
                    ),
                    None => refuse(
                        Rule::DoRequiresAuthority,
                        Some(r.seq),
                        format!("DO without PREPARED authority for {consequence_id}"),
                    ),
                }
                done.entry(consequence_id.clone())
                    .or_default()
                    .push((r.seq, result.changed));
            }
            Entry::Receipted { receipt } => {
                receipts += 1;
                if receipt.receipt_digest != receipt.compute_digest() {
                    refuse(
                        Rule::ReceiptDigestValid,
                        Some(r.seq),
                        format!("receipt digest mismatch for {}", receipt.consequence_id),
                    );
                }
                if receipt.previous_receipt != prev_receipt {
                    refuse(
                        Rule::ReceiptDigestValid,
                        Some(r.seq),
                        "previous_receipt does not chain".to_string(),
                    );
                }
                prev_receipt = receipt.receipt_digest.clone();
                match prepared.get(&(receipt.consequence_id.clone(), receipt.attempt_id.clone())) {
                    Some((d, grant, _)) => {
                        if *d != receipt.construct_digest
                            || grant.grant_digest() != receipt.authority_grant_digest
                        {
                            refuse(
                                Rule::ReceiptDigestValid,
                                Some(r.seq),
                                format!(
                                    "receipt binds a different construct/grant for {}",
                                    receipt.consequence_id
                                ),
                            );
                        }
                        if constructs
                            .get(d)
                            .is_some_and(|a| a.subject != receipt.subject)
                        {
                            refuse(
                                Rule::ReceiptDigestValid,
                                Some(r.seq),
                                format!(
                                    "receipt subject substituted for {}",
                                    receipt.consequence_id
                                ),
                            );
                        }
                    }
                    None => refuse(
                        Rule::ReceiptDigestValid,
                        Some(r.seq),
                        format!(
                            "receipt for unprepared consequence {}",
                            receipt.consequence_id
                        ),
                    ),
                }
                if unknown.contains(&receipt.consequence_id) && receipt.effect.executed {
                    refuse(
                        Rule::UnknownNotPromoted,
                        Some(r.seq),
                        format!(
                            "EXECUTION_UNKNOWN promoted to executed for {}",
                            receipt.consequence_id
                        ),
                    );
                }
                receipted.insert(receipt.consequence_id.clone());
            }
            Entry::Reconciled {
                consequence_id,
                verdict,
                ..
            } => {
                if matches!(
                    verdict,
                    ReconciliationVerdict::ExecutionUnknown
                        | ReconciliationVerdict::BlockedReconciliation
                ) {
                    unknown.insert(consequence_id.clone());
                }
            }
            _ => {}
        }
    }

    // ZERO_UNRECEIPTED_ACTUATION: ledger-side and world-side.
    for (cid, events) in &done {
        if !receipted.contains(cid) {
            refuse(
                Rule::ZeroUnreceiptedActuation,
                events.first().map(|e| e.0),
                format!("DO for {cid} has no receipt"),
            );
        }
    }
    let world_consequences = world.consequences();
    for cid in &world_consequences {
        if !receipted.contains(cid) {
            refuse(
                Rule::ZeroUnreceiptedActuation,
                None,
                format!("external consequence {cid} has no receipt"),
            );
        }
    }

    // AT_MOST_ONCE_CONSEQUENCE
    for (cid, events) in &done {
        let changes = events.iter().filter(|e| e.1).count();
        if changes > 1 {
            refuse(
                Rule::AtMostOnceConsequence,
                events.get(1).map(|e| e.0),
                format!("{cid} changed external state {changes} times"),
            );
        }
    }

    // CRASH_WINDOW_RECONCILED
    let pending = pending_consequences(ledger);
    for cid in &pending {
        refuse(
            Rule::CrashWindowReconciled,
            None,
            format!(
                "{cid} PREPARED without receipt or reconciliation{}",
                if unknown.contains(cid) {
                    " (EXECUTION_UNKNOWN)"
                } else {
                    ""
                }
            ),
        );
    }

    refusals.sort_by_key(|r| (r.rule, r.seq));
    CourtVerdict {
        standing: if refusals.is_empty() {
            "ADMITTED"
        } else {
            "REFUSED"
        }
        .to_string(),
        rules: Rule::ALL.to_vec(),
        refusals,
        records: recs.len(),
        receipts,
        consequences_observed: world_consequences.len(),
        ledger_head: ledger.head(),
        replay_digest: replay_digest(ledger),
    }
}

impl BrceLedger {
    /// In-memory ledger holding `records` verbatim (no re-chaining). Used to judge
    /// ledgers obtained elsewhere; the [`court`] decides whether they are lawful.
    #[must_use]
    pub fn from_records(records: Vec<LedgerRecord>) -> Self {
        Self {
            records,
            path: None,
        }
    }

    /// In-memory ledger built by appending `entries` in order (a correctly chained
    /// ledger whose *content* may still be unlawful).
    #[must_use]
    pub fn from_entries(entries: impl IntoIterator<Item = Entry>) -> Self {
        let mut l = Self::new();
        for e in entries {
            let _ = l.append(e);
        }
        l
    }

    /// Entries in order.
    #[must_use]
    pub fn entries(&self) -> Vec<Entry> {
        self.records.iter().map(|r| r.entry.clone()).collect()
    }
}

/// Point-in-time snapshot of an [`Observer`]'s world, extendable with extra effects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaticWorld {
    /// consequence id → observed effect digest.
    pub effects: BTreeMap<String, String>,
}

impl StaticWorld {
    /// Snapshot every consequence `world` reports.
    #[must_use]
    pub fn snapshot(world: &dyn Observer) -> Self {
        let effects = world
            .consequences()
            .into_iter()
            .filter_map(|c| match world.observe(&c) {
                Observation::Effect(d) => Some((c, d)),
                _ => None,
            })
            .collect();
        Self { effects }
    }
}

impl Observer for StaticWorld {
    fn observe(&self, consequence_id: &str) -> Observation {
        self.effects
            .get(consequence_id)
            .map_or(Observation::NoEffect, |d| Observation::Effect(d.clone()))
    }

    fn consequences(&self) -> Vec<String> {
        self.effects.keys().cloned().collect()
    }
}

/// Outcome of one refusing mutant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutantOutcome {
    /// Rule the mutant targets.
    pub rule: Rule,
    /// What was mutated.
    pub mutation: String,
    /// Court standing on the mutant.
    pub standing: String,
    /// Rules that refused.
    pub refused_rules: Vec<Rule>,
    /// True iff the targeted rule refused.
    pub killed: bool,
}

/// Apply one unlawful mutation per [`Rule`] to a lawful `base` ledger + `world` and
/// run the [`court`] on each. A rule whose mutant is not killed carries no bits.
///
/// Requires `base` to contain at least one receipted consequence.
#[must_use]
pub fn mutant_suite(base: &BrceLedger, world: &dyn Observer) -> Vec<MutantOutcome> {
    let world = StaticWorld::snapshot(world);
    let entries = base.entries();
    let receipt = entries.iter().find_map(|e| match e {
        Entry::Receipted { receipt } if receipt.profile == PROFILE_ACTUATION => {
            Some(receipt.clone())
        }
        _ => None,
    });
    let Some(receipt) = receipt else {
        return Vec::new();
    };
    let prepared = entries
        .iter()
        .find_map(|e| match e {
            Entry::Prepared {
                consequence_id,
                construct_digest,
                grant,
                at,
                ..
            } if *consequence_id == receipt.consequence_id => {
                Some((construct_digest.clone(), grant.clone(), *at))
            }
            _ => None,
        })
        .unwrap_or_else(|| (String::new(), receipt_grant_placeholder(), 0));
    let (construct_digest, grant, at) = prepared;
    let cid = receipt.consequence_id.clone();
    let mut out = Vec::new();
    let mut judge = |rule: Rule, mutation: &str, ledger: &BrceLedger, w: &StaticWorld| {
        let v = court(ledger, w);
        let refused = v.refused_rules();
        out.push(MutantOutcome {
            rule,
            mutation: mutation.to_string(),
            standing: v.standing.clone(),
            killed: refused.contains(&rule),
            refused_rules: refused.into_iter().collect(),
        });
    };

    // CHAIN_INTEGRITY: edit a persisted record in place without re-chaining.
    let mut recs = base.records().to_vec();
    if let Some(r) = recs
        .iter_mut()
        .find(|r| matches!(r.entry, Entry::Done { .. }))
    {
        if let Entry::Done { result, .. } = &mut r.entry {
            result.changed = !result.changed;
        }
    }
    judge(
        Rule::ChainIntegrity,
        "flip Done.result.changed in place, chain not recomputed",
        &BrceLedger::from_records(recs),
        &world,
    );

    // ZERO_UNRECEIPTED_ACTUATION: an external consequence the ledger never receipted.
    let mut rogue = world.clone();
    rogue
        .effects
        .insert("rogue-unreceipted".to_string(), digest(&"rogue"));
    judge(
        Rule::ZeroUnreceiptedActuation,
        "external effect 'rogue-unreceipted' written outside the pipeline",
        base,
        &rogue,
    );

    // DO_REQUIRES_AUTHORITY: DO recorded with no PREPARED authority for its attempt.
    let mut e = entries.clone();
    e.push(Entry::Done {
        consequence_id: cid.clone(),
        attempt_id: "mutant-unauthorized".to_string(),
        construct_digest: construct_digest.clone(),
        result: ActuationResult {
            result_class: "SUCCESS".to_string(),
            result_digest: receipt.actuation.result_digest.clone(),
            changed: false,
        },
        executor_identity: "mutant".to_string(),
        at: at + 100,
    });
    judge(
        Rule::DoRequiresAuthority,
        "Done appended for an attempt that never passed PREPARED/authority",
        &BrceLedger::from_entries(e),
        &world,
    );

    // CONSTRUCT_REQUIRES_ADMISSION: admission outcome rewritten to Refused.
    let e: Vec<Entry> = entries
        .iter()
        .cloned()
        .map(|en| match en {
            Entry::Admitted {
                request_digest,
                admission: Admission::Admitted,
                ..
            } => {
                let admission = Admission::Refused("mutant".to_string());
                Entry::Admitted {
                    request_digest,
                    admission_digest: digest(&admission),
                    admission,
                }
            }
            other => other,
        })
        .collect();
    judge(
        Rule::ConstructRequiresAdmission,
        "every Admitted outcome rewritten to Refused, construct kept",
        &BrceLedger::from_entries(e),
        &world,
    );

    // RECEIPT_DIGEST_VALID: receipt body edited, ledger re-chained.
    let e: Vec<Entry> = entries
        .iter()
        .cloned()
        .map(|en| match en {
            Entry::Receipted { mut receipt } if receipt.consequence_id == cid => {
                receipt.effect.changed = !receipt.effect.changed;
                Entry::Receipted { receipt }
            }
            other => other,
        })
        .collect();
    judge(
        Rule::ReceiptDigestValid,
        "receipt.effect.changed flipped after sealing; ledger re-chained",
        &BrceLedger::from_entries(e),
        &world,
    );

    // AT_MOST_ONCE_CONSEQUENCE: second changing effect for the same consequence.
    let mut e = entries.clone();
    e.push(Entry::Prepared {
        consequence_id: cid.clone(),
        attempt_id: "mutant-dup".to_string(),
        construct_digest: construct_digest.clone(),
        grant: grant.clone(),
        at,
    });
    e.push(Entry::Done {
        consequence_id: cid.clone(),
        attempt_id: "mutant-dup".to_string(),
        construct_digest: construct_digest.clone(),
        result: ActuationResult {
            result_class: "SUCCESS".to_string(),
            result_digest: receipt.actuation.result_digest.clone(),
            changed: true,
        },
        executor_identity: "mutant".to_string(),
        at: at + 1,
    });
    judge(
        Rule::AtMostOnceConsequence,
        "second PREPARED+Done(changed=true) for an already-receipted consequence",
        &BrceLedger::from_entries(e),
        &world,
    );

    // CRASH_WINDOW_RECONCILED: PREPARED intent left dangling (crash, never reconciled).
    let mut e = entries.clone();
    e.push(Entry::Prepared {
        consequence_id: cid.clone(),
        attempt_id: "mutant-crash".to_string(),
        construct_digest: construct_digest.clone(),
        grant: grant.clone(),
        at,
    });
    judge(
        Rule::CrashWindowReconciled,
        "PREPARED appended with no Done/receipt/reconciliation",
        &BrceLedger::from_entries(e),
        &world,
    );

    // UNKNOWN_NOT_PROMOTED: EXECUTION_UNKNOWN followed by an executed receipt.
    let mut e = entries.clone();
    e.push(Entry::Prepared {
        consequence_id: cid.clone(),
        attempt_id: "mutant-unknown".to_string(),
        construct_digest,
        grant,
        at,
    });
    e.push(Entry::Reconciled {
        consequence_id: cid.clone(),
        prepared_record_digest: GENESIS.to_string(),
        verdict: ReconciliationVerdict::ExecutionUnknown,
        evidence_digest: digest(&Observation::Unknown),
    });
    let mut promoted = receipt.clone();
    promoted.attempt_id = "mutant-unknown".to_string();
    promoted.profile = PROFILE_RECONCILIATION.to_string();
    promoted.previous_receipt = BrceLedger::from_entries(entries.clone()).last_receipt_digest();
    e.push(Entry::Receipted {
        receipt: promoted.seal(),
    });
    judge(
        Rule::UnknownNotPromoted,
        "EXECUTION_UNKNOWN reconciliation followed by an executed=true receipt",
        &BrceLedger::from_entries(e),
        &world,
    );

    out
}

fn receipt_grant_placeholder() -> AuthorityGrant {
    AuthorityGrant {
        grant_id: String::new(),
        issuer: String::new(),
        subject: String::new(),
        operation: String::new(),
        target: String::new(),
        construct_digest: String::new(),
        expires_at: 0,
        maximum_uses: 0,
    }
}
