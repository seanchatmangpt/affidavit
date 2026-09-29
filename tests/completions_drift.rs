//! Drift court for the shell completions (W4-L2, v26.9.28).
//!
//! `completions/affi.bash`, `completions/affi.zsh` and `completions/affi.fish`
//! are GENERATED projections of the static `REGISTRY` in `src/registry.rs`
//! (generator: `scripts/generate_completions.py`, runner: `just completions`).
//!
//! This test is the admission gate that keeps the projection from drifting the
//! way the hand-maintained files did: they claimed "Generated from
//! src/registry.rs — 79 verbs" while the registry reached 83, and never knew
//! the `keys`/`envelope` nouns existed. Like the registry's own drift tests,
//! it parses the artifacts at test time — no cached state, no re-generation:
//! a stale file fails here, loud.
//!
//! Courts (per shell):
//! 1. every registry (noun, verb) pair is completable — adding a verb without
//!    regenerating fails;
//! 2. every completed pair exists in the registry — no orphans;
//! 3. completed noun set == registry noun set, both directions;
//! 4. the header count equals `REGISTRY.len()` — the file cannot quietly
//!    describe a registry it no longer matches;
//! 5. the file still declares itself generated — a hand edit that survives
//!    regeneration silently would defeat the whole chain.

use affidavit::registry::{verb_count, REGISTRY};
use std::collections::{HashMap, HashSet};
use std::path::Path;

fn completions_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/completions"))
}

fn read_completion(file: &str) -> String {
    std::fs::read_to_string(completions_dir().join(file))
        .unwrap_or_else(|e| panic!("completions/{file} unreadable: {e}"))
}

/// The `(noun, verb)` pairs the registry declares.
fn registry_pairs() -> HashSet<(&'static str, &'static str)> {
    REGISTRY
        .iter()
        .map(|entry| (entry.noun, entry.verb))
        .collect()
}

/// The registry noun set.
fn registry_nouns() -> HashSet<&'static str> {
    REGISTRY.iter().map(|entry| entry.noun).collect()
}

/// Registry noun -> its verb set.
fn registry_noun_verbs() -> HashMap<&'static str, HashSet<&'static str>> {
    let mut map: HashMap<&'static str, HashSet<&'static str>> = HashMap::new();
    for entry in REGISTRY {
        map.entry(entry.noun).or_default().insert(entry.verb);
    }
    map
}

/// The `Registry verbs: N` figure each generated header carries.
fn header_verb_count(text: &str, file: &str) -> usize {
    let marker = "Registry verbs: ";
    let line = text
        .lines()
        .find(|l| l.contains(marker))
        .unwrap_or_else(|| {
            panic!(
                "completions/{file} has no `Registry verbs: N` header line — \
                 it was not produced by scripts/generate_completions.py"
            )
        });
    let digits: String = line
        .split_once(marker)
        .unwrap()
        .1
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("completions/{file} header count `{digits}` is not a number"))
}

/// Parse `local <noun>_verbs="a b c"` lines out of the bash script.
fn bash_noun_verbs(text: &str) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("local ") else {
            continue;
        };
        let Some(noun_end) = rest.find("_verbs=\"") else {
            continue;
        };
        let noun = &rest[..noun_end];
        let start = noun_end + "_verbs=\"".len();
        let Some(end_rel) = rest[start..].find('"') else {
            continue;
        };
        let verbs = rest[start..start + end_rel]
            .split_whitespace()
            .map(str::to_string)
            .collect();
        map.insert(noun.to_string(), verbs);
    }
    map
}

/// The token of a zsh `'name[description]'` spec line, if the line is one.
fn zsh_spec_token(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix('\'')?;
    let bracket = rest.find('[')?;
    Some(rest[..bracket].to_string())
}

/// Parse the zsh `noun)` section (noun list) and the per-noun `verb)` case
/// branches into noun -> completed verbs.
fn zsh_completions(text: &str) -> (HashSet<String>, HashMap<String, Vec<String>>) {
    let mut nouns = HashSet::new();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut section = "";
    let mut current_noun: Option<String> = None;
    for line in text.lines() {
        let t = line.trim();
        if t == "noun)" {
            section = "noun";
            continue;
        }
        if t == "verb)" {
            section = "verb";
            continue;
        }
        if t == "args)" || (t == "esac" && section == "verb") {
            section = "";
            current_noun = None;
            continue;
        }
        if section == "noun" {
            if let Some(token) = zsh_spec_token(t) {
                nouns.insert(token);
            }
        } else if section == "verb" {
            // A case branch label like `receipt)` opens a noun block; spec
            // lines always start with `'`, `_values` lines with `_`.
            if t.ends_with(')') && !t.starts_with('\'') && !t.starts_with('_') {
                let noun = t.trim_end_matches(')').to_string();
                map.entry(noun.clone()).or_default();
                current_noun = Some(noun);
                continue;
            }
            if let (Some(noun), Some(token)) = (current_noun.as_ref(), zsh_spec_token(t)) {
                map.entry(noun.clone()).or_default().push(token);
            }
        }
    }
    (nouns, map)
}

/// Parse the fish `complete` lines into the noun set and noun -> verbs.
fn fish_completions(text: &str) -> (HashSet<String>, HashMap<String, Vec<String>>) {
    const NOUN_GUARD: &str = "__affi_no_noun' -a ";
    const VERB_GUARD: &str = "__affi_no_verb' -a ";
    const NOUN_SCOPE: &str = "__affi_using_noun ";
    let mut nouns = HashSet::new();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in text.lines() {
        if let Some(rest) = line.split_once(NOUN_GUARD).map(|(_, r)| r) {
            if let Some(noun) = rest.split_whitespace().next() {
                nouns.insert(noun.to_string());
            }
            continue;
        }
        let Some((guard, rest)) = line.split_once(VERB_GUARD) else {
            continue;
        };
        // The guard sits inside `-n '__affi_using_noun <noun>; and ...'`.
        let noun = guard
            .split_once(NOUN_SCOPE)
            .map(|(_, tail)| tail.split(';').next().unwrap_or("").trim().to_string());
        let Some(noun) = noun else { continue };
        if let Some(verb) = rest.split_whitespace().next() {
            map.entry(noun).or_default().push(verb.to_string());
        }
    }
    (nouns, map)
}

/// The shared admission court: coverage (pairwise), orphans, noun-set
/// equality both directions, and the flat every-registry-verb-appears check.
fn assert_projection_matches_registry(
    file: &str,
    completed: &HashMap<String, Vec<String>>,
    completed_nouns: Option<&HashSet<String>>,
) {
    let regenerate = "run `python3 scripts/generate_completions.py` (or `just completions`)";

    // 1. Every registry (noun, verb) pair is completable.
    let mut missing: Vec<String> = Vec::new();
    for (noun, verb) in registry_pairs() {
        let covered = completed
            .get(noun)
            .is_some_and(|verbs| verbs.iter().any(|v| v == verb));
        if !covered {
            missing.push(format!("{noun} {verb}"));
        }
    }
    assert!(
        missing.is_empty(),
        "completions/{file} does not complete these registered commands: {missing:?}. \
         The completions drifted from src/registry.rs — {regenerate}."
    );

    // 2. No orphans: every completed pair exists in the registry.
    let registry = registry_noun_verbs();
    let mut orphans: Vec<String> = Vec::new();
    for (noun, verbs) in completed {
        let Some(known) = registry.get(noun.as_str()) else {
            orphans.push(format!("{noun} (noun not in registry): {verbs:?}"));
            continue;
        };
        for verb in verbs {
            if !known.contains(verb.as_str()) {
                orphans.push(format!("{noun} {verb}"));
            }
        }
    }
    assert!(
        orphans.is_empty(),
        "completions/{file} completes commands the registry does not declare: {orphans:?}. \
         Remove them at the source or {regenerate}."
    );

    // 3. Noun set equality, both directions (when the format exposes the
    //    completed noun list as a first-class thing).
    if let Some(nouns) = completed_nouns {
        let want: HashSet<String> = registry_nouns().into_iter().map(str::to_string).collect();
        let missing_nouns: Vec<_> = want.difference(nouns).collect();
        let extra_nouns: Vec<_> = nouns.difference(&want).collect();
        assert!(
            missing_nouns.is_empty() && extra_nouns.is_empty(),
            "completions/{file} noun set drifted: missing {missing_nouns:?}, extra {extra_nouns:?} — {regenerate}"
        );
    }

    // 4. Flat check: every registry verb token appears somewhere in the file.
    let all_tokens: HashSet<&str> = completed
        .values()
        .flat_map(|verbs| verbs.iter().map(String::as_str))
        .chain(completed.keys().map(String::as_str))
        .collect();
    let absent: Vec<_> = REGISTRY
        .iter()
        .map(|entry| entry.verb)
        .filter(|verb| !all_tokens.contains(verb))
        .collect();
    assert!(
        absent.is_empty(),
        "completions/{file} never mentions these registry verbs: {absent:?} — {regenerate}"
    );
}

fn assert_header_count(text: &str, file: &str) {
    let count = header_verb_count(text, file);
    assert_eq!(
        count,
        verb_count(),
        "completions/{file} header claims {count} verbs but REGISTRY has {}. \
         The file is stale or hand-edited — run `python3 scripts/generate_completions.py`.",
        verb_count()
    );
}

fn assert_generated_banner(text: &str, file: &str) {
    for marker in [
        "GENERATED FILE — DO NOT EDIT.",
        "scripts/generate_completions.py",
    ] {
        assert!(
            text.contains(marker),
            "completions/{file} lost its generated banner (`{marker}`) — \
             hand edits to a projection are contract violations; regenerate."
        );
    }
}

#[test]
fn bash_completions_match_registry() {
    let text = read_completion("affi.bash");
    assert_generated_banner(&text, "affi.bash");
    assert_header_count(&text, "affi.bash");
    let completed = bash_noun_verbs(&text);
    // bash exposes nouns both as the `local nouns=` list and one `_verbs`
    // variable per noun; the per-noun maps make the noun set implicit.
    let nouns: HashSet<String> = completed.keys().cloned().collect();
    assert_projection_matches_registry("affi.bash", &completed, Some(&nouns));
}

#[test]
fn zsh_completions_match_registry() {
    let text = read_completion("affi.zsh");
    assert_generated_banner(&text, "affi.zsh");
    assert_header_count(&text, "affi.zsh");
    let (nouns, completed) = zsh_completions(&text);
    assert_projection_matches_registry("affi.zsh", &completed, Some(&nouns));
}

#[test]
fn fish_completions_match_registry() {
    let text = read_completion("affi.fish");
    assert_generated_banner(&text, "affi.fish");
    assert_header_count(&text, "affi.fish");
    let (nouns, completed) = fish_completions(&text);
    assert_projection_matches_registry("affi.fish", &completed, Some(&nouns));
}

/// The mutation lens (C05): the parser must be able to fail. If the drift
/// parser silently returned empty maps, court 1 would pass vacuously on a
/// truncated file. Assert the parsers actually saw the full surface.
#[test]
fn parsers_are_not_vacuous() {
    for file in ["affi.bash", "affi.zsh", "affi.fish"] {
        let text = read_completion(file);
        let parsed = match file {
            "affi.bash" => bash_noun_verbs(&text).values().map(Vec::len).sum::<usize>(),
            "affi.zsh" => zsh_completions(&text)
                .1
                .values()
                .map(Vec::len)
                .sum::<usize>(),
            _ => fish_completions(&text)
                .1
                .values()
                .map(Vec::len)
                .sum::<usize>(),
        };
        assert_eq!(
            parsed,
            verb_count(),
            "the {file} drift parser extracted {parsed} verbs from a file whose header claims {} — \
             the parser regressed and every coverage court above is vacuous",
            verb_count()
        );
    }
}
