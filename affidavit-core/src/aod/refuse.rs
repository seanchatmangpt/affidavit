//! AOD boundary refusal codes `0xE501..=0xE50D` (AFFI-SPEC-26.11.19 §4.4).

/// Typed AOD refusal with stable machine codes on the `0xE5xx` band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AodRefusal {
    /// `0xE501` — input pointer/slice not 64-byte aligned (when alignment is demanded).
    UnalignedInput,
    /// `0xE502` — file shorter than the minimum envelope (shadow + PQC trailer).
    TruncatedFile,
    /// `0xE503` — header sentinel is not [`super::AOD_MAGIC`].
    InvalidSentinel,
    /// `0xE504` — header reserved word is non-zero.
    NonZeroReserved,
    /// `0xE505` — `trailer_offset` is not `file_len - 352 - PQC_MATERIAL`.
    InvalidTrailerOffset,
    /// `0xE506` — live gate: ontology hash mismatch.
    UnknownOntology,
    /// `0xE507` — live gate: buffer_count mismatch vs compiled ontology.
    BufferCountMismatch,
    /// `0xE508` — directory/data bounds or contiguous layout violated.
    BoundsOrSlack,
    /// `0xE509` — buffer byte_length is not the ontology's fixed stride × rows.
    FixedStrideViolation,
    /// `0xE50A` — non-zero padding bytes in a 64-byte pad region.
    NonZeroPadding,
    /// `0xE50B` — BLAKE3 payload digest mismatch.
    DigestMismatch,
    /// `0xE50C` — ML-DSA-65 PQC seal verification failed.
    PqcSealFailure,
    /// `0xE50D` — monoatomic lease spent, expired, or refused by the registry.
    LeaseSpentOrExpired,
}

impl AodRefusal {
    /// Stable machine code for diagnostics / CLI exit mapping.
    #[inline]
    pub const fn machine_code(self) -> u16 {
        match self {
            Self::UnalignedInput => 0xE501,
            Self::TruncatedFile => 0xE502,
            Self::InvalidSentinel => 0xE503,
            Self::NonZeroReserved => 0xE504,
            Self::InvalidTrailerOffset => 0xE505,
            Self::UnknownOntology => 0xE506,
            Self::BufferCountMismatch => 0xE507,
            Self::BoundsOrSlack => 0xE508,
            Self::FixedStrideViolation => 0xE509,
            Self::NonZeroPadding => 0xE50A,
            Self::DigestMismatch => 0xE50B,
            Self::PqcSealFailure => 0xE50C,
            Self::LeaseSpentOrExpired => 0xE50D,
        }
    }
}
