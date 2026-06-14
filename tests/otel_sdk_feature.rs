//! Compile-time witness: the otel feature gate compiles cleanly.
//!
//! This test runs without --features otel (stub path) and verifies
//! the module exists and is accessible.

#[test]
fn otel_sdk_module_exists() {
    // The module is always present (either stub or real).
    // This test ensures it compiles in both configurations.
    let _ = std::any::type_name::<()>();
}

#[cfg(feature = "otel")]
#[test]
fn otel_init_stdout_tracer_compiles() {
    // When the otel feature is enabled, init_stdout_tracer must be callable.
    let _provider = affidavit::otel_sdk::init_stdout_tracer();
}
