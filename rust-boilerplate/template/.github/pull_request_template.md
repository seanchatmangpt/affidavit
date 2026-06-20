<!--
House convention: an unchecked box is more useful than a falsely checked one.
Check only what you actually verified. Leave the rest unchecked and explain.
-->

## What & why

<!-- One or two sentences: what does this change and why. -->

## What I verified

- [ ] `just ci` passes locally (fmt-check, clippy `-D warnings`, test, deny, typos)
- [ ] New/changed behavior is covered by tests
- [ ] Public API changes are documented (rustdoc + CHANGELOG entry)
- [ ] No new `unwrap`/`expect`/`panic` in library code paths

## What I did NOT verify

<!-- Be honest. e.g. "cross-compile targets", "the otel path", "Windows". -->

## Notes for reviewers

<!-- Anything subtle, follow-ups, or deliberate trade-offs. -->
