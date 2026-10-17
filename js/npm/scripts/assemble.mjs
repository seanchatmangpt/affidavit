import { cpSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, '..');

const tscOut = join(pkgRoot, 'tsc-out');
if (!existsSync(join(tscOut, 'index.js'))) {
  console.error('assemble: tsc-out/index.js missing — run `tsc -p tsconfig.build.json` first');
  process.exit(1);
}

const dist = join(pkgRoot, 'dist-nodejs');
mkdirSync(dist, { recursive: true });
cpSync(tscOut, dist, { recursive: true });

mkdirSync(join(dist, 'raw'), { recursive: true });
const js = `export { callRaw, callRawJson } from '../raw.js';\n`;
writeFileSync(join(dist, 'raw', 'index.js'), js);
const dts = `export { callRaw, callRawJson } from '../raw.js';\nexport type { AbiRefused, AbiResult } from '../types.js';\n`;
writeFileSync(join(dist, 'raw', 'index.d.ts'), dts);

console.log('assemble: dist-nodejs ready');

const runtimeWebReexport = `export * from './runtime.web.js';\n`;
const runtimeWebReexportDts = `export * from './runtime.web.js';\n`;

function ensureWebRuntime(target) {
  if (existsSync(join(target, 'runtime.web.js'))) return;

  const src = join(pkgRoot, 'src', 'runtime.web.ts');
  if (!existsSync(src)) {
    console.error(`assemble: ${src} missing — cannot assemble web runtime entry`);
    process.exit(1);
  }

  const tmp = mkdtempSync(join(tmpdir(), 'affidavit-runtime-web-'));
  try {
    execFileSync(
      'npx',
      [
        'tsc', src,
        '--outDir', tmp,
        '--target', 'ES2022',
        '--module', 'NodeNext',
        '--moduleResolution', 'NodeNext',
        '--strict',
        '--skipLibCheck',
      ],
      { cwd: pkgRoot, stdio: 'inherit' },
    );
  } finally {
    const outJs = join(tmp, 'runtime.web.js');
    if (existsSync(outJs)) {
      for (const f of ['runtime.web.js', 'runtime.web.d.ts']) {
        if (existsSync(join(tmp, f))) cpSync(join(tmp, f), join(target, f));
      }
    }
    rmSync(tmp, { recursive: true, force: true });
  }
}

for (const name of ['dist-web', 'dist-bundler']) {
  const target = join(pkgRoot, name);
  mkdirSync(target, { recursive: true });
  cpSync(tscOut, target, { recursive: true });

  ensureWebRuntime(target);

  writeFileSync(join(target, 'runtime.js'), runtimeWebReexport);
  writeFileSync(join(target, 'runtime.d.ts'), runtimeWebReexportDts);

  mkdirSync(join(target, 'raw'), { recursive: true });
  writeFileSync(join(target, 'raw', 'index.js'), js);
  writeFileSync(join(target, 'raw', 'index.d.ts'), dts);

  console.log(`assemble: ${name} ready (runtime -> runtime.web)`);
}
