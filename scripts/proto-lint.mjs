#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const manifestPath = resolve(dirname(fileURLToPath(import.meta.url)), '..', 'Cargo.toml');

const result = spawnSync(
  'cargo',
  ['run', '-q', '--manifest-path', manifestPath, '-p', 'oya-dev-cli', '--', 'lint', 'proto', ...process.argv.slice(2)],
  { stdio: 'inherit' },
);

if (result.error) {
  console.error(result.error.message);
}

process.exit(result.status ?? 1);
