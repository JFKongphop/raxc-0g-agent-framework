#!/usr/bin/env node
'use strict';
// Build script: bundles raxclaw.tsx → dist/raxclaw.mjs (ESM, all deps inlined)
//               dist/raxclaw        → thin sh entry  (no tsx needed at runtime)
//
// Why ESM?  Ink v7 + yoga-layout use top-level await — CJS bundling is not possible.
// Why .mjs? The dist/ folder has no package.json type:module, so .mjs forces ESM parsing.
const { build } = require('esbuild');
const { writeFileSync, mkdirSync, chmodSync } = require('fs');
const { resolve } = require('path');

mkdirSync('dist', { recursive: true });

async function main() {
  // ── Step 1: esbuild ESM bundle ───────────────────────────────────────────
  await build({
    entryPoints: ['raxclaw.tsx'],
    bundle: true,
    platform: 'node',
    format: 'esm',          // Must be ESM — ink v7 / yoga-layout use top-level await
    target: 'node18',
    outfile: 'dist/raxclaw.mjs',
    external: [
      // Node built-ins (always available, never bundle)
      'child_process', 'fs', 'path', 'url', 'os', 'stream', 'events',
      'http', 'https', 'net', 'tls', 'crypto', 'util', 'assert',
      'readline', 'tty', 'zlib', 'buffer', 'string_decoder',
    ],
    jsx: 'transform',
    jsxFactory: 'React.createElement',
    jsxFragment: 'React.Fragment',
    logLevel: 'info',
    // Inject a CJS-compatible require() shim so bundled packages that call
    // require('assert'), require('events'), etc. work inside the ESM output.
    banner: {
      js: `import { createRequire } from 'module';\nconst require = createRequire(import.meta.url);`,
    },
    // Stub out ink's optional devtools dep — not installed, not needed at runtime
    plugins: [{
      name: 'stub-react-devtools-core',
      setup(build) {
        build.onResolve({ filter: /^react-devtools-core$/ }, () => ({
          path: 'react-devtools-core',
          namespace: 'devtools-stub',
        }));
        build.onLoad({ filter: /.*/, namespace: 'devtools-stub' }, () => ({
          contents: 'export default null; export const connectToDevTools = () => {};',
          loader: 'js',
        }));
      },
    }],
  });

  // ── Step 2: dist/raxclaw — executable shell entry ────────────────────────
  // Resolves to the .mjs bundle next to itself; no tsx or node_modules needed.
  const entry = [
    '#!/usr/bin/env sh',
    'SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"',
    'exec node "$SCRIPT_DIR/raxclaw.mjs" "$@"',
  ].join('\n') + '\n';

  const outPath = resolve('dist', 'raxclaw');
  writeFileSync(outPath, entry, 'utf8');
  chmodSync(outPath, 0o755);

  console.log('\n✓  dist/raxclaw.mjs  (bundled ESM — all deps inlined)');
  console.log('✓  dist/raxclaw      (executable entry → node raxclaw.mjs)');
  console.log('\nTo also pre-compile the Rust cognition engine (faster startup):');
  console.log('  pnpm build:rust');
  console.log('  # or build everything at once:');
  console.log('  pnpm build:all');
  console.log('\nUsage:');
  console.log('  ./dist/raxclaw run');
  console.log('  ./dist/raxclaw run --file MyContract.sol');
  console.log('  ./dist/raxclaw list');
  console.log('  ./dist/raxclaw show <report>');
  console.log('  ./dist/raxclaw analyze MyContract.sol\n');
}

main().catch(e => { console.error(e); process.exit(1); });

