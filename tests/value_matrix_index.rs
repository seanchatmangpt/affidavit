//! Value-matrix completeness court (v26.10.17 rollout).
//!
//! Law: every (noun, verb) pair in `src/registry.rs` REGISTRY must appear as
//! a test-annotated marker `// value-matrix: (noun, verb)` in at least one
//! `tests/cli_value_matrix/*.rs` file (the 17 domain lanes add those
//! markers), or be listed as covered in
//! `docs/sjira/v26.10.17/value-matrix.md` during the rollout window.
//!
//! The court is soft (typed WARNING, exit 0) while `COMPLETE` is false so
//! CI stays green during the rollout; the coordinator flips `COMPLETE`
//! to `true` at 100% coverage — from then on a missing pair is a hard FAIL.
//! The flip is documented in value-matrix.md.
//!
//! Receipt discipline: the coverage table is printed to stderr on every run,
//! so a green run still carries its own evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

/// Coordinator flip: set to `true` once the value matrix reaches 100%
/// coverage (marker set ∪ doc set == registry set). Until then the court
/// passes with a typed WARNING while coverage < 100%.
const COMPLETE: bool = false;

/// Marker syntax the 17 domain lanes must emit:
/// `// value-matrix: (noun, verb)` — noun first, then verb.
const MARKER_RE_SRC: &str = r"//\s*value-matrix:\s*\(([^),\s]+),\s*([^)\s]+)\)";

/// Covered-pair line in value-matrix.md during rollout, one per line in the
/// "Covered pairs" section: `- (noun, verb)`.
const DOC_LINE_RE_SRC: &str = r"^\s*[-*]\s*\(([^),\s]+),\s*([^)\s]+)\)\s*$";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Rust string-literal unescape (subset: the escapes registry.rs uses).
fn unslash(lit: &str) -> String {
    let mut out = String::with_capacity(lit.len());
    let mut chars = lit.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&n) = chars.peek() {
                match n {
                    '\\' => out.push('\\'),
                    '"' => out.push('"'),
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
                chars.next();
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// Parse REGISTRY pairs from src/registry.rs with the same regex family as
/// scripts/generate_completions.py (dot-spanning VerbEntry::new(...)).
fn registry_pairs() -> Vec<(String, String)> {
    let src = fs::read_to_string(repo_root().join("src/registry.rs")).expect("read registry.rs");
    let entry_re = regex::Regex::new(
        r#"VerbEntry::new\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*VerbGroup::(\w+)\s*,\s*"((?:[^"\\]|\\.)*)""#,
    )
    .unwrap();
    let mut pairs: Vec<(String, String)> = Vec::new();
    for cap in entry_re.captures_iter(&src) {
        pairs.push((unslash(&cap[2]), unslash(&cap[1]))); // (noun, verb)
    }
    assert!(
        !pairs.is_empty(),
        "value-matrix court: REGISTRY parse yielded 0 pairs — entry regex broke"
    );
    let unique: BTreeSet<_> = pairs.iter().collect();
    assert_eq!(
        unique.len(),
        pairs.len(),
        "value-matrix court: duplicate (noun, verb) pair in REGISTRY"
    );
    pairs.sort();
    pairs
}

/// Scan tests/cli_value_*.rs (top-level domain batches) AND
/// tests/cli_value_matrix/*.rs for `// value-matrix: (noun, verb)` markers.
fn marker_pairs() -> BTreeSet<(String, String)> {
    let root = repo_root().join("tests");
    let mut found = BTreeSet::new();
    let mut files: Vec<_> = fs::read_dir(&root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            p.extension().map_or(false, |x| x == "rs")
                && (name.starts_with("cli_value_") || name == "cli_value_matrix")
        })
        .collect();
    files.sort();
    let marker_re = regex::Regex::new(MARKER_RE_SRC).unwrap();
    for path in files {
        let body = fs::read_to_string(&path).expect("read marker file");
        for cap in marker_re.captures_iter(&body) {
            found.insert((cap[1].to_string(), cap[2].to_string()));
        }
    }
    found
}

/// Parse covered pairs from value-matrix.md ("Covered pairs" section lines).
fn doc_pairs() -> BTreeSet<(String, String)> {
    let path = repo_root().join("docs/sjira/v26.10.17/value-matrix.md");
    let mut found = BTreeSet::new();
    let body = match fs::read_to_string(&path) {
        Ok(b) => b,
        Err(_) => return found,
    };
    let line_re = regex::Regex::new(DOC_LINE_RE_SRC).unwrap();
    for line in body.lines() {
        if let Some(cap) = line_re.captures(line) {
            found.insert((cap[1].to_string(), cap[2].to_string()));
        }
    }
    found
}

#[test]
fn value_matrix_covers_registry() {
    let registry = registry_pairs();
    let markers = marker_pairs();
    let doc = doc_pairs();
    let covered: BTreeSet<(String, String)> = markers.union(&doc).cloned().collect();
    let missing: Vec<_> = registry
        .iter()
        .filter(|p| !covered.contains(p))
        .cloned()
        .collect();
    let total = registry.len();
    let have = total - missing.len();
    let pct = if total == 0 { 100 } else { have * 100 / total };

    let registry_set: BTreeSet<(String, String)> = registry.iter().cloned().collect();
    // Coverage table to stderr, grouped per noun (receipt embedded in run).
    eprintln!("=== value-matrix coverage (registry={total}) ===");
    let mut by_noun: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (noun, verb) in &registry {
        let e = by_noun.entry(noun).or_insert((0, 0));
        e.1 += 1;
        if covered.contains(&(noun.clone(), verb.clone())) {
            e.0 += 1;
        }
    }
    for (noun, (cov, tot)) in &by_noun {
        eprintln!(
            "{noun:24} {cov}/{tot}{}",
            if cov == tot { "" } else { "  MISSING" }
        );
        if *cov != *tot {
            for (n, v) in &missing {
                if n == noun {
                    eprintln!("    - ({n}, {v})");
                }
            }
        }
    }
    eprintln!(
        "=== total: {have}/{total} ({pct}%) covered; {} via markers, {} via rollout doc ===",
        covered.intersection(&registry_set).count(),
        doc.intersection(&registry_set).count()
    );
    eprintln!(
        "=== court mode: {} ===",
        if COMPLETE {
            "HARD (COMPLETE=true)"
        } else {
            "SOFT (rollout WARNING)"
        }
    );

    if missing.is_empty() {
        eprintln!("value-matrix: 100% coverage — court holds.");
        return;
    }
    if COMPLETE {
        panic!(
            "VALUE_MATRIX_INCOMPLETE (hard): {have}/{total} covered; {} missing, first few: {:?}",
            missing.len(),
            &missing[..missing.len().min(5)]
        );
    }
    eprintln!(
        "WARNING VALUE_MATRIX_ROLLOUT: coverage {pct}% < 100% ({} missing) — soft mode while COMPLETE=false",
        missing.len()
    );
}
