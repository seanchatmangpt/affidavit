# How-To: Emitting Self-Authenticating Court Evidence (FRE 902(14))

**Goal:** Generate a court-admissible electronic records affidavit without requiring live custodian testimony.  

---

## 1. Statutory Context

Federal Rule of Evidence 902(14) provides that certified data copied from an electronic device, storage medium, or file is self-authenticating if it is accompanied by a written certification of a qualified person or process complying with the certification requirements.

---

## 2. Recipe

1. **Verify Receipt Chain Integrity:**
   ```bash
   affi verify receipt.json
   ```

2. **Generate Rule 902(14) Certificate:**
   ```bash
   affi certificate fre-902
   ```

3. **Output Artifact:**
   The CLI outputs an evidentiary declaration referencing the exact BLAKE3 root and ML-DSA-65 witness block:
   ```text
   AFFIDAVIT OF AUTHENTICITY PURSUANT TO FEDERAL RULE OF EVIDENCE 902(14)
   --------------------------------------------------------------------
   I hereby certify that the electronic data and records sealed in the
   attached Compact Trust Receipt Token (CTRT) chain have been verified
   by mathematical cryptographic process (BLAKE3 / FIPS 204 ML-DSA-65).
   No alteration, deletion, or spoliation was detected.
   Status: SELF_AUTHENTICATING_FRE_902_VALID
   ```
   This certificate can be attached directly to a Rule 56 Motion for Summary Judgment.
