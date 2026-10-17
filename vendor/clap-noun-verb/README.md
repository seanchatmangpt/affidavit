# clap-noun-verb

[![Crates.io](https://img.shields.io/crates/v/clap-noun-verb)](https://crates.io/crates/clap-noun-verb)
[![Documentation](https://docs.rs/clap-noun-verb/badge.svg)](https://docs.rs/clap-noun-verb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

**Declarative noun-verb CLI framework for type-safe, agent-ready command registration.**

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "26.6.1"
clap-noun-verb-macros = "26.6.1" # For proc-macros
```

Or with `cargo add`:

```bash
cargo add clap-noun-verb clap-noun-verb-macros
```

## The Noun-Verb Pattern

A **noun-verb command** separates domain concepts from actions. Instead of flat command names like `login` or `logout`, organize commands hierarchically:

```
myapp session login     # noun: session, verb: login
myapp session verify     # noun: session, verb: verify
myapp user create --name Bob # noun: user, verb: create (with flags)
```

This pattern naturally maps to your domain model:

- **Noun** = a resource or entity (user, session, config)
- **Verb** = an action on that noun (create, list, delete, verify)

The `#[noun]` and `#[verb]` proc-macros auto-discover and register all commands at compile time. No manual routing.

## Quick Start

Here's a working example in 2 minutes. Create a new Rust project:

```bash
cargo new myapp && cd myapp
cargo add clap-noun-verb clap-noun-verb-macros serde
```

Add to `src/main.rs`:

```rust
use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct CalcResult {
  result: i32,
}

// Pure domain logic
fn add(x: i32, y: i32) -> i32 {
  x + y
}

// Thin CLI wrapper
#[verb("add")]
fn cmd_add(x: i32, y: i32) -> Result<CalcResult> {
  Ok(CalcResult {
    result: add(x, y),
  })
}

#[verb("multiply")]
fn cmd_multiply(
  x: i32,
  y: i32,
  /// Profile to use [default: default]
  #[arg(long)]
  profile_id: Option<String>,
) -> Result<CalcResult> {
  Ok(CalcResult {
    result: x * y,
  })
}

fn main() -> Result<()> {
  clap_noun_verb::run()
}
```

Run it:

```bash
$ cargo run -- calc add 2 3
{"result": 5}

$ cargo run -- calc multiply 4 5 --profile-id premium
{"result": 20}

$ cargo run -- --help
```

**Key Points:**
- Verbs are registered via `#[verb("name")]` macro.
- Nouns are auto-detected from module structure.
- Flags are kebab-case by convention (`--profile-id`, `--dry-run`).
- Output is JSON by default (agent-ready).

## Feature Matrix

| Feature | Type | Example |
|---------|------|---------|
| Noun-verb auto-discovery | Required | `#[verb("add")]` registers `calc add` |
| Doc comment tags (`#[arg]`) | Required | `[default: json]`, `[group: format]`, `[env: VAR]` |
| Kebab-case flag normalization | Required | `--profile-id`, `--dry-run` (idiomatic CLI) |
| JSON output formatting | Required | All results serialize to JSON by default |
| In-process command chaining (`++`) | Optional | `myapp session login ++ session verify @{1.token}` |
| Stdin extraction (`@-`, `@-::json.path`) | Optional | `echo '{"x": 5}' \| myapp cmd @-::x` |
| Dynamic shell completions | Optional | `myapp completions zsh` |
| LLM introspection (`--introspect`) | Optional | Output tool schema for OpenAI/Anthropic |
| Structured errors (`--structured-errors`) | Optional | JSON error format with action templates |
| Interactive REPL mode | Feature-gated (`repl`) | `clap_noun_verb::Repl::new(registry).run()` |
| Tracing & telemetry | Feature-gated (`telemetry`) | OpenTelemetry integration with W3C traceparent |

## Playground How-To

1. **Create a minimal noun:**

```rust
mod services; // Create src/services.rs

// src/services.rs
#[clap_noun_verb_macros::verb("status")]
fn cmd_status() -> clap_noun_verb::Result<Status> {
  Ok(Status { healthy: true })
}
```

2. **Register the noun in main:**

```rust
// src/main.rs
mod services;

fn main() -> Result<()> {
  clap_noun_verb::run()
}
```

3. **Run:**

```bash
$ cargo run -- services status
{"healthy": true}
```

4. **Add arguments to verbs:**

```rust
#[verb("deploy")]
fn cmd_deploy(
  /// Service name
  service: String,
  /// Skip health checks [default: false]
  #[arg(long)]
  dry_run: bool,
) -> Result<DeployResult> {
  // Your logic here
}
```

## Learn More

### Tutorials
- [Domain Separation Architecture](docs/tutorial/02-domain-separation.md) — Learn the separation of concerns pattern
- [Tutorial Series](docs/tutorial/README.md) — 6 progressive lessons (10 mins to 2 hours)

### How-Tos
- [How-To Guides](docs/howto/README.md) — Solve specific problems (testing, errors, custom formatting)
- [Production Guides](docs/howto/production/deployment.md) — Deployment, monitoring, configuration, security

### Reference
- [#[verb] Macro API](docs/reference/api/verb-macro.md) — Detailed syntax and options
- [API Reference](docs/reference/README.md) — Types, errors, CLI runner, telemetry
- [Advanced Features](docs/reference/api/advanced-features.md) — Chaining, introspection, REPL, completions
- [API Catalog](docs/reference/api-catalog.md) — Doc comment tags, argument configuration

### Explanations
- [Architecture Philosophy](docs/explanation/README.md) — Why noun-verb design, type-first thinking, agent-grade CLIs
- [Changelog](CHANGELOG.md) — Version history and breaking changes

## Contributing

Issues and PRs welcome: [github.com/seanchatmangpt/clap-noun-verb](https://github.com/seanchatmangpt/clap-noun-verb)

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
