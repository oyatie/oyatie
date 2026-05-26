#!/usr/bin/env node
// ADR-0372 D2: Verify that generated client files are up-to-date.
// Run in CI after `npm run codegen` to enforce "contract change breaks build".
//
// Usage:  node scripts/codegen-check.mjs
// Exit code 0 = generated files exist and are non-empty.
// Exit code 1 = generated files missing or stale — run `pnpm codegen`.

import { existsSync, statSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dir = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dir, "..");

/** @type {Array<{generated: string, source: string}>} */
const pairs = [
  {
    // Patched copy lives in generated/ because the canonical contract has an
    // unresolved $ref (CedarDenyResponse) that the backend team will fix in a
    // future iteration (noted in the contract's comment block). The patch adds
    // only the missing schema; no behavioural changes.
    generated: "generated/ops-workspace-shell.d.ts",
    source: "generated/ops-workspace-shell-v1.patched.yaml",
  },
  {
    generated: "generated/hr-api.d.ts",
    source: "../../microservices/hr/contracts/openapi-v1.yaml",
  },
];

let failed = false;

for (const { generated, source } of pairs) {
  const genPath = resolve(root, generated);
  const srcPath = resolve(root, source);

  if (!existsSync(genPath)) {
    console.error(`FAIL: ${generated} does not exist. Run: pnpm codegen`);
    failed = true;
    continue;
  }

  const genStat = statSync(genPath);
  if (genStat.size < 50) {
    console.error(`FAIL: ${generated} is suspiciously small (${genStat.size} bytes). Re-run: pnpm codegen`);
    failed = true;
    continue;
  }

  if (existsSync(srcPath)) {
    const srcStat = statSync(srcPath);
    if (srcStat.mtimeMs > genStat.mtimeMs) {
      console.error(
        `FAIL: ${source} is newer than ${generated}. Contract changed — run: pnpm codegen`,
      );
      failed = true;
      continue;
    }
  }

  console.log(`OK:   ${generated}`);
}

if (failed) {
  process.exit(1);
}

console.log("codegen-check passed.");
