#!/usr/bin/env node
// Run the vinxi CLI bundled under @solidjs/start's pnpm peer-resolution tree.
// This keeps @oyatie/app-shell-frontend buildable without adding a new package
// while the SolidJS transition shell remains archived pending ADR-0393 migration.

import { realpathSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const vinxiBin = realpathSync(resolve(root, "node_modules/@solidjs/start/node_modules/.bin/vinxi"));
const result = spawnSync(vinxiBin, process.argv.slice(2), {
  cwd: root,
  stdio: "inherit",
  env: process.env,
});

if (typeof result.status === "number") {
  process.exit(result.status);
}

if (result.error) {
  console.error(result.error.message);
}
process.exit(1);
