# How-To: Verifying Paid-Delivery Linkage Laws

**Goal:** Cryptographically link code delivery commits with commercial invoice settlements.  

---

## 1. Scenario

When executing software delivery contracts, disputes often arise over whether specific delivered Git commits correspond to specific paid milestone disbursements.

---

## 2. Recipe

1. **Tag Commit with Delivery Receipt:**
   ```bash
   printf '{"sha":"%s"}' "$(git rev-parse HEAD)" > commit.json
   affi receipt emit --event-type "delivery.git.commit" --object repo:main --payload commit.json
   ```

2. **Bind Settlement Transaction:**
   ```bash
   echo '{"tx_id":"tx_9874521","amount_usd":25000.00}' > tx.json
   affi receipt emit --event-type "finance.settlement.tx" --object repo:main --payload tx.json
   ```

3. **Assemble and Seal Delivery Receipt:**
   ```bash
   affi receipt assemble --out delivery_seal.json
   ```

4. **Verify the sealed receipt:**
   ```bash
   affi receipt verify delivery_seal.json
   ```
   Both parties now possess an unalterable, non-repudiable proof of paid delivery.
