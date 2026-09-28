// Release-identity gates.
//
// affidavit binds its chain genesis seed to the package version:
//
//     const GENESIS_SEED_STR: &str =
//         concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis");
//
// so a receipt only verifies under the exact binary version that assembled it.
// That makes the version string load-bearing evidence, not metadata. These
// tests hold the pieces of the release identity to each other so they cannot
// drift apart silently — which is exactly how bug B4 (`GENESIS_SEED` pinned to
// 26.6.14 while the package was 26.6.17) happened.

use assert_cmd::Command;
use std::fs;

/// The version this test binary was compiled against — the same expansion the
/// genesis seed uses.
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
fn the_binary_reports_the_affidavit_version_not_the_framework_version() {
    // `clap-noun-verb` builds its root command with its OWN CARGO_PKG_VERSION,
    // so before v26.9.6 `affi --version` answered `cli 26.6.2`. An operator
    // diagnosing a cross-version chain_integrity failure reads this string
    // first; it has to name the binary that produced the receipt.
    let output = Command::cargo_bin("affi")
        .expect("affi binary builds")
        .arg("--version")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let reported = String::from_utf8(output).expect("version output is UTF-8");
    let reported = reported.trim();

    assert_eq!(
        reported,
        format!("affi {PKG_VERSION}"),
        "affi --version must report the affidavit package version"
    );
    assert!(
        !reported.contains("cli "),
        "affi --version must not leak the clap-noun-verb framework version: {reported}"
    );
}

#[test]
fn the_genesis_seed_tracks_the_package_version() {
    // The seed is derived, never typed. If someone replaces the `concat!` with
    // a literal, this fails at the next version bump instead of silently
    // producing receipts that no other build can verify.
    let seed = std::str::from_utf8(affidavit::chain::GENESIS_SEED).expect("seed is UTF-8");
    assert_eq!(
        seed,
        format!("affidavit-v{PKG_VERSION}-genesis"),
        "GENESIS_SEED must be derived from CARGO_PKG_VERSION (bug B4)"
    );
}

#[test]
fn the_changelog_documents_the_current_version() {
    // A release whose version moved without a changelog entry is a release
    // nobody can audit. This gate makes the entry mandatory rather than
    // customary.
    let changelog = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/CHANGELOG.md"))
        .expect("CHANGELOG.md is readable");
    let heading = format!("## [{PKG_VERSION}]");
    assert!(
        changelog.contains(&heading),
        "CHANGELOG.md has no `{heading}` section. Bumping the version without \
         documenting it leaves operators unable to tell why their receipts stopped verifying."
    );
}

#[test]
fn the_readme_states_the_live_verb_count() {
    // README, registry, and binary must agree on how many verbs exist. The
    // registry is authoritative; the README is a projection of it.
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("README.md is readable");
    let claim = format!("{} canonical verbs", affidavit::registry::verb_count());
    assert!(
        readme.contains(&claim),
        "README.md must state `{claim}` to match src/registry.rs"
    );
}

#[test]
fn the_browser_verifier_uses_the_same_genesis_seed_as_the_binary() {
    // web/ ships a TypeScript re-implementation of the chain verifier. Its
    // genesis seed is a hand-typed literal, and it had silently drifted three
    // releases behind (26.6.17 while the crate was 26.6.22) — which means the
    // browser verifier rejected every receipt the binary produced. Nothing
    // caught it because the web lane's only gate is `tsc --noEmit`, which
    // cannot know what the Rust seed is. This test can.
    let expected = format!("affidavit-v{PKG_VERSION}-genesis");
    for relative in ["/web/lib/verify-client.ts", "/web/app/visualizer/model.ts"] {
        let path = format!("{}{relative}", env!("CARGO_MANIFEST_DIR"));
        let Ok(source) = fs::read_to_string(&path) else {
            // The web lane is an optional sibling; skip if it is not checked out.
            continue;
        };
        let stale: Vec<&str> = source
            .match_indices("affidavit-v")
            .map(|(i, _)| {
                let rest = &source[i..];
                let end = rest.find('"').unwrap_or(rest.len());
                &rest[..end]
            })
            .filter(|seed| *seed != expected)
            .collect();
        assert!(
            stale.is_empty(),
            "{path} carries genesis seed(s) {stale:?} but the binary uses `{expected}`. \
             A drifted seed makes the browser verifier reject every real receipt."
        );
        assert!(
            source.contains(&expected),
            "{path} does not mention the current genesis seed `{expected}`"
        );
    }
}

#[test]
fn orphaned_sources_are_declared_or_excluded() {
    // A file under src/ that no `mod` declares and no `#[path]` maps is never
    // seen by the compiler: it is never type-checked, never linted, and never
    // tested — yet `cargo package` would ship it as part of the crate. Either
    // wire it up or name it in Cargo.toml's `exclude`. This test refuses the
    // third option of leaving it silently in between.
    let root = env!("CARGO_MANIFEST_DIR");
    let lib_rs = fs::read_to_string(format!("{root}/src/lib.rs")).expect("src/lib.rs is readable");
    let manifest =
        fs::read_to_string(format!("{root}/Cargo.toml")).expect("Cargo.toml is readable");

    let mut orphans = Vec::new();
    for entry in fs::read_dir(format!("{root}/src")).expect("src/ is readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("file name is UTF-8")
            .to_string();
        if file_name == "lib.rs" {
            continue;
        }
        let stem = file_name.trim_end_matches(".rs");

        // Declared as `mod <stem>;` (possibly `pub`, possibly cfg-gated above)?
        let declared = lib_rs.contains(&format!("mod {stem};"));
        // Mapped by `#[path = "<file>"]`?
        let mapped = lib_rs.contains(&format!("#[path = \"{file_name}\"]"));
        // Named in the package `exclude` list?
        let excluded = manifest.contains(&format!("\"src/{file_name}\""));

        if !declared && !mapped && !excluded {
            orphans.push(file_name);
        }
    }
    orphans.sort();

    assert!(
        orphans.is_empty(),
        "these files sit in src/ but are never compiled and are not excluded from the \
         published package: {orphans:?}. Declare them with `mod`, map them with `#[path]`, \
         or add them to Cargo.toml's `exclude`."
    );
}

#[test]
fn shell_completions_offer_only_dispatchable_verbs() {
    // The completions are hand-maintained, and they had copied the registry's
    // old snake_case spelling: 39 of the names they offered (`verify_compliance`,
    // `root_cause`, `emit_from_github`, …) are rejected by the binary, which
    // dispatches kebab-case. A completion that types a command the CLI refuses
    // is worse than no completion. They also advertised a `quality` noun and
    // three `guide` verbs that do not exist.
    //
    // Generating them from REGISTRY is the real fix (ROADMAP P1-5); until then
    // this test is what keeps them honest.
    let live: std::collections::HashSet<String> = affidavit::registry::REGISTRY
        .iter()
        .map(|entry| entry.verb.to_string())
        .collect();
    let nouns: std::collections::HashSet<String> = affidavit::registry::REGISTRY
        .iter()
        .map(|entry| entry.noun.to_string())
        .collect();

    for shell in ["bash", "zsh", "fish"] {
        let path = format!("{}/completions/affi.{shell}", env!("CARGO_MANIFEST_DIR"));
        let source = fs::read_to_string(&path).expect("completion script is readable");

        // Any token that looks like a verb name but carries `_` cannot be
        // dispatched. Shell *variable* names (receipt_verbs, affi_verbs) are
        // not offered to the user, so only quoted/offered tokens are checked.
        let offered: Vec<&str> = match shell {
            // bash: the space-separated word lists assigned to *_verbs
            "bash" => source
                .split("_verbs=\"")
                .skip(1)
                .filter_map(|rest| rest.split('"').next())
                .flat_map(str::split_whitespace)
                .collect(),
            // zsh: 'verb[Description]'
            "zsh" => source
                .split('\'')
                .filter(|chunk| chunk.contains('['))
                .filter_map(|chunk| chunk.split('[').next())
                .filter(|tok| {
                    !tok.is_empty()
                        && tok.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
                        })
                })
                .collect(),
            // fish: `-a <token>`
            _ => source
                .split(" -a ")
                .skip(1)
                .filter_map(|rest| rest.split_whitespace().next())
                .filter(|tok| {
                    tok.chars().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
                    })
                })
                .collect(),
        };

        assert!(
            !offered.is_empty(),
            "parsed no completion tokens out of {path} — the parser is wrong, not the file"
        );

        let undispatchable: Vec<&&str> = offered
            .iter()
            // Flags are offered too and are not registry entries.
            .filter(|tok| !tok.starts_with('-'))
            .filter(|tok| !matches!(**tok, "help"))
            .filter(|tok| !live.contains(**tok) && !nouns.contains(**tok))
            .collect();
        assert!(
            undispatchable.is_empty(),
            "completions/affi.{shell} offers tokens the binary does not dispatch: {undispatchable:?}. \
             The CLI uses kebab-case; every offered verb must appear in REGISTRY."
        );
    }
}
