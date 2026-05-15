---
id: ADR-0037
status: proposed
sunset_topic: adr-0037-public-api-deprecation-doctrine
sunset_milestone: doctrine-not-time-bounded
---

# ADR-0037: Public API stability tiers — preview / stable / GA with semver-diff PR gate, contract-first SDK generation, per-deprecation telemetry

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0011, ADR-0033, ADR-0036, ADR-0038, ADR-0042, ADR-0050

---

## Context

Every axis ships APIs. Tenants and ISVs build against those APIs. Without a structured stability tier and deprecation discipline, every API evolution becomes a per-tenant compatibility break, every minor version becomes a customer-support incident, and the cohesion thesis ("one product across all microservices") collapses into a per-microservice API governance debate.

The pack-of-19 foundation ADRs named API stability as a need but did not pin the tier vocabulary, the per-PR semver-diff gate, the contract-first SDK generation pipeline, or the per-deprecation telemetry surface. This ADR pins them so that an API consumer (tenant developer, ISV, internal-axis caller) can read a single contract, see its tier, see its deprecation timeline, and consume a per-tier SDK without bespoke versioning conversations.

---

## Decision

We adopt **three stability tiers** (preview / stable / GA), a **per-PR semver-diff gate** that classifies every API change, **contracts-first artifacts** at `contracts/`, **auto-generated SDKs** per language, **per-deprecation event emission** to the audit chain, and a **trust-portal mirror** of stability tiers visible to tenants.

### Tier vocabulary

| Tier | Breaking-change policy | Deprecation lead time | SLA |
|---|---|---|---|
| **preview** | Breaking changes allowed without deprecation | None — caller assumes risk | None |
| **stable** | Semver: minor = additive, major = breaking | 6 months minimum | 99.9% |
| **GA** | Semver: minor = additive, major = breaking, per-endpoint deprecation telemetry mandatory | 12 months minimum | 99.95% |

### Per-PR semver-diff gate

Every PR that touches a contract artifact runs `oya contract-diff` (lane: `oya-foundry-fitness-api-semver`). The tool classifies the diff:

```rust
// crates/oya-shared-semver-check-cli
pub enum SemverDiff {
    /// Additive change (new endpoint, new optional field, new enum value at end)
    Minor { additions: Vec<Addition> },
    /// Breaking change (removal, type change, required-field add, enum reorder)
    Major { breaks: Vec<BreakingChange> },
    /// Documentation-only / non-functional
    Patch,
}
```

PR labels follow the diff classification (`api-minor`, `api-major`, `api-patch`). For tier-`stable` and tier-`GA` contracts, an `api-major` label requires (a) an ADR amendment in this pack or downstream, (b) a per-endpoint deprecation timeline declared, (c) reviewer pair from `council-architecture`.

### Contracts at `contracts/`

```
contracts/
  openapi/                # REST APIs (OpenAPI 3.2)
    workspace/
      mail-v1.yaml
      calendar-v1.yaml
    vertical/
      healthcare-v1.yaml
    cloud/
      compute-v1.yaml
  proto/                  # gRPC APIs (Protocol Buffers)
    foundry/
      capability-v1.proto
    workflow/
      workflow-v1.proto
  asyncapi/               # Event APIs (AsyncAPI 3.0)
    audit/
      audit-events-v1.yaml
    workflow/
      workflow-events-v1.yaml
  graphql/                # GraphQL surfaces
    workspace/
      drive-v1.graphql
```

Per-contract `meta.yaml` declares: tier, owner, sunset, related ADRs.

### SDK auto-generation

Per language, an SDK is auto-generated from contracts:

| Language | SDK package | Generator |
|---|---|---|
| Rust | `oya-sdk-rust` | per-contract derive macros + `prost` (gRPC) + `oapi-codegen-rs` (REST) |
| TypeScript | `oya-sdk-ts` | `openapi-typescript` + `@bufbuild/protobuf` |
| Python | `oya-sdk-py` | `openapi-python-client` + `grpcio-tools` |
| Go | `oya-sdk-go` | `oapi-codegen` + `protoc-gen-go-grpc` |
| Java | `oya-sdk-java` | `openapi-generator-cli` + `protoc` (J2 GA tier only) |

SDK release cadence matches contract release cadence. SDKs at preview tier are published to a separate channel (e.g. npm `@oya-preview` scope).

### Per-deprecation event emission

Every deprecated endpoint emits a `DeprecationUsed` event to the audit chain (per ADR-0003) on each call:

```rust
pub struct DeprecationUsed {
    pub endpoint: EndpointId,
    pub deprecated_at: DateTime<Utc>,
    pub sunset_at: DateTime<Utc>,
    pub caller: CallerIdentity,
    pub replacement: Option<EndpointId>,
}
```

Per-tenant deprecation report aggregates the events; tenant admin sees which deprecated endpoints they are still calling and the sunset date.

### Trust-portal mirror

Per ADR-0038, every tenant's trust portal shows:

- Per-API tier (preview / stable / GA).
- Per-endpoint deprecation status (with sunset date).
- Per-tenant deprecation usage summary.
- Per-API SLA + recent uptime.

### Per-tier change-management process

#### Preview tier

- Add / change / remove freely.
- Must be flagged in contract `meta.yaml` as `tier: preview`.
- Cannot be promoted to stable without (a) removal of all `unstable` markers, (b) ADR documenting the promotion, (c) at least one external consumer using it for ≥ 30 days, (d) per-endpoint SLA observed for ≥ 30 days.

#### Stable tier

- Additions are minor (semver bump only).
- Breaking changes require ADR + 6-month deprecation + per-endpoint deprecation telemetry.
- Promotion to GA: ADR + ≥ 90-day stable observation + per-endpoint SLA observed + customer migration path documented.

#### GA tier

- Additions are minor.
- Breaking changes require ADR + 12-month deprecation + per-endpoint deprecation telemetry + tenant migration tooling shipped.
- GA endpoints cannot be removed; only deprecated → eventually superseded.

### API versioning convention

- **URL-path versioning for REST** (`/api/v1/`, `/api/v2/`).
- **Package-versioning for gRPC** (`oya.foundry.v1.CapabilityService`, `v2`).
- **Schema-version field for AsyncAPI events** (`schema_version: "1.0"`).
- **GraphQL deprecation directive** (`@deprecated(reason: "..., use X")`).

A major-version bump is a parallel deployment, never an in-place rewrite. Old + new coexist for the deprecation window.

### Per-axis API ownership

Each axis owns its API contracts:

| Axis | Contracts owner |
|---|---|
| SaaS platform | `foundry` |
| Workspace | `axis-workspace` |
| Vertical | `axis-vertical` (per-vertical sub-owners) |
| Foundry | `foundry` |
| Cloud | `cloud` |
| Search | `axis-search` |
| Ads/Analytics | `axis-ads-analytics` |

Cross-axis contracts (e.g. Search↔Foundry RAG endpoint) are co-owned + reviewed by `council-architecture`.

### Anti-scope

This ADR does not define internal-only API governance (those follow lighter rules per axis). It does not define plugin APIs (per ADR-0036). It does not define the audit chain event schema (per ADR-0003).

---

## Consequences

### Positive

- A single, mechanical tier vocabulary lets tenants and ISVs make informed compatibility decisions.
- Per-PR semver-diff gate catches accidental breaking changes before merge.
- Contract-first + auto-SDK guarantees the SDK is always consistent with the contract.
- Per-deprecation telemetry gives both us and our tenants visibility into migration progress.
- Trust-portal mirror surfaces stability commitments to the customer-facing side.

### Negative

- Three-tier vocabulary requires discipline to maintain — preview can become a long-term home for "we'll get to it eventually" APIs.
- 12-month GA deprecation is long; we cannot retire a GA endpoint quickly even when its replacement is ready.
- Per-language SDK maintenance is a real recurring cost.
- Semver-diff classification has edge cases; council-architecture reviews ambiguous PRs.

### Operational

- Per-axis API stability dashboard; per-tier endpoint count + recent diff classification.
- Per-PR `oya-foundry-fitness-api-semver` lane gating.
- Per-quarter API sunset queue review by `council-architecture`.
- Per-tenant deprecation report mailed monthly (Workspace mail per ADR-0029).
- Per-SDK CI lane runs against per-tier contracts; SDK release blocks if contract diff is unclassified.

---

## Alternatives considered

### Alternative A — Two tiers (stable / experimental)

- **Pros:** simpler vocabulary.
- **Cons:** nowhere to land "we ship this with strong commitments but reserve breaking changes for major" — which is exactly the GA tier; collapsing GA into stable lengthens stable-tier semantics inappropriately.
- **Rejected because:** the GA tier is a meaningful commitment level we want to make.

### Alternative B — Calendar-versioned APIs (e.g. `/api/2026-05-01/`)

- **Pros:** unambiguous version identity.
- **Cons:** doesn't express compatibility (a new calendar version may or may not be compatible); SDKs can't auto-detect compatibility.
- **Rejected because:** semver expresses compatibility, calendar versioning expresses age.

### Alternative C — Per-axis stability vocabulary (each axis defines its own tiers)

- **Pros:** microservice-team flexibility.
- **Cons:** consumers see N vocabularies; cohesion violated.
- **Rejected because:** the cohesion thesis applies to API governance.

### Alternative D — No formal tiers; every API is "best effort"

- **Pros:** no overhead.
- **Cons:** consumers cannot plan; ISVs cannot commit; the deprecation discipline doesn't form.
- **Rejected because:** API stability is a primary moat for the ecosystem.

---

## Open questions

1. **Q1.** Per-tier SLA target — GA at 99.95% or 99.99%? Default: 99.95% at GA; per-endpoint elevation possible. → ADR-0042.
2. **Q2.** Java SDK at GA, or defer? Default: defer to W+12; Rust + TS + Python + Go cover 95% of demand. → owner: `council-architecture`.
3. **Q3.** Per-deprecation per-tenant migration tool requirement (mandatory at GA breaking change)? Default: yes for GA; advisory for stable. → owner: `council-architecture`.
4. **Q4.** Cross-axis contract registry surface to ISVs — readable, write-restricted? Default: read-only at GA; ISV proposals via per-ADR PR. → ADR-0011.
5. **Q5.** GA endpoint sunset can extend to 24 months for KR public-sector / regulated-industry tenants? Default: yes, per-tenant negotiated, not default. → ADR-0034.

---

## References

- `docs/PRD.md` §7 (API surface), §11 (per-tenant SDK)
- `docs/DESIGN.md` §10 (cross-microservice contracts), §11 (cross-microservice contradictions)
- OpenAPI 3.2 spec; AsyncAPI 3.0 spec; Protocol Buffers; GraphQL spec
- Semver 2.0.0 spec
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0011 (capability registry), ADR-0033 (vertical pack), ADR-0036 (plugin substrate), ADR-0038 (trust portal), ADR-0042 (observability), ADR-0050 (automation pipeline)
