---
id: ADR-0011
status: Superseded
superseded_by: [ADR-701]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0011: Cross-microservice contract registry — `contracts/microservice-contracts.yaml` source-of-truth, openapi/proto/asyncapi sub-directories, oya-check-contracts CI lane, cross-microservice contract change protocol, auto-generated SDKs

> **Status:** Accepted
> **Owner:** `oya-foundry` (registry surface) + `council-architecture`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — "axis" terminology replaced with "microservice")
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0010, ADR-0015, ADR-0019, ADR-0058

---

## Context

The cohesion thesis (ADR-0001) hinges on cross-microservice contracts being mechanical artifacts, not slide-deck text. DESIGN §10 enumerates ~25 cross-microservice contract rows (Tenant kernel, Identity / RBAC, Capability invocation, Autonomy ceiling policy, Audit-chain event, Capability registry record, Plane class, Cloud resource type, Region/AZ/Cell, IAM/SSO, Ontology property tier, Search index lifecycle, Ad slot inventory, Billing event, DSR cascade, Webhook delivery, Public REST stability tier, Marketplace listing, Eventing backbone, Cloud↔Search capacity & residency, Search↔Ads SERP & query privacy, Foundry↔Cloud mutation control, Foundry↔Search retrieval boundary, Tenant↔Ads/Analytics eligibility, Revenue/metering/tax invoice). Without a single registry, contracts drift independently and the cohesion claim degrades.

---

## Decision

We adopt **`contracts/microservice-contracts.yaml`** as the source-of-truth registry of cross-microservice contracts, with sub-directories for protocol-specific specs, a gating CI lane, an explicit cross-microservice contract change protocol, and auto-generated multi-language SDKs.

### Registry layout

```
contracts/
  microservice-contracts.yaml        # source of truth: every cross-microservice contract row
  openapi/<contract-id>.yaml         # public REST + webhook contracts
  proto/<contract-id>.proto          # gRPC + service-to-service contracts
  asyncapi/<contract-id>.yaml        # event topic + schema contracts (CloudEvents per ADR-0005)
  cedar/<contract-id>.cedar          # Cedar policy contracts (ADR-0007)
  schemas/<contract-id>.json         # JSON Schema contracts (capability records, catalog records)
  sdks/                              # generated; never hand-edited
    rust/<contract-id>/
    typescript/<contract-id>/
    python/<contract-id>/
    go/<contract-id>/
```

### `microservice-contracts.yaml` shape

```yaml
contracts:
  - id: TENANT_KERNEL
    owner_microservice: tenancy
    consumer_microservices: [connect, ontology, foundry, cloud, search, ads, medical, payments]
    surface_kind: kernel-type
    source_of_truth: crates/oya-tenancy-kernel
    spec_paths: [proto/tenant-kernel.proto, schemas/tenant-record.json]
    plane: control
    review_class: cross-microservice-tenant-mutation
    required_reviewers: [council-architecture, council-privacy]
    audit_emission_topic: oya.tenancy.kernel-mutation.v1
    sdk_languages: [rust, typescript, python, go]

  - id: CAPABILITY_INVOCATION
    owner_microservice: foundry
    consumer_microservices: [tenancy, connect, ontology, cloud, search, ads, medical, payments]
    surface_kind: rest-api
    source_of_truth: crates/oya-intelligence-runtime-rest
    spec_paths: [openapi/capability-invocation.yaml]
    plane: control
    review_class: cross-microservice-capability
    required_reviewers: [council-foundry, consuming-microservice-team]
    audit_emission_topic: oya.foundry.capability.invoked.v1
    sdk_languages: [rust, typescript, python, go]

  - id: ONTOLOGY_PROPERTY_TIER
    owner_microservice: ontology
    consumer_microservices: [search, ads, medical, payments, connect, hr, payroll]
    surface_kind: kernel-type
    source_of_truth: crates/oya-ontology-entity-kernel
    spec_paths: [proto/ontology-entity.proto, schemas/ontology-property-tier.json]
    plane: data
    review_class: cross-microservice-ontology
    required_reviewers: [council-architecture, consuming-microservice-team]
    audit_emission_topic: oya.ontology.entity.mutated.v1
    sdk_languages: [rust, typescript, python, go]

  - id: AUDIT_CHAIN_EVENT
    owner_microservice: audit-chain
    consumer_microservices: [tenancy, connect, ontology, foundry, cloud, search, ads, medical, payments]
    surface_kind: event
    source_of_truth: crates/oya-audit-chain-kernel
    spec_paths: [asyncapi/audit-chain.yaml]
    plane: control
    review_class: cross-microservice-audit
    required_reviewers: [council-architecture]
    audit_emission_topic: oya.audit-chain.event.v1
    sdk_languages: [rust, typescript, python, go]
  # ... (~25 rows total per DESIGN §10)
```

### CI lane: `oya-check-contracts`

The lane runs on every PR and:

1. Validates every contract row's `source_of_truth` crate exists and exports the named types.
2. Validates every spec path exists and is syntactically valid (OpenAPI 3 / Protobuf / AsyncAPI 2.6 / Cedar / JSON Schema).
3. Detects breaking changes per protocol (`oasdiff` for OpenAPI, `buf breaking` for Protobuf, AsyncAPI delta for events).
4. Verifies the PR's labels include `cross-microservice-<contract-id>` whenever the spec changes.
5. Hard-fails any contract change PR whose `## Code Review` block lacks the `required_reviewers` set.
6. Regenerates SDKs in `contracts/sdks/` and verifies the generated tree is committed.

### Cross-microservice contract change protocol

A PR that touches any contract row MUST:

- Carry the `cross-microservice-<contract-id>` label.
- Include a regenerated diff of the affected SDK in the PR's `## Evidence` section.
- Carry sign-off from the `required_reviewers` set.
- Emit `EVT-CROSS-MICROSERVICE-CONTRACT-CHANGED` to the audit chain (ADR-0003) at merge time.
- Cite the contract row id in the PR's `## Traceability` section.

Breaking-change protocol (per ADR-0019 deprecation governance):
- Wave N: introduce v(n+1) alongside v(n); both shipped.
- Wave N+1: deprecation-warning emission on v(n) consumers.
- Wave N+2: v(n) removal.

### SDK generation auto from registry

The registry is the single input to per-language SDK codegen (`oya-intelligence-sdk-gen-*`). SDKs are committed under `contracts/sdks/<lang>/<contract-id>/`. Hand-edited SDKs are rejected by the lane.

---

## Consequences

### Positive

- The registry becomes the auditable cohesion artifact.
- SDK consumers pull a versioned, signed contract; drift surfaces as a build break.
- Auditor evidence: contract-change history generated per release.

### Negative

- Promotion bar for microservice-internal → cross-microservice is real; some PRs become contract-level events.
- Codegen pipelines for four SDK languages add CI time; mitigated by per-PR-affected SDK regen only.

### Operational

- On-call: `EVT-CONTRACT-LANE-FAIL-RATE > N` daily rollup.
- Runbooks: `runbooks/contract-introduction.md`, `runbooks/contract-breaking-change.md`.
- Planned enforcement: `oya-check-contracts` is an advisory P0 lane reference until the crate exists; active merge blocking stays with shipped gates.
- Per-quarter audit by `council-architecture`: every contract row reviewed; orphan rows removed.

---

## Related

- ADR-0001 (cohesion — cross-microservice contracts enforce the moat)
- ADR-0002 (Tenant kernel as registry row)
- ADR-0003 (audit emission per contract change)
- ADR-0006 / ADR-0055 (Ontology property tier as contract row)
- ADR-0058 (Flat microservice catalog — contract registry spans all µservices)
- ADR-0059 (Workflow + Ontology — Workflow event contracts + Ontology entity contracts are registry rows)
