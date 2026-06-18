# affidavit — Definition of Done: CI/CD Pipeline Integration Requirements & Configuration

**Version:** 26.6.17  
**Branch:** `claude/zen-cerf-oq87br`  
**Initiative:** DX/QOL 1000x (5 phases, 22 features)  
**Repo:** `seanchatmangpt/affidavit`

---

## Table of Contents

1. [Pipeline Architecture Overview](#1-pipeline-architecture-overview)
2. [Stage 1: Pre-commit Hooks](#2-stage-1-pre-commit-hooks)
3. [Stage 2: PR Validation Pipeline](#3-stage-2-pr-validation-pipeline)
4. [Stage 3: Benchmark Regression Gate](#4-stage-3-benchmark-regression-gate)
5. [Stage 4: Feature Matrix Testing](#5-stage-4-feature-matrix-testing)
6. [Stage 5: Documentation Gate](#6-stage-5-documentation-gate)
7. [Stage 6: Security Scan](#7-stage-6-security-scan)
8. [Stage 7: Release Pipeline](#8-stage-7-release-pipeline)
9. [GitHub Actions YAML](#9-github-actions-yaml)
10. [Local Development Makefile](#10-local-development-makefile)
11. [DoD: CI Integration Checklist](#11-dod-ci-integration-checklist)

---

## 1. Pipeline Architecture Overview

### 1.1 Pipeline Stages

```
┌──────────────────────────────────────────────────────────────────────┐
│                    affidavit CI/CD Pipeline                          │
│                                                                      │
│  pre-commit (local)                                                  │
│    fmt-check → clippy → lib-tests                                    │
│                              │                                       │
│  PR pipeline (GitHub Actions)                                        │
│    check → fmt → clippy → test → feature-matrix → doc → release-build│
│                              │                                       │
│  PR to main (additional gates)                                       │
│    bench-regression → security-scan                                  │
│                              │                                       │
│  main push                                                           │
│    all-above → store-bench-baseline → publish-artifacts              │
│                              │                                       │
│  release (manual trigger)                                            │
│    version-bump → dry-run → tag → publish-crates-io                 │
└──────────────────────────────────────────────────────────────────────┘
```

### 1.2 Branch Strategy

```
feature/*  ──────────────────────────────────→ PR Validation
                                                  (Stages 2, 4, 5, 6)
                     ↓ merge
claude/*   ──────────────────────────────────→ PR Validation + Bench Gate
(dev branch)                                      (Stages 2, 3, 4, 5, 6)
                     ↓ merge
main       ──────────────────────────────────→ All Stages + Baseline Update
                     ↓ manual trigger
release/*  ──────────────────────────────────→ Stage 7 (Release Pipeline)
```

| Branch Pattern | Triggers | Stages |
|---|---|---|
| `feature/*` | push, PR open/sync | 2, 4, 5, 6 |
| `claude/*` | push, PR open/sync | 2, 3, 4, 5, 6 |
| `main` | push | 2, 3, 4, 5, 6 + baseline update |
| `release/*` | manual dispatch | 7 |
| any | scheduled (nightly) | 2, 3, 4, 5, 6 (audit + beta toolchain) |

### 1.3 When Each Stage Runs

| Stage | PR Open/Update | Push to main | PR to main | Nightly | Manual |
|---|---|---|---|---|---|
| Pre-commit hooks | local only | local only | local only | — | — |
| PR Validation | ✓ | ✓ | ✓ | ✓ | — |
| Bench Regression | — | ✓ | ✓ | ✓ | ✓ |
| Feature Matrix | ✓ | ✓ | ✓ | ✓ | — |
| Doc Gate | ✓ | ✓ | ✓ | ✓ | — |
| Security Scan | ✓ | ✓ | ✓ | ✓ | — |
| Release Pipeline | — | — | — | — | ✓ |

### 1.4 Caching Strategy

```yaml
# Cargo registry cache — keyed on Cargo.lock
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-registry-

# Target directory cache — keyed on toolchain + Cargo.lock + source hash
- uses: actions/cache@v4
  with:
    path: target/
    key: ${{ runner.os }}-cargo-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('src/**/*.rs') }}
    restore-keys: |
      ${{ runner.os }}-cargo-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}-
      ${{ runner.os }}-cargo-target-${{ matrix.rust }}-
```

Cache invalidation rules:
- Registry cache: invalidated when `Cargo.lock` changes (new/updated dependencies)
- Target cache: invalidated when Rust source files or `Cargo.lock` change
- Benchmark baseline: stored as a GitHub Actions artifact, keyed to `main` branch SHA

---

## 2. Stage 1: Pre-commit Hooks

These run **locally before every commit**. They are fast (< 30 seconds), catching formatting and clippy violations before they reach CI.

### 2.1 Hook Script

Create `.git/hooks/pre-commit`:

```bash
#!/usr/bin/env bash
# affidavit pre-commit hook
# Runs: fmt-check, clippy, lib-unit-tests
# Fast path: fails fast on first error.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "[pre-commit] Running format check..."
cargo fmt --check 2>&1 || {
    echo ""
    echo "FAIL: Code is not formatted. Run 'cargo fmt' and re-stage."
    exit 1
}

echo "[pre-commit] Running clippy..."
cargo clippy -- -D warnings 2>&1 || {
    echo ""
    echo "FAIL: Clippy found warnings. Fix them before committing."
    exit 1
}

echo "[pre-commit] Running library unit tests..."
cargo test --lib 2>&1 || {
    echo ""
    echo "FAIL: Library tests failed. Fix them before committing."
    exit 1
}

echo "[pre-commit] All checks passed."
```

### 2.2 Pre-push Hook Script

Create `.git/hooks/pre-push` for a slightly heavier check before pushing:

```bash
#!/usr/bin/env bash
# affidavit pre-push hook
# Runs the full test suite (not just lib) before pushing to remote.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "[pre-push] Running full test suite..."
cargo test 2>&1 || {
    echo ""
    echo "FAIL: Tests failed. Fix them before pushing."
    exit 1
}

echo "[pre-push] Running golden-run example..."
bash examples/golden_run.sh 2>&1 || {
    echo ""
    echo "FAIL: Golden run failed. The full emit→assemble→verify lifecycle is broken."
    exit 1
}

echo "[pre-push] All pre-push checks passed."
```

### 2.3 Installation

Add a Makefile target (see §10) and document the one-liner:

```bash
# Install hooks (run once after cloning)
make install-hooks

# Or manually:
chmod +x .githooks/pre-commit .githooks/pre-push
git config core.hooksPath .githooks
```

Store canonical hook scripts in `.githooks/` (tracked in git) rather than `.git/hooks/` (untracked). The `make install-hooks` target points git at the tracked directory.

```bash
# .githooks/pre-commit  ← tracked in repo
# .githooks/pre-push    ← tracked in repo

# Makefile target:
install-hooks:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit .githooks/pre-push
	@echo "Hooks installed. git will now run .githooks/pre-commit before each commit."
```

---

## 3. Stage 2: PR Validation Pipeline

Runs on every PR open, synchronize (push), and reopen event. Also runs on direct pushes to `claude/*` and `main`.

### 3.1 Steps (in order)

```bash
# Step 1: Dependency check — fast, no compilation
cargo check --all-features

# Step 2: Format check
cargo fmt --check

# Step 3: Clippy — all targets including tests/benches/examples
cargo clippy --all-targets -- -D warnings

# Step 4: Full test suite (all 30+ tests)
cargo test

# Step 5: otel feature tests
cargo test --features otel

# Step 6: discovery + conformance + predictive features
cargo test --features discovery,conformance,predictive

# Step 7: mutation + shell + json-output features
cargo test --features mutation,shell,json-output

# Step 8: Documentation check (no broken links, no missing docs)
cargo doc --all-features --no-deps 2>&1 | grep -E "^error" && exit 1 || true

# Step 9: Release build (ensure no release-only failures)
cargo build --release
```

### 3.2 Expected Test Count

| Test group | Count | Command |
|---|---|---|
| Library unit tests | 19 | `cargo test --lib` |
| Dispatch tests (handlers.rs) | 6 | `cargo test --lib` |
| E2E integration tests | 4 | `cargo test --test '*'` |
| UI/compile-fail tests | 1 | `cargo test --test ui` |
| **Total** | **30** | `cargo test` |

Additional tests enabled per feature flag are accumulated on top of the base 30.

### 3.3 Failure Behavior

- Any step failure aborts the pipeline immediately (fail-fast within each job).
- The PR is blocked from merging until all steps pass.
- Failed steps post an inline annotation in the GitHub PR "Files changed" tab via the `--message-format=json` clippy output parser (see Actions YAML §9).

---

## 4. Stage 3: Benchmark Regression Gate

Runs on PRs targeting `main` and on every push to `main`.

### 4.1 Benchmark Suite

```
benches/receipt_operations/
└── receipt_operations.rs
    ├── chain_append_single_event      ← ~100µs baseline
    ├── chain_finalize/1               ← <1ms
    ├── chain_finalize/10              ← ~5ms
    ├── chain_finalize/100             ← ~50ms
    ├── chain_finalize/1000            ← ~500ms
    └── verifier_pipeline/10_events    ← ~75ms
```

### 4.2 Regression Thresholds

| Threshold | Action |
|---|---|
| Regression > 10% | **FAIL** — blocks merge |
| Regression 5–10% | **WARN** — posts comment, does not block |
| Regression < 5% | PASS (within noise floor) |
| Improvement any% | PASS — logs improvement |

### 4.3 Baseline Storage

```
# Baselines stored as GitHub Actions artifacts on main branch
artifact name: bench-baseline-{sha}
artifact path: target/criterion/

# On merge to main: upload new baseline
# On PR to main: download baseline from latest main SHA, compare
```

### 4.4 Comparison Script

```bash
#!/usr/bin/env bash
# scripts/bench-compare.sh
# Compare current Criterion output against stored baseline.
# Exits 1 if any benchmark regresses > FAIL_THRESHOLD.
# Exits 2 if any benchmark regresses > WARN_THRESHOLD (warning only, no block).
set -euo pipefail

FAIL_THRESHOLD="${BENCH_FAIL_THRESHOLD:-10}"   # percent
WARN_THRESHOLD="${BENCH_WARN_THRESHOLD:-5}"    # percent
BASELINE_DIR="${BASELINE_DIR:-./bench-baseline}"
CURRENT_DIR="${CURRENT_DIR:-./target/criterion}"
REPORT_FILE="${REPORT_FILE:-bench-report.md}"

fail_count=0
warn_count=0
report_lines=()

report_lines+=("## Benchmark Regression Report")
report_lines+=("")
report_lines+=("| Benchmark | Baseline (ns) | Current (ns) | Change | Status |")
report_lines+=("|---|---|---|---|---|")

# Parse Criterion's estimates.json for each benchmark
for current_estimates in "$CURRENT_DIR"/*/new/estimates.json; do
    bench_name=$(basename "$(dirname "$(dirname "$current_estimates")")")
    baseline_estimates="$BASELINE_DIR/$bench_name/new/estimates.json"

    if [[ ! -f "$baseline_estimates" ]]; then
        report_lines+=("| $bench_name | — | — | NEW | ✅ NEW |")
        continue
    fi

    # Extract mean point estimate in nanoseconds
    current_mean=$(python3 -c "
import json, sys
with open('$current_estimates') as f:
    d = json.load(f)
print(d['mean']['point_estimate'])
")
    baseline_mean=$(python3 -c "
import json, sys
with open('$baseline_estimates') as f:
    d = json.load(f)
print(d['mean']['point_estimate'])
")

    # Calculate percent change
    pct_change=$(python3 -c "
baseline = float('$baseline_mean')
current = float('$current_mean')
if baseline == 0:
    print(0)
else:
    print(round((current - baseline) / baseline * 100, 2))
")

    # Determine status
    if python3 -c "exit(0 if float('$pct_change') > $FAIL_THRESHOLD else 1)"; then
        status="❌ FAIL (+${pct_change}%)"
        fail_count=$((fail_count + 1))
    elif python3 -c "exit(0 if float('$pct_change') > $WARN_THRESHOLD else 1)"; then
        status="⚠️ WARN (+${pct_change}%)"
        warn_count=$((warn_count + 1))
    elif python3 -c "exit(0 if float('$pct_change') < 0 else 1)"; then
        status="✅ IMPROVED (${pct_change}%)"
    else
        status="✅ PASS (+${pct_change}%)"
    fi

    baseline_ms=$(python3 -c "print(round(float('$baseline_mean') / 1e6, 3))")
    current_ms=$(python3 -c "print(round(float('$current_mean') / 1e6, 3))")
    report_lines+=("| $bench_name | ${baseline_ms}ms | ${current_ms}ms | ${pct_change}% | $status |")
done

# Write report
printf '%s\n' "${report_lines[@]}" > "$REPORT_FILE"
cat "$REPORT_FILE"

if [[ $fail_count -gt 0 ]]; then
    echo ""
    echo "FAIL: $fail_count benchmark(s) regressed by more than ${FAIL_THRESHOLD}%."
    exit 1
fi

if [[ $warn_count -gt 0 ]]; then
    echo ""
    echo "WARN: $warn_count benchmark(s) regressed by ${WARN_THRESHOLD}–${FAIL_THRESHOLD}%. Review before merging."
    # exit 2 signals warn — Actions job continues but posts warning comment
    exit 2
fi

echo "All benchmarks within regression thresholds."
```

---

## 5. Stage 4: Feature Matrix Testing

Tests every meaningful feature combination. Feature flags are defined in `Cargo.toml` under `[features]`.

### 5.1 Feature Dependency Graph

```
default (no features)
otel
discovery
conformance  ──── implies ───→ discovery
predictive   ──── implies ───→ conformance → discovery
mutation
lsp
shell
json-output
metrics      ──── implies ───→ otel
full         ──── all features enabled
```

### 5.2 Test Matrix

| # | Feature Set | Cargo command | Expected result |
|---|---|---|---|
| 1 | default | `cargo test` | PASS |
| 2 | otel | `cargo test --features otel` | PASS |
| 3 | discovery | `cargo test --features discovery` | PASS |
| 4 | conformance | `cargo test --features conformance` | PASS |
| 5 | predictive | `cargo test --features predictive` | PASS |
| 6 | mutation | `cargo test --features mutation` | PASS |
| 7 | lsp | `cargo test --features lsp` | PASS |
| 8 | shell | `cargo test --features shell` | PASS |
| 9 | json-output | `cargo test --features json-output` | PASS |
| 10 | metrics | `cargo test --features metrics` | PASS |
| 11 | full (all) | `cargo test --all-features` | PASS |

### 5.3 Feature Combination Script

```bash
#!/usr/bin/env bash
# scripts/feature-matrix-test.sh
# Runs cargo test for every feature combination in the matrix.
set -euo pipefail

PASS=0
FAIL=0
RESULTS=()

run_feature_test() {
    local label="$1"
    local features="$2"

    echo "--- Testing feature set: $label ---"
    if [[ -z "$features" ]]; then
        CMD="cargo test --no-default-features"
    else
        CMD="cargo test --no-default-features --features $features"
    fi

    if eval "$CMD" 2>&1; then
        RESULTS+=("✅ PASS | $label")
        PASS=$((PASS + 1))
    else
        RESULTS+=("❌ FAIL | $label")
        FAIL=$((FAIL + 1))
    fi
}

run_feature_test "default"                      ""
run_feature_test "otel"                         "otel"
run_feature_test "discovery"                    "discovery"
run_feature_test "conformance"                  "conformance"
run_feature_test "predictive"                   "predictive"
run_feature_test "mutation"                     "mutation"
run_feature_test "lsp"                          "lsp"
run_feature_test "shell"                        "shell"
run_feature_test "json-output"                  "json-output"
run_feature_test "metrics"                      "metrics"
run_feature_test "full (all features)"          "otel,discovery,conformance,predictive,mutation,lsp,shell,json-output,metrics"

echo ""
echo "=== Feature Matrix Results ==="
printf '%s\n' "${RESULTS[@]}"
echo ""
echo "PASS: $PASS | FAIL: $FAIL"

[[ $FAIL -eq 0 ]] || exit 1
```

---

## 6. Stage 5: Documentation Gate

### 6.1 Requirements

1. `cargo doc --all-features --no-deps` produces **zero warnings** — rustdoc warnings are promoted to errors via `RUSTDOCFLAGS="-D warnings"`.
2. Every `examples/*.sh` script exits code 0 when run against the built binary.
3. `CHANGELOG.md` contains an entry for the current version (`26.6.17`).
4. `CLAUDE.md` architecture section is up to date (verified by checking that all `src/verbs/*.rs` files are listed).

### 6.2 Documentation Check Commands

```bash
# Doc build — zero warnings
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

# Check CHANGELOG has current version
VERSION=$(cargo metadata --no-deps --format-version 1 | \
    python3 -c "import json,sys; d=json.load(sys.stdin); print(d['packages'][0]['version'])")
grep -qF "[$VERSION]" CHANGELOG.md || {
    echo "FAIL: CHANGELOG.md is missing an entry for v$VERSION"
    exit 1
}
```

### 6.3 Auto-Example Runner

```bash
#!/usr/bin/env bash
# scripts/run-examples.sh
# Runs every *.sh file in examples/ and verifies exit code 0.
# Builds the release binary first.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "Building release binary..."
cargo build --release --bin affi

PASS=0
FAIL=0
RESULTS=()

for script in examples/*.sh; do
    echo "--- Running: $script ---"
    if bash "$script"; then
        RESULTS+=("✅ PASS | $script")
        PASS=$((PASS + 1))
    else
        RESULTS+=("❌ FAIL | $script (exit $?)")
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "=== Example Runner Results ==="
printf '%s\n' "${RESULTS[@]}"
echo "PASS: $PASS | FAIL: $FAIL"

[[ $FAIL -eq 0 ]] || exit 1
```

### 6.4 CLAUDE.md Architecture Freshness Check

```bash
#!/usr/bin/env bash
# scripts/check-claude-md.sh
# Verifies that every src/verbs/*.rs file is mentioned in CLAUDE.md.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
FAIL=0

for verb_file in "$REPO_ROOT"/src/verbs/*.rs; do
    verb_name=$(basename "$verb_file" .rs)
    if ! grep -qF "$verb_name" "$REPO_ROOT/CLAUDE.md"; then
        echo "WARN: src/verbs/$verb_name.rs is not mentioned in CLAUDE.md"
        FAIL=$((FAIL + 1))
    fi
done

if [[ $FAIL -gt 0 ]]; then
    echo ""
    echo "FAIL: CLAUDE.md architecture section is missing $FAIL verb(s)."
    echo "Update the verbs/ listing in CLAUDE.md before merging."
    exit 1
fi

echo "CLAUDE.md architecture check passed."
```

---

## 7. Stage 6: Security Scan

### 7.1 Requirements

1. No hardcoded secrets (API keys, tokens, passwords) in any tracked file.
2. No raw payload bytes stored in receipts — only BLAKE3 commitments are persisted.
3. `cargo audit` passes with no known vulnerabilities in the dependency tree.

### 7.2 Secret Detection

```bash
#!/usr/bin/env bash
# scripts/secret-scan.sh
# Scans for common secret patterns in the working tree.
# Intentional test payloads (small, named "payload") are allowlisted.
set -euo pipefail

FAIL=0

# Patterns that should never appear in source
SECRET_PATTERNS=(
    'AKIA[0-9A-Z]{16}'              # AWS Access Key
    'ghp_[A-Za-z0-9]{36}'          # GitHub Personal Access Token
    'sk-[A-Za-z0-9]{48}'           # OpenAI API key
    'xoxb-[0-9]+-[A-Za-z0-9]+'    # Slack bot token
    'password\s*=\s*"[^"]+'        # Hardcoded password assignment
    'secret\s*=\s*"[^"]+'          # Hardcoded secret assignment
)

for pattern in "${SECRET_PATTERNS[@]}"; do
    matches=$(git grep -rn -E "$pattern" -- \
        ':(exclude)*.md' \
        ':(exclude)*.txt' \
        ':(exclude)examples/golden_run.sh' \
        2>/dev/null || true)
    if [[ -n "$matches" ]]; then
        echo "FAIL: Potential secret found (pattern: $pattern):"
        echo "$matches"
        FAIL=$((FAIL + 1))
    fi
done

[[ $FAIL -eq 0 ]] || exit 1
echo "Secret scan passed."
```

### 7.3 Receipt Payload Safety Check

```bash
# Verify that no assembled receipt stores raw payload bytes (only commitments).
# This checks that the commitment field is a 64-char hex string (BLAKE3 digest)
# and that no "payload" key appears at the event level in receipt JSON files.
#!/usr/bin/env bash
set -euo pipefail

FAIL=0
for receipt in tests/fixtures/*.json tests/*.json 2>/dev/null; do
    [[ -f "$receipt" ]] || continue
    if python3 - "$receipt" <<'EOF'
import json, sys, re
with open(sys.argv[1]) as f:
    data = json.load(f)
events = data.get("events", [])
for i, ev in enumerate(events):
    if "payload" in ev:
        print(f"FAIL: event {i} in {sys.argv[1]} contains raw 'payload' field")
        sys.exit(1)
    commitment = ev.get("commitment", "")
    if not re.fullmatch(r'[0-9a-f]{64}', commitment):
        print(f"FAIL: event {i} commitment is not a valid BLAKE3 hex digest: {commitment!r}")
        sys.exit(1)
sys.exit(0)
EOF
    then
        :
    else
        FAIL=$((FAIL + 1))
    fi
done

[[ $FAIL -eq 0 ]] || exit 1
echo "Receipt payload safety check passed."
```

### 7.4 cargo audit

```bash
# Install cargo-audit if not present
cargo install cargo-audit --locked 2>/dev/null || true

# Run audit — fail on any known vulnerability
cargo audit
```

---

## 8. Stage 7: Release Pipeline

Manual trigger only (`workflow_dispatch`). Requires a `release/*` branch.

### 8.1 Release Checklist (human pre-flight)

Before triggering the release workflow, verify:

- [ ] `Cargo.toml` `[package] version` matches the intended release (`X.Y.Z`)
- [ ] `CHANGELOG.md` has a `## [X.Y.Z]` section with release notes
- [ ] All CI checks pass on `main`
- [ ] `cargo publish --dry-run` was run locally

### 8.2 Version Bump

```bash
# scripts/version-bump.sh <new-version>
# Updates version in Cargo.toml and ggen.toml, then commits.
set -euo pipefail
NEW_VERSION="${1?Usage: version-bump.sh <new-version>}"

# Cargo.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# ggen.toml (if present)
[[ -f ggen.toml ]] && sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" ggen.toml

# Verify
cargo check --quiet
echo "Version bumped to $NEW_VERSION"
```

### 8.3 Release Steps

```bash
# Step 1: Verify Cargo.toml version matches git tag intent
VERSION=$(cargo metadata --no-deps --format-version 1 | \
    python3 -c "import json,sys; d=json.load(sys.stdin); print(d['packages'][0]['version'])")

# Step 2: Dry-run publish to crates.io
cargo publish --dry-run

# Step 3: Tag and push
git tag -s "v$VERSION" -m "Release v$VERSION"
git push origin "v$VERSION"

# Step 4: Create GitHub release with CHANGELOG notes
NOTES=$(awk "/^## \[$VERSION\]/,/^## \[/" CHANGELOG.md | head -n -1 | tail -n +2)
gh release create "v$VERSION" \
    --title "affidavit v$VERSION" \
    --notes "$NOTES" \
    --verify-tag

# Step 5: Publish to crates.io (actual)
cargo publish
```

---

## 9. GitHub Actions YAML

Create `.github/workflows/ci.yml`:

```yaml
# .github/workflows/ci.yml
# affidavit CI/CD Pipeline — DX/QOL 1000x initiative
# Branch: claude/zen-cerf-oq87br → main
# Covers: check, fmt, clippy, test, feature-matrix, doc, bench, security, release

name: CI

on:
  push:
    branches:
      - main
      - "claude/**"
      - "feature/**"
      - "release/**"
  pull_request:
    branches:
      - main
      - "claude/**"
  schedule:
    # Nightly run at 02:00 UTC — catches dependency rot and beta toolchain breaks
    - cron: "0 2 * * *"
  workflow_dispatch:
    inputs:
      release:
        description: "Trigger release pipeline (requires release/* branch)"
        required: false
        default: "false"
      bench_only:
        description: "Run benchmark suite only"
        required: false
        default: "false"

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"

jobs:
  # ──────────────────────────────────────────────────────────────────────────
  # Job 1: PR Validation (Stage 2)
  # Runs on every PR and push to main/claude/* branches.
  # ──────────────────────────────────────────────────────────────────────────
  validate:
    name: "Validate (${{ matrix.rust }})"
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        rust:
          - nightly   # primary (matches rust-toolchain.toml)
          - beta      # forward-compatibility check
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain (${{ matrix.rust }})
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-registry-

      - name: Cache build target
        uses: actions/cache@v4
        with:
          path: target/
          key: ${{ runner.os }}-cargo-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('src/**/*.rs') }}
          restore-keys: |
            ${{ runner.os }}-cargo-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}-
            ${{ runner.os }}-cargo-target-${{ matrix.rust }}-

      - name: Check (all features)
        run: cargo check --all-features

      - name: Format check
        run: cargo fmt --check

      - name: Clippy (all targets, deny warnings)
        run: cargo clippy --all-targets -- -D warnings

      - name: Run tests (default features)
        run: cargo test

      - name: Run tests (otel feature)
        run: cargo test --features otel

      - name: Run tests (discovery + conformance + predictive)
        run: cargo test --features discovery,conformance,predictive
        # Feature flags not yet in Cargo.toml are skipped gracefully
        continue-on-error: ${{ matrix.rust == 'beta' }}

      - name: Run tests (mutation + shell + json-output)
        run: cargo test --features mutation,shell,json-output
        continue-on-error: ${{ matrix.rust == 'beta' }}

      - name: Doc check (no warnings)
        run: RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

      - name: Release build
        run: cargo build --release

  # ──────────────────────────────────────────────────────────────────────────
  # Job 2: Feature Matrix Testing (Stage 4)
  # Runs every feature combination to catch cfg-gated compilation failures.
  # ──────────────────────────────────────────────────────────────────────────
  feature-matrix:
    name: "Feature Matrix"
    runs-on: ubuntu-latest
    needs: validate
    strategy:
      fail-fast: false
      matrix:
        features:
          - ""                        # default
          - "otel"
          - "discovery"
          - "conformance"
          - "predictive"
          - "mutation"
          - "lsp"
          - "shell"
          - "json-output"
          - "metrics"
          - "otel,discovery,conformance,predictive,mutation,lsp,shell,json-output,metrics"
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain (nightly)
        uses: dtolnay/rust-toolchain@nightly

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-registry-

      - name: Cache build target
        uses: actions/cache@v4
        with:
          path: target/
          key: ${{ runner.os }}-cargo-target-nightly-matrix-${{ matrix.features }}-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-target-nightly-matrix-

      - name: Test features=[${{ matrix.features || 'default' }}]
        run: |
          if [[ -z "${{ matrix.features }}" ]]; then
            cargo test --no-default-features
          else
            cargo test --no-default-features --features "${{ matrix.features }}"
          fi

  # ──────────────────────────────────────────────────────────────────────────
  # Job 3: Documentation Gate (Stage 5)
  # Verifies docs, examples, CHANGELOG freshness, and CLAUDE.md completeness.
  # ──────────────────────────────────────────────────────────────────────────
  doc-gate:
    name: "Documentation Gate"
    runs-on: ubuntu-latest
    needs: validate
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain (nightly)
        uses: dtolnay/rust-toolchain@nightly

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-registry-

      - name: Build docs (all features, deny doc warnings)
        run: RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

      - name: Build release binary for example runner
        run: cargo build --release --bin affi

      - name: Run all shell examples
        run: |
          PASS=0; FAIL=0; RESULTS=()
          for script in examples/*.sh; do
            echo "--- $script ---"
            if bash "$script"; then
              RESULTS+=("PASS: $script")
              PASS=$((PASS + 1))
            else
              RESULTS+=("FAIL: $script")
              FAIL=$((FAIL + 1))
            fi
          done
          printf '%s\n' "${RESULTS[@]}"
          echo "PASS=$PASS FAIL=$FAIL"
          [[ $FAIL -eq 0 ]]

      - name: Check CHANGELOG has current version entry
        run: |
          VERSION=$(cargo metadata --no-deps --format-version 1 | \
            python3 -c "import json,sys; d=json.load(sys.stdin); print(d['packages'][0]['version'])")
          echo "Checking CHANGELOG.md for [$VERSION]..."
          grep -qF "[$VERSION]" CHANGELOG.md || {
            echo "FAIL: CHANGELOG.md missing entry for v$VERSION"
            exit 1
          }
          echo "CHANGELOG check passed for v$VERSION"

      - name: Check CLAUDE.md architecture completeness
        run: |
          FAIL=0
          for verb_file in src/verbs/*.rs; do
            verb_name=$(basename "$verb_file" .rs)
            if ! grep -qF "$verb_name" CLAUDE.md; then
              echo "WARN: src/verbs/$verb_name.rs not mentioned in CLAUDE.md"
              FAIL=$((FAIL + 1))
            fi
          done
          [[ $FAIL -eq 0 ]] || {
            echo "FAIL: CLAUDE.md architecture section is missing $FAIL verb(s)"
            exit 1
          }
          echo "CLAUDE.md architecture check passed"

  # ──────────────────────────────────────────────────────────────────────────
  # Job 4: Security Scan (Stage 6)
  # Secret detection + receipt payload safety + cargo audit.
  # ──────────────────────────────────────────────────────────────────────────
  security:
    name: "Security Scan"
    runs-on: ubuntu-latest
    needs: validate
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain (nightly)
        uses: dtolnay/rust-toolchain@nightly

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-registry-

      - name: Scan for hardcoded secrets
        run: |
          FAIL=0
          declare -a PATTERNS=(
            'AKIA[0-9A-Z]{16}'
            'ghp_[A-Za-z0-9]{36}'
            'sk-[A-Za-z0-9]{48}'
            'xoxb-[0-9]+-[A-Za-z0-9]+'
          )
          for pattern in "${PATTERNS[@]}"; do
            matches=$(git grep -rn -E "$pattern" -- \
              ':(exclude)*.md' ':(exclude)*.txt' \
              2>/dev/null || true)
            if [[ -n "$matches" ]]; then
              echo "FAIL: Potential secret (pattern: $pattern):"
              echo "$matches"
              FAIL=$((FAIL + 1))
            fi
          done
          [[ $FAIL -eq 0 ]] || exit 1
          echo "Secret scan: PASS"

      - name: Install cargo-audit
        run: cargo install cargo-audit --locked

      - name: Run cargo audit
        run: cargo audit

  # ──────────────────────────────────────────────────────────────────────────
  # Job 5: Benchmark Regression Gate (Stage 3)
  # Runs on PRs to main and pushes to main.
  # Compares against stored baseline; fails on >10% regression.
  # ──────────────────────────────────────────────────────────────────────────
  bench:
    name: "Benchmark Regression Gate"
    runs-on: ubuntu-latest
    # Only run on PRs to main or pushes to main (not every feature branch PR)
    if: |
      github.ref == 'refs/heads/main' ||
      (github.event_name == 'pull_request' && github.base_ref == 'main') ||
      github.event.inputs.bench_only == 'true' ||
      github.event_name == 'schedule'
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain (nightly)
        uses: dtolnay/rust-toolchain@nightly

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-registry-

      - name: Cache build target (release)
        uses: actions/cache@v4
        with:
          path: target/
          key: ${{ runner.os }}-cargo-target-nightly-bench-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-target-nightly-bench-

      - name: Download baseline artifact (if exists)
        uses: actions/download-artifact@v4
        with:
          name: bench-baseline
          path: bench-baseline/
        continue-on-error: true  # First run has no baseline yet

      - name: Run benchmarks
        run: cargo bench --bench receipt_operations -- --output-format bencher | tee bench-output.txt

      - name: Run Criterion benchmarks (HTML report)
        run: cargo bench --bench receipt_operations
        env:
          # Criterion saves results under target/criterion
          CRITERION_HOME: target/criterion

      - name: Compare against baseline
        id: bench-compare
        run: |
          if [[ -d bench-baseline ]]; then
            bash scripts/bench-compare.sh 2>&1 | tee bench-report.md
            echo "comparison_done=true" >> "$GITHUB_OUTPUT"
          else
            echo "No baseline found — this run will establish the baseline."
            echo "comparison_done=false" >> "$GITHUB_OUTPUT"
          fi
        continue-on-error: true  # capture exit code below

      - name: Fail on regression
        if: steps.bench-compare.outcome == 'failure'
        run: |
          echo "FAIL: Benchmark regression gate failed. See bench-report.md for details."
          exit 1

      - name: Post benchmark report as PR comment
        if: github.event_name == 'pull_request' && steps.bench-compare.outputs.comparison_done == 'true'
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const fs = require('fs');
            const report = fs.existsSync('bench-report.md')
              ? fs.readFileSync('bench-report.md', 'utf8')
              : '_No benchmark report generated._';
            const body = `## Benchmark Regression Report\n\n${report}\n\n_Triggered by commit ${{ github.sha }}_`;
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body
            });

      - name: Upload Criterion HTML report
        uses: actions/upload-artifact@v4
        with:
          name: criterion-report-${{ github.sha }}
          path: target/criterion/
          retention-days: 30

      - name: Upload new baseline (on main push only)
        if: github.ref == 'refs/heads/main' && github.event_name == 'push'
        uses: actions/upload-artifact@v4
        with:
          name: bench-baseline
          path: target/criterion/
          retention-days: 90
          overwrite: true

  # ──────────────────────────────────────────────────────────────────────────
  # Job 6: PR Summary Comment
  # Posts a unified test summary comment on every PR.
  # ──────────────────────────────────────────────────────────────────────────
  pr-summary:
    name: "PR Test Summary"
    runs-on: ubuntu-latest
    needs: [validate, feature-matrix, doc-gate, security]
    if: always() && github.event_name == 'pull_request'
    steps:
      - name: Post PR summary
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const jobs = {
              validate:       '${{ needs.validate.result }}',
              featureMatrix:  '${{ needs.feature-matrix.result }}',
              docGate:        '${{ needs.doc-gate.result }}',
              security:       '${{ needs.security.result }}',
            };
            const icon = r => r === 'success' ? '✅' : r === 'skipped' ? '⏭️' : '❌';
            const lines = [
              '## CI Pipeline Summary',
              '',
              '| Stage | Status |',
              '|---|---|',
              `| Validate (nightly + beta) | ${icon(jobs.validate)} ${jobs.validate} |`,
              `| Feature Matrix (11 combos) | ${icon(jobs.featureMatrix)} ${jobs.featureMatrix} |`,
              `| Documentation Gate | ${icon(jobs.docGate)} ${jobs.docGate} |`,
              `| Security Scan | ${icon(jobs.security)} ${jobs.security} |`,
              '',
              `_Commit: ${{ github.sha }}_`,
            ];
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: lines.join('\n'),
            });

  # ──────────────────────────────────────────────────────────────────────────
  # Job 7: Release Pipeline (Stage 7)
  # Manual trigger only. Requires release/* branch and passing main CI.
  # ──────────────────────────────────────────────────────────────────────────
  release:
    name: "Release Pipeline"
    runs-on: ubuntu-latest
    if: |
      github.event_name == 'workflow_dispatch' &&
      github.event.inputs.release == 'true' &&
      startsWith(github.ref, 'refs/heads/release/')
    environment: crates-io  # requires crates.io token in environment secrets
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # full history for tag verification

      - name: Install Rust toolchain (nightly)
        uses: dtolnay/rust-toolchain@nightly

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Get version from Cargo.toml
        id: version
        run: |
          VERSION=$(cargo metadata --no-deps --format-version 1 | \
            python3 -c "import json,sys; d=json.load(sys.stdin); print(d['packages'][0]['version'])")
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"
          echo "Release version: $VERSION"

      - name: Verify CHANGELOG entry exists
        run: |
          grep -qF "[${{ steps.version.outputs.version }}]" CHANGELOG.md || {
            echo "FAIL: CHANGELOG.md missing entry for v${{ steps.version.outputs.version }}"
            exit 1
          }

      - name: Run full CI pre-flight
        run: |
          cargo fmt --check
          cargo clippy --all-targets -- -D warnings
          cargo test
          RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

      - name: Dry-run publish
        run: cargo publish --dry-run
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Extract CHANGELOG notes for this release
        id: notes
        run: |
          VERSION="${{ steps.version.outputs.version }}"
          NOTES=$(awk "/^## \[$VERSION\]/,/^## \[/{if (/^## \[/ && !/^## \[$VERSION\]/) exit; print}" CHANGELOG.md | tail -n +2)
          echo "notes<<EOF" >> "$GITHUB_OUTPUT"
          echo "$NOTES" >> "$GITHUB_OUTPUT"
          echo "EOF" >> "$GITHUB_OUTPUT"

      - name: Create git tag
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git tag -a "v${{ steps.version.outputs.version }}" \
            -m "Release v${{ steps.version.outputs.version }}"
          git push origin "v${{ steps.version.outputs.version }}"

      - name: Create GitHub release
        run: |
          gh release create "v${{ steps.version.outputs.version }}" \
            --title "affidavit v${{ steps.version.outputs.version }}" \
            --notes "${{ steps.notes.outputs.notes }}" \
            --verify-tag
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Publish to crates.io
        run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

---

## 10. Local Development Makefile

Create `Makefile` at the repo root:

```makefile
# affidavit Makefile
# Targets mirror CI pipeline stages for local parity.
# Usage: make <target>

CARGO     := cargo
AFFI      := cargo run --bin affi --
SHELL_CMD := /bin/bash

.PHONY: all check fmt lint test bench doc golden-run clean install-hooks \
        feature-matrix security release-dry audit

## ── Default ──────────────────────────────────────────────────────────────────

all: check fmt lint test doc
	@echo "All local checks passed."

## ── Stage 2: PR Validation ───────────────────────────────────────────────────

# Fast dependency + type check (no codegen)
check:
	$(CARGO) check --all-features

# Run format check (mirrors CI)
fmt:
	$(CARGO) fmt --check

# Apply formatting in-place
fmt-fix:
	$(CARGO) fmt

# Lint with clippy (all targets, deny warnings)
lint:
	$(CARGO) clippy --all-targets -- -D warnings

# Full test suite (30+ tests)
test:
	$(CARGO) test

# Test with otel feature
test-otel:
	$(CARGO) test --features otel

# Test library unit tests only (fast, ~5s)
test-lib:
	$(CARGO) test --lib

# Test with all features enabled
test-all-features:
	$(CARGO) test --all-features

## ── Stage 3: Benchmarks ──────────────────────────────────────────────────────

# Run the full benchmark suite
bench:
	$(CARGO) bench --bench receipt_operations

# Run benchmarks and open Criterion HTML report
bench-open:
	$(CARGO) bench --bench receipt_operations
	@if command -v xdg-open >/dev/null 2>&1; then \
	    xdg-open target/criterion/report/index.html; \
	elif command -v open >/dev/null 2>&1; then \
	    open target/criterion/report/index.html; \
	fi

## ── Stage 4: Feature Matrix ──────────────────────────────────────────────────

# Test every feature combination
feature-matrix:
	$(SHELL_CMD) scripts/feature-matrix-test.sh

## ── Stage 5: Documentation ───────────────────────────────────────────────────

# Build docs (deny doc warnings)
doc:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --all-features --no-deps

# Build docs and open in browser
doc-open: doc
	@if command -v xdg-open >/dev/null 2>&1; then \
	    xdg-open target/doc/affidavit/index.html; \
	elif command -v open >/dev/null 2>&1; then \
	    open target/doc/affidavit/index.html; \
	fi

# Run all shell examples against the built binary
examples-run:
	$(SHELL_CMD) scripts/run-examples.sh

# Check CHANGELOG and CLAUDE.md freshness
doc-check:
	@VERSION=$$($(CARGO) metadata --no-deps --format-version 1 | \
	    python3 -c "import json,sys; d=json.load(sys.stdin); print(d['packages'][0]['version'])"); \
	grep -qF "[$$VERSION]" CHANGELOG.md || { \
	    echo "FAIL: CHANGELOG.md missing entry for v$$VERSION"; exit 1; }; \
	echo "CHANGELOG check passed for v$$VERSION"
	$(SHELL_CMD) scripts/check-claude-md.sh

## ── Stage 6: Security ────────────────────────────────────────────────────────

# Run secret scan + cargo audit
security:
	$(SHELL_CMD) scripts/secret-scan.sh
	$(CARGO) audit

# Install cargo-audit
install-audit:
	$(CARGO) install cargo-audit --locked

## ── Golden Run ───────────────────────────────────────────────────────────────

# Full lifecycle smoke test: emit → assemble → verify (honest + tampered)
golden-run:
	$(SHELL_CMD) examples/golden_run.sh

## ── Release ──────────────────────────────────────────────────────────────────

# Dry-run publish to crates.io (does not publish)
release-dry:
	$(CARGO) publish --dry-run

## ── Hooks ────────────────────────────────────────────────────────────────────

# Install pre-commit and pre-push hooks (one-time setup after clone)
install-hooks:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit .githooks/pre-push
	@echo "Git hooks installed. Hooks will run from .githooks/"

## ── Cleanup ──────────────────────────────────────────────────────────────────

# Clean build artifacts
clean:
	$(CARGO) clean

# Clean and rebuild from scratch
rebuild: clean all

## ── Help ─────────────────────────────────────────────────────────────────────

help:
	@echo ""
	@echo "affidavit local development targets:"
	@echo ""
	@echo "  make check           cargo check --all-features"
	@echo "  make fmt             cargo fmt --check"
	@echo "  make fmt-fix         cargo fmt (apply in-place)"
	@echo "  make lint            cargo clippy --all-targets -D warnings"
	@echo "  make test            cargo test (all 30+ tests)"
	@echo "  make test-lib        cargo test --lib (unit tests only, fast)"
	@echo "  make test-all-features  cargo test --all-features"
	@echo "  make bench           cargo bench --bench receipt_operations"
	@echo "  make bench-open      bench + open Criterion HTML report"
	@echo "  make feature-matrix  Test all 11 feature combinations"
	@echo "  make doc             Build docs (deny warnings)"
	@echo "  make doc-open        Build docs and open in browser"
	@echo "  make doc-check       Check CHANGELOG + CLAUDE.md freshness"
	@echo "  make examples-run    Run all examples/*.sh scripts"
	@echo "  make security        Secret scan + cargo audit"
	@echo "  make golden-run      Full emit→assemble→verify smoke test"
	@echo "  make release-dry     cargo publish --dry-run"
	@echo "  make install-hooks   Install pre-commit and pre-push hooks"
	@echo "  make clean           cargo clean"
	@echo "  make all             check + fmt + lint + test + doc"
	@echo ""
```

---

## 11. DoD: CI Integration Checklist

The DX/QOL 1000x initiative is **done** when every item in this checklist passes. This checklist is the authoritative gate for merging `claude/zen-cerf-oq87br` into `main`.

### 11.1 Infrastructure: Files In Place

- [ ] `.github/workflows/ci.yml` exists and parses without YAML errors
- [ ] `.githooks/pre-commit` exists and is executable
- [ ] `.githooks/pre-push` exists and is executable
- [ ] `Makefile` exists with all targets defined in §10
- [ ] `scripts/bench-compare.sh` exists and is executable
- [ ] `scripts/feature-matrix-test.sh` exists and is executable
- [ ] `scripts/run-examples.sh` exists and is executable
- [ ] `scripts/secret-scan.sh` exists and is executable
- [ ] `scripts/check-claude-md.sh` exists and is executable

### 11.2 Stage 1: Pre-commit Hooks

- [ ] `make install-hooks` runs without error
- [ ] `git config core.hooksPath` returns `.githooks`
- [ ] Pre-commit hook blocks a commit with unformatted code
- [ ] Pre-commit hook blocks a commit with a clippy warning
- [ ] Pre-commit hook blocks a commit when `cargo test --lib` fails
- [ ] Pre-push hook runs the full test suite before push
- [ ] Pre-push hook runs `golden_run.sh` before push

### 11.3 Stage 2: PR Validation (all must be green)

- [ ] `cargo check --all-features` — exits 0
- [ ] `cargo fmt --check` — exits 0 (no unformatted files)
- [ ] `cargo clippy --all-targets -- -D warnings` — exits 0 (zero warnings)
- [ ] `cargo test` — all 30 tests pass (19 lib + 6 dispatch + 4 e2e + 1 ui)
- [ ] `cargo test --features otel` — passes
- [ ] `cargo test --features discovery,conformance,predictive` — passes
- [ ] `cargo test --features mutation,shell,json-output` — passes
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps` — exits 0
- [ ] `cargo build --release` — exits 0

### 11.4 Stage 3: Benchmark Regression Gate

- [ ] `cargo bench --bench receipt_operations` runs and produces output
- [ ] Criterion HTML report is generated at `target/criterion/`
- [ ] `scripts/bench-compare.sh` exits 0 when no regression
- [ ] `scripts/bench-compare.sh` exits 1 on a simulated >10% regression (verified in test)
- [ ] GitHub Actions uploads Criterion HTML report as artifact
- [ ] Benchmark baseline artifact is uploaded on push to `main`
- [ ] PR to `main` downloads baseline and runs comparison
- [ ] PR comment with benchmark table is posted automatically

### 11.5 Stage 4: Feature Matrix

- [ ] All 11 feature combinations in the matrix compile without error
- [ ] All 11 feature combinations pass their respective test suites
- [ ] `scripts/feature-matrix-test.sh` exits 0 with PASS=11 FAIL=0

### 11.6 Stage 5: Documentation Gate

- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps` — zero doc warnings
- [ ] `examples/golden_run.sh` — exits 0 (ACCEPT + REJECT both work)
- [ ] All other `examples/*.sh` scripts — each exits 0
- [ ] `CHANGELOG.md` contains entry for current version (`26.6.17`)
- [ ] `CLAUDE.md` architecture section lists all files in `src/verbs/*.rs`
- [ ] `scripts/check-claude-md.sh` — exits 0

### 11.7 Stage 6: Security Scan

- [ ] `scripts/secret-scan.sh` — exits 0 (no secret patterns found)
- [ ] No assembled receipt JSON in `tests/` stores raw payload bytes
- [ ] All event `commitment` fields in fixture receipts are 64-char lowercase hex (BLAKE3)
- [ ] `cargo audit` — exits 0 (no known vulnerabilities)

### 11.8 Stage 7: Release Readiness

- [ ] `cargo publish --dry-run` — exits 0
- [ ] Version in `Cargo.toml` matches intended release tag
- [ ] `CHANGELOG.md` entry for current version contains complete release notes
- [ ] `CARGO_REGISTRY_TOKEN` secret is configured in the `crates-io` GitHub Actions environment
- [ ] Release workflow only triggers on `release/*` branches with manual dispatch

### 11.9 GitHub Actions: Pipeline Health

- [ ] `validate` job passes on both `nightly` and `beta` Rust toolchains
- [ ] `feature-matrix` job passes all 11 matrix entries
- [ ] `doc-gate` job passes
- [ ] `security` job passes
- [ ] `bench` job runs on PRs to `main` and posts a comment
- [ ] `pr-summary` job posts a unified status table on every PR
- [ ] No workflow uses `continue-on-error: true` on the nightly toolchain for blocking steps
- [ ] All jobs use pinned `actions/cache@v4`, `actions/checkout@v4`, `dtolnay/rust-toolchain`

### 11.10 DX/QOL 1000x Feature Coverage

- [ ] Phase 1 (Receipt Inspection): all 5 features implemented, tested, documented
- [ ] Phase 2 (Process Discovery & Conformance): all features behind `discovery`/`conformance`/`predictive` gates
- [ ] Phase 3 (Benchmarking & Regression): Criterion benches pass, `bench` CI job active
- [ ] Phase 4 (Test Generation & Mutation): `mutation` feature gate compiles and tests pass
- [ ] Phase 5 (Observability): `otel`/`metrics` feature gates compile and tests pass
- [ ] All 22 features listed in `DX_QOL_IMPLEMENTATION_CHECKLIST.md` have at least one green test
- [ ] `make all` runs cleanly in a fresh checkout (after `make install-hooks`)
- [ ] `make golden-run` exits 0 from a clean temp dir
- [ ] `make bench` produces Criterion output in `target/criterion/`

### 11.11 The Single Merge Criterion

> **The `claude/zen-cerf-oq87br` branch may merge to `main` if and only if every checkbox above is ticked AND the GitHub Actions CI pipeline shows all green on the PR.**

No exceptions. The verifier certifies; it does not decide honesty.

---

## Appendix A: Required GitHub Repository Settings

Configure in `https://github.com/seanchatmangpt/affidavit/settings/branches`:

```
Branch protection rule for: main
  ✅ Require a pull request before merging
  ✅ Require status checks to pass before merging
    Required status checks:
      - validate (nightly)
      - validate (beta)
      - feature-matrix (all 11 entries)
      - doc-gate
      - security
  ✅ Require branches to be up to date before merging
  ✅ Require conversation resolution before merging
  ✅ Do not allow bypassing the above settings
```

## Appendix B: Secrets Required

| Secret name | Scope | Purpose |
|---|---|---|
| `GITHUB_TOKEN` | Auto-provided | PR comments, release creation |
| `CARGO_REGISTRY_TOKEN` | `crates-io` environment | `cargo publish` |

## Appendix C: Toolchain Version Matrix

| Toolchain | Purpose | Status |
|---|---|---|
| `nightly` (from `rust-toolchain.toml`) | Primary — matches local dev | Required |
| `beta` | Forward-compat check — catches nightly-only syntax leaks | Advisory (continue-on-error for new feature flags) |
| `stable` | Not tested — project uses nightly features | Not applicable |

---

*Last updated: 2026-06-14 | affidavit v26.6.17 | DX/QOL 1000x initiative*
