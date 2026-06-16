//! OCEL-shaped operation events end-to-end: build an event from object refs with
//! a monotonic SeqCounter, round-trip a qualified ref through parse_object_ref,
//! and trigger the real validation refusal.
//!
//! See the doc on `build_event` / `validate_event` / `parse_object_ref` in src/ocel.rs.
//! Run: `cargo run --example ocel_events`

use affidavit::ocel::{
    build_event, object_ref, parse_object_ref, qualified_object_ref, validate_event, OcelError,
    SeqCounter,
};

fn main() {
    // --- Build an OCEL event from object refs using a monotonic counter ---
    let mut counter = SeqCounter::new();
    assert_eq!(counter.peek(), 0, "fresh counter peeks at 0");

    let plain = object_ref("o1", "file");
    let qualified = qualified_object_ref("o2", "report", "output");

    let event = build_event(
        "write",
        vec![plain.clone(), qualified.clone()],
        b"payload bytes",
        &mut counter,
    )
    .expect("well-formed event must build");

    // ASSERT the event's fields are exactly as constructed.
    assert_eq!(event.seq, 0, "first event draws seq 0");
    assert_eq!(event.id, "evt-0", "id derived from seq");
    assert_eq!(event.event_type, "write");
    assert_eq!(event.objects.len(), 2);
    assert_eq!(event.objects[0], plain);
    assert_eq!(event.objects[1], qualified);
    assert_eq!(event.objects[1].qualifier.as_deref(), Some("output"));
    assert_eq!(counter.peek(), 1, "counter advanced past 0");

    // Second event off the same counter advances monotonically.
    let event2 = build_event("read", vec![], b"", &mut counter).expect("builds");
    assert_eq!(event2.seq, 1);
    assert_eq!(event2.id, "evt-1");

    // --- Round-trip parse_object_ref on a qualified ref ---
    let spec = "o2:report:output";
    let parsed = parse_object_ref(spec).expect("qualified spec parses");
    assert_eq!(
        parsed, qualified,
        "parsed ref reconstructs the qualified ref"
    );
    assert_eq!(parsed.qualifier.as_deref(), Some("output"));

    // --- EDGE: validate_event rejects a malformed (empty-id) object ref ---
    // build_event runs validate_event internally; an empty object id is rejected
    // at index 0 with OcelError::EmptyObjectId(0).
    let err = build_event(
        "write",
        vec![object_ref("", "file")],
        b"x",
        &mut SeqCounter::new(),
    )
    .expect_err("empty object id must be refused");
    assert_eq!(
        err,
        OcelError::EmptyObjectId(0),
        "edge: empty object id rejected"
    );

    // EDGE: a malformed ref string yields MalformedObjectRef.
    let parse_err = parse_object_ref("nope").expect_err("no ':' must be refused");
    assert_eq!(
        parse_err,
        OcelError::MalformedObjectRef("nope".to_string()),
        "edge: unparseable spec rejected"
    );

    // EDGE: validate_event directly refuses an empty event_type.
    let bad = build_event("   ", vec![], b"x", &mut SeqCounter::new());
    assert_eq!(bad.unwrap_err(), OcelError::EmptyEventType);

    // A hand-built well-formed event passes validate_event.
    validate_event(&event).expect("constructed event is valid");

    // --- SeqCounter::starting_at + next_seq: resume numbering from an offset ---
    // A counter started at 5 hands out 5, 6, 7... directly via next_seq (the raw
    // monotonic draw that build_event uses internally).
    let mut resumed = SeqCounter::starting_at(5);
    assert_eq!(resumed.peek(), 5, "starting_at(5) peeks at 5");
    assert_eq!(resumed.next_seq(), 5, "first draw is the starting value");
    assert_eq!(resumed.next_seq(), 6, "next_seq is strictly monotonic");
    assert_eq!(resumed.peek(), 7, "peek shows the next undrawn seq");
    let resumed_event = build_event("amend", vec![], b"z", &mut resumed).expect("builds");
    assert_eq!(
        resumed_event.seq, 7,
        "build_event draws the resumed counter's next seq"
    );

    println!("OK: built evt-0/evt-1, parsed {spec}, refused empty id + malformed ref + empty type, resumed seq 5->7");
}
