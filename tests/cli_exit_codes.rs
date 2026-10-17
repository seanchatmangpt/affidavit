//! Court: CLI framework surface errors must exit with stable codes from the
//! 1500-1599 range in `src/diag.rs` — never a bare 1.
//!
//! Register row 22. Spawns the real built binary (Chicago: real collaborators,
//! no mocks) and asserts on actual process exit codes and structured output.

use std::process::Command;

/// Strip ANSI escapes so hint assertions are terminal-independent.
fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&n) = chars.peek() {
                    chars.next();
                    if n as u32 >= 0x40 && n as u32 <= 0x7E {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn run_affi(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(args)
        .output()
        .expect("spawn affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// POSIX exit status is 8-bit: the stable 1500–1599 diagnostic code is
/// encoded mod 256 on the process exit status; the JSON `code` field carries
/// the full value.
const EXIT_COMMAND_NOT_FOUND: i32 = 1500 % 256; // 220
const EXIT_VERB_NOT_FOUND: i32 = 1501 % 256; // 221

#[test]
fn bogus_noun_exits_in_1500_range_with_did_you_mean_hint() {
    let (code, stdout, stderr) = run_affi(&["bogus-noun-xyz"]);
    assert_eq!(
        code, EXIT_COMMAND_NOT_FOUND,
        "exit must encode CliCommandNotFound (1500 mod 256 = 220)"
    );
    let clean = strip_ansi(&format!("{stdout}{stderr}"));
    assert!(
        clean.contains("bogus-noun-xyz"),
        "output must name the offending noun: {clean}"
    );
    // Structured diagnostic: full stable code + structured hint field.
    assert!(
        clean.contains("\"code\":1500"),
        "diagnostic must carry the stable 1500-range code: {clean}"
    );
    assert!(
        clean.contains("\"hint\":"),
        "diagnostic must be structured JSON with a hint field: {clean}"
    );
}

#[test]
fn bogus_noun_close_to_real_noun_carries_levenshtein_hint() {
    // `reciept` is within Levenshtein distance 3 of `receipt`.
    let (code, stdout, stderr) = run_affi(&["reciept"]);
    assert_eq!(
        code, EXIT_COMMAND_NOT_FOUND,
        "reclassified as CliCommandNotFound (1500 mod 256 = 220)"
    );
    let clean = strip_ansi(&format!("{stdout}{stderr}"));
    assert!(
        clean.contains("\"code\":1500"),
        "stable code in JSON: {clean}"
    );
    assert!(
        clean.contains("\"hint\":\"receipt\""),
        "structured hint field must be exactly the suggestion: {clean}"
    );
}

#[test]
fn bogus_verb_exits_1501_with_hint() {
    let (code, stdout, stderr) = run_affi(&["receipt", "bogus-verb-xyz"]);
    assert_eq!(
        code, EXIT_VERB_NOT_FOUND,
        "exit must encode CliVerbNotFound (1501 mod 256 = 221); stdout={stdout} stderr={stderr}"
    );
    let clean = strip_ansi(&format!("{stdout}{stderr}"));
    assert!(clean.contains("bogus-verb-xyz"), "output: {clean}");
    assert!(
        clean.contains("\"code\":1501"),
        "diagnostic stable code in output: {clean}"
    );
    assert!(clean.contains("\"hint\""), "structured JSON: {clean}");
}

#[test]
fn valid_invocation_exits_zero() {
    let (code, _stdout, stderr) = run_affi(&["--version"]);
    assert_eq!(code, 0, "stderr={stderr}");
}

#[test]
fn valid_noun_verb_invocation_exits_zero() {
    let (code, _stdout, stderr) = run_affi(&["receipt", "verify", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
}
#[test]
fn valid_noun_help_exits_zero() {
    let (code, _stdout, stderr) = run_affi(&["receipt", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
}
