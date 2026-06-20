//! The `Out` handle — a single, testable output abstraction.
//!
//! All handler verbs should eventually route their output through [`Out`]
//! instead of calling `println!` / `eprintln!` directly.  This provides:
//!
//! - **Consistent formatting:** human vs. JSON output selected in one place.
//! - **Testability:** construct an `Out` backed by `Vec<u8>` in tests.
//! - **Separation of concerns:** data goes to *stdout*; diagnostics go to
//!   *stderr*.  Callers never think about which sink is which.
//!
//! # Quick start
//!
//! ```rust
//! use affidavit::output::{Format, Out};
//!
//! let mut out = Out::with_format(Format::Human);
//! // Write a diagnostic to stderr:
//! use affidavit::diag::{Diag, ErrorCode};
//! let d = Diag::new(ErrorCode::SeqGap, "gap at seq 3");
//! out.diag(&d).ok();
//! ```

use std::io::{self, Write};

use serde::Serialize;

// ---------------------------------------------------------------------------
// Format
// ---------------------------------------------------------------------------

/// Output format selection for the `affi` CLI and library consumers.
///
/// The default is [`Format::Human`] (plain text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Format {
    /// Plain, human-readable text (default).
    #[default]
    Human,
    /// Compact pretty-printed JSON.
    Json,
    /// YAML (falls back to human-readable text for now; full YAML support can
    /// be added later by enabling the `serde_yaml` feature).
    Yaml,
}

impl Format {
    /// Parse a format string as supplied by `--format` CLI flag.
    ///
    /// Unrecognised values fall back to [`Format::Human`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::output::Format;
    /// assert_eq!(Format::from_str("json"), Format::Json);
    /// assert_eq!(Format::from_str("yaml"), Format::Yaml);
    /// assert_eq!(Format::from_str("human"), Format::Human);
    /// assert_eq!(Format::from_str("other"), Format::Human);
    /// ```
    pub fn from_str(s: &str) -> Self {
        match s {
            "json" => Self::Json,
            "yaml" => Self::Yaml,
            _ => Self::Human,
        }
    }
}

// ---------------------------------------------------------------------------
// TTY detection (safe, no external crate)
// ---------------------------------------------------------------------------

/// Detect whether stdout is connected to a terminal.
///
/// Returns `false` by default (safe, no unsafe code, no external crate).
/// Real terminal detection (`isatty(1)`) can be layered in later behind a
/// feature flag once the `colored` dependency is activated.
#[inline]
fn stdout_is_tty() -> bool {
    // Safe default: assume no TTY so we never emit ANSI codes to a pipe.
    // Future: gate behind `#[cfg(feature = "ui")]` with the `colored` crate.
    false
}

// ---------------------------------------------------------------------------
// Out
// ---------------------------------------------------------------------------

/// The single output handle for the `affi` CLI and library consumers.
///
/// Routes **data** (results) to *stdout* and **diagnostics / warnings** to
/// *stderr*, respecting the selected [`Format`] and verbosity flags.
///
/// Construct with [`Out::new`] or the convenience [`Out::with_format`].
/// Use builder methods to set `quiet`, `verbose`, and `no_color`.
///
/// # Examples
///
/// ```rust
/// use affidavit::output::{Format, Out};
///
/// let mut out = Out::with_format(Format::Json);
/// // out.data(&my_value).ok();  // → stdout as pretty JSON
/// ```
pub struct Out {
    /// Selected output format.
    format: Format,
    /// Whether to emit ANSI colour codes.
    ///
    /// Currently unused in output; reserved for future `colored`-feature use.
    #[allow(dead_code)]
    color: bool,
    /// When `true`, suppress all non-data output (warnings, info, diagnostics).
    quiet: bool,
    /// When `true`, emit informational messages to stderr.
    verbose: bool,
    /// Primary output sink — data (results) go here.
    stdout: Box<dyn Write + Send>,
    /// Diagnostic sink — warnings, errors, hints go here.
    stderr: Box<dyn Write + Send>,
}

impl Out {
    /// Construct an `Out` using real stdout/stderr and the given format.
    ///
    /// Color output defaults to the result of TTY detection (currently always
    /// `false` pending the `colored` feature).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::output::{Format, Out};
    /// let mut out = Out::new(Format::Human);
    /// ```
    pub fn new(format: Format) -> Self {
        Self {
            format,
            color: stdout_is_tty(),
            quiet: false,
            verbose: false,
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
        }
    }

    /// Convenience constructor — equivalent to [`Out::new`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::output::{Format, Out};
    /// let mut out = Out::with_format(Format::Json);
    /// ```
    pub fn with_format(format: Format) -> Self {
        Self::new(format)
    }

    /// Construct an `Out` backed by arbitrary [`Write`] sinks.
    ///
    /// Useful in tests: pass `Vec<u8>` buffers to capture output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::output::{Format, Out};
    ///
    /// let mut stdout_buf: Vec<u8> = Vec::new();
    /// let mut stderr_buf: Vec<u8> = Vec::new();
    /// // SAFETY: we need the raw pointer dance to construct two boxes from one buffer;
    /// // use the public API via Into<Box<dyn Write + Send>> instead.
    /// let out = Out::with_sinks(
    ///     Format::Json,
    ///     Box::new(std::io::Cursor::new(Vec::<u8>::new())),
    ///     Box::new(std::io::Cursor::new(Vec::<u8>::new())),
    /// );
    /// ```
    pub fn with_sinks(
        format: Format,
        stdout: Box<dyn Write + Send>,
        stderr: Box<dyn Write + Send>,
    ) -> Self {
        Self {
            format,
            color: false,
            quiet: false,
            verbose: false,
            stdout,
            stderr,
        }
    }

    // -----------------------------------------------------------------------
    // Builder methods
    // -----------------------------------------------------------------------

    /// Suppress all non-data output (warnings, info messages, diagnostics).
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Enable informational messages on stderr.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Disable ANSI colour codes (no-op until the `colored` feature is active).
    pub fn no_color(mut self) -> Self {
        self.color = false;
        self
    }

    // -----------------------------------------------------------------------
    // Output methods
    // -----------------------------------------------------------------------

    /// Write a data value to *stdout*.
    ///
    /// In [`Format::Json`] mode the value is serialized as pretty-printed JSON.
    /// In all other modes the value's [`Display`](std::fmt::Display) output is
    /// used.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to the underlying sink fails.
    pub fn data<T: Serialize + std::fmt::Display>(&mut self, value: &T) -> io::Result<()> {
        match self.format {
            Format::Json => {
                let json = serde_json::to_string_pretty(value)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                writeln!(self.stdout, "{}", json)
            }
            Format::Human | Format::Yaml => writeln!(self.stdout, "{}", value),
        }
    }

    /// Write a raw [`serde_json::Value`] to *stdout* as pretty-printed JSON.
    ///
    /// This is useful when the caller has already constructed a JSON value and
    /// does not need the format-selection logic in [`data`](Self::data).
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if serialization or writing fails.
    pub fn json(&mut self, value: &serde_json::Value) -> io::Result<()> {
        let s = serde_json::to_string_pretty(value)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        writeln!(self.stdout, "{}", s)
    }

    /// Write a [`Diag`](crate::diag::Diag) to *stderr*.
    ///
    /// Emits `[E<code>] <message>` followed by `  hint: <hint>` when a hint
    /// is present.  Suppressed when [`quiet`](Self::quiet) is active.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to the underlying sink fails.
    pub fn diag(&mut self, diag: &crate::diag::Diag) -> io::Result<()> {
        if !self.quiet {
            writeln!(self.stderr, "[E{:04}] {}", diag.code, diag.message)?;
            if let Some(hint) = &diag.hint {
                writeln!(self.stderr, "  hint: {}", hint)?;
            }
        }
        Ok(())
    }

    /// Write a warning message to *stderr*.
    ///
    /// Suppressed when [`quiet`](Self::quiet) is active.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to the underlying sink fails.
    pub fn warn(&mut self, msg: &str) -> io::Result<()> {
        if !self.quiet {
            writeln!(self.stderr, "warn: {}", msg)?;
        }
        Ok(())
    }

    /// Write an informational message to *stderr*.
    ///
    /// Only emitted when [`verbose`](Self::verbose) is active and
    /// [`quiet`](Self::quiet) is not.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to the underlying sink fails.
    pub fn info(&mut self, msg: &str) -> io::Result<()> {
        if self.verbose && !self.quiet {
            writeln!(self.stderr, "info: {}", msg)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use super::*;
    use crate::diag::{Diag, ErrorCode};

    // -----------------------------------------------------------------------
    // Test helper: shared-arc writer so we can inspect bytes after the call.
    // -----------------------------------------------------------------------

    struct SharedVec(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedVec {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Build an `Out` backed by in-memory buffers; returns the `Out` plus the
    /// `Arc<Mutex<Vec<u8>>>` handles so callers can inspect bytes after each
    /// write.  No raw pointers, no unsafe.
    fn build_out(
        format: Format,
    ) -> (Out, Arc<Mutex<Vec<u8>>>, Arc<Mutex<Vec<u8>>>) {
        let stdout_arc: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let stderr_arc: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

        let out = Out::with_sinks(
            format,
            Box::new(SharedVec(Arc::clone(&stdout_arc))),
            Box::new(SharedVec(Arc::clone(&stderr_arc))),
        );
        (out, stdout_arc, stderr_arc)
    }

    // -----------------------------------------------------------------------

    #[test]
    fn format_from_str_parses_known_values() {
        assert_eq!(Format::from_str("json"), Format::Json);
        assert_eq!(Format::from_str("yaml"), Format::Yaml);
        assert_eq!(Format::from_str("human"), Format::Human);
    }

    #[test]
    fn format_from_str_falls_back_to_human() {
        assert_eq!(Format::from_str(""), Format::Human);
        assert_eq!(Format::from_str("unknown"), Format::Human);
        assert_eq!(Format::from_str("JSON"), Format::Human); // case-sensitive
    }

    #[test]
    fn format_default_is_human() {
        assert_eq!(Format::default(), Format::Human);
    }

    #[test]
    fn data_human_uses_display() {
        struct Msg;
        impl std::fmt::Display for Msg {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("hello from display")
            }
        }
        impl Serialize for Msg {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str("hello from display")
            }
        }

        let (mut out, so, _se) = build_out(Format::Human);
        out.data(&Msg).expect("data");
        let bytes = so.lock().unwrap().clone();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("hello from display"));
    }

    #[test]
    fn data_json_emits_pretty_json() {
        #[derive(Serialize)]
        struct Item {
            x: u32,
        }
        impl std::fmt::Display for Item {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Item({})", self.x)
            }
        }

        let (mut out, so, _se) = build_out(Format::Json);
        out.data(&Item { x: 42 }).expect("data");
        let bytes = so.lock().unwrap().clone();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("\"x\""));
        assert!(s.contains("42"));
    }

    #[test]
    fn json_writes_to_stdout() {
        let (mut out, so, _se) = build_out(Format::Human);
        out.json(&json!({"key": "value"})).expect("json");
        let bytes = so.lock().unwrap().clone();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("\"key\""));
        assert!(s.contains("\"value\""));
    }

    #[test]
    fn diag_writes_code_and_message_to_stderr() {
        let (mut out, _so, se) = build_out(Format::Human);
        let d = Diag::new(ErrorCode::SeqGap, "gap at seq 5");
        out.diag(&d).expect("diag");
        let bytes = se.lock().unwrap().clone();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("[E1003]"), "expected E1003 got: {s}");
        assert!(s.contains("gap at seq 5"));
    }

    #[test]
    fn diag_writes_hint_when_present() {
        let (mut out, _so, se) = build_out(Format::Human);
        let d = Diag::new(ErrorCode::SeqGap, "gap").with_hint("fix the gap");
        out.diag(&d).expect("diag");
        let bytes = se.lock().unwrap().clone();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("hint: fix the gap"));
    }

    #[test]
    fn diag_suppressed_when_quiet() {
        let (out, _so, se) = build_out(Format::Human);
        let mut out = out.quiet();
        let d = Diag::new(ErrorCode::SeqGap, "gap");
        out.diag(&d).expect("diag");
        let bytes = se.lock().unwrap().clone();
        assert!(bytes.is_empty(), "quiet mode should suppress diag output");
    }

    #[test]
    fn warn_writes_to_stderr() {
        let (mut out, _so, se) = build_out(Format::Human);
        out.warn("something smells wrong").expect("warn");
        let bytes = se.lock().unwrap().clone();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("warn: something smells wrong"));
    }

    #[test]
    fn warn_suppressed_when_quiet() {
        let (out, _so, se) = build_out(Format::Human);
        let mut out = out.quiet();
        out.warn("ignored").expect("warn");
        let bytes = se.lock().unwrap().clone();
        assert!(bytes.is_empty());
    }

    #[test]
    fn info_only_emits_when_verbose() {
        // Not verbose → no output
        {
            let (mut out, _so, se) = build_out(Format::Human);
            out.info("detail").expect("info");
            let bytes = se.lock().unwrap().clone();
            assert!(bytes.is_empty(), "info should be silent without verbose");
        }
        // Verbose → output
        {
            let (out, _so, se) = build_out(Format::Human);
            let mut out = out.verbose();
            out.info("detail").expect("info");
            let bytes = se.lock().unwrap().clone();
            let s = String::from_utf8(bytes).unwrap();
            assert!(s.contains("info: detail"));
        }
    }

    #[test]
    fn info_suppressed_when_quiet_and_verbose() {
        let (out, _so, se) = build_out(Format::Human);
        let mut out = out.verbose().quiet();
        out.info("detail").expect("info");
        let bytes = se.lock().unwrap().clone();
        assert!(bytes.is_empty(), "quiet takes precedence over verbose");
    }

    #[test]
    fn diag_writes_nothing_to_stdout() {
        let (mut out, so, _se) = build_out(Format::Human);
        let d = Diag::new(ErrorCode::ReceiptNotFound, "missing");
        out.diag(&d).expect("diag");
        let bytes = so.lock().unwrap().clone();
        assert!(bytes.is_empty(), "diag must not write to stdout");
    }

    #[test]
    fn no_color_sets_color_false() {
        // Just checks the builder method doesn't panic.
        let out = Out::with_format(Format::Human).no_color();
        drop(out);
    }
}
