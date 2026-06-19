// Reference witness: `OcelRefusal::DanglingEventObjectLink` and
// `OcelRefusal::EmptyEventObjectLinks` are reachable — the structural integrity
// laws at the OCEL admission boundary (COVERAGE.md §7 correction — these laws
// fire against their violations).

use wasm4pm_compat::ocel::{
    EventObjectLink, Object, ObjectChange, ObjectObjectLink, OcelEvent, OcelLog, OcelRefusal,
};

fn log(objects: Vec<Object>, events: Vec<OcelEvent>, e2o: Vec<EventObjectLink>) -> OcelLog {
    OcelLog::new(
        objects,
        events,
        e2o,
        Vec::<ObjectObjectLink>::new(),
        Vec::<ObjectChange>::new(),
    )
}

#[test]
fn ocel_admission_refuses_empty_event_object_links() {
    let l = log(
        vec![],
        vec![OcelEvent::new("evt-0", "create")],
        vec![], // zero e2o links
    );
    assert_eq!(
        l.validate(),
        Err(OcelRefusal::EmptyEventObjectLinks),
        "a log with events but no e2o links must be refused by name"
    );
}

#[test]
fn ocel_admission_refuses_dangling_event_object_link() {
    // The e2o link references object "ghost" which is not in the object set.
    let l = log(
        vec![Object::new("real", "artifact")],
        vec![OcelEvent::new("evt-0", "create")],
        vec![EventObjectLink::new("evt-0", "ghost")], // dangling reference
    );
    assert_eq!(
        l.validate(),
        Err(OcelRefusal::DanglingEventObjectLink),
        "a dangling e2o link must be refused by name"
    );
}
