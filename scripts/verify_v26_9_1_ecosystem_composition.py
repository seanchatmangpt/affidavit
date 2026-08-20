#!/usr/bin/env python3
from __future__ import annotations
import argparse, copy, hashlib, json, re
from pathlib import Path
HEX40=re.compile(r"^[0-9a-f]{40}$")
SCHEMA="chatman.v26.9.1.ecosystem-composition/1"
RECEIPT_SCHEMA="chatman.v26.9.1.ecosystem-composition-receipt/1"
EXPECTED_REPOS={"chatman-ecosystem","ggen","ggen-marketplace","autofde-lab","autofde","bcinr","mfw","gymact","affidavit"}
EXPECTED_EDGES={"public_semantic_o_star","rdf_composition","qlever_structural_similarity","planner_league_meta_selection_falsification","authoritative_cmca","mfw_consumption","executable_domain_world_court","brce_boundary","independent_observation","canonical_receipt_ocel","bcre_ecosystem_composition","deterministic_manufacture","deterministic_marketplace_manufacture","production_runtime_pinning"}
class Refusal(RuntimeError): pass
def canon(v): return json.dumps(v,sort_keys=True,separators=(",",":")).encode()
def digest(v): return hashlib.sha256(canon(v)).hexdigest()
def load(p):
    v=json.loads(Path(p).read_text())
    if not isinstance(v,dict): raise Refusal("REFUSED:COMPOSITION_OBJECT_REQUIRED")
    return v
def verify(m):
    if m.get("schema")!=SCHEMA: raise Refusal("REFUSED:SCHEMA")
    if m.get("subject")!="v26.9.1-release-crown-composition": raise Refusal("REFUSED:SUBJECT")
    base=m.get("base",{})
    if base.get("repository")!="seanchatmangpt/affidavit" or not HEX40.fullmatch(str(base.get("sha",""))): raise Refusal("REFUSED:BASE_IDENTITY")
    s=m.get("separations",{})
    if s.get("planner_policy_role_agent_authority")!="DISTINCT": raise Refusal("REFUSED:ROLE_COLLAPSE")
    if s.get("bcre_brce")!="DISTINCT_UNLESS_EQUIVALENCE_PROOF": raise Refusal("REFUSED:BCRE_BRCE_COLLAPSE")
    if s.get("live_azure")!="BLOCKED:LIVE_AZURE_AUTHORITY": raise Refusal("REFUSED:LIVE_AZURE_AUTHORITY")
    if s.get("ww3gym")!="SIMULATION_EVALUATION_ONLY": raise Refusal("REFUSED:WW3GYM_SCOPE")
    pins=m.get("pins",{})
    if set(pins)!=EXPECTED_REPOS: raise Refusal("REFUSED:REPO_CLOSURE")
    covered=set(); identities=set()
    for repo,pin in pins.items():
        sha=str(pin.get("sha",""))
        if not HEX40.fullmatch(sha): raise Refusal(f"REFUSED:PIN_SHA:{repo}")
        ident=(repo,pin.get("ref"),sha)
        if ident in identities: raise Refusal(f"REFUSED:DUPLICATE_IDENTITY:{repo}")
        identities.add(ident)
        edges=pin.get("edges")
        if not isinstance(edges,list) or not edges: raise Refusal(f"REFUSED:PIN_EDGES:{repo}")
        covered.update(edges)
    req=m.get("required_edges")
    if not isinstance(req,list) or set(req)!=EXPECTED_EDGES: raise Refusal("REFUSED:EDGE_CLOSURE")
    if covered!=EXPECTED_EDGES: raise Refusal("REFUSED:EDGE_COVERAGE")
    p=m.get("standing_policy",{})
    if p!={"pin_is_not_alive":True,"all_edge_alive_requires_exact_execution_receipts":True,"composition_admission_can_be_alive_while_release_crown_is_partial":True}: raise Refusal("REFUSED:STANDING_TRANSFER")
    return {"schema":RECEIPT_SCHEMA,"subject":m["subject"],"manifest_sha256":digest(m),"base_sha":base["sha"],"pin_count":len(pins),"edge_count":len(req),"composition_standing":"ALIVE","release_crown_standing":"PARTIAL_ALIVE","live_azure":"BLOCKED:LIVE_AZURE_AUTHORITY","ww3gym":"SIMULATION_EVALUATION_ONLY","standing_transfer":False}
def seal(r):
    x=dict(r); x["receipt_sha256"]=digest(r); return x
def replay(r):
    d=dict(r); got=d.pop("receipt_sha256",None)
    if got!=digest(d): raise Refusal("REFUSED:RECEIPT_TAMPER")
    if d.get("schema")!=RECEIPT_SCHEMA or d.get("composition_standing")!="ALIVE": raise Refusal("REFUSED:RECEIPT_STANDING")
    return d
def self_test():
    base=json.loads(Path("profiles/v26.9.1-ecosystem-composition.json").read_text()); assert verify(base)["release_crown_standing"]=="PARTIAL_ALIVE"; n=1
    mutations=[("ROLE_COLLAPSE",lambda x:x["separations"].__setitem__("planner_policy_role_agent_authority","COLLAPSED")),("BCRE_BRCE_COLLAPSE",lambda x:x["separations"].__setitem__("bcre_brce","EQUIVALENT")),("LIVE_AZURE_AUTHORITY",lambda x:x["separations"].__setitem__("live_azure","ALIVE")),("WW3GYM_SCOPE",lambda x:x["separations"].__setitem__("ww3gym","PRODUCTION")),("PIN_SHA",lambda x:x["pins"]["gymact"].__setitem__("sha","latest")),("EDGE_COVERAGE",lambda x:x["pins"]["ggen"].__setitem__("edges",["deterministic_marketplace_manufacture"])),("STANDING_TRANSFER",lambda x:x["standing_policy"].__setitem__("pin_is_not_alive",False))]
    for code,mut in mutations:
        x=copy.deepcopy(base); mut(x)
        try: verify(x)
        except Refusal as e: assert code in str(e),(code,e)
        else: raise AssertionError(code)
        n+=1
    r=seal(verify(base)); replay(r); r["release_crown_standing"]="ALIVE"
    try: replay(r)
    except Refusal as e: assert "RECEIPT_TAMPER" in str(e)
    else: raise AssertionError("tamper")
    return n+1
def main():
    ap=argparse.ArgumentParser(); ap.add_argument("--manifest"); ap.add_argument("--receipt"); ap.add_argument("--replay"); ap.add_argument("--self-test",action="store_true"); a=ap.parse_args()
    try:
        if a.self_test:
            n=self_test(); print(f"ALIVE:V26_9_1_COMPOSITION_FALSIFIERS tests={n}"); return 0
        if a.replay:
            d=replay(load(a.replay)); print(f"ALIVE:V26_9_1_COMPOSITION_REPLAY manifest={d['manifest_sha256']} crown={d['release_crown_standing']}"); return 0
        if not a.manifest: raise Refusal("REFUSED:MANIFEST_REQUIRED")
        r=seal(verify(load(a.manifest)))
        if a.receipt: Path(a.receipt).write_text(json.dumps(r,sort_keys=True,indent=2)+"\n")
        print(f"ALIVE:V26_9_1_COMPOSITION manifest={r['manifest_sha256']} crown={r['release_crown_standing']} pins={r['pin_count']} edges={r['edge_count']}"); return 0
    except (Refusal,AssertionError,ValueError,KeyError) as e:
        print(str(e)); return 2
if __name__=="__main__": raise SystemExit(main())
