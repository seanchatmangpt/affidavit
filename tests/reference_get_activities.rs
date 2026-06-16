// Reference witness: wasm4pm EventLog::get_activities on a receipt-derived log —
// the activity-set extraction the discovery engine runs (COVERAGE.md §2 —
// activity extraction on the Shape-B log).
//
// affidavit projects a receipt into a wasm4pm EventLog (discovery::project_to_event_log).
// get_activities(key) extracts the distinct activities — the vocabulary discovery
// mines. This witnesses the extraction returns exactly the receipt's event types.

use affidavit::chain::ChainAssembler;
use affidavit::discovery::{project_to_event_log, ACTIVITY_KEY};
use affidavit::ocel::{build_event, object_ref, SeqCounter};

#[test]
fn get_activities_extracts_the_receipts_event_types() {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for act in ["create", "transform", "release", "create"] {
        let ev = build_event(
            act,
            vec![object_ref("o", "artifact")],
            act.as_bytes(),
            &mut counter,
        )
        .expect("event");
        asm.append(ev).expect("append");
    }
    let receipt = asm.finalize();
    let log = project_to_event_log(&receipt);

    let mut activities = log.get_activities(ACTIVITY_KEY);
    activities.sort_unstable();
    activities.dedup();
    // Exactly the three distinct event types (create appears twice → deduped).
    assert_eq!(
        activities,
        vec![
            "create".to_string(),
            "release".to_string(),
            "transform".to_string()
        ],
        "the discovered activity vocabulary is the receipt's distinct event types"
    );
}
