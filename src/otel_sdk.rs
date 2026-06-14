//! Optional OpenTelemetry SDK integration — real OTLP/stdout span export.
//!
//! Enabled with `--features otel`. When active, replaces the thread-local
//! stub sink with a real OpenTelemetry tracer that exports spans to stdout.
//! Without the feature, this module is empty and the stub sink is used.

#[cfg(feature = "otel")]
pub use real::*;

#[cfg(feature = "otel")]
mod real {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::trace::TracerProvider;
    use opentelemetry_stdout::SpanExporter;

    /// Initialize a stdout OpenTelemetry tracer. Returns the provider.
    /// Call this at program startup to route spans to stdout in OTLP format.
    pub fn init_stdout_tracer() -> TracerProvider {
        let exporter = SpanExporter::default();
        TracerProvider::builder()
            .with_simple_exporter(exporter)
            .build()
    }

    /// Get a named tracer from the global provider.
    pub fn tracer(name: &'static str) -> opentelemetry_sdk::trace::Tracer {
        let provider = init_stdout_tracer();
        provider.tracer(name)
    }
}
