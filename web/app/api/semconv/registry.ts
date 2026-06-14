// Faithful reader for the affidavit OTel Weaver semantic-convention registry.
//
// Every value surfaced here is parsed from the REAL YAML under semconv/registry/
// at request time. There are NO fixtures. If the registry is absent or a group
// cannot be parsed, callers receive an explicit { available: false } marker so the
// UI can show an honest "unavailable" state rather than fabricate span/attribute
// data. See STATUS.md ("Honest OTel split"): the semconv registry surface is CLOSED
// (validated by real `weaver registry check`), while SDK export to a live collector
// remains OPEN-substrate.
//
// We carry no YAML dependency, so this is a small, structural hand parser scoped to
// this registry's shape: 2-space-indented `key: value`, nested mappings, and `- id:`
// list items (the `attributes:` and enum `members:` blocks). It is deliberately
// narrow — it understands the fields we surface, not arbitrary YAML.

import { readFile } from "node:fs/promises";
import path from "node:path";

// Repo root resolution mirrors web/lib/affidavit.ts (parent of web/, overridable).
const REPO_ROOT = process.env.AFFIDAVIT_ROOT
  ? path.resolve(process.env.AFFIDAVIT_ROOT)
  : path.resolve(process.cwd(), "..");

// Candidate group YAML files inside the registry (filename(s) used by this repo).
const REGISTRY_DIR = path.join("semconv", "registry");
const GROUP_FILE = "affidavit.yaml";
const MANIFEST_FILE = "manifest.yaml";

/** One enum member under an attribute whose type is `members:`. */
export interface EnumMember {
  id: string;
  value: string;
  brief?: string;
}

/** One attribute pinned by a span group. */
export interface SemconvAttribute {
  id: string;
  /** Resolved type: a primitive name (e.g. "string") or "enum" when members present. */
  type: string;
  brief?: string;
  requirementLevel?: string;
  /** Present only when the attribute is an enum (type === "enum"). */
  members?: EnumMember[];
}

/** A span group from the registry. */
export interface SemconvGroup {
  id: string;
  type?: string;
  spanKind?: string;
  stability?: string;
  brief?: string;
  attributes: SemconvAttribute[];
}

export interface SemconvRegistry {
  available: true;
  /** Registry name + semconv version, from manifest.yaml (best-effort). */
  name?: string;
  semconvVersion?: string;
  groups: SemconvGroup[];
  /** Repo-relative source paths actually read. */
  sources: string[];
}

export interface SemconvUnavailable {
  available: false;
  message: string;
  /** The repo-relative path we looked for (for an honest source line). */
  expectedSource: string;
}

export type SemconvResult = SemconvRegistry | SemconvUnavailable;

// ── small structural YAML helpers ──────────────────────────────────────────────

/** Number of leading spaces (indent depth) on a line. */
function indentOf(line: string): number {
  let n = 0;
  while (n < line.length && line[n] === " ") n++;
  return n;
}

/** Strip a trailing `# comment` that is not inside quotes; trim. */
function stripComment(s: string): string {
  let inSingle = false;
  let inDouble = false;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (c === "'" && !inDouble) inSingle = !inSingle;
    else if (c === '"' && !inSingle) inDouble = !inDouble;
    else if (c === "#" && !inSingle && !inDouble) {
      // A '#' only begins a comment when preceded by whitespace or at line start.
      if (i === 0 || s[i - 1] === " " || s[i - 1] === "\t") return s.slice(0, i).trimEnd();
    }
  }
  return s.trimEnd();
}

/** Unwrap a scalar: drop surrounding quotes, decode the few escapes we expect. */
function unquote(raw: string): string {
  const v = raw.trim();
  if (v.length >= 2 && v[0] === '"' && v[v.length - 1] === '"') {
    return v.slice(1, -1).replace(/\\"/g, '"').replace(/\\n/g, "\n").replace(/\\\\/g, "\\");
  }
  if (v.length >= 2 && v[0] === "'" && v[v.length - 1] === "'") {
    return v.slice(1, -1).replace(/''/g, "'");
  }
  return v;
}

/**
 * Logical line: indent + key + inline value, with list-item dashes recognized.
 * For a `- key: value` item the structural `indent` is the column of the `-`
 * itself (NOT the key text two columns right): in YAML a sequence item sits at
 * the same indent as the mapping key that introduces the list, and its sibling
 * scalar keys are nested one level (+2) under the dash. Keeping `indent` at the
 * dash column is what lets the `>` / `===` comparisons in the parsers line a
 * list element up against its own children and its parent key correctly. The
 * `isItem` flag tells the caller a new list element starts here; the inline
 * `key`/`value` are parsed from the text after the dash and do not depend on
 * the column the key text begins in.
 */
interface LLine {
  indent: number;
  key: string | null; // null for a bare scalar / non key:value line
  value: string | null; // inline scalar value after the colon, if any
  isItem: boolean; // started with "- "
  raw: string;
}

function lex(yaml: string): LLine[] {
  const out: LLine[] = [];
  for (const rawLine of yaml.split(/\r?\n/)) {
    const noComment = stripComment(rawLine);
    if (noComment.trim() === "") continue;
    const baseIndent = indentOf(noComment);
    let rest = noComment.slice(baseIndent);
    let isItem = false;
    // Structural indent is the dash's own column; an item's child keys are the
    // ones that read as +2 deeper in the source.
    const indent = baseIndent;
    if (rest.startsWith("- ")) {
      isItem = true;
      rest = rest.slice(2);
    } else if (rest === "-") {
      isItem = true;
      rest = "";
    }
    // Split "key: value" — only on the FIRST colon followed by space or EOL.
    let key: string | null = null;
    let value: string | null = null;
    const m = rest.match(/^([^:\s][^:]*?):(?:\s+(.*))?$/);
    if (m) {
      key = m[1].trim();
      value = m[2] !== undefined ? m[2].trim() : null;
    } else if (rest.endsWith(":")) {
      key = rest.slice(0, -1).trim();
      value = null;
    } else {
      value = rest.trim();
    }
    out.push({ indent, key, value: value ?? null, isItem, raw: rawLine });
  }
  return out;
}

// ── registry parsing (group YAML) ──────────────────────────────────────────────

function parseAttributes(lines: LLine[], start: number, attrIndent: number): { attrs: SemconvAttribute[]; next: number } {
  const attrs: SemconvAttribute[] = [];
  let i = start;
  let cur: SemconvAttribute | null = null;
  // Track whether we are inside a `type:` block that holds enum `members:`.
  let inTypeBlock = false;
  let typeBlockIndent = -1;
  let inMembers = false;
  let membersIndent = -1;
  let curMember: EnumMember | null = null;

  const closeMember = () => {
    if (cur && curMember && curMember.id) {
      (cur.members ??= []).push(curMember);
    }
    curMember = null;
  };

  for (; i < lines.length; i++) {
    const l = lines[i];
    // Dedent below the attribute list → attributes block is finished.
    if (l.indent < attrIndent) break;

    if (l.indent === attrIndent && l.isItem) {
      // New attribute begins.
      closeMember();
      cur = { id: "", type: "" };
      attrs.push(cur);
      inTypeBlock = false;
      inMembers = false;
      typeBlockIndent = -1;
      membersIndent = -1;
      if (l.key === "id" && l.value !== null) cur.id = unquote(l.value);
      continue;
    }

    if (!cur) continue; // defensive: stray line before any attribute item

    // Inside an enum members list?
    if (inMembers && l.indent >= membersIndent) {
      if (l.indent === membersIndent && l.isItem) {
        closeMember();
        curMember = { id: "", value: "" };
        if (l.key === "id" && l.value !== null) curMember.id = unquote(l.value);
        continue;
      }
      if (curMember && l.key && l.indent > membersIndent - 2) {
        const v = l.value !== null ? unquote(l.value) : "";
        if (l.key === "id") curMember.id = v;
        else if (l.key === "value") curMember.value = v;
        else if (l.key === "brief") curMember.brief = v;
        continue;
      }
      continue;
    } else if (inMembers && l.indent < membersIndent) {
      closeMember();
      inMembers = false;
    }

    // Inside a `type:` mapping block (may contain `members:`)?
    if (inTypeBlock && l.indent > typeBlockIndent) {
      if (l.key === "members" && l.value === null) {
        cur.type = "enum";
        inMembers = true;
        // members items are indented under `members:` — capture their indent next.
        membersIndent = l.indent + 2;
      }
      continue;
    } else if (inTypeBlock && l.indent <= typeBlockIndent) {
      inTypeBlock = false;
    }

    // Plain key on the attribute itself.
    if (l.key) {
      if (l.key === "type") {
        if (l.value === null) {
          // Block form: either a nested mapping (enum members) — resolve later.
          inTypeBlock = true;
          typeBlockIndent = l.indent;
          if (cur.type === "") cur.type = "enum"; // refined to "enum" if members appear
        } else {
          cur.type = unquote(l.value);
        }
      } else if (l.key === "id") {
        cur.id = l.value !== null ? unquote(l.value) : cur.id;
      } else if (l.key === "brief") {
        cur.brief = l.value !== null ? unquote(l.value) : cur.brief;
      } else if (l.key === "requirement_level") {
        cur.requirementLevel = l.value !== null ? unquote(l.value) : cur.requirementLevel;
      }
    }
  }
  closeMember();
  return { attrs, next: i };
}

function parseGroups(yaml: string): SemconvGroup[] {
  const lines = lex(yaml);
  const groups: SemconvGroup[] = [];
  // Find the top-level `groups:` key (indent 0).
  let gi = lines.findIndex((l) => l.indent === 0 && l.key === "groups");
  if (gi < 0) return groups;
  gi += 1;
  // Group items sit one level under `groups:`; capture their item indent.
  let groupItemIndent = -1;
  let cur: SemconvGroup | null = null;

  for (let i = gi; i < lines.length; i++) {
    const l = lines[i];
    if (l.indent === 0) break; // back to another top-level key
    if (groupItemIndent === -1 && l.isItem) groupItemIndent = l.indent;

    if (l.indent === groupItemIndent && l.isItem) {
      cur = { id: "", attributes: [] };
      groups.push(cur);
      if (l.key === "id" && l.value !== null) cur.id = unquote(l.value);
      continue;
    }
    if (!cur) continue;

    if (l.key === "attributes" && l.value === null) {
      // Parse the attributes list; its items are indented under this key.
      const { attrs, next } = parseAttributes(lines, i + 1, l.indent + 2);
      cur.attributes = attrs;
      i = next - 1;
      continue;
    }
    if (l.indent > groupItemIndent && l.key) {
      const v = l.value !== null ? unquote(l.value) : "";
      if (l.key === "id") cur.id = v || cur.id;
      else if (l.key === "type") cur.type = v;
      else if (l.key === "span_kind") cur.spanKind = v;
      else if (l.key === "stability") cur.stability = v;
      else if (l.key === "brief") cur.brief = v;
    }
  }
  return groups;
}

function parseManifest(yaml: string): { name?: string; semconvVersion?: string } {
  const lines = lex(yaml);
  const out: { name?: string; semconvVersion?: string } = {};
  for (const l of lines) {
    if (l.indent !== 0 || !l.key || l.value === null) continue;
    if (l.key === "name") out.name = unquote(l.value);
    else if (l.key === "semconv_version") out.semconvVersion = unquote(l.value);
    // Stop early once both are found.
    if (out.name && out.semconvVersion) break;
  }
  return out;
}

// ── public entry point ──────────────────────────────────────────────────────────

/**
 * Read and parse the real semconv registry from the repo. Never throws: any
 * failure (missing dir, unreadable file, empty parse) resolves to an explicit
 * unavailable marker so the UI shows an honest state.
 */
export async function readSemconvRegistry(): Promise<SemconvResult> {
  const groupPath = path.join(REPO_ROOT, REGISTRY_DIR, GROUP_FILE);
  const manifestPath = path.join(REPO_ROOT, REGISTRY_DIR, MANIFEST_FILE);
  const groupSrc = path.posix.join("semconv", "registry", GROUP_FILE);

  let groupYaml: string;
  try {
    groupYaml = await readFile(groupPath, "utf8");
  } catch {
    return {
      available: false,
      message: `semconv registry not found — expected ${groupSrc} in the repo (run from the affidavit checkout, or set AFFIDAVIT_ROOT).`,
      expectedSource: groupSrc,
    };
  }

  let groups: SemconvGroup[];
  try {
    groups = parseGroups(groupYaml);
  } catch (e) {
    return {
      available: false,
      message: `failed to parse ${groupSrc}: ${e instanceof Error ? e.message : String(e)}`,
      expectedSource: groupSrc,
    };
  }

  if (groups.length === 0) {
    return {
      available: false,
      message: `no span groups found in ${groupSrc} — registry shape not recognized.`,
      expectedSource: groupSrc,
    };
  }

  const sources = [groupSrc];
  let manifest: { name?: string; semconvVersion?: string } = {};
  try {
    manifest = parseManifest(await readFile(manifestPath, "utf8"));
    sources.push(path.posix.join("semconv", "registry", MANIFEST_FILE));
  } catch {
    // manifest is optional context; absence is not fatal.
  }

  return {
    available: true,
    name: manifest.name,
    semconvVersion: manifest.semconvVersion,
    groups,
    sources,
  };
}
