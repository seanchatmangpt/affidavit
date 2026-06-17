// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `affi-shell` — Combinatorial Maximalist REPL for Affidavit.
//!
//! Provides an interactive environment for managing receipts, models, and
//! provenance evidence. Built with `rustyline` for professional CLI UX.

#[cfg(feature = "shell")]
mod shell_impl {
    use anyhow::{anyhow, Result};
    use rustyline::completion::{Completer, FilenameCompleter, Pair};
    use rustyline::error::ReadlineError;
    use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
    use rustyline::hint::{Hinter, HistoryHinter};
    use rustyline::validate::{
        MatchingBracketValidator, ValidationContext, ValidationResult, Validator,
    };
    use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
    use shlex::split;
    use std::borrow::Cow;

    use affidavit::handlers;

    #[derive(Helper)]
    struct AffiHelper {
        completer: FilenameCompleter,
        highlighter: MatchingBracketHighlighter,
        hinter: HistoryHinter,
        validator: MatchingBracketValidator,
        verbs: Vec<String>,
    }

    impl Completer for AffiHelper {
        type Candidate = Pair;

        fn complete(
            &self,
            line: &str,
            pos: usize,
            ctx: &Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Pair>)> {
            // First, try filename completion
            let mut result = self.completer.complete(line, pos, ctx)?;

            // Then, add verb completions if we're at the start of a word
            let word = &line[..pos];
            let last_space = word.rfind(' ').map(|i| i + 1).unwrap_or(0);
            let current_word = &word[last_space..];

            for verb in &self.verbs {
                if verb.starts_with(current_word) {
                    result.1.push(Pair {
                        display: verb.clone(),
                        replacement: verb.clone(),
                    });
                }
            }

            Ok(result)
        }
    }

    impl Hinter for AffiHelper {
        type Hint = String;
        fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
            self.hinter.hint(line, pos, ctx)
        }
    }

    impl Highlighter for AffiHelper {
        fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
            &'s self,
            prompt: &'p str,
            default: bool,
        ) -> Cow<'b, str> {
            if default {
                Cow::Owned(format!("\x1b[1;32m{}\x1b[0m", prompt))
            } else {
                Cow::Borrowed(prompt)
            }
        }

        fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
            Cow::Owned(format!("\x1b[2;37m{}\x1b[0m", hint))
        }

        fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
            self.highlighter.highlight(line, pos)
        }

        fn highlight_char(&self, line: &str, pos: usize) -> bool {
            self.highlighter.highlight_char(line, pos)
        }
    }

    impl Validator for AffiHelper {
        fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
            let input = ctx.input();
            // Multi-line support: if line ends with '\', it's incomplete.
            if input.ends_with('\\') {
                Ok(ValidationResult::Incomplete)
            } else {
                self.validator.validate(ctx)
            }
        }
    }

    pub async fn run() -> Result<()> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();

        let h = AffiHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            validator: MatchingBracketValidator::new(),
            verbs: vec![
                "receipt".into(),
                "emit".into(),
                "assemble".into(),
                "verify".into(),
                "show".into(),
                "inspect".into(),
                "stats".into(),
                "graph".into(),
                "replay".into(),
                "model".into(),
                "conformance".into(),
                "diagnose".into(),
                "help".into(),
                "exit".into(),
                "clear".into(),
                "history".into(),
            ],
        };

        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(h));

        let history_path = std::env::current_dir()?.join(".affi_history");
        if history_path.exists() {
            let _ = rl.load_history(&history_path);
        }

        println!("\x1b[1;34mAffidavit Shell (affi-shell)\x1b[0m — v26.6.14");
        println!("Type 'help' for commands, '\\' at end for multi-line, Ctrl-D to exit.");

        loop {
            let p = "affi> ";
            let readline = rl.readline(p);
            match readline {
                Ok(line) => {
                    let mut line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }

                    // Handle multi-line continuation character
                    if line.ends_with('\\') {
                        line.pop();
                    }

                    rl.add_history_entry(&line)?;

                    if let Err(e) = dispatch(&line).await {
                        eprintln!("\x1b[1;31merror:\x1b[0m {e}");
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Interrupted (Ctrl-C)");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Readline error: {:?}", err);
                    break;
                }
            }
        }

        rl.save_history(&history_path)?;
        Ok(())
    }

    async fn dispatch(line: &str) -> Result<()> {
        let args = split(line).ok_or_else(|| anyhow!("Invalid quoting in command"))?;
        if args.is_empty() {
            return Ok(());
        }

        match args[0].as_str() {
            "exit" | "quit" => std::process::exit(0),
            "help" => {
                println!("\x1b[1mAvailable commands:\x1b[0m");
                println!("  \x1b[33mreceipt emit <payload> <object...> --type <type>\x1b[0m");
                println!("  \x1b[33mreceipt assemble [out]\x1b[0m");
                println!("  \x1b[33mreceipt verify <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt show <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt inspect <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt stats <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt graph <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt replay <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt model <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt conformance <receipt>\x1b[0m");
                println!("  \x1b[33mreceipt diagnose <receipt>\x1b[0m");
                println!("  \x1b[36mhistory\x1b[0m - instructions for history navigation");
                println!("  \x1b[36mclear\x1b[0m   - clear terminal");
                println!("  \x1b[36mhelp\x1b[0m    - show this help");
                println!("  \x1b[36mexit\x1b[0m    - exit shell");
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
            }
            "history" => {
                println!("History is persisted to .affi_history.");
                println!("Use Up/Down arrows to navigate, Ctrl-R to search.");
            }
            "receipt" => {
                if args.len() < 2 {
                    return Err(anyhow!("Missing subcommand for 'receipt'"));
                }
                match args[1].as_str() {
                    "emit" => {
                        let mut payload = String::new();
                        let mut objects = Vec::new();
                        let mut r#type = String::new();

                        let mut i = 2;
                        while i < args.len() {
                            match args[i].as_str() {
                                "--type" | "-t" => {
                                    if i + 1 < args.len() {
                                        r#type = args[i + 1].clone();
                                        i += 2;
                                    } else {
                                        return Err(anyhow!("Missing value for --type"));
                                    }
                                }
                                _ => {
                                    if payload.is_empty() {
                                        payload = args[i].clone();
                                    } else {
                                        objects.push(args[i].clone());
                                    }
                                    i += 1;
                                }
                            }
                        }
                        if payload.is_empty() || r#type.is_empty() {
                            return Err(anyhow!(
                                "Usage: receipt emit <payload> <object...> --type <type>"
                            ));
                        }
                        tokio::task::spawn_blocking(move || {
                            handlers::emit(r#type, objects, payload, None)
                        })
                        .await?
                        .map_err(|e| anyhow!("{}", e))?;
                    }
                    "assemble" => {
                        let out = args.get(2).cloned();
                        tokio::task::spawn_blocking(move || handlers::assemble(None, out))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "verify" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt verify <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || {
                            handlers::verify(receipt, None, None, None)
                        })
                        .await?
                        .map_err(|e| anyhow!("{}", e))?;
                    }
                    "show" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt show <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::show(receipt, None))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "inspect" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt inspect <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::inspect(receipt, None))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "stats" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt stats <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::stats(receipt, None))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "graph" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt graph <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::graph(receipt, None))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "replay" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt replay <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::replay(receipt))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "model" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt model <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::model(receipt))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "conformance" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt conformance <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::conformance(receipt))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    "diagnose" => {
                        if args.len() < 3 {
                            return Err(anyhow!("Usage: receipt diagnose <receipt>"));
                        }
                        let receipt = args[2].clone();
                        tokio::task::spawn_blocking(move || handlers::diagnose(receipt))
                            .await?
                            .map_err(|e| anyhow!("{}", e))?;
                    }
                    _ => return Err(anyhow!("Unknown receipt subcommand: {}", args[1])),
                }
            }
            _ => return Err(anyhow!("Unknown command: {}", args[0])),
        }
        Ok(())
    }
}

#[cfg(feature = "shell")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shell_impl::run().await
}

#[cfg(not(feature = "shell"))]
fn main() {
    println!("The 'shell' feature is not enabled. Re-compile with --features shell.");
}
