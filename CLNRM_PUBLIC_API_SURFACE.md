# clnrm-core Public API Surface for affidavit Integration

**Extracted:** 2026-06-14  
**Version:** clnrm-core 26.6.14  
**Scope:** Three capabilities for 80/20 integration (templates, validators, mutations)

---

## 1. Template Generation

**Location:** `clnrm_core::template::*`

### Generated Functions

All return `Result<String>` where `Result = std::result::Result<T, CleanroomError>`.

#### `generate_otel_template()`

Generates a basic OTEL validation template using Tera syntax.

**Output:** TOML string with sections:
- `[meta]` — name, version, description
- `[otel]` — exporter config, endpoint, sample_ratio, resources
- `[service.*]` — container definitions
- `[[scenario]]` — test scenarios
- `[[expect.span]]` — span assertions
- `[expect.counts]` — span/error count bounds
- `[determinism]` — optional seed/clock freeze
- `[report]` — output file paths

**Template Variables (Tera syntax):**
```
{{ vars.name | default(value="otel_validation") }}
{{ vars.image | default(value="alpine:latest") }}
{{ vars.report_dir | default(value="reports") }}
{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}
{{ now_rfc3339() }}
{{ sha256(s=...) }}
```

**Use in affidavit:**
```rust
let template = clnrm_core::template::generate_otel_template()?;
let injected = template.replace("{{ vars.name }}", "receipt_test_001");
```

---

#### `generate_full_validation_template()`

Comprehensive template showcasing all validators: order, status, count, window, graph, hermeticity.

**Key Sections:**
```toml
[[expect.span]]
name = "..."
kind = "server" | "client" | "internal"
attrs.all = { "key" = "value" }
attrs.any = { ... }

[expect.order]
must_precede = [["span_a", "span_b"], ...]
must_follow = [["span_b", "span_a"], ...]

[expect.status]
all = "ok"
by_name."pattern.*" = "error"

[expect.counts]
spans_total = { gte = 3, lte = 10 }
errors_total = { eq = 0 }
spans_by_kind.internal = { gte = 1 }

[[expect.window]]
name = "startup_window"
start_span = "service.start"
end_span = "service.exec"
max_duration_ms = 5000
min_span_count = 1

[expect.graph]
parent_child = [["parent", "child"], ...]
max_depth = 3
must_be_connected = true

[expect.hermeticity]
allow_network = false
allow_filesystem_read = true
allow_filesystem_write = false
allowed_env_vars = ["PATH", "HOME"]
forbidden_syscalls = ["socket", "connect"]

[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
```

---

#### `generate_macro_library()`

Reusable Tera macros for common patterns.

**Macros:**
```tera
{% macro container_lifecycle_events() %}
["container.start", "container.exec", "container.stop"]
{% endmacro %}

{% macro otel_standard_resources(service_name, version) %}
{
  "service.name" = "{{ service_name }}",
  "service.version" = "{{ version }}",
  "deployment.environment" = "{{ env(name="ENV") | default(value="test") }}"
}
{% endmacro %}

{% macro span_assertions(prefix, kind) %}
[[expect.span]]
name = "{{ prefix }}.*"
kind = "{{ kind }}"
attrs.all = { "test.framework" = "clnrm" }
{% endmacro %}

{% macro temporal_order(first, second) %}
[expect.order]
must_precede = [["{{ first }}", "{{ second }}"]]
{% endmacro %}

{% macro status_validation(pattern, status) %}
[expect.status]
by_name."{{ pattern }}" = "{{ status }}"
{% endmacro %}

{% macro standard_reports(dir) %}
[report]
json = "{{ dir }}/report.json"
junit = "{{ dir }}/junit.xml"
digest = "{{ dir }}/digest.sha256"
{% endmacro %}
```

---

#### `generate_matrix_template()`

Cross-product testing template for matrix parametrization.

**Structure:**
```toml
[matrix]
os = ["alpine", "ubuntu", "debian"]
version = ["3.18", "22.04", "bullseye"]

{% for i in range(end=vars.test_count) %}
[[scenario]]
name = "test_{{ i }}_{{ matrix.os }}"
{% endfor %}

[otel.resources]
"test.os" = "{{ matrix.os }}"
"test.version" = "{{ matrix.version }}"
```

---

### Supporting Config Functions

**Location:** `clnrm_core::config::*`

#### `parse_toml_config(toml_str: &str) -> Result<CleanroomConfig>`

Parses a TOML string into structured `CleanroomConfig`.

**Returned Type:**
```rust
pub struct CleanroomConfig {
    pub meta: MetaSection,
    pub otel: Option<OtelSection>,
    pub service: BTreeMap<String, ServiceConfig>,
    pub scenario: Vec<ScenarioConfig>,
    pub expect: Option<ExpectSection>,
    pub determinism: Option<DeterminismConfig>,
    pub limits: Option<ResourceLimits>,
}
```

---

#### `load_cleanroom_config_from_file(path: &Path) -> Result<CleanroomConfig>`

Loads TOML from file, parses to config.

---

## 2. Validation Rules

**Location:** `clnrm_core::validation::*`

All validator types implement trait patterns for composition.

### Span Validator

**Type:** `pub struct SpanValidator`

**Key Types:**
```rust
pub struct SpanAssertion {
    pub name: String,  // e.g., "container.start"
    pub kind: SpanKind,  // server | client | internal | producer | consumer
    pub attrs: AssertionAttributes,  // all/any assertions
}

pub enum SpanKind {
    Server,
    Client,
    Internal,
    Producer,
    Consumer,
}

pub struct SpanData {
    pub name: String,
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub kind: SpanKind,
    pub attributes: BTreeMap<String, String>,
    pub status: StatusCode,
    pub duration_nanos: u64,
}

pub struct ValidationResult {
    pub passed: bool,
    pub failures: Vec<FailureDetails>,
}
```

**Methods:**
```rust
impl SpanValidator {
    pub fn validate(&self, assertion: &SpanAssertion, spans: &[SpanData]) 
        -> Result<ValidationResult>;
}
```

---

### Order Validator

**Type:** `pub struct OrderValidator`

**Key Types:**
```rust
pub struct OrderExpectation {
    pub must_precede: Vec<[String; 2]>,  // [["span_a", "span_b"], ...]
    pub must_follow: Vec<[String; 2]>,   // [["span_b", "span_a"], ...]
}
```

**Behavior:** Verifies temporal ordering constraints on spans.

- `must_precede`: If both spans present, `span_a` must end before `span_b` starts
- `must_follow`: Inverse constraint

---

### Count Validator

**Type:** `pub struct CountValidator`

**Key Types:**
```rust
pub struct CountExpectation {
    pub spans_total: Option<CountBound>,
    pub errors_total: Option<CountBound>,
    pub spans_by_kind: BTreeMap<String, CountBound>,
}

pub struct CountBound {
    pub gte: Option<usize>,  // >= bound
    pub lte: Option<usize>,  // <= bound
    pub eq: Option<usize>,   // == bound (exclusive with gte/lte)
}
```

**Behavior:** Count spans matching kind/status and verify against bounds.

---

### Window Validator

**Type:** `pub struct WindowValidator`

**Key Types:**
```rust
pub struct WindowExpectation {
    pub name: String,
    pub start_span: String,
    pub end_span: String,
    pub max_duration_ms: u64,
    pub min_span_count: usize,
}
```

**Behavior:** Between start and end span, assert:
- Duration ≤ `max_duration_ms`
- Minimum `min_span_count` spans present

---

### Graph Validator

**Type:** `pub struct GraphValidator`

**Key Types:**
```rust
pub struct GraphExpectation {
    pub parent_child: Vec<[String; 2]>,  // [["parent_span", "child_span"], ...]
    pub max_depth: Option<usize>,
    pub must_be_connected: bool,
}
```

**Behavior:** Verify trace topology:
- Parent-child relationships exist
- Depth constraints respected
- Graph is connected (if required)

---

### Hermeticity Validator

**Type:** `pub struct HermeticityValidator`

**Key Types:**
```rust
pub struct HermeticityExpectation {
    pub allow_network: bool,
    pub allow_filesystem_read: bool,
    pub allow_filesystem_write: bool,
    pub allowed_env_vars: Vec<String>,
    pub forbidden_syscalls: Vec<String>,
}

pub enum ViolationType {
    NetworkAccess(String),
    FilesystemWrite(String),
    ForbiddenSyscall(String),
    UnallowedEnvVar(String),
}

pub struct HermeticityViolation {
    pub violation_type: ViolationType,
    pub span_id: String,
    pub detail: String,
}
```

**Behavior:** Scan span attributes for non-hermetic actions.

---

### OTEL Validator

**Type:** `pub struct OtelValidator`

**Key Types:**
```rust
pub struct OtelValidationConfig {
    pub spans: Vec<SpanAssertion>,
    pub ordering: Option<OrderExpectation>,
    pub counts: Option<CountExpectation>,
    pub windows: Vec<WindowExpectation>,
    pub graph: Option<GraphExpectation>,
    pub hermeticity: Option<HermeticityExpectation>,
}

pub struct SpanValidationResult {
    pub passed: bool,
    pub assertion: SpanAssertion,
    pub matched_spans: usize,
    pub failures: Vec<String>,
}

pub struct TraceValidationResult {
    pub passed: bool,
    pub span_results: Vec<SpanValidationResult>,
    pub order_violations: Vec<String>,
    pub count_violations: Vec<String>,
    pub hermeticity_violations: Vec<HermeticityViolation>,
}
```

**Usage Pattern:**
```rust
let validator = OtelValidator::new(config);
let result = validator.validate_trace(&spans)?;
assert!(result.passed);
```

---

### Validation Orchestrator

**Type:** `pub struct PrdExpectations` (Phase Requirements Document)

Encapsulates all expectations for a single test.

---

## 3. Event Mutations & Chaos

**Location:** `clnrm_core::chaos::*` + `clnrm_core::stress_test::*`

### Chaos Orchestrator

**Type:** `pub struct ChaosOrchestrator`

**Static Methods:**
```rust
impl ChaosOrchestrator {
    pub fn create_plugin(name: &str, config: &ChaosConfigSection) 
        -> Result<ChaosEnginePlugin>;
    
    fn map_experiments_to_scenarios(experiments: &[ChaosExperiment]) 
        -> Result<Vec<ChaosScenario>>;
    
    fn map_single_experiment(exp: &ChaosExperiment) 
        -> Result<ChaosScenario>;
}
```

---

### Chaos Scenarios

**Enum:** `pub enum ChaosScenario`

Variants (NIST-aligned):

```rust
pub enum ChaosScenario {
    // Network faults
    LatencySpikes { duration_secs: u32, max_latency_ms: u32 },
    PacketLoss { duration_secs: u32, loss_percent: f32 },
    NetworkPartition { duration_secs: u32 },
    
    // Resource exhaustion
    CpuSaturation { duration_secs: u32, target_percent: u32 },
    MemoryExhaustion { duration_secs: u32, target_mb: u32 },
    DiskPressure { duration_secs: u32, target_percent: u32 },
    
    // Container lifecycle
    ContainerKill { signal: u8 },
    ContainerStop { timeout_secs: u32 },
    
    // Cryptography
    CryptoFailure { percent: f32 },
    
    // File system
    FsReadErrors { percent: f32 },
    FsWriteErrors { percent: f32 },
    
    // Telemetry corruption
    DropSpans { percent: f32 },
    CorruptSpanAttributes { percent: f32 },
}
```

---

### Chaos Configuration (TOML)

**TOML Structure:**
```toml
[[chaos.experiments]]
name = "network_latency_test"
experiment_type = "network_latency"
duration_seconds = 5
latency_ms = 100

[[chaos.experiments]]
name = "cpu_stress_test"
experiment_type = "cpu_stress"
duration_seconds = 10
cpu_percent = 80

[[chaos.experiments]]
name = "memory_test"
experiment_type = "memory_stress"
memory_mb = 512
```

**Parsed to:**
```rust
pub struct ChaosConfigSection {
    pub experiments: Vec<ChaosExperiment>,
}

pub struct ChaosExperiment {
    pub name: String,
    pub experiment_type: String,
    pub duration_seconds: Option<u32>,
    pub latency_ms: Option<u32>,
    pub cpu_percent: Option<u32>,
    pub memory_mb: Option<u32>,
    // ...
}
```

---

### Stress Test Permutation

**Location:** `clnrm_core::stress_test::*`

**Type:** `pub struct TestPermutation`

**Usage:**
```rust
pub struct StressTestConfig {
    pub parameters: BTreeMap<String, Vec<String>>,
    pub max_combinations: Option<usize>,
}

impl StressTestConfig {
    pub fn generate(&self) -> Result<Vec<TestPermutation>>;
    pub fn generate_batched(&self, batch_size: usize) 
        -> Result<Vec<Vec<TestPermutation>>>;
}

pub struct TestPermutation {
    pub combination: BTreeMap<String, String>,
    pub id: String,
}
```

**Example:**
```rust
let config = StressTestConfig {
    parameters: vec![
        ("os".into(), vec!["alpine", "ubuntu"]),
        ("memory".into(), vec!["256", "512", "1024"]),
    ].into_iter().collect(),
    max_combinations: None,
};

let perms = config.generate()?;
// Generates 2 × 3 = 6 TestPermutation objects
```

---

### NIST Chaos Modules

Each module provides scenario generators for a threat category:

#### `nist_core` — Core system failures
- Process crashes
- Deadlocks
- Timeout/hang scenarios

#### `nist_crypto` — Cryptographic failures
- Invalid signatures
- Key mismatches
- Hash collisions (simulated)

#### `nist_dos` — Denial of Service
- Resource exhaustion
- Infinite loops
- Slow-down attacks

#### `nist_escape` — Sandbox escape attempts
- Privilege escalation
- Syscall violations
- Container escape

#### `nist_fs` — File system attacks
- Corruption
- Partial reads/writes
- Inaccessible files

#### `nist_network` — Network faults
- Partitions
- Man-in-the-middle (simulated)
- Packet loss

#### `nist_telemetry` — Observability corruption
- Span drops
- Attribute injection
- Timing manipulation

---

## 4. Integration Type Mappings

### Receipt Event → Span Analog

For affidavit receipts to work with clnrm validators:

```rust
// affidavit receipt event
pub struct OperationEvent {
    pub seq: u64,
    pub event_type: String,
    pub object_ref: ObjectRef,
    pub commitment: Option<String>,
    // ...
}

// Map to clnrm SpanData for validation
fn receipt_event_to_span(event: &OperationEvent) -> SpanData {
    SpanData {
        name: format!("receipt.{}", event.event_type),
        span_id: format!("{:016x}", event.seq),
        trace_id: "receipt_trace".into(),
        parent_span_id: if event.seq > 0 {
            Some(format!("{:016x}", event.seq - 1))
        } else {
            None
        },
        kind: SpanKind::Internal,
        attributes: vec![
            ("event.type".into(), event.event_type.clone()),
            ("object.id".into(), event.object_ref.id.clone()),
        ].into_iter().collect(),
        status: StatusCode::Ok,
        duration_nanos: 1000,  // Synthetic
    }
}
```

---

## 5. Error Handling

**Common Error Type:** `clnrm_core::error::CleanroomError`

**Conversions:**
```rust
pub type Result<T> = std::result::Result<T, CleanroomError>;

// Recoverable errors
impl CleanroomError {
    pub fn validation_error(msg: &str) -> Self { /* ... */ }
    pub fn config_error(msg: &str) -> Self { /* ... */ }
    pub fn io_error(msg: &str) -> Self { /* ... */ }
}
```

**Affidavit Integration:**
```rust
use clnrm_core::Result as ClnrmResult;
use anyhow::Result as AnyhowResult;

fn clnrm_to_anyhow<T>(result: ClnrmResult<T>) -> AnyhowResult<T> {
    result.map_err(|e| anyhow::anyhow!("clnrm error: {}", e))
}
```

---

## 6. Feature Flags

clnrm-core exposes these features (relevant for affidavit):

```toml
[features]
default = ["backend-gvisor"]
backend-gvisor = []        # Default: use gVisor (no Docker)
backend-docker = []        # Docker API backend
backend-testcontainers = [] # Legacy testcontainers
backend-auto = []          # Auto-detect backend
otel = ["otel-traces", "otel-metrics", "otel-logs"]
otel-traces = []
otel-metrics = []
otel-logs = []
crypto = ["dep:ed25519-dalek"]  # Ed25519 receipt signatures
```

**For affidavit:** Use default `backend-gvisor` (no Docker dependency); enable `crypto` if receipt signing required.

---

## 7. Key Traits for Custom Implementations

### Span Matcher Trait (implicit)

Custom span assertions can be built via:
- `SpanAssertion` with regex name matching
- Attribute matchers using `attrs.all` and `attrs.any`
- Kind-based filtering

### Config Loader Trait (implicit)

TOML → `CleanroomConfig` via `parse_toml_config()`; no custom traits exposed.

### Chaos Plugin Trait (implicit)

`ChaosOrchestrator` is the sole entry point; plugins are opaque `ChaosEnginePlugin`.

---

## Summary: 80/20 API for affidavit

| Capability | Module | Key Types | Usage |
|---|---|---|---|
| **Templates** | `clnrm_core::template` | `generate_*_template() -> String` | Render scenarios for receipt tests |
| **Validators** | `clnrm_core::validation` | `SpanValidator`, `OrderValidator`, `CountValidator`, `HermeticityValidator` | Verify receipt conformance |
| **Mutations** | `clnrm_core::chaos`, `stress_test` | `ChaosScenario`, `TestPermutation` | Generate adversarial receipt corpus |
| **Config** | `clnrm_core::config` | `parse_toml_config()`, `CleanroomConfig` | TOML ↔ Rust config |
| **Errors** | `clnrm_core::error` | `CleanroomError`, `Result<T>` | Error propagation |

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-14  
**Audience:** affidavit integration team
