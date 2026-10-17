# Explanation: Certify, Don't Decide

**Document Version:** `1.0`  
**Classification:** Foundational System Doctrine  

---

## 1. The Undecidability of Subjective Honesty

In complex, multi-agent, distributed environments, asking a software system to "detect fraud" or "verify honesty" is fundamentally undecidable:
- An attacker can forge benign-looking telemetry.
- An auditor cannot inspect hidden state transitions retroactively.
- Subjective narratives conflict during post-incident analysis.

`affidavit` replaces heuristic detection with **formal certification**.

---

## 2. The 7-Stage Decidable Pipeline

The verifier does not evaluate intent or ethics. It tests a candidate witness (the receipt) against an exact algebraic standard across 7 stages:

1. **Decode:** Check structural presence, endianness, and magic bytes (`CTRT`).
2. **Format Check:** Verify alignment against schema specifications.
3. **Chain Integrity:** Re-compute rolling BLAKE3 hashes to prove zero byte alteration.
4. **Continuity:** Verify monotonically increasing sequence numbers and parent hashes.
5. **Commitment Verify:** Check cryptographic signatures and Merkle roots.
6. **Profile Evaluation:** Evaluate deontic modalities, revenue ceilings, and authorized expunction seals.
7. **Final Verdict:** Output an atomic `ACCEPT` or `REJECT`.

Because every stage is decidable, verification executes in constant time with zero branch ambiguity.
