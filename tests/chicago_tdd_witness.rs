// Witness: chicago-tdd-tools is GENUINELY integrated — affidavit's admission law
// is asserted using chicago-tdd-tools' own assertion macros.
//
// Failing-when-fake: these macros (`assert_ok!`, `assert_err!`, `assert_in_range!`)
// are re-exported from chicago_tdd_tools. Remove the dependency and this file does
// not COMPILE — so a passing build is itself evidence the library is wired in, and
// a passing test is evidence affidavit's law holds under chicago-tdd's harness.

use affidavit::admission::admit;
use affidavit::chain::{ChainAssembler, recompute_chain, FORMAT_VERSION};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{Blake3Hash, OperationEvent, Receipt};
use chicago_tdd_tools::{assert_err, assert_in_range, assert_ok};

fn honest_receipt() -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    let event = build_event(
        "create",
        vec![object_ref("file-1", "artifact")],
        b"content",
        &mut counter,
    )
    .expect("build event");
    asm.append(event).expect("append");
    asm.finalize()
}

#[test]
fn chicago_tdd_asserts_honest_receipt_is_admitted() {
    // Arrange
    let receipt = honest_receipt();
    // chicago-tdd assertion: event count is in the expected range.
    assert_in_range!(receipt.events.len(), 1, 10);
    // Act
    let result = admit(receipt);
    // Assert (chicago-tdd macro): the court admits an honest receipt.
    assert_ok!(result, "honest receipt must pass the structural law");
}

#[test]
fn chicago_tdd_asserts_forged_receipt_is_refused() {
    // Arrange: a continuity-violating receipt whose chain_hash MATCHES its events
    // (so it survives deserialization's chain re-verification), reaching admit()
    // through the only door an external crate has — the deserialize path. The
    // ONLY thing that can then reject it is admit()'s structural-law pass.
    //
    // Note: an external test CANNOT construct Receipt directly — `_seal` is
    // private (E0451). That is the carrier non-forgeability guarantee (ADR-3)
    // working; we go through serde, exactly as a real attacker's file would.
    let forged_event = OperationEvent {
        id: "evt-7".to_string(),
        seq: 7, // illegal: continuity requires seq contiguous from 0
        event_type: "create".to_string(),
        objects: vec![object_ref("file-1", "artifact")],
        payload_commitment: Blake3Hash::from_bytes(b"content"),
    };
    let chain_hash =
        recompute_chain(std::slice::from_ref(&forged_event)).expect("recompute chain");
    let forged_json = serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [forged_event],
        "chain_hash": chain_hash,
    });
    // Deserialization recomputes the chain and matches (we supplied the right
    // hash), so this succeeds — the receipt is chain-consistent but structurally
    // illegal. This is the adversary's strongest forgery: a correct-looking chain.
    let forged: Receipt =
        serde_json::from_value(forged_json).expect("chain-consistent receipt deserializes");

    // Act
    let result = admit(forged);

    // Assert (chicago-tdd macro): the court refuses a forged receipt by name.
    assert_err!(result, "forged receipt must be refused — admission must run the law");
}
