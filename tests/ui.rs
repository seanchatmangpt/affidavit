// UI tests for Receipt sealing (ADR-3 witness: the bypass is unconstructable).
//
// These tests verify that attempts to bypass the Receipt seam fail at
// compile-time. Each test file in tests/ui/compile_fail/ is expected to
// fail to compile with a specific error (E0451: private field).

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/compile_fail/*.rs");
}
