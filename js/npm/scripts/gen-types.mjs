// Generates src/generated/types.ts from registry/capability-registry.json
// (the CI-checked mirror of the Rust tables). Do not hand-edit the output;
// run `npm run gen` to regenerate.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, '..');
const repoRoot = resolve(pkgRoot, '..', '..');
const registryPath = join(repoRoot, 'registry', 'capability-registry.json');
const registry = JSON.parse(readFileSync(registryPath, 'utf8'));

// Registry type vocabulary -> TS:
//   string, integer, boolean, any, data_spec, json_or_string, string|null,
//   list<string>, list<object>, list<term>, list<list<string>>,
//   list<model:X>, model:X, enum:X, list<enum:X>
function tsType(t) {
  if (t == null) return 'unknown';
  if (t.endsWith('|null')) return tsType(t.slice(0, -5));
  switch (t) {
    case 'string':
      return 'string';
    case 'integer':
      return 'number';
    case 'boolean':
      return 'boolean';
    case 'any':
      return 'unknown';
    case 'data_spec':
      return 'DataSpec';
    case 'json_or_string':
      return 'string | Record<string, unknown>';
  }
  if (t.startsWith('list<enum:')) return `${t.slice(10, -1)}[]`;
  if (t.startsWith('list<model:')) return t.slice(11, -1);
  if (t.startsWith('list<term>')) return 'Term[]';
  if (t.startsWith('list<list<string>>')) return 'string[][]';
  if (t.startsWith('list<list<term>>')) return '(Term | null)[][]';
  if (t.startsWith('list<object>')) return 'Record<string, unknown>[]';
  if (t.startsWith('list<string>')) return 'string[]';
  if (t.startsWith('enum:')) return t.slice(5);
  if (t.startsWith('model:')) return t.slice(6);
  return 'unknown';
}

// Hand-maintained structured-shape overrides (documented in the ABI contract
// report; the registry only says `object` / `list<object>` for these).
// Key: `${op}.${request|response}.${fieldName}` -> TS type.
const FIELD_OVERRIDES = {
  'law.request.signed_lease': 'SignedLease',
  'law.request.lease': 'Lease',
  'law.response.receipts': 'Receipt[]',
  'shacl.response.results': 'ShaclResult[]',
  'shex.response.entries': 'ShexEntry[]',
  'hooks.response.firings': 'HookFiring[]',
  'datalog.request.rules': 'DatalogRule[]',
};
// Interfaces emitted verbatim into the generated file for the override shapes.
const SHAPE_INTERFACES = `/**
 * One SHACL validation result (shacl response \`results\`).
 */
export interface ShaclResult {
  /** Focus node of the violation. */
  focus: string;
  /** Property path (IRI) the violation was found on. */
  path: string;
  /** offending value. */
  value: unknown;
  /** Severity IRI (e.g. sh:Violation). */
  severity: string;
  /** Constraint component IRI (e.g. sh:MinCountConstraintComponent). */
  component: string;
  /** Shape IRI that produced the result. */
  shape: string;
  /** Human-readable message, when present. */
  message?: string;
}

/**
 * One ShEx validation entry (shex response \`entries\`).
 */
export interface ShexEntry {
  /** Node that was validated. */
  node: Term;
  /** Shape IRI it was validated against. */
  shape: string;
  /** Validation status. */
  status: 'conformant' | 'nonconformant' | string;
  /** Failure reason, when nonconformant. */
  reason?: string;
}

/**
 * One hook firing within a round (hooks response \`firings\`).
 */
export interface HookFiring {
  /** Hook name. */
  hook: string;
  /** Round in which it fired. */
  round: number;
  /** Quads added by the hook. */
  added: number;
  /** Resulting row. */
  row: unknown;
}

/**
 * One Datalog rule (datalog request \`rules\`).
 */
export interface DatalogRule {
  /** Rule head as a [s, p, o] triple. */
  head: [string, string, string];
  /** Rule body as a list of [s, p, o] triples. */
  body: [string, string, string][];
}
`;

const out = [];
function line(s = '') {
  out.push(s);
}

line('// GENERATED from registry/capability-registry.json');
line(`// registry_sha256: ${registry.registry_sha256}`);
line('// Do not edit by hand; run `npm run gen` to regenerate.');
line();
line("import type { DataSpec, Term } from '../types.js';");
line();
line('// Structured response/request shapes documented in the ABI contract report');
line('// (the registry types these as bare object / list<object>).');
line(SHAPE_INTERFACES);

// Models (Lease, SignedLease, Receipt, Attestation, Plan, Action, PolicyEntry,
// PolicyOutcome) — interface per model, fields in registry order.
const models = [...registry.models].sort((a, b) => a.order - b.order);
for (const m of models) {
  line(`/** ${m.doc} */`);
  line(`export interface ${m.name} {`);
  for (const f of [...m.fields].sort((a, b) => a.order - b.order)) {
    const opt = f.required ? '' : '?';
    const nul = f.nullable ? ' | null' : '';
    if (f.doc) line(`  /** ${f.doc} */`);
    line(`  ${f.name}${opt}: ${tsType(f.type)}${nul};`);
  }
  line('}');
  line();
}

// Enums (Ceiling, LeaseReason, ReceiptReason, PolicyRefusalKind).
for (const e of registry.model_enums) {
  line(`/** Registry enum ${e.name}. */`);
  line(`export type ${e.name} = ${e.values.map((v) => JSON.stringify(v)).join(' | ')};`);
  line(`export const ${e.name}_VALUES = [${e.values.map((v) => JSON.stringify(v)).join(', ')}] as const;`);
  line();
}

// Closed unions from registry lists.
line('/** Refusal kinds (error.kind). */');
line(`export type RefusalKind = ${registry.refusal_kinds.map((v) => JSON.stringify(v)).join(' | ')};`);
line(`export const REFUSAL_KIND_VALUES = [${registry.refusal_kinds.map((v) => JSON.stringify(v)).join(', ')}] as const;`);
line();
line('/** Refusal detail codes (error.details.code). */');
line(`export type RefusalCode = ${registry.refusal_codes.map((c) => JSON.stringify(c.code)).join(' | ')};`);
line();
line('/** Backend engines. */');
line(`export type Engine = ${registry.engines.map((v) => JSON.stringify(v)).join(' | ')};`);
line();
// Dialect wire names (request side) and Rust variant response names.
const dialects = [...registry.rdf_dialects, ...registry.other_dialects].sort((a, b) => a.order - b.order);
line('/** Dialect wire names accepted in requests. */');
line(`export type DialectName = ${dialects.map((d) => JSON.stringify(d.name)).join(' | ')};`);
line('/** Dialect names as reported in responses (Rust variant names). */');
line(`export type DialectResponseName = ${dialects.map((d) => JSON.stringify(d.response_name)).join(' | ')};`);
line();
line('/** Entailment regimes. */');
line(`export type Regime = ${registry.regimes.map((v) => JSON.stringify(v)).join(' | ')};`);
line();

// Per-op request/response types.
function capitalize(s) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
function fieldDecl(f, inResponse, overrideKey) {
  const opt = inResponse || !f.required ? '?' : '';
  const nul = f.nullable ? ' | null' : '';
  const doc = f.doc ? `  /** ${f.doc} */\n` : '';
  const type = FIELD_OVERRIDES[overrideKey] ?? tsType(f.type);
  return `${doc}  ${f.name}${opt}: ${type}${nul};`;
}
function renderFields(fields, inResponse, opName) {
  const body = [...fields]
    .sort((a, b) => a.order - b.order)
    .map((f) => fieldDecl(f, inResponse, `${opName}.${inResponse ? 'response' : 'request'}.${f.name}`))
    .join('\n');
  return body;
}
for (const op of [...registry.ops].sort((a, b) => a.order - b.order)) {
  const Name = capitalize(op.name);
  line(`/** ${op.summary} */`);
  line(`export interface ${Name}Request {`);
  // law op: lease fields are handled via the shared models already; registry
  // types them as model:Lease / model:SignedLease directly.
  line(renderFields(op.request.fields, false, op.name));
  line('}');
  line();
  const responses = op.responses;
  if (responses.length === 1) {
    line(`/** Success response for ${op.name}. */`);
    line(`export interface ${Name}Response {`);
    line(renderFields(responses[0].fields, true, op.name));
    line('}');
  } else {
    line(`/** Success response union for ${op.name}, discriminated by ${responses[0].tag_field}. */`);
    line(`export type ${Name}Response =`);
    const variants = responses.map((r) => {
      const variantName = `${Name}${capitalize(r.tag)}`;
      return `  | (${variantName})`;
    });
    line(variants.join('\n'));
    line(';');
    for (const r of responses) {
      const variantName = `${Name}${capitalize(r.tag)}`;
      line(`/** Success response for ${op.name} (${r.tag}). */`);
      line(`export interface ${variantName} {`);
      line(`  ${r.tag_field}: ${JSON.stringify(r.tag)};`);
      line(renderFields(r.fields, true, op.name));
      line('}');
      line();
    }
  }
  line();
}

mkdirSync(join(pkgRoot, 'src', 'generated'), { recursive: true });
writeFileSync(join(pkgRoot, 'src', 'generated', 'types.ts'), out.join('\n') + '\n');
console.log('generated src/generated/types.ts from', registryPath);
