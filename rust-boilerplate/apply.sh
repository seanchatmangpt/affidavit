#!/usr/bin/env bash
# apply.sh — drop the house standardization layer into an existing Rust repo.
#
# Usage:
#   rust-boilerplate/apply.sh [TARGET_DIR] [--force] [--dry-run]
#
# Copies project-AGNOSTIC hygiene/config files from template/ into TARGET_DIR
# (default: current directory). Existing files are left untouched unless
# --force. Project-SPECIFIC files (Cargo.toml, README, CHANGELOG, src/) are
# never written; guidance for those is printed at the end.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE="$SCRIPT_DIR/template"

TARGET="."
FORCE=0
DRY=0
for arg in "$@"; do
  case "$arg" in
    --force)   FORCE=1 ;;
    --dry-run) DRY=1 ;;
    -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    -*)        echo "unknown flag: $arg" >&2; exit 2 ;;
    *)         TARGET="$arg" ;;
  esac
done

[[ -d "$TARGET"   ]] || { echo "error: target '$TARGET' is not a directory" >&2; exit 1; }
[[ -d "$TEMPLATE" ]] || { echo "error: template dir not found at $TEMPLATE" >&2; exit 1; }

# Project-agnostic files to drop in (relative paths preserved).
FILES=(
  rustfmt.toml
  deny.toml
  typos.toml
  .editorconfig
  rust-toolchain.toml
  SECURITY.md
  justfile
  LICENSE-MIT
  LICENSE-APACHE
  .github/workflows/ci.yml
  .github/workflows/release.yml
  .github/dependabot.yml
  .github/pull_request_template.md
)

# Source stubs — only written if the target src/ directory already exists
# (i.e. this is a Rust crate root) and the file is not yet present.
SRC_STUBS=(
  src/error.rs
  src/types.rs
)

copied=0; skipped=0
echo "Applying house boilerplate -> $TARGET"
for rel in "${FILES[@]}"; do
  src="$TEMPLATE/$rel"
  dst="$TARGET/$rel"
  [[ -f "$src" ]] || { echo "  ! missing in template: $rel"; continue; }
  if [[ -e "$dst" && $FORCE -eq 0 ]]; then
    echo "  skip (exists): $rel"; skipped=$((skipped + 1)); continue
  fi
  if [[ $DRY -eq 1 ]]; then
    echo "  would copy: $rel"; continue
  fi
  mkdir -p "$(dirname "$dst")"
  cp "$src" "$dst"
  echo "  copied: $rel"; copied=$((copied + 1))
done

if [[ -d "$TARGET/src" ]]; then
  echo
  echo "Applying src stubs -> $TARGET/src"
  for rel in "${SRC_STUBS[@]}"; do
    src="$TEMPLATE/$rel"
    dst="$TARGET/$rel"
    [[ -f "$src" ]] || { echo "  ! missing in template: $rel"; continue; }
    if [[ -e "$dst" && $FORCE -eq 0 ]]; then
      echo "  skip (exists): $rel"; skipped=$((skipped + 1)); continue
    fi
    if [[ $DRY -eq 1 ]]; then
      echo "  would copy: $rel"; continue
    fi
    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
    echo "  copied: $rel"; copied=$((copied + 1))
  done
fi

echo
echo "Done. copied=$copied skipped=$skipped  (use --force to overwrite, --dry-run to preview)"
cat <<'EOF'

Manual steps apply.sh will NOT do for you (project-specific):

  1. Cargo.toml metadata:
       license      = "MIT OR Apache-2.0"
       repository   = https://github.com/seanchatmangpt/<repo>   (fix any placeholder!)
       homepage     = same
       rust-version = "1.82"   and   edition = "2021"
       keywords     <= 5 (lowercase) ; categories <= 5 (valid crates.io ids)
  2. Lints: paste the [lints]/[workspace.lints] block from
     template/Cargo.toml (single) or template/Cargo.workspace.toml (workspace),
     then run:  cargo clippy --all-targets --all-features -- -D warnings
     and fix the fallout (expect todo!/unwrap warnings).
  3. CI assumes the default branch is main|master|develop — adjust ci.yml if not.
  4. Commit on a branch and open a PR using the new pull_request_template.
EOF
