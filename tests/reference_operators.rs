// Reference witness: process-tree operators and cross-log correlation schemas
// are constructed as exhaustive censuses (COVERAGE.md §2).
//
// wasm4pm-compat v26.8.7 exposes five structural process-tree operators:
// Sequence/Xor/Parallel/Loop/Silent. Inclusive-OR is not in the closed enum and
// therefore must not be manufactured locally as a compatibility fiction.

use wasm4pm_compat::correlation::CorrelationSchema as Corr;
use wasm4pm_compat::process_tree::ProcessTreeOperator as Op;

#[test]
fn process_tree_operators_are_constructed() {
    let all = [Op::Sequence, Op::Xor, Op::Parallel, Op::Loop, Op::Silent];
    // No-wildcard control-flow family classification: a new ProcessTreeOperator
    // variant breaks compilation here (the match is exhaustive, no `_` arm).
    fn family(o: Op) -> &'static str {
        match o {
            Op::Sequence => "ordering",
            Op::Xor => "choice",
            Op::Parallel => "concurrency",
            Op::Loop => "iteration",
            Op::Silent => "tau",
        }
    }
    let families: std::collections::BTreeSet<&str> = all.iter().copied().map(family).collect();
    assert_eq!(families.len(), 5, "five structural control-flow families");
    let debugs: std::collections::BTreeSet<String> = all.iter().map(|o| format!("{o:?}")).collect();
    assert_eq!(debugs.len(), 5, "five distinct process-tree operators");
}

#[test]
fn correlation_schemas_are_constructed() {
    let all = [
        Corr::ByCase,
        Corr::ByObject,
        Corr::ByTimestamp,
        Corr::ByAttribute,
    ];
    fn label(c: Corr) -> &'static str {
        match c {
            Corr::ByCase => "by-case",
            Corr::ByObject => "by-object",
            Corr::ByTimestamp => "by-timestamp",
            Corr::ByAttribute => "by-attribute",
        }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(s.len(), 4, "four cross-log correlation schemas");
}
