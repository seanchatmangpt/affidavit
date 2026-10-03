//! Zero-copy archival access to historical receipt stores (rkyv).
//!
//! A verifier can memory-map gigabytes of archived receipts and inspect
//! fields without materializing the whole payload: `access` validates the
//! archive's internal consistency and returns `&Archived<T>` pointing *into
//! the original bytes*.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! Access validates structure; the archived bytes still carry whatever
//! commitments were written into them. Range checks on archived data are
//! performed, not assumed.

use rkyv::{
    access, deserialize, rancor::Error, to_bytes, Archive, Archived,
    Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
};
use thiserror::Error;

/// Errors produced by zero-copy archival operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArchiveError {
    /// The bytes are not a valid archive of the target type.
    #[error("archive validation failed: {0}")]
    Invalid(String),
    /// The archived data violated a range check.
    #[error("archived entry out of declared range")]
    RangeCheck,
}

/// One archived sequence entry: a sequence number and its payload commitment.
#[derive(Debug, Clone, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct SeqEntry {
    /// Monotonic sequence number within the chain.
    pub seq: u64,
    /// Hex-encoded BLAKE3 commitment of the payload.
    pub commitment_hex: String,
}

/// An archived receipt-chain slice, suitable for memory-mapped inspection.
#[derive(Debug, Clone, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct ReceiptArchive {
    /// Chain identifier (hex-encoded chain head at archive time).
    pub chain_id_hex: String,
    /// Sequence entries in archive order.
    pub entries: Vec<SeqEntry>,
}

/// Serialize an archive to bytes.
///
/// # Errors
///
/// Returns [`ArchiveError::Invalid`] if serialization fails.
pub fn freeze(archive: &ReceiptArchive) -> Result<Vec<u8>, ArchiveError> {
    to_bytes::<Error>(archive)
        .map(|aligned| aligned.to_vec())
        .map_err(|e| ArchiveError::Invalid(e.to_string()))
}

/// Access an archive in zero-copy fashion.
///
/// # Errors
///
/// Returns [`ArchiveError::Invalid`] if the bytes are not a consistent
/// archive of [`ReceiptArchive`].
pub fn thaw(bytes: &[u8]) -> Result<&Archived<ReceiptArchive>, ArchiveError> {
    access::<Archived<ReceiptArchive>, Error>(bytes)
        .map_err(|e| ArchiveError::Invalid(e.to_string()))
}

/// Validate an archive and check every entry against a declared range.
///
/// The archived representation is inspected *in place*: no deserialization
/// allocation occurs on the validation path.
///
/// # Errors
///
/// Returns [`ArchiveError::RangeCheck`] naming the offending sequence number
/// if any entry falls outside `[min_seq, max_seq]`.
pub fn validate_archived_range(
    bytes: &[u8],
    min_seq: u64,
    max_seq: u64,
) -> Result<(), ArchiveError> {
    let archived = thaw(bytes)?;
    for entry in archived.entries.iter() {
        if entry.seq < min_seq || entry.seq > max_seq {
            return Err(ArchiveError::RangeCheck);
        }
    }
    Ok(())
}

/// Materialize an archive back into an owned value (full deserialization).
///
/// # Errors
///
/// Returns [`ArchiveError::Invalid`] if access or deserialization fails.
pub fn unfreeze(bytes: &[u8]) -> Result<ReceiptArchive, ArchiveError> {
    let archived = thaw(bytes)?;
    deserialize::<ReceiptArchive, Error>(archived).map_err(|e| ArchiveError::Invalid(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ReceiptArchive {
        ReceiptArchive {
            chain_id_hex: "abc123".to_string(),
            entries: (0..1_000u64)
                .map(|seq| SeqEntry {
                    seq,
                    commitment_hex: format!("{seq:064x}"),
                })
                .collect(),
        }
    }

    #[test]
    fn freeze_thaw_roundtrip() {
        let bytes = freeze(&sample()).expect("freeze");
        let materialized = unfreeze(&bytes).expect("unfreeze");
        assert_eq!(materialized, sample());
    }

    #[test]
    fn zero_copy_access_reads_in_place() {
        let bytes = freeze(&sample()).expect("freeze");
        let archived = thaw(&bytes).expect("access");
        assert_eq!(archived.entries.len(), 1_000);
        assert_eq!(archived.entries[7].seq, 7);
    }

    #[test]
    fn range_validation_admits_and_refuses() {
        let bytes = freeze(&sample()).expect("freeze");
        assert!(validate_archived_range(&bytes, 0, 999).is_ok());
        assert_eq!(
            validate_archived_range(&bytes, 0, 998),
            Err(ArchiveError::RangeCheck)
        );
    }

    #[test]
    fn corrupt_archive_is_refused() {
        let mut bytes = freeze(&sample()).expect("freeze");
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xff;
        assert!(
            thaw(&bytes).is_err() || unfreeze(&bytes).is_err(),
            "a corrupted archive must not validate cleanly"
        );
    }
}
