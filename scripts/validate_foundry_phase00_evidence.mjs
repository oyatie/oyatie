#!/usr/bin/env node
// M02-P03-IP-001: Phase 00 evidence validator.
//
// Validates the Phase 00 evidence contract for the foundry milestone:
//   * 8 oya-foundry-account-* crates exist (Cargo.toml present).
//   * P03 fitness-lane kernel crates exist.
//   * IP-001 / IP-002 / IP-003 ImplementationPlan files exist.
//   * No raw-secret-looking strings in account-crate sources (heuristic).
//
// Real secret-scanning happens via cargo-deny + gitleaks in CI; this gate is
// the deterministic pre-merge guard.
//
// Exit codes:
//   0  OK
//   1  one or more checks failed

import { readFileSync, existsSync, readdirSync, statSync } from 'fs';
import { join } from 'path';

const ROOT = process.argv[2] || '.';

const REQUIRED_ACCOUNT_CRATES = [
  'oya-foundry-account-kernel',
  'oya-foundry-account-domain',
  'oya-foundry-account-app',
  'oya-foundry-account-adapter-codex-cli',
  'oya-foundry-account-adapter-claude-code',
  'oya-foundry-account-adapter-gemini-cli',
  'oya-foundry-account-adapter-openbao',
  'oya-foundry-account-runtime',
];

const REQUIRED_P03_KERNEL_CRATES = [
  'oya-foundry-fitness-claim-ceiling-kernel',
  'oya-foundry-fitness-bypass-kernel',
  'oya-foundry-fitness-pr-traceability-kernel',
  'oya-foundry-fitness-pre-push-kernel',
  'oya-foundry-fitness-quality-lane-kernel',
  'oya-foundry-fitness-cohesion-fitness-kernel',
  'oya-foundry-bypass-ledger-kernel',
];

const REQUIRED_IPS = [
  '.omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-001-phase00-evidence-validator.md',
  '.omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-002-foundry-fitness-lane-ratchet.md',
  '.omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-003-adr-template-bypass-ledger.md',
];

// Heuristic regexes for the secret scan. Real CI uses gitleaks; this is the
// pre-merge gate.
const SECRET_PATTERNS = [
  /AKIA[0-9A-Z]{16}/,                         // AWS access key id
  /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/,
  /xox[abrs]-[0-9A-Za-z-]{10,}/,              // Slack token
  /ghp_[0-9A-Za-z]{36,}/,                     // GitHub personal access token
  /sk-[A-Za-z0-9]{32,}/,                      // OpenAI-style secret key
];

const errors = [];

function checkCrate(name) {
  const path = join(ROOT, 'crates', name, 'Cargo.toml');
  if (!existsSync(path)) {
    errors.push(`missing crate: ${name}`);
    return false;
  }
  return true;
}

function checkFile(rel) {
  const path = join(ROOT, rel);
  if (!existsSync(path)) {
    errors.push(`missing file: ${rel}`);
    return false;
  }
  return true;
}

function statusComplete(rel) {
  const txt = readFileSync(join(ROOT, rel), 'utf8');
  if (!/^status:\s*complete\b/m.test(txt)) {
    errors.push(`IP not marked complete: ${rel}`);
  }
}

function scanForSecrets(crate) {
  const srcDir = join(ROOT, 'crates', crate, 'src');
  if (!existsSync(srcDir)) return;
  const walk = (dir) => {
    for (const entry of readdirSync(dir)) {
      const full = join(dir, entry);
      const s = statSync(full);
      if (s.isDirectory()) {
        walk(full);
      } else if (entry.endsWith('.rs')) {
        const txt = readFileSync(full, 'utf8');
        for (const re of SECRET_PATTERNS) {
          if (re.test(txt)) {
            errors.push(`raw secret heuristic match in ${full}: ${re}`);
          }
        }
      }
    }
  };
  walk(srcDir);
}

for (const c of REQUIRED_ACCOUNT_CRATES) {
  if (checkCrate(c)) scanForSecrets(c);
}
for (const c of REQUIRED_P03_KERNEL_CRATES) {
  checkCrate(c);
}
for (const ip of REQUIRED_IPS) {
  if (checkFile(ip)) statusComplete(ip);
}

if (errors.length) {
  console.error('Phase 00 evidence validator: FAIL');
  for (const e of errors) console.error(`  - ${e}`);
  process.exit(1);
}

console.log('Phase 00 evidence validator: OK');
