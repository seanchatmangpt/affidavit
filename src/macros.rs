//! Internal stdout macros that keep the crate's `#![deny(clippy::print_stdout)]`
//! gate honest without threading an [`Out`](crate::output::Out) handle through
//! every function.
//!
//! Library code uses `outln!` / `out!` instead of `println!` / `print!`. Both
//! route through the single sanctioned sink in [`crate::output`]
//! ([`line`](crate::output::line) / [`print_`](crate::output::print_)), which is
//! the one place allowed to touch stdout directly. Diagnostics still go to
//! stderr via `eprintln!` (not covered by the lint).
//!
//! Prefer a real [`Out`](crate::output::Out) for new, format-aware/testable
//! output; these macros exist to retire the large body of legacy `println!`
//! calls in one mechanical, behaviour-preserving step.

/// Like `println!`, but routes through [`crate::output::line`] (the single
/// `#[allow(clippy::print_stdout)]` site) instead of writing to stdout directly.
//
// `allow(unused_macros)`: some feature combinations compile out every caller.
#[allow(unused_macros)]
macro_rules! outln {
    () => { $crate::output::line(format_args!("")) };
    ($($arg:tt)*) => { $crate::output::line(format_args!($($arg)*)) };
}

/// Like `print!` (no trailing newline), routed through [`crate::output::print_`].
//
// `allow(unused_macros)`: the only callers live behind optional features.
#[allow(unused_macros)]
macro_rules! out {
    ($($arg:tt)*) => { $crate::output::print_(format_args!($($arg)*)) };
}
