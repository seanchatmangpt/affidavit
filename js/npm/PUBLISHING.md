# publishing

Operator runbook for the first npm release of `graphlaw` (calver `26.10.13`,
kept in lockstep with the root `Cargo.toml` version by
`scripts/sync-version.mjs`).

Doctrine (mirrors `publish-npm.yml` / `release.yml`, operator decision
2026-10-10): **CI never holds registry credentials.** No `NPM_TOKEN` secret
exists or may be added; publication authenticates via GitHub OIDC against npm
Trusted Publishing.

## One-time npm setup

1. **Claim the package name deliberately from CI.** The first successful
   publish creates and claims the name `graphlaw` on the npm registry. Until
   then nothing is reserved, so the first publish must be the deliberate,
   validated CI publish — not an ad hoc local `npm publish`.
2. **Configure the Trusted Publisher.** On npmjs.com, open the package page
   for `graphlaw` and set Trusted Publisher to:
   - repository: `seanchatmangpt/graphlaw`
   - workflow filename: `publish-npm.yml`
   - environment: `npm`
   The configuration is **not validated until first use** — a typo in any of
   these fields surfaces only as a failed OIDC exchange at publish time.
3. **Create the GitHub environment `npm`.** Repository Settings →
   Environments → `npm`. It holds **no secrets** — the publish job needs only
   `id-token: write` for OIDC. Optional reviewers/protection rules apply to
   every publish.
4. **Repository must be public.** Provenance attestation is only visible for
   public repositories; a private repo publishes without visible provenance.

## Release flow

1. Bump the version in the root `Cargo.toml` (`[package] version`).
2. Sync the npm package:
   `node scripts/sync-version.mjs` (updates `js/npm/package.json`; commit
   both files).
3. Commit, then tag `v<version>` (e.g. `v26.10.13`) and push the tag.
4. `release.yml` runs automatically: it creates the GitHub release for the
   version declared in `Cargo.toml` (tagging is automatic there too — the
   manual tag drives `publish-npm.yml`, which triggers on `v*` tags).
5. `publish-npm.yml` then: verifies the tag matches BOTH `js/npm/package.json`
   and root `Cargo.toml` (fails closed), rebuilds the wasm artifact and
   checks its sha256 against `priv/graphlaw.wasm.sha256`, installs/builds/
   tests the package, packs twice and diffs the file lists (tarball drift
   check), uploads the tarball, then the `publish` job re-verifies the
   sha256 and publishes with OIDC (`--tag latest`, `--access public`).
6. `RELEASE_HOLD` gate: while the repository variable `RELEASE_HOLD` is
   `true`, nothing is validated or published (both jobs' `if:` conditions).
   Set it to `false`/unset to release.

## Same-day re-release rule

npm calver versions are immutable once published. If a same-day fix is
needed after `26.10.13` is live, publish a prerelease-bumped version, e.g.
`26.10.13-1`, not a rewritten `26.10.13`. Note the tag check requires the
tag to match the package version, so the re-release gets its own tag
`v26.10.13-1` and its own `Cargo.toml` bump.

## Verifying

- `npm view graphlaw dist-tags` — `latest` should point at the new version.
- Provenance: the attestation is visible on the npmjs.com package page
  (Provenance section, linking the public repo and workflow run).
- `npm audit signatures` — verifies registry signatures.

## Local publish fallback

Only if OIDC is blocked. This path forges no CI credentials — it uses your
personal npm account, not any stored token:

1. `cd js/npm && npm login` (WebAuthn/hardware-key 2FA).
2. `npm publish --provenance` — the `--provenance` flag is for token-based
   local flows (CI omits it; OIDC provenance there is automatic).
3. Same immutability rules apply — a local publish claims the name just as
   permanently as CI would.
