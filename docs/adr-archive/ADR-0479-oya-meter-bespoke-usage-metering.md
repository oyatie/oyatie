---
id: ADR-0479
title: "oya-meter — bespoke Rust usage metering substrate"
status: Superseded
date: 2026-05-28
authority: founder
owner: axis-cloud
planning_impact: true
supersedes: [ADR-0429]
superseded_by: [ADR-701]
amended_by: [ADR-0632]
milestone: M-METERING-V2
related: [ADR-0429, ADR-0478, ADR-0193, ADR-0397, ADR-0083, ADR-0411, ADR-0403, ADR-0420, ADR-0449, ADR-0131, ADR-0132, ADR-0509]
---
# ADR-0479 — oya-meter: bespoke Rust usage metering substrate

## Status

Accepted — 2026-05-28. Founder-locked. Supersedes ADR-0429 (OpenMeter Phase-1 stepping stone).

## ADR-0632 product-protocol reconciliation

Tenants query usage through public HTTPS REST documented by OpenAPI 3.2.0; signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, and WebSocket remain the allowed public delivery and streaming carriers. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. Meter-to-billing and other sibling-service calls may use internal-only gRPC/proto3 over HTTP/2; public SDKs are generated from OpenAPI, not Connect or Protobuf.

## Context

AWS, GCP, and Azure all operate bespoke usage-metering substrates internally; none depend on a
third-party SaaS metering product at platform scale. OpenMeter (ADR-0429) was adopted as a
Phase-1 stepping stone to unblock tenant billing. It has served that purpose. Three forces now
justify replacing it with a bespoke substrate:

1. **Bounded complexity**: the metering surface for oyatie's tenant model (token counts, compute
   seconds, storage bytes, message counts, cache hits) is well-defined and stable. A bespoke
   implementation fits in ~4–6 months of focused Rust engineering.
2. **Rust doctrine**: every data-path substrate in oyatie is Rust-native (ADR-0131/0132 flat µservice
   model, Axum/Tokio blessed stack). A Go or managed-service dependency at the metering tier
   creates a seam that complicates supply-chain control, Cedar integration, and OTel emission.
3. **Hyperscaler pattern alignment**: AWS Metering Service and GCP Usage Tracking are internal
   primitives, not wrappers around commercial vendors. Controlling the metering kernel lets oyatie
   enforce per-tenant isolation, Cedar-gated aggregate APIs, and Pulsar-native ingest without an
   impedance mismatch layer.

## Decision

Ship `microservices/oya-meter/` as a single-concern Rust µservice (ADR-0131 flat layout, ADR-0132
no-suite). OpenMeter is retired as an active dependency (ADR-0429 → Superseded).

### D1 — µservice scaffold

`microservices/oya-meter/` — Rust workspace, Axum public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2. **ClickHouse** (ADR-0193) is the
time-series usage backend; **PostgreSQL** is the meter-catalog store (tenant meter definitions,
resource dimension registry).

### D2 — Usage event ingest

Consume typed CloudEvents (ADR-0403) from Pulsar (ADR-0397). Event topic schema:
`usage.{tenant}.{resource}.{action}.v1`. Per-µservice meters:

| Resource | Action | Unit |
|---|---|---|
| cloud-intelligence | token | token |
| vllm | inference | second |
| seaweedfs | storage | byte |
| pulsar | message | count |
| buck2-cache | hit | count |

### D3 — Aggregation engine

Time-window aggregates (1m / 5m / 1h / 1d) computed via **Polars** (ADR-0420) materialized
streams written to ClickHouse. **Cedar** (ADR-0083) gates per-tenant aggregate API: tenants may
only query their own namespace; operator realm has cross-tenant read.

### D4 — Crossplane provisioning

**Crossplane** (ADR-0411) TenantApplication XR provisions per-tenant meter namespaces on tenant
onboarding. Feeds **oya-billing** (ADR-0478) billable-metrics via a well-typed internal-only gRPC/proto3 over HTTP/2 surface.

### D5 — Tenant usage API

Tenants query their own usage via public HTTPS REST documented by OpenAPI; SDK auto-generated via **Kiota** (ADR-0449). Rate: tenant
aggregate queries at `/usage/v1/{tenant_id}/aggregates`; raw event replay at
`/usage/v1/{tenant_id}/events` (operator only).

## Hyperscaler-lens

| Check | Result |
|---|---|
| Active upstream (all deps) | ✅ Axum/Tokio/Polars/ClickHouse/Pulsar all active |
| Clean license | ✅ Apache-2 / MIT throughout |
| Fully self-hostable | ✅ No managed-service dependency |
| Hyperscaler-internal equivalent | ✅ AWS Metering Service / GCP Usage Tracking |

## Alternatives Considered

| Alternative | Reason rejected |
|---|---|
| Retain OpenMeter (ADR-0429) | Go runtime seam; no Cedar integration; managed-service coupling risk at scale; Phase-1 stepping stone fulfilled its purpose |
| Metronome / Amberflo (commercial SaaS) | Violates self-hostable invariant; no sovereign-cloud / air-gap path |
| Extend cloud-billing (ADR-0478) with metering | Violates ADR-0132 single-concern; metering throughput profile (high-volume ingest) is different from billing ledger (low-volume settlement) |

## Consequences

- OpenMeter containers are removed from the deployment manifests after oya-meter D1–D3 pass
  acceptance gates.
- oya-billing (ADR-0478) switches its billable-metrics source from OpenMeter to oya-meter internal gRPC/proto3
  once D4 is accepted.
- Polars (ADR-0420) materialized-stream pattern is the canonical aggregate computation path; no
  separate streaming-SQL engine required.
- Kiota SDK (ADR-0449) generates the tenant-facing client from OpenAPI, never from an internal Protobuf contract.

## Integration

- **Pulsar** (ADR-0397): oya-meter is a Pulsar consumer on `usage.*` topics.
- **ClickHouse** (ADR-0193): time-series write path; aggregate query path.
- **Cedar** (ADR-0083): per-tenant namespace gate on aggregate API.
- **Crossplane** (ADR-0411): TenantApplication XR provisions meter namespace.
- **oya-billing** (ADR-0478): consumes billable-metrics from oya-meter.

## Promotion Rationale

Bounded scope (~4–6 months), Rust-native, no external managed-service dependency, direct
equivalent to AWS/GCP internal metering primitives, Cedar-native authorization — all five
hyperscaler gates pass. Founder-locked 2026-05-28.

## Implementation pattern (ADR-0509 alignment)

Per ADR-0509 (Hyperscaler service decomposition pattern), `oya-meter` ships as **single-crate-per-service with mod-based subsystems**. Per-use-case crate sprawl is superseded. Use cases remain valid as domain concepts (subsystem boundaries inside `src/<subsystem>/`).
