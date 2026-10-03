//! Cryptographic accumulators and inclusion witnesses.
//!
//! Provides sub-linear witness generation and verification over append-only
//! event sequences.

pub mod mmr;

pub use mmr::{
    bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError,
    MmrProof, MountainPeak,
};
