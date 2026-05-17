---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cell-substrate + gtm-customer-success
deciders: axis-cell-substrate, council-architecture
related_adrs: [ADR-0130, ADR-0131]
related_artifacts:
  - microservices/cell/contracts/openapi/cell.yaml
  - microservices/cell/contracts/proto/cell.proto
  - microservices/cell/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (cell µservice)

## Purpose

Workload µservices on the hot path need to resolve their tenant's cell-assignment in p99 ≤ 50ms. First-party SDKs in oyatie's languages (Rust + TS + Python + Go + JVM) provide:

- Auth (OIDC + per-cell SVID).
- In-process LRU cache with 60s TTL + event-driven invalidation (subscribe to `CellAssigned` + `CellRebalanced`).
- Tenant-scoped client construction (binds to tenant at construction; cannot pivot).
- Retry policy + circuit-breaker.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; oyatie's own language) | First-party (`oya-cell-{cell-registry,tenant-assignment}-sdk` crates) | axis-cell-substrate |
| **TypeScript** | M01+1 (workload µservices increasingly TS) | OpenAPI-generated baseline + ergonomic wrapper | axis-cell-substrate + gtm |
| **Python** | M02 | OpenAPI-generated | axis-cell-substrate + gtm |
| **Go** | M02 | gRPC-generated baseline + ergonomic wrapper | axis-cell-substrate + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated baseline + ergonomic wrapper | axis-cell-substrate + gtm |
| **C# / .NET** | M03+ | OpenAPI-generated | axis-cell-substrate + gtm |
| **Ruby / PHP** | none — no current demand | n/a | n/a |

Drivers: oyatie's own µservice languages first; workload µservices' actual language mix.

## Generation Strategy

### Rust SDK (first-party)

Per BC:
- `microservices/cell/src/crates/oya-cell-cell-registry-sdk/`: client for cell-registry read API + streaming.
- `microservices/cell/src/crates/oya-cell-tenant-assignment-sdk/`: client for tenant-assignment lookup + migration-plan read.

Public surface:

```rust
let client = CellAssignmentClient::new(opts)?;          // tenant-bound
let assignment = client.get_assignment(tenant_id).await?;  // cache-hit ≤ 5ms; cache-miss ≤ 50ms
let mut stream = client.stream_assignment_changes(tenant_id).await?;
while let Some(evt) = stream.next().await { /* ... */ }
```

Properties:
- Authentication: `Client` accepts an OIDC token provider OR per-cell SVID via mTLS.
- Tenancy: Bound at construction; `X-Scope-OrgID` populated automatically.
- Cache: LRU 60s TTL + event-driven invalidation (subscribes to `CellAssigned` + `CellRebalanced` topics).
- Retry: exponential backoff for transient 5xx and 429.
- No `unsafe`; `#![deny(unsafe_code)]`.
- Re-exports types from `*-kernel` crates for consistent shapes.
- Published to oyatie's internal registry.

### Generated SDKs (TS / Python / Go / JVM / C#)

Generation pipeline at `microservices/cell/sdk-generation/`:

1. Source of truth: `contracts/openapi/cell.yaml` (REST) + `contracts/proto/cell.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer providing:
   - First-class auth helpers (OIDC token provider abstraction).
   - Tenant binding at construction.
   - LRU cache with event-driven invalidation.
   - Retry + circuit-breaker matching Rust SDK behavior.
   - Idiomatic naming per language convention.
5. Each SDK ships with: README + quick-start + versioning matrix + open-source decision.
6. Per-language CI lane: build + lint + integration test against staging cell substrate.

## Public Surface

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Read assignment | `getCellAssignment(tenant_id)` | `CellAssignment` |
| Stream assignment changes | `streamCellAssignmentChanges(tenant_id, since)` | streaming `CellAssignment` |
| Read migration history | `getMigrationHistory(tenant_id)` | `MigrationPlan[]` |
| Read cell metadata (own bound cell) | `getCell(cell_id)` | `Cell` |
| Read placement decision | `requestPlacementDecision(...)` | `PlacementDecision` (scheduler-internal; restricted) |

Cell lifecycle + decommission methods are NOT exposed in tenant-facing SDKs — those are operator-only. SDKs include a clearly-marked operator-scope namespace for ops use cases.

## Workload SDK Onboarding

| Step | Owner |
|---|---|
| Issue per-cell SPIFFE SVID for workload µservice via SPIRE | ops-security |
| Provide workload µservice template with SDK quickstart (per language) | axis-cell-substrate + gtm |
| Sample workflow: subscribe to CellAssigned + CellRebalanced; invalidate own cache | axis-cell-substrate |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-cell-substrate |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% workload usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice + auto-migration if possible |
| Breaking API change in cell µservice | per-release | major version bump in SDK; backwards-compatible adapter for 1 prior major version |

## Versioning

Cell µservice version: semver.
SDK version per language: matches cell major.minor; SDK patch independent.
Compatibility matrix published per-language; CI lane verifies SDK against current + 1 prior major.

Example:
- cell v1.2.0 + Rust SDK v1.2.0 + TS SDK v1.2.0 → all compatible.
- cell v2.0.0 + Rust SDK v1.2.0 → compatible via backward-adapter for 12mo after v2 launch.
- cell v2.0.0 + Rust SDK v0.9.0 → INCOMPATIBLE; upgrade required.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API has been stable in production for ≥ 6mo. Default: closed-source until tenant-driven or competitive reason makes open-source the right move.

## Verification

- Per-SDK CI lane: build + lint + integration test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against cell versions N-1, N, N+1.
- Annual SDK telemetry review.

## References

- `microservices/cell/contracts/openapi/cell.yaml`.
- `microservices/cell/contracts/proto/cell.proto`.
- `microservices/cell/PRD.md` BC layer mapping.
- ADR-0105 (13-layer; sdk is canonical).
- OpenAPI Generator; gRPC tooling.
- Stripe SDK; Twilio SDK precedents.
