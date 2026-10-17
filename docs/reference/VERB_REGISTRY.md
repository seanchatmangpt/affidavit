# Reference: Canonical Verb Registry (100 Verbs)

**Authority:** [`src/registry.rs`](file:///Users/sac/affidavit/src/registry.rs) & [`ontology/affi-cli.ttl`](file:///Users/sac/affidavit/ontology/affi-cli.ttl)  
**Registered Verbs:** 100  
**Total Nouns / Groups:** 13 nouns / 11 groups  

---

## 1. Groups & Noun Summary

The 100 canonical verbs are organized into the following functional groupings:

1. **Core Provenance:** `emit`, `assemble`, `verify`, `show`, `diff`, `stats`, `timeline`, `model`.
2. **Compliance & Statutory:** `legal-hold`, `restitution`, `certificate`, `sbom`, `license-compliance`, `pci-dss`, `hipaa`, `gdpr-proof`, `soc2-audit`.
3. **Evidence Federation:** `standing`, `ecosystem`, `errc`, `envelope`, `evidence`.
4. **Key Management:** `keys` (`generate`, `import`, `list`, `rotate`, `revoke`).
5. **Dossier & Diagnostics:** `dossier`, `doctor`, `diagnose`, `fix`, `why`, `explain-incident`.
6. **Analytics & Metrics:** `dora-metrics`, `bus-factor`, `team-velocity`, `tech-debt`, `trend-analysis`.
7. **Security & Vulnerability:** `security-debt`, `find-blast-radius`, `root-cause`, `causality-chain`.
8. **Process Mining & Discovery:** `conformance`, `variance`, `profile`, `query`, `search`.
9. **Guidance & Education:** `guide`.

---

## 2. Statutory Compliance Verbs (v26.10.16 extensions; behavior current as of v26.10.17)

### `affi legal-hold status`
- **Syntax:** `affi legal-hold status [--matter <id>]`
- **Description:** Audits local OCEL and CTRT event chains to certify FRCP 37(e) evidentiary hold integrity.
- **Output:** JSON object reporting matter ID, hold status, spoliation flags, and preserved event count.

### `affi restitution calculate`
- **Syntax:** `affi restitution calculate [--tariff-schedule <azure|aws|gcp>]`
- **Description:** Computes liquidated avoided-cost distributions under Restatement (Third) of Restitution § 42.
- **Output:** Figures derived from the selected schedule's rate table × the chain's actual compute events (schedules differ); typed refusals on no-compute-event chains or unknown schedules.

### `affi certificate fre-902`
- **Syntax:** `affi certificate fre-902`
- **Description:** Emits a self-authenticating digital evidence affidavit under Federal Rule of Evidence 902(14).
- **Output:** Certified only over verifiable chains: recomputes each chain's BLAKE3 integrity before certifying (typed refusals on empty/tampered input); output references the actual chain hashes + event counts.
