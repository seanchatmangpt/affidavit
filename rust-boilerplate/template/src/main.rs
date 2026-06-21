//! Binary entrypoint for {{project-name}}.

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "{{project-name}}",
    version,
    about = "{{description}}",
    long_about = None,
)]
struct Cli {}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let _cli = Cli::parse();

    Ok(())
}
