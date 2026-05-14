#!/usr/bin/env node
// Build script: creates dist/raxclaw executable wrapper
const { writeFileSync, mkdirSync, chmodSync } = require("fs");
const { resolve } = require("path");

mkdirSync("dist", { recursive: true });

// Shell wrapper that uses the local tsx binary to run raxclaw.tsx
// This handles ink v7's ESM requirements without bundling complexity
const shebang = "#!/usr/bin/env bash";
const wrapper = `${shebang}
REPO_DIR="$(cd "$(dirname "\${BASH_SOURCE[0]}")/.." && pwd)"
exec "$REPO_DIR/node_modules/.bin/tsx" "$REPO_DIR/raxclaw.tsx" "$@"
`;

const outPath = resolve("dist", "raxclaw");
writeFileSync(outPath, wrapper, { encoding: "utf8" });
chmodSync(outPath, 0o755);

console.log("BUILT dist/raxclaw");

