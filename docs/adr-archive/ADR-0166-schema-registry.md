---
id: ADR-0166
status: Superseded
deciders: council-architecture, axis-governance, axis-eventing, axis-ontology, axis-application
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-705]
related: [ADR-0005, ADR-0011, ADR-0029, ADR-0100, ADR-0105, ADR-0128, ADR-0131, ADR-0145, ADR-0148, ADR-0157]
related_specs:
  - /specs/schema-registry-canonical.json
  - /specs/hyperscaler-architecture-invariants.json
---

# ADR-0166 — Schema Registry (Apicurio Registry; Confluent-compat API; AsyncAPI 3.x + proto3 + OpenAPI 3.1; backward-compat lane)

## Status

Accepted (2026-05-18). Adopts Apicurio Registry 3.x as the canonical schema registry. Implements the Confluent Schema Registry compatibility API + AsyncAPI 3.x + Protobuf + OpenAPI 3.1 schema kinds. Backward-compatibility check is a release-blocking CI lane.

## Context

ADR-0005 named the eventing backbone + outbox pattern. ADR-0011 named the cross-microservice contract registry. ADR-0029 named the connect dual-context architecture. ADR-0100 named the supervisor public-contract lean-A10 lane. ADR-0145 named the inter-µservice communication reform with first-class gRPC / proto3 + REST / OpenAPI 3.1 surfaces.

What does NOT exist canonically yet:

1. A single **schema registry** holding every event class (AsyncAPI / Avro / proto3) + every REST surface (OpenAPI 3.1) + every gRPC surface (proto3) shipped by oyatie.
2. A **versioning + compatibility check** that catches breaking changes BEFORE the breaking PR merges.
3. A **per-event-class lookup** for consumers ("what is the latest schema for `WorkflowStepCompleted`?").

Without these:

- Consumers hard-code schema versions inside their code.
- A producer changes a field name; the consumer breaks in production; rollback is required.
- The lean-A10 silent-regression invariant (ADR-0100) is impossible to enforce automatically.

The hyperscaler precedent is uniform:

- **Confluent Schema Registry** — the de-facto Kafka schema registry; per-subject compatibility level (BACKWARD / FORWARD / FULL / NONE); paid commercial.
- **Apicurio Registry** (Red Hat, Apache 2.0) — open-source; Confluent-compat API; broader schema-kind support (AsyncAPI / OpenAPI / proto3 / Avro / JSON Schema / XSD); GraphQL too.
- **AWS Glue Schema Registry** — AWS-managed; AWS-specific.
- **Google Cloud Pub/Sub Schema Registry** — GCP-managed; GCP-specific.
- **EventCatalog** — documentation-tier, not runtime registry.

ADR-0166 adopts Apicurio Registry — open-source, Confluent-compat (so future Kafka consumers integrate trivially), and supports the broader schema-kind surface oyatie needs.

## Decision

Oyatie adopts **Apicurio Registry 3.x** as the canonical schema registry. Properties:

### Schema kinds

The registry holds:

- **AsyncAPI 3.x** — event-driven contracts (eventing backbone per ADR-0005). One AsyncAPI doc per event-emitting µservice; per-event-class subject.
- **Protobuf 3 (proto3)** — gRPC service definitions (per ADR-0145). One `.proto` file per gRPC service.
- **OpenAPI 3.1** — REST surface definitions. One OpenAPI doc per REST-exposing µservice.
- **Avro** — accepted but discouraged; legacy Kafka producers may use Avro; new code uses proto3.
- **JSON Schema** — accepted for non-event configuration documents (manifest.json schemas, scorecard schemas).

### Subject naming convention

`<schema-kind>.<microservice>.<surface>.<event-class-or-resource>` — examples:

- `asyncapi.audit-chain.events.audit-seal-emitted`
- `proto3.tenancy.api.TenancyService.CreateTenant`
- `openapi.workflow-engine.rest.v1.workflows`
- `jsonschema.governance.manifest.scorecard-overrides`

### Versioning + compatibility

- **Semantic versions per subject** — `<major>.<minor>.<patch>`. The registry assigns + tracks.
- **Compatibility levels per subject:**
  - `BACKWARD` — new schema can read old data. Default for events + REST responses.
  - `FORWARD` — old schema can read new data. Default for REST requests.
  - `FULL` — both. Default for ontology + audit-chain (where both producers and consumers persist across versions).
  - `NONE` — no compatibility check. Reserved for explicit-ADR carve-out.
- **Major version bump** — breaks compatibility intentionally; requires ADR + sunset window per ADR-0100 (no silent regression).

### Schema-registry CI lane

`cloud-ci/Rust gate packet schema-registry-backward-compat`:

- Reads every µservice's `contracts/asyncapi-v*.yaml` + `contracts/proto3/*.proto` + `contracts/openapi-v*.yaml`.
- For each schema, compares against the previously-published version in the registry.
- Fails the build if the new version breaks the declared compatibility level.
- Required-status-check on every branch promoted to `dev`/`staging`/`production`.

### Registry deployment

- **Per-cell deployment** (ADR-0009 cell architecture). Each cell hosts an Apicurio Registry instance with PostgreSQL backing.
- **Per-pack overlay** for sovereign packs (ADR-0164) — air-gap variant uses in-cell PostgreSQL.
- **Read-replica for build runners** — CI runners read the registry via a global read-replica for fast schema lookup.

### Per-µservice contract publication

Every µservice's release pipeline publishes:

- AsyncAPI doc → `asyncapi.<ms>.*` subjects.
- proto3 service definitions → `proto3.<ms>.*` subjects.
- OpenAPI doc → `openapi.<ms>.*` subjects.

Publication is a step in the ChangeSet promotion pipeline (ADR-0110) — staging publish, then production publish; the registry maintains both side-by-side until the prior major version is sunset.

### Runtime consumer integration

- gRPC consumers fetch the proto3 schema at startup from the registry.
- AsyncAPI consumers fetch the event-class schema; deserialization validates against the schema.
- REST consumers may pin the OpenAPI version in their client SDK at build time.

## Alternatives considered

### Alternative A — Confluent Schema Registry (commercial)

- **Pros:** the most mature; Kafka-native.
- **Cons:** commercial license; doesn't natively support AsyncAPI 3.x / OpenAPI 3.1 / proto3 outside Kafka envelope; sovereign-pack (ADR-0164) cannot use commercial SaaS.
- **Rejected because:** scope mismatch + license + sovereign constraint.

### Alternative B — AWS Glue Schema Registry

- **Pros:** AWS-managed; integrates with Kinesis + Kafka.
- **Cons:** AWS-specific (ADR-0121 portability invariant); sovereign-pack cannot use.
- **Rejected because:** portability + sovereignty.

### Alternative C — No schema registry; ship schemas as JSON in `contracts/` directory; CI diff-check

- **Pros:** simplest.
- **Cons:** consumers cannot dynamically fetch latest schema; no runtime version negotiation; compatibility check requires custom Rust tool; this is the historical anti-pattern.
- **Rejected because:** runtime fetch + spec-compatibility tooling are real value; reinventing them is wasteful.

### Alternative D — Apicurio Registry 3.x (this ADR)

- **Pros:** Apache 2.0; Confluent-compat API; supports AsyncAPI + OpenAPI + proto3 + Avro + JSON Schema; broad schema-kind surface; Red Hat backed; per-cell deployable; sovereign-compatible.
- **Cons:** another µservice tier per cell; PostgreSQL backing per cell.
- **Accepted.**

### Alternative E — Buf BSR (Buf Schema Registry)

- **Pros:** excellent proto3 tooling; modern; buf-cli integration.
- **Cons:** primarily proto3-focused (AsyncAPI / OpenAPI support is partial); commercial SaaS for hosted variant; self-host is more limited; less broad than Apicurio.
- **Rejected because:** scope narrower than oyatie needs; commercial-SaaS sovereign concern.

## Consequences

### Positive

1. **Single registry for all schema kinds.** Events + REST + gRPC + JSON Schema all in one substrate.
2. **Backward-compat structurally enforced.** Breaking changes blocked at CI before merge.
3. **Lean-A10 silent-regression invariant (ADR-0100) closed.** Every public contract has a registry entry; every change versioned.
4. **Consumer integration uniform.** Runtime fetch + version pinning + compatibility negotiation all standardized.
5. **Sovereign-pack compatible.** Per-cell deployment + PostgreSQL backing; no external SaaS.
6. **Confluent-compat API.** Future Kafka consumers integrate trivially.
7. **AsyncAPI 3.x event documentation.** Event-driven contracts machine-readable; downstream tools (EventCatalog, dashboards) consume.

### Negative

1. **Apicurio operator per cell.** Each cell adds Apicurio Registry + PostgreSQL backing.
2. **Per-µservice contract authoring cost.** Every µservice must author AsyncAPI / proto3 / OpenAPI; this is the lean-A10 invariant in practice.
3. **Registry availability is critical-path.** If the registry is down, builds cannot fetch schemas; CI fails. Mitigated by per-cell deployment + read-replica.
4. **Compatibility-level choice is per-subject.** Defaults declared but per-subject override allowed; review burden non-zero.

### Operational

1. New µservice scaffolded at `microservices/governance/iac/helm/apicurio-registry/Chart.yaml` (Companion).
2. CI lane `cloud-ci/Rust gate packet schema-registry-backward-compat` enforces.
3. Per-µservice `contracts/asyncapi-v1.yaml` + `contracts/proto3/*.proto` + `contracts/openapi-v1.yaml` shipped by every µservice.
4. Subject naming validated by `cloud-ci/Rust gate packet schema-registry-naming-convention`.
5. Per-pack overlay for sovereign packs at `microservices/governance/iac/kustomize/components/pack-{ksa,kr-fsc,...}/apicurio-registry/`.
6. Companion spec `specs/schema-registry-canonical.json` declares the subject naming, compatibility-level defaults, and major-version sunset policy.

## References

- Apicurio Registry — https://www.apicur.io/registry/
- Confluent Schema Registry — https://docs.confluent.io/platform/current/schema-registry/index.html
- AsyncAPI 3.x specification — https://www.asyncapi.com/docs/reference/specification/v3.0.0
- OpenAPI 3.1 specification — https://spec.openapis.org/oas/v3.1.0
- Protobuf 3 language guide — https://protobuf.dev/programming-guides/proto3/
- Avro 1.x specification — https://avro.apache.org/docs/1.11.1/specification/
- Buf BSR — https://buf.build/product/bsr
- EventCatalog — https://www.eventcatalog.dev/
- ADR-0005 — eventing backbone + outbox pattern.
- ADR-0011 — cross-microservice contract registry (this ADR is the runtime substrate).
- ADR-0029 — connect dual-context architecture.
- ADR-0100 — supervisor public-contract lean-A10 lane (silent-regression invariant; this ADR closes).
- ADR-0105 — 13-layer enum + check-family patterns.
- ADR-0128 — hyperscaler architecture invariants.
- ADR-0131 — per-µservice flat layout (contracts/ subdir is part of canonical layout).
- ADR-0145 — inter-µservice communication reform (proto3 + OpenAPI 3.1 surfaces).
- ADR-0148 — Istio service mesh.
- ADR-0157 — api-gateway tier (consumes OpenAPI schemas).
