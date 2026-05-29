---
id: ADR-WS-0002
title: DSL canonical form for workflow_spec.v1.json (round-trip byte-equality)
microservice: workflow-studio
status: Accepted
date: 2026-05-17
owner: axis-workflow + council-architecture
deciders: council-architecture, axis-workflow, axis-workflow-engine
supersedes: []
superseded_by: []
related: [ADR-0056, ADR-0105, ADR-0131]
related_specs: [/specs/microservices/workflow-studio.json, /specs/microservices/workflow.json]
related_artifacts:
  - microservices/workflow-studio/PRD.md (AC-02, FR-04, FR-05, FR-06)
  - microservices/workflow-studio/IP-003-dsl-emitter-loader-kernel-domain.md
  - microservices/workflow-engine/PRD.md (spec-store boundary)
purpose: Resolve the canonical serialization shape of workflow_spec.v1.json so that AC-02 round-trip byte-equality (load(emit(load(spec))) == spec) holds for 100% of the reference corpus at GA.
doc_status: published
---

# ADR-WS-0002: DSL canonical form — JSON Canonicalization Scheme (RFC 8785) + workflow-studio profile

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD §"Functional Requirements" FR-04 (Studio emits `workflow_spec.v1.json` on save), FR-05 (Studio loads the same spec format and renders semantically identical visual), FR-06 (round-trip byte-equality), AC-02 (100% byte-equality over a 100-spec reference corpus at GA) collectively make the spec's canonical serialization form a load-bearing invariant. The workflow-engine µservice consumes the same spec format; engine + studio + git-PR hand-edits MUST all produce byte-identical output for semantically equivalent specs.

This is uncommon for visual authoring tools. The competitor parity matrix (`competitor-parity-matrix.md` §"Visual authoring core") documents: n8n, Zapier, Workato, Make, Power Automate, Foundry Pipeline Builder, Retool, Tines, Step Functions Studio all fail this property. Camunda's BPMN XML is partial. **Round-trip byte-equality is oyatie's unique differentiator** (per `competitor-parity-matrix.md` §"Key oyatie Differentiators" #1). The serialization choice determines whether the invariant is achievable structurally or merely aspirational.

Concrete failure modes the canonical form must structurally prevent:

- **Map-key ordering nondeterminism** — standard JSON serializers emit object keys in insertion order, which depends on the in-memory data structure. Two semantically-equal documents can disagree byte-wise.
- **Number representation drift** — `1.0` vs `1` vs `1e0` all parse to the same IEEE 754 double but serialize differently.
- **Whitespace drift** — pretty-printed vs compact, tabs vs spaces, trailing newlines.
- **String escape choice** — `"é"` vs `"é"`; `"/"` vs `"\/"`.
- **Unicode normalization drift** — NFC vs NFD for combining-character strings (a real concern given KR/EU/US-HC localization packs).
- **Floating-point trailing zeros** — `1.10` vs `1.1`.

Substrate constraints:
- The format is consumed by Rust (workflow-engine, workflow-studio kernel/domain), browser-WASM (Leptos canvas), and human hand-edit (git PR workflow). All three must produce identical bytes for identical semantics.
- The format must be diff-friendly (developers read git diffs of workflow specs).
- The format must support Ed25519-signed audit seals (per Bominal ADR-0028 inherited); the signed body MUST be the canonical bytes, otherwise signatures break when the same spec round-trips through different serializers.
- The format must be backward-compatible across `workflow_spec` major versions; canonical-form rules MUST NOT change for an existing major version.

## Decision

Adopt **RFC 8785 (JSON Canonicalization Scheme, JCS)** as the canonical serialization form of `workflow_spec.v1.json`, with a workflow-studio canonical-form profile layered on top to close ambiguities RFC 8785 leaves to the application.

### The workflow-studio JCS profile

1. **Serialization**: RFC 8785 §3 rules apply — lex-sorted UTF-16 code-unit map keys, ES2017 (`Number.prototype.toString`-compatible) number serialization, RFC 8259 string escaping with the JCS-mandated minimal escape set, no insignificant whitespace, UTF-8 output without BOM.
2. **Whitespace**: zero insignificant whitespace in the on-disk canonical form. Pretty-printed views (Studio's DSL pane; `oya vcs diff` output) are projections, not canonical — they MUST NOT be stored or signed.
3. **Trailing newline**: exactly one LF byte (`0x0A`) at end of file. POSIX text-file convention; required for `git diff` to behave; consumers strip before canonical-bytes hashing.
4. **Number representation**: integers serialize without decimal point (`1`, not `1.0`). Floats use the shortest round-trippable representation (ES2017 §7.1.12.1, matching V8 / JSC / SpiderMonkey output). Per RFC 8785, this is unambiguous. NaN / Infinity are forbidden in the spec; if encountered, validation rejects the spec with a precise diagnostic.
5. **Unicode normalization**: all string fields are NFC-normalized at the emitter boundary. The dsl-loader rejects (with a precise diagnostic) any spec whose strings are not NFC-normalized. This closes the NFC/NFD drift hole for KR/EU multi-byte strings.
6. **Field set**: the workflow_spec.v1 JSON Schema (authored under `microservices/workflow-engine/contracts/`) is the closed set of allowed fields; unknown fields are rejected at load. This prevents silent-field-loss round-trip violations.
7. **Array ordering**: arrays are semantically-significant (node order matters; edge order matters). The emitter MUST preserve input order; no implicit sort. (Maps are sorted by JCS; arrays are not.)
8. **Audit-chain seal**: Ed25519 signatures are computed over the canonical-form bytes minus the trailing LF. The seal includes `(tenant_id, spec_id, version_sha = sha256(canonical_bytes), author_identity, parent_version_sha, timestamp)`.
9. **JSON Schema validation**: the spec MUST validate against `workflow_spec.v1.schema.json` (additionalProperties: false; per-field constraints) before canonical bytes are computed. Validation failure is a precise, line-numbered error surfaced in the editor.
10. **Profile versioning**: this canonical-form profile is `workflow-studio-jcs-profile@1`. Future profile versions are additive (e.g., extending the schema) and require an ADR superseding ADR-WS-0002. The profile version is embedded in the spec's top-level `$schema` field.

### CI lane enforcement

`oya-governance-workflow-spec-roundtrip` (per PHASE-01 IP-003 + IP-015) executes:
1. Load each spec in the 100-spec reference corpus.
2. Emit through dsl-emitter using the workflow-studio JCS profile.
3. Assert byte-equal to the corpus input.
4. Repeat with a property-test fuzzer that mutates valid specs and asserts canonical-form determinism (load → emit → load → emit → byte-equal).

The lane is BLOCKER on `dev` per PHASE-01 §"branch-protection.yaml diff preview".

## Alternatives Considered

### Alternative A — YAML 1.2

YAML is the most common visual-tool serialization (n8n exports YAML; GitHub Actions uses YAML).

- **Pros**
  - Human-friendly; comments survive round-trips in libraries that support them (e.g., `serde_yaml_ng`).
  - Multi-line strings without escaping noise.
  - Widely adopted in CI/CD configuration where workflows live.
- **Cons**
  - **YAML 1.2 has no canonicalization standard**. Library implementations differ on flow-style vs block-style, quoting heuristics ("yes" parsed as boolean), anchor/alias expansion, multi-document handling.
  - The "Norway problem" (`NO` parsed as `false`) and the "Sexagesimal problem" (`12:34:56` parsed as 45296) are well-documented production bugs.
  - YAML's tagged-scalar system makes typing implicit; this is the opposite of what the engine + studio need.
  - No widely-adopted canonical-form RFC (only proprietary library opinions).
- **Rejected reason**: byte-equality is not achievable without re-authoring the entire YAML canonicalization standard, which is research-grade work. The "human-friendly" advantage is not load-bearing because Studio always renders the spec in a pretty-printed editor pane; the on-disk form does not need to be human-formatted.

### Alternative B — CUE

CUE (cuelang.org) is a typed configuration language with strong validation, growing in popularity for Kubernetes / config-as-code.

- **Pros**
  - First-class type system; could subsume schema validation.
  - Built-in canonical form (`cue export --simple`).
  - Compositional (constraints + templates compose cleanly).
- **Cons**
  - Tenant ecosystem is small relative to JSON; tenant hand-edit workflows assume JSON tooling (jq, jsonschema, jsonpath, etc.).
  - Browser-WASM CUE evaluator does not exist at production grade; would require porting or a server-side eval round-trip on every save.
  - Engine + studio + git-PR triad all need CUE — adds a learning-curve dependency to every workflow-aware product.
  - Canonical form is library-managed, not RFC-standardized; cross-implementation byte-equality is not contractual.
- **Rejected reason**: ecosystem cost. JSON is ubiquitous; CUE is niche. Studio's differentiator is round-trip byte-equality, not the configuration language; piggy-backing on RFC 8785's JSON ecosystem is the lower-risk path.

### Alternative C — Custom S-expression / EDN

A homegrown Lisp-style serialization could be designed to be canonical by construction (only one printable form per value).

- **Pros**
  - Trivially canonical (no quoting choice, no map-ordering ambiguity if pairs are ordered).
  - Compact.
- **Cons**
  - No tenant tooling — every consumer (Studio canvas, engine, git diff viewer, tenant LLM, audit-archive grep) needs a custom parser.
  - Loses JSON's vast ecosystem (JSON Schema, jq, jsonpath, OpenAPI references).
  - Failure mode: when oyatie does eventually need a non-Rust SDK or a CLI hand-edit, every contributor confronts a custom syntax.
- **Rejected reason**: ecosystem cost dwarfs the byte-equality benefit. JCS achieves the same byte-equality without surrendering the ecosystem.

### Alternative D — Default `serde_json::to_string_pretty` with a custom map-sort wrapper

The path of least resistance: keep using standard `serde_json` but add a thin wrapper that sorts map keys.

- **Pros**
  - Zero new dependencies.
  - Familiar to every Rust developer.
- **Cons**
  - Does not close the number-representation drift hole (`1.0` vs `1`).
  - Does not close the Unicode-normalization hole.
  - Does not close the escape-choice hole (`"/"` vs `"\/"`).
  - Not a standard; oyatie owns the canonicalization specification and the bugs.
  - Audit seals computed under this form are not portable to any external verifier without re-implementing the same wrapper.
- **Rejected reason**: re-implements a non-trivial slice of RFC 8785 informally. The benefit of using a published RFC is that external auditors and tenant tooling can verify oyatie's canonical bytes independently.

## Consequences

### Architectural

- The dsl-emitter `-domain` crate depends on a JCS implementation (the chosen Rust crate is `serde_jcs` at `crates.io/crates/serde_jcs` v0.1.x, vendored if upstream goes unmaintained).
- The dsl-loader `-domain` crate rejects non-NFC strings and unknown fields with precise diagnostics; the canvas surfaces these as line-numbered editor errors.
- The audit-chain seal (Bominal ADR-0028 inherited) signs canonical bytes; this becomes the single bytes-on-the-wire definition for signature verification across studio + engine + tenancy.
- The JSON Schema for `workflow_spec.v1` lives in `microservices/workflow-engine/contracts/`; studio + engine both validate against it.

### Downstream impact on other µservices and IPs

1. **IP-003 (dsl-emitter/dsl-loader kernel + domain)** — adopts `serde_jcs`; round-trip property test + 100-spec reference corpus authored here; `oya-governance-workflow-spec-roundtrip` lane wired BLOCKER on dev.
2. **workflow-engine µservice** — spec-store accepts only canonical-form bytes; non-canonical submission is a 400 with a precise diagnostic; engine validates JCS conformance on receive.
3. **tenancy µservice** — Cedar policies that read spec fields receive canonical-form bytes; entity-encoded for Cedar evaluation deterministically.
4. **observability µservice** — `definition_saved` events carry `version_sha = sha256(canonical_bytes)`; this enables cross-region replay verification.
5. **foundry-providers µservice** — LLM-assist drafts are passed through the dsl-loader + dsl-emitter cycle BEFORE save; the canonical-form pass rejects LLM-emitted invalid specs with line-numbered errors per AC-05.
6. **git PR workflow** — git diff shows canonical-form bytes; pretty-printed views in Studio are display-only.
7. **All workflow-aware product µservices** (Connect, healthcare workflows, supply-chain workflows) — consume the same canonical form via workflow-engine-sdk; behavioural change is transparent at the SDK boundary.

### SLOs and CI lanes affected

- `oya-governance-workflow-spec-roundtrip` — new BLOCKER lane on `dev`, `staging` (per PHASE-01 §"branch-protection.yaml diff preview").
- `workflow-studio.spec_validation_p99_ms` — bounded by PRD §"Performance" 10ms p50 / 50ms p99 / 200ms p999 for browser-side validation.
- `workflow-studio.round_trip_byte_equal_rate` — 100% required at GA per AC-02.

### Tooling impacts

- A `oya workflow-studio canonicalize <spec>` CLI subcommand is added under `oya-dev-cli` so tenants can canonicalize hand-edited specs offline before submission.
- The `oya vcs diff` rendering for `.workflow_spec.json` files canonicalizes both sides before diffing, preventing whitespace-only PRs.
- Tenant-side `pre-commit` hook (shipped under `microservices/workflow-studio/iac/`) runs canonicalization on every commit touching `*.workflow_spec.json`.

### Risk register

- **Risk**: `serde_jcs` upstream maintainer-attrition. **Mitigation**: vendor the crate under `microservices/workflow-studio/src/crates/` if upstream goes stale; RFC 8785 itself is stable and self-contained.
- **Risk**: Performance impact of JCS on huge specs (10k+ nodes). **Mitigation**: incremental hashing in the loader; pre-compute version_sha at emit time.
- **Risk**: Profile drift — future change to the canonical-form profile breaks audit-seal verification. **Mitigation**: profile is versioned; profile version is in `$schema`; ADR-WS-0002 supersession is the only way to bump.

## References

- PRD `microservices/workflow-studio/PRD.md` §"Functional Requirements" FR-04..FR-06, AC-02.
- `microservices/workflow-studio/IP-003-dsl-emitter-loader-kernel-domain.md`.
- `microservices/workflow-studio/competitor-parity-matrix.md` §"Visual authoring core", §"Key oyatie Differentiators" #1.
- RFC 8785 — "JSON Canonicalization Scheme (JCS)," `datatracker.ietf.org/doc/html/rfc8785`.
- RFC 8259 — "The JavaScript Object Notation (JSON) Data Interchange Format," `datatracker.ietf.org/doc/html/rfc8259`.
- Unicode Standard Annex #15 — "Unicode Normalization Forms," `unicode.org/reports/tr15/`.
- ADR-WS-0001 (CRDT library) — projection-to-canonical contract.
- Bominal ADR-0028 — audit-chain Ed25519 seal (inherited).
- ADR-0164 (Bominal) — Workflow canonical spec format (inherited; this ADR is the workflow-studio-side serialization specification).
- `serde_jcs` — `crates.io/crates/serde_jcs`.
