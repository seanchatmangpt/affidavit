## Challenge Summary

**Overall risk assessment**: MEDIUM

## Challenges

### [Medium] Challenge 1: Markdown Subcommand Anchor Link Collisions in Nested Hierarchies

- Assumption challenged: The markdown generator assumes subcommand names are unique across the entire command hierarchy when generating table of contents anchor links.
- Attack scenario: A command structure contains duplicate subcommand names at different hierarchical paths (e.g., `root -> user -> get` and `root -> config -> get`). The table of contents lists both subcommands, but both links point to `#get`, ignoring the hierarchy.
- Blast radius: Users navigate to the wrong command section via TOC links. Some sections are unreachable from the TOC.
- Mitigation: Include parent hierarchy names in the anchor/slug generation (e.g., `#user-get` and `#config-get`), or suffix duplicates dynamically (e.g., `#get` and `#get-1`).

### [Medium] Challenge 2: Layout Box and Table Border Misalignment with Combining Characters

- Assumption challenged: The layout engine assumes that any character not matching CJK or Emoji ranges has a display width of exactly 1.
- Attack scenario: Cell/box content includes Unicode combining characters (such as diaeresis/umlauts `\u{0308}`, zero-width joiners `\u{200D}`, etc.). Each combining character is incorrectly counted as width 1 instead of 0.
- Blast radius: Visual boxes/tables are misaligned. The right borders shift left/right and create uneven margins.
- Mitigation: Use a standard Unicode character width measurement library (like `unicode-width`) instead of a manual range-based `char_display_width` function.

## Stress Test Results

- **Subcommands with spaces** -> `- [`sub command space`](#sub-command-space)` -> Generates correct slugified anchor. -> PASS
- **Subcommands with nested hierarchy** -> Correct header level scaling (`#`, `##`, `###`) but table of contents link ignores hierarchy -> PASS (formatting/nesting levels) / FAIL (link uniqueness)
- **Layout box CJK wrapping/padding** -> Correctly measures Hiragana, Katakana, Hangul, Fullwidth punctuation as width 2, matching terminal width -> PASS
- **Layout box Emoji wrapping/padding** -> Correctly measures Emojis in the `0x1F300..=0x1FAFF` range as width 2, matching terminal width -> PASS
- **Layout box Tab Expansion** -> Correctly expands tabs to next 4-space stop alignment -> PASS
- **Layout box with combining characters** -> `e\u{0308}` counted as width 2 instead of 1, resulting in shifted border alignment -> FAIL
- **Table formatting with newlines/multi-line cells** -> Cells align properly across multi-line heights -> PASS

## Unchallenged Areas

- **Completions, Manpage generators** — Out of scope of the visual and doc layout verification task.
