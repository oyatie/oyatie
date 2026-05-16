#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const [contractPath] = process.argv.slice(2);

function fail(message) {
  console.error(`asyncapi-lint: ${message}`);
  process.exit(1);
}

function assertIncludes(text, needle, label = needle) {
  if (!text.includes(needle)) {
    fail(`missing ${label}`);
  }
}

if (!contractPath) {
  fail('usage: node scripts/asyncapi-lint.mjs <asyncapi-yaml>');
}

if (!existsSync(contractPath)) {
  fail(`contract does not exist: ${contractPath}`);
}

const source = readFileSync(contractPath, 'utf8');

assertIncludes(source, 'asyncapi: 3.0.0');
assertIncludes(source, 'defaultContentType: application/cloudevents+protobuf');
assertIncludes(source, 'address: oya.platform.audit');
assertIncludes(source, 'action: send');
assertIncludes(source, 'name: audit.event.emit.v1');
assertIncludes(source, 'contentType: application/cloudevents+protobuf');
assertIncludes(source, 'additionalProperties: false');
assertIncludes(source, 'required: [specversion, id, source, type, subject, time, datacontenttype]');
assertIncludes(source, "const: '1.0'", 'CloudEvents specversion 1.0 header');
assertIncludes(source, 'const: oyatie://platform/audit-chain');
assertIncludes(source, 'const: audit.event.emit.v1');
assertIncludes(source, 'const: application/protobuf');
assertIncludes(source, 'schemaFormat: application/vnd.google.protobuf;version=3');

for (const header of ['specversion', 'id', 'source', 'type', 'subject', 'time', 'datacontenttype']) {
  assertIncludes(source, `          ${header}:`, `CloudEvents header property ${header}`);
}

const refMatch = source.match(/\$ref:\s*['"]?([^'"\n]+audit-event-v1\.proto#[^'"\n]+)['"]?/);
if (!refMatch) {
  fail('missing audit-event-v1.proto payload $ref');
}

const [protoRelPath, messageRef] = refMatch[1].split('#');
if (messageRef !== '/platform.audit.v1.AuditEvent') {
  fail(`payload $ref must target /platform.audit.v1.AuditEvent, got ${messageRef}`);
}

const protoPath = resolve(dirname(contractPath), protoRelPath);
if (!existsSync(protoPath)) {
  fail(`payload $ref target does not exist: ${protoPath}`);
}

const proto = readFileSync(protoPath, 'utf8');
if (!/package\s+platform\.audit\.v1\s*;/.test(proto)) {
  fail('payload proto package platform.audit.v1 not found');
}
if (!/message\s+AuditEvent\s*\{/.test(proto)) {
  fail('payload proto message AuditEvent not found');
}

console.log(`asyncapi-lint: ok ${contractPath}`);
