/**
 * affidavit micro-verifier — AOD Historical Witness C ABI
 * AFFI-SPEC-26.11.19 PR-8 (embeddable micro-verifier)
 *
 * Authority: affidavit-core::aod::verify_witness only.
 * Live Gate (map_live) is intentionally absent — auditors never map Region 3.
 *
 * Protocol:
 *   1. af_aod_abi_version()  → must equal AF_AOD_ABI_VERSION
 *   2. af_aod_witness_verify(raw, len, &out) → 0 ACCEPT, else AodRefusal machine code
 *   3. On ACCEPT, `out` holds inert digests/counts (no Region 3 pointers)
 */

#ifndef AFFIDAVIT_AOD_H
#define AFFIDAVIT_AOD_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** ABI revision. Bump on any incompatible layout / semantics change. */
#define AF_AOD_ABI_VERSION 1u

/**
 * Inert Historical Witness attestation (Mode B).
 * Layout is fixed LE; size is 152 bytes (4×32 digests + 2×u64 + u32 + pad).
 * No pointers into the input buffer — Region 3 stays unmapped.
 */
typedef struct AfAodWitnessAttestation {
    uint8_t ontology_hash[32];
    uint8_t payload_digest[32];
    uint64_t lease_epoch;
    uint8_t lease_nullifier[32];
    uint8_t folded_receipt[32];
    uint64_t row_count;
    uint32_t buffer_count;
    uint32_t _reserved; /* must be written 0 by the callee */
} AfAodWitnessAttestation;

/** Return the ABI revision (`AF_AOD_ABI_VERSION`). */
uint32_t af_aod_abi_version(void);

/**
 * Historical Witness over a raw `.aod` byte buffer.
 *
 * @param raw  pointer to file bytes (may be unaligned; alignment is not demanded)
 * @param len  byte length
 * @param out  non-NULL destination; filled only when the return value is 0
 * @return 0 on ACCEPT; otherwise a positive AodRefusal machine code (0xE501..0xE50D)
 *         or -1 for null-pointer / zero-length input
 */
int32_t af_aod_witness_verify(const uint8_t *raw, size_t len,
                              AfAodWitnessAttestation *out);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* AFFIDAVIT_AOD_H */
