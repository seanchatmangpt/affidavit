// Test: Receipt construction through the sealed seam compiles (ADR-2/3).
//
// This test proves that while direct struct-literal construction fails (E0451),
// the proper seam (crate::chain::ChainAssembler) allows receipt construction.

use affidavit::chain::ChainAssembler;
use affidavit::ocel::SeqCounter;
use affidavit::types::{Blake3Hash, ObjectRef, OperationEvent};

fn main() {
    // Correct: use ChainAssembler (the sealed seam)
    let mut assembler = ChainAssembler::new();

    let event = OperationEvent {
        id: "evt-0".to_string(),
        seq: 0,
        event_type: "test".to_string(),
        objects: vec![ObjectRef {
            id: "obj1".to_string(),
            obj_type: "artifact".to_string(),
            qualifier: None,
        }],
        payload_commitment: Blake3Hash::from_bytes(b"test payload"),
    };

    assembler.append(event).expect("append");
    let _receipt = assembler.finalize(); // OK: sealed construction
}
