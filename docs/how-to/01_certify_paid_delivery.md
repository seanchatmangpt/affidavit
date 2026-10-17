# How-To: Verifying Paid-Delivery Linkage Laws

**Goal:** Cryptographically link code delivery commits with commercial invoice settlements.  

---

## 1. Scenario

When executing software delivery contracts, disputes often arise over whether specific delivered Git commits correspond to specific paid milestone disbursements.

---

## 2. Recipe

1. **Tag Commit with Delivery Receipt:**
   ```bash
   affi emit --event-type "delivery.git.commit" --payload "{\"sha\":\"$(git rev-parse HEAD)\"}"
   ```

2. **Bind Settlement Transaction:**
   ```bash
   affi emit --event-type "finance.settlement.tx" --payload '{"tx_id":"tx_9874521","amount_usd":25000.00}'
   ```

3. **Assemble and Seal Delivery Receipt:**
   ```bash
   affi assemble --output delivery_seal.json
   ```

4. **Verify Standing:**
   ```bash
   affi standing verify delivery_seal.json
   ```
   Both parties now possess an unalterable, non-repudiable proof of paid delivery.
