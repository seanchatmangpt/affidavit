//! Telemetry and tracing initialization.

use crate::Result;

/// Initialize tracing with an `EnvFilter` (defaults to `"info"`).
///
/// Idempotent: if a subscriber has already been set this function
/// silently succeeds.
pub fn init_tracing() -> Result<()> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer());

    // Ignore "already set" error — idempotent.
    let _ = subscriber.try_init();
    Ok(())
}

/// Stub OTLP exporter init (no opentelemetry crates added).
///
/// # TODO
/// Wire opentelemetry-otlp when dep is added to Cargo.toml.
#[cfg(feature = "otel")]
pub fn init_otel(endpoint: &str) -> Result<()> {
    tracing::warn!("init_otel called with endpoint={endpoint} but OTLP is not yet wired");
    // TODO: wire opentelemetry-otlp integration here
    Ok(())
}
