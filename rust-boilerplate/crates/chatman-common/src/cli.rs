//! CLI helpers: output format, color mode, and global args.

// TODO: wire clap-noun-verb integration

use clap::{ArgAction, Parser, ValueEnum};

use crate::Result;

/// Output format selection.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// JSON output.
    Json,
    /// YAML output.
    Yaml,
    /// Human-readable text output.
    Text,
}

/// Color output control.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum ColorMode {
    /// Use color when the output is a terminal.
    Auto,
    /// Always use color.
    Always,
    /// Never use color.
    Never,
}

/// Global arguments shared across all subcommands.
#[derive(Debug, Clone, Parser)]
pub struct GlobalArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Color output.
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,

    /// Verbosity level (pass multiple times to increase).
    #[arg(short = 'v', action = ArgAction::Count)]
    pub verbose: u8,
}

/// Initialize global settings from parsed args.
pub fn init(args: &GlobalArgs) -> Result<()> {
    crate::telemetry::init_tracing()?;
    let _ = args; // further wiring goes here
    Ok(())
}
