---
id: ADR-FORMS-0001
title: Form-definition schema — RFC 8785 JSON Canonicalization Scheme + form.v1 JSON-Schema profile
microservice: forms
status: Accepted
date: 2026-05-17
owner: axis-forms + council-architecture
deciders: council-architecture, axis-forms, ops-security, council-design-system, axis-sdk
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0126, ADR-0131, ADR-0132, ADR-WS-0002]
related_specs: [/specs/products/forms.json]
related_artifacts:
  - microservices/forms/PRD.md FR-01 + AC-02
  - microservices/forms/contracts/openapi/forms.openapi.yaml /components/schemas/FormSpecV1
  - microservices/forms/contracts/proto/forms.proto
doc_status: published
---

# ADR-FORMS-0001: Form definition schema — RFC 8785 JCS over a typed JSON-Schema (draft 2020-12) profile

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Forms is the entry point for typed user data into the oyatie platform. The form-definition schema is the most-load-bearing public contract: tenant authoring tools, the form-builder Leptos-WASM, the response-collector REST surface, the AI-form-build LLM-assist adapter, the export workers, and every cross-µservice consumer (sheets-bridge, workflow-trigger, ontology Form entity binding) all depend on it.

We must choose:
1. **Canonical-form representation** — how a form definition is hashed, signed, transported, and round-tripped without drift.
2. **Schema-definition language** — how field types + validation are declared.
3. **Portability vs proprietary trade-off** — should the form definition be exportable to other vendors (Typeform / Jotform / SurveyMonkey) for vendor-portability?

Constraints:
- Forms must support 12 field types (text, number, date, datetime, single_choice, multi_choice, scale, grid, file_upload, signature, payment, conditional).
- Forms must support per-field `data_class` declaration (NORMAL, PII_IDENTIFYING, PII_QUASI_IDENTIFIER, SENSITIVE_GDPR_ART9, PHI, FINANCIAL, BEHAVIORAL_TENANT_PRODUCT, SECRET).
- Forms must support conditional-logic branching (see ADR-FORMS-0004) and cross-field validation.
- Forms must support i18n labels per field (≥ 14 locales, RTL included).
- Forms must be ChangeSet-bound (ADR-0110): two consecutive emits of the same form definition MUST produce byte-identical bytes (round-trip byte equality AC-02).
- Forms is competitive with Google Forms / Microsoft Forms / Typeform / Jotform / Tally / Airtable Forms / SurveyMonkey / Wufoo / Formstack / Survicate / Qualtrics / HubSpot Forms / Mailchimp Forms / Hotjar Surveys (per `competitor-parity-matrix.md`).
- Sibling ADR-WS-0002 (workflow-studio DSL canonical form) chose RFC 8785; forms benefits from the same canonicalisation reasoning for the same audit-chain + ChangeSet reasons.

## Decision

Adopt **RFC 8785 JSON Canonicalization Scheme (JCS) over a typed JSON-Schema (draft 2020-12) profile named `form.v1`**.

### Canonical form

- **Wire format**: JSON encoded with RFC 8785 JCS canonicalisation. Keys sorted lexicographically; numbers in shortest round-trippable form; strings UTF-8 with NFC normalisation; no insignificant whitespace.
- **Hashing**: SHA-256 over the JCS bytes produces the canonical `schema_hash` carried in `Form.schema_hash`. The audit-chain seal references this hash.
- **Byte-equality invariant** (AC-02): `load(emit(x)) == x` and `emit(load(emit(x))) == emit(x)` for every valid form definition.
- **Field-order semantic invariant**: array order is significant (e.g., `Page.sections[]`, `Section.fields[]`, `Field.validation.allowed_mime_types[]`); object key order is not (canonicalisation handles).

### Schema definition

- **JSON Schema draft 2020-12** is the schema-definition language for per-field validation. The `form.v1` profile constrains JSON Schema to a subset:
  - `type` ∈ {string, number, integer, boolean, array, object}.
  - `pattern` allowed for strings; ECMAScript regex flavour.
  - `minimum`, `maximum`, `minLength`, `maxLength`, `minItems`, `maxItems` allowed.
  - `format` allowed only from a fixed allow-list: `email`, `uri`, `date`, `date-time`, `phone-e164`, `uuid`.
  - `enum` allowed for single_choice / multi_choice.
  - `$ref` allowed only within the same form spec; no external `$ref`.
  - `additionalProperties` defaults to `false`.
- **Form-level extensions** (declared as JSON Schema `x-` keywords, per JSON Schema convention):
  - `x-data-class` — required on every leaf field; values from the data-class taxonomy.
  - `x-i18n-label` — required if form is published; map locale → string.
  - `x-cross-field-rules` — CEL expressions per ADR-FORMS-0004.
  - `x-branching` — DAG of branch predicates per ADR-FORMS-0004.

### Portability

- **Export**: forms exports to OpenSchema (vendor-neutral form schema standard) for portability claims; OpenSchema export is lossy for advanced features (CEL branching, eIDAS signature classes) and explicitly flagged at export time.
- **Import**: forms can import from OpenSchema-compliant exports of Typeform / Jotform / SurveyMonkey / Google Forms (best-effort; unsupported features become diagnostics).
- **Internal canonical**: form.v1 + JCS is the canonical internal form; OpenSchema is the export layer only.

## Alternatives Considered

### Alternative A — Custom DSL (per-product proprietary)

Define a bespoke form DSL with custom syntax (similar to Typeform's internal DSL or Jotform's Form Builder syntax).

- **Pros**
  - Full control over feature set; can model advanced features (eIDAS QES, payment fields) natively.
  - Compact wire format if optimised.
- **Cons**
  - No off-the-shelf tooling (no JSON Schema editors, no JSON Schema linters, no JSON Schema test generators).
  - LLM (AI-form-build) cannot leverage broad JSON Schema training data.
  - Sibling ADR-WS-0002 chose RFC 8785 + JSON-flavour for the same reasons; divergence would create a `feedback_flat_product_catalog.md` consistency violation.
  - Tenant developers must learn another syntax beyond standard JSON Schema.
- **Rejected reason**: ecosystem maturity. JSON Schema draft 2020-12 has industrial-grade tooling that Forms benefits from immediately.

### Alternative B — Plain JSON (no canonicalisation)

Use ordinary JSON with sort-on-serialise; accept that whitespace differences are tolerated.

- **Pros**
  - Simplest implementation.
  - Wide tooling support.
- **Cons**
  - No byte-identical round-trip; ChangeSet hashes drift.
  - Audit-chain seal verification fragile (different parsers produce different bytes).
  - AC-02 (round-trip byte equality) cannot be enforced structurally.
  - Competitor parity: Qualtrics' XM XML supports canonicalisation; modern Forms competitors expect deterministic hashes.
- **Rejected reason**: load-bearing audit-chain + ChangeSet hashes require deterministic canonicalisation. RFC 8785 is the standardised JSON canonicalisation; Alternative B fails AC-02.

### Alternative C — JSON Schema without canonicalisation, no x- extensions

Use plain JSON Schema for validation; carry forms-specific metadata in a sidecar file.

- **Pros**
  - Minimal coupling to forms-specific schema.
- **Cons**
  - Two-file authoring; tenant tools must keep them in sync.
  - LLM-assist must learn both formats.
  - Audit-chain must seal both files; chain-of-seals complexity doubles.
  - `data_class` declaration on a sidecar can be silently omitted; tenant publishes "by accident" without the data-class declaration on PII fields.
- **Rejected reason**: defence in depth. Carrying `x-data-class` etc. inline with JSON Schema means the form-builder + LLM-assist + dsl-loader + audit-chain all see one source of truth.

### Alternative D — Protocol Buffers as canonical (binary)

Define form.v1 as a `.proto` schema; serialise to protobuf binary as the canonical form.

- **Pros**
  - Compact binary; fast parse.
  - Strong typing.
- **Cons**
  - Protobuf has multiple wire encodings (proto3 vs proto3+JSON); not human-readable on the wire by default.
  - LLM (AI-form-build) generates JSON; converting to protobuf is an extra round-trip.
  - Audit-chain replay is harder; tooling must read the .proto schema to interpret bytes.
  - Tenants cannot inspect a form definition with `jq` or `cat`.
- **Rejected reason**: human-readability + LLM-friendliness. Forms is authored + reviewed by humans; canonical JSON wins over canonical protobuf for this product.

### Alternative E — OpenSchema (industry-neutral) as canonical

Adopt OpenSchema directly as the canonical form schema; no proprietary extensions.

- **Pros**
  - Vendor portability built-in.
  - Reduces lock-in.
- **Cons**
  - OpenSchema lacks features oyatie needs (eIDAS QES classes, Annex III §4 attestation, per-tenant DEK column declarations, CEL branching).
  - The "neutral" subset would force feature deletion to fit.
  - OpenSchema is itself evolving; pinning a moving target as canonical is fragile.
- **Rejected reason**: OpenSchema is excellent for **export** (portability claim) but insufficient for **canonical** (feature set). The chosen design uses form.v1 + JCS as canonical and exports to OpenSchema for portability.

## Consequences

### Architectural

- The `oya-forms-domain` kernel exposes `FormSpecV1` as the canonical type; all adapters (Postgres, Meilisearch, REST, gRPC, AsyncAPI) serialise via the same JCS-aware encoder.
- The dsl-loader rejects non-canonical input on the read path; emit path is canonical by construction.
- The `Form.schema_hash` field carries the SHA-256(JCS(form)) — this is the audit-chain seal identifier.
- Form-builder Leptos-WASM uses the same JCS encoder client-side for offline preview; server-side re-canonicalises authoritatively.

### Downstream µservices

1. **audit-chain**: receives `schema_hash` per form publish; chain-of-seals references it.
2. **ontology**: `Form` entity carries `schema_hash`; queryable for any consumer.
3. **workflow-engine**: form-submission event carries `(form_id, form_version, schema_hash)` triple; engine binds workflow to specific schema_hash for replay determinism.
4. **sheets**: sheets-bridge consumes responses against `schema_hash` of submission-time spec; old responses always interpretable against old spec.
5. **foundry-providers**: LLM-assist invocations include `form.v1` profile in system prompt; LLM outputs validated against the profile.
6. **All SDKs** (Rust / TypeScript / Python / Go / Java): include `form.v1` JSON Schema as a shipped resource; clients validate locally before round-trip.

### SLOs and CI lanes affected

- `oya-governance-canonical-form-byte-equality` — exercised on every form publish path (AC-02).
- `oya-governance-form-schema-conformance` — every published form must pass JSON Schema validation against `form.v1`.
- `oya-governance-form-schema-x-data-class-required` — every leaf field carries `x-data-class`; missing → publish blocked.
- `oya-forms-openapi-conformance` — REST surface matches OpenAPI; the OpenAPI references `form.v1` schema.

### Compliance + audit

- GDPR Art. 5(1)(b) purpose limitation: `purpose` field in spec is mandatory at publish (validator-enforced).
- GDPR Art. 7 consent: `consent_notice` field mandatory if any field has `x-data-class=PII_*`.
- EU AI Act Art. 50 transparency: `ai_build_origin` boolean in spec; submitter banner rendered when true.
- HIPAA: `x-data-class=PHI` declaration gates publish in pack-us-healthcare only.

### Risk register

- **Risk**: JSON Schema draft 2020-12 spec evolves; current profile becomes stale. **Mitigation**: pin `$schema` to draft-2020-12 explicitly; supersession ADR if upgrade required.
- **Risk**: JCS edge cases (unicode normalisation; number precision). **Mitigation**: shared canonicaliser library (RFC 8785-reference); property test suite over the 14-locale corpus + boundary numbers.
- **Risk**: LLM output (AI-form-build) generates non-canonical JSON. **Mitigation**: dsl-loader re-canonicalises before save; LLM output is never canonical-by-construction.

## References

- IETF RFC 8785 — JSON Canonicalization Scheme (JCS) — `tools.ietf.org/html/rfc8785`.
- JSON Schema draft 2020-12 — `json-schema.org/draft/2020-12/`.
- ADR-WS-0002 (workflow-studio DSL canonical form) — sibling decision.
- ADR-0110 ChangeSet state machine.
- ADR-0028 audit-chain Ed25519 seal.
- ADR-0131 per-microservice flat layout.
- OpenSchema (vendor-neutral form schema) — `openschema.io/`.
- Typeform / Jotform / SurveyMonkey internal schemas (public docs).
- `feedback_no_silent_regression.md` (round-trip invariant).
- `competitor-parity-matrix.md`.
