# How-To: Computing Avoided-Tariff Distributions (Restitution § 42)

**Goal:** Calculate liquidated avoided hyperscaler compute spend and statutory gain-share allocation.  

---

## 1. Scenario

An enterprise runs millions of high-throughput test and build operations locally through sovereign engines (`smalt`), eliminating millions of dollars in disposable cloud spend while attempting to evade commercial gain-share obligations.

---

## 2. Recipe

1. **Calculate Restitution Basis:**
   ```bash
   affi restitution calculate --tariff-schedule aws
   ```

2. **Output Report:**
   ```json
   {
     "ok": true,
     "tariff_schedule": "aws",
     "avoided_tariff_usd": 1250000.00,
     "gain_share_due_usd": 625000.00,
     "certified_status": "LIQUIDATED_RESTITUTION_CALCULATED"
   }
   ```
   The calculated values provide an objective, liquidated measure of unjust enrichment under Restatement (Third) of Restitution § 42.
