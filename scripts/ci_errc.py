#!/usr/bin/env python3
"""Exact-head ERRC routing and fast-admission receipt for Affidavit.

Reconstituted from:
  seanchatmangpt/ggen-legacy@60d38265b8d1d94c43f04ca6bdb8537184e510a8
  scripts/ci_errc.py

This fast court never crowns the repository ALIVE. It certifies exact-head
identity, deterministic changed-file routing, and parseable changed JSON/TOML,
then emits PARTIAL_ALIVE when those bounded checks pass. Heavy execution remains
owned by the routed repository courts.
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import os
import subprocess
import sys
import time
import tomllib
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]

SOURCE = {
    "repository": "seanchatmangpt/ggen-legacy",
    "commit": "60d38265b8d1d94c43f04ca6bdb8537184e510a8",
    "artifact": "scripts/ci_errc.py",
}

CLAIM_CEILING = "EXACT_HEAD_ROUTING_AND_STRUCTURED_ADMISSION_ONLY"

ERRC = {
    "eliminate": [
        "ambient inference that a green workflow executed the PR head",
        "untyped changed-file routing",
        "cross-unit aggregation disguised as optimization evidence",
    ],
    "reduce": [
        "time-to-first-falsifier before expensive repository courts",
        "unnecessary heavy-lane execution for unrelated surfaces",
    ],
    "raise": [
        "exact-subject identity and replay visibility",
        "typed failure visibility and deterministic evidence ownership",
        "preservation of certify-dont-decide boundaries",
    ],
    "create": [
        "machine-readable ERRC fast-court receipt",
        "replayable path-to-evidence-lane classification",
        "formal ERRC domain receipt in src/errc.rs",
    ],
}

# A path can route to multiple courts when it crosses boundaries. That is
# deliberate: one failed edge is topology, not global graph failure.
LANE_RULES: dict[str, dict[str, tuple[str, ...]]] = {
    "root_rust": {
        "include": (
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "src/**",
            "tests/**",
            "benches/**",
            "examples/**",
            "stubs/**",
            ".github/workflows/rust.yml",
        ),
        "exclude": ("src/verbs/**",),
    },
    "generation": {
        "include": (
            "ggen.toml",
            "ontology/**",
            ".ggen/**",
            "src/verbs/**",
        ),
        "exclude": (),
    },
    "affidavit_core": {
        "include": (
            "affidavit-core/**",
            ".github/workflows/affidavit-core.yml",
        ),
        "exclude": (),
    },
    "web": {
        "include": (
            "web/**",
            ".github/workflows/web.yml",
        ),
        "exclude": (),
    },
    "confevo": {
        "include": (
            "tools/confevo/**",
            ".github/workflows/confevo.yml",
        ),
        "exclude": (),
    },
    "governance": {
        "include": (
            "AGENTS.md",
            "README.md",
            "STATUS.md",
            "CHANGELOG.md",
            "CONTRIBUTING.md",
            "CLAUDE.md",
            "docs/**",
            "scripts/**",
            ".github/workflows/errc.yml",
        ),
        "exclude": ("scripts/tests/**",),
    },
}


def _matches(path: str, patterns: Iterable[str]) -> bool:
    return any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)


def classify_path(path: str) -> list[str]:
    """Return deterministic evidence lanes owning *path*."""
    lanes: list[str] = []
    for lane, rules in LANE_RULES.items():
        if _matches(path, rules["include"]) and not _matches(path, rules["exclude"]):
            lanes.append(lane)
    return lanes


def classify_paths(paths: Iterable[str]) -> dict[str, list[str]]:
    """Partition changed paths into owned lanes plus an explicit fast-only set."""
    result = {lane: [] for lane in LANE_RULES}
    result["fast_only"] = []
    for path in sorted(set(paths)):
        lanes = classify_path(path)
        if not lanes:
            result["fast_only"].append(path)
        for lane in lanes:
            result[lane].append(path)
    return result


def classify_fast_standing(failures: Iterable[dict[str, object]]) -> str:
    """Map fast-court failures to standing without overclaiming source breakage.

    A structured source artifact that fails admission establishes BUILD_BROKEN at
    this bounded court. Exact-head mismatch and change-discovery failures mean
    the intended subject was not lawfully inspected, so the court is BLOCKED.
    Success is capped at PARTIAL_ALIVE; this court never returns ALIVE.
    """
    codes = {str(failure.get("failure", "")) for failure in failures}
    if not codes:
        return "PARTIAL_ALIVE"
    if "STRUCTURED_FILE_INVALID" in codes:
        return "BUILD_BROKEN"
    return "BLOCKED"


def git_changed_files(root: Path, base: str, head: str) -> list[str]:
    """Discover ACMR changes between the admitted base and exact candidate."""
    if base and base != head:
        command = ["git", "diff", "--name-only", "--diff-filter=ACMR", f"{base}...{head}"]
    else:
        command = ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", head]
    completed = subprocess.run(
        command,
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            "CHANGED_FILE_DISCOVERY_FAILED: "
            + (completed.stderr.strip() or completed.stdout.strip() or str(command))
        )
    return [line.strip() for line in completed.stdout.splitlines() if line.strip()]


def exact_head_check(root: Path, expected_head: str) -> dict[str, object]:
    """Observe local HEAD and bind the fast receipt to the exact candidate."""
    started = time.monotonic()
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed_ms = round((time.monotonic() - started) * 1000)
    observed = completed.stdout.strip()
    passed = completed.returncode == 0 and observed == expected_head
    result: dict[str, object] = {
        "id": "exact-head",
        "command": ["git", "rev-parse", "HEAD"],
        "expected": expected_head,
        "observed": observed or None,
        "exit_code": completed.returncode,
        "elapsed_ms": elapsed_ms,
        "passed": passed,
    }
    if not passed:
        result["failure"] = "EXACT_HEAD_MISMATCH"
        if completed.stderr:
            result["stderr_tail"] = completed.stderr[-4000:]
    return result


def validate_structured_files(root: Path, changed: Iterable[str]) -> list[dict[str, object]]:
    """Parse changed JSON/TOML files without inventing validators for other formats."""
    checks: list[dict[str, object]] = []
    for rel in sorted(set(changed)):
        path = root / rel
        if not path.is_file():
            continue
        try:
            if path.suffix == ".json":
                json.loads(path.read_text(encoding="utf-8"))
            elif path.suffix == ".toml":
                tomllib.loads(path.read_text(encoding="utf-8"))
            else:
                continue
            checks.append({"id": f"parse:{rel}", "passed": True})
        except Exception as exc:
            checks.append(
                {
                    "id": f"parse:{rel}",
                    "passed": False,
                    "failure": "STRUCTURED_FILE_INVALID",
                    "detail": str(exc),
                }
            )
    return checks


def write_github_outputs(path: Path, routing: dict[str, list[str]], standing: str) -> None:
    lines = [f"standing={standing}"]
    for lane in LANE_RULES:
        lines.append(f"{lane}={'true' if routing[lane] else 'false'}")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_summary(report: dict[str, object]) -> None:
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return
    routing = report["routing"]
    deep = [lane for lane in LANE_RULES if routing.get(lane)]
    with Path(summary_path).open("a", encoding="utf-8") as handle:
        handle.write("## Affidavit ERRC fast court\n\n")
        handle.write(f"**Standing:** `{report['standing']}`  \n")
        handle.write(f"**Claim ceiling:** `{report['claim_ceiling']}`  \n")
        handle.write(f"**Changed files:** `{len(report['changed_files'])}`  \n")
        handle.write(
            "**Evidence lanes:** "
            + (", ".join(f"`{lane}`" for lane in deep) if deep else "`fast_only`")
            + "\n"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--base", default="")
    parser.add_argument("--head", required=True)
    parser.add_argument("--changed-file", action="append", default=[])
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--github-output", type=Path)
    parser.add_argument(
        "--skip-exact-head",
        action="store_true",
        help="Tests only: do not observe git HEAD.",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    checks: list[dict[str, object]] = []

    if not args.skip_exact_head:
        checks.append(exact_head_check(root, args.head))

    try:
        changed = args.changed_file or git_changed_files(root, args.base, args.head)
        discovery_error = None
    except RuntimeError as exc:
        changed = []
        discovery_error = str(exc)

    routing = classify_paths(changed)
    checks.extend(validate_structured_files(root, changed))

    if discovery_error:
        checks.append(
            {
                "id": "changed-file-discovery",
                "passed": False,
                "failure": "CHANGED_FILE_DISCOVERY_FAILED",
                "detail": discovery_error,
            }
        )

    failures = [check for check in checks if not check.get("passed", False)]
    standing = classify_fast_standing(failures)

    report: dict[str, object] = {
        "schema": "affidavit.ci.errc.fast-receipt.v1",
        "source": SOURCE,
        "subject": {
            "repository": os.environ.get("GITHUB_REPOSITORY", "local/affidavit"),
            "base": args.base or None,
            "head": args.head,
            "workflow": os.environ.get("GITHUB_WORKFLOW"),
            "run_id": os.environ.get("GITHUB_RUN_ID"),
        },
        "execution_environment": {
            "python": sys.version.split()[0],
            "runner_os": os.environ.get("RUNNER_OS"),
            "runner_arch": os.environ.get("RUNNER_ARCH"),
            "runner_image": os.environ.get("ImageOS"),
        },
        "errc": ERRC,
        "changed_files": sorted(set(changed)),
        "routing": routing,
        "checks": checks,
        "failures": failures,
        "standing": standing,
        "claim_ceiling": CLAIM_CEILING,
        "replay": {
            "command": (
                f"python3 scripts/ci_errc.py --base {args.base or '<base>'} "
                f"--head {args.head} --report evidence/ci/errc-fast.json"
            )
        },
    }

    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2, sort_keys=True))

    if args.github_output:
        write_github_outputs(args.github_output, routing, standing)
    write_summary(report)

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
