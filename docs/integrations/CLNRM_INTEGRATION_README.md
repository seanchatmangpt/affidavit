# clnrm Integration for affidavit — Quick Reference

**Status:** Plan Ready  
**Date:** 2026-06-14  
**Version:** affidavit 26.6.17 + clnrm-core 26.6.17

---

## What's Being Integrated?

Affidavit (the Provenance Layer) is integrating **three capabilities from clnrm-core** to strengthen receipt verification:

| Capability | What It Does | Affidavit Use |
|---|---|---|
| **Scenario Templates** | Generate test scenarios from TOML templates using Tera syntax | Generate parameterized receipt test cases |
| **Validators** | Verify span data against assertions (order, count, graph, hermeticity) | Validate receipt events conform to temporal/structural laws |
| **Mutations** | Generate adversarial test variants via chaos/permutations | Generate receipt mutants to verify verifier rejection |

---

## Three Integration Documents

### 1. **CLNRM_INTEGRATION_PLAN_26.6.17.md** (This Folder)
**The Master Plan**
- Executive summary
- 80/20 scope (clnrm-core only, no workspace)
- Three integration layers with code
- Witness tests (proof of integration)
- Implementation roadmap (5 phases)
- Risk mitigation

**When to Read:** Before starting implementation; reference throughout

---

### 2. **CLNRM_PUBLIC_API_SURFACE.md** (This Folder)
**The API Reference**
- Complete public API of clnrm-core
- Template functions and TOML schema
- Validator types and behavior
- Chaos scenarios and mutation enums
- Configuration structures
- Type mappings (receipt event → span)

**When to Read:** When writing integration code; use for type signatures

---

### 3. **CLNRM_INTEGRATION_EXAMPLES.md** (This Folder)
**The Code Cookbook**
- Copy-paste ready Rust examples
- Example 1: Template generation
- Example 2: Validator integration
- Example 3: Adversarial mutations
- Example 4: Integration test witness
- Example 5: Cargo.toml configuration

**When to Read:** When implementing; copy code blocks directly

---

## Quick Implementation Path

### Phase 1: Setup (30 min)
```bash
# 1. Add to Cargo.toml
[dependencies]
clnrm-core = { path = "../clnrm/crates/clnrm-core" }

# 2. Create module structure
mkdir -p src/clnrm_integration
touch src/clnrm_integration/{mod.rs,templates.rs,validators.rs,mutations.rs}

# 3. Create test files
touch tests/{clnrm_scenario_witness,clnrm_validator_witness,clnrm_adversarial_witness}.rs

# 4. Compile
cargo build
```

### Phase 2: Templates (1 hour)
Copy from `CLNRM_INTEGRATION_EXAMPLES.md` → Example 1:
- Implement `generate_receipt_verification_scenario()`
- Implement `load_receipt_scenario()`
- Test in `tests/clnrm_scenario_witness.rs`

### Phase 3: Validators (2 hours)
Copy from `CLNRM_INTEGRATION_EXAMPLES.md` → Example 2:
- Implement `receipt_event_to_span()`
- Implement `validate_receipt_ordering()`
- Implement `validate_receipt_counts()`
- Add `stage_clnrm_validation()` to `src/verifier.rs`
- Test in `tests/clnrm_validator_witness.rs`

### Phase 4: Mutations (2 hours)
Copy from `CLNRM_INTEGRATION_EXAMPLES.md` → Example 3:
- Implement `ReceiptMutation` enum
- Implement `mutate_receipt()`
- Implement `AdversarialCorpusGenerator`
- Test in `tests/clnrm_adversarial_witness.rs`

### Phase 5: Verify (1 hour)
```bash
cargo test --test '*clnrm*'      # Run witness tests
cargo test                        # Run all tests (check for regressions)
cargo clippy                      # Linter check
cargo doc --no-deps              # Generate docs
```

---

## Key Files to Modify

```
affidavit/
├── Cargo.toml                                    # ADD: clnrm-core dependency
├── src/
│   ├── lib.rs                                   # MODIFY: pub mod clnrm_integration
│   ├── clnrm_integration/                       # NEW
│   │   ├── mod.rs                              # PUBLIC API
│   │   ├── templates.rs                        # Template generation
│   │   ├── validators.rs                       # Validator adapters
│   │   └── mutations.rs                        # Mutation generation
│   ├── verifier.rs                             # MODIFY: Add clnrm_validation stage
│   └── verification/
│       └── validators.rs                       # NEW: Adapter functions
├── tests/
│   ├── clnrm_scenario_witness.rs               # NEW: Template test
│   ├── clnrm_validator_witness.rs              # NEW: Validator test
│   └── clnrm_adversarial_witness.rs            # NEW: Mutation test
└── docs/
    └── CLNRM_INTEGRATION_PLAN_26.6.17.md      # (this folder)
```

---

## Expected Test Output

After integration, running `cargo test` should show:

```
test clnrm_scenario_witness::test_generate_receipt_scenario ... ok
test clnrm_validator_witness::test_receipt_passes_clnrm_validation ... ok
test clnrm_validator_witness::test_receipt_fails_when_unordered ... ok
test clnrm_validator_witness::test_clnrm_stage_in_verdict ... ok
test clnrm_adversarial_witness::test_corrupted_hash_receipt_rejected ... ok
test clnrm_adversarial_witness::test_reordered_events_receipt_rejected ... ok
test clnrm_adversarial_witness::test_dropped_event_receipt_rejected ... ok
test clnrm_adversarial_witness::test_nist_corpus_all_rejected ... ok

test result: ok. 8 passed; 0 failed; 0 ignored
```

---

## Dependency Tree (After Integration)

```
affidavit
├── clnrm-core (26.6.17)
│   ├── tokio
│   ├── serde/serde_json
│   ├── tracing
│   ├── anyhow
│   ├── clnrm-template (1.3)
│   ├── chicago-tdd-tools (1.4.0)
│   └── ... (validation, chaos modules are internal)
├── blake3
├── serde/serde_json
├── anyhow
├── thiserror
└── ... (existing deps)
```

**Size Impact:** ~5-10 MB added (clnrm-core compiled artifacts)  
**Compile Time:** +30-60 seconds (one-time; incremental builds ~5 sec)

---

## Success Criteria

- [x] Plan documented (this folder)
- [x] Public API surface catalogued
- [x] Code examples provided
- [ ] Cargo.toml updated
- [ ] Modules created and compiled
- [ ] Witness tests written and passing
- [ ] No regressions in existing tests
- [ ] Clippy clean (no warnings)
- [ ] API docs generated

---

## Common Questions

### Q: Why not just use all of clnrm (cli/lsp/workspace)?
**A:** 80/20: Only three capabilities are needed for receipt verification. The full workspace adds CLI, LSP, and Docker complexity that affidavit doesn't need. We consume clnrm-core as a library only.

### Q: Can I use this with published clnrm-core on crates.io?
**A:** Yes! During development, use path dependency `{ path = "../clnrm/crates/clnrm-core" }`. When clnrm-core is published, change to `{ version = "26.6.17" }`.

### Q: What if clnrm-core API changes?
**A:** We've pinned to 26.6.17. Use `Cargo.lock` to lock transitive dependencies. If upgrading clnrm-core, update adapters in `validators.rs` (a light layer).

### Q: Do I need to understand clnrm internals?
**A:** No. The public API surface is documented. Treat clnrm-core as a black box that validates spans, generates templates, and produces mutations.

### Q: Will this make affidavit slow?
**A:** No. Validators run once per receipt (micro-seconds). Templates are generated once at compile-time. Mutations are test-only (not in release).

### Q: How do I test this locally without publishing?
**A:** Use `path = "../clnrm/crates/clnrm-core"` in Cargo.toml. Rust Cargo will resolve relative paths automatically.

---

## References

- **clnrm Repository:** `~/clnrm`
- **affidavit Repository:** `~/affidavit`
- **clnrm-core Source:** `~/clnrm/crates/clnrm-core/src`
- **clnrm Public API:** `~/clnrm/crates/clnrm-core/src/lib.rs`

---

## Integration Phases Timeline

| Phase | Duration | Tasks | Deliverable |
|---|---|---|---|
| 1: Setup | 0.5h | Cargo.toml, modules, structure | Compiling skeleton |
| 2: Templates | 1h | Template functions, TOML parsing, test | Template witness passing |
| 3: Validators | 2h | Receipt→Span adapter, validators, verifier stage | Validator witness passing |
| 4: Mutations | 2h | Mutation enum, corpus generation, test | Adversarial witness passing |
| 5: Verify | 1h | Full test, clippy, docs, cleanup | All tests passing, no regressions |
| **Total** | **6.5h** | | **Fully integrated & tested** |

---

## Next Steps

1. **Read** `CLNRM_INTEGRATION_PLAN_26.6.17.md` (Executive Summary)
2. **Reference** `CLNRM_PUBLIC_API_SURFACE.md` (API Types)
3. **Code** from `CLNRM_INTEGRATION_EXAMPLES.md` (Copy-paste)
4. **Test** with provided witness tests
5. **Commit** with message: `feat(clnrm): integrate templates, validators, mutations for receipt certification`

---

**Created:** 2026-06-14  
**Author:** Claude Code Agent  
**For:** Sean Chatman (xpointsh@gmail.com)
