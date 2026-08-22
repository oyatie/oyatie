---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud + gtm-customer-success
deciders: axis-cloud, council-architecture
related_adrs: [ADR-0121, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml
  - microservices/cloud-k8s/contracts/proto/cloud-k8s.proto
  - microservices/cloud-k8s/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (cloud-k8s µservice)

## Purpose

Two SDK surfaces are required:

1. **Operator + Foundry-agent SDK** — internal callers (oyatie operators, Foundry agents, sibling µservices) invoke cloud-k8s capabilities programmatically. Rust SDK at M01; TS / Python / Go follow per below.
2. **Tenant SDK (limited)** — tenants do not directly mutate clusters (operator-only domain). Tenants get a **read-only SDK** for querying their own namespace's cluster state (NetworkPolicy status, PodDisruptionBudget conformance, mesh AuthorizationPolicy effective rules). Full write access remains operator-only.

The base Kubernetes API (kubectl, helm, client-go, etc.) is **not** wrapped by oyatie SDKs — tenants who need raw kubectl access use the standard upstream tooling, with all calls routed through the `kubernetes-api-proxy` (which applies Cedar + audit-chain transparently).

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary) | First-party authored (`cloud-k8s-cluster-bootstrap-sdk`, `cloud-k8s-kubernetes-api-proxy-sdk` crates) | axis-cloud |
| **TypeScript** | M01+1 | OpenAPI-generated + first-party wrapper; npm | axis-cloud + gtm |
| **Python** | M02 | OpenAPI-generated; PyPI | axis-cloud + gtm |
| **Go** | M02 | gRPC-generated + wrapper; go-module | axis-cloud + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated + wrapper; Maven Central | axis-cloud + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; NuGet | axis-cloud + gtm |

Prioritisation: oyatie's own µservice languages first (Rust); then largest tenant developer-population (TS + Python lead).

## Generation Strategy

### Rust SDK (first-party)

Two crates per BC where SDK applies:
- `cloud-k8s-cluster-bootstrap-sdk` — operator-facing read + write
- `cloud-k8s-kubernetes-api-proxy-sdk` — tenant-facing read of own namespace state

Public surface:
- `Client::new(opts) -> Client`
- `client.get_cluster(id) -> Result<Cluster, _>`
- `client.list_nodes(cluster_id) -> Result<Vec<Node>, _>`
- `client.bootstrap_cluster(req) -> Result<BootstrapResp, _>` (operator-only; Cedar enforces)
- `client.apply_network_policy(req) -> Result<ApplyResp, _>` (operator + tenant-PR-derived; Cedar enforces)
- `client.stream_cluster_events(cluster_id) -> impl Stream<Item=ClusterEvent>` via gRPC streaming

Auth: `Client` accepts OIDC token provider (closure / trait impl). For tenant SDK: bound to a tenant at construction; `X-Scope-Tenant` header auto-populated.

Retry policy: exponential backoff for transient 5xx and 429.

No `unsafe`; `#![deny(unsafe_code)]`.

Published to oyatie internal crate registry.

### Generated SDKs (TS / Python / Go / JVM / C#)

Generation pipeline at `microservices/cloud-k8s/sdk-generation/`:

1. Source of truth: `contracts/openapi/cloud-k8s.yaml` (REST) + `contracts/proto/cloud-k8s.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x.
3. Proto → language: `protoc` + per-language plugin.
4. Hand-authored thin wrapper provides:
   - First-class auth helpers (OIDC token provider abstraction).
   - Tenant-context binding at client construction.
   - Retry + circuit-breaker matching Rust SDK behavior.
   - Idiomatic naming + error handling per language convention.
5. Per-language SDK ships with README + quick-start; versioning matches cloud-k8s major.minor; compatibility matrix.
6. Per-language CI lane: build + lint + integration-test against staging cluster.

## Public Surface (across languages)

| Capability | Method | Returns | Audience |
|---|---|---|---|
| Read cluster state | `getCluster(id)` | `Cluster` | operator + tenant-namespace-scoped |
| List clusters in pack | `listClusters(pack)` | `Cluster[]` | operator |
| Bootstrap cluster | `bootstrapCluster(req)` | `BootstrapResp` | operator (T3 + 2-person) |
| Upgrade cluster | `upgradeCluster(req)` | `UpgradeResp` | operator (T3 + 2-person) |
| Snapshot etcd | `snapshotEtcd(cluster_id)` | `SnapshotResp` | operator |
| Read bootstrap evidence | `getBootstrapEvidence(cluster_id)` | `BootstrapEvidence` | operator + auditor |
| List / get / add / cordon / drain / remove node | per-method | per-type | operator (T2) |
| List / apply NetworkPolicy / AuthorizationPolicy | per-method | per-type | operator + tenant-PR-derived (T2) |
| Stream cluster events | `streamClusterEvents(...)` | stream | operator + observability |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue tenant API key (read-only-on-own-namespace) via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start (per language) | gtm-customer-success |
| Sample workflow: NetworkPolicy authoring via git PR + verify via SDK | axis-cloud |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-cloud |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of operator/tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib deprecated upstream | dep-deprecated | 12mo advance notice + auto-migration |
| Breaking API change in cloud-k8s µservice | per-release | major version bump in SDK; backwards-compat adapter for 1 prior major |

Per ADR-#### deprecation discipline: every SDK sunset emits ADR-shaped notice + deprecation-warning + tenant comms.

## Versioning

cloud-k8s µservice version: semver.
SDK version per language: matches cloud-k8s major.minor; SDK patch independent.
Compatibility matrix: published per-language; CI lane verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API stable in production ≥ 6mo. Default: keep closed-source until tenant-driven open-source request. Open-source-when-stable matches Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against cloud-k8s versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused flagged for sunset.

## References

- `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`.
- `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- `microservices/cloud-k8s/PRD.md` BC layer mapping (`-sdk` crate per ADR-0105).
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC tooling — `grpc.io`.
- Stripe + Twilio SDK precedents — `stripe.com/docs/libraries` + `twilio.com/docs/libraries`.
