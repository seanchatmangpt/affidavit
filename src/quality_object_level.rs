//! Fine-grained quality metrics at file, module, and package levels.
//!
//! This module tracks code quality violations at three granularities:
//! - **File-level**: `FileQualityMetrics` — stub_ratio, cyclomatic_complexity, coverage per file
//! - **Module-level**: `ModuleQualityMetrics` — aggregated mean/stddev across files in a module
//! - **Package-level**: `PackageHealthScore` — 0–100 health index per package
//! - **Object-level violations**: `ObjectViolation` — fine-grained violation targeting (file/module/package)
//!
//! Supports cross-package analysis: API breaking changes, dependency violations.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// File-level quality metrics with fine granularity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileQualityMetrics {
    /// File path (absolute or relative)
    pub path: String,

    /// Ratio of stub functions (todo!/unimplemented!/panic!) (0.0–1.0)
    pub stub_ratio: f64,

    /// Mean cyclomatic complexity across functions (>=1.0)
    pub cyclomatic_complexity: f64,

    /// Maximum cyclomatic complexity in this file
    pub max_cyclomatic_complexity: f64,

    /// Test coverage percentage for this file (0–100)
    pub test_coverage: f64,

    /// Documentation coverage ratio (0.0–1.0)
    pub doc_coverage: f64,

    /// Lines of code (excluding comments and blanks)
    pub loc: usize,

    /// Comment lines in this file
    pub comment_lines: usize,

    /// Number of public items (functions, types, modules, consts)
    pub public_items: usize,

    /// Number of public items with documentation
    pub documented_public_items: usize,

    /// Cognitive complexity score (lower = simpler)
    pub cognitive_complexity: f64,

    /// Type annotation coverage (0.0–1.0)
    pub type_coverage: f64,
}

impl FileQualityMetrics {
    /// Create a new FileQualityMetrics with given path.
    pub fn new(path: String) -> Self {
        Self {
            path,
            stub_ratio: 0.0,
            cyclomatic_complexity: 1.0,
            max_cyclomatic_complexity: 1.0,
            test_coverage: 0.0,
            doc_coverage: 0.0,
            loc: 0,
            comment_lines: 0,
            public_items: 0,
            documented_public_items: 0,
            cognitive_complexity: 0.0,
            type_coverage: 1.0,
        }
    }

    /// Compute maintainability index (0–100) for this file.
    pub fn maintainability_index(&self) -> f64 {
        // Simplified formula: higher coverage = higher index
        let coverage_score = (self.test_coverage + (self.doc_coverage * 100.0)) / 2.0;
        let complexity_penalty = (self.cyclomatic_complexity * 10.0).min(50.0);
        (coverage_score - complexity_penalty).clamp(0.0, 100.0)
    }
}

/// Module-level aggregated metrics (mean/stddev across files).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleQualityMetrics {
    /// Module name (e.g., "verifier", "handlers", "lsp")
    pub module_name: String,

    /// Number of files in this module
    pub file_count: usize,

    /// Mean stub ratio across files
    pub mean_stub_ratio: f64,

    /// Standard deviation of stub ratio
    pub stddev_stub_ratio: f64,

    /// Mean cyclomatic complexity across files
    pub mean_cyclomatic_complexity: f64,

    /// Standard deviation of cyclomatic complexity
    pub stddev_cyclomatic_complexity: f64,

    /// Max cyclomatic complexity in any file of this module
    pub max_cyclomatic_complexity: f64,

    /// Mean test coverage across files
    pub mean_test_coverage: f64,

    /// Standard deviation of test coverage
    pub stddev_test_coverage: f64,

    /// Mean doc coverage across files
    pub mean_doc_coverage: f64,

    /// Standard deviation of doc coverage
    pub stddev_doc_coverage: f64,

    /// Total lines of code in module
    pub total_loc: usize,

    /// Total public items across files
    pub total_public_items: usize,

    /// Total documented items across files
    pub total_documented_items: usize,

    /// Overall type annotation coverage
    pub mean_type_coverage: f64,
}

impl ModuleQualityMetrics {
    /// Create a new ModuleQualityMetrics with given module name.
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            file_count: 0,
            mean_stub_ratio: 0.0,
            stddev_stub_ratio: 0.0,
            mean_cyclomatic_complexity: 1.0,
            stddev_cyclomatic_complexity: 0.0,
            max_cyclomatic_complexity: 1.0,
            mean_test_coverage: 0.0,
            stddev_test_coverage: 0.0,
            mean_doc_coverage: 0.0,
            stddev_doc_coverage: 0.0,
            total_loc: 0,
            total_public_items: 0,
            total_documented_items: 0,
            mean_type_coverage: 1.0,
        }
    }
}

/// Package-level health score (0–100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageHealthScore {
    /// Package name
    pub package_name: String,

    /// Overall health score (0–100, higher = better)
    pub health_score: f64,

    /// Test coverage percentage
    pub test_coverage: f64,

    /// Documentation coverage percentage
    pub doc_coverage: f64,

    /// Stub ratio (lower = better)
    pub stub_ratio: f64,

    /// Mean cyclomatic complexity (lower = better)
    pub mean_complexity: f64,

    /// Number of quality violations detected
    pub violation_count: usize,

    /// API breaking changes detected
    pub api_breaking_changes: usize,

    /// Dependency violations (policy/security)
    pub dependency_violations: usize,
}

impl PackageHealthScore {
    /// Create a new PackageHealthScore for a package.
    pub fn new(package_name: String) -> Self {
        Self {
            package_name,
            health_score: 100.0,
            test_coverage: 0.0,
            doc_coverage: 0.0,
            stub_ratio: 0.0,
            mean_complexity: 1.0,
            violation_count: 0,
            api_breaking_changes: 0,
            dependency_violations: 0,
        }
    }

    /// Update health score based on metrics.
    pub fn update_health_score(&mut self) {
        let mut score = 100.0;

        // Penalize for stub ratio (each 10% = 10 points)
        score -= (self.stub_ratio * 100.0) / 10.0;

        // Penalize for low test coverage (each 10% missing = 5 points)
        score -= ((100.0 - self.test_coverage) / 10.0) * 5.0;

        // Penalize for low doc coverage (each 10% missing = 3 points)
        score -= ((100.0 - self.doc_coverage) / 10.0) * 3.0;

        // Penalize for high complexity (each unit > 3 = 2 points)
        if self.mean_complexity > 3.0 {
            score -= (self.mean_complexity - 3.0) * 2.0;
        }

        // Penalize for violations (each violation = 5 points)
        score -= (self.violation_count as f64) * 5.0;

        // Penalize for API breaking changes (each = 10 points)
        score -= (self.api_breaking_changes as f64) * 10.0;

        // Penalize for dependency violations (each = 8 points)
        score -= (self.dependency_violations as f64) * 8.0;

        self.health_score = score.clamp(0.0, 100.0);
    }
}

/// Fine-grained violation at specific object (file/module/package).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectViolation {
    /// File-level violation (e.g., high stub ratio in a single file)
    FileViolation {
        file_path: String,
        violation_type: String,
        value: f64,
        threshold: f64,
        severity: String,
    },

    /// Module-level violation (e.g., inconsistent quality across module)
    ModuleViolation {
        module_name: String,
        violation_type: String,
        mean_value: f64,
        stddev_value: f64,
        severity: String,
    },

    /// Package-level violation (e.g., health score below threshold)
    PackageViolation {
        package_name: String,
        violation_type: String,
        current_score: f64,
        threshold: f64,
        severity: String,
    },

    /// Cross-package API breaking change
    APIBreakingChange {
        from_package: String,
        to_package: String,
        description: String,
        severity: String,
    },

    /// Dependency policy violation
    DependencyViolation {
        package_name: String,
        dependency_name: String,
        reason: String,
        severity: String,
    },
}

impl ObjectViolation {
    /// Get violation severity (CRITICAL, HIGH, MEDIUM, LOW, INFO)
    pub fn severity(&self) -> &str {
        match self {
            Self::FileViolation { severity, .. } => severity,
            Self::ModuleViolation { severity, .. } => severity,
            Self::PackageViolation { severity, .. } => severity,
            Self::APIBreakingChange { severity, .. } => severity,
            Self::DependencyViolation { severity, .. } => severity,
        }
    }

    /// Get human-readable description of violation
    pub fn description(&self) -> String {
        match self {
            Self::FileViolation {
                file_path,
                violation_type,
                value,
                threshold,
                ..
            } => {
                format!(
                    "{}: {} (value={:.2}, threshold={:.2}) in file {}",
                    violation_type,
                    self.severity(),
                    value,
                    threshold,
                    file_path
                )
            }
            Self::ModuleViolation {
                module_name,
                violation_type,
                mean_value,
                stddev_value,
                ..
            } => {
                format!(
                    "{}: {} (mean={:.2}, stddev={:.2}) in module {}",
                    violation_type,
                    self.severity(),
                    mean_value,
                    stddev_value,
                    module_name
                )
            }
            Self::PackageViolation {
                package_name,
                violation_type,
                current_score,
                threshold,
                ..
            } => {
                format!(
                    "{}: {} (score={:.2}, threshold={:.2}) in package {}",
                    violation_type,
                    self.severity(),
                    current_score,
                    threshold,
                    package_name
                )
            }
            Self::APIBreakingChange {
                from_package,
                to_package,
                description,
                ..
            } => {
                format!(
                    "API Breaking Change: {} (from {} to {}): {}",
                    self.severity(),
                    from_package,
                    to_package,
                    description
                )
            }
            Self::DependencyViolation {
                package_name,
                dependency_name,
                reason,
                ..
            } => {
                format!(
                    "Dependency Violation: {} (package={}, dependency={}): {}",
                    self.severity(),
                    package_name,
                    dependency_name,
                    reason
                )
            }
        }
    }
}

/// Measure quality metrics for a single file.
pub fn measure_file_quality(path: &str) -> anyhow::Result<FileQualityMetrics> {
    use regex::Regex;
    use std::fs;

    let mut metrics = FileQualityMetrics::new(path.to_string());

    let path_obj = Path::new(path);
    if !path_obj.exists() || !path_obj.is_file() {
        return Ok(metrics);
    }

    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Count various metrics
    let mut stub_count = 0;
    let mut function_count = 0;
    let mut public_items = 0;
    let mut doc_items = 0;
    let mut total_complexity: f64 = 0.0;
    let mut max_complexity: f64 = 1.0;
    let mut total_params = 0;

    // Patterns
    let stub_pattern = Regex::new(r"\b(todo|unimplemented|panic)!\s*\(")?;
    let fn_pattern = Regex::new(r"\bfn\s+\w+")?;
    let pub_pattern = Regex::new(r"\bpub\s+(fn|struct|enum|trait|mod|const|type)")?;
    let doc_pattern = Regex::new(r"^\s*///\s*")?;
    let typed_fn_pattern = Regex::new(r"->\s*\w+")?;

    // Count lines and comments
    let mut loc_count = 0;
    let mut comment_count = 0;

    for line in &lines {
        let trimmed = line.trim();

        // Count LOC (excluding comments and blanks)
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            loc_count += 1;
        }

        // Count comment lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            comment_count += 1;
        }

        // Count documentation
        if doc_pattern.is_match(line) {
            doc_items += 1;
        }

        // Count public items
        if pub_pattern.is_match(line) {
            public_items += 1;
        }

        // Count functions
        if fn_pattern.is_match(line) {
            function_count += 1;

            // Type coverage heuristic: count "->" in signature
            if typed_fn_pattern.is_match(line) {
                total_params += 1;
            }
        }

        // Count stubs
        stub_count += stub_pattern.find_iter(line).count();

        // Estimate cyclomatic complexity (simple heuristic)
        let mut line_complexity = 1.0;
        line_complexity += (line.matches(" if ").count()
            + line.matches("else if ").count()
            + line.matches(" match ").count()
            + line.matches(" for ").count()
            + line.matches(" while ").count()
            + line.matches("||").count()
            + line.matches("&&").count()) as f64;

        if line_complexity > 1.0 {
            max_complexity = max_complexity.max(line_complexity);
            total_complexity += line_complexity;
        }
    }

    // Compute metrics
    if function_count > 0 {
        metrics.stub_ratio = stub_count as f64 / function_count as f64;
        metrics.cyclomatic_complexity = total_complexity / function_count as f64;
        metrics.max_cyclomatic_complexity = max_complexity;
        metrics.type_coverage = if total_params > 0 {
            (total_params as f64 / function_count as f64).min(1.0)
        } else {
            1.0
        };
    }

    metrics.public_items = public_items;
    metrics.documented_public_items = doc_items.min(public_items);
    if public_items > 0 {
        metrics.doc_coverage = doc_items as f64 / public_items as f64;
    }

    metrics.loc = loc_count;
    metrics.comment_lines = comment_count;

    // Heuristic test coverage: count #[test] attributes
    let test_count = content.matches("#[test]").count() + content.matches("#[cfg(test)]").count();
    if function_count > 0 {
        metrics.test_coverage = (test_count as f64 / function_count as f64 * 100.0).min(100.0);
    }

    Ok(metrics)
}

/// Aggregate file metrics into module-level metrics.
pub fn aggregate_module_metrics(files: &[FileQualityMetrics]) -> ModuleQualityMetrics {
    let mut result = ModuleQualityMetrics::new("aggregated".to_string());

    if files.is_empty() {
        return result;
    }

    result.file_count = files.len();

    // Compute means
    let stub_ratios: Vec<f64> = files.iter().map(|f| f.stub_ratio).collect();
    let complexities: Vec<f64> = files.iter().map(|f| f.cyclomatic_complexity).collect();
    let coverages: Vec<f64> = files.iter().map(|f| f.test_coverage).collect();
    let doc_coverages: Vec<f64> = files.iter().map(|f| f.doc_coverage).collect();
    let type_coverages: Vec<f64> = files.iter().map(|f| f.type_coverage).collect();

    result.mean_stub_ratio = compute_mean(&stub_ratios);
    result.stddev_stub_ratio = compute_stddev(&stub_ratios, result.mean_stub_ratio);

    result.mean_cyclomatic_complexity = compute_mean(&complexities);
    result.stddev_cyclomatic_complexity =
        compute_stddev(&complexities, result.mean_cyclomatic_complexity);

    result.mean_test_coverage = compute_mean(&coverages);
    result.stddev_test_coverage = compute_stddev(&coverages, result.mean_test_coverage);

    result.mean_doc_coverage = compute_mean(&doc_coverages);
    result.stddev_doc_coverage = compute_stddev(&doc_coverages, result.mean_doc_coverage);

    result.mean_type_coverage = compute_mean(&type_coverages);

    result.max_cyclomatic_complexity = files
        .iter()
        .map(|f| f.max_cyclomatic_complexity)
        .fold(f64::NEG_INFINITY, f64::max);

    result.total_loc = files.iter().map(|f| f.loc).sum();
    result.total_public_items = files.iter().map(|f| f.public_items).sum();
    result.total_documented_items = files.iter().map(|f| f.documented_public_items).sum();

    result
}

/// Compute package health score from module metrics.
pub fn compute_package_health(modules: &[ModuleQualityMetrics]) -> PackageHealthScore {
    let mut score = PackageHealthScore::new("package".to_string());

    if modules.is_empty() {
        return score;
    }

    // Aggregate across modules
    let test_coverages: Vec<f64> = modules.iter().map(|m| m.mean_test_coverage).collect();
    let doc_coverages: Vec<f64> = modules.iter().map(|m| m.mean_doc_coverage).collect();
    let stub_ratios: Vec<f64> = modules.iter().map(|m| m.mean_stub_ratio).collect();
    let complexities: Vec<f64> = modules
        .iter()
        .map(|m| m.mean_cyclomatic_complexity)
        .collect();

    score.test_coverage = compute_mean(&test_coverages);
    score.doc_coverage = compute_mean(&doc_coverages) * 100.0;
    score.stub_ratio = compute_mean(&stub_ratios);
    score.mean_complexity = compute_mean(&complexities);

    score.update_health_score();

    score
}

/// Detect violations at file level using baseline and stddev thresholds.
pub fn detect_object_level_violations(
    object_metric: &FileQualityMetrics,
    baseline: f64,
    stddev: f64,
) -> Vec<ObjectViolation> {
    let mut violations = Vec::new();

    // Check stub ratio (should be low)
    let stub_threshold = baseline + 2.0 * stddev;
    if object_metric.stub_ratio > stub_threshold && object_metric.stub_ratio > 0.1 {
        violations.push(ObjectViolation::FileViolation {
            file_path: object_metric.path.clone(),
            violation_type: "high_stub_ratio".to_string(),
            value: object_metric.stub_ratio,
            threshold: stub_threshold,
            severity: if object_metric.stub_ratio > baseline + 3.0 * stddev {
                "CRITICAL".to_string()
            } else {
                "HIGH".to_string()
            },
        });
    }

    // Check cyclomatic complexity (should be low)
    let complexity_threshold = baseline + 2.0 * stddev;
    if object_metric.cyclomatic_complexity > complexity_threshold
        && object_metric.cyclomatic_complexity > 3.0
    {
        violations.push(ObjectViolation::FileViolation {
            file_path: object_metric.path.clone(),
            violation_type: "high_cyclomatic_complexity".to_string(),
            value: object_metric.cyclomatic_complexity,
            threshold: complexity_threshold,
            severity: if object_metric.cyclomatic_complexity > baseline + 3.0 * stddev {
                "HIGH".to_string()
            } else {
                "MEDIUM".to_string()
            },
        });
    }

    // Check doc coverage (should be high)
    if object_metric.doc_coverage < 0.5 {
        violations.push(ObjectViolation::FileViolation {
            file_path: object_metric.path.clone(),
            violation_type: "low_doc_coverage".to_string(),
            value: object_metric.doc_coverage,
            threshold: 0.5,
            severity: "MEDIUM".to_string(),
        });
    }

    // Check type coverage (should be high)
    if object_metric.type_coverage < 0.7 {
        violations.push(ObjectViolation::FileViolation {
            file_path: object_metric.path.clone(),
            violation_type: "low_type_coverage".to_string(),
            value: object_metric.type_coverage,
            threshold: 0.7,
            severity: "LOW".to_string(),
        });
    }

    violations
}

/// Helper: compute mean of a slice of f64
fn compute_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Helper: compute standard deviation from a slice and mean
fn compute_stddev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> std::io::Result<String> {
        let path = dir.join(name);
        fs::write(&path, content)?;
        Ok(path.to_string_lossy().to_string())
    }

    // ========================================================================
    // File-level Tests
    // ========================================================================

    #[test]
    fn test_measure_file_quality_basic() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let path = create_test_file(
            temp_dir.path(),
            "test.rs",
            r#"
/// Documented function
pub fn good_fn() -> i32 {
    42
}

fn stub_fn() {
    todo!("implement later")
}
"#,
        )
        .expect("create test file");

        let metrics = measure_file_quality(&path).expect("measure file quality");
        assert!(metrics.stub_ratio > 0.0);
        assert!(metrics.public_items > 0);
    }

    #[test]
    fn test_measure_file_quality_with_complex_function() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let path = create_test_file(
            temp_dir.path(),
            "complex.rs",
            r#"
pub fn complex_fn(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x + 1
        }
    } else if x < 0 {
        -x
    } else {
        0
    }
}
"#,
        )
        .expect("create test file");

        let metrics = measure_file_quality(&path).expect("measure file quality");
        assert!(metrics.cyclomatic_complexity > 1.0);
    }

    #[test]
    fn test_measure_file_quality_doc_coverage() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let path = create_test_file(
            temp_dir.path(),
            "documented.rs",
            r#"
/// Well documented function
pub fn documented() {}

pub fn undocumented() {}
"#,
        )
        .expect("create test file");

        let metrics = measure_file_quality(&path).expect("measure file quality");
        assert!(metrics.doc_coverage > 0.0);
        assert!(metrics.doc_coverage < 1.0);
    }

    #[test]
    fn test_measure_file_quality_type_coverage() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let path = create_test_file(
            temp_dir.path(),
            "typed.rs",
            r#"
pub fn typed_fn(x: i32, y: String) -> bool {
    true
}

pub fn untyped_fn() {
    outln!("no return type");
}
"#,
        )
        .expect("create test file");

        let metrics = measure_file_quality(&path).expect("measure file quality");
        assert!(metrics.type_coverage > 0.0);
    }

    #[test]
    fn test_file_quality_metrics_maintainability_index() {
        let mut metrics = FileQualityMetrics::new("test.rs".to_string());
        metrics.test_coverage = 80.0;
        metrics.doc_coverage = 0.7;
        metrics.cyclomatic_complexity = 2.0;

        let idx = metrics.maintainability_index();
        assert!(idx > 0.0 && idx <= 100.0);
    }

    // ========================================================================
    // Module-level Tests
    // ========================================================================

    #[test]
    fn test_aggregate_module_metrics_single_file() {
        let file_metrics = vec![FileQualityMetrics {
            path: "file1.rs".to_string(),
            stub_ratio: 0.05,
            cyclomatic_complexity: 2.5,
            max_cyclomatic_complexity: 5.0,
            test_coverage: 85.0,
            doc_coverage: 0.8,
            loc: 200,
            comment_lines: 40,
            public_items: 10,
            documented_public_items: 8,
            cognitive_complexity: 5.0,
            type_coverage: 0.9,
        }];

        let module_metrics = aggregate_module_metrics(&file_metrics);

        assert_eq!(module_metrics.file_count, 1);
        assert_eq!(module_metrics.mean_stub_ratio, 0.05);
        assert_eq!(module_metrics.mean_cyclomatic_complexity, 2.5);
        assert_eq!(module_metrics.total_loc, 200);
    }

    #[test]
    fn test_aggregate_module_metrics_multiple_files() {
        let files = vec![
            FileQualityMetrics {
                path: "file1.rs".to_string(),
                stub_ratio: 0.10,
                cyclomatic_complexity: 3.0,
                max_cyclomatic_complexity: 6.0,
                test_coverage: 90.0,
                doc_coverage: 0.9,
                loc: 150,
                comment_lines: 30,
                public_items: 5,
                documented_public_items: 4,
                cognitive_complexity: 4.0,
                type_coverage: 0.95,
            },
            FileQualityMetrics {
                path: "file2.rs".to_string(),
                stub_ratio: 0.05,
                cyclomatic_complexity: 2.0,
                max_cyclomatic_complexity: 4.0,
                test_coverage: 80.0,
                doc_coverage: 0.7,
                loc: 100,
                comment_lines: 20,
                public_items: 8,
                documented_public_items: 6,
                cognitive_complexity: 3.0,
                type_coverage: 0.85,
            },
        ];

        let module_metrics = aggregate_module_metrics(&files);

        assert_eq!(module_metrics.file_count, 2);
        assert!(module_metrics.mean_stub_ratio > 0.05 && module_metrics.mean_stub_ratio < 0.10);
        assert_eq!(module_metrics.total_loc, 250);
        assert_eq!(module_metrics.total_public_items, 13);
    }

    #[test]
    fn test_aggregate_module_stddev_calculation() {
        let files = vec![
            FileQualityMetrics {
                path: "file1.rs".to_string(),
                stub_ratio: 0.0,
                cyclomatic_complexity: 1.0,
                max_cyclomatic_complexity: 1.0,
                test_coverage: 50.0,
                doc_coverage: 0.5,
                loc: 50,
                comment_lines: 5,
                public_items: 2,
                documented_public_items: 1,
                cognitive_complexity: 1.0,
                type_coverage: 1.0,
            },
            FileQualityMetrics {
                path: "file2.rs".to_string(),
                stub_ratio: 0.2,
                cyclomatic_complexity: 5.0,
                max_cyclomatic_complexity: 10.0,
                test_coverage: 70.0,
                doc_coverage: 0.9,
                loc: 250,
                comment_lines: 50,
                public_items: 8,
                documented_public_items: 7,
                cognitive_complexity: 8.0,
                type_coverage: 0.8,
            },
        ];

        let module_metrics = aggregate_module_metrics(&files);

        assert!(module_metrics.stddev_stub_ratio > 0.0);
        assert!(module_metrics.stddev_cyclomatic_complexity > 0.0);
    }

    // ========================================================================
    // Package-level Tests
    // ========================================================================

    #[test]
    fn test_package_health_score_new() {
        let health = PackageHealthScore::new("mypackage".to_string());
        assert_eq!(health.package_name, "mypackage");
        assert_eq!(health.health_score, 100.0);
    }

    #[test]
    fn test_package_health_score_update() {
        let mut health = PackageHealthScore::new("mypackage".to_string());
        health.test_coverage = 50.0;
        health.doc_coverage = 30.0;
        health.stub_ratio = 0.15;
        health.mean_complexity = 4.0;
        health.violation_count = 2;

        health.update_health_score();

        assert!(health.health_score < 100.0);
        assert!(health.health_score > 0.0);
    }

    #[test]
    fn test_compute_package_health_from_modules() {
        let modules = vec![
            ModuleQualityMetrics {
                module_name: "module1".to_string(),
                file_count: 2,
                mean_stub_ratio: 0.05,
                stddev_stub_ratio: 0.02,
                mean_cyclomatic_complexity: 2.5,
                stddev_cyclomatic_complexity: 0.5,
                max_cyclomatic_complexity: 5.0,
                mean_test_coverage: 85.0,
                stddev_test_coverage: 5.0,
                mean_doc_coverage: 0.8,
                stddev_doc_coverage: 0.1,
                total_loc: 500,
                total_public_items: 20,
                total_documented_items: 16,
                mean_type_coverage: 0.9,
            },
            ModuleQualityMetrics {
                module_name: "module2".to_string(),
                file_count: 3,
                mean_stub_ratio: 0.10,
                stddev_stub_ratio: 0.05,
                mean_cyclomatic_complexity: 3.0,
                stddev_cyclomatic_complexity: 1.0,
                max_cyclomatic_complexity: 7.0,
                mean_test_coverage: 75.0,
                stddev_test_coverage: 10.0,
                mean_doc_coverage: 0.6,
                stddev_doc_coverage: 0.15,
                total_loc: 800,
                total_public_items: 30,
                total_documented_items: 18,
                mean_type_coverage: 0.85,
            },
        ];

        let health = compute_package_health(&modules);

        assert!(health.health_score > 0.0);
        assert!(health.health_score <= 100.0);
        assert!(health.test_coverage > 0.0);
    }

    // ========================================================================
    // Violation Detection Tests
    // ========================================================================

    #[test]
    fn test_detect_high_stub_ratio_violation() {
        let metrics = FileQualityMetrics {
            path: "stubs.rs".to_string(),
            stub_ratio: 0.25,
            cyclomatic_complexity: 2.0,
            max_cyclomatic_complexity: 3.0,
            test_coverage: 50.0,
            doc_coverage: 0.4,
            loc: 100,
            comment_lines: 10,
            public_items: 5,
            documented_public_items: 2,
            cognitive_complexity: 3.0,
            type_coverage: 0.9,
        };

        let violations = detect_object_level_violations(&metrics, 0.05, 0.02);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| {
            matches!(v, ObjectViolation::FileViolation { violation_type, .. } if violation_type == "high_stub_ratio")
        }));
    }

    #[test]
    fn test_detect_high_complexity_violation() {
        let metrics = FileQualityMetrics {
            path: "complex.rs".to_string(),
            stub_ratio: 0.0,
            cyclomatic_complexity: 6.0,
            max_cyclomatic_complexity: 8.0,
            test_coverage: 80.0,
            doc_coverage: 0.8,
            loc: 200,
            comment_lines: 30,
            public_items: 10,
            documented_public_items: 8,
            cognitive_complexity: 10.0,
            type_coverage: 0.9,
        };

        let violations = detect_object_level_violations(&metrics, 2.0, 1.0);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| {
            matches!(v, ObjectViolation::FileViolation { violation_type, .. } if violation_type == "high_cyclomatic_complexity")
        }));
    }

    #[test]
    fn test_detect_low_doc_coverage_violation() {
        let metrics = FileQualityMetrics {
            path: "undocumented.rs".to_string(),
            stub_ratio: 0.0,
            cyclomatic_complexity: 2.0,
            max_cyclomatic_complexity: 3.0,
            test_coverage: 90.0,
            doc_coverage: 0.2,
            loc: 150,
            comment_lines: 20,
            public_items: 10,
            documented_public_items: 2,
            cognitive_complexity: 2.0,
            type_coverage: 0.95,
        };

        let violations = detect_object_level_violations(&metrics, 0.0, 0.05);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| {
            matches!(v, ObjectViolation::FileViolation { violation_type, .. } if violation_type == "low_doc_coverage")
        }));
    }

    #[test]
    fn test_detect_low_type_coverage_violation() {
        let metrics = FileQualityMetrics {
            path: "untyped.rs".to_string(),
            stub_ratio: 0.0,
            cyclomatic_complexity: 2.0,
            max_cyclomatic_complexity: 3.0,
            test_coverage: 80.0,
            doc_coverage: 0.7,
            loc: 100,
            comment_lines: 15,
            public_items: 5,
            documented_public_items: 3,
            cognitive_complexity: 2.0,
            type_coverage: 0.5,
        };

        let violations = detect_object_level_violations(&metrics, 0.0, 0.05);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| {
            matches!(v, ObjectViolation::FileViolation { violation_type, .. } if violation_type == "low_type_coverage")
        }));
    }

    #[test]
    fn test_object_violation_severity_and_description() {
        let violation = ObjectViolation::FileViolation {
            file_path: "test.rs".to_string(),
            violation_type: "high_stub_ratio".to_string(),
            value: 0.25,
            threshold: 0.10,
            severity: "HIGH".to_string(),
        };

        assert_eq!(violation.severity(), "HIGH");
        let desc = violation.description();
        assert!(desc.contains("test.rs"));
        assert!(desc.contains("high_stub_ratio"));
    }

    #[test]
    fn test_api_breaking_change_violation() {
        let violation = ObjectViolation::APIBreakingChange {
            from_package: "core".to_string(),
            to_package: "handlers".to_string(),
            description: "Removed EventHandler::new() method".to_string(),
            severity: "CRITICAL".to_string(),
        };

        assert_eq!(violation.severity(), "CRITICAL");
        let desc = violation.description();
        assert!(desc.contains("core"));
        assert!(desc.contains("handlers"));
        assert!(desc.contains("EventHandler"));
    }

    #[test]
    fn test_dependency_violation() {
        let violation = ObjectViolation::DependencyViolation {
            package_name: "affidavit".to_string(),
            dependency_name: "unsecure-crypto".to_string(),
            reason: "Known vulnerability CVE-2024-123".to_string(),
            severity: "CRITICAL".to_string(),
        };

        assert_eq!(violation.severity(), "CRITICAL");
        let desc = violation.description();
        assert!(desc.contains("affidavit"));
        assert!(desc.contains("unsecure-crypto"));
    }
}
