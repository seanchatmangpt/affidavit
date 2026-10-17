# How-To: Auditing Local Repositories for Spoliation Gaps (FRCP 37(e))

**Goal:** Audit event streams and nullifiers to verify compliance with an affirmative legal hold.  

---

## 1. Context

Under Federal Rule of Civil Procedure 37(e), when electronically stored information that should have been preserved is lost because a party failed to take reasonable steps, the court may presume that the lost information was unfavorable to the party.

---

## 2. Recipe

1. **Attach Matter Scope to Legal Hold:**
   ```bash
   affi legal-hold status --matter "litigation-matter-2026-01"
   ```

2. **Output Evaluation:**
   - If output contains:
     ```json
     {"ok":true,"matter":"litigation-matter-2026-01","status":"EVIDENTIARY_HOLD_ACTIVE","spoliation_detected":false,"events_preserved":42}
     ```
     The event chain is unbroken and compliant.
   - If any log pruning or nullifier sequence gap is discovered, `spoliation_detected` returns `true`, triggering immediate Stage 6 rejection.
