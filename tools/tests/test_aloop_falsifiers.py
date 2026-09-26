"""Chicago-style tests for tools/aloop_falsifiers.py and the ALOOP ExecutionReceipt
profile: real files on disk, the real CLI as a subprocess, the real jsonschema
validator, state-based assertions on verdicts / falsifier ids / broken terms.
No test doubles.

Run: python3 -m pytest tools/tests -q
"""
import copy, json, re, subprocess, sys
from pathlib import Path

import jsonschema
import pytest

TOOLS = Path(__file__).resolve().parent.parent
REPO = TOOLS.parent
sys.path.insert(0, str(TOOLS))
sys.path.insert(0, str(TOOLS / "bench"))
import aloop_falsifiers as A  # noqa: E402
import bench_aloop_falsifiers as B  # noqa: E402

CLI = [sys.executable, str(TOOLS / "aloop_falsifiers.py")]
GOLDEN = REPO / "fixtures/aloop/golden_case"
PROFILE = REPO / "schemas/aloop-execution-receipt.schema.json"


def cli(*args):
    return subprocess.run([*CLI, *map(str, args)], capture_output=True, text=True, cwd=REPO)


def check_json(case):
    p = cli("check", case)
    rep = json.loads(p.stdout)  # never a traceback: always a JSON verdict
    assert p.returncode == (0 if rep["verdict"] == "CONFORMANT" else 1), p.stderr
    return rep


def fired(rep):
    return {(v["falsifier"], v["term"]) for v in rep["violations"]}


def _seq(e):
    # test-local order key (independent of the module under test, so a reverted
    # verifier fails on semantics, not on a missing helper)
    v = e.get("seq")
    return v if isinstance(v, int) and not isinstance(v, bool) else 10 ** 9


def rechain(receipts, events):
    prev = None
    for e in sorted(events, key=_seq):
        e["prev_hash"] = prev
        e["hash"] = A.canonical(e)
        prev = e["hash"]
    by_id = {e["event_id"]: e for e in events}
    for r in receipts:
        mine = [by_id[x] for x in r["replay_binding"]["event_ids"] if x in by_id]
        if mine:
            r["replay_binding"]["chain_head_hash"] = max(mine, key=_seq)["hash"]


def case(tmp_path, name, fn=None, rechain_after=False):
    rs, ev = A.golden_case()
    if fn:
        fn(rs, ev)
    if rechain_after:
        rechain(rs, ev)
    d = tmp_path / name
    A.write_case(d, rs, ev)
    return d


# ------------------------------------------------------------- golden / projection

def test_committed_golden_fixture_is_the_generator_projection(tmp_path):
    out = tmp_path / "golden"
    assert cli("emit-golden", out).returncode == 0
    committed = sorted(p.relative_to(GOLDEN) for p in GOLDEN.rglob("*") if p.is_file())
    emitted = sorted(p.relative_to(out) for p in out.rglob("*") if p.is_file())
    assert committed == emitted
    for rel in committed:
        assert (GOLDEN / rel).read_bytes() == (out / rel).read_bytes(), rel


def test_golden_receipts_conform_to_the_aloop_profile():
    validator = jsonschema.Draft202012Validator(json.loads(PROFILE.read_text()))
    for p in sorted((GOLDEN / "receipts").glob("*.json")):
        errs = [e.message for e in validator.iter_errors(json.loads(p.read_text()))]
        assert errs == [], (p.name, errs)


def test_golden_case_is_conformant_and_acyclic():
    rep = check_json(GOLDEN)
    assert rep["verdict"] == "CONFORMANT" and rep["uar_count"] == 0
    assert rep["profile_validation"] == "CHECKED"
    p = cli("dag", GOLDEN)
    dag = json.loads(p.stdout)
    assert p.returncode == 0 and dag["cycles"]["acyclic"] and dag["bridges"]["bridge_count"] > 0


def test_self_test_witnesses_every_refusal(tmp_path):
    p = cli("self-test", tmp_path / "st")
    assert p.returncode == 0, p.stdout
    assert "failed=0" in p.stdout and "[FAIL]" not in p.stdout


def test_check_is_byte_deterministic():
    assert cli("check", GOLDEN).stdout == cli("check", GOLDEN).stdout


def test_vendored_dfcm_schema_in_sync():
    p = subprocess.run([sys.executable, str(TOOLS / "check_dfcm_schema_sync.py")],
                       capture_output=True, text=True)
    assert p.returncode == 0, p.stdout


# ------------------------------------------------------------- adversarial corpus

def _pop_all(key):
    def fn(rs, ev):
        for e in ev:
            e.pop(key, None)
    return fn


def _relink(rs, ev):
    # rewrite e1.prev_hash and recompute every downstream hash + both receipt heads:
    # the hash clause is satisfied, only the prev_hash linkage clause can refuse it
    ev[1]["prev_hash"] = "f" * 64
    ev[1]["hash"] = A.canonical(ev[1])
    ev[2]["prev_hash"] = ev[1]["hash"]
    ev[2]["hash"] = A.canonical(ev[2])
    rs[0]["replay_binding"]["chain_head_hash"] = ev[1]["hash"]
    rs[1]["replay_binding"]["chain_head_hash"] = ev[2]["hash"]


def _recon_unclaimed(rs, ev):
    A.add_event(rs, ev, event_id="e3", actuation_id="act-9999", consequence_hash="99" * 32,
                cmd="git push --force", reconstructed=True)


# name, mutation, rechain, expected (falsifier, term), exact clause. Every refusal
# clause in tools/aloop_falsifiers.py has >=1 row here whose ONLY witness is that clause
# (asserted by clause id, not family), so replacing any clause with `pass` fails the
# suite (court round 2, affidavit#85: 10/30 clauses had no witness at 01d17e40).
ADVERSARIAL = [
    ("stale_subject", lambda rs, ev: rs[1]["identity"].__setitem__("subject_sha", "2" * 40),
     False, ("F09", "R_missing_identity"), "F09.subject_mismatch"),
    ("unauthorized_self_grant", lambda rs, ev: ev[2].__setitem__("authority_grant", "self-granted"),
     True, ("F10", "R_missing_authority"), "F10.authority"),
    ("unauthorized_actor", lambda rs, ev: ev[0].__setitem__("actor", "intruder"),
     True, ("F10", "R_missing_authority"), "F10.authority"),
    ("wrong_digest_payload_tamper", lambda rs, ev: ev[1].__setitem__("cmd", "rm -rf /"),
     False, ("F05", "R_missing_identity"), "F05.hash"),
    ("prev_hash_relinked_and_rehashed", _relink, False, ("F05", "R_missing_identity"), "F05.prev_link"),
    ("wrong_chain_head", lambda rs, ev: rs[0]["replay_binding"].__setitem__("chain_head_hash", "cd" * 32),
     False, ("F05", "R_missing_identity"), "F05.head_mismatch"),
    ("replay_head_points_at_earlier_event",
     lambda rs, ev: rs[1]["replay_binding"].__setitem__("chain_head_hash", ev[0]["hash"]),
     False, ("F05", "R_missing_identity"), "F05.head_mismatch"),
    ("chain_head_hash_removed", lambda rs, ev: [r["replay_binding"].pop("chain_head_hash") for r in rs],
     False, ("F05", "R_missing_identity"), "F05.head_missing"),
    ("reordered", lambda rs, ev: ev.reverse(), False, ("F02", "mu_on_O"), "F02.order"),
    ("duplicated_seq_is_not_append_order", lambda rs, ev: ev[2].__setitem__("seq", 1),
     True, ("F02", "mu_on_O"), "F02.order"),
    ("duplicate_delivery_event", lambda rs, ev: ev.append(copy.deepcopy(ev[-1])),
     False, ("F03", "R_missing_authority"), "F03.dup"),
    ("duplicate_delivery_receipt", lambda rs, ev: rs.append(copy.deepcopy(rs[0])),
     False, ("F04", "admission_vacuous"), "F04.wo_twice"),
    ("phantom_claim", lambda rs, ev: rs[0]["replay_binding"]["event_ids"].append("ghost"),
     False, ("F01", "R_missing_replay"), "F01.phantom_claim"),
    ("double_claim", lambda rs, ev: rs[1]["replay_binding"]["event_ids"].append("e0"),
     False, ("F04", "admission_vacuous"), "F04.multi_claim"),
    ("shared_consequence_hash",
     lambda rs, ev: ev[2].__setitem__("consequence_hash", ev[1]["consequence_hash"]),
     True, ("F04", "admission_vacuous"), "F04.consequence_shared"),
    ("cross_work_order_claim",
     lambda rs, ev: (rs[0]["replay_binding"]["event_ids"].append("e2"),
                     rs[1]["replay_binding"].__setitem__("event_ids", ["e1"])),
     False, ("F09", "R_missing_identity"), "F09.cross_wo"),
    ("provider_execution_id_join_mismatch",
     lambda rs, ev: rs[1].__setitem__("provider_execution_id", "lane-8-run-9999"),
     False, ("F08", "R_missing_identity"), "F08.peid_join"),
    ("event_provider_rewrite", lambda rs, ev: ev[2].__setitem__("provider", "claude"),
     True, ("F08", "R_missing_identity"), "F08.provider_rewrite"),
    ("receipt_provider_name_join", lambda rs, ev: rs[1]["provider"].__setitem__("name", "claude"),
     False, ("F08", "R_missing_identity"), "F08.provider_join"),
    ("missing_actuation_id", lambda rs, ev: ev[1].pop("actuation_id"),
     True, ("F03", "R_missing_authority"), "F03.no_actuation_id"),
    ("duplicate_event_id", lambda rs, ev: ev[2].__setitem__("event_id", "e1"),
     True, ("F05", "R_missing_identity"), "F05.event_id_dup"),
    ("empty_event_id", lambda rs, ev: ev[1].__setitem__("event_id", ""),
     True, ("F05", "R_missing_identity"), "F05.event_id_missing"),
    ("unsupported_hash_algo", lambda rs, ev: ev[0].__setitem__("hash_algo", "md5"),
     False, ("F05", "R_missing_identity"), "F05.hash_algo"),
    ("prev_hash_key_absent", lambda rs, ev: ev[0].pop("prev_hash"),
     False, ("F05", "R_missing_identity"), "F05.prev_hash_key"),
    ("event_ids_not_a_list", lambda rs, ev: rs[0]["replay_binding"].__setitem__("event_ids", "e0"),
     False, ("F05", "R_missing_identity"), "F05.event_ids_type"),
    ("non_string_claim", lambda rs, ev: rs[0]["replay_binding"]["event_ids"].append(7),
     False, ("F05", "R_missing_identity"), "F05.claim_type"),
    ("seq_not_integer", lambda rs, ev: ev[1].__setitem__("seq", "1"),
     True, ("F01", "R_missing_replay"), "F01.seq_type"),
    ("seq_bool", lambda rs, ev: ev[0].__setitem__("seq", False),
     True, ("F01", "R_missing_replay"), "F01.seq_type"),
    ("negative_seq_prefix", lambda rs, ev: [e.__setitem__("seq", e["seq"] + 3) for e in ev],
     True, ("F01", "R_missing_replay"), "F01.prefix"),
    ("seq_gap", lambda rs, ev: ev[2].__setitem__("seq", 5),
     True, ("F01", "R_missing_replay"), "F01.gap"),
    ("dangling_predecessor",
     lambda rs, ev: rs[1]["replay_binding"].__setitem__("predecessor_work_order_ids", ["NOPE"]),
     False, ("F01", "R_missing_replay"), "F01.dangling_predecessor"),
    ("posthoc_offset_ts",
     lambda rs, ev: ev[2].__setitem__("recorded_at", "2026-09-25T10:00:00+02:00"),
     True, ("F11", "R_not_fed_back"), "F11.posthoc"),
    ("naive_timestamp", lambda rs, ev: ev[0].__setitem__("ts", "2026-09-25T09:00:00"),
     True, ("F11", "R_not_fed_back"), "F11.unparseable"),
    ("garbage_timestamp", lambda rs, ev: ev[0].__setitem__("recorded_at", "yesterday"),
     True, ("F11", "R_not_fed_back"), "F11.unparseable"),
    ("reconstructed_truthy_string_is_not_a_marking",
     lambda rs, ev: (ev[2].__setitem__("recorded_at", "2026-09-25T08:00:00Z"),
                     ev[2].__setitem__("reconstructed", "yes")),
     True, ("F11", "R_not_fed_back"), "F11.posthoc"),
    ("malformed_event_subject", lambda rs, ev: [e.__setitem__("subject_sha", "ZZ") for e in ev],
     True, ("F09", "R_missing_identity"), "F09.subject_format"),
    # omission bypasses (court P02-P05): leaving a field out is refused by its owner
    ("event_subject_omitted", _pop_all("subject_sha"), True,
     ("F09", "R_missing_identity"), "F09.event_field_missing"),
    ("event_provider_omitted", _pop_all("provider"), True,
     ("F08", "R_missing_identity"), "F08.event_field_missing"),
    ("event_timestamps_omitted", lambda rs, ev: (_pop_all("ts")(rs, ev), _pop_all("recorded_at")(rs, ev)),
     True, ("F11", "R_not_fed_back"), "F11.event_field_missing"),
    ("event_consequence_hash_omitted", _pop_all("consequence_hash"), True,
     ("F06", "R_missing_consequence"), "F06.event_field_missing"),
    ("event_actor_omitted", _pop_all("actor"), True,
     ("F10", "R_missing_authority"), "F10.event_field_missing"),
    ("event_type_omitted", _pop_all("event_type"), True,
     ("F05", "R_missing_identity"), "F05.event_field_missing"),
    # consequence binding (court P01, P09): a receipt cannot certify what no event produced
    ("receipt_consequence_rewrite",
     lambda rs, ev: rs[0].__setitem__("consequences", [{"hash": "ee" * 32, "kind": "file"}]),
     False, ("F06", "R_missing_consequence"), "F06.unbound_consequence"),
    ("receipt_consequence_omitted",
     lambda rs, ev: rs[0].__setitem__("consequences", [{"hash": "11" * 32, "kind": "file"}]),
     False, ("F06", "R_missing_consequence"), "F06.unreceipted_consequence"),
    ("receipt_files_changed_rewrite",
     lambda rs, ev: rs[0]["consequence"].__setitem__("files_changed", ["src/main.rs"]),
     False, ("F06", "R_missing_consequence"), "F06.field_mismatch"),
    ("event_files_changed_rewrite",
     lambda rs, ev: ev[2].__setitem__("files_changed", ["README.md"]),
     True, ("F06", "R_missing_consequence"), "F06.field_mismatch"),
    ("receipt_missing_provider", lambda rs, ev: rs[0].pop("provider"),
     False, ("F08", "R_missing_identity"), "F08.receipt_field_missing"),
    ("receipt_profile_nonconformant", lambda rs, ev: rs[0].__setitem__("exit_status", "maybe"),
     False, ("F00", "mu_on_O"), "F00.profile"),
    ("receipt_without_consequence",
     lambda rs, ev: (rs[1].__setitem__("consequence", {"commits": [], "files_changed": [], "remote_effects": []}),
                     rs[1].__setitem__("consequences", [])),
     False, ("F06", "R_missing_consequence"), "F06.empty"),
    ("unclaimed_event", lambda rs, ev: rs[1]["replay_binding"].__setitem__("event_ids", ["e0"]),
     False, ("F07", "mu_unlawful"), "F07.unclaimed"),
    # court P06: the subject cannot waive ZeroUnreceiptedActuation by marking itself
    ("unreceipted_actuation_marked_reconstructed", _recon_unclaimed, False,
     ("F07", "mu_unlawful"), "F07.unclaimed"),
]


def clauses(rep):
    return {v["clause"] for v in rep["violations"]}


@pytest.mark.parametrize("name,fn,re,want,clause", ADVERSARIAL, ids=[a[0] for a in ADVERSARIAL])
def test_adversarial_case_is_refused_with_typed_term(tmp_path, name, fn, re, want, clause):
    rep = check_json(case(tmp_path, name, fn, rechain_after=re))
    assert rep["verdict"] == "REFUSED"
    assert want in fired(rep), rep["violations"]
    assert clause in clauses(rep), (clause, rep["violations"])
    assert all(v["clause"].startswith(v["falsifier"] + ".") for v in rep["violations"])
    # the DAG never crashes on refused input
    p = cli("dag", tmp_path / name)
    assert p.returncode in (0, 1) and "Traceback" not in p.stderr


def _source_clauses():
    """Every refusal clause the verifier can emit, read from its own source."""
    src = (TOOLS / "aloop_falsifiers.py").read_text()
    lit = set(re.findall(r'V\("(F\d\d)", "(\w+)"', src))
    dyn = set(re.findall(r'V\(f, "(\w+)"', src))
    out = {f"{f}.{c}" for f, c in lit}
    out |= {f"{f}.{c}" for c in dyn for f in {f for _, f in A.EVENT_REQUIRED}}
    return out


def test_every_refusal_clause_has_a_clause_pinned_witness(tmp_path):
    """anti-vacuity over the rules layer: a clause with no witness carries no bits.
    Adding a clause without a witnessing corpus row fails here."""
    witnessed = {c for *_, c in ADVERSARIAL}
    witnessed |= {"F05.parse", "F01.empty", "F01.anchor"}  # witnessed by dedicated tests below
    # event_field_missing is one clause per owning falsifier; each owner needs a row
    missing = sorted(_source_clauses() - witnessed)
    assert missing == [], missing


def test_prev_hash_linkage_is_the_only_refusing_clause_on_a_relinked_chain(tmp_path):
    """court MAJOR (affidavit#85): with the hash clause satisfied, only prev_link sees
    a relinked chain."""
    rep = check_json(case(tmp_path, "relink", _relink))
    assert [v["clause"] for v in rep["violations"]] == ["F05.prev_link"], rep["violations"]


def test_honest_reconstruction_claimed_by_a_receipt_is_conformant_and_counted(tmp_path):
    def fn(rs, ev):
        ev[2]["recorded_at"] = "2026-09-25T08:00:00Z"
        ev[2]["reconstructed"] = True
    rep = check_json(case(tmp_path, "recon", fn, rechain_after=True))
    assert rep["verdict"] == "CONFORMANT" and rep["reconstructed"] == 1
    assert rep["reconstructed_unclaimed"] == 0 and rep["uar_count"] == 0


def test_unclaimed_reconstruction_is_an_unreceipted_actuation(tmp_path):
    rep = check_json(case(tmp_path, "recon_unclaimed", _recon_unclaimed))
    assert rep["verdict"] == "REFUSED" and rep["uar_count"] == 1
    assert rep["reconstructed"] == 1 and rep["reconstructed_unclaimed"] == 1


def test_duplicate_json_key_is_refused_as_parser_dependent(tmp_path):
    d = case(tmp_path, "dupkey")
    p = d / "events.ndjson"
    lines = p.read_text().splitlines()
    lines[1] = lines[1].replace('{"actor"', '{"cmd": "rm -rf /", "actor"', 1)
    p.write_text("\n".join(lines) + "\n")
    rep = check_json(d)
    assert rep["verdict"] == "REFUSED" and "F05.parse" in clauses(rep)
    assert any("duplicate key" in v["detail"] for v in rep["violations"])


def test_duplicate_json_key_in_receipt_is_refused(tmp_path):
    d = case(tmp_path, "dupkey_receipt")
    rp = sorted((d / "receipts").glob("*.json"))[0]
    rp.write_text(rp.read_text().replace("{", '{"work_order_id": "forged",', 1))
    rep = check_json(d)
    assert rep["verdict"] == "REFUSED" and "F05.parse" in clauses(rep)


def test_external_anchor_detects_suffix_truncation_with_its_receipt(tmp_path):
    """court MINOR P08: dropping the tail event AND its receipt is invisible from inside
    the case; an externally held anchor (the tail hash) refuses it."""
    rs, ev = A.golden_case()
    anchor = ev[-1]["hash"]
    d = case(tmp_path, "trunc", lambda rs, ev: (ev.pop(2), rs.pop(1)))
    assert check_json(d)["verdict"] == "CONFORMANT"  # documented limitation without anchor
    p = cli("check", d, "--anchor", anchor)
    rep = json.loads(p.stdout)
    assert p.returncode == 1 and "F01.anchor" in clauses(rep)
    full = cli("check", GOLDEN, "--anchor", anchor)
    assert full.returncode == 0 and json.loads(full.stdout)["anchor"] == anchor


def test_emit_golden_refuses_to_wipe_a_foreign_directory(tmp_path):
    """court MAJOR: emit-golden previously rmtree'd any path it was given."""
    victim = tmp_path / "victim"
    (victim / "sub").mkdir(parents=True)
    (victim / "sub" / "keep.txt").write_text("precious\n")
    (victim / "notes.md").write_text("mine\n")
    p = cli("emit-golden", victim)
    assert p.returncode == 2 and "REFUSED(UNSAFE_OVERWRITE)" in p.stderr
    assert (victim / "sub" / "keep.txt").read_text() == "precious\n"
    assert (victim / "notes.md").read_text() == "mine\n"
    # a directory holding a stray non-json file under receipts/ is also foreign
    d = case(tmp_path, "stray")
    (d / "receipts" / "README").write_text("x\n")
    assert cli("emit-golden", d).returncode == 2 and (d / "receipts" / "README").exists()


def test_emit_golden_regenerates_an_existing_case_in_place(tmp_path):
    d = case(tmp_path, "regen", lambda rs, ev: rs.append(copy.deepcopy(rs[0])))
    assert len(list((d / "receipts").glob("*.json"))) == 3
    assert cli("emit-golden", d).returncode == 0
    assert len(list((d / "receipts").glob("*.json"))) == 2
    assert check_json(d)["verdict"] == "CONFORMANT"


def test_dag_lists_dangling_predecessor_instead_of_dropping_it_silently(tmp_path):
    d = case(tmp_path, "dangling",
             lambda rs, ev: rs[1]["replay_binding"].__setitem__("predecessor_work_order_ids", ["NOPE"]))
    g = A.build_dag(d)
    assert any("dangling" in x for x in g["dropped"])


@pytest.mark.parametrize("name,content", [
    ("truncated_json_line", "{\"seq\": 0, \"event_id\""),
    ("array_line", "[1, 2, 3]"),
    ("scalar_line", "42"),
    ("null_line", "null"),
])
def test_malformed_event_lines_are_refused_not_crashed(tmp_path, name, content):
    d = case(tmp_path, name)
    with (d / "events.ndjson").open("a") as f:
        f.write(content + "\n")
    rep = check_json(d)
    assert rep["verdict"] == "REFUSED" and ("F05", "R_missing_identity") in fired(rep)
    assert "F05.parse" in clauses(rep)
    assert all(isinstance(v, dict) for v in rep["violations"])


@pytest.mark.parametrize("body", ["[]", "\"receipt\"", "{not json", "null"])
def test_malformed_receipt_files_are_refused_not_crashed(tmp_path, body):
    d = case(tmp_path, "bad_receipt")
    (d / "receipts" / "zz_bad.json").write_text(body)
    rep = check_json(d)
    assert rep["verdict"] == "REFUSED" and ("F05", "R_missing_identity") in fired(rep)
    assert "Traceback" not in cli("dag", d).stderr


def test_invalid_utf8_is_refused(tmp_path):
    d = case(tmp_path, "bad_utf8")
    (d / "events.ndjson").write_bytes(b"\xff\xfe\x00garbage\n")
    rep = check_json(d)
    assert rep["verdict"] == "REFUSED"


def test_empty_case_is_refused(tmp_path):
    (tmp_path / "empty").mkdir()
    rep = check_json(tmp_path / "empty")
    assert rep["verdict"] == "REFUSED" and ("F01", "R_missing_replay") in fired(rep)
    assert "F01.empty" in clauses(rep)


def test_events_without_any_receipt_count_every_uar(tmp_path):
    d = case(tmp_path, "no_receipts")
    for p in (d / "receipts").glob("*.json"):
        p.unlink()
    rep = check_json(d)
    assert rep["uar_count"] == 3


def test_mutation_of_a_single_byte_changes_the_verdict(tmp_path):
    """anti-vacuity: the gate refuses a one-byte payload change in the committed golden."""
    d = tmp_path / "flip"
    A.write_case(d, *A.golden_case())
    raw = (d / "events.ndjson").read_text().replace("dogfood scan", "dogfood scaN", 1)
    (d / "events.ndjson").write_text(raw)
    assert check_json(d)["verdict"] == "REFUSED"


# ------------------------------------------------------------- DAG

def test_self_predecessor_is_a_cycle(tmp_path):
    d = case(tmp_path, "selfloop",
             lambda rs, ev: rs[0]["replay_binding"].__setitem__("predecessor_work_order_ids", [rs[0]["work_order_id"]]))
    assert not A.detect_cycles(A.build_dag(d))["acyclic"]


def test_dag_drops_idless_nodes_instead_of_crashing(tmp_path):
    d = case(tmp_path, "idless", lambda rs, ev: (ev[1].pop("event_id"), rs[1].pop("work_order_id")),
             rechain_after=False)
    g = A.build_dag(d)
    assert len(g["dropped"]) == 2
    assert A.detect_cycles(g)["acyclic"]


# ------------------------------------------------------------- benchmark regression bound

# Measured 2026-09-25 on darwin arm64, python 3.14.3 (tools/bench/aloop_falsifiers.bench.json):
# check ~125-140 us/event with profile validation, dag ~15-22 us/event, linear 1k..50k.
# Bounds use process CPU time (host load average was >200 during measurement, so wall
# time is not a stable oracle) and carry >=7x headroom; the scaling bound catches an
# accidental O(n^2) (10x more events must cost < 25x more CPU; quadratic is ~100x).
CHECK_US_PER_EVENT_MAX = 1000.0
DAG_US_PER_EVENT_MAX = 250.0
SCALING_RATIO_MAX = 25.0


def test_benchmark_regression_bound(tmp_path):
    rows = B.run([2000, 20000], repeat=3, root=tmp_path)
    small, big = rows
    assert big["check_cpu_us_per_event"] <= CHECK_US_PER_EVENT_MAX, rows
    assert big["dag_cpu_us_per_event"] <= DAG_US_PER_EVENT_MAX, rows
    assert big["check_cpu_s"] / max(small["check_cpu_s"], 1e-6) <= SCALING_RATIO_MAX, rows
    assert big["dag_cpu_s"] / max(small["dag_cpu_s"], 1e-6) <= SCALING_RATIO_MAX, rows


def test_committed_bench_receipt_is_well_formed():
    rec = json.loads((TOOLS / "bench/aloop_falsifiers.bench.json").read_text())
    assert rec["tool"] == "tools/aloop_falsifiers.py" and rec["profile_validation"] is True
    assert [r["events"] for r in rec["rows"]] == [1000, 10000, 50000]
    for row in rec["rows"]:
        assert row["check_cpu_us_per_event"] <= CHECK_US_PER_EVENT_MAX
        assert row["dag_cpu_us_per_event"] <= DAG_US_PER_EVENT_MAX
