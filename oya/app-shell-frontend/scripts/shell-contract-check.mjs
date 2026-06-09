#!/usr/bin/env node
// Static shell-contract check used as the package's dependency-free lint/test gate.
// It prevents the archived SolidJS transition surface from drifting back to a
// false ADR-0372/SolidJS-canonical or prototype/demo production story.

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const packageJson = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8"));

const failures = [];

if (!packageJson.description.includes("ADR-0393")) {
  failures.push("package description must cite ADR-0393 canonical frontend authority");
}

if (!packageJson.scripts?.build?.includes("@solidjs/start/node_modules/.bin")) {
  failures.push("build script must use the existing @solidjs/start-scoped vinxi bin under pnpm");
}

const checkedFiles = [
  "package.json",
  "app.config.ts",
  "scripts/codegen-check.mjs",
  "src/app.tsx",
  "src/routes/index.tsx",
  "src/components/DashboardIsland.tsx",
  "src/components/ShellHeader.tsx",
  "src/components/ShellRail.tsx",
  "src/lib/api.ts",
  "src/lib/render-envelope.ts",
  "src/styles/app.css",
  "src/styles/tokens.css",
];

const bannedPatterns = [
  /Prototype\/demo only/i,
  /superseding the Leptos prototype/i,
  /SolidJS\s*·\s*ADR-0372/i,
  /local command only/i,
  /local visual routes/i,
  /deliberately unwired/i,
  /Select demo context/i,
  /Demo context switcher/i,
  /Available demo contexts/i,
  /oya-prototype-app/i,
  /prototype-title/i,
  /prototype-notice/i,
];

for (const file of checkedFiles) {
  const text = readFileSync(resolve(root, file), "utf8");
  for (const pattern of bannedPatterns) {
    if (pattern.test(text)) {
      failures.push(`${file} contains stale shell authority/copy: ${pattern}`);
    }
  }
}

if (failures.length > 0) {
  console.error("shell-contract-check failed:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log("shell-contract-check passed.");
