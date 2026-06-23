//! 1000X COMBINATORIAL MAXIMALISM: OTel Semantic Convention 1000x.
//!
//! This module defines 100+ hyper-granular OpenTelemetry attributes, spans, and metrics
//! for the affidavit pipeline. These cover everything from WASM memory allocation
//! during verification to branch-prediction misses.
//!
//! These are intended to be used with the `affidavit` tracing substrate to provide
//! maximalist observability into the internal execution state of the system.

/// WASM Runtime Attributes
pub mod wasm {
    pub const MEMORY_ALLOCATED: &str = "affidavit.wasm.memory.allocated";
    pub const MEMORY_FREED: &str = "affidavit.wasm.memory.freed";
    pub const MEMORY_PEAK: &str = "affidavit.wasm.memory.peak";
    pub const STACK_DEPTH_MAX: &str = "affidavit.wasm.stack.depth.max";
    pub const STACK_DEPTH_CURRENT: &str = "affidavit.wasm.stack.depth.current";
    pub const INSTRUCTIONS_TOTAL: &str = "affidavit.wasm.instructions.total";
    pub const INSTRUCTIONS_VERIFIED: &str = "affidavit.wasm.instructions.verified";
    pub const INSTRUCTIONS_SKIPPED: &str = "affidavit.wasm.instructions.skipped";
    pub const TABLE_ELEMENTS: &str = "affidavit.wasm.table.elements";
    pub const TABLE_GROW_COUNT: &str = "affidavit.wasm.table.grow_count";
    pub const IMPORTS_RESOLVED: &str = "affidavit.wasm.imports.resolved";
    pub const IMPORTS_FAILED: &str = "affidavit.wasm.imports.failed";
    pub const EXPORTS_INVOKED: &str = "affidavit.wasm.exports.invoked";
    pub const INIT_DURATION_NS: &str = "affidavit.wasm.init.duration_ns";
    pub const RUNTIME_ENGINE: &str = "affidavit.wasm.runtime.engine";
    pub const MODULE_HASH: &str = "affidavit.wasm.module.hash";
    pub const MODULE_SIZE_BYTES: &str = "affidavit.wasm.module.size_bytes";
    pub const PAGES_INITIAL: &str = "affidavit.wasm.pages.initial";
    pub const PAGES_MAXIMUM: &str = "affidavit.wasm.pages.maximum";
    pub const TRAPS_COUNT: &str = "affidavit.wasm.traps.count";
    pub const JIT_COMPILE_DURATION_NS: &str = "affidavit.wasm.jit.compile_duration_ns";
    pub const JIT_CODE_SIZE_BYTES: &str = "affidavit.wasm.jit.code_size_bytes";
}

/// Cryptographic Operation Attributes
pub mod crypto {
    pub const BLAKE3_HASH_DURATION_NS: &str = "affidavit.crypto.blake3.hash_duration_ns";
    pub const BLAKE3_BYTES_HASHED: &str = "affidavit.crypto.blake3.bytes_hashed";
    pub const BLAKE3_CHUNKS_PROCESSED: &str = "affidavit.crypto.blake3.chunks_processed";
    pub const SIGNATURE_VERIFY_DURATION_NS: &str = "affidavit.crypto.signature.verify_duration_ns";
    pub const SIGNATURE_ALGORITHM: &str = "affidavit.crypto.signature.algorithm";
    pub const KEY_DERIVATION_DURATION_NS: &str = "affidavit.crypto.key.derivation_duration_ns";
    pub const KEY_ROTATION_COUNT: &str = "affidavit.crypto.key.rotation_count";
    pub const ENTROPY_SOURCE: &str = "affidavit.crypto.entropy.source";
    pub const SALT_LENGTH: &str = "affidavit.crypto.salt.length";
    pub const ITERATIONS_COUNT: &str = "affidavit.crypto.iterations.count";
    pub const PADDING_SCHEME: &str = "affidavit.crypto.padding.scheme";
    pub const BLOCK_SIZE: &str = "affidavit.crypto.block.size";
    pub const CIPHER_MODE: &str = "affidavit.crypto.cipher.mode";
    pub const NONCE_VALUE: &str = "affidavit.crypto.nonce.value";
    pub const MAC_VERIFY_SUCCESS: &str = "affidavit.crypto.mac.verify_success";
    pub const MAC_VERIFY_DURATION_NS: &str = "affidavit.crypto.mac.verify_duration_ns";
    pub const CERTIFICATES_CHAIN_LENGTH: &str = "affidavit.crypto.certificates.chain_length";
    pub const CERTIFICATES_EXPIRY_DAYS: &str = "affidavit.crypto.certificates.expiry_days";
    pub const RANDOM_BYTES_REQUESTED: &str = "affidavit.crypto.random.bytes_requested";
    pub const ENCRYPTION_DURATION_NS: &str = "affidavit.crypto.encryption.duration_ns";
    pub const DECRYPTION_DURATION_NS: &str = "affidavit.crypto.decryption_duration_ns";
    pub const KEY_STRENGTH_BITS: &str = "affidavit.crypto.key.strength_bits";
}

/// Low-level Execution Attributes
pub mod execution {
    pub const BRANCH_PREDICTION_MISSES: &str = "affidavit.execution.branch.prediction_misses";
    pub const BRANCH_TOTAL: &str = "affidavit.execution.branch.total";
    pub const LOOP_ITERATIONS: &str = "affidavit.execution.loop.iterations";
    pub const RECURSION_DEPTH: &str = "affidavit.execution.recursion.depth";
    pub const BASIC_BLOCKS_EXECUTED: &str = "affidavit.execution.basic_blocks.executed";
    pub const FUNCTIONS_INVOKED: &str = "affidavit.execution.functions.invoked";
    pub const REGISTERS_SPILLS: &str = "affidavit.execution.registers.spills";
    pub const CACHE_L1_HITS: &str = "affidavit.execution.cache.l1.hits";
    pub const CACHE_L1_MISSES: &str = "affidavit.execution.cache.l1.misses";
    pub const CACHE_L2_HITS: &str = "affidavit.execution.cache.l2.hits";
    pub const CACHE_L2_MISSES: &str = "affidavit.execution.cache.l2.misses";
    pub const TLB_MISSES: &str = "affidavit.execution.tlb.misses";
    pub const PAGE_FAULTS_MAJOR: &str = "affidavit.execution.page_faults.major";
    pub const PAGE_FAULTS_MINOR: &str = "affidavit.execution.page_faults.minor";
    pub const CONTEXT_SWITCHES_VOLUNTARY: &str = "affidavit.execution.context_switches.voluntary";
    pub const CONTEXT_SWITCHES_INVOLUNTARY: &str = "affidavit.execution.context_switches.involuntary";
    pub const THREAD_ID: &str = "affidavit.execution.thread.id";
    pub const CPU_ID: &str = "affidavit.execution.cpu.id";
    pub const PROCESS_UPTIME_NS: &str = "affidavit.execution.process.uptime_ns";
    pub const SYSCALLS_COUNT: &str = "affidavit.execution.syscalls.count";
    pub const INSTRUCTION_RETIRED: &str = "affidavit.execution.instructions.retired";
    pub const CYCLES_TOTAL: &str = "affidavit.execution.cycles.total";
}

/// Verification Logic Attributes
pub mod verification {
    pub const CONSTRAINTS_TOTAL: &str = "affidavit.verification.constraints.total";
    pub const CONSTRAINTS_SATISFIED: &str = "affidavit.verification.constraints.satisfied";
    pub const CONSTRAINTS_VIOLATED: &str = "affidavit.verification.constraints.violated";
    pub const RULES_EVALUATED: &str = "affidavit.verification.rules.evaluated";
    pub const RULES_DURATION_NS: &str = "affidavit.verification.rules.duration_ns";
    pub const EVIDENCE_COUNT: &str = "affidavit.verification.evidence.count";
    pub const EVIDENCE_SIZE_BYTES: &str = "affidavit.verification.evidence.size_bytes";
    pub const WITNESS_ID: &str = "affidavit.verification.witness.id";
    pub const WITNESS_LEVEL: &str = "affidavit.verification.witness.level";
    pub const VERDICT_OUTCOME: &str = "affidavit.verification.verdict.outcome";
    pub const FORMAT_VERSION: &str = "affidavit.verification.format.version";
    pub const SCHEMA_VALIDATION_DURATION_NS: &str = "affidavit.verification.schema.validation_duration_ns";
    pub const CYCLE_DETECTION_DURATION_NS: &str = "affidavit.verification.cycle.detection_duration_ns";
    pub const PATH_LENGTH: &str = "affidavit.verification.path.length";
    pub const NODE_COUNT: &str = "affidavit.verification.node.count";
    pub const EDGE_COUNT: &str = "affidavit.verification.edge.count";
    pub const REACHABILITY_CHECK_DURATION_NS: &str = "affidavit.verification.reachability.check_duration_ns";
    pub const CONSISTENCY_CHECK_DURATION_NS: &str = "affidavit.verification.consistency.check_duration_ns";
    pub const SATURATION_REACHED: &str = "affidavit.verification.saturation.reached";
    pub const PROOF_SIZE_BYTES: &str = "affidavit.verification.proof.size_bytes";
    pub const HYPOTHESIS_COUNT: &str = "affidavit.verification.hypothesis.count";
    pub const REFUTATION_COUNT: &str = "affidavit.verification.refutation.count";
}

/// OCEL (Object-Centric Event Log) Attributes
pub mod ocel {
    pub const EVENT_COUNT: &str = "affidavit.ocel.event.count";
    pub const OBJECT_COUNT: &str = "affidavit.ocel.object.count";
    pub const RELATIONSHIP_COUNT: &str = "affidavit.ocel.relationship.count";
    pub const EVENT_TYPE_COUNT: &str = "affidavit.ocel.event.type_count";
    pub const OBJECT_TYPE_COUNT: &str = "affidavit.ocel.object.type_count";
    pub const ATTRIBUTE_COUNT: &str = "affidavit.ocel.attribute.count";
    pub const NESTING_MAX_DEPTH: &str = "affidavit.ocel.nesting.max_depth";
    pub const SEQUENCE_GAP_COUNT: &str = "affidavit.ocel.sequence.gap_count";
    pub const ID_COLLISIONS: &str = "affidavit.ocel.id.collisions";
    pub const LOG_SIZE_BYTES: &str = "affidavit.ocel.log.size_bytes";
    pub const COMPRESSION_RATIO: &str = "affidavit.ocel.compression.ratio";
    pub const PARSING_DURATION_NS: &str = "affidavit.ocel.parsing.duration_ns";
    pub const SERIALIZATION_DURATION_NS: &str = "affidavit.ocel.serialization.duration_ns";
    pub const TRANSFORMATION_COUNT: &str = "affidavit.ocel.transformation.count";
    pub const FILTERING_DURATION_NS: &str = "affidavit.ocel.filtering.duration_ns";
    pub const SORTING_DURATION_NS: &str = "affidavit.ocel.sorting.duration_ns";
    pub const MAPPING_DURATION_NS: &str = "affidavit.ocel.mapping.duration_ns";
    pub const PROJECTION_DURATION_NS: &str = "affidavit.ocel.projection.duration_ns";
    pub const JOIN_DURATION_NS: &str = "affidavit.ocel.join.duration_ns";
    pub const AGGREGATION_DURATION_NS: &str = "affidavit.ocel.aggregation.duration_ns";
    pub const ANOMALY_SCORE: &str = "affidavit.ocel.anomaly_score";
    pub const DISCOVERY_ALGORITHM: &str = "affidavit.ocel.discovery.algorithm";
}

/// Speculative and Sub-instruction Attributes (Maximalist)
pub mod speculative {
    pub const SPECULATIVE_PATH_COUNT: &str = "affidavit.speculative.path_count";
    pub const SPECULATIVE_EXECUTION_DURATION_NS: &str = "affidavit.speculative.duration_ns";
    pub const SPECULATIVE_RETIRED_COUNT: &str = "affidavit.speculative.retired_count";
    pub const SPECULATIVE_SQUASHED_COUNT: &str = "affidavit.speculative.squashed_count";
    pub const SUB_INSTRUCTION_MICRO_OPS: &str = "affidavit.speculative.micro_ops";
    pub const PIPELINE_STALL_DURATION_NS: &str = "affidavit.speculative.pipeline_stall_ns";
    pub const REORDER_BUFFER_OCCUPANCY: &str = "affidavit.speculative.reorder_buffer_occupancy";
}

/// I/O and System Attributes
pub mod io {
    pub const BYTES_READ: &str = "affidavit.io.bytes_read";
    pub const BYTES_WRITTEN: &str = "affidavit.io.bytes_written";
    pub const FILES_OPENED: &str = "affidavit.io.files_opened";
    pub const IO_WAIT_DURATION_NS: &str = "affidavit.io.wait_duration_ns";
    pub const NETWORK_PACKETS_SENT: &str = "affidavit.io.network.packets_sent";
    pub const NETWORK_PACKETS_RECEIVED: &str = "affidavit.io.network.packets_received";
    pub const SOCKET_COUNT: &str = "affidavit.io.socket_count";
    pub const DNS_LOOKUP_DURATION_NS: &str = "affidavit.io.dns_lookup_duration_ns";
    pub const DISK_LATENCY_NS: &str = "affidavit.io.disk.latency_ns";
    pub const FS_SYNC_COUNT: &str = "affidavit.io.fs.sync_count";
}

/// Combinatorial Maximalist Span Definitions
pub mod spans {
    pub const VERIFY_WASM: &str = "affidavit.span.verify_wasm";
    pub const CRYPTO_BLAKE3: &str = "affidavit.span.crypto_blake3";
    pub const EXECUTION_INNER_LOOP: &str = "affidavit.span.execution_inner_loop";
    pub const OCEL_PARSING: &str = "affidavit.span.ocel_parsing";
    pub const VERIFICATION_REACHABILITY: &str = "affidavit.span.verification_reachability";
}

/// Combinatorial Maximalist Metric Definitions
pub mod metrics {
    pub const WASM_MEMORY_USAGE: &str = "affidavit.metric.wasm_memory_usage";
    pub const BRANCH_MISS_RATE: &str = "affidavit.metric.branch_miss_rate";
    pub const HASH_THROUGHPUT: &str = "affidavit.metric.hash_throughput";
    pub const VERIFICATION_SUCCESS_RATE: &str = "affidavit.metric.verification_success_rate";
    pub const OCEL_EVENT_THROUGHPUT: &str = "affidavit.metric.ocel_event_throughput";
}

/// Helper to get all 100+ attribute keys (useful for registry checks)
pub fn all_attribute_keys() -> Vec<&'static str> {
    let mut keys = Vec::new();
    
    // WASM
    keys.push(wasm::MEMORY_ALLOCATED);
    keys.push(wasm::MEMORY_FREED);
    keys.push(wasm::MEMORY_PEAK);
    keys.push(wasm::STACK_DEPTH_MAX);
    keys.push(wasm::STACK_DEPTH_CURRENT);
    keys.push(wasm::INSTRUCTIONS_TOTAL);
    keys.push(wasm::INSTRUCTIONS_VERIFIED);
    keys.push(wasm::INSTRUCTIONS_SKIPPED);
    keys.push(wasm::TABLE_ELEMENTS);
    keys.push(wasm::TABLE_GROW_COUNT);
    keys.push(wasm::IMPORTS_RESOLVED);
    keys.push(wasm::IMPORTS_FAILED);
    keys.push(wasm::EXPORTS_INVOKED);
    keys.push(wasm::INIT_DURATION_NS);
    keys.push(wasm::RUNTIME_ENGINE);
    keys.push(wasm::MODULE_HASH);
    keys.push(wasm::MODULE_SIZE_BYTES);
    keys.push(wasm::PAGES_INITIAL);
    keys.push(wasm::PAGES_MAXIMUM);
    keys.push(wasm::TRAPS_COUNT);
    keys.push(wasm::JIT_COMPILE_DURATION_NS);
    keys.push(wasm::JIT_CODE_SIZE_BYTES);

    // Crypto
    keys.push(crypto::BLAKE3_HASH_DURATION_NS);
    keys.push(crypto::BLAKE3_BYTES_HASHED);
    keys.push(crypto::BLAKE3_CHUNKS_PROCESSED);
    keys.push(crypto::SIGNATURE_VERIFY_DURATION_NS);
    keys.push(crypto::SIGNATURE_ALGORITHM);
    keys.push(crypto::KEY_DERIVATION_DURATION_NS);
    keys.push(crypto::KEY_ROTATION_COUNT);
    keys.push(crypto::ENTROPY_SOURCE);
    keys.push(crypto::SALT_LENGTH);
    keys.push(crypto::ITERATIONS_COUNT);
    keys.push(crypto::PADDING_SCHEME);
    keys.push(crypto::BLOCK_SIZE);
    keys.push(crypto::CIPHER_MODE);
    keys.push(crypto::NONCE_VALUE);
    keys.push(crypto::MAC_VERIFY_SUCCESS);
    keys.push(crypto::MAC_VERIFY_DURATION_NS);
    keys.push(crypto::CERTIFICATES_CHAIN_LENGTH);
    keys.push(crypto::CERTIFICATES_EXPIRY_DAYS);
    keys.push(crypto::RANDOM_BYTES_REQUESTED);
    keys.push(crypto::ENCRYPTION_DURATION_NS);
    keys.push(crypto::DECRYPTION_DURATION_NS);
    keys.push(crypto::KEY_STRENGTH_BITS);

    // Execution
    keys.push(execution::BRANCH_PREDICTION_MISSES);
    keys.push(execution::BRANCH_TOTAL);
    keys.push(execution::LOOP_ITERATIONS);
    keys.push(execution::RECURSION_DEPTH);
    keys.push(execution::BASIC_BLOCKS_EXECUTED);
    keys.push(execution::FUNCTIONS_INVOKED);
    keys.push(execution::REGISTERS_SPILLS);
    keys.push(execution::CACHE_L1_HITS);
    keys.push(execution::CACHE_L1_MISSES);
    keys.push(execution::CACHE_L2_HITS);
    keys.push(execution::CACHE_L2_MISSES);
    keys.push(execution::TLB_MISSES);
    keys.push(execution::PAGE_FAULTS_MAJOR);
    keys.push(execution::PAGE_FAULTS_MINOR);
    keys.push(execution::CONTEXT_SWITCHES_VOLUNTARY);
    keys.push(execution::CONTEXT_SWITCHES_INVOLUNTARY);
    keys.push(execution::THREAD_ID);
    keys.push(execution::CPU_ID);
    keys.push(execution::PROCESS_UPTIME_NS);
    keys.push(execution::SYSCALLS_COUNT);
    keys.push(execution::INSTRUCTION_RETIRED);
    keys.push(execution::CYCLES_TOTAL);

    // Verification
    keys.push(verification::CONSTRAINTS_TOTAL);
    keys.push(verification::CONSTRAINTS_SATISFIED);
    keys.push(verification::CONSTRAINTS_VIOLATED);
    keys.push(verification::RULES_EVALUATED);
    keys.push(verification::RULES_DURATION_NS);
    keys.push(verification::EVIDENCE_COUNT);
    keys.push(verification::EVIDENCE_SIZE_BYTES);
    keys.push(verification::WITNESS_ID);
    keys.push(verification::WITNESS_LEVEL);
    keys.push(verification::VERDICT_OUTCOME);
    keys.push(verification::FORMAT_VERSION);
    keys.push(verification::SCHEMA_VALIDATION_DURATION_NS);
    keys.push(verification::CYCLE_DETECTION_DURATION_NS);
    keys.push(verification::PATH_LENGTH);
    keys.push(verification::NODE_COUNT);
    keys.push(verification::EDGE_COUNT);
    keys.push(verification::REACHABILITY_CHECK_DURATION_NS);
    keys.push(verification::CONSISTENCY_CHECK_DURATION_NS);
    keys.push(verification::SATURATION_REACHED);
    keys.push(verification::PROOF_SIZE_BYTES);
    keys.push(verification::HYPOTHESIS_COUNT);
    keys.push(verification::REFUTATION_COUNT);

    // OCEL
    keys.push(ocel::EVENT_COUNT);
    keys.push(ocel::OBJECT_COUNT);
    keys.push(ocel::RELATIONSHIP_COUNT);
    keys.push(ocel::EVENT_TYPE_COUNT);
    keys.push(ocel::OBJECT_TYPE_COUNT);
    keys.push(ocel::ATTRIBUTE_COUNT);
    keys.push(ocel::NESTING_MAX_DEPTH);
    keys.push(ocel::SEQUENCE_GAP_COUNT);
    keys.push(ocel::ID_COLLISIONS);
    keys.push(ocel::LOG_SIZE_BYTES);
    keys.push(ocel::COMPRESSION_RATIO);
    keys.push(ocel::PARSING_DURATION_NS);
    keys.push(ocel::SERIALIZATION_DURATION_NS);
    keys.push(ocel::TRANSFORMATION_COUNT);
    keys.push(ocel::FILTERING_DURATION_NS);
    keys.push(ocel::SORTING_DURATION_NS);
    keys.push(ocel::MAPPING_DURATION_NS);
    keys.push(ocel::PROJECTION_DURATION_NS);
    keys.push(ocel::JOIN_DURATION_NS);
    keys.push(ocel::AGGREGATION_DURATION_NS);
    keys.push(ocel::ANOMALY_SCORE);
    keys.push(ocel::DISCOVERY_ALGORITHM);

    // Speculative
    keys.push(speculative::SPECULATIVE_PATH_COUNT);
    keys.push(speculative::SPECULATIVE_EXECUTION_DURATION_NS);
    keys.push(speculative::SPECULATIVE_RETIRED_COUNT);
    keys.push(speculative::SPECULATIVE_SQUASHED_COUNT);
    keys.push(speculative::SUB_INSTRUCTION_MICRO_OPS);
    keys.push(speculative::PIPELINE_STALL_DURATION_NS);
    keys.push(speculative::REORDER_BUFFER_OCCUPANCY);

    // IO
    keys.push(io::BYTES_READ);
    keys.push(io::BYTES_WRITTEN);
    keys.push(io::FILES_OPENED);
    keys.push(io::IO_WAIT_DURATION_NS);
    keys.push(io::NETWORK_PACKETS_SENT);
    keys.push(io::NETWORK_PACKETS_RECEIVED);
    keys.push(io::SOCKET_COUNT);
    keys.push(io::DNS_LOOKUP_DURATION_NS);
    keys.push(io::DISK_LATENCY_NS);
    keys.push(io::FS_SYNC_COUNT);

    keys
}

/// Maximalist Instrumentation Helpers
///
/// These functions demonstrate how to use the hyper-granular attributes
/// to record deep internal state during pipeline operations.
pub mod instrumentation {
    use super::*;

    /// Record a hyper-granular WASM verification span.
    pub fn record_wasm_verify_maximalist(
        _module_hash: &str,
        _instruction_count: u64,
        _memory_peak: u64,
    ) {
        // In a real implementation with an OTel SDK:
        // let mut span = tracer.start("verify_wasm");
        // span.set_attribute(wasm::MODULE_HASH, _module_hash);
        // span.set_attribute(wasm::INSTRUCTIONS_TOTAL, _instruction_count);
        // span.set_attribute(wasm::MEMORY_PEAK, _memory_peak);
        // span.set_attribute(execution::BRANCH_PREDICTION_MISSES, 42); // Simulated
    }

    /// Record a hyper-granular Crypto BLAKE3 span.
    pub fn record_crypto_blake3_maximalist(
        _bytes_hashed: u64,
        _duration_ns: u64,
    ) {
        // span.set_attribute(crypto::BLAKE3_BYTES_HASHED, _bytes_hashed);
        // span.set_attribute(crypto::BLAKE3_HASH_DURATION_NS, _duration_ns);
        // span.set_attribute(execution::CYCLES_TOTAL, _duration_ns * 3); // Estimated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_is_maximalist() {
        let keys = all_attribute_keys();
        outln!("Total attributes: {}", keys.len());
        assert!(keys.len() >= 100, "Must define at least 100 hyper-granular attributes");
    }

    #[test]
    fn keys_are_prefixed() {
        for key in all_attribute_keys() {
            assert!(key.starts_with("affidavit."), "Key '{}' must start with 'affidavit.'", key);
        }
    }
}
