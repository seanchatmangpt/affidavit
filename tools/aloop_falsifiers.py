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
import copy, json, hashlib, re, shutil, sys, tempfile
from datetime import datetime
from pathlib import Path

RECEIPT_REQUIRED = ("work_order_id", "origin_authority", "provider", "provider_execution_id")
FALSIFIERS = {
    "F00": ("profile_nonconformant", "mu_on_O"),
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

HEX40 = re.compile(r"^[0-9a-f]{40}$")
PROFILE_SCHEMA = Path(__file__).resolve().parent.parent / "schemas/aloop-execution-receipt.schema.json"


def canonical(ev):
    """sha256 over canonical JSON of the event minus `hash`. Returns None for an
    unsupported hash_algo (the caller refuses it explicitly, never passes it)."""
    payload = {k: v for k, v in ev.items() if k != "hash"}
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    algo = ev.get("hash_algo", "sha256")
    return hashlib.sha256(blob.encode()).hexdigest() if algo == "sha256" else None


def is_seq(v):
    return isinstance(v, int) and not isinstance(v, bool) and v >= 0


def seq_key(e):
    """Total order key that never raises on malformed seq (malformed seqs sort last and
    are refused separately by F01)."""
    v = e.get("seq")
    return (0, v, "") if is_seq(v) else (1, 0, repr(v))


def parse_ts(v):
    """RFC 3339 instant -> aware datetime (UTC-normalized comparison), else None."""
    if not isinstance(v, str):
        return None
    try:
        d = datetime.fromisoformat(v.replace("Z", "+00:00"))
    except ValueError:
        return None
    return d if d.tzinfo is not None else None


def load_case(case_dir):
    """Load receipts + events. Anything that is not a JSON object is a parse error
    (typed, refused by check_case) instead of a crash downstream."""
    case_dir = Path(case_dir)
    receipts, parse_errors = [], []
    rdir = case_dir / "receipts"
    if rdir.is_dir():
        for p in sorted(rdir.glob("*.json")):
            try:
                obj = json.loads(p.read_text())
            except (json.JSONDecodeError, UnicodeDecodeError) as e:
                parse_errors.append(f"receipt {p.name}: {e}")
                continue
            if not isinstance(obj, dict):
                parse_errors.append(f"receipt {p.name}: top level is {type(obj).__name__}, not object")
                continue
            obj.setdefault("__file__", p.name)
            receipts.append(obj)
    events, order = [], []
    efile = case_dir / "events.ndjson"
    if efile.exists():
        try:
            text = efile.read_text()
        except UnicodeDecodeError as e:
            parse_errors.append(f"events.ndjson: {e}")
            text = ""
        for i, line in enumerate(text.splitlines()):
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError as e:
                parse_errors.append(f"events.ndjson line {i + 1}: {e}")
                continue
            if not isinstance(obj, dict):
                parse_errors.append(f"events.ndjson line {i + 1}: {type(obj).__name__}, not object")
                continue
            events.append(obj)
            order.append(len(events) - 1)
    return receipts, events, order, parse_errors


def write_case(case_dir, receipts, events):
    case_dir = Path(case_dir)
    shutil.rmtree(case_dir, ignore_errors=True)
    (case_dir / "receipts").mkdir(parents=True)
    used = set()
    for r in receipts:
        stem = str(r.get("work_order_id") or "no_work_order").replace("/", "_")
        name, n = f"{stem}.json", 1
        while name in used:  # never let a duplicate delivery overwrite (and hide) its twin
            n += 1
            name = f"{stem}.{n}.json"
        used.add(name)
        body = {k: v for k, v in r.items() if k != "__file__"}
        (case_dir / "receipts" / name).write_text(json.dumps(body, indent=2, sort_keys=True) + "\n")
    with (case_dir / "events.ndjson").open("w") as f:
        for ev in events:
            f.write(json.dumps(ev, sort_keys=True) + "\n")


def profile_errors(receipts):
    """Validate each receipt against the ALOOP profile schema (F00). Returns
    (violations, status). Without jsonschema the status is the typed
    SKIPPED(jsonschema-missing) and is surfaced in the report, never hidden."""
    try:
        import jsonschema
    except ImportError:
        return [], "SKIPPED(jsonschema-missing)"
    schema = json.loads(PROFILE_SCHEMA.read_text())
    validator = jsonschema.Draft202012Validator(schema)
    out = []
    for r in receipts:
        body = {k: v for k, v in r.items() if k != "__file__"}
        for err in sorted(validator.iter_errors(body), key=lambda e: list(e.absolute_path)):
            loc = "/".join(str(x) for x in err.absolute_path) or "<root>"
            out.append({"falsifier": "F00", "term": "mu_on_O",
                        "detail": f"receipt {r.get('__file__')} violates ALOOP profile at {loc}: {err.message[:160]}"})
    return out, "CHECKED"


# ---------------------------------------------------------------- checks

def V(f, detail):
    return {"falsifier": f, "term": FALSIFIERS[f][1], "detail": detail}


def check_case(case_dir, validate_profile=True):
    receipts, events, order, parse_errors = load_case(case_dir)
    violations = [V("F05", p) for p in parse_errors]
    if not events and not parse_errors:
        violations.append(V("F01", "no events in chain"))

    # receipt identity + duplicate delivery of receipts
    claimed, wo_seen = {}, {}
    for r in receipts:
        for miss in [k for k in RECEIPT_REQUIRED if k not in r]:
            violations.append(V("F08", f"receipt {r.get('__file__')} missing {miss}"))
        wo = r.get("work_order_id")
        if wo in wo_seen:
            violations.append(V("F04", f"work order {wo} receipted twice ({wo_seen[wo]}, {r.get('__file__')}); duplicate delivery"))
        wo_seen.setdefault(wo, r.get("__file__"))
        rb = r.get("replay_binding") if isinstance(r.get("replay_binding"), dict) else {}
        eids = rb.get("event_ids", [])
        if not isinstance(eids, list):
            violations.append(V("F05", f"receipt {wo} replay_binding.event_ids is not a list"))
            eids = []
        for eid in eids:
            if not isinstance(eid, str):
                violations.append(V("F05", f"receipt {wo} claims non-string event id {eid!r}"))
                continue
            claimed.setdefault(eid, []).append(r)

    # event identity: seq type, unique event_id
    ids = {}
    for e in events:
        if not is_seq(e.get("seq")):
            violations.append(V("F01", f"event {e.get('event_id')} seq {e.get('seq')!r} is not a non-negative integer"))
        eid = e.get("event_id")
        if not isinstance(eid, str) or not eid:
            violations.append(V("F05", f"event at seq {e.get('seq')!r} has no event_id"))
        elif eid in ids:
            violations.append(V("F05", f"event_id {eid} appears twice in chain; receipts cannot bind it uniquely"))
        ids.setdefault(eid, e)

    by_seq = sorted(events, key=seq_key)
    # F01 missing event: chain must start at 0 and be gap-free; claims must resolve
    seqs = [e.get("seq") for e in by_seq if is_seq(e.get("seq"))]
    if seqs and seqs[0] != 0:
        violations.append(V("F01", f"chain starts at seq {seqs[0]}; prefix 0..{seqs[0] - 1} missing"))
    for a, b in zip(seqs, seqs[1:]):
        if b - a > 1:
            violations.append(V("F01", f"seq gap {a} -> {b}"))
    for eid, rs in claimed.items():
        if eid not in ids:
            violations.append(V("F01", f"receipt(s) {sorted(str(r.get('work_order_id')) for r in rs)} claim event {eid} absent from chain"))
    # F02 reordered (strict: equal seq twice is also not an append order)
    file_seqs = [events[i].get("seq") for i in order]
    num = [s if is_seq(s) else -1 for s in file_seqs]
    if any(a >= b for a, b in zip(num, num[1:])):
        violations.append(V("F02", f"append order {file_seqs} is not strictly seq-monotonic"))
    # F03 duplicate actuation (and actuation with no id at all)
    seen = set()
    for e in events:
        aid = e.get("actuation_id")
        if not isinstance(aid, str) or not aid:
            violations.append(V("F03", f"event {e.get('event_id')} carries no actuation_id; actuation not attributable to a grant"))
            continue
        if aid in seen:
            violations.append(V("F03", f"actuation {aid} executed twice; second run has no fresh grant"))
        seen.add(aid)
    # F04 duplicate consequence across distinct actuations; one event claimed twice
    cons = {}
    for e in events:
        cons.setdefault(e.get("consequence_hash"), set()).add(str(e.get("actuation_id")))
    for ch, acts in cons.items():
        if ch and len(acts) > 1:
            violations.append(V("F04", f"consequence {str(ch)[:12]}… claimed by {sorted(acts)}; receipts no longer discriminate"))
    for eid, rs in claimed.items():
        wos = sorted({str(r.get("work_order_id")) for r in rs})
        if len(rs) > 1:
            violations.append(V("F04", f"event {eid} claimed by {len(rs)} receipts {wos}; one actuation, many certifications"))
    # F05 broken chain + replay binding head
    prev = None
    for e in by_seq:
        if e.get("prev_hash") != prev:
            violations.append(V("F05", f"event {e.get('event_id')} prev_hash does not link predecessor"))
        c = canonical(e)
        if c is None:
            violations.append(V("F05", f"event {e.get('event_id')} hash_algo {e.get('hash_algo')!r} unsupported by this verifier; refused, not passed"))
        elif c != e.get("hash"):
            violations.append(V("F05", f"event {e.get('event_id')} hash does not commit its payload"))
        prev = e.get("hash")
    for r in receipts:
        rb = r.get("replay_binding") if isinstance(r.get("replay_binding"), dict) else {}
        head = rb.get("chain_head_hash")
        mine = [ids[x] for x in rb.get("event_ids", []) if isinstance(x, str) and x in ids]
        if head is not None and mine:
            want = max(mine, key=seq_key).get("hash")
            if head != want:
                violations.append(V("F05", f"receipt {r.get('work_order_id')} chain_head_hash {str(head)[:12]}… != hash of its last claimed event {str(want)[:12]}…; replay mismatch"))
    # F06 receipt without consequence
    for r in receipts:
        c = r.get("consequence") if isinstance(r.get("consequence"), dict) else {}
        empty_r = not (c.get("commits") or c.get("files_changed") or c.get("remote_effects"))
        empty_a = not r.get("consequences")
        if empty_r and empty_a:
            violations.append(V("F06", f"receipt {r.get('work_order_id')} records no consequence"))
    # F07 consequence without receipt (unreceipted actuation; the UAR count)
    reconstructed = 0
    for e in events:
        if e.get("event_id") not in claimed:
            if e.get("reconstructed") is True:
                reconstructed += 1
                continue
            violations.append(V("F07", f"actuation event {e.get('event_id')} ({e.get('actuation_id')}) claimed by no receipt"))
    # F08 provider identity rewrite
    names = {}
    for e in events:
        names.setdefault(e.get("provider_execution_id"), set()).add(e.get("provider"))
    for r in receipts:
        prov = r.get("provider") if isinstance(r.get("provider"), dict) else {}
        names.setdefault(r.get("provider_execution_id"), set()).add(prov.get("name"))
    for peid, ns in names.items():
        real = {n for n in ns if isinstance(n, str) and n}
        if len(real) > 1:
            violations.append(V("F08", f"provider_execution_id {peid} carried by providers {sorted(real)}"))
    # F08 (join) / F09 subject / F10 authority on every claim; F11 post-hoc fabrication
    for e in events:
        esub = e.get("subject_sha")
        if esub is not None and not (isinstance(esub, str) and HEX40.match(esub)):
            violations.append(V("F09", f"event {e.get('event_id')} subject_sha {esub!r} is not a 40-hex git commit"))
        for r in claimed.get(e.get("event_id"), []):
            wo = r.get("work_order_id")
            if e.get("work_order_id") != wo:
                violations.append(V("F09", f"event {e.get('event_id')} belongs to work order {e.get('work_order_id')} but is claimed by receipt {wo}"))
            if e.get("provider_execution_id") != r.get("provider_execution_id"):
                violations.append(V("F08", f"event {e.get('event_id')} provider_execution_id {e.get('provider_execution_id')} != receipt {wo} {r.get('provider_execution_id')}"))
            ident = r.get("identity") if isinstance(r.get("identity"), dict) else {}
            rsub = ident.get("subject_sha")
            if esub and rsub and esub != rsub:
                violations.append(V("F09", f"event {e['event_id']} subject {str(esub)[:12]}… != receipt {wo} subject {str(rsub)[:12]}…"))
            oa = r.get("origin_authority") if isinstance(r.get("origin_authority"), dict) else {}
            if (e.get("actor"), e.get("authority_grant")) != (oa.get("actor"), oa.get("grant")):
                violations.append(V("F10", f"event {e.get('event_id')} acted as ({e.get('actor')}, {e.get('authority_grant')}) but receipt {wo} records ({oa.get('actor')}, {oa.get('grant')})"))
        ts, rec = e.get("ts"), e.get("recorded_at")
        if ts is not None or rec is not None:
            t, rc = parse_ts(ts), parse_ts(rec)
            if t is None or rc is None:
                violations.append(V("F11", f"event {e.get('event_id')} ts/recorded_at ({ts!r}, {rec!r}) not both RFC 3339 instants with offset"))
            elif rc < t and e.get("reconstructed") is not True:
                violations.append(V("F11", f"event {e.get('event_id')} recorded {rec} before its claimed occurrence {ts} with no reconstructed marking"))
    schema_status = "NOT_REQUESTED"
    if validate_profile:
        errs, schema_status = profile_errors(receipts)
        violations.extend(errs)
    return {
        "case": str(case_dir),
        "events": len(events),
        "receipts": len(receipts),
        "violations": violations,
        "uar_count": sum(1 for v in violations if v["falsifier"] == "F07"),
        "reconstructed_unclaimed": reconstructed,
        "profile_validation": schema_status,
        "verdict": "CONFORMANT" if not violations else "REFUSED",
    }


# ---------------------------------------------------------------- DAG

def build_dag(case_dir):
    """Causal DAG. Malformed events/receipts (no id) are left out of the graph and
    listed in `dropped`; check_case refuses them, the DAG never crashes on them."""
    receipts, events, _, parse_errors = load_case(case_dir)
    dropped = []
    events_ok, receipts_ok = [], []
    for e in events:
        if isinstance(e.get("event_id"), str) and e["event_id"]:
            events_ok.append(e)
        else:
            dropped.append(f"event seq={e.get('seq')!r} without event_id")
    for r in receipts:
        if isinstance(r.get("work_order_id"), str) and r["work_order_id"]:
            receipts_ok.append(r)
        else:
            dropped.append(f"receipt {r.get('__file__')} without work_order_id")
    nodes, edges = [], []
    for e in events_ok:
        nodes.append({"id": f"E:{e['event_id']}", "kind": "event", "seq": e.get("seq"),
                      "work_order_id": e.get("work_order_id")})
    for r in receipts_ok:
        st = r.get("standing") if isinstance(r.get("standing"), dict) else {}
        nodes.append({"id": f"R:{r['work_order_id']}", "kind": "receipt", "standing": st.get("value")})
    ids = {r["work_order_id"] for r in receipts_ok}
    by_seq = sorted(events_ok, key=seq_key)
    for a, b in zip(by_seq, by_seq[1:]):
        edges.append({"from": f"E:{b['event_id']}", "to": f"E:{a['event_id']}", "kind": "seq"})
    for e in events_ok:
        if e.get("work_order_id") in ids:
            edges.append({"from": f"E:{e['event_id']}", "to": f"R:{e['work_order_id']}", "kind": "claimed_by"})
    hash_to_event = {e.get("hash"): e["event_id"] for e in events_ok if isinstance(e.get("hash"), str)}
    for r in receipts_ok:
        rb = r.get("replay_binding") if isinstance(r.get("replay_binding"), dict) else {}
        for pred in rb.get("predecessor_work_order_ids", []) or []:
            if isinstance(pred, str) and pred in ids:
                edges.append({"from": f"R:{r['work_order_id']}", "to": f"R:{pred}", "kind": "predecessor"})
        head = rb.get("chain_head_hash")
        if isinstance(head, str) and head in hash_to_event:
            edges.append({"from": f"R:{r['work_order_id']}", "to": f"E:{hash_to_event[head]}", "kind": "binds"})
    return {"nodes": nodes, "edges": edges, "parse_errors": parse_errors, "dropped": dropped,
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
         "commands": [{"cmd": "tools/aloop_falsifiers.py self-test", "exit": 0, "cwd": "."}],
         "evidence": [{"kind": "falsifier", "verifier": "tools/aloop_falsifiers.py self-test", "exit": 0}],
         "exit_status": "ok",
         "timestamps": {"started_at": "2026-09-25T09:00:00Z", "finished_at": "2026-09-25T09:05:04Z"},
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
         "commands": [{"cmd": "tools/aloop_falsifiers.py check", "exit": 0, "cwd": "."}],
         "evidence": [{"kind": "falsifier", "verifier": "tools/aloop_falsifiers.py check", "exit": 0}],
         "exit_status": "ok",
         "timestamps": {"started_at": "2026-09-25T09:40:00Z", "finished_at": "2026-09-25T09:40:06Z"},
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
        # a clean witness must be fully CONFORMANT; a refusal witness must be REFUSED
        ok = (rep["verdict"] == "CONFORMANT") if want_absent else (hit and rep["verdict"] == "REFUSED")
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
                                     ev[2].__setitem__("hash", canonical(ev[2])),
                                     rs[1]["replay_binding"].__setitem__("chain_head_hash", ev[2]["hash"])))
    expect("F11_reconstructed_marked_clean", *m, "F11", "R_not_fed_back", want_absent=True)

    # hardening witnesses (each was a CONFORMANT false-accept or a crash before)
    m = mutate(case, lambda rs, ev: rs[1]["replay_binding"]["event_ids"].append("e99"))
    expect("H01_phantom_claim", *m, "F01", "R_missing_replay")
    m = mutate(case, lambda rs, ev: rs[0]["replay_binding"]["event_ids"].append("e2"))
    expect("H02_double_claim", *m, "F04", "admission_vacuous")
    m = mutate(case, lambda rs, ev: (rs[0]["replay_binding"].__setitem__("event_ids", ["e0", "e1", "e2"]),
                                     rs[1]["replay_binding"].__setitem__("event_ids", ["e2"])))
    expect("H03_cross_work_order_claim", *m, "F09", "R_missing_identity")
    m = mutate(case, lambda rs, ev: rs[1]["replay_binding"].__setitem__("chain_head_hash", "ab" * 32))
    expect("H04_wrong_chain_head", *m, "F05", "R_missing_identity")
    m = mutate(case, lambda rs, ev: rs.append(copy.deepcopy(rs[1])))
    expect("H05_duplicate_receipt_delivery", *m, "F04", "admission_vacuous")

    def truncate(rs, ev):
        ev.pop(0)
        prev = None
        for e in ev:
            e["prev_hash"] = prev
            e["hash"] = canonical(e)
            prev = e["hash"]
        rs[0]["replay_binding"].update(event_ids=["e1"], chain_head_hash=ev[0]["hash"])
        rs[1]["replay_binding"]["chain_head_hash"] = ev[1]["hash"]
    expect("H06_truncated_prefix_rechained", *mutate(case, truncate), "F01", "R_missing_replay")

    def offset_posthoc(rs, ev):
        # 10:00+02:00 == 08:00Z, before ts 09:40Z; lexicographic comparison accepted this
        ev[2]["recorded_at"] = "2026-09-25T10:00:00+02:00"
        ev[2]["hash"] = canonical(ev[2])
        rs[1]["replay_binding"]["chain_head_hash"] = ev[2]["hash"]
    expect("H07_offset_posthoc", *mutate(case, offset_posthoc), "F11", "R_not_fed_back")
    m = mutate(case, lambda rs, ev: rs[0]["identity"].__setitem__("subject_sha", "zz"))
    expect("H08_profile_nonconformant", *m, "F00", "mu_on_O")

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
