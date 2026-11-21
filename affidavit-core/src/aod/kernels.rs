//! Avec / FFOR / AHDCLT kernels over sealed Region-3 pages
//! (AFFI-SPEC-26.11.19 PR-4 / PR-7).
//!
//! These are **consumers** of already-admitted sealed pages — they never touch
//! the CTRT shadow layout. All paths are `no_std`, allocation-free, and free of
//! `unsafe` (safe SWAR bit extract over little-endian packed ordinals).
//!
//! * [`AvecView`] — fixed-stride row slices (Arrow-avec style).
//! * [`FforView`] — fused Frame-of-Reference + bit-unpack (`value = base + Δ`).
//! * [`OrdinalPerm`] — optional AHDCLT ordinal permutation (ggen-ready) that
//!   maps logical domain ordinals ↔ opaque on-disk codes for RE opacity.

use super::refuse::AodRefusal;

// ---------------------------------------------------------------------------
// Avec — fixed-stride views
// ---------------------------------------------------------------------------

/// Fixed-stride avec-style view over a sealed page (`stride × row_count` bytes).
///
/// The page is borrowed; no copy. Row `i` is bytes `[i*stride, (i+1)*stride)`.
#[derive(Clone, Copy, Debug)]
pub struct AvecView<'a> {
    page: &'a [u8],
    stride: usize,
    row_count: usize,
}

impl<'a> AvecView<'a> {
    /// Admit a page whose length equals `stride × row_count` (both non-zero).
    ///
    /// Refuses [`AodRefusal::FixedStrideViolation`] when the product does not
    /// match `page.len()`, or either dimension is zero.
    #[inline]
    pub fn try_new(page: &'a [u8], stride: usize, row_count: usize) -> Result<Self, AodRefusal> {
        if stride == 0 || row_count == 0 {
            return Err(AodRefusal::FixedStrideViolation);
        }
        let expected = stride
            .checked_mul(row_count)
            .ok_or(AodRefusal::FixedStrideViolation)?;
        if page.len() != expected {
            return Err(AodRefusal::FixedStrideViolation);
        }
        Ok(Self {
            page,
            stride,
            row_count,
        })
    }

    /// Borrow buffer slot `i` from an admitted [`super::AodView`] as an avec.
    ///
    /// `stride` is the compiled-ontology per-row byte length for that slot.
    #[inline]
    pub fn from_aod_buffer(
        view: &super::AodView<'a>,
        slot: usize,
        stride: usize,
    ) -> Result<Self, AodRefusal> {
        let page = view.buffer(slot)?;
        let rows = view.row_count as usize;
        Self::try_new(page, stride, rows)
    }

    /// Per-row byte stride.
    #[inline]
    pub const fn stride(self) -> usize {
        self.stride
    }

    /// Number of logical rows.
    #[inline]
    pub const fn row_count(self) -> usize {
        self.row_count
    }

    /// Full page bytes.
    #[inline]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.page
    }

    /// Borrow row `i` as a fixed-length byte slice.
    #[inline]
    pub fn row(self, i: usize) -> Result<&'a [u8], AodRefusal> {
        if i >= self.row_count {
            return Err(AodRefusal::BoundsOrSlack);
        }
        let start = i
            .checked_mul(self.stride)
            .ok_or(AodRefusal::BoundsOrSlack)?;
        let end = start
            .checked_add(self.stride)
            .ok_or(AodRefusal::BoundsOrSlack)?;
        Ok(&self.page[start..end])
    }
}

// ---------------------------------------------------------------------------
// FFOR — fused Frame-of-Reference + bit-unpack
// ---------------------------------------------------------------------------

/// Maximum supported FFOR bit-width (fits a `u64` delta).
pub const FFOR_MAX_BIT_WIDTH: u8 = 64;

/// Byte length of a little-endian bit-packed stream of `count` values at
/// `bit_width` bits each (rounded up). Returns `None` on overflow or when
/// `bit_width` is outside `1..=64`.
#[inline]
pub const fn ffor_packed_bytes(count: usize, bit_width: u8) -> Option<usize> {
    if bit_width == 0 || bit_width > FFOR_MAX_BIT_WIDTH {
        return None;
    }
    let bits = match count.checked_mul(bit_width as usize) {
        Some(b) => b,
        None => return None,
    };
    Some(bits.div_ceil(8))
}

/// Fused Frame-of-Reference view: `value[i] = base + unpack(Δ_i)`.
///
/// Deltas are packed little-endian, contiguous in the bit stream (no padding
/// between values). Extraction is pure safe SWAR over `u64` windows.
#[derive(Clone, Copy, Debug)]
pub struct FforView<'a> {
    packed: &'a [u8],
    base: u64,
    bit_width: u8,
    count: usize,
}

impl<'a> FforView<'a> {
    /// Admit a packed FFOR page.
    ///
    /// `packed.len()` must equal [`ffor_packed_bytes`]`(count, bit_width)`.
    #[inline]
    pub fn try_new(
        packed: &'a [u8],
        base: u64,
        bit_width: u8,
        count: usize,
    ) -> Result<Self, AodRefusal> {
        let need = ffor_packed_bytes(count, bit_width).ok_or(AodRefusal::FixedStrideViolation)?;
        if packed.len() != need {
            return Err(AodRefusal::FixedStrideViolation);
        }
        if count == 0 {
            return Err(AodRefusal::FixedStrideViolation);
        }
        Ok(Self {
            packed,
            base,
            bit_width,
            count,
        })
    }

    /// Frame-of-reference base.
    #[inline]
    pub const fn base(self) -> u64 {
        self.base
    }

    /// Bits per packed delta.
    #[inline]
    pub const fn bit_width(self) -> u8 {
        self.bit_width
    }

    /// Number of values.
    #[inline]
    pub const fn count(self) -> usize {
        self.count
    }

    /// Decode value at index `i` (`base + Δ_i`).
    #[inline]
    pub fn get(self, i: usize) -> Result<u64, AodRefusal> {
        if i >= self.count {
            return Err(AodRefusal::BoundsOrSlack);
        }
        let delta = extract_bits_le(self.packed, i * self.bit_width as usize, self.bit_width)?;
        self.base
            .checked_add(delta)
            .ok_or(AodRefusal::BoundsOrSlack)
    }

    /// Unpack all values into `out` (must be exactly `count` long).
    pub fn unpack_into(self, out: &mut [u64]) -> Result<(), AodRefusal> {
        if out.len() != self.count {
            return Err(AodRefusal::FixedStrideViolation);
        }
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = self.get(i)?;
        }
        Ok(())
    }
}

/// Pack `values` as FFOR deltas relative to `base` into `out`.
///
/// Every `values[i]` must be `≥ base` and `(values[i] - base)` must fit in
/// `bit_width` bits. `out.len()` must equal [`ffor_packed_bytes`].
pub fn ffor_pack(
    values: &[u64],
    base: u64,
    bit_width: u8,
    out: &mut [u8],
) -> Result<(), AodRefusal> {
    let need =
        ffor_packed_bytes(values.len(), bit_width).ok_or(AodRefusal::FixedStrideViolation)?;
    if out.len() != need || values.is_empty() {
        return Err(AodRefusal::FixedStrideViolation);
    }
    out.fill(0);
    let mask = if bit_width == 64 {
        u64::MAX
    } else {
        (1u64 << bit_width) - 1
    };
    for (i, &v) in values.iter().enumerate() {
        let delta = v.checked_sub(base).ok_or(AodRefusal::BoundsOrSlack)?;
        if delta > mask {
            return Err(AodRefusal::BoundsOrSlack);
        }
        deposit_bits_le(out, i * bit_width as usize, bit_width, delta)?;
    }
    Ok(())
}

/// Extract `width` bits starting at `bit_offset` from a little-endian bit stream.
///
/// Uses overlapping `u64` windows (safe SWAR) — never more than 64 bits, so at
/// most two byte-aligned loads are required.
#[inline]
fn extract_bits_le(packed: &[u8], bit_offset: usize, width: u8) -> Result<u64, AodRefusal> {
    debug_assert!((1..=64).contains(&width));
    let byte_off = bit_offset / 8;
    let bit_in_byte = bit_offset % 8;
    // Need enough bytes to cover bit_in_byte + width bits.
    let bits_needed = bit_in_byte + width as usize;
    let bytes_needed = bits_needed.div_ceil(8);
    if byte_off
        .checked_add(bytes_needed)
        .map(|e| e > packed.len())
        .unwrap_or(true)
    {
        return Err(AodRefusal::BoundsOrSlack);
    }
    // Assemble up to 8 bytes into a u64 LE window (pad with zeros on the right).
    let mut window = 0u64;
    let take = bytes_needed.min(8);
    for j in 0..take {
        window |= (packed[byte_off + j] as u64) << (8 * j);
    }
    // If the window spans > 8 bytes (only when bit_in_byte + width > 64 — impossible
    // for width ≤ 64 and bit_in_byte < 8, since max is 71). Handle the 9th byte.
    if bytes_needed > 8 {
        let extra = packed[byte_off + 8] as u64;
        // Shift the high fragment into place after extracting from the low window.
        let low = window >> bit_in_byte;
        let high_bits = bits_needed - 64;
        let high_mask = (1u64 << high_bits) - 1;
        let high = (extra & high_mask) << (64 - bit_in_byte);
        let mask = if width == 64 {
            u64::MAX
        } else {
            (1u64 << width) - 1
        };
        return Ok((low | high) & mask);
    }
    let shifted = window >> bit_in_byte;
    let mask = if width == 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    Ok(shifted & mask)
}

/// Deposit `width` bits of `value` at `bit_offset` into a little-endian stream.
#[inline]
fn deposit_bits_le(
    packed: &mut [u8],
    bit_offset: usize,
    width: u8,
    value: u64,
) -> Result<(), AodRefusal> {
    debug_assert!((1..=64).contains(&width));
    let mask = if width == 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    let value = value & mask;
    for b in 0..width as usize {
        if (value >> b) & 1 == 0 {
            continue;
        }
        let abs = bit_offset + b;
        let byte = abs / 8;
        let bit = abs % 8;
        if byte >= packed.len() {
            return Err(AodRefusal::BoundsOrSlack);
        }
        packed[byte] |= 1u8 << bit;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// AHDCLT — ordinal permutation opacity hook
// ---------------------------------------------------------------------------

/// AHDCLT ordinal permutation — compile-time / ggen-ready opacity hook.
///
/// Ggen emits a bijective pair:
/// * `forward[logical] = opaque` (written into FFOR pages)
/// * `inverse[opaque] = logical` (restored by live consumers)
///
/// Without this table (bound into the compiled ontology binary), on-disk
/// ordinals do not map to domain symbols — raising RE opacity.
#[derive(Clone, Copy, Debug)]
pub struct OrdinalPerm<'a> {
    forward: &'a [u16],
    inverse: &'a [u16],
}

impl<'a> OrdinalPerm<'a> {
    /// Admit a ggen-emitted forward/inverse pair.
    ///
    /// Refuses empty tables, length mismatch, or non-bijective mappings
    /// (`inverse[forward[i]] != i` or `forward[inverse[i]] != i`).
    pub fn admit(forward: &'a [u16], inverse: &'a [u16]) -> Result<Self, AodRefusal> {
        if forward.is_empty() || forward.len() != inverse.len() {
            return Err(AodRefusal::FixedStrideViolation);
        }
        let n = forward.len();
        if n > u16::MAX as usize {
            return Err(AodRefusal::BoundsOrSlack);
        }
        for (i, &opaque) in forward.iter().enumerate() {
            let oi = opaque as usize;
            if oi >= n || inverse[oi] as usize != i {
                return Err(AodRefusal::BoundsOrSlack);
            }
        }
        for (opaque, &logical) in inverse.iter().enumerate() {
            let li = logical as usize;
            if li >= n || forward[li] as usize != opaque {
                return Err(AodRefusal::BoundsOrSlack);
            }
        }
        Ok(Self { forward, inverse })
    }

    /// Domain cardinality (`N`).
    #[inline]
    pub const fn len(self) -> usize {
        self.forward.len()
    }

    /// `true` when the permutation is empty (never after a successful [`Self::admit`]).
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.forward.is_empty()
    }

    /// Map logical ordinal → opaque on-disk code.
    #[inline]
    pub fn encode(self, logical: u16) -> Result<u16, AodRefusal> {
        self.forward
            .get(logical as usize)
            .copied()
            .ok_or(AodRefusal::BoundsOrSlack)
    }

    /// Map opaque on-disk code → logical ordinal.
    #[inline]
    pub fn decode(self, opaque: u16) -> Result<u16, AodRefusal> {
        self.inverse
            .get(opaque as usize)
            .copied()
            .ok_or(AodRefusal::BoundsOrSlack)
    }

    /// Decode an FFOR value that stores an opaque ordinal into a logical one.
    #[inline]
    pub fn decode_ffor_value(self, opaque_value: u64) -> Result<u16, AodRefusal> {
        if opaque_value > u16::MAX as u64 {
            return Err(AodRefusal::BoundsOrSlack);
        }
        self.decode(opaque_value as u16)
    }
}

/// Build the inverse of a forward permutation into `inverse_out`.
///
/// `forward` must already be a bijection of `0..N`; `inverse_out.len() == N`.
/// Useful for ggen projections that only emit the forward table.
pub fn invert_ordinal_perm(forward: &[u16], inverse_out: &mut [u16]) -> Result<(), AodRefusal> {
    if forward.is_empty() || forward.len() != inverse_out.len() {
        return Err(AodRefusal::FixedStrideViolation);
    }
    // Fill with sentinel then place; any leftover sentinel ⇒ non-bijective.
    inverse_out.fill(u16::MAX);
    let n = forward.len();
    for (logical, &opaque) in forward.iter().enumerate() {
        let oi = opaque as usize;
        if oi >= n || inverse_out[oi] != u16::MAX {
            return Err(AodRefusal::BoundsOrSlack);
        }
        inverse_out[oi] = logical as u16;
    }
    if inverse_out.contains(&u16::MAX) {
        return Err(AodRefusal::BoundsOrSlack);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avec_stride_rows() {
        // 4 rows × stride 8
        let mut page = [0u8; 32];
        for (i, row) in page.as_chunks_mut::<8>().0.iter_mut().enumerate() {
            row.fill(i as u8 + 1);
        }
        let view = AvecView::try_new(&page, 8, 4).unwrap();
        assert_eq!(view.row_count(), 4);
        assert_eq!(view.stride(), 8);
        assert_eq!(view.row(0).unwrap(), &[1u8; 8]);
        assert_eq!(view.row(3).unwrap(), &[4u8; 8]);
        assert_eq!(view.row(4).unwrap_err(), AodRefusal::BoundsOrSlack);
    }

    #[test]
    fn avec_refuses_bad_stride_product() {
        let page = [0u8; 15];
        assert_eq!(
            AvecView::try_new(&page, 8, 2).unwrap_err(),
            AodRefusal::FixedStrideViolation
        );
        assert_eq!(
            AvecView::try_new(&page, 0, 2).unwrap_err(),
            AodRefusal::FixedStrideViolation
        );
    }

    #[test]
    fn ffor_pack_unpack_round_trip() {
        let values = [100u64, 105, 103, 107, 100, 104];
        let base = 100u64;
        let bit_width = 3u8; // deltas 0..7
        let nbytes = ffor_packed_bytes(values.len(), bit_width).unwrap();
        let mut packed = [0u8; 8];
        assert_eq!(nbytes, 3); // 6*3 = 18 bits → 3 bytes
        ffor_pack(&values, base, bit_width, &mut packed[..nbytes]).unwrap();

        let view = FforView::try_new(&packed[..nbytes], base, bit_width, values.len()).unwrap();
        let mut out = [0u64; 6];
        view.unpack_into(&mut out).unwrap();
        assert_eq!(out, values);
        assert_eq!(view.get(1).unwrap(), 105);
        assert_eq!(view.get(6).unwrap_err(), AodRefusal::BoundsOrSlack);
    }

    #[test]
    fn ffor_swar_crosses_byte_boundary() {
        // bit_width=5, 5 values → 25 bits; indices cross byte edges.
        let values = [10u64, 11, 12, 13, 14];
        let base = 10;
        let w = 5u8;
        let n = ffor_packed_bytes(values.len(), w).unwrap();
        let mut packed = [0u8; 4];
        ffor_pack(&values, base, w, &mut packed[..n]).unwrap();
        let view = FforView::try_new(&packed[..n], base, w, values.len()).unwrap();
        for (i, &v) in values.iter().enumerate() {
            assert_eq!(view.get(i).unwrap(), v);
        }
    }

    #[test]
    fn ffor_bit_width_64() {
        let values = [u64::MAX - 1, u64::MAX];
        let base = u64::MAX - 1;
        let n = ffor_packed_bytes(2, 64).unwrap();
        assert_eq!(n, 16);
        let mut packed = [0u8; 16];
        ffor_pack(&values, base, 64, &mut packed).unwrap();
        let view = FforView::try_new(&packed, base, 64, 2).unwrap();
        assert_eq!(view.get(0).unwrap(), values[0]);
        assert_eq!(view.get(1).unwrap(), values[1]);
    }

    #[test]
    fn ordinal_perm_round_trip() {
        // Logical 0..4 → opaque {3,1,4,0,2}
        let forward: [u16; 5] = [3, 1, 4, 0, 2];
        let mut inverse = [0u16; 5];
        invert_ordinal_perm(&forward, &mut inverse).unwrap();
        let perm = OrdinalPerm::admit(&forward, &inverse).unwrap();
        for logical in 0u16..5 {
            let opaque = perm.encode(logical).unwrap();
            assert_eq!(perm.decode(opaque).unwrap(), logical);
        }
        // Full round-trip through FFOR-shaped opaque codes.
        let logicals = [0u64, 2, 4, 1, 3];
        let opaques: [u64; 5] = [
            perm.encode(0).unwrap() as u64,
            perm.encode(2).unwrap() as u64,
            perm.encode(4).unwrap() as u64,
            perm.encode(1).unwrap() as u64,
            perm.encode(3).unwrap() as u64,
        ];
        let base = 0u64;
        let w = 3u8;
        let nbytes = ffor_packed_bytes(5, w).unwrap();
        let mut packed = [0u8; 4];
        ffor_pack(&opaques, base, w, &mut packed[..nbytes]).unwrap();
        let ffor = FforView::try_new(&packed[..nbytes], base, w, 5).unwrap();
        for (i, &logical) in logicals.iter().enumerate() {
            let opaque = ffor.get(i).unwrap();
            assert_eq!(perm.decode_ffor_value(opaque).unwrap() as u64, logical);
        }
    }

    #[test]
    fn ordinal_perm_refuses_non_bijective() {
        let forward = [0u16, 0, 2]; // duplicate 0
        let inverse = [0u16, 1, 2];
        assert_eq!(
            OrdinalPerm::admit(&forward, &inverse).unwrap_err(),
            AodRefusal::BoundsOrSlack
        );
        let mut inv = [0u16; 3];
        assert!(invert_ordinal_perm(&forward, &mut inv).is_err());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn avec_over_sealed_aod_page() {
        use crate::aod::live::{map_live, CompiledOntology, InMemoryLeaseRegistry};
        use crate::aod::writer::{seal_aod, AodBufferSpec, SealParams};

        let ontology = [0x11u8; 32];
        let nullifier = [0x22u8; 32];
        let folded = [0u8; 32];
        // 3 rows × stride 4
        let page = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let raw = seal_aod(
            &[AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 3,
                lease_epoch: 1,
                lease_nullifier: &nullifier,
                folded_receipt: &folded,
            },
        )
        .unwrap();
        let compiled = CompiledOntology {
            ontology_hash: &ontology,
            strides: &[4],
        };
        let mut leases = InMemoryLeaseRegistry::new();
        let aod = map_live(&raw, &compiled, &mut leases).unwrap();
        let avec = AvecView::from_aod_buffer(&aod, 0, 4).unwrap();
        assert_eq!(avec.row(1).unwrap(), &[5, 6, 7, 8]);
    }
}
