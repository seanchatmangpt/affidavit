# wasm4pm Integration Planning — Complete Index

**Date:** 2026-06-14  
**Status:** Planning ✅ Complete  
**Total Docs:** 5 planning documents (82 KB)  
**Ready for:** Implementation  

---

## Document Map

### 1. WASM4PM_QUICK_REFERENCE.md (11 KB) ← START HERE
**Audience:** Developers, project managers  
**Format:** One-page reference + decision trees  
**Contains:**
- TL;DR (three surfaces at a glance)
- Files to create/modify
- Feature flags
- Quick test checklist
- Effort estimate (3–6 days)
- Risk flags and success metrics

**When to use:** First intake, quick lookup, team sync

---

### 2. WASM4PM_INTEGRATION_SUMMARY.md (10 KB) ← EXECUTIVE LEVEL
**Audience:** Stakeholders, architects  
**Format:** Executive summary + numbered deliverables  
**Contains:**
- Mission statement
- The numbers (80/20 split, LOC, tests)
- Key deliverables per phase
- Success criteria
- Next steps with timeline
- FAQ

**When to use:** Pitch, approval, scope discussion

---

### 3. WASM4PM_INTEGRATION_PLAN.md (17 KB) ← COMPREHENSIVE ROADMAP
**Audience:** Lead developers, architects  
**Format:** Detailed roadmap with full specifications  
**Contains:**
- Current architecture diagram
- wasm4pm capability surface (discovery, conformance, predictive)
- 20% glue integration points (4 sections)
- Witness test specifications (all 3)
- Feature flags (with table)
- CLI surface (new verbs + examples)
- Risk mitigation matrix
- File changes summary (detailed)
- Success criteria checklist
- References to sister docs

**When to use:** Implementation planning, architecture review, detailed guidance

---

### 4. WASM4PM_80_20_BREAKDOWN.md (20 KB) ← TECHNICAL DEEP-DIVE
**Audience:** Core developers (mining/tracing/CLI specialists)  
**Format:** Component-by-component breakdown with code examples  
**Contains:**
- Part 1: The 80% (wasm4pm modules)
  - 1.1 Discovery engines (HIM, Alpha, Inductive)
  - 1.2 Conformance checking (alignment fitness, replay)
  - 1.3 Predictive monitoring (next-activity, remaining-time)
  - 1.4 Format converters (OCEL I/O)
- Part 2: The 20% (affidavit glue)
  - Glue 1: Receipt ↔ OCEL conversion (60 LOC)
  - Glue 2: Verifier stages 8 & 9 (80 LOC)
  - Glue 3: OTel tracing hook (50 LOC)
  - Glue 4: CLI dispatch (90 LOC)
- Summary table: 80/20 distribution
- Data flow diagram
- Concrete 3-event example
- Maintenance burden analysis

**When to use:** Code design, integration architecture, algorithm selection

---

### 5. WASM4PM_WITNESS_TEST_TEMPLATES.md (24 KB) ← CODE STUBS
**Audience:** Test engineers, QA, developers  
**Format:** Runnable test templates with inline documentation  
**Contains:**
- Chicago TDD doctrine statement
- Test Template 1: Discovery Witness
  - `discovery_linearizes_simple_receipt()` (full code)
  - `discovery_detects_concurrent_objects()` (full code)
  - Helper functions
- Test Template 2: Conformance Witness
  - `conformance_accepts_lawful_receipt()` (full code)
  - `conformance_rejects_violated_flow()` (full code)
  - Model builder helper
- Test Template 3: Predictive Witness
  - `predictive_next_activity_from_prefix()` (full code)
  - `predictive_otel_spans_emitted()` (full code)
- Test execution instructions
- Witness claim statements (table)
- Fixture reuse recommendations

**When to use:** Test implementation, copy-paste stubs, validation approach

---

## How to Navigate This Planning

### Scenario 1: "I need to understand the plan in 15 minutes"
1. Read: **WASM4PM_QUICK_REFERENCE.md** (entire, 2 min)
2. Skim: **WASM4PM_INTEGRATION_SUMMARY.md** (deliverables section, 5 min)
3. Review: **Effort estimate + success criteria** (5 min)

### Scenario 2: "I need to present this to stakeholders"
1. Lead with: **WASM4PM_INTEGRATION_SUMMARY.md** (executive framing)
2. Show: Numbers (80/20, 290 LOC, 3 tests)
3. Answer: Risk mitigation matrix + FAQ

### Scenario 3: "I'm implementing Phase 2.1 (Discovery)"
1. Start: **WASM4PM_80_20_BREAKDOWN.md** § Glue 1 & 2 (architecture)
2. Design: `src/mining.rs` from breakdown examples
3. Implement: `src/verifier.rs` stage 8
4. Test: Stub from **WASM4PM_WITNESS_TEST_TEMPLATES.md** § Discovery Witness
5. Reference: **WASM4PM_INTEGRATION_PLAN.md** § Feature Flags for Cargo.toml

### Scenario 4: "I need to code review the integration"
1. Check: **WASM4PM_80_20_BREAKDOWN.md** (expected LOC per module)
2. Verify: Feature flags per **WASM4PM_INTEGRATION_PLAN.md**
3. Validate: Tests match witness claims in **WASM4PM_WITNESS_TEST_TEMPLATES.md**
4. Confirm: CI/CD integration (coverage, feature isolation)

### Scenario 5: "Something broke; how do I debug?"
1. Inspect: **WASM4PM_80_20_BREAKDOWN.md** § data flow diagram
2. Trace: Receipt → OCEL → mining engine → net → stages
3. Test: Use witness test stubs to isolate failure
4. Reference: **WASM4PM_INTEGRATION_PLAN.md** § Risk Mitigation for common issues

---

## Key Takeaways

| Aspect | Value | Reference |
|--------|-------|-----------|
| **Total effort** | 3–6 days | QUICK_REFERENCE.md (Effort Estimate) |
| **wasm4pm contribution** | 80% (algorithms) | 80_20_BREAKDOWN.md (Summary) |
| **Affidavit contribution** | 20% (glue, ~290 LOC) | 80_20_BREAKDOWN.md (Part 2) |
| **New features** | 3 (discovery, conformance, predictive) | INTEGRATION_PLAN.md (Feature Flags) |
| **Feature gate strategy** | All off-by-default; stable Rust unaffected | INTEGRATION_PLAN.md (Flags) |
| **Test count** | 3 witness tests (~420 LOC) | WITNESS_TEST_TEMPLATES.md |
| **Nightly requirement** | Only with wasm4pm features | QUICK_REFERENCE.md (Feature Flags) |
| **Chicago TDD doctrine** | "Log cannot lie" | WITNESS_TEST_TEMPLATES.md (intro) |

---

## File Locations

All planning documents are in `/Users/sac/affidavit/`:

```
WASM4PM_QUICK_REFERENCE.md              (11 KB) — one-pager
WASM4PM_INTEGRATION_SUMMARY.md          (10 KB) — executive
WASM4PM_INTEGRATION_PLAN.md             (17 KB) — comprehensive
WASM4PM_80_20_BREAKDOWN.md              (20 KB) — technical deep-dive
WASM4PM_WITNESS_TEST_TEMPLATES.md       (24 KB) — code stubs
WASM4PM_INDEX.md                        (this file)

Total: 82 KB of planning
```

---

## Implementation Checklist

### Before You Start
- [ ] Read QUICK_REFERENCE.md (decision tree)
- [ ] Review 80_20_BREAKDOWN.md (architecture)
- [ ] Understand witness test approach (WITNESS_TEST_TEMPLATES.md)

### Phase 2.1: Discovery
- [ ] Create `src/mining.rs` (~60 LOC)
  - [ ] Stub: `receipt_to_ocel()`
  - [ ] Copy examples from 80_20_BREAKDOWN.md
- [ ] Modify `src/verifier.rs` (~40 LOC)
  - [ ] Add stage 8: `stage_discover_process()`
  - [ ] Copy from WITNESS_TEST_TEMPLATES.md setup
- [ ] Create test file: `tests/wasm4pm_discovery_witness.rs`
  - [ ] Copy stubs from WITNESS_TEST_TEMPLATES.md
  - [ ] Adapt fixture builders
- [ ] Update `Cargo.toml`
  - [ ] Add wasm4pm dependency
  - [ ] Add feature: `discovery`
- [ ] Validate
  - [ ] `cargo test --features discovery`
  - [ ] `cargo test` (no features, stable Rust)

### Phase 2.2: Conformance
- (Repeat structure for stages 2.2 & 2.3)

### Phase 2.3: Predictive
- (Repeat structure for stage 2.3)

### Final
- [ ] All tests pass: `cargo test --all-features`
- [ ] Stable Rust unaffected: `cargo test` (no features)
- [ ] Documentation updated: INTEGRATIONS.md
- [ ] Version bumped: v26.6.14 → v26.7.0

---

## Cross-References

### From INTEGRATION_PLAN.md
- § Current Architecture → 80_20_BREAKDOWN.md (data flow)
- § Feature Flags → QUICK_REFERENCE.md (feature table)
- § Witness Tests → WITNESS_TEST_TEMPLATES.md (full code)

### From 80_20_BREAKDOWN.md
- § Glue 1-4 → INTEGRATION_PLAN.md (§ 20% Glue)
- § Examples → WITNESS_TEST_TEMPLATES.md (tests instantiate these)
- § Data flow → QUICK_REFERENCE.md (TL;DR)

### From WITNESS_TEST_TEMPLATES.md
- § Chicago TDD → INTEGRATION_PLAN.md (doctrine statement)
- § Feature gates → QUICK_REFERENCE.md (feature flags)
- § Setup/Act/Assert → 80_20_BREAKDOWN.md (architecture)

### From INTEGRATION_SUMMARY.md
- § Key Deliverables → INTEGRATION_PLAN.md (full roadmap)
- § Success Criteria → QUICK_REFERENCE.md (checklist)
- § References → all documents above

---

## Questions? Start Here

| Question | Answer Location |
|----------|-----------------|
| "What's the high-level plan?" | INTEGRATION_SUMMARY.md |
| "How do I start coding?" | 80_20_BREAKDOWN.md + WITNESS_TEST_TEMPLATES.md |
| "How long will this take?" | QUICK_REFERENCE.md (Effort Estimate) |
| "What's the architectural split?" | 80_20_BREAKDOWN.md (Summary table) |
| "What are the witness tests?" | WITNESS_TEST_TEMPLATES.md (all three) |
| "What are the risk factors?" | INTEGRATION_PLAN.md (Risk Mitigation) |
| "What's the Chicago TDD approach?" | WITNESS_TEST_TEMPLATES.md (intro + doctrine) |
| "Can I use this without nightly Rust?" | QUICK_REFERENCE.md (Feature Flags table) |
| "What's the one-pager version?" | QUICK_REFERENCE.md (entire) |

---

## Document Statistics

| Document | Size | Line Count | Sections | Code Examples |
|----------|------|-----------|----------|---|
| QUICK_REFERENCE.md | 11 KB | ~250 | 13 | 8 |
| INTEGRATION_SUMMARY.md | 10 KB | ~230 | 11 | 2 |
| INTEGRATION_PLAN.md | 17 KB | ~420 | 14 | 6 |
| 80_20_BREAKDOWN.md | 20 KB | ~500 | 18 | 12 |
| WITNESS_TEST_TEMPLATES.md | 24 KB | ~550 | 10 | 18 |
| **TOTAL** | **82 KB** | **~1950** | **66** | **46** |

---

## Next Actions

### For Implementation Teams
1. ✅ Read QUICK_REFERENCE.md (15 min)
2. ✅ Review WITNESS_TEST_TEMPLATES.md stubs (30 min)
3. ⏭️ Design `src/mining.rs` from 80_20_BREAKDOWN.md examples
4. ⏭️ Code Phase 2.1 with test-first approach
5. ⏭️ Verify against checklist (Phase 2.1 section)

### For Code Review
1. ✅ Understand 80/20 split from 80_20_BREAKDOWN.md
2. ✅ Check LOC expectations from QUICK_REFERENCE.md
3. ✅ Validate witness tests match WITNESS_TEST_TEMPLATES.md
4. ⏭️ Review feature flag gating per INTEGRATION_PLAN.md
5. ⏭️ Confirm no regressions on stable Rust

### For Stakeholders
1. ✅ Review INTEGRATION_SUMMARY.md (executive level)
2. ✅ Understand risk/mitigation matrix
3. ⏭️ Approve feature-flag strategy (stable Rust unaffected)
4. ⏭️ Greenlight Phase 2 implementation

---

**Planning Status:** ✅ **COMPLETE**  
**Ready for:** Implementation on signal  
**Contact:** Refer to individual planning documents for details  

---

## Version Info

- **Affidavit:** v26.6.14
- **wasm4pm:** 26.6.12 (workspace)
- **wasm4pm-compat:** 26.6.14 (nightly-required)
- **Planning date:** 2026-06-14
- **Phase 1 status:** Complete (emit/assemble/verify/show, 41 tests passing)
- **Phase 2 status:** Planning complete; ready for implementation
