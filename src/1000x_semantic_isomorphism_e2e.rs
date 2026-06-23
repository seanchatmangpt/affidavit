// 1000X COMBINATORIAL MAXIMALISM: Semantic Preservation Tests
//
// E2E suite that translates a receipt through 5 different format standard
// versions (v1 -> v2 -> v3 -> v4 -> v5 -> v1) ensuring strict isomorphism
// and 0% semantic loss.
//
// Every version carries exactly the same semantic payload but uses a different
// structural "standard". The test proves that the provenance data survives
// standard evolution without corruption or data drop.

use crate::types::{Blake3Hash, ObjectRef, OperationEvent, Receipt};
use serde::{Deserialize, Serialize};

/// Semantic payload of a receipt, decoupled from any specific format version.
/// This is what we must preserve with 100% fidelity.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticReceipt {
    format_version: String,
    events: Vec<OperationEvent>,
    chain_hash: Blake3Hash,
}

impl From<Receipt> for SemanticReceipt {
    fn from(r: Receipt) -> Self {
        SemanticReceipt {
            format_version: r.format_version,
            events: r.events,
            chain_hash: r.chain_hash,
        }
    }
}

// =============================================================================
// FORMAT VERSION 2: Compact (Renamed Fields)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FormatV2 {
    #[serde(rename = "fv")]
    version: String,
    #[serde(rename = "evs")]
    log: Vec<EventV2>,
    #[serde(rename = "ch")]
    hash: Blake3Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EventV2 {
    #[serde(rename = "i")]
    id: String,
    #[serde(rename = "s")]
    seq: u64,
    #[serde(rename = "t")]
    kind: String,
    #[serde(rename = "o")]
    refs: Vec<ObjectRef>,
    #[serde(rename = "p")]
    commitment: Blake3Hash,
}

impl From<SemanticReceipt> for FormatV2 {
    fn from(s: SemanticReceipt) -> Self {
        FormatV2 {
            version: s.format_version,
            hash: s.chain_hash,
            log: s.events.into_iter().map(|e| EventV2 {
                id: e.id,
                seq: e.seq,
                kind: e.event_type,
                refs: e.objects,
                commitment: e.payload_commitment,
            }).collect(),
        }
    }
}

impl From<FormatV2> for SemanticReceipt {
    fn from(v2: FormatV2) -> Self {
        SemanticReceipt {
            format_version: v2.version,
            chain_hash: v2.hash,
            events: v2.log.into_iter().map(|e| OperationEvent {
                id: e.id,
                seq: e.seq,
                event_type: e.kind,
                objects: e.refs,
                payload_commitment: e.commitment,
            }).collect(),
        }
    }
}

// =============================================================================
// FORMAT VERSION 3: Nested (Envelope + Body)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FormatV3 {
    header: HeaderV3,
    payload: BodyV3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct HeaderV3 {
    spec: String,
    root: Blake3Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BodyV3 {
    entries: Vec<OperationEvent>,
}

impl From<SemanticReceipt> for FormatV3 {
    fn from(s: SemanticReceipt) -> Self {
        FormatV3 {
            header: HeaderV3 {
                spec: s.format_version,
                root: s.chain_hash,
            },
            payload: BodyV3 {
                entries: s.events,
            },
        }
    }
}

impl From<FormatV3> for SemanticReceipt {
    fn from(v3: FormatV3) -> Self {
        SemanticReceipt {
            format_version: v3.header.spec,
            chain_hash: v3.header.root,
            events: v3.payload.entries,
        }
    }
}

// =============================================================================
// FORMAT VERSION 4: OCEL-Flat (Standardized OCEL Attribute Bag)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FormatV4 {
    version: String,
    ocel_events: Vec<OcelEventV4>,
    rolling_hash: Blake3Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct OcelEventV4 {
    id: String,
    #[serde(rename = "ocel:timestamp")]
    seq: u64, // Using seq as timestamp for isomorphism
    #[serde(rename = "ocel:activity")]
    activity: String,
    #[serde(rename = "ocel:omap")]
    objects: Vec<String>,
    #[serde(rename = "ocel:vmap")]
    attributes: std::collections::HashMap<String, String>,
}

impl From<SemanticReceipt> for FormatV4 {
    fn from(s: SemanticReceipt) -> Self {
        FormatV4 {
            version: s.format_version,
            rolling_hash: s.chain_hash,
            ocel_events: s.events.into_iter().map(|e| {
                let mut attributes = std::collections::HashMap::new();
                attributes.insert("payload_commitment".to_string(), e.payload_commitment.as_hex().to_string());
                
                // Encode objects as "id:type" or "id:type:qualifier" for recovery
                let objects = e.objects.into_iter().map(|o| {
                    match o.qualifier {
                        Some(q) => format!("{}:{}:{}", o.id, o.obj_type, q),
                        None => format!("{}:{}", o.id, o.obj_type),
                    }
                }).collect();

                OcelEventV4 {
                    id: e.id,
                    seq: e.seq,
                    activity: e.event_type,
                    objects,
                    attributes,
                }
            }).collect(),
        }
    }
}

impl From<FormatV4> for SemanticReceipt {
    fn from(v4: FormatV4) -> Self {
        SemanticReceipt {
            format_version: v4.version,
            chain_hash: v4.rolling_hash,
            events: v4.ocel_events.into_iter().map(|e| {
                let commitment_hex = e.attributes.get("payload_commitment").cloned().expect("missing commitment");
                
                let objects = e.objects.into_iter().map(|s| {
                    let parts: Vec<&str> = s.split(':').collect();
                    match parts.len() {
                        3 => ObjectRef { id: parts[0].to_string(), obj_type: parts[1].to_string(), qualifier: Some(parts[2].to_string()) },
                        2 => ObjectRef { id: parts[0].to_string(), obj_type: parts[1].to_string(), qualifier: None },
                        _ => panic!("malformed object ref in V4: {}", s),
                    }
                }).collect();

                OperationEvent {
                    id: e.id,
                    seq: e.seq,
                    event_type: e.activity,
                    objects,
                    payload_commitment: Blake3Hash::from_hex(commitment_hex),
                }
            }).collect(),
        }
    }
}

// =============================================================================
// FORMAT VERSION 5: Semantic Tuple (Minimalist Array-of-Arrays)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FormatV5(String, Blake3Hash, Vec<EventV5>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EventV5(String, u64, String, Vec<(String, String, Option<String>)>, Blake3Hash);

impl From<SemanticReceipt> for FormatV5 {
    fn from(s: SemanticReceipt) -> Self {
        FormatV5(
            s.format_version,
            s.chain_hash,
            s.events.into_iter().map(|e| EventV5(
                e.id,
                e.seq,
                e.event_type,
                e.objects.into_iter().map(|o| (o.id, o.obj_type, o.qualifier)).collect(),
                e.payload_commitment
            )).collect()
        )
    }
}

impl From<FormatV5> for SemanticReceipt {
    fn from(v5: FormatV5) -> Self {
        SemanticReceipt {
            format_version: v5.0,
            chain_hash: v5.1,
            events: v5.2.into_iter().map(|e| OperationEvent {
                id: e.0,
                seq: e.1,
                event_type: e.2,
                objects: e.3.into_iter().map(|(id, ty, q)| ObjectRef { id, obj_type: ty, qualifier: q }).collect(),
                payload_commitment: e.4,
            }).collect(),
        }
    }
}

// =============================================================================
// TEST SUITE
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainAssembler;
    use crate::ocel::{build_event, object_ref, SeqCounter};

    #[test]
    fn test_5_stage_semantic_isomorphism() {
        // --- PREPARATION: Create original receipt ---
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        
        asm.append(build_event(
            "order_placed",
            vec![object_ref("order-1", "Order"), object_ref("cust-A", "Customer")],
            b"{\"total\": 100}",
            &mut counter
        ).unwrap()).unwrap();

        asm.append(build_event(
            "payment_received",
            vec![object_ref("order-1", "Order"), object_ref("pay-99", "Transaction")],
            b"{\"status\": \"ok\"}",
            &mut counter
        ).unwrap()).unwrap();

        let original_receipt = asm.finalize();
        let original_semantic = SemanticReceipt::from(original_receipt.clone());

        // --- STAGE 1: Standard -> Compact (v1 -> v2) ---
        let v2: FormatV2 = original_semantic.clone().into();
        let json_v2 = serde_json::to_string_pretty(&v2).unwrap();
        outln!("V2 (Compact):\n{}", json_v2);

        // --- STAGE 2: Compact -> Nested (v2 -> v3) ---
        let v3: FormatV3 = SemanticReceipt::from(v2).into();
        let json_v3 = serde_json::to_string_pretty(&v3).unwrap();
        outln!("V3 (Nested):\n{}", json_v3);

        // --- STAGE 3: Nested -> OCEL-Flat (v3 -> v4) ---
        let v4: FormatV4 = SemanticReceipt::from(v3).into();
        let json_v4 = serde_json::to_string_pretty(&v4).unwrap();
        outln!("V4 (OCEL-Flat):\n{}", json_v4);

        // --- STAGE 4: OCEL-Flat -> Tuple (v4 -> v5) ---
        let v5: FormatV5 = SemanticReceipt::from(v4).into();
        let json_v5 = serde_json::to_string_pretty(&v5).unwrap();
        outln!("V5 (Tuple):\n{}", json_v5);

        // --- STAGE 5: Tuple -> Standard (v5 -> v1) ---
        let final_semantic = SemanticReceipt::from(v5);
        
        // --- VALIDATION: Strict Isomorphism ---
        // 1. Semantic equality
        assert_eq!(original_semantic, final_semantic, "0% semantic loss through 5 transitions");

        // 2. Structural equality (mapping back to original Receipt struct)
        // Since we can't construct Receipt directly (sealed), we check components.
        assert_eq!(final_semantic.format_version, original_receipt.format_version);
        assert_eq!(final_semantic.events, original_receipt.events);
        assert_eq!(final_semantic.chain_hash, original_receipt.chain_hash);

        // 3. Chain Integrity Validation
        // If we re-assembled the events, the hash must match exactly.
        let recomputed = crate::chain::recompute_chain(&final_semantic.events).unwrap();
        assert_eq!(recomputed, final_semantic.chain_hash, "Chain integrity preserved");

        outln!("SUCCESS: Strict isomorphism verified across 5 format versions.");
    }
}
}
}
