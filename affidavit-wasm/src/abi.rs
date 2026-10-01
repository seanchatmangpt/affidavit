//! The JSON ABI: one request in, one response out. Pure, total, native-testable.
//!
//! Ops: `capabilities`, `commit`, `assemble`, `verify`, `mine`, `conform`.
//! Every failure is a structured `{"ok":false,"error":{"code","message"}}`;
//! [`call`] never panics on any input.

use crate::abi_meta::{ERROR_CODES, MAX_JSON_DEPTH, OPS};
use crate::crypto;
use crate::external_evidence;
use crate::receipt::{self, ObjectRef, OperationEvent, Receipt};
use affidavit_core::mining::conformance::replay;
use affidavit_core::mining::{
    AlphaRelation, DirectlyFollowsGraph, Footprint, LogStatistics, Trace,
};
use serde_json::{json, Map, Value};

// ABI revision and request limit come from the generated `abi_meta` (rendered
// from `ontology/affi-wasm.ttl`); they are re-exported here for callers.
pub use crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES};

/// Footprints are quadratic in activities; beyond this they are omitted.
const MAX_FOOTPRINT_ACTIVITIES: usize = 256;

/// A structured, host-readable failure.
#[derive(Debug, PartialEq, Eq)]
pub struct AbiError {
    /// Stable machine-readable code; always one of [`ERROR_CODES`].
    pub code: &'static str,
    /// Human-readable explanation.
    pub message: String,
    /// Extra typed fields merged into the `error` object (limit failures).
    pub details: Option<Value>,
}

pub(crate) fn err(code: &'static str, message: impl Into<String>) -> AbiError {
    AbiError {
        code,
        message: message.into(),
        details: None,
    }
}

/// A typed resource-limit failure: `too_large` (bytes) or `too_deep` (nesting).
fn limit(name: &str, observed: usize, max: usize) -> AbiError {
    let code = if name == "json_depth" {
        "too_deep"
    } else {
        "too_large"
    };
    AbiError {
        code,
        message: format!("resource limit `{name}` exceeded: {observed} > {max}"),
        details: Some(json!({"limit": name, "observed": observed, "max": max})),
    }
}

fn error_body(e: AbiError) -> Value {
    let mut error = json!({"code": e.code, "message": e.message});
    if let (Some(Value::Object(extra)), Some(map)) = (e.details, error.as_object_mut()) {
        map.extend(extra);
    }
    json!({"ok": false, "error": error})
}

/// Maximum bracket nesting of a JSON text (string-aware, allocation-free).
fn json_depth(b: &[u8]) -> usize {
    let (mut depth, mut max, mut in_str, mut esc) = (0usize, 0usize, false, false);
    for &c in b {
        if in_str {
            if esc {
                esc = false;
            } else if c == b'\\' {
                esc = true;
            } else if c == b'"' {
                in_str = false;
            }
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'[' | b'{' => {
                depth += 1;
                max = max.max(depth);
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    max
}

/// Typed response for a request over a resource limit (used by the FFI shell).
pub fn limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> {
    encode(error_body(limit(name, observed, max)))
}

/// Typed response for a call whose request buffer does not exist (`af_alloc`
/// refused it, or it was never allocated).
pub fn missing_buffer_response() -> Vec<u8> {
    encode(error_body(err(
        "missing_buffer",
        "request buffer missing: af_alloc refused or was never called",
    )))
}

fn encode(v: Value) -> Vec<u8> {
    // Serializing a `Value` cannot fail; the fallback keeps the ABI total anyway.
    serde_json::to_vec(&v).unwrap_or_else(|_| {
        br#"{"ok":false,"error":{"code":"internal","message":"encode"}}"#.to_vec()
    })
}

type Res<T> = Result<T, AbiError>;

/// Execute one UTF-8 JSON request and return the UTF-8 JSON response.
pub fn call(request: &[u8]) -> Vec<u8> {
    let response = match dispatch(request) {
        Ok(mut body) => {
            body.insert("ok".into(), Value::Bool(true));
            Value::Object(body)
        }
        Err(e) => error_body(e),
    };
    encode(response)
}

fn dispatch(request: &[u8]) -> Res<Map<String, Value>> {
    if request.len() > MAX_REQUEST_BYTES {
        return Err(limit("request_bytes", request.len(), MAX_REQUEST_BYTES));
    }
    let depth = json_depth(request);
    if depth > MAX_JSON_DEPTH {
        return Err(limit("json_depth", depth, MAX_JSON_DEPTH));
    }
    let req: Value = serde_json::from_slice(request).map_err(|e| err("bad_json", e.to_string()))?;
    let op = req
        .get("op")
        .and_then(Value::as_str)
        .ok_or_else(|| err("missing_field", "request needs a string field `op`"))?;
    let mut out = match op {
        "capabilities" => capabilities(),
        "commit" => commit(&req)?,
        "assemble" => assemble(&req)?,
        "verify" => verify(&req)?,
        "mine" => mine(&req)?,
        "conform" => conform(&req)?,
        "verify_signature_input" => crypto::op_verify_signature_input(&req)?,
        "certify_authzen_evidence" => external_evidence::op_certify_authzen_evidence(&req)?,
        "certify_spiffe_evidence" => external_evidence::op_certify_spiffe_evidence(&req)?,
        other => {
            return Err(err(
                "unknown_op",
                format!("unknown op `{other}`; known ops: {}", OPS.join(", ")),
            ))
        }
    };
    out.insert("op".into(), Value::String(op.to_string()));
    Ok(out)
}

pub(crate) fn obj(v: Value) -> Map<String, Value> {
    match v {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn capabilities() -> Map<String, Value> {
    obj(json!({
        "module": "affidavit-wasm",
        "version": env!("CARGO_PKG_VERSION"),
        "abi_version": ABI_VERSION,
        "format_version": receipt::FORMAT_VERSION,
        "genesis_seed": receipt::GENESIS_SEED,
        "hash": "blake3",
        "ops": OPS,
        "limits": {"max_request_bytes": MAX_REQUEST_BYTES, "max_json_depth": MAX_JSON_DEPTH},
        "error_codes": ERROR_CODES,
    }))
}

// ---- field helpers ---------------------------------------------------------

pub(crate) fn field<'a>(req: &'a Value, name: &str) -> Res<&'a Value> {
    req.get(name)
        .ok_or_else(|| err("missing_field", format!("missing field `{name}`")))
}

fn parse_receipt(v: &Value, what: &str) -> Res<Receipt> {
    serde_json::from_value(v.clone()).map_err(|e| err("bad_field", format!("{what}: {e}")))
}

pub(crate) fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let digit = |c: u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    };
    s.as_bytes()
        .chunks(2)
        .map(|p| Some(digit(p[0])? << 4 | digit(p[1])?))
        .collect()
}

/// Payload bytes from `payload` (UTF-8 text) or `payload_hex`.
fn payload_bytes(v: &Value, ctx: &str) -> Res<Option<Vec<u8>>> {
    if let Some(p) = v.get("payload") {
        let s = p
            .as_str()
            .ok_or_else(|| err("bad_field", format!("{ctx}: `payload` must be a string")))?;
        return Ok(Some(s.as_bytes().to_vec()));
    }
    if let Some(p) = v.get("payload_hex") {
        let s = p.as_str().ok_or_else(|| {
            err(
                "bad_field",
                format!("{ctx}: `payload_hex` must be a string"),
            )
        })?;
        return hex_decode(s).map(Some).ok_or_else(|| {
            err(
                "bad_field",
                format!("{ctx}: `payload_hex` is not valid hex"),
            )
        });
    }
    Ok(None)
}

// ---- ops -------------------------------------------------------------------

fn commit(req: &Value) -> Res<Map<String, Value>> {
    let bytes = payload_bytes(req, "commit")?
        .ok_or_else(|| err("missing_field", "commit needs `payload` or `payload_hex`"))?;
    Ok(obj(
        json!({"commitment": receipt::blake3_hex(&bytes), "bytes": bytes.len()}),
    ))
}

/// `id:type[:qualifier]` (the `affi emit --object` form) or an object.
fn parse_object(v: &Value, ctx: &str) -> Res<ObjectRef> {
    match v {
        Value::String(s) => {
            let mut parts = s.splitn(3, ':');
            let (id, ty) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
            if id.is_empty() || ty.is_empty() {
                return Err(err(
                    "bad_field",
                    format!("{ctx}: object `{s}` must look like id:type[:qualifier]"),
                ));
            }
            Ok(ObjectRef {
                id: id.into(),
                obj_type: ty.into(),
                qualifier: parts.next().filter(|q| !q.is_empty()).map(str::to_string),
            })
        }
        Value::Object(_) => serde_json::from_value(v.clone())
            .map_err(|e| err("bad_field", format!("{ctx}: object: {e}"))),
        _ => Err(err(
            "bad_field",
            format!("{ctx}: object must be a string or an object"),
        )),
    }
}

fn assemble(req: &Value) -> Res<Map<String, Value>> {
    let events = field(req, "events")?
        .as_array()
        .ok_or_else(|| err("bad_field", "`events` must be an array"))?;
    let mut built = Vec::with_capacity(events.len());
    for (i, e) in events.iter().enumerate() {
        let ctx = format!("events[{i}]");
        let event_type = e
            .get("event_type")
            .and_then(Value::as_str)
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| {
                err(
                    "bad_field",
                    format!("{ctx}: needs a non-empty string `event_type`"),
                )
            })?;
        let commitment = match (payload_bytes(e, &ctx)?, e.get("payload_commitment")) {
            (Some(bytes), None) => receipt::blake3_hex(&bytes),
            (None, Some(c)) => {
                let c = c.as_str().ok_or_else(|| {
                    err(
                        "bad_field",
                        format!("{ctx}: `payload_commitment` must be a string"),
                    )
                })?;
                let ok = c.len() == 64
                    && c.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
                if !ok {
                    return Err(err(
                        "bad_field",
                        format!("{ctx}: `payload_commitment` must be 64 lowercase hex chars"),
                    ));
                }
                c.to_string()
            }
            _ => {
                return Err(err(
                    "bad_field",
                    format!(
                        "{ctx}: give exactly one of `payload`, `payload_hex`, `payload_commitment`"
                    ),
                ))
            }
        };
        let objects = match e.get("objects") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(a)) => a
                .iter()
                .map(|o| parse_object(o, &ctx))
                .collect::<Res<Vec<_>>>()?,
            Some(_) => {
                return Err(err(
                    "bad_field",
                    format!("{ctx}: `objects` must be an array"),
                ))
            }
        };
        let id = match e.get("id") {
            None | Some(Value::Null) => format!("evt-{i}"),
            Some(v) => v
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    err(
                        "bad_field",
                        format!("{ctx}: `id` must be a non-empty string"),
                    )
                })?
                .to_string(),
        };
        built.push(OperationEvent {
            id,
            seq: i as u64,
            event_type: event_type.to_string(),
            objects,
            payload_commitment: commitment,
        });
    }
    let sealed = receipt::seal(built).map_err(|e| err("internal", e.to_string()))?;
    let address = receipt::content_address(&sealed).map_err(|e| err("internal", e.to_string()))?;
    Ok(obj(json!({
        "receipt": sealed,
        "content_address": address,
        "chain_hash": sealed.chain_hash,
    })))
}

fn verify(req: &Value) -> Res<Map<String, Value>> {
    let raw = field(req, "receipt")?;
    // A receipt that cannot even be decoded is a REJECT verdict at stage 1, not
    // an ABI error: a verifier answers "no" rather than throwing.
    let (verdict, address) = match serde_json::from_value::<Receipt>(raw.clone()) {
        Ok(r) => {
            let addr = receipt::content_address(&r).ok();
            (receipt::verify(&r), addr)
        }
        Err(e) => (
            receipt::finish(vec![receipt::CheckOutcome {
                stage: "decode".into(),
                passed: false,
                detail: format!("receipt does not decode: {e}"),
            }]),
            None,
        ),
    };
    let mut out = obj(serde_json::to_value(&verdict).map_err(|e| err("internal", e.to_string()))?);
    out.insert("content_address".into(), json!(address));
    Ok(out)
}

/// Decode `receipts` into activity sequences (one trace per receipt, in order).
fn traces_of(req: &Value, name: &str) -> Res<Vec<(Receipt, bool)>> {
    let arr = field(req, name)?.as_array().ok_or_else(|| {
        err(
            "bad_field",
            format!("`{name}` must be an array of receipts"),
        )
    })?;
    arr.iter()
        .enumerate()
        .map(|(i, v)| {
            let r = parse_receipt(v, &format!("{name}[{i}]"))?;
            let accepted = receipt::verify(&r).accepted;
            Ok((r, accepted))
        })
        .collect()
}

fn activities(r: &Receipt) -> Vec<&str> {
    r.events.iter().map(|e| e.event_type.as_str()).collect()
}

fn relation_name(r: AlphaRelation) -> &'static str {
    match r {
        AlphaRelation::Causality => "causality",
        AlphaRelation::ReverseCausality => "reverse_causality",
        AlphaRelation::Parallel => "parallel",
        AlphaRelation::Choice => "choice",
    }
}

fn mine(req: &Value) -> Res<Map<String, Value>> {
    let receipts = traces_of(req, "receipts")?;
    let acts: Vec<Vec<&str>> = receipts.iter().map(|(r, _)| activities(r)).collect();
    let traces: Vec<Trace<'_>> = acts.iter().map(|a| Trace::from_activities(a)).collect();
    let dfg = DirectlyFollowsGraph::discover(&traces);
    let stats = LogStatistics::from_traces(&traces);

    let footprint = if dfg.activities.len() <= MAX_FOOTPRINT_ACTIVITIES {
        let fp = Footprint::from_dfg(&dfg);
        let names = fp.activities();
        let mut rel = Vec::new();
        for (i, a) in names.iter().enumerate() {
            for b in &names[i..] {
                rel.push(json!({"a": a, "b": b, "relation": relation_name(fp.relation(a, b))}));
            }
        }
        Value::Array(rel)
    } else {
        Value::Null
    };

    let edges: Vec<Value> = dfg
        .edges
        .iter()
        .map(|((a, b), n)| json!({"from": a, "to": b, "count": n}))
        .collect();
    let variants: Vec<Value> = stats
        .variants
        .iter()
        .map(|(seq, n)| json!({"activities": seq, "count": n}))
        .collect();
    let rejected: Vec<usize> = receipts
        .iter()
        .enumerate()
        .filter(|(_, (_, ok))| !ok)
        .map(|(i, _)| i)
        .collect();

    Ok(obj(json!({
        "trace_count": stats.trace_count,
        "event_count": stats.event_count,
        "activities": dfg.activities,
        "edges": edges,
        "start": dfg.start,
        "end": dfg.end,
        "activity_frequency": stats.activity_frequency,
        "variants": variants,
        "distinct_variants": stats.distinct_variants(),
        "footprint": footprint,
        "footprint_omitted": footprint.is_null(),
        "all_accepted": rejected.is_empty(),
        "rejected_receipts": rejected,
    })))
}

fn conform(req: &Value) -> Res<Map<String, Value>> {
    let model_receipts = traces_of(req, "model")?;
    let subject = parse_receipt(field(req, "trace")?, "trace")?;
    let subject_accepted = receipt::verify(&subject).accepted;

    let model_acts: Vec<Vec<&str>> = model_receipts.iter().map(|(r, _)| activities(r)).collect();
    let model_traces: Vec<Trace<'_>> = model_acts
        .iter()
        .map(|a| Trace::from_activities(a))
        .collect();
    let model = DirectlyFollowsGraph::discover(&model_traces);

    let subject_acts = activities(&subject);
    let result = replay(&model, &Trace::from_activities(&subject_acts));
    let fitness = result.fitness();
    Ok(obj(json!({
        "verdict": if result.is_conformant() { "conformant" } else { "non_conformant" },
        "fitness": if fitness.is_finite() { fitness } else { 0.0 },
        "legal_moves": result.legal_moves,
        "total_moves": result.total_moves,
        "start_ok": result.start_ok,
        "end_ok": result.end_ok,
        "first_violation": result.first_violation,
        "model_traces": model_traces.len(),
        "trace_accepted": subject_accepted,
        "model_all_accepted": model_receipts.iter().all(|(_, ok)| *ok),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(v: Value) -> Value {
        serde_json::from_slice(&call(&serde_json::to_vec(&v).unwrap())).unwrap()
    }

    fn assembled(types: &[&str]) -> Value {
        let events: Vec<Value> = types
            .iter()
            .map(|t| json!({"event_type": t, "payload": format!("payload-{t}"), "objects": ["art1:artifact:input"]}))
            .collect();
        let r = run(json!({"op": "assemble", "events": events}));
        assert_eq!(r["ok"], true, "{r}");
        r["receipt"].clone()
    }

    #[test]
    fn capabilities_advertise_the_release_identity() {
        let r = run(json!({"op": "capabilities"}));
        assert_eq!(r["ok"], true);
        assert_eq!(r["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(r["abi_version"], ABI_VERSION);
        assert_eq!(r["format_version"], "core/v1");
        assert_eq!(
            r["genesis_seed"],
            format!("affidavit-v{}-genesis", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(r["ops"].as_array().unwrap().len(), OPS.len());
    }

    #[test]
    fn assemble_then_verify_round_trips_and_tamper_is_rejected() {
        let receipt = assembled(&["build", "test"]);
        let v = run(json!({"op": "verify", "receipt": receipt}));
        assert_eq!(v["accepted"], true, "{v}");
        assert_eq!(v["outcomes"].as_array().unwrap().len(), 6);

        let mut tampered = receipt.clone();
        tampered["events"][1]["payload_commitment"] = json!(receipt::blake3_hex(b"evil"));
        let v = run(json!({"op": "verify", "receipt": tampered}));
        assert_eq!(v["ok"], true);
        assert_eq!(v["accepted"], false);
        assert!(v["reason"]
            .as_str()
            .unwrap()
            .starts_with("chain_integrity:"));
    }

    #[test]
    fn assemble_is_deterministic_and_matches_commit() {
        let a = assembled(&["build"]);
        assert_eq!(a, assembled(&["build"]));
        let c = run(json!({"op": "commit", "payload": "payload-build"}));
        assert_eq!(a["events"][0]["payload_commitment"], c["commitment"]);
        let hex = run(json!({"op": "commit", "payload_hex": "7061796c6f61642d6275696c64"}));
        assert_eq!(hex["commitment"], c["commitment"]);
    }

    #[test]
    fn object_shorthand_matches_the_cli_form() {
        let r = assembled(&["seed"]);
        assert_eq!(
            r["events"][0]["objects"][0],
            json!({"id": "art1", "obj_type": "artifact", "qualifier": "input"})
        );
    }

    #[test]
    fn undecodable_receipt_is_a_reject_verdict_not_an_abi_error() {
        let v = run(json!({"op": "verify", "receipt": {"nonsense": true}}));
        assert_eq!(v["ok"], true);
        assert_eq!(v["accepted"], false);
        assert!(v["reason"].as_str().unwrap().starts_with("decode:"));
    }

    #[test]
    fn mine_discovers_the_dfg_and_flags_rejected_receipts() {
        let a = assembled(&["build", "test", "deploy"]);
        let b = assembled(&["build", "test", "deploy"]);
        let mut bad = assembled(&["build", "deploy"]);
        bad["chain_hash"] = json!(receipt::blake3_hex(b"forged"));
        let m = run(json!({"op": "mine", "receipts": [a, b, bad]}));
        assert_eq!(m["ok"], true, "{m}");
        assert_eq!(m["trace_count"], 3);
        assert_eq!(m["event_count"], 8);
        assert_eq!(m["activities"], json!(["build", "deploy", "test"]));
        let edge = m["edges"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["from"] == "build" && e["to"] == "test")
            .unwrap();
        assert_eq!(edge["count"], 2);
        assert_eq!(m["distinct_variants"], 2);
        assert_eq!(m["all_accepted"], false);
        assert_eq!(m["rejected_receipts"], json!([2]));
        let rel = m["footprint"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["a"] == "build" && r["b"] == "test")
            .unwrap();
        assert_eq!(rel["relation"], "causality");
    }

    #[test]
    fn conform_replays_a_trace_against_a_discovered_model() {
        let model = vec![assembled(&["build", "test", "deploy"])];
        let ok = run(
            json!({"op": "conform", "model": model, "trace": assembled(&["build", "test", "deploy"])}),
        );
        assert_eq!(ok["verdict"], "conformant", "{ok}");
        assert_eq!(ok["fitness"], 1.0);

        let bad =
            run(json!({"op": "conform", "model": model, "trace": assembled(&["build", "deploy"])}));
        assert_eq!(bad["verdict"], "non_conformant");
        assert_eq!(bad["first_violation"], 0);
        assert_eq!(bad["legal_moves"], 0);
    }

    #[test]
    fn limit_failures_are_typed_with_observed_and_max() {
        let r: Value = serde_json::from_slice(&limit_response("request_bytes", 9, 4)).unwrap();
        assert_eq!(r["error"]["code"], "too_large");
        assert_eq!(r["error"]["limit"], "request_bytes");
        assert_eq!(r["error"]["observed"], 9);
        assert_eq!(r["error"]["max"], 4);
        let d: Value = serde_json::from_slice(&limit_response("json_depth", 65, 64)).unwrap();
        assert_eq!(d["error"]["code"], "too_deep");
        let m: Value = serde_json::from_slice(&missing_buffer_response()).unwrap();
        assert_eq!(m["error"]["code"], "missing_buffer");
    }

    #[test]
    fn depth_counts_brackets_outside_strings_only() {
        assert_eq!(json_depth(br#"{"a":"[[[[","b":[1,{"c":2}]}"#), 3);
        assert_eq!(json_depth(br#""\"[[""#), 0);
        let at_limit = format!(
            "{}{}",
            "[".repeat(MAX_JSON_DEPTH),
            "]".repeat(MAX_JSON_DEPTH)
        );
        let r: Value = serde_json::from_slice(&call(at_limit.as_bytes())).unwrap();
        assert_eq!(r["error"]["code"], "missing_field"); // parsed, not limit-refused
    }

    #[test]
    fn every_emitted_error_code_is_declared() {
        for req in [
            &b"junk"[..],
            b"{}",
            br#"{"op":"nope"}"#,
            br#"{"op":"verify"}"#,
            br#"{"op":"mine","receipts":[{"x":1}]}"#,
        ] {
            let r: Value = serde_json::from_slice(&call(req)).unwrap();
            let code = r["error"]["code"].as_str().unwrap();
            assert!(ERROR_CODES.contains(&code), "undeclared code {code}");
        }
        assert_eq!(
            run(json!({"op": "capabilities"}))["error_codes"],
            json!(ERROR_CODES)
        );
    }

    #[test]
    fn errors_are_structured_and_the_abi_is_total() {
        let code = |req: &[u8]| -> String {
            let r: Value = serde_json::from_slice(&call(req)).unwrap();
            assert_eq!(r["ok"], false, "{r}");
            r["error"]["code"].as_str().unwrap().to_string()
        };
        assert_eq!(code(b"not json"), "bad_json");
        assert_eq!(code(b"{}"), "missing_field");
        assert_eq!(code(br#"{"op":"nope"}"#), "unknown_op");
        assert_eq!(code(br#"{"op":"verify"}"#), "missing_field");
        assert_eq!(
            code(br#"{"op":"assemble","events":[{"event_type":"x"}]}"#),
            "bad_field"
        );
        assert_eq!(
            code(br#"{"op":"assemble","events":[{"event_type":"","payload":"p"}]}"#),
            "bad_field"
        );
        assert_eq!(
            code(br#"{"op":"assemble","events":[{"event_type":"x","payload_commitment":"ABC"}]}"#),
            "bad_field"
        );
        assert_eq!(code(br#"{"op":"mine","receipts":[{"x":1}]}"#), "bad_field");
        assert_eq!(code(&vec![b' '; MAX_REQUEST_BYTES + 1]), "too_large");
        let deep = format!(
            "{}{}",
            "[".repeat(MAX_JSON_DEPTH + 1),
            "]".repeat(MAX_JSON_DEPTH + 1)
        );
        assert_eq!(code(deep.as_bytes()), "too_deep");
        // Arbitrary bytes never panic.
        for junk in [
            &b""[..],
            b"\xff\xfe",
            b"[]",
            b"null",
            b"\"op\"",
            b"{\"op\":1}",
        ] {
            let _ = call(junk);
        }
    }
}
