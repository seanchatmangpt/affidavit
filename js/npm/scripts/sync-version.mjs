// Syncs the npm package version from the root Cargo.toml (single source of
// truth, calendar versioning). Fails loudly on drift instead of guessing.
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, '..');
const repoRoot = resolve(pkgRoot, '..', '..');

const cargo = readFileSync(join(repoRoot, 'Cargo.toml'), 'utf8');
const match = /^version\s*=\s*"([^"]+)"/m.exec(cargo);
if (!match) {
  console.error('sync-version: could not find `version` in root Cargo.toml');
  process.exit(1);
}
const version = match[1];

const pkgPath = join(pkgRoot, 'package.json');
const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
if (pkg.version !== version) {
  pkg.version = version;
  writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`sync-version: package.json -> ${version}`);
} else {
  console.log(`sync-version: already at ${version}`);
}
