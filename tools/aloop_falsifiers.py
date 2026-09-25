#!/usr/bin/env python3
"""ZeroUnreceiptedActuation falsifiers + causal-DAG derivation for ALOOP execution
receipts (dfcm R v2 profile, schemas/aloop-execution-receipt.schema.json).

Operates over a case directory:
    receipts/*.json   dfcm R v2 receipts (ALOOP fields: work_order_id,
                      origin_authority, provider, provider_execution_id, consequence,
                      replay_binding.{event_ids,predecessor_work_order_ids})
    events.ndjson     OCEL event chain, one JSON object per line, fields:
                      seq, event_id, event_type, work_order_id, actor, authority_grant,
                      provider, provider_execution_id, subject_sha, actuation_id, cmd,
                      consequence_hash, ts, recorded_at, reconstructed?, hash_algo,
                      prev_hash, hash
                      hash = sha256(canonical-json(event minus hash)), prev_hash links
                      the previous seq; logical seq, never wall-clock (repo law).

The eleven falsifiers, each naming the broken term of the dfcm failure taxonomy:
    F01 missing_event             gap in the seq chain              R_missing_replay
    F02 reordered_event           file order != seq order           mu_on_O
    F03 duplicate_actuation       one actuation_id run twice        R_missing_authority
    F04 duplicate_consequence     one consequence, two actuations   admission_vacuous
    F05 broken_chain              hash/prev_hash linkage fails      R_missing_identity
    F06 receipt_without_consequence                                 R_missing_consequence
    F07 consequence_without_receipt (unreceipted actuation)         mu_unlawful
    F08 provider_identity_rewrite same peid, two provider names     R_missing_identity
    F09 subject_mismatch          event vs receipt subject_sha      R_missing_identity
    F10 authority_mismatch        event vs receipt origin_authority R_missing_authority
    F11 posthoc_fabricated        recorded before it claims to have
                                  happened, unmarked                R_not_fed_back
An event honestly marked reconstructed:true is not a violation; it is counted and
reported separately (RECONSTRUCTED), so the honest count survives.

An affidavit certifies what occurred; it NEVER decides planning policy.

Usage:
    aloop_falsifiers.py check <case_dir>
    aloop_falsifiers.py dag <case_dir> [--out FILE]    # DAG + cycle/bridge detection
    aloop_falsifiers.py emit-golden <case_dir>         # deterministic golden case
    aloop_falsifiers.py self-test [tmpdir]             # witness every refusal
"""
import copy, json, hashlib, shutil, subprocess, sys, tempfile
from pathlib import Path

RECEIPT_REQUIRED = ("work_order_id", "origin_authority", "provider", "provider_execution_id")
FALSIFIERS = {
    "F01": ("missing_event", "R_missing_replay"),
    "F02": ("reordered_event", "mu_on_O"),
    "F03": ("duplicate_actuation", "R_missing_authority"),
    "F04": ("duplicate_consequence", "admission_vacuous"),
    "F05": ("broken_chain", "R_missing_identity"),
    "F06": ("receipt_without_consequence", "R_missing_consequence"),
    "F07": ("consequence_without_receipt", "mu_unlawful"),
    "F08": ("provider_identity_rewrite", "R_missing_identity"),
    "F09": ("subject_mismatch", "R_missing_identity"),
    "F10": ("authority_mismatch", "R_missing_authority"),
    "F11": ("posthoc_fabricated", "R_not_fed_back"),
}


# ---------------------------------------------------------------- case io

def canonical(ev):
    payload = {k: v for k, v in ev.items() if k != "hash"}
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    algo = ev.get("hash_algo", "sha256")
    return hashlib.sha256(blob.encode()).hexdigest() if algo == "sha256" else None


def load_case(case_dir):
    case_dir = Path(case_dir)
    receipts, parse_errors = [], []
    rdir = case_dir / "receipts"
    if rdir.is_dir():
        for p in sorted(rdir.glob("*.json")):
            try:
                receipts.append(json.loads(p.read_text()))
            except json.JSONDecodeError as e:
                parse_errors.append(f"receipt {p.name}: {e}")
    events, order = [], []
    efile = case_dir / "events.ndjson"
    if efile.exists():
        for i, line in enumerate(efile.read_text().splitlines()):
            line = line.strip()
            if not line:
                continue
            try:
                events.append(json.loads(line))
                order.append(len(events) - 1)
            except json.JSONDecodeError as e:
                parse_errors.append(f"events.ndjson line {i + 1}: {e}")
    return receipts, events, order, parse_errors


def write_case(case_dir, receipts, events):
    case_dir = Path(case_dir)
    shutil.rmtree(case_dir, ignore_errors=True)
    (case_dir / "receipts").mkdir(parents=True)
    for r in receipts:
        (case_dir / "receipts" / f"{r['work_order_id'].replace('/', '_')}.json").write_text(
            json.dumps(r, indent=2, sort_keys=True) + "\n")
    with (case_dir / "events.ndjson").open("w") as f:
        for ev in events:
            f.write(json.dumps(ev, sort_keys=True) + "\n")


# ---------------------------------------------------------------- checks

def check_case(case_dir):
    receipts, events, order, parse_errors = load_case(case_dir)
    violations = list(parse_errors)
    for p in parse_errors:
        violations.append({"falsifier": "F05", "term": "R_missing_identity", "detail": p})
    if not events and not parse_errors:
        violations.append({"falsifier": "F01", "term": "R_missing_replay",
                           "detail": "no events in chain"})
    claimed = {}
    for r in receipts:
        for miss in [k for k in RECEIPT_REQUIRED if k not in r]:
            violations.append({"falsifier": "F08", "term": "R_missing_identity",
                               "detail": f"receipt missing {miss}"})
        for eid in r.get("replay_binding", {}).get("event_ids", []):
            claimed.setdefault(eid, []).append(r)

    by_seq = sorted(events, key=lambda e: e.get("seq", -1))
    # F01 missing event
    seqs = [e.get("seq", -1) for e in by_seq]
    for i in range(len(seqs) - 1):
        if seqs[i + 1] - seqs[i] > 1:
            violations.append({"falsifier": "F01", "term": "R_missing_replay",
                               "detail": f"seq gap {seqs[i]} -> {seqs[i + 1]}"})
    # F02 reordered
    file_seqs = [events[i].get("seq", -1) for i in order]
    if any(a > b for a, b in zip(file_seqs, file_seqs[1:])):
        violations.append({"falsifier": "F02", "term": "mu_on_O",
                           "detail": f"append order {file_seqs} is not seq-monotonic"})
    # F03 duplicate actuation
    seen = set()
    for e in events:
        aid = e.get("actuation_id")
        if aid in seen:
            violations.append({"falsifier": "F03", "term": "R_missing_authority",
                               "detail": f"actuation {aid} executed twice; second run has no fresh grant"})
        seen.add(aid)
    # F04 duplicate consequence across distinct actuations
    cons = {}
    for e in events:
        cons.setdefault(e.get("consequence_hash"), set()).add(e.get("actuation_id"))
    for ch, acts in cons.items():
        if ch and len(acts) > 1:
            violations.append({"falsifier": "F04", "term": "admission_vacuous",
                               "detail": f"consequence {ch[:12]}… claimed by {sorted(acts)}; receipts no longer discriminate"})
    # F05 broken chain
    prev = None
    for e in by_seq:
        if e.get("prev_hash") != prev:
            violations.append({"falsifier": "F05", "term": "R_missing_identity",
                               "detail": f"event {e.get('event_id')} prev_hash does not link predecessor"})
        if canonical(e) != e.get("hash"):
            violations.append({"falsifier": "F05", "term": "R_missing_identity",
                               "detail": f"event {e.get('event_id')} hash does not commit its payload"})
        prev = e.get("hash")
    # F06 receipt without consequence
    for r in receipts:
        c = r.get("consequence", {})
        empty_r = not (c.get("commits") or c.get("files_changed") or c.get("remote_effects"))
        empty_a = not r.get("consequences")
        if empty_r and empty_a:
            violations.append({"falsifier": "F06", "term": "R_missing_consequence",
                               "detail": f"receipt {r.get('work_order_id')} records no consequence"})
    # F07 consequence without receipt (unreceipted actuation; the UAR count)
    reconstructed = 0
    for e in events:
        if e.get("event_id") not in claimed:
            if e.get("reconstructed"):
                reconstructed += 1
                continue
            violations.append({"falsifier": "F07", "term": "mu_unlawful",
                               "detail": f"actuation event {e.get('event_id')} ({e.get('actuation_id')}) claimed by no receipt"})
    # F08 provider identity rewrite
    names = {}
    for e in events:
        names.setdefault(e.get("provider_execution_id"), set()).add(e.get("provider"))
    for r in receipts:
        names.setdefault(r.get("provider_execution_id"), set()).add(r.get("provider", {}).get("name"))
    for peid, ns in names.items():
        if len({n for n in ns if n}) > 1:
            violations.append({"falsifier": "F08", "term": "R_missing_identity",
                               "detail": f"provider_execution_id {peid} carried by providers {sorted(n for n in ns if n)}"})
    # F09 subject mismatch, F10 authority mismatch, F11 post-hoc fabrication
    for e in events:
        for r in claimed.get(e.get("event_id"), []):
            if e.get("subject_sha") and r.get("identity", {}).get("subject_sha") \
                    and e["subject_sha"] != r["identity"]["subject_sha"]:
                violations.append({"falsifier": "F09", "term": "R_missing_identity",
                                   "detail": f"event {e['event_id']} subject {e['subject_sha'][:12]}… != receipt {r['work_order_id']} subject {r['identity']['subject_sha'][:12]}…"})
            oa = r.get("origin_authority", {})
            if (e.get("actor"), e.get("authority_grant")) != (oa.get("actor"), oa.get("grant")):
                violations.append({"falsifier": "F10", "term": "R_missing_authority",
                                   "detail": f"event {e['event_id']} acted as ({e.get('actor')}, {e.get('authority_grant')}) but receipt {r['work_order_id']} records ({oa.get('actor')}, {oa.get('grant')})"})
        ts, rec = e.get("ts"), e.get("recorded_at")
        if ts and rec and rec < ts and not e.get("reconstructed"):
            violations.append({"falsifier": "F11", "term": "R_not_fed_back",
                               "detail": f"event {e.get('event_id')} recorded {rec} before its claimed occurrence {ts} with no reconstructed marking"})
    return {
        "case": str(case_dir),
        "events": len(events),
        "receipts": len(receipts),
        "violations": violations,
        "uar_count": sum(1 for v in violations if v["falsifier"] == "F07"),
        "reconstructed_unclaimed": reconstructed,
        "verdict": "CONFORMANT" if not violations else "REFUSED",
    }


# ---------------------------------------------------------------- DAG

def build_dag(case_dir):
    receipts, events, _, parse_errors = load_case(case_dir)
    nodes, edges = [], []
    for e in events:
        nodes.append({"id": f"E:{e['event_id']}", "kind": "event", "seq": e.get("seq"),
                      "work_order_id": e.get("work_order_id")})
    for r in receipts:
        nodes.append({"id": f"R:{r['work_order_id']}", "kind": "receipt",
                      "standing": r.get("standing", {}).get("value")})
    by_seq = sorted(events, key=lambda e: e.get("seq", -1))
    for a, b in zip(by_seq, by_seq[1:]):
        edges.append({"from": f"E:{b['event_id']}", "to": f"E:{a['event_id']}", "kind": "seq"})
    for e in events:
        edges.append({"from": f"E:{e['event_id']}", "to": f"R:{e.get('work_order_id')}", "kind": "claimed_by"})
    ids = {r["work_order_id"] for r in receipts}
    hash_to_event = {e.get("hash"): e["event_id"] for e in events if e.get("hash")}
    for r in receipts:
        rb = r.get("replay_binding", {})
        for pred in rb.get("predecessor_work_order_ids", []):
            if pred in ids:
                edges.append({"from": f"R:{r['work_order_id']}", "to": f"R:{pred}", "kind": "predecessor"})
        head = rb.get("chain_head_hash")
        if head and head in hash_to_event:
            edges.append({"from": f"R:{r['work_order_id']}", "to": f"E:{hash_to_event[head]}", "kind": "binds"})
    return {"nodes": nodes, "edges": edges, "parse_errors": parse_errors,
            "edge_kinds": {"cycle_detection": ["seq", "predecessor", "claimed_by"],
                           "bridge_detection": ["seq", "predecessor", "binds"]}}


def detect_cycles(dag):
    indeg = {n["id"]: 0 for n in dag["nodes"]}
    adj = {}
    for e in dag["edges"]:
        if e["kind"] not in ("seq", "predecessor", "claimed_by"):
            continue  # binds edges are grounding, not causal ordering
        adj.setdefault(e["from"], []).append(e["to"])
        indeg[e["to"]] = indeg.get(e["to"], 0) + 1
        indeg.setdefault(e["from"], indeg.get(e["from"], 0))
    queue = [n for n, d in indeg.items() if d == 0]
    seen = 0
    while queue:
        n = queue.pop()
        seen += 1
        for m in adj.get(n, []):
            indeg[m] -= 1
            if indeg[m] == 0:
                queue.append(m)
    cyclic = sorted(n for n, d in indeg.items() if d > 0)
    return {"acyclic": seen == len(indeg), "cycle_nodes": cyclic}


def detect_bridges(dag):
    """Iterative Tarjan bridges over the undirected CAUSAL SPINE (seq + predecessor +
    binds edges; claimed_by edges would saturate every cycle). A bridge is an edge whose
    loss splits the causal record — a single point of chain failure."""
    adj = {}
    for e in dag["edges"]:
        if e["kind"] not in ("seq", "predecessor", "binds"):
            continue
        adj.setdefault(e["from"], set()).add(e["to"])
        adj.setdefault(e["to"], set()).add(e["from"])
    disc, low, timer = {}, {}, [0]
    bridges = []
    for root in adj:
        if root in disc:
            continue
        stack = [(root, None, iter(sorted(adj.get(root, []))))]
        disc[root] = low[root] = timer[0]; timer[0] += 1
        while stack:
            node, parent, it = stack[-1]
            advanced = False
            for nxt in it:
                if nxt == parent:
                    parent = None  # skip parallel edge only once
                    continue
                if nxt in disc:
                    low[node] = min(low[node], disc[nxt])
                else:
                    disc[nxt] = low[nxt] = timer[0]; timer[0] += 1
                    stack.append((nxt, node, iter(sorted(adj[nxt]))))
                    advanced = True
                    break
            if not advanced:
                stack.pop()
                if stack:
                    p = stack[-1][0]
                    low[p] = min(low[p], low[node])
                    if low[node] > disc[p]:
                        bridges.append(sorted([p, node]))
    return {"bridges": bridges, "bridge_count": len(bridges)}


# ---------------------------------------------------------------- golden + self-test

GOLDEN_SUBJECT = "0abd0cc351b08c8c19414410e0cd365bfeac0ccb"
WO1, WO2 = "ALOOP-ZCODE-DOGFOOD-001/lane-8-profile", "ALOOP-ZCODE-DOGFOOD-001/lane-8-dogfood"


def golden_case():
    events = [
        {"seq": 0, "event_id": "e0", "event_type": "actuation", "work_order_id": WO1,
         "actor": "lane-8", "authority_grant": "operator-dispatch ALOOP-ZCODE-DOGFOOD-001",
         "provider": "zcode", "provider_execution_id": "lane-8-run-0001",
         "subject_sha": GOLDEN_SUBJECT, "actuation_id": "act-0001",
         "cmd": "write schemas/aloop-execution-receipt.schema.json",
         "consequence_hash": "11" * 32, "ts": "2026-09-25T09:00:00Z",
         "recorded_at": "2026-09-25T09:00:05Z", "hash_algo": "sha256"},
        {"seq": 1, "event_id": "e1", "event_type": "verification", "work_order_id": WO1,
         "actor": "lane-8", "authority_grant": "operator-dispatch ALOOP-ZCODE-DOGFOOD-001",
         "provider": "zcode", "provider_execution_id": "lane-8-run-0001",
         "subject_sha": GOLDEN_SUBJECT, "actuation_id": "act-0002",
         "cmd": "python3 tools/aloop_falsifiers.py check golden",
         "consequence_hash": "22" * 32, "ts": "2026-09-25T09:05:00Z",
         "recorded_at": "2026-09-25T09:05:04Z", "hash_algo": "sha256"},
        {"seq": 2, "event_id": "e2", "event_type": "actuation", "work_order_id": WO2,
         "actor": "lane-8", "authority_grant": "operator-dispatch ALOOP-ZCODE-DOGFOOD-001",
         "provider": "zcode", "provider_execution_id": "lane-8-run-0002",
         "subject_sha": GOLDEN_SUBJECT, "actuation_id": "act-0003",
         "cmd": "dogfood scan of lane manifests",
         "consequence_hash": "33" * 32, "ts": "2026-09-25T09:40:00Z",
         "recorded_at": "2026-09-25T09:40:06Z", "hash_algo": "sha256"},
    ]
    prev = None
    for e in events:
        e["prev_hash"] = prev
        e["hash"] = canonical(e)
        prev = e["hash"]
    receipts = [
        {"work_order_id": WO1,
         "origin_authority": {"ceiling": "DO", "grant": "operator-dispatch ALOOP-ZCODE-DOGFOOD-001", "actor": "lane-8"},
         "provider": {"name": "zcode", "transport": "local-session", "authority_ceiling": "DO"},
         "provider_execution_id": "lane-8-run-0001",
         "identity": {"subject": "aloop-execution-receipt-profile", "repo": "affidavit",
                      "subject_sha": GOLDEN_SUBJECT, "base_sha": GOLDEN_SUBJECT},
         "authority": {"ceiling": "DO", "grant": "operator-dispatch ALOOP-ZCODE-DOGFOOD-001", "actor": "lane-8"},
         "consequence": {"commits": [], "files_changed": ["schemas/aloop-execution-receipt.schema.json"], "remote_effects": []},
         "consequences": [{"hash": "11" * 32, "kind": "file"}, {"hash": "22" * 32, "kind": "artifact"}],
         "replay": {"commands": [{"cmd": "tools/aloop_falsifiers.py self-test", "exit": 0, "cwd": "."}]},
         "standing": {"value": "PARTIAL_ALIVE", "derived_from": "self-test witnessed"},
         "replay_binding": {"event_ids": ["e0", "e1"], "chain_head_hash": events[1]["hash"], "chain_hash_algo": "sha256"}},
        {"work_order_id": WO2,
         "origin_authority": {"ceiling": "DO", "grant": "operator-dispatch ALOOP-ZCODE-DOGFOOD-001", "actor": "lane-8"},
         "provider": {"name": "zcode", "transport": "local-session", "authority_ceiling": "DO"},
         "provider_execution_id": "lane-8-run-0002",
         "identity": {"subject": "aloop-dogfood-scan", "repo": "affidavit",
                      "subject_sha": GOLDEN_SUBJECT, "base_sha": GOLDEN_SUBJECT},
         "authority": {"ceiling": "DO", "grant": "operator-dispatch ALOOP-ZCODE-DOGFOOD-001", "actor": "lane-8"},
         "consequence": {"commits": [], "files_changed": ["docs/ALOOP_EXECUTION_RECEIPT.md"], "remote_effects": []},
         "consequences": [{"hash": "33" * 32, "kind": "artifact"}],
         "replay": {"commands": [{"cmd": "tools/aloop_falsifiers.py check", "exit": 0, "cwd": "."}]},
         "standing": {"value": "PARTIAL_ALIVE", "derived_from": "dogfood scan run"},
         "replay_binding": {"event_ids": ["e2"], "chain_head_hash": events[2]["hash"], "chain_hash_algo": "sha256",
                            "predecessor_work_order_ids": [WO1]}},
    ]
    return receipts, events


def mutate(case, fn):
    receipts, events = copy.deepcopy(case)
    fn(receipts, events)
    return receipts, events


def add_event(receipts, events, **over):
    e = copy.deepcopy(events[-1])
    e.update(over)
    e["seq"] = events[-1]["seq"] + 1
    e["prev_hash"] = events[-1]["hash"]
    e.pop("hash", None)
    e["hash"] = canonical(e)
    events.append(e)
    return e


def self_test(tmp=None):
    tmp = Path(tmp or tempfile.mkdtemp(prefix="aloop-falsifiers-"))
    receipts, events = golden_case()
    case = (receipts, events)
    results = []

    def expect(name, rs, evs, want_f, want_term, want_absent=False):
        d = tmp / name
        write_case(d, rs, evs)
        rep = check_case(d)
        hit = any(v["falsifier"] == want_f and v["term"] == want_term for v in rep["violations"])
        ok = (not hit) if want_absent else hit
        results.append((name, want_f, want_term, ok, rep))

    expect("00_golden_clean", *case, "F01", "R_missing_replay", want_absent=True)
    m = mutate(case, lambda rs, ev: ev.pop(1))
    expect("F01_missing_event", *m, "F01", "R_missing_replay")
    m = mutate(case, lambda rs, ev: ev.insert(0, ev.pop(1)))
    expect("F02_reordered_event", *m, "F02", "mu_on_O")
    m = mutate(case, lambda rs, ev: add_event(rs, ev, event_id="e3", actuation_id="act-0003",
                                              provider_execution_id="lane-8-run-0003",
                                              work_order_id=WO2))
    expect("F03_duplicate_actuation", *m, "F03", "R_missing_authority")
    m = mutate(case, lambda rs, ev: (ev[2].__setitem__("consequence_hash", ev[1]["consequence_hash"]),
                                     ev[2].__setitem__("hash", canonical(ev[2]))))
    expect("F04_duplicate_consequence", *m, "F04", "admission_vacuous")
    m = mutate(case, lambda rs, ev: (ev[1].__setitem__("prev_hash", "f" * 64),
                                     ev[1].__setitem__("hash", canonical(ev[1]))))
    expect("F05_broken_chain", *m, "F05", "R_missing_identity")
    m = mutate(case, lambda rs, ev: rs[0].__setitem__("consequence", {"commits": [], "files_changed": [], "remote_effects": []}) or rs[0].__setitem__("consequences", []))
    expect("F06_receipt_without_consequence", *m, "F06", "R_missing_consequence")
    m = mutate(case, lambda rs, ev: rs[1]["replay_binding"].__setitem__("event_ids", []))
    expect("F07_consequence_without_receipt", *m, "F07", "mu_unlawful")
    m = mutate(case, lambda rs, ev: (ev[2].__setitem__("provider", "claude"),
                                     ev[2].__setitem__("hash", canonical(ev[2]))))
    expect("F08_provider_identity_rewrite", *m, "F08", "R_missing_identity")
    m = mutate(case, lambda rs, ev: rs[1]["identity"].__setitem__("subject_sha", "2" * 40))
    expect("F09_subject_mismatch", *m, "F09", "R_missing_identity")
    m = mutate(case, lambda rs, ev: (ev[2].__setitem__("authority_grant", "self-granted"),
                                     ev[2].__setitem__("hash", canonical(ev[2]))))
    expect("F10_authority_mismatch", *m, "F10", "R_missing_authority")
    m = mutate(case, lambda rs, ev: (ev[2].__setitem__("recorded_at", "2026-09-25T08:00:00Z"),
                                     ev[2].__setitem__("hash", canonical(ev[2]))))
    expect("F11_posthoc_fabricated", *m, "F11", "R_not_fed_back")
    # reconstructed marking: recorded-before-occurrence WITH reconstructed:true is honest, not a violation
    m = mutate(case, lambda rs, ev: (ev[2].__setitem__("recorded_at", "2026-09-25T08:00:00Z"),
                                     ev[2].__setitem__("reconstructed", True),
                                     ev[2].__setitem__("hash", canonical(ev[2]))))
    expect("F11_reconstructed_marked_clean", *m, "F11", "R_not_fed_back", want_absent=True)

    # DAG witnesses
    d = tmp / "00_golden_clean"
    dag = build_dag(d)
    cyc = detect_cycles(dag)
    brg = detect_bridges(dag)
    dag_ok = cyc["acyclic"] and brg["bridge_count"] > 0
    results.append(("DAG_golden_acyclic_with_bridges", "DAG", "cycle/bridge", dag_ok, cyc))
    m = mutate(case, lambda rs, ev: rs[0]["replay_binding"].__setitem__("predecessor_work_order_ids", [WO2]))
    d = tmp / "DAG_cycle"
    write_case(d, *m)
    cyc = detect_cycles(build_dag(d))
    dag_ok = (not cyc["acyclic"]) and bool(cyc["cycle_nodes"])
    results.append(("DAG_cycle_detected", "DAG", "cycle", dag_ok, cyc))

    failed = 0
    for name, f, term, ok, rep in results:
        failed += not ok
        print(f"[{'OK ' if ok else 'FAIL'}] WITNESSED {name}: expected {'NO ' if term == 'cycle/bridge' and 'absent' in name else ''}{f} -> {term}"
              + (f"  [violations={len(rep.get('violations', []))}]" if isinstance(rep, dict) and 'violations' in rep else f"  {rep}"))
    clean = check_case(tmp / "00_golden_clean")
    print(f"\ngolden clean case: {clean['verdict']} events={clean['events']} receipts={clean['receipts']}")
    print(f"witnessed={len(results) - failed}/{len(results)} failed={failed}")
    return 1 if failed else 0


def main(argv):
    if len(argv) >= 2 and argv[0] == "check":
        rep = check_case(argv[1])
        print(json.dumps(rep, indent=2))
        return 0 if rep["verdict"] == "CONFORMANT" else 1
    if len(argv) >= 2 and argv[0] == "dag":
        dag = build_dag(argv[1])
        dag["cycles"] = detect_cycles(dag)
        dag["bridges"] = detect_bridges(dag)
        out = json.dumps(dag, indent=2)
        if "--out" in argv:
            Path(argv[argv.index("--out") + 1]).write_text(out + "\n")
        print(out)
        return 0 if dag["cycles"]["acyclic"] else 1
    if len(argv) >= 2 and argv[0] == "emit-golden":
        write_case(argv[1], *golden_case())
        print(f"golden case written to {argv[1]}")
        return 0
    if argv[:1] == ["self-test"]:
        return self_test(argv[1] if len(argv) > 1 else None)
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
