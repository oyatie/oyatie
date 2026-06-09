#!/usr/bin/env node
// ADR-0372 D2: Verify that OpenAPI client generation is deterministic and clean-checkout safe.
// Run in CI after `pnpm codegen` (or by itself) to enforce "contract change breaks build".
//
// Usage:  node scripts/codegen-check.mjs
// Exit code 0 = sources generate non-empty clients; if ignored in-tree clients exist, they match.
// Exit code 1 = source missing, generator failed, generated output empty, or in-tree output stale.

import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dir = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dir, "..");

/** @type {Array<{generated: string, source: string}>} */
const pairs = [
  {
    generated: "generated/ops-workspace-shell.d.ts",
    source: "../../contracts/ops-workspace-shell-v1.openapi.yaml",
  },
  {
    generated: "generated/hr-api.d.ts",
    source: "../hr/contracts/openapi-v1.yaml",
  },
];

const tempRoot = mkdtempSync(join(tmpdir(), "oyatie-app-shell-codegen-"));
let failed = false;

try {
  for (const { generated, source } of pairs) {
    const genPath = resolve(root, generated);
    const srcPath = resolve(root, source);
    const tempPath = join(tempRoot, generated.replaceAll("/", "__"));

    if (!existsSync(srcPath)) {
      console.error(`FAIL: ${source} does not exist`);
      failed = true;
      continue;
    }

    try {
      execFileSync("openapi-typescript", [srcPath, "-o", tempPath], {
        cwd: root,
        stdio: "pipe",
      });
    } catch (error) {
      console.error(`FAIL: ${source} did not generate ${generated}`);
      if (error.stderr) {
        console.error(String(error.stderr));
      }
      failed = true;
      continue;
    }

    const tempStat = statSync(tempPath);
    if (tempStat.size < 50) {
      console.error(`FAIL: ${generated} generated suspiciously small output (${tempStat.size} bytes)`);
      failed = true;
      continue;
    }

    if (existsSync(genPath)) {
      const current = readFileSync(genPath, "utf8");
      const regenerated = readFileSync(tempPath, "utf8");
      if (current !== regenerated) {
        console.error(`FAIL: ${generated} is stale. Run: pnpm codegen`);
        failed = true;
        continue;
      }
      console.log(`OK:   ${generated} matches regenerated output`);
    } else {
      console.log(`OK:   ${generated} regenerates from ${source} (not committed)`);
    }
  }
} finally {
  rmSync(tempRoot, { force: true, recursive: true });
}

if (failed) {
  process.exit(1);
}

console.log("codegen-check passed.");
