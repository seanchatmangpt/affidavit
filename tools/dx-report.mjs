#!/usr/bin/env node
// =============================================================================
// tools/dx-report.mjs
//
// DX / repo-health metrics for the `affidavit` repository.
//
// A dependency-free Node ESM script (built-ins only: node:fs, node:path,
// node:url). It walks the repository tree from the repo root, computes REAL
// metrics (Rust, Web, Docs, Tooling, semconv), writes a Markdown report to
// `DX_REPORT.md` at the repo root, and prints a short summary to stdout.
//
// DOCTRINE / NO FABRICATION: every number emitted is mined from the actual
// tree at run time. Nothing is hardcoded or guessed. If a metric cannot be
// measured it is reported as "n/a".
//
// Usage:
//   node tools/dx-report.mjs            Scan the tree, write DX_REPORT.md,
//                                       print a summary.
//   node tools/dx-report.mjs --help     Show this usage and exit.
//   node tools/dx-report.mjs --stdout   Print the full report to stdout too.
//
// The repo root is resolved relative to this script's location (../ from
// tools/), so the tool can be run from any working directory.
//
// Author: A10 (DX/QOL push). Output is machine-generated; do not edit
// DX_REPORT.md by hand.
// =============================================================================

import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

// ---------------------------------------------------------------------------
// Constants & root resolution
// ---------------------------------------------------------------------------

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
// Script lives in <repo>/tools, so the repo root is one level up.
const REPO_ROOT = path.resolve(__dirname, "..");

// Directories we never descend into while walking the tree.
const IGNORE_DIRS = new Set([
  ".git",
  "node_modules",
  "target",
  ".next",
  "dist",
  "build",
]);

const OUTPUT_FILE = path.join(REPO_ROOT, "DX_REPORT.md");

// ---------------------------------------------------------------------------
// CLI argument handling
// ---------------------------------------------------------------------------

const args = process.argv.slice(2);
if (args.includes("--help") || args.includes("-h")) {
  printUsage();
  process.exit(0);
}
const ALSO_STDOUT = args.includes("--stdout");

function printUsage() {
  process.stdout.write(
    [
      "tools/dx-report.mjs — DX / repo-health metrics for affidavit",
      "",
      "Usage:",
      "  node tools/dx-report.mjs            Scan tree, write DX_REPORT.md, print summary",
      "  node tools/dx-report.mjs --help     Show this help and exit",
      "  node tools/dx-report.mjs --stdout   Also print the full report to stdout",
      "",
      "All metrics are mined from the live repository tree at run time.",
      "No values are hardcoded; unmeasurable metrics are reported as n/a.",
      "",
    ].join("\n") + "\n",
  );
}

// ---------------------------------------------------------------------------
// Robust filesystem helpers (every fs touch is guarded)
// ---------------------------------------------------------------------------

/** Safe lstat; returns null on any error. */
function safeLstat(p) {
  try {
    return fs.lstatSync(p);
  } catch {
    return null;
  }
}

/** Does a path exist (file, dir, or symlink)? */
function exists(p) {
  return safeLstat(p) !== null;
}

/** Read a directory's entries (Dirent[]); returns [] on any error. */
function safeReadDir(p) {
  try {
    return fs.readdirSync(p, { withFileTypes: true });
  } catch {
    return [];
  }
}

/** Read a file as UTF-8 text; returns null on any error. */
function safeReadText(p) {
  try {
    return fs.readFileSync(p, "utf8");
  } catch {
    return null;
  }
}

/** Count lines in a text blob (number of newline-separated segments). */
function countLines(text) {
  if (text == null || text.length === 0) return 0;
  // Normalise CRLF, then count segments. A trailing newline does not add an
  // extra empty line to the count.
  const normalised = text.replace(/\r\n/g, "\n");
  const n = normalised.split("\n").length;
  return normalised.endsWith("\n") ? n - 1 : n;
}

// ---------------------------------------------------------------------------
// Tree walker
// ---------------------------------------------------------------------------

/**
 * Recursively walk `dir`, invoking `onFile(absPath, relPath)` for every
 * regular file encountered. Ignores IGNORE_DIRS. Does not follow directory
 * symlinks (avoids cycles). Fully guarded so one bad entry never aborts the
 * walk.
 */
function walk(dir, onFile) {
  const stack = [dir];
  while (stack.length > 0) {
    const current = stack.pop();
    const entries = safeReadDir(current);
    for (const ent of entries) {
      let name;
      try {
        name = ent.name;
      } catch {
        continue;
      }
      const abs = path.join(current, name);
      let isDir = false;
      let isFile = false;
      try {
        // Dirent methods are cheap and avoid an extra stat for most entries.
        isDir = ent.isDirectory();
        isFile = ent.isFile();
        // Resolve symlinks one level to classify them.
        if (ent.isSymbolicLink()) {
          const st = safeLstat(abs);
          // Treat symlinked dirs as non-traversable to avoid cycles; classify
          // symlinked files via a real stat.
          try {
            const real = fs.statSync(abs);
            isFile = real.isFile();
            isDir = false; // never descend through symlinked dirs
          } catch {
            isFile = false;
            isDir = false;
          }
          void st;
        }
      } catch {
        continue;
      }

      if (isDir) {
        if (IGNORE_DIRS.has(name)) continue;
        stack.push(abs);
      } else if (isFile) {
        const rel = path.relative(REPO_ROOT, abs);
        try {
          onFile(abs, rel);
        } catch {
          // Never let a single file's handler kill the whole walk.
        }
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Metric collection
// ---------------------------------------------------------------------------

// Accumulators populated during a single full-tree walk.
const acc = {
  rustFiles: 0,
  rustLoc: 0,
  testAttrs: 0, // #[test]
  tokioTestAttrs: 0, // #[tokio::test]
  webTsLoc: 0,
  topLevelMdFiles: 0, // *.md directly in repo root
  topLevelMdLines: 0,
};

// Convert a relative path to forward slashes for stable matching on any OS.
function toPosix(rel) {
  return rel.split(path.sep).join("/");
}

walk(REPO_ROOT, (abs, rel) => {
  const p = toPosix(rel);
  const lower = p.toLowerCase();

  // --- Rust source files (anywhere in the tree) ---
  if (lower.endsWith(".rs")) {
    acc.rustFiles += 1;
    const text = safeReadText(abs);
    if (text != null) {
      acc.rustLoc += countLines(text);
      // Count attribute occurrences. These are proxies for unit/async tests.
      // Match #[test] and #[tokio::test], tolerating surrounding whitespace.
      const tMatches = text.match(/#\[\s*test\s*\]/g);
      if (tMatches) acc.testAttrs += tMatches.length;
      const tkMatches = text.match(/#\[\s*tokio::test[^\]]*\]/g);
      if (tkMatches) acc.tokioTestAttrs += tkMatches.length;
    }
  }

  // --- Web TypeScript LOC (web/**/*.ts,tsx; node_modules/.next already
  //     excluded by the walker). Skip TS declaration build artifacts that are
  //     generated, like next-env.d.ts, to avoid noise? Keep .d.ts honest:
  //     count real source only by including .ts/.tsx but the walker already
  //     skips .next; node_modules is skipped too. ---
  if (p.startsWith("web/") && (lower.endsWith(".ts") || lower.endsWith(".tsx"))) {
    const text = safeReadText(abs);
    if (text != null) acc.webTsLoc += countLines(text);
  }

  // --- Top-level Markdown files (directly under repo root only) ---
  if (lower.endsWith(".md") && !p.includes("/")) {
    acc.topLevelMdFiles += 1;
    const text = safeReadText(abs);
    if (text != null) acc.topLevelMdLines += countLines(text);
  }
});

// --- Targeted directory counts (glob-style, via direct dir reads) ---

/** Count files in `dirRel` (relative to root) whose name matches `re`. */
function countFilesMatching(dirRel, re) {
  const dirAbs = path.join(REPO_ROOT, dirRel);
  if (!exists(dirAbs)) return 0;
  let n = 0;
  for (const ent of safeReadDir(dirAbs)) {
    try {
      if (ent.isFile() && re.test(ent.name)) n += 1;
    } catch {
      /* skip */
    }
  }
  return n;
}

/** Count files matched by a recursive walk of `dirRel` against a predicate. */
function countFilesRecursive(dirRel, predicate) {
  const dirAbs = path.join(REPO_ROOT, dirRel);
  if (!exists(dirAbs)) return 0;
  let n = 0;
  walk(dirAbs, (abs, rel) => {
    if (predicate(toPosix(rel), abs)) n += 1;
  });
  return n;
}

// Rust structure
const cliVerbs = countFilesMatching("src/verbs", /\.rs$/);
const examples = countFilesMatching("examples", /\.rs$/);
const integrationTests = countFilesMatching("tests", /\.rs$/);
const compileFailFixtures = countFilesMatching("tests/ui/compile_fail", /\.rs$/);

// Web routes
const webPages = countFilesRecursive("web/app", (rel) => rel.endsWith("/page.tsx") || rel === "web/app/page.tsx");
const webApiRoutes = countFilesRecursive(
  "web/app/api",
  (rel) => rel.endsWith("/route.ts"),
);

// Docs: coverage green count from reference/COVERAGE.md
function countCoverageGreen() {
  const text = safeReadText(path.join(REPO_ROOT, "reference/COVERAGE.md"));
  if (text == null) return null; // n/a — file absent/unreadable
  const m = text.match(/🟢/gu);
  return m ? m.length : 0;
}
const coverageGreen = countCoverageGreen();

// Tooling presence/counts
const workflowCount = countFilesMatching(".github/workflows", /\.ya?ml$/);
const hasJustfile = exists(path.join(REPO_ROOT, "justfile")) || exists(path.join(REPO_ROOT, "Justfile"));
const hasDevcontainer = exists(path.join(REPO_ROOT, ".devcontainer"));
const scriptsCount = (() => {
  const d = path.join(REPO_ROOT, "scripts");
  if (!exists(d)) return null; // n/a — directory absent
  let n = 0;
  for (const ent of safeReadDir(d)) {
    try {
      if (ent.isFile()) n += 1;
    } catch {
      /* skip */
    }
  }
  return n;
})();
const completionsCount = (() => {
  const d = path.join(REPO_ROOT, "completions");
  if (!exists(d)) return null; // n/a — directory absent
  let n = 0;
  for (const ent of safeReadDir(d)) {
    try {
      if (ent.isFile()) n += 1;
    } catch {
      /* skip */
    }
  }
  return n;
})();

// semconv YAML files (recursive — registry + any siblings)
const semconvYaml = countFilesRecursive(
  "semconv",
  (rel) => rel.endsWith(".yaml") || rel.endsWith(".yml"),
);

// Repo description from Cargo.toml
function readCargoDescription() {
  const text = safeReadText(path.join(REPO_ROOT, "Cargo.toml"));
  if (text == null) return null;
  // Match the first top-level `description = "..."` line.
  const m = text.match(/^\s*description\s*=\s*"((?:[^"\\]|\\.)*)"/m);
  if (!m) return null;
  // Unescape simple backslash escapes.
  return m[1].replace(/\\(["\\])/g, "$1");
}
const cargoDescription = readCargoDescription();

const generatedAt = new Date().toISOString();

// ---------------------------------------------------------------------------
// Report rendering
// ---------------------------------------------------------------------------

/** Render a value for the table, mapping null -> "n/a". */
function v(x) {
  if (x === null || x === undefined) return "n/a";
  if (typeof x === "boolean") return x ? "yes" : "no";
  return String(x);
}

const sections = [
  {
    title: "Rust",
    rows: [
      ["`.rs` files (tree-wide)", v(acc.rustFiles)],
      ["Total Rust LOC", v(acc.rustLoc)],
      ["`#[test]` occurrences", v(acc.testAttrs)],
      ["`#[tokio::test]` occurrences", v(acc.tokioTestAttrs)],
      ["CLI verbs (`src/verbs/*.rs`)", v(cliVerbs)],
      ["Examples (`examples/*.rs`)", v(examples)],
      ["Integration test files (`tests/*.rs`)", v(integrationTests)],
      ["Compile-fail UI fixtures (`tests/ui/compile_fail/*.rs`)", v(compileFailFixtures)],
    ],
  },
  {
    title: "Web",
    rows: [
      ["Next.js routes (`web/app/**/page.tsx`)", v(webPages)],
      ["API routes (`web/app/api/**/route.ts`)", v(webApiRoutes)],
      ["Web TS LOC (`web/**/*.{ts,tsx}`, excl. node_modules/.next)", v(acc.webTsLoc)],
    ],
  },
  {
    title: "Docs",
    rows: [
      ["Top-level `*.md` files", v(acc.topLevelMdFiles)],
      ["Total top-level markdown lines", v(acc.topLevelMdLines)],
      ["Coverage 🟢 count (`reference/COVERAGE.md`)", v(coverageGreen)],
    ],
  },
  {
    title: "Tooling",
    rows: [
      ["GitHub workflows (`.github/workflows/*.yml`)", v(workflowCount)],
      ["`justfile` present", v(hasJustfile)],
      ["`.devcontainer` present", v(hasDevcontainer)],
      ["Files under `scripts/`", v(scriptsCount)],
      ["Files under `completions/`", v(completionsCount)],
    ],
  },
  {
    title: "semconv",
    rows: [["YAML files under `semconv/`", v(semconvYaml)]],
  },
];

function renderMarkdown() {
  const out = [];
  out.push("# DX / Repo Health Report");
  out.push("");
  out.push(`_Generated at ${generatedAt} by \`tools/dx-report.mjs\` (machine-generated — do not edit by hand)._`);
  out.push("");
  if (cargoDescription) {
    out.push(`**Repo:** ${cargoDescription}`);
    out.push("");
  }
  for (const sec of sections) {
    out.push(`## ${sec.title}`);
    out.push("");
    out.push("| Metric | Value |");
    out.push("| --- | --- |");
    for (const [metric, value] of sec.rows) {
      out.push(`| ${metric} | ${value} |`);
    }
    out.push("");
  }
  out.push("---");
  out.push("");
  out.push("### How this was measured");
  out.push("");
  out.push(
    "All values above are mined from the live repository tree at run time by " +
      "`tools/dx-report.mjs`, a dependency-free Node ESM script (built-ins only). " +
      "The walker starts at the repo root, skips `.git`, `node_modules`, `target`, " +
      "`.next`, `dist`, and `build`, and never follows directory symlinks. " +
      "No metric is hardcoded or read from a fixture; anything that could not be " +
      "measured is shown as `n/a`. Counts of `#[test]` / `#[tokio::test]` are " +
      "textual proxies for unit/async tests, not a compiler-verified test count.",
  );
  out.push("");
  return out.join("\n");
}

const markdown = renderMarkdown();

// ---------------------------------------------------------------------------
// Emit
// ---------------------------------------------------------------------------

let wrote = false;
try {
  fs.writeFileSync(OUTPUT_FILE, markdown, "utf8");
  wrote = true;
} catch (err) {
  process.stderr.write(`dx-report: failed to write ${OUTPUT_FILE}: ${err && err.message}\n`);
}

// Concise stdout summary (always printed).
const summary = [
  "DX report" + (wrote ? ` written to ${path.relative(REPO_ROOT, OUTPUT_FILE)}` : " (write FAILED)"),
  `  repo root        : ${REPO_ROOT}`,
  `  generated at      : ${generatedAt}`,
  `  Rust .rs files    : ${v(acc.rustFiles)}   (LOC ${v(acc.rustLoc)})`,
  `  #[test]/tokio     : ${v(acc.testAttrs)} / ${v(acc.tokioTestAttrs)}`,
  `  CLI verbs         : ${v(cliVerbs)}`,
  `  examples/intg test: ${v(examples)} / ${v(integrationTests)}  (compile-fail ${v(compileFailFixtures)})`,
  `  web pages/api     : ${v(webPages)} / ${v(webApiRoutes)}   (web TS LOC ${v(acc.webTsLoc)})`,
  `  top-level md      : ${v(acc.topLevelMdFiles)} files / ${v(acc.topLevelMdLines)} lines`,
  `  coverage 🟢       : ${v(coverageGreen)}`,
  `  workflows         : ${v(workflowCount)}   justfile=${v(hasJustfile)} devcontainer=${v(hasDevcontainer)} scripts=${v(scriptsCount)} completions=${v(completionsCount)}`,
  `  semconv yaml      : ${v(semconvYaml)}`,
].join("\n");

process.stdout.write(summary + "\n");

if (ALSO_STDOUT) {
  process.stdout.write("\n" + markdown + "\n");
}

process.exit(wrote ? 0 : 1);
