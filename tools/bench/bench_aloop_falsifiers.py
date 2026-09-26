#!/usr/bin/env python3
"""Deterministic benchmark for tools/aloop_falsifiers.py (check + dag).

Builds synthetic, fully CONFORMANT ALOOP cases (sha256 event chain, one receipt per
EVENTS_PER_RECEIPT events, profile-valid receipts) at several sizes on real disk, runs
the real check_case / build_dag+detect_cycles+detect_bridges, and reports best-of-K
wall time and process CPU time (the regression bound in tools/tests uses CPU time). The case content is a pure function of N (byte-identical across runs), so
only the timing varies.

Usage:
    bench_aloop_falsifiers.py [--sizes 1000,10000,50000] [--repeat 3] [--out FILE]
Exit 1 if any case is not CONFORMANT (a benchmark of a refusing verifier is void).
"""
import argparse, json, platform, sys, tempfile, time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
import aloop_falsifiers as A  # noqa: E402

EVENTS_PER_RECEIPT = 10
SUBJECT = A.GOLDEN_SUBJECT
GRANT = "operator-dispatch ALOOP-BENCH"


def synth_case(n):
    events, receipts, prev = [], [], None
    for i in range(n):
        wo = f"ALOOP-BENCH/wo-{i // EVENTS_PER_RECEIPT:06d}"
        e = {"seq": i, "event_id": f"e{i}", "event_type": "actuation", "work_order_id": wo,
             "actor": "bench", "authority_grant": GRANT, "provider": "zcode",
             "provider_execution_id": f"run-{i // EVENTS_PER_RECEIPT:06d}",
             "subject_sha": SUBJECT, "actuation_id": f"act-{i:07d}", "cmd": f"step {i}",
             "consequence_hash": f"{i:064x}", "ts": "2026-09-25T09:00:00Z",
             "recorded_at": "2026-09-25T09:00:01Z", "hash_algo": "sha256", "prev_hash": prev}
        e["hash"] = A.canonical(e)
        prev = e["hash"]
        events.append(e)
    for w in range(0, n, EVENTS_PER_RECEIPT):
        chunk = events[w:w + EVENTS_PER_RECEIPT]
        wo = chunk[0]["work_order_id"]
        cmd = [{"cmd": "bench", "exit": 0, "cwd": "."}]
        r = {"work_order_id": wo,
             "origin_authority": {"ceiling": "DO", "grant": GRANT, "actor": "bench"},
             "provider": {"name": "zcode"}, "provider_execution_id": chunk[0]["provider_execution_id"],
             "identity": {"subject": wo, "repo": "affidavit", "subject_sha": SUBJECT, "base_sha": SUBJECT},
             "authority": {"ceiling": "DO", "grant": GRANT, "actor": "bench"},
             "consequence": {"commits": [], "files_changed": [], "remote_effects": []},
             "consequences": [{"hash": e["consequence_hash"], "kind": "artifact"} for e in chunk],
             "replay": {"commands": cmd}, "commands": cmd,
             "evidence": [{"kind": "falsifier", "verifier": "bench", "exit": 0}],
             "exit_status": "ok",
             "timestamps": {"started_at": "2026-09-25T09:00:00Z", "finished_at": "2026-09-25T09:00:01Z"},
             "standing": {"value": "PARTIAL_ALIVE", "derived_from": "bench"},
             "replay_binding": {"event_ids": [e["event_id"] for e in chunk],
                                "chain_head_hash": chunk[-1]["hash"], "chain_hash_algo": "sha256"}}
        if receipts:
            r["replay_binding"]["predecessor_work_order_ids"] = [receipts[-1]["work_order_id"]]
        receipts.append(r)
    return receipts, events


def best(fn, repeat):
    """best-of-K (wall seconds, process CPU seconds). CPU time is the regression-bound
    quantity: it is insensitive to host contention from other processes, wall is not."""
    walls, cpus, out = [], [], None
    for _ in range(repeat):
        t, c = time.perf_counter(), time.process_time()
        out = fn()
        cpus.append(time.process_time() - c)
        walls.append(time.perf_counter() - t)
    return min(walls), min(cpus), out


def run(sizes, repeat, validate_profile=True, root=None):
    root = Path(root or tempfile.mkdtemp(prefix="aloop-bench-"))
    rows = []
    for n in sizes:
        d = root / f"n{n}"
        A.write_case(d, *synth_case(n))
        t_check, c_check, rep = best(lambda: A.check_case(d, validate_profile=validate_profile), repeat)
        if rep["verdict"] != "CONFORMANT":
            raise SystemExit(f"bench case n={n} not CONFORMANT: {rep['violations'][:3]}")

        def dag():
            g = A.build_dag(d)
            return A.detect_cycles(g), A.detect_bridges(g)
        t_dag, c_dag, (cyc, _) = best(dag, repeat)
        assert cyc["acyclic"]
        rows.append({"events": n, "receipts": rep["receipts"],
                     "check_s": round(t_check, 4), "dag_s": round(t_dag, 4),
                     "check_us_per_event": round(t_check / n * 1e6, 2),
                     "dag_us_per_event": round(t_dag / n * 1e6, 2),
                     "check_cpu_s": round(c_check, 4), "dag_cpu_s": round(c_dag, 4),
                     "check_cpu_us_per_event": round(c_check / n * 1e6, 2),
                     "dag_cpu_us_per_event": round(c_dag / n * 1e6, 2)})
    return rows


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--sizes", default="1000,10000,50000")
    ap.add_argument("--repeat", type=int, default=3)
    ap.add_argument("--no-profile", action="store_true")
    ap.add_argument("--out")
    a = ap.parse_args(argv)
    rows = run([int(x) for x in a.sizes.split(",")], a.repeat, validate_profile=not a.no_profile)
    report = {"tool": "tools/aloop_falsifiers.py", "events_per_receipt": EVENTS_PER_RECEIPT,
              "repeat": a.repeat, "profile_validation": not a.no_profile,
              "python": platform.python_version(), "machine": platform.machine(),
              "system": platform.system(), "rows": rows}
    out = json.dumps(report, indent=2)
    if a.out:
        Path(a.out).write_text(out + "\n")
    print(out)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
