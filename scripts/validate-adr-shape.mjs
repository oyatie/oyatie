#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const validStatuses = new Set(['Proposed', 'Accepted', 'Superseded', 'Deprecated', 'Retracted']);
const requiredSections = ['Context', 'Decision', 'Consequences'];

function usage() {
  console.error('usage: node scripts/validate-adr-shape.mjs <docs/decisions/ADR-NNNN-slug.md>');
}

function readStatus(text) {
  const match = text.match(/^(?:-\s*)?>\s*\*\*Status:\*\*\s*([^\n]+)$/m)
    ?? text.match(/^(?:-\s*)?\*\*Status:\*\*\s*([^\n]+)$/m)
    ?? text.match(/^status:\s*([^\n]+)$/mi)
    ?? text.match(/^##\s+Status\s*\n+\s*([^\n]+)$/mi);
  if (!match) return null;
  return match[1].replace(/[`*.]/g, '').trim().split(/\s+[-—(]/)[0];
}

function headings(text) {
  return text
    .split(/\r?\n/)
    .filter((line) => line.startsWith('## '))
    .map((line) => line.replace(/^##\s+/, '').trim().toLowerCase());
}

function validate(file) {
  const text = fs.readFileSync(file, 'utf8');
  const base = path.basename(file);
  if (!/^ADR-\d{4}-.+\.md$/.test(base)) {
    throw new Error(`${file}: expected ADR-NNNN-slug.md filename`);
  }
  if (!text.split(/\r?\n/).some((line) => line.startsWith('# ADR-'))) {
    throw new Error(`${file}: missing ADR title`);
  }
  const status = readStatus(text);
  if (!status || !validStatuses.has(status)) {
    throw new Error(`${file}: invalid or missing status (${status ?? 'missing'})`);
  }
  const found = headings(text);
  for (const section of requiredSections) {
    if (!found.includes(section.toLowerCase())) {
      throw new Error(`${file}: missing required section ## ${section}`);
    }
  }
  const positions = requiredSections.map((section) => found.indexOf(section.toLowerCase()));
  for (let i = 1; i < positions.length; i += 1) {
    if (positions[i] < positions[i - 1]) {
      throw new Error(`${file}: required sections out of order (${requiredSections.join(' -> ')})`);
    }
  }
  return { file, status, sections: requiredSections.length };
}

const files = process.argv.slice(2);
if (files.length !== 1) {
  usage();
  process.exit(2);
}

try {
  const result = validate(files[0]);
  console.log(`adr-shape ok: file=${result.file} status=${result.status} sections=${result.sections}`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
