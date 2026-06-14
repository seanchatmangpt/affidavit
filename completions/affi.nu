# affi Nushell completions
# Install: source (affi receipt completions nushell | save -f /tmp/affi-completions.nu && echo "/tmp/affi-completions.nu")
# Or add to your config.nu: source ~/.config/nushell/affi-completions.nu

def "nu-complete affi verbs" [] {
    ["emit", "assemble", "verify", "show", "inspect", "model", "conformance", "replay", "graph", "stats", "diagnose", "mutate", "bench", "completions", "help-refs"]
}

def "nu-complete affi shells" [] {
    ["bash", "zsh", "fish", "powershell", "nushell"]
}

# Receipt operations — emit a new event
extern "affi receipt emit" [
    --type(-t): string   # Event type (e.g. create, transform, release)
    --object(-o): string # Object ref as id:type or id:type:qualifier (can repeat)
    --payload(-p): string # Payload source: file path or - for stdin
    --help(-h)           # Show help
]

# Finalize the working receipt into an immutable file
extern "affi receipt assemble" [
    --out(-o): string    # Output path (default: <content-hash>.json)
    --help(-h)           # Show help
]

# Run the 7-stage certify pipeline
extern "affi receipt verify" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Show a human-readable dump of a receipt
extern "affi receipt show" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Detailed structural analysis of a receipt
extern "affi receipt inspect" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Discover process model (wasm4pm) from admitted receipt
extern "affi receipt model" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Compute conformance metrics (fitness, activity_coverage, simplicity)
extern "affi receipt conformance" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Replay event sequence in lawful seq order
extern "affi receipt replay" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Show directly-follows graph (wasm4pm DFG)
extern "affi receipt graph" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# One-shot aggregate statistics for a receipt
extern "affi receipt stats" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Render verify outcomes as LSP-shaped diagnostics
extern "affi receipt diagnose" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Demonstrate tamper-evidence on a receipt
extern "affi receipt mutate" [
    receipt: string      # Path to receipt JSON file
    --help(-h)           # Show help
]

# Time core receipt operations inline
extern "affi receipt bench" [
    iterations?: int     # Number of iterations (default: 100)
    --help(-h)           # Show help
]

# Print shell completion script to stdout
extern "affi receipt completions" [
    shell: string@"nu-complete affi shells"  # Target shell
    --help(-h)           # Show help
]

# Print ARDPRD cross-reference map
extern "affi receipt help-refs" [
    --help(-h)           # Show help
]
