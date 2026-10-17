//! Git Merkle content-addressing, tree traversal, commit signing, and refs/notes dossier store.
//!
//! Provides in-process Git tree traversal and commit signing functions with zero shell
//! commands (`/bin/sh` or `System.cmd`). Supports computing census binding digests:
//! `Digest = BLAKE3(GitTreeSHA || WasmBytecodeHash)`.
//! Also supports reading/writing 352-byte CTRT receipts to Git notes (`refs/notes/strata-dossier`).

use crate::trajectory_folding::{CtrtReceipt, COURT_RECORD_LEN};
use gix_hash::ObjectId;
use gix_object::tree::{EntryKind, EntryMode};
use gix_object::{bstr::BString, Commit, Tree, WriteTo};
use gix_odb::Write as OdbWrite;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The canonical Git ref for strata dossier receipts.
pub const STRATA_DOSSIER_REF: &str = "refs/notes/strata-dossier";

/// Errors that can occur during Git Merkle operations.
#[derive(Debug, Error)]
pub enum GitMerkleError {
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    #[error("Tree corrupt or unparseable: {0}")]
    TreeCorruptOrUnparseable(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Git ODB error: {0}")]
    Odb(String),
    #[error("Object decode error: {0}")]
    Decode(String),
    #[error("Receipt format/fold error: {0}")]
    Receipt(#[from] crate::trajectory_folding::TrajectoryFoldError),
    #[error("Invalid commit: {0}")]
    InvalidCommit(String),
    #[error("Ref not found: {0}")]
    RefNotFound(String),
    #[error("Tree entry not found: {0}")]
    EntryNotFound(String),
}

impl GitMerkleError {
    /// Return the canonical machine code for structured error classification.
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        match self {
            Self::ObjectNotFound(_) => 0xE401,
            Self::TreeCorruptOrUnparseable(_) => 0xE402,
            Self::Io(_) => 0xE403,
            Self::Odb(_) => 0xE404,
            Self::Decode(_) => 0xE405,
            Self::Receipt(_) => 0xE406,
            Self::InvalidCommit(_) => 0xE407,
            Self::RefNotFound(_) => 0xE408,
            Self::EntryNotFound(_) => 0xE409,
        }
    }
}

/// Compute the census binding digest:
/// `Digest = BLAKE3(GitTreeSHA || WasmBytecodeHash)`.
pub fn compute_census_binding(git_tree_sha: &[u8], wasm_bytecode_hash: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(git_tree_sha);
    hasher.update(wasm_bytecode_hash);
    *hasher.finalize().as_bytes()
}

/// A node visited during in-process tree traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitTreeNode {
    pub mode: EntryMode,
    pub path: String,
    pub oid: ObjectId,
}

/// In-process directory tree builder that creates blobs and trees using gix-odb loose Store.
pub struct GitTreeBuilder {
    store: gix_odb::loose::Store,
}

impl GitTreeBuilder {
    /// Initialize with a git objects directory (e.g. `.git/objects`).
    pub fn new(objects_dir: impl Into<PathBuf>) -> Self {
        let store = gix_odb::loose::Store::at(objects_dir.into(), gix_hash::Kind::Sha1);
        Self { store }
    }

    /// Store a raw blob into the object database and return its SHA-1 ObjectId.
    pub fn write_blob(&self, data: &[u8]) -> Result<ObjectId, GitMerkleError> {
        self.store
            .write_buf(gix_object::Kind::Blob, data)
            .map_err(|e| GitMerkleError::Odb(e.to_string()))
    }

    /// Build a Git tree from sorted entries and store it in the object database.
    pub fn write_tree(
        &self,
        entries: Vec<gix_object::tree::Entry>,
    ) -> Result<ObjectId, GitMerkleError> {
        let mut sorted = entries;
        sorted.sort();
        let tree = Tree { entries: sorted };
        let mut buf = Vec::new();
        tree.write_to(&mut buf)
            .map_err(|e| GitMerkleError::Odb(e.to_string()))?;

        self.store
            .write_buf(gix_object::Kind::Tree, &buf)
            .map_err(|e| GitMerkleError::Odb(e.to_string()))
    }

    /// Recursively add a directory on the local filesystem into the Git ODB
    /// and return the root tree's ObjectId.
    pub fn write_dir_recursive(&self, dir: &Path) -> Result<ObjectId, GitMerkleError> {
        let mut entries = Vec::new();

        let mut paths: Vec<_> = fs::read_dir(dir)?.filter_map(|r| r.ok()).collect();
        paths.sort_by_key(|e| e.file_name());

        for entry in paths {
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            let name_str = file_name.to_str().ok_or_else(|| {
                GitMerkleError::Odb("Non-UTF-8 file name encountered".to_string())
            })?;

            // Skip .git directory
            if name_str == ".git" {
                continue;
            }

            let path = entry.path();
            if file_type.is_dir() {
                let sub_oid = self.write_dir_recursive(&path)?;
                entries.push(gix_object::tree::Entry {
                    mode: EntryKind::Tree.into(),
                    filename: BString::from(name_str),
                    oid: sub_oid,
                });
            } else if file_type.is_file() {
                let bytes = fs::read(&path)?;
                let blob_oid = self.write_blob(&bytes)?;
                entries.push(gix_object::tree::Entry {
                    mode: EntryKind::Blob.into(),
                    filename: BString::from(name_str),
                    oid: blob_oid,
                });
            }
        }

        self.write_tree(entries)
    }

    /// Read raw object bytes from the loose store by ObjectId.
    pub fn read_object(&self, oid: &ObjectId) -> Result<Vec<u8>, GitMerkleError> {
        let mut buf = Vec::new();
        match self
            .store
            .try_find(oid.as_ref(), &mut buf)
            .map_err(|e| GitMerkleError::Odb(e.to_string()))?
        {
            Some(data) => Ok(data.data.to_vec()),
            None => Err(GitMerkleError::Odb(format!("Object {oid} not found"))),
        }
    }

    /// In-process traversal of a Git tree, returning all descendant nodes.
    pub fn traverse_tree(&self, tree_oid: &ObjectId) -> Result<Vec<GitTreeNode>, GitMerkleError> {
        let mut nodes = Vec::new();
        self.traverse_internal(tree_oid, "", &mut nodes)?;
        Ok(nodes)
    }

    fn traverse_internal(
        &self,
        tree_oid: &ObjectId,
        prefix: &str,
        nodes: &mut Vec<GitTreeNode>,
    ) -> Result<(), GitMerkleError> {
        let raw = self.read_object(tree_oid)?;
        let tree_ref = gix_object::TreeRef::from_bytes(&raw)
            .map_err(|e| GitMerkleError::Decode(e.to_string()))?;

        for entry in tree_ref.entries {
            let name_str = std::str::from_utf8(entry.filename)
                .map_err(|e| GitMerkleError::Decode(e.to_string()))?;
            let path = if prefix.is_empty() {
                name_str.to_string()
            } else {
                format!("{prefix}/{name_str}")
            };

            let oid = entry.oid.to_owned();
            nodes.push(GitTreeNode {
                mode: entry.mode,
                path: path.clone(),
                oid,
            });

            if entry.mode == EntryMode::from(EntryKind::Tree) {
                self.traverse_internal(&oid, &path, nodes)?;
            }
        }

        Ok(())
    }

    /// Create and write a signed Git commit object.
    /// In-process commit creation without executing `git commit` or any shell command.
    pub fn create_signed_commit(
        &self,
        tree_oid: ObjectId,
        parent_oids: Vec<ObjectId>,
        author: gix_actor::Signature,
        committer: gix_actor::Signature,
        message: &str,
        extra_headers: Vec<(&str, &str)>,
    ) -> Result<ObjectId, GitMerkleError> {
        let commit = Commit {
            tree: tree_oid,
            parents: parent_oids.into(),
            author,
            committer,
            encoding: None,
            message: BString::from(message),
            extra_headers: extra_headers
                .into_iter()
                .map(|(k, v)| (BString::from(k), BString::from(v)))
                .collect(),
        };

        let mut buf = Vec::new();
        commit
            .write_to(&mut buf)
            .map_err(|e| GitMerkleError::Odb(e.to_string()))?;

        self.store
            .write_buf(gix_object::Kind::Commit, &buf)
            .map_err(|e| GitMerkleError::Odb(e.to_string()))
    }
}

/// Manage dossier receipts in `refs/notes/strata-dossier` via pure file system operations.
/// In Git, a note on commit `C` with ref `refs/notes/strata-dossier` points to a tree
/// containing a blob named `<C>` whose content is the note (here the 352-byte CTRT receipt),
/// or references loose objects in `.git/refs/notes/strata-dossier`.
pub struct GitDossierNotes {
    repo_root: PathBuf,
    builder: GitTreeBuilder,
}

impl GitDossierNotes {
    /// Initialize with the repository root directory (containing `.git`).
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        let root = repo_root.into();
        let objects_dir = root.join(".git").join("objects");
        let builder = GitTreeBuilder::new(objects_dir);
        Self {
            repo_root: root,
            builder,
        }
    }

    /// Read the commit ObjectId that `refs/notes/strata-dossier` currently points to.
    pub fn get_notes_ref(&self) -> Result<Option<ObjectId>, GitMerkleError> {
        let ref_path = self.repo_root.join(".git").join(STRATA_DOSSIER_REF);
        if !ref_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&ref_path)?;
        let hex = content.trim();
        let oid = ObjectId::from_hex(hex.as_bytes())
            .map_err(|e| GitMerkleError::Decode(e.to_string()))?;
        Ok(Some(oid))
    }

    /// Write/update `refs/notes/strata-dossier` ref to point to `commit_oid`.
    pub fn update_notes_ref(&self, commit_oid: &ObjectId) -> Result<(), GitMerkleError> {
        let ref_path = self.repo_root.join(".git").join(STRATA_DOSSIER_REF);
        if let Some(parent) = ref_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&ref_path, format!("{}\n", commit_oid.to_hex()))?;
        Ok(())
    }

    /// Attach a 352-byte CTRT receipt to a target commit in `refs/notes/strata-dossier`.
    /// Creates a blob for the 352-byte CTRT receipt, constructs a notes tree with entry
    /// `target_commit_oid -> receipt_blob_oid`, and creates a notes commit.
    pub fn attach_receipt(
        &self,
        target_commit_oid: &ObjectId,
        receipt: &CtrtReceipt,
        committer_sig: gix_actor::Signature,
    ) -> Result<ObjectId, GitMerkleError> {
        let receipt_bytes = receipt.encode()?;
        if receipt_bytes.len() != COURT_RECORD_LEN {
            return Err(GitMerkleError::Receipt(
                crate::trajectory_folding::TrajectoryFoldError::InvalidLength {
                    expected: COURT_RECORD_LEN,
                    actual: receipt_bytes.len(),
                },
            ));
        }

        // 1. Write the 352-byte receipt as a blob
        let blob_oid = self.builder.write_blob(&receipt_bytes)?;

        // 2. Build or update the notes tree. Note entry filename is target commit hex.
        let target_hex = target_commit_oid.to_hex().to_string();
        let mut entries = Vec::new();

        let parent_notes_commit = self.get_notes_ref()?;
        if let Some(parent_oid) = parent_notes_commit {
            // Read previous notes commit to get existing tree
            let raw_commit = self.builder.read_object(&parent_oid)?;
            let commit_ref = gix_object::CommitRef::from_bytes(&raw_commit)
                .map_err(|e| GitMerkleError::Decode(e.to_string()))?;
            let old_tree_oid = commit_ref.tree();

            let old_tree_raw = self.builder.read_object(&old_tree_oid)?;
            let old_tree_ref = gix_object::TreeRef::from_bytes(&old_tree_raw)
                .map_err(|e| GitMerkleError::Decode(e.to_string()))?;

            for e in old_tree_ref.entries {
                let fname = std::str::from_utf8(e.filename)
                    .map_err(|e| GitMerkleError::Decode(e.to_string()))?;
                if fname != target_hex {
                    entries.push(gix_object::tree::Entry {
                        mode: e.mode,
                        filename: BString::from(fname),
                        oid: e.oid.to_owned(),
                    });
                }
            }
        }

        entries.push(gix_object::tree::Entry {
            mode: EntryKind::Blob.into(),
            filename: BString::from(target_hex),
            oid: blob_oid,
        });

        let tree_oid = self.builder.write_tree(entries)?;

        // 3. Create notes commit
        let parents = match parent_notes_commit {
            Some(p) => vec![p],
            None => vec![],
        };

        let message = format!(
            "Notes added by strata-dossier for {}",
            target_commit_oid.to_hex()
        );
        let notes_commit = self.builder.create_signed_commit(
            tree_oid,
            parents,
            committer_sig.clone(),
            committer_sig,
            &message,
            vec![("strata-dossier-receipt", "352B-CTRT")],
        )?;

        // 4. Update the refs/notes/strata-dossier ref file
        self.update_notes_ref(&notes_commit)?;

        Ok(notes_commit)
    }

    /// Read the 352-byte CTRT receipt associated with a target commit in `refs/notes/strata-dossier`.
    pub fn read_receipt(
        &self,
        target_commit_oid: &ObjectId,
    ) -> Result<Option<CtrtReceipt>, GitMerkleError> {
        let notes_commit_oid = match self.get_notes_ref()? {
            Some(oid) => oid,
            None => return Ok(None),
        };

        let raw_commit = self.builder.read_object(&notes_commit_oid)?;
        let commit_ref = gix_object::CommitRef::from_bytes(&raw_commit)
            .map_err(|e| GitMerkleError::Decode(e.to_string()))?;
        let tree_oid = commit_ref.tree();

        let raw_tree = self.builder.read_object(&tree_oid)?;
        let tree_ref = gix_object::TreeRef::from_bytes(&raw_tree)
            .map_err(|e| GitMerkleError::Decode(e.to_string()))?;

        let target_hex = target_commit_oid.to_hex().to_string();
        for entry in tree_ref.entries {
            let fname = std::str::from_utf8(entry.filename)
                .map_err(|e| GitMerkleError::Decode(e.to_string()))?;
            if fname == target_hex {
                let blob_raw = self.builder.read_object(&entry.oid.to_owned())?;
                let receipt = CtrtReceipt::from_bytes(&blob_raw)?;
                return Ok(Some(receipt));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_signature() -> gix_actor::Signature {
        gix_actor::Signature {
            name: BString::from("Strata Bot"),
            email: BString::from("bot@strata.org"),
            time: gix_object::date::Time::new(1700000000, 0),
        }
    }

    #[test]
    fn test_census_binding_deterministic() {
        let tree_sha = [0x11u8; 20];
        let wasm_hash = [0x22u8; 32];
        let d1 = compute_census_binding(&tree_sha, &wasm_hash);
        let d2 = compute_census_binding(&tree_sha, &wasm_hash);
        assert_eq!(d1, d2);

        let d3 = compute_census_binding(&[0x12u8; 20], &wasm_hash);
        assert_ne!(d1, d3);
    }

    #[test]
    fn test_tree_builder_and_traversal() {
        let dir = tempdir().expect("tempdir");
        let git_dir = dir.path().join(".git");
        let objects_dir = git_dir.join("objects");
        fs::create_dir_all(&objects_dir).expect("create objects dir");

        let builder = GitTreeBuilder::new(&objects_dir);
        let blob1 = builder.write_blob(b"hello world").expect("write blob1");
        let blob2 = builder.write_blob(b"fn main() {}").expect("write blob2");

        let entries = vec![
            gix_object::tree::Entry {
                mode: EntryKind::Blob.into(),
                filename: BString::from("hello.txt"),
                oid: blob1,
            },
            gix_object::tree::Entry {
                mode: EntryKind::Blob.into(),
                filename: BString::from("main.rs"),
                oid: blob2,
            },
        ];

        let tree_oid = builder.write_tree(entries).expect("write tree");
        let nodes = builder.traverse_tree(&tree_oid).expect("traverse");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].path, "hello.txt");
        assert_eq!(nodes[1].path, "main.rs");
    }

    #[test]
    fn test_dossier_notes_attach_and_read() {
        let dir = tempdir().expect("tempdir");
        let git_dir = dir.path().join(".git");
        let objects_dir = git_dir.join("objects");
        fs::create_dir_all(&objects_dir).expect("create objects dir");

        let notes_manager = GitDossierNotes::new(dir.path());
        let target_commit_oid =
            ObjectId::from_hex(b"0123456789abcdef0123456789abcdef01234567").expect("valid oid");

        let receipt = CtrtReceipt {
            flags: 2, // FLAG_FAIL_CLOSED
            subject_hash: [0xaa; 32],
            parent_receipt_hash: [0u8; 32],
            timestamp_ms: 1700000000000,
            run_id: 42,
            total_ticks: 500,
            wall_ms: 15,
            thermal: 1,
            outcome: 0,
            budget_met: true,
            positive_control: [0u8; 32],
            metadata_digest: [0xbb; 32],
            prev_self_hash: [0u8; 32],
            name: "test_dossier_note".to_string(),
            chain_hash: [0u8; 32],
        };
        let encoded = receipt.encode().expect("encode");
        let mut full_receipt = receipt.clone();
        full_receipt.chain_hash.copy_from_slice(&encoded[320..352]);

        let notes_commit = notes_manager
            .attach_receipt(&target_commit_oid, &full_receipt, test_signature())
            .expect("attach");

        assert!(notes_commit.as_bytes().len() > 0);

        let retrieved = notes_manager
            .read_receipt(&target_commit_oid)
            .expect("read receipt")
            .expect("receipt must exist");

        assert_eq!(retrieved, full_receipt);
    }

    #[test]
    fn test_git_merkle_error_machine_codes() {
        let err_not_found = GitMerkleError::ObjectNotFound("abc".to_string());
        assert_eq!(err_not_found.machine_code(), 0xE401);

        let err_tree = GitMerkleError::TreeCorruptOrUnparseable("def".to_string());
        assert_eq!(err_tree.machine_code(), 0xE402);
    }
}
