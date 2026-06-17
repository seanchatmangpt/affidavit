# The Affidavit Phase Change: From Audit Trail to Real-Time Process Control

## Executive Summary

The integration of Western Electric statistical process control (SPC) rules into affidavit represents a fundamental phase change in how we can monitor and govern distributed coding systems. We have transformed affidavit from a **post-hoc audit system** (record what happened, verify it later) into a **real-time process control system** (detect deviations as they occur, alert immediately).

This is not merely a feature addition. It is a shift in the ontological role of provenance data: from evidence to signal.

---

## Part I: The Original Phase (Provenance as Evidence)

### What Was Affidavit Before

Affidavit was designed as the **Provenance Layer**:

1. **Emit** — record operation-events into `.affi/working.json`
2. **Assemble** — finalize the chain into a BLAKE3-sealed Receipt
3. **Verify** — run 7-stage certification pipeline: decode → format → chain_integrity → continuity → verify_commitments → evaluate_profile → emit_verdict

The model was **retrospective and binary**:
- ACCEPT ✓ (receipt is well-formed, chain hash is sound, events are contiguous)
- REJECT ✗ (violation found at stage N, reason: …)

The verifier was an **attestation engine**: "I certify this chain is legitimate." It could never answer: "Is this process *good*?"

### The Constraint

Receipt verification answered a single question: **Did this chain's maker cheat?** (Did they tamper with it after sealing?)

It could not answer: **Is this chain's maker *slacking*?** (Are they writing stubs, skipping tests, accumulating debt?)

This is because:
- Verification is synchronous and local (one receipt, one verifier)
- Verification is deterministic (same receipt → same verdict always)
- Verification is binary (ACCEPT/REJECT with no degrees of freedom)
- Verification is backward-looking (checking what was already sealed)

---

## Part II: The Phase Change (Provenance as Control Signal)

### What Affidavit Becomes

With Western Electric integration, affidavit becomes a **process control system**:

1. **Measure** — collect code quality metrics (stubs, types, churn, complexity, clippy, etc.)
2. **Emit** — record measurements as `quality.measurement` events into the receipt chain
3. **Analyze** — run 7 Western Electric rules on rolling window of measurements
4. **Alert** — emit `quality.violation` events the moment a rule triggers
5. **Act** — fail CI/CD, block merge, send Slack notification, trigger remediation

The model is **predictive and continuous**:
- Watch a rolling window (last 20 measurements)
- Detect the *first moment* a process goes out of control
- Emit alerts in real-time, not after the fact
- Multiple severity levels (CRITICAL, HIGH, MEDIUM, INFO)

The analyzer is a **control chart engine**: "This process is deviating from baseline." It answers: **Is this process trending wrong?** and **When did it start?**

### The Capability Leap

This answers an entirely new question: **What is the health trajectory of this swarm?**

#### Seven Western Electric Rules as Seven Lenses

Each rule detects a different failure mode:

| Rule | Detects | LLM Cheating Signal |
|------|---------|-------------------|
| **1σ rule** | Sudden spike | Hallucination explosion (todo! ratio jumps from 2% → 35%) |
| **9-in-a-row** | Persistent out-of-control | Zombie code (unmaintainable for 9+ commits) |
| **Trend** | Systematic degradation | Rushing (complexity/churn monotonically increasing) |
| **Alternating** | Uncertainty | Flip-flopping (LLM rewriting same code repeatedly) |
| **2-of-3 beyond 2σ** | Early warning | Early signs of trouble (before it becomes critical) |
| **4-of-5 beyond 1σ** | Sustained deviation | Dragging baseline (consistently worse than baseline) |
| **15-in-a-row within 1σ** | Plateau/stagnation | Stuck in local minimum (no improvement, just treading water) |

**Together, these 7 rules form a complete topology of process failure.**

They are orthogonal: a process can fail in any one (or multiple) ways simultaneously.

---

## Part III: Why This Is a Phase Change (Not Just a Feature)

### Three Criteria for Phase Change

A true phase change has three characteristics:

#### 1. **Change in Dimensionality**

**Before:** 1-dimensional (time) + 1-dimensional (verdict: accept/reject)
- One receipt → one answer → done

**After:** Multi-dimensional (time × metrics × rules × severity × output_channels)
- Continuous stream of measurements → rolling window analysis → 7 independent rule checks → 3 severity levels → 4 output channels (stderr, JSON, events, webhooks)

The state space exploded. We went from a binary classifier to a continuous monitoring system.

#### 2. **Change in Real-Time Capability**

**Before:** Batch (emit → assemble → verify takes seconds; results appear only after finalization)

**After:** Real-time (measurements and alerts flow within seconds of code changes)
- Git hook captures violation **before merge** (abort the PR)
- Webhook fires to Slack **instantly** (developers see alert in chat)
- File watcher detects drift **as user types** (live feedback loop)

This is a change from **reactive** (check after the fact) to **proactive** (predict before the fact).

#### 3. **Change in Ontological Role**

**Before:** Provenance as **artifact**
- The receipt is the deliverable: "Here is proof of what happened"
- Verification is about **authenticity**: "Is this real?"

**After:** Provenance as **signal**
- Measurements and violations flow into real-time dashboards
- Analysis is about **health**: "Is this good? Is it trending worse?"

The receipt chain is no longer just a record of what happened—it's a **control signal** fed back to the swarm to maintain quality.

---

## Part IV: The Swarm Governance Implication

### From Audit to Governance

This phase change enables **autonomous swarm governance**:

**Old model:** Humans audit receipts after the fact.
- "Did LLM-A cheat?" → verify receipt
- "How well did LLM-B perform?" → run analytics on finalized receipts
- Latency: hours or days

**New model:** The system self-governs in real-time.
- "Is this LLM starting to cheat?" → Western Electric rule fires
- "Should this merge be blocked?" → CRITICAL violation → reject
- "Does this swarm need attention?" → webhook to ops
- Latency: seconds

### The 300+ Repos Use Case

For the original goal—making affidavit the nexus of 300+ repos:

**Phase 0:** Each repo runs its own CI/CD, emits events to affidavit
- Affidavit is a dumb logger: emit → assemble → verify

**Phase 1 (NOW):** Central quality monitoring of all repos simultaneously
- Every repo's code goes through quality measurement
- All measurements feed into a global rolling window
- Single command: `affi receipt monitor --watch /repos/* --output webhook`
- One dashboard, 300 repos, real-time health signals

This scales the governance problem from "Can we verify 300 receipts?" to "Can we keep 300 coding swarms in control?"

---

## Part V: The Western Electric Rules as Formal Specification

### From Informal Heuristics to Formal Detection

Before this phase change, LLM quality monitoring was ad-hoc:
- "This code looks rushed" (subjective)
- "This has too many todos" (threshold-based, brittle)
- "The commits are getting bigger" (eyeball-and-hope)

Western Electric rules are **formal, statistical, proven in manufacturing**:
- Each rule is a **mathematical statement** about the process distribution
- Each rule has **theoretical guarantees** (false-positive rate, power)
- The 7 rules together form a **complete basis** for out-of-control detection

This elevates LLM monitoring from art to engineering.

---

## Part VI: The Combinatorial Maximalism

### Why All Metrics, All Rules, All Outputs?

The implementation deliberately chose **combinatorial maximalism**:

- **All 13 metrics:** stubs, types, churn, comments, complexity, clippy, rustfmt, cargo-deny, cargo-audit, test coverage, doc coverage, + 2 more
- **All 7 rules:** 1σ, 9-in-a-row, Trend, Alternating, 2-of-3, 4-of-5, 15-in-row
- **All watch mechanisms:** CLI, git hook, file watcher, CI/CD, manual
- **All outputs:** stderr, JSON, receipt events, webhooks

This is not accidental. It reflects an insight: **we don't yet know which signals matter most for detecting LLM cheating.**

By instrumenting everything and routing to all channels, we create a **combinatorial discovery space**. Teams can experiment: "Which metrics best predict code quality? Which rules catch the most cheating? Which watch mechanism gives earliest signal?"

Maximalism here is exploratory, not wasteful.

---

## Part VII: Implications for the Larger Vision

### The Three Layers of Affidavit

This work reveals three conceptual layers:

#### Layer 0: Receipt Chain (Completed)
- Sealing, content-addressing, BLAKE3 rolling hash
- 7-stage verification pipeline
- Immutable audit trail
- **Role:** Proof-of-work, tamper detection

#### Layer 1: Quality Monitoring (Just Completed)
- Western Electric SPC rules
- Real-time violation detection
- Severity-based alerting
- **Role:** Process control, drift detection

#### Layer 2: Predictive Governance (Future)
- Machine learning on violation patterns
- Predictive remediation (auto-suggest fixes)
- Swarm-level strategy (allocate coding tasks based on health)
- **Role:** Autonomous governance, self-optimization

This phase change opens the door to Layer 2.

---

## Part VIII: The Philosophical Shift

### From "What Happened?" to "What Will Happen?"

**Phase 0 provenance:** "Prove that X happened."
- Retrospective
- Binary (happened/didn't happen)
- For **evidence**, **audit**, **blame**

**Phase 1 + Western Electric:** "Warn me when X is about to happen."
- Predictive
- Continuous (degree of deviation)
- For **control**, **adaptation**, **prevention**

This is the difference between:
- A **black box recorder** (captured what was in the fuselage when it crashed)
- A **flight control system** (prevents the crash from happening)

Affidavit has moved from black box to flight control.

---

## Conclusion: The New Era

With Western Electric integration, affidavit enters a new era:

1. **From Audit to Control** — Verification shifted from "did this happen?" to "is this healthy?"
2. **From Reactive to Proactive** — Alerts fire before code is merged, not after it's deployed
3. **From Single-Receipt to Swarm-Level** — The system now governs populations of LLMs, not just individual chains
4. **From Evidence to Signal** — Receipts are no longer just proof; they are **continuous feedback** feeding back to the system being measured

The phase change is real. The capability leap is substantial. The implications for AI governance are profound.

We have built the infrastructure for **real-time, decentralized, statistical process control of coding swarms.**

What we do with it is up to us.

---

**Date:** 2026-06-17  
**Author:** Claude + User (collaborative development)  
**Context:** Combinatorial maximalism approach to LLM quality monitoring
