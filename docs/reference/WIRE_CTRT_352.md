# Reference: 352-Byte Compact Trust Receipt Token (CTRT)

**Classification:** Byte-Exact Wire Protocol Specification  
**Version:** `1.0` (`0x0001`)  
**Source Module:** [`src/wire_ctrt.rs`](file:///Users/sac/affidavit/src/wire_ctrt.rs)  

---

## 1. Structural Layout

The Compact Trust Receipt Token (CTRT) is a packed, fixed-size 352-byte binary token. It seals state transition admissions, execution ticks, liquid avoided cloud tariffs, statutory anchors, and post-quantum cryptographic signatures into an immutable wire format.

```
Offset (Bytes)   Field Name                 Type       Description / Semantics
────────────────────────────────────────────────────────────────────────────────────────
0..4             magic                      [u8; 4]    ASCII "CTRT" (0x43545254)
4..6             version                    u16 LE     Wire format version (0x0001)
6..38            parent_receipt_hash        [u8; 32]   Rolling BLAKE3 root of parent event
38..70           court_id                   [u8; 32]   Canonical verification court identifier
70..102          subject_merkle_root        [u8; 32]   BLAKE3 root of payload data
102..134         compiler_binary_digest     [u8; 32]   Engine build hash / git tree digest
134..142         execution_ticks            u64 LE     Monotonic CPU ticks expended
142..150         elapsed_cycles             u64 LE     Hardware execution cycle count
150..151         verdict_status             u8         0x01 ACCEPT, 0x02 REJECT

--- STATUTORY & DEONTIC EXTENSION (201 Bytes: 151..352) ---
151..167         statutory_anchor           [u8; 16]   SOX_1514A | FRCP_37E | GAIN_5050
167..175         tariff_basis_cents         u64 LE     Liquid avoided compute (USD cents)
175..176         deontic_modality           u8         0x01 PERMIT, 0x02 OBLIG, 0x03 HOLD
176..208         nullifier_commitment       [u8; 32]   Deterministic anti-replay nullifier
208..240         ocel_event_digest          [u8; 32]   IEEE OCEL 2.0 event trace linkage
240..352         pqc_witness_signature      [u8; 112]  Truncated ML-DSA-65 witness block
────────────────────────────────────────────────────────────────────────────────────────
Total: Exactly 352 Bytes
```

---

## 2. Statutory Anchors (`bytes 151..167`)

Bytes `151..167` are zero-padded ASCII strings encoding the active federal or commercial statutory authority:

- `SOX_1514A\0\0\0\0\0\0\0`: Sarbanes-Oxley Act § 1514A (Whistleblower Retaliation Shield). Prohibits retaliation or adverse employment actions against agents emitting compliance telemetry.
- `FRCP_37E\0\0\0\0\0\0\0\0\0`: Federal Rule of Civil Procedure 37(e) (Evidentiary Preservation Hold). Establishes immediate spoliation inferences if logs are deleted.
- `GAIN_5050\0\0\0\0\0\0\0\0`: Constructive Trust 50/50 Gain-Share Allocation. Binds enterprise avoided-cost savings directly to the intellectual property licensor under Restatement (Third) of Restitution § 42.

---

## 3. Deontic Modality Codes (`byte 175`)

- `0x01` (`DEONTIC_PERMITTED`): Royalty-free developmental or sub-enterprise execution ($<\$50\text{M}$ revenue).
- `0x02` (`DEONTIC_OBLIGATED`): Mandatory commercial gain-share tier ($\ge \$50\text{M}$ revenue). Stage 6 verifier requires unpruned continuity.
- `0x03` (`DEONTIC_PRESERVATION_HOLD`): Adversarial spoliation or unremitted deployment detected; affirmative litigation hold attached.

---

## 4. Constant-Time Deserialization

`wire_ctrt::CompactTrustReceiptToken::deserialize_ct(&bytes)` enforces constant-time branchless decoding over the complete 352-byte slice, eliminating timing side-channels during high-throughput auditor scans.
