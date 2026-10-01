// Hand-authored verb wrapper.
//! `receipt completions` verb — print shell completion script to stdout.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Print shell completion script to stdout (bash, zsh, or fish)
///
/// Usage:
///   eval "$(affi receipt completions bash)"
///   affi receipt completions zsh > ~/.zsh/completions/_affi
///   affi receipt completions fish > ~/.config/fish/completions/affi.fish
#[verb("completions", "receipt")]
pub fn completions(
    #[arg(index = 1)]
    shell: String,
) -> Result<()> {
    crate::handlers::completions(shell)
}
