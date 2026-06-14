// Test: Receipt struct-literal construction is unconstructable (ADR-3).
// This file should FAIL to compile with E0451 (field `_seal` is private).

use affidavit::types::{Blake3Hash, Receipt};

fn main() {
    let _receipt = Receipt {
        format_version: "core/v1".to_string(),
        events: vec![],
        chain_hash: Blake3Hash::from_hex("a".repeat(64)),
        _seal: (),  // ERROR: field `_seal` is private
    };
}
