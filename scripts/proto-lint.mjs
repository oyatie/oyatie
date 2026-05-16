#!/usr/bin/env node
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const [inputPath] = process.argv.slice(2);

function fail(message) {
  console.error(`proto-lint: ${message}`);
  process.exit(1);
}

function collectProtoFiles(path) {
  if (!existsSync(path)) {
    fail(`path does not exist: ${path}`);
  }

  const stat = statSync(path);
  if (stat.isFile()) {
    return path.endsWith('.proto') ? [path] : [];
  }

  return readdirSync(path)
    .flatMap((entry) => collectProtoFiles(join(path, entry)))
    .filter((entry) => entry.endsWith('.proto'))
    .sort();
}

function messageBody(source, messageName) {
  const header = new RegExp(`message\\s+${messageName}\\s*\\{`, 'g');
  const match = header.exec(source);
  if (!match) {
    fail(`${messageName} message missing`);
  }

  let depth = 1;
  let cursor = match.index + match[0].length;
  while (cursor < source.length && depth > 0) {
    if (source[cursor] === '{') {
      depth += 1;
    } else if (source[cursor] === '}') {
      depth -= 1;
    }
    cursor += 1;
  }

  if (depth !== 0) {
    fail(`${messageName} message has unbalanced braces`);
  }

  return source.slice(match.index + match[0].length, cursor - 1);
}

function parseFields(source) {
  const fields = new Map();
  const fieldPattern = /^\s*(repeated\s+)?([A-Za-z0-9_.]+)\s+([a-z0-9_]+)\s*=\s*([0-9]+);/gm;
  let match;
  while ((match = fieldPattern.exec(source)) !== null) {
    fields.set(match[3], {
      repeated: Boolean(match[1]),
      type: match[2],
      number: Number(match[4]),
    });
  }
  return fields;
}

function requireField(fields, name, type, number, repeated = false) {
  const field = fields.get(name);
  if (!field) {
    fail(`missing field ${name}`);
  }
  if (field.type !== type || field.number !== number || field.repeated !== repeated) {
    const actual = `${field.repeated ? 'repeated ' : ''}${field.type} ${name} = ${field.number}`;
    const expected = `${repeated ? 'repeated ' : ''}${type} ${name} = ${number}`;
    fail(`field mismatch for ${name}: expected ${expected}, got ${actual}`);
  }
}

if (!inputPath) {
  fail('usage: node scripts/proto-lint.mjs <proto-file-or-directory>');
}

const protoFiles = collectProtoFiles(inputPath);
if (protoFiles.length === 0) {
  fail(`no .proto files found under ${inputPath}`);
}

const auditProto = protoFiles.find((path) => path.endsWith('audit-event-v1.proto'));
if (!auditProto) {
  fail('audit-event-v1.proto not found');
}

const source = readFileSync(auditProto, 'utf8');

if (!/^syntax\s*=\s*"proto3";/m.test(source)) {
  fail('syntax must be proto3');
}
if (!/^package\s+platform\.audit\.v1;/m.test(source)) {
  fail('package must be platform.audit.v1');
}
if (!/message\s+AuditEventEd25519Signature\s*\{/.test(source)) {
  fail('AuditEventEd25519Signature message missing');
}
if (!/message\s+AuditEvent\s*\{/.test(source)) {
  fail('AuditEvent message missing');
}
if (/message\s+AuditEventEmit\s*\{/.test(source)) {
  fail('legacy AuditEventEmit message must not remain as a second source schema');
}

const signatureFields = parseFields(messageBody(source, 'AuditEventEd25519Signature'));
requireField(signatureFields, 'key_id', 'string', 1);
requireField(signatureFields, 'public_key_hex', 'string', 2);
requireField(signatureFields, 'signature_hex', 'string', 3);

const auditFields = parseFields(messageBody(source, 'AuditEvent'));
requireField(auditFields, 'id', 'string', 1);
requireField(auditFields, 'tenant_id', 'string', 2);
requireField(auditFields, 'surface', 'string', 3);
requireField(auditFields, 'plane', 'string', 4);
requireField(auditFields, 'purpose', 'string', 5);
requireField(auditFields, 'data_classes_touched', 'string', 6, true);
requireField(auditFields, 'decision', 'string', 7);
requireField(auditFields, 'idempotency_key', 'string', 8);
requireField(auditFields, 'emitted_at_epoch_seconds', 'uint64', 9);
requireField(auditFields, 'tenant_shard', 'string', 10);
requireField(auditFields, 'sequence', 'uint64', 11);
requireField(auditFields, 'previous_hash', 'string', 12);
requireField(auditFields, 'hash', 'string', 13);
requireField(auditFields, 'merkle_root', 'string', 14);
requireField(auditFields, 'ed25519_signature', 'AuditEventEd25519Signature', 15);

for (const [messageName, fields] of [
  ['AuditEventEd25519Signature', signatureFields],
  ['AuditEvent', auditFields],
]) {
  const tagNumbers = [...fields.values()].map((field) => field.number);
  if (tagNumbers.length !== new Set(tagNumbers).size) {
    fail(`duplicate protobuf field tags detected in ${messageName}`);
  }
}

console.log(`proto-lint: ok ${auditProto}`);
