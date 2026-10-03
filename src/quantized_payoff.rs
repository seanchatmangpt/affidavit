//! QuantizedPayoff: the float-to-fixed quantization seam between process
//! mining scores and the CMCA metabolic allocator's Q16.16 payoff layout.
//!
//! `wasm4pm-compat` exports `DependencyMeasure<NUM, DEN>` — a compile-time
//! static fraction (`Require<{ NUM <= DEN }>`) with an `f64` projection —
//! while `bcinr-cmca` consumes `[[NonNegativeFixed; 2*Q]; N]` (Q16.16 bit
//! patterns). Those formats do not interoperate directly (verified ground
//! boundary, C4 checklist item 1). This module is the typed boundary.
//!
//! ## Quantization law (exact)
//!
//! - `bits = round(score * 65536)` clamped into `[0, 65535]`.
//! - `1.0` quantizes to `0xFFFF`: Q16.16 cannot represent 1.0 exactly, and
//!   `0xFFFF` (65535/65536) is the nearest representable value below it.
//! - `NaN`, `±inf`, `score < 0.0`, and `score > 1.0` are **typed refusals**,
//!   never silent saturations — a payoff outside the unit interval is a
//!   malformed claim, not a boundary case.
//! - Integer fractions (`quantize_fraction`) take the exact `DependencyMeasure`
//!   content (`num`, `den`) on an integer-only path: total, no `f64`, no
//!   refusal — the range law is enforced upstream by the type system.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! Quantization is a pure, replayable byte transformation. The matrix
//! certifies "these scores, at this precision" — it never decides whether
//! the underlying process scores are *good*.

use thiserror::Error;

/// Errors produced at the float-to-fixed boundary.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum QuantizationRefusal {
    /// The score is not a finite number (NaN or ±infinity).
    #[error("payoff score is not finite: {score}")]
    NotFinite {
        /// The offending score.
        score: f64,
    },
    /// The score lies outside the unit interval.
    #[error("payoff score outside [0.0, 1.0]: {score}")]
    OutOfRange {
        /// The offending score.
        score: f64,
    },
}

/// Q16.16 fixed-point bit pattern for one payoff cell.
pub type Q16F16Bits = u32;

/// Scale factor: 2^16.
const SCALE: f64 = 65536.0;
/// Maximum representable Q16.16 magnitude in the unit interval (65535).
const MAX_BITS: f64 = 65535.0;

/// Quantize one `f64` score in `[0.0, 1.0]` to Q16.16 bits.
///
/// # Errors
///
/// - [`QuantizationRefusal::NotFinite`] for NaN and ±infinity.
/// - [`QuantizationRefusal::OutOfRange`] for values outside `[0.0, 1.0]`.
pub fn quantize(score: f64) -> Result<Q16F16Bits, QuantizationRefusal> {
    if !score.is_finite() {
        return Err(QuantizationRefusal::NotFinite { score });
    }
    if !(0.0..=1.0).contains(&score) {
        return Err(QuantizationRefusal::OutOfRange { score });
    }
    let scaled = (score * SCALE).round();
    Ok(scaled.min(MAX_BITS) as Q16F16Bits)
}

/// Quantize an integer fraction `num/den` (a `DependencyMeasure`'s static
/// content) on the integer path: `(num * 65536 + den/2) / den`, rounded to
/// nearest, saturating at 65535.
///
/// Total for `den > 0` (every `DependencyMeasure` guarantees that at the
/// type boundary). `den == 0` yields 0 — unreachable from a measure, and
/// refused nowhere because a panicking seam is not a seam. Call it as
/// `quantize_fraction(m.num(), m.den())`; no `f64` is touched.
#[must_use]
pub fn quantize_fraction(num: u64, den: u64) -> Q16F16Bits {
    if den == 0 {
        return 0;
    }
    let scaled = num.min(u64::MAX / 65536) * 65536;
    let rounded = (scaled + den / 2) / den;
    rounded.min(65535) as Q16F16Bits
}

/// A CMCA-shaped payoff matrix: `[N rows][2*Q cols]` of Q16.16 bit patterns.
///
/// The `2*Q` column split mirrors the allocator's direct leaf allocation
/// vs. descendant propagation flows; `N` is the node cardinality of the
/// generated cascade instance.
///
/// Dimensions are runtime values, not const generics: stable Rust forbids
/// const arithmetic (`2 * Q`) in array lengths (failed edge, 2026-10-03 —
/// it is gated behind `generic_const_exprs`), and `bcinr-cmca`'s N/K/Q are
/// generation-time constants anyway (see the C4 checklist item 3 verdict):
/// the generator instantiates this type with its concrete dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoffMatrix {
    matrix: Vec<Vec<Q16F16Bits>>,
    /// Number of policy lenses Q (columns are `2 * Q`).
    lenses: usize,
}

/// Errors assembling a payoff matrix.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum MatrixError {
    /// One of the scores violated the quantization law.
    #[error("matrix cell [{row}][{col}]: {source}")]
    Cell {
        /// Row index of the refused cell.
        row: usize,
        /// Column index of the refused cell.
        col: usize,
        /// The underlying refusal.
        source: QuantizationRefusal,
    },
    /// The input was ragged or misdimensioned.
    #[error(transparent)]
    Dimensions(#[from] DimensionError),
}

/// Errors assembling a payoff matrix from ragged or misdimensioned input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DimensionError {
    /// Row count does not match the declared node cardinality N.
    #[error("expected {expected} rows, got {got}")]
    RowCount {
        /// Declared rows.
        expected: usize,
        /// Observed rows.
        got: usize,
    },
    /// A row's column count does not match `2 * Q`.
    #[error("row {row}: expected {expected} columns, got {got}")]
    ColCount {
        /// Row index.
        row: usize,
        /// Declared columns (`2 * Q`).
        expected: usize,
        /// Observed columns.
        got: usize,
    },
}

impl PayoffMatrix {
    /// Allocate a zeroed matrix for `n_nodes` nodes and `lenses` lenses.
    #[must_use]
    pub fn zeroed(n_nodes: usize, lenses: usize) -> Self {
        Self {
            matrix: vec![vec![0u32; 2 * lenses]; n_nodes],
            lenses,
        }
    }

    /// Assemble from raw `f64` scores with declared dimensions, refusing
    /// malformed cells with coordinates and misdimensioned input with
    /// typed errors.
    ///
    /// # Errors
    ///
    /// - [`DimensionError`] if the input is ragged or misdimensioned.
    /// - [`MatrixError::Cell`] if any score violates the quantization law.
    pub fn from_scores(
        n_nodes: usize,
        lenses: usize,
        rows: &[Vec<f64>],
    ) -> Result<Self, MatrixError> {
        if rows.len() != n_nodes {
            return Err(MatrixError::Dimensions(DimensionError::RowCount {
                expected: n_nodes,
                got: rows.len(),
            }));
        }
        let mut matrix = Vec::with_capacity(n_nodes);
        for (row, row_scores) in rows.iter().enumerate() {
            if row_scores.len() != 2 * lenses {
                return Err(MatrixError::Dimensions(DimensionError::ColCount {
                    row,
                    expected: 2 * lenses,
                    got: row_scores.len(),
                }));
            }
            let mut out_row = Vec::with_capacity(2 * lenses);
            for (col, score) in row_scores.iter().enumerate() {
                out_row.push(quantize(*score).map_err(|source| MatrixError::Cell {
                    row,
                    col,
                    source,
                })?);
            }
            matrix.push(out_row);
        }
        Ok(Self { matrix, lenses })
    }

    /// Assemble from integer fractions `(num, den)` per cell — the
    /// `DependencyMeasure` content type-erased to its exact integer law.
    /// Total apart from dimension checks: no quantization refusal path.
    ///
    /// # Errors
    ///
    /// See [`DimensionError`].
    pub fn from_fractions(
        n_nodes: usize,
        lenses: usize,
        rows: &[Vec<(u64, u64)>],
    ) -> Result<Self, DimensionError> {
        if rows.len() != n_nodes {
            return Err(DimensionError::RowCount {
                expected: n_nodes,
                got: rows.len(),
            });
        }
        let mut matrix = Vec::with_capacity(n_nodes);
        for (row, row_fracs) in rows.iter().enumerate() {
            if row_fracs.len() != 2 * lenses {
                return Err(DimensionError::ColCount {
                    row,
                    expected: 2 * lenses,
                    got: row_fracs.len(),
                });
            }
            matrix.push(
                row_fracs
                    .iter()
                    .map(|(num, den)| quantize_fraction(*num, *den))
                    .collect(),
            );
        }
        Ok(Self { matrix, lenses })
    }

    /// The Q16.16 bit patterns, laid out for the CMCA allocator
    /// (`NonNegativeFixed` consumes the raw bits).
    #[must_use]
    pub fn bits(&self) -> &[Vec<Q16F16Bits>] {
        &self.matrix
    }

    /// Declared node cardinality N.
    #[must_use]
    pub fn nodes(&self) -> usize {
        self.matrix.len()
    }

    /// Declared policy lenses Q (columns are `2 * Q`).
    #[must_use]
    pub fn lenses(&self) -> usize {
        self.lenses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_interval_quantizes_round_to_nearest() {
        assert_eq!(quantize(0.0).expect("zero"), 0);
        assert_eq!(quantize(1.0).expect("one"), 0xFFFF, "1.0 clamps to 0xFFFF");
        assert_eq!(quantize(0.5).expect("half"), 32768);
        assert_eq!(quantize(0.25).expect("quarter"), 16384);
        // Round-to-nearest: 0.50002 * 65536 = 32769.3 -> 32769.
        assert_eq!(quantize(0.50002).expect("epsilon above half"), 32769);
    }

    #[test]
    fn malformed_scores_are_typed_refusals() {
        assert!(matches!(
            quantize(f64::NAN),
            Err(QuantizationRefusal::NotFinite { .. })
        ));
        assert!(matches!(
            quantize(f64::INFINITY),
            Err(QuantizationRefusal::NotFinite { .. })
        ));
        assert!(matches!(
            quantize(f64::NEG_INFINITY),
            Err(QuantizationRefusal::NotFinite { .. })
        ));
        assert!(matches!(
            quantize(-0.0001),
            Err(QuantizationRefusal::OutOfRange { .. })
        ));
        assert!(matches!(
            quantize(1.0001),
            Err(QuantizationRefusal::OutOfRange { .. })
        ));
    }

    #[test]
    fn fraction_path_is_total_and_exact() {
        // 1/2 -> 32768 exactly.
        assert_eq!(quantize_fraction(1, 2), 32768);
        // 1/3 -> round(65536/3) = 21845.
        assert_eq!(quantize_fraction(1, 3), 21845);
        // 3/4 -> 49152.
        assert_eq!(quantize_fraction(3, 4), 49152);
        // 1/1 -> saturates at 0xFFFF.
        assert_eq!(quantize_fraction(1, 1), 0xFFFF);
        // den == 0 (unreachable from a measure) yields 0, never a panic.
        assert_eq!(quantize_fraction(1, 0), 0);
    }

    #[test]
    fn matrix_preserves_layout_and_names_the_bad_cell() {
        let good = PayoffMatrix::from_scores(
            2,
            2,
            &[vec![0.5, 0.25, 1.0, 0.0], vec![0.0, 0.5, 0.25, 1.0]],
        )
        .expect("all cells in range");
        assert_eq!(good.nodes(), 2, "N rows");
        assert_eq!(good.lenses(), 2);
        let bits = good.bits();
        assert_eq!(bits[0].len(), 4, "2*Q columns");
        assert_eq!(bits[0][0], 32768);
        assert_eq!(bits[1][3], 0xFFFF);

        let bad = vec![vec![0.5, 0.5, 0.5, 0.5], vec![0.5, 0.5, f64::NAN, 0.5]];
        assert!(matches!(
            PayoffMatrix::from_scores(2, 2, &bad),
            Err(MatrixError::Cell {
                row: 1,
                col: 2,
                source: QuantizationRefusal::NotFinite { .. }
            })
        ));
    }

    #[test]
    fn ragged_input_is_a_typed_dimension_refusal() {
        let ragged = vec![vec![0.5, 0.5], vec![0.5, 0.5, 0.5, 0.5]];
        assert_eq!(
            PayoffMatrix::from_scores(2, 2, &ragged),
            Err(MatrixError::Dimensions(DimensionError::ColCount {
                row: 0,
                expected: 4,
                got: 2
            }))
        );
        let short = vec![vec![0.5; 4]];
        assert_eq!(
            PayoffMatrix::from_scores(2, 2, &short),
            Err(MatrixError::Dimensions(DimensionError::RowCount {
                expected: 2,
                got: 1
            }))
        );
    }

    #[test]
    fn matrix_from_fractions_is_total() {
        let m = PayoffMatrix::from_fractions(2, 1, &[vec![(1, 2), (1, 3)], vec![(3, 4), (1, 1)]])
            .expect("dimensions match");
        assert_eq!(m.lenses(), 1);
        let bits = m.bits();
        assert_eq!(bits[0][0], 32768);
        assert_eq!(bits[0][1], 21845);
        assert_eq!(bits[1][0], 49152);
        assert_eq!(bits[1][1], 0xFFFF);
    }
}
