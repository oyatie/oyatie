#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const result = spawnSync(
  'buck2',
  ['run', '//oya/developer-sdk/crates/oya-dev-cli:oya', '--', 'lint', 'foundry-phase00-evidence', ...process.argv.slice(2)],
  { cwd: repoRoot, stdio: 'inherit' },
);

if (result.error) {
  console.error(result.error.message);
}

process.exit(result.status ?? 1);
