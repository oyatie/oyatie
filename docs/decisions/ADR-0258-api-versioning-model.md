---
id: ADR-0258
status: Accepted
doc_status: published
date: 2026-05-20
deciders: council-architecture, axis-foundry, axis-cloud, axis-all-microservices, ops-sre-reliability
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
amended_by: [ADR-0565]
related: [ADR-0011, ADR-0037, ADR-0064, ADR-0131, ADR-0145, ADR-0150, ADR-0203, ADR-0244, ADR-0250, ADR-0565]
related_specs:
  - /specs/microservices/manifest-schema.json
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/api-versioning-contract.json
retires_feedback_memory: []
sunset_topic: adr-0258-api-versioning-model
sunset_milestone: doctrine-not-time-bounded
tier: tier-1-lockdown
authority_chain: council-architecture
---

# ADR-0258 — API Versioning Model (Stripe-style request-time pinning for public, URL versioning for internal mesh)

## Status

Accepted (2026-05-20). Tier-1 lockdown ADR. Closes the "API versioning model" gap left open by ADR-0037 (which set tier vocabulary but did not pin the canonical version-negotiation algorithm, the per-tenant pinning override, the per-µservice independent cadence, or the SDK auto-generation pipeline).

This ADR is BINDING on every µservice that exposes a public REST/OpenAPI, webhook, AsyncAPI event, or streaming surface and on every internal mesh interface that crosses a µservice boundary. GraphQL is historical rejected context only; ADR-0565 removed it from the owned surface set.

## ADR-0203 public-contract reconciliation

ADR-0203 fixes the public documentation and contract boundary at OpenAPI 3.2 REST plus AsyncAPI
3.1 event, webhook, and streaming references. Public gRPC or proto3 exposure is not authorized.
The gRPC package and URL-versioning rules below remain binding only for internal service-to-service
RPC under mTLS; displaying internal Protobuf descriptors does not create a public contract.

## Context

oyatie ships a hyperscaler-grade API surface: the public REST/OpenAPI, webhook, AsyncAPI event, and streaming surface (Workspace, Cloud, Foundry, Verticals, Connect, Search) and the internal mesh surface (µservice ↔ µservice gRPC under mTLS, per ADR-0145). GraphQL is not an owned surface under ADR-0565. Both surfaces evolve continuously. Without a single canonical versioning model:

1. Tenant SDKs and ISV integrations break silently when µservices ship breaking changes (violates `feedback_no_silent_regression`).
2. Per-µservice teams invent ad-hoc conventions (URL versioning here, query-param versioning there, header versioning elsewhere), producing the per-axis-vocabulary fragmentation that ADR-0001 (cohesion thesis) forbids.
3. Mesh routing becomes ambiguous when a µservice's internal v1 and v2 surfaces are distinguished only by header — Cilium Service Mesh / Istio Ambient L7 routing rules cannot deterministically dispatch on request bodies.
4. Tenants on Korean regulated-industry tier (per ADR-0064 localization pack #1) and SOC2/HIPAA/GDPR enterprise tiers (per ADR-0251 certification levels) cannot pin to a known-good API generation across the migration window dictated by their compliance review cycle.
5. Webhooks (outbound HTTP calls to tenants) carry no version field, so a payload shape change silently breaks every tenant receiver.

ADR-0037 (Public API stability tiers — preview/stable/GA) defined the tier vocabulary, the per-PR semver-diff gate, the contract-first artifact layout, and the 6-month/12-month deprecation windows. It explicitly named the version convention shape (URL-path for REST, package for gRPC, schema-version field for AsyncAPI) but deferred the **version-negotiation mechanism** — how a client requests a specific API generation, how the server resolves which generation to execute, how per-tenant pinning interacts with default pinning, and how the SDK auto-generation pipeline keeps clients on a stable generation.

ADR-0145 (Inter-microservice communication: hyperscaler shape) made direct sibling-µservice gRPC the canonical internal substrate, with mTLS + Cedar + OTel + audit-chain seal invariants. Inter-µservice calls must therefore carry a deterministic version selector that the mesh can route on.

ADR-0150 (Cursor pagination canonical) demonstrated the pattern: a single canonical mechanism, BLOCKER-day-1 lane, OpenAPI 3.2.0 component refs, per-µservice manifest declaration.

The `feedback_no_silent_regression` directive (Linus-mode workspace-wide) requires every contract change to be **loud + immediate + CI-detectable**. Version-pinning is the primary mechanism that makes a contract change non-silent for tenants: a tenant pinned to `2026-05-20` continues to receive the `2026-05-20` payload shape until the explicit sunset cycle drops them onto the next generation.

PR-258A's idea-refine pass (2026-05-19) reviewed Stripe API Versioning (2024), GitHub API versioning (2022 calendar model), Twilio API versioning, AWS service date-stamped APIs (e.g. EC2 `2016-11-15`), Square API versioning, and Plaid API versioning. The consensus pattern across these hyperscalers is **request-time pinning via a version header** combined with a default-version policy per account. URL versioning is used by hyperscalers' internal mesh APIs (Stripe internal, AWS internal Twirp, Google internal Stubby) because L7 mesh routing dispatches on URL path, not headers.

oyatie therefore adopts a **dual-mode versioning model**: Stripe-style request-time pinning for external APIs (header-driven; per-tenant default; account-level pinning override) and URL versioning for internal mesh APIs (path-driven; deterministic for mesh routing).

## Decision

We adopt twelve interlocking decisions (D-1 through D-12) that together constitute the canonical oyatie API versioning model. Each decision is independently enforceable; the bundle composes into a coherent system.

### D-1 — External (public) APIs use Stripe-style request-time pinning via `X-Oyatie-API-Version` header

Every public API surface (Workspace, Cloud, Foundry, Verticals, Connect, Search, Marketplace) accepts a request header:

```
X-Oyatie-API-Version: 2026-05-20
```

The header value is a date stamp in `YYYY-MM-DD` format. Each date stamp identifies a **frozen API generation** — the request/response schemas, status code mappings, error vocabulary, validation rules, and side-effect semantics that were canonical on that date.

When the header is present, the µservice executes the request against the named generation. When the header is absent, the µservice resolves the version per D-7 (per-tenant pinning override) and falls back to the µservice's `default_public_version` declared in `manifest.json`.

Date stamps are MINTED only on dates when a public-API change ships. The set of valid date stamps for a given µservice is the µservice's `public_api_generations: [...]` list in `manifest.json`. Requests carrying an unknown date stamp receive `400 Bad Request` with `error.code = "unknown_api_version"` and `error.valid_generations: [...]` enumerating the live set.

**Why date-stamped rather than semver-stamped for the public-API request header**: a date stamp is unambiguous to humans, mechanically sortable, and maps 1:1 onto the audit-chain emission timestamp. Semver-stamped public APIs would conflict with the parallel internal-mesh URL versioning (D-2) and would force tenants to track per-µservice semver families. Stripe demonstrated this pattern at scale across a decade of API evolution; Square and Plaid both copied it.

**Compatibility window**: every minted date stamp is supported for at least 12 months from announcement (per D-4). Generations older than the sunset window are removed; their date stamp returns `410 Gone` with `Sunset` header (per RFC 8594).

Response side: every public API response includes:

```
X-Oyatie-API-Version: 2026-05-20
X-Oyatie-Default-Version: 2026-05-20
X-Oyatie-Latest-Version: 2026-05-20
Sunset: <RFC 8594 timestamp when applicable>
Deprecation: <RFC 8594 timestamp when applicable>
```

The `X-Oyatie-Latest-Version` field gives clients a discovery path to the newest generation. The `X-Oyatie-Default-Version` field tells the client what they will get if they omit the header on the next request.

### D-2 — Internal mesh APIs use URL versioning `/v1/`, `/v2/`

Internal mesh APIs (µservice → µservice gRPC and REST traffic that never leaves the cell-private mesh) use **URL-path versioning**:

```
/v1/capability/invoke
/v2/capability/invoke
```

For gRPC, the package path carries the version:

```
oya.foundry.v1.CapabilityService
oya.foundry.v2.CapabilityService
```

**Why URL versioning for the mesh and not header pinning**:

1. **L7 routing determinism**: Cilium Service Mesh and Istio Ambient L7 routing rules dispatch on URL path + method + (optionally) authority. Routing on a request header — especially one whose name conflicts with the public-API header — produces brittle routing tables, harder canary deployments, and ambiguous failure modes.
2. **Per-version canary deployment**: a v2 mesh service can be deployed as a separate Deployment behind a separate Service object, with VirtualService routing 1% of traffic to v2 and 99% to v1. This pattern is the canonical Kubernetes mesh canary; URL versioning is the standard substrate.
3. **Per-version SLO accounting**: each v1 / v2 Service has its own SLO budget (per ADR-0130 agentic SLO-gated promotion). Mixing them on a single endpoint defeats the SLO-gating substrate.
4. **Per-version dependency graph**: ADR-0011 (cross-microservice contract registry) records µservice ↔ µservice dependencies. URL versioning makes the dependency edge concrete; header pinning makes it implicit.
5. **No ambiguity for the caller**: internal callers are oyatie code, not third-party tenants. They have no need for the per-account default-version-pinning ergonomics that Stripe's header model provides. URL versioning is unambiguous and explicit.

Internal mesh URL versioning is `/v1/`, `/v2/`, … with **integer major version**. Minor and patch versions of internal APIs are additive (per D-8) and do NOT require URL bumps. Breaking changes bump the integer.

### D-3 — Per-µservice independent versioning

Each µservice owns its own version line. There is NO workspace-wide "oyatie v3" version. The Workspace Mail µservice may be on public-version `2026-05-20` and internal-mesh `/v3/` while the Cloud Compute µservice is on public-version `2026-04-01` and internal-mesh `/v1/`.

The Stripe model is the same: there is no "Stripe v3" — each resource family (charges, subscriptions, customers) evolves on its own cadence, glued together by the date-stamped generation.

**Enforcement**: per-µservice `manifest.json` declares:

```json
{
  "public_api_generations": [
    "2026-01-15",
    "2026-03-01",
    "2026-05-20"
  ],
  "default_public_version": "2026-05-20",
  "mesh_api_major_versions": ["v1", "v2"],
  "default_mesh_major_version": "v2"
}
```

The `oya-check-api-version-coverage` lane (defined in D-11 below) parses this section and verifies that every declared generation has a corresponding OpenAPI contract artifact at `microservices/<ms>/contracts/openapi/<generation>.yaml` (and analogously for gRPC `.proto`).

### D-4 — 12-month minimum sunset window; RFC 8594 Sunset header per response

A public-API generation that has been minted MUST remain callable for at least **12 months** from the date it was first deprecated. Within that 12-month window, requests pinned to the deprecated generation receive a successful response with the deprecated payload shape AND the following response headers (per RFC 8594):

```
Deprecation: Tue, 20 May 2026 00:00:00 GMT
Sunset: Wed, 20 May 2027 00:00:00 GMT
Link: <https://docs.oyatie.com/api/migration/2026-05-20-to-2027-05-20>; rel="deprecation"
Link: <https://docs.oyatie.com/api/sunset/2026-05-20>; rel="sunset"
```

The `Deprecation` header (RFC 8594 §2) is a date-time indicating WHEN the resource became deprecated. The `Sunset` header (RFC 8594 §3) is a date-time indicating when the resource will become unavailable.

After the Sunset timestamp, requests pinned to the sunset generation receive `410 Gone` with body:

```json
{
  "error": {
    "code": "api_version_sunset",
    "message": "API version 2026-05-20 was sunset on 2027-05-20. Migrate to a supported generation.",
    "sunset_at": "2027-05-20T00:00:00Z",
    "supported_generations": ["2027-03-01", "2027-04-15", "2027-05-20"],
    "migration_guide": "https://docs.oyatie.com/api/migration/2026-05-20-to-2027-05-20"
  }
}
```

**Why 12 months and not 6**: ADR-0037 already names 12 months as the GA-tier deprecation window. ADR-0258 raises every public-API generation to that minimum because:

- Korean public-sector tenants on ADR-0064 pack contracts have an annual procurement / regulatory review cycle.
- SOC2 Type II observation windows are 12 months.
- HIPAA covered-entity migration committees typically meet quarterly; 12 months guarantees four review cycles.
- Stripe's API has 12-month-plus deprecation windows; matching them sets the industry benchmark.

**Per-tenant extension**: a tenant on a regulated-industry pack (ADR-0064) MAY negotiate per-tenant sunset extension up to 24 months. The extension is recorded in the tenant's Cedar policy under `tenant_api_sunset_extension` (see D-7). The extension is opt-in and time-boxed; it does NOT extend the global sunset for other tenants.

### D-5 — 6-month deprecation announcement before sunset

Every API generation, before it enters the 12-month sunset window, MUST be announced as deprecated at least **6 months** before the sunset window begins. That is, the timeline for a generation looks like:

```
T-18mo: generation minted (status: stable)
T-6mo:  deprecation announced (Deprecation header begins emitting)
T:      sunset begins (Sunset header timestamp arrives; 410 Gone returned)
```

During the 6-month deprecation announcement window, the generation remains fully callable but every response carries the `Deprecation` header. The tenant's trust portal (per ADR-0038) shows a "deprecation alert" for the affected generation. The per-tenant deprecation-usage report (per ADR-0037) aggregates calls to deprecated generations and emails the tenant admin monthly.

**Why a 6-month announcement window separate from the 12-month sunset**: the 6-month window gives tenants time to plan the migration (scope it, get budget approval, schedule the developer time); the 12-month sunset window gives them time to execute it. Stripe's deprecation cycle uses a similar pattern (announce → coexist → sunset). Combining the two into a single 18-month window from announcement to sunset gives tenants the longest commitment any oyatie API surface makes.

**Announcement channels**:
1. Per-tenant trust portal banner (ADR-0038).
2. Per-tenant deprecation-usage report email (ADR-0037, monthly).
3. `https://docs.oyatie.com/api/changelog/<YYYY-MM>` page entry.
4. SDK release notes for affected language SDKs.
5. Audit-chain event `ApiGenerationDeprecationAnnounced` emitted to the per-tenant audit segment.

### D-6 — SDK auto-generation from OpenAPI 3.2.0 + AsyncAPI 3.1.0

oyatie ships per-language SDKs that are auto-generated from the contract artifacts. The generation pipeline is:

```
contracts/openapi/<ms>/<generation>.yaml   ──┐
contracts/proto/<ms>/<version>.proto       ──┼──> sdk-gen pipeline ──> oya-sdk-rust@<gen>
contracts/asyncapi/<ms>/<generation>.yaml  ──┘                       oya-sdk-ts@<gen>
                                                                     oya-sdk-py@<gen>
                                                                     oya-sdk-go@<gen>
                                                                     oya-sdk-java@<gen>  (GA only)
```

| Language | Generator | Output package |
|---|---|---|
| Rust | `progenitor` (REST) + `prost-build` (gRPC) + custom AsyncAPI macros | `oya-sdk-rust` |
| TypeScript | `openapi-typescript` + `@bufbuild/protobuf` + `@asyncapi/generator` | `@oyatie/sdk` (npm) |
| Python | `openapi-python-client` + `grpcio-tools` + `asyncapi-python` | `oyatie-sdk` (PyPI) |
| Go | `oapi-codegen` + `protoc-gen-go-grpc` + `asyncapi-codegen-go` | `github.com/oyatie/sdk-go` |
| Java | `openapi-generator-cli` + `protoc` + `asyncapi-jvm` | `com.oyatie:sdk` (Maven Central, GA only) |

The contract artifact format is pinned at:
- **OpenAPI 3.2.0** for REST surfaces (per `feedback_no_silent_regression`: schema-version-field bump is non-silent; 3.2.0 vs 3.1.0 vs 3.0.x is a known-version surface).
- **Protocol Buffers proto3 + edition 2023** for gRPC surfaces.
- **AsyncAPI 3.1.0** for event/topic surfaces.
- **Historical rejected context:** the GraphQL October 2021 specification was considered before
  ADR-0565 removed GraphQL from the owned surface.

The SDK release pipeline is gated by the `oya-check-sdk-contract-parity` lane: an SDK release is BLOCKED if any of the per-language SDKs would diverge from the contract source. This guarantees the SDK is a deterministic derivative of the contract, never a hand-edited drift.

Per-SDK version stamps follow a hybrid scheme:
- The SDK's own semver tracks the SDK code (e.g. `oya-sdk-rust 1.4.2` ships a bug fix in the generated code).
- The SDK targets a specific API generation by date stamp (`oya-sdk-rust 1.4.2 --api-version=2026-05-20`).
- Each SDK release supports the last 4 minted generations (configurable via SDK runtime; defaults to the latest at publish time).

### D-7 — Per-tenant API version pinning override (Cedar-gated)

A tenant MAY pin a specific API generation as their default. When the tenant pins, the µservice resolves the request version in this priority order:

1. **Request header** `X-Oyatie-API-Version: <date>` (highest priority).
2. **Tenant pin** declared in the tenant's Cedar policy as `tenant_default_api_version: <date>` for the µservice.
3. **µservice default** declared in `manifest.json` as `default_public_version: <date>`.

The tenant pin is **Cedar-gated** (per ADR-0243 Cedar-as-universal-gate): an authority within the tenant (the tenant admin, or a delegated principal with the `tenant.api.version.pin` permission) writes a Cedar policy fragment:

```cedar
permit (
    principal,
    action == Action::"api.invoke",
    resource is Workspace::Mail
) when {
    principal.tenant.id == "tenant-abc"
} advice {
    pin_api_version: "2026-05-20"
}
```

The Cedar `advice` block (per Cedar v4.2 LTS advice extension) propagates the pin to the µservice's version-resolution logic.

**Why Cedar-gated**: ADR-0243 makes Cedar the universal authorization substrate. Per-tenant API pinning is a permission ("this tenant has chosen to lock to this generation"), and permissions live in Cedar. This composes with ADR-0244 (tenant-as-universal-scoping-primitive): the tenant is the natural scope for a pin.

**Audit emission**: when a tenant updates their pin, an `ApiVersionPinChanged` audit-chain event is emitted with `{tenant_id, microservice, old_version, new_version, principal, timestamp}`. The event is sealed into the per-tenant audit segment (per ADR-0003 audit chain).

**Pin downgrade restriction**: a tenant MAY pin to any non-sunset generation. A tenant cannot pin to a generation older than 24 months from the current date (the maximum extension window in D-4), regardless of sunset status.

### D-8 — Breaking change definition (matches Stripe's)

The definition of "breaking change" is critical because it determines when a new generation MUST be minted vs when the existing generation can be extended additively.

A change is **breaking** if any of the following is true (this list matches Stripe's API versioning policy and is the BLOCKER set for `oya-governance-api-semver`):

1. **Field removal**: any response field that was previously present is no longer guaranteed.
2. **Field semantic change**: a field's meaning, units, scale, or interpretation changes (e.g. `amount: cents` → `amount: dollars`).
3. **Field type change**: a field's JSON Schema type changes (string → number, optional → required, nullable → non-nullable).
4. **Enum narrowing**: an enum value previously returned is no longer returned (callers may have switch statements that assume completeness).
5. **Enum reorder**: an enum value's ordinal position changes (affects gRPC proto field numbers; Protobuf field number reuse is forbidden by `feedback_no_silent_regression`).
6. **Required-field addition on request**: a previously optional or absent request field becomes required.
7. **Status code change**: an endpoint that previously returned 200 now returns 201, or 404 → 400 for the same input shape.
8. **Default value change**: the default for an optional field changes (e.g. `limit` default 20 → 50).
9. **Validation tightening**: a request value that previously validated now rejects (e.g. max-length 255 → 100).
10. **Authentication scheme change**: the auth header format or required scopes change.
11. **Idempotency key semantic change**: a request that was previously idempotent on key X is no longer idempotent on the same key.
12. **Side-effect change**: an endpoint that previously emitted a single event now emits two events, or vice versa (affects tenant webhook receivers).
13. **Pagination cursor format change**: the opaque cursor encoding changes such that a cursor minted by v1 cannot be decoded by v2 (per ADR-0150 cursor pagination — cursors are opaque, but a tenant's stored cursors must remain valid through the deprecation window).
14. **URL path change**: an endpoint moves from `/users` to `/accounts`.
15. **HTTP method change**: an endpoint's canonical method changes (PUT → POST).
16. **Webhook payload schema change** (see D-12).

A change is **additive (non-breaking)** if it falls into:
1. New optional response field.
2. New optional request field with a documented default.
3. New endpoint at a new URL path.
4. New enum value at the END of an enum (callers should be open-to-extension on enums; this is documented in SDK conventions).
5. New OAuth scope (existing scopes continue to function).
6. New webhook event type (existing event types continue to fire).
7. Documentation improvements.
8. Internal performance improvements that do not affect the observable contract.

The `oya-governance-api-semver` lane (per ADR-0037) uses `oasdiff` (https://github.com/oasdiff/oasdiff) to classify the diff between the previous-generation OpenAPI artifact and the proposed-generation OpenAPI artifact. The lane emits a SemverDiff classification (Patch / Minor / Major). When the diff is Major, the lane requires:

- A new generation date stamp in `manifest.json`.
- A `migration_guide` document under `docs/api/migration/<old-generation>-to-<new-generation>.md`.
- An ADR amendment or new ADR referencing ADR-0258 and the affected µservice.
- A reviewer pair from `council-architecture`.

### D-9 — Version negotiation algorithm

The canonical version-negotiation algorithm runs at every µservice's request entry point (before authentication, before authorization, before business logic):

```rust
// crates/oya-shared-api-version-kernel/src/negotiation.rs
pub fn resolve_request_version(
    req: &HttpRequest,
    tenant: &Tenant,
    manifest: &ServiceManifest,
    cedar_advice: &CedarAdviceSet,
    now: DateTime<Utc>,
) -> Result<ResolvedVersion, VersionError> {
    // Step 1: read the request header
    if let Some(header) = req.headers().get("X-Oyatie-API-Version") {
        let version = DateStamp::parse(header)?;
        if !manifest.public_api_generations.contains(&version) {
            return Err(VersionError::UnknownGeneration {
                requested: version,
                valid: manifest.public_api_generations.clone(),
            });
        }
        if manifest.is_sunset(&version, now) {
            return Err(VersionError::SunsetGeneration {
                requested: version,
                sunset_at: manifest.sunset_at(&version),
                supported: manifest.live_generations(now),
            });
        }
        return Ok(ResolvedVersion {
            generation: version,
            source: ResolutionSource::RequestHeader,
            deprecation: manifest.deprecation_for(&version),
        });
    }

    // Step 2: read the Cedar advice for tenant pin
    if let Some(pinned) = cedar_advice.get("pin_api_version") {
        let version = DateStamp::parse(pinned)?;
        if manifest.public_api_generations.contains(&version)
            && !manifest.is_sunset(&version, now)
        {
            return Ok(ResolvedVersion {
                generation: version,
                source: ResolutionSource::TenantPin,
                deprecation: manifest.deprecation_for(&version),
            });
        }
        // Tenant pin pointed at a sunset generation: fall through with audit emit
        emit_audit(AuditEvent::TenantPinFellThrough { tenant: tenant.id, requested: version });
    }

    // Step 3: fall back to µservice default
    let default = manifest.default_public_version.clone();
    Ok(ResolvedVersion {
        generation: default,
        source: ResolutionSource::ServiceDefault,
        deprecation: None,
    })
}
```

**Properties of the algorithm**:

1. **Deterministic**: for fixed inputs, the output is fixed.
2. **Fail-loud on unknown generations**: a typo in the header (e.g. `2026-05-29` when the valid set is `[2026-05-15, 2026-05-20]`) returns 400, not silent fallback to default.
3. **Fail-loud on sunset generations**: a header naming a sunset generation returns 410, not silent upgrade to default.
4. **Tenant-pin fall-through is audited**: if the tenant's Cedar pin points at a sunset generation, the request silently uses the service default — BUT an audit event is emitted so the tenant can detect the drift.
5. **Time-stable**: the algorithm uses `now` as the time reference; clock skew across mesh nodes does not produce per-node version dispatch divergence.

**Response side**: every response carries the headers from D-1 plus a debug header (only in non-production cells):

```
X-Oyatie-API-Resolution-Source: request-header | tenant-pin | service-default
```

This enables tenant-side debugging when an API call returns unexpected payload shape.

### D-10 — Pre-GA / Beta / Stable / Deprecated / Sunset lifecycle states

Every API generation lives in one of five lifecycle states. The state transitions are:

```
Pre-GA ──> Beta ──> Stable ──> Deprecated ──> Sunset
   ▲          │        │            │             │
   │          │        │            │             ▼
   └──────────┴────────┴────────────┴──────────  Removed (410 Gone forever)
       (backward state transitions are forbidden; rollback creates a NEW generation)
```

| State | Caller-visible behavior | Allowed in production tenant traffic? | SLA | Deprecation header? | Sunset header? |
|---|---|---|---|---|---|
| **Pre-GA** | Behind a feature flag; only internal tenants (oyatie itself per ADR-0242) and explicit early-access tenants can invoke. | No (default-off for general tenants). | None. | No. | No. |
| **Beta** | Available to general tenants who opt in. Breaking changes allowed with 30-day notice. | Yes (opt-in). | None. | No. | No. |
| **Stable** | Available to all tenants. Breaking changes require new generation. | Yes (default-on). | 99.9% per ADR-0037 stable tier. | No. | No. |
| **Deprecated** | Still available; every response carries `Deprecation` and `Sunset` headers per RFC 8594. | Yes. | 99.9% (unchanged from Stable). | Yes. | Yes. |
| **Sunset** | Returns `410 Gone` with `Sunset` header. | No (terminal). | N/A. | N/A. | Yes (as last response). |

**Promotion rules** (each transition is gated by an ADR amendment or a per-generation `manifest.json` change reviewed by `council-architecture`):

- **Pre-GA → Beta**: at least one internal tenant has integrated successfully (per `feedback_autonomous_implementation_artifacts`: no stub paths exercised; real-traffic only).
- **Beta → Stable**: at least 30 days at Beta with at least one external tenant; observed P99 latency within SLO; observed error rate < 0.5%.
- **Stable → Deprecated**: ADR amendment OR newer generation reached Stable AND `council-architecture` approves the deprecation calendar.
- **Deprecated → Sunset**: 12 months from Deprecation start AND `lean-a10-regression` confirms no Stable surfaces still reference the deprecated generation.
- **Sunset → Removed**: the µservice ships a release where the generation is removed from `public_api_generations` in `manifest.json` AND the audit event `ApiGenerationRemoved` is emitted.

**Per-tier SLA bindings**: see ADR-0037 stable / GA tier SLA values. ADR-0258 does NOT alter the SLA values; it pins the generation lifecycle states to those tier SLA values.

### D-11 — Audit emission on deprecation usage

Every request that resolves (per D-9) to a **Deprecated** generation MUST emit an `ApiDeprecationUsed` audit-chain event:

```rust
pub struct ApiDeprecationUsed {
    pub tenant_id: TenantId,
    pub microservice: MicroserviceId,
    pub generation: DateStamp,
    pub deprecation_started_at: DateTime<Utc>,
    pub sunset_at: DateTime<Utc>,
    pub principal: PrincipalId,
    pub endpoint_path: String,
    pub method: HttpMethod,
    pub request_id: RequestId,
    pub timestamp: DateTime<Utc>,
}
```

The event is sealed into the per-tenant audit segment (per ADR-0003 audit chain). Per-tenant aggregation is computed nightly and surfaced in:

1. The tenant's trust portal (ADR-0038) — "Deprecation usage" panel showing top-N deprecated endpoints by call volume.
2. The tenant admin monthly email (ADR-0037).
3. The `ops.oyatie.com /tenant/<id>/deprecation` report (ADR-0067).

**CI gate**: `oya-check-api-deprecation-emission` verifies that every µservice's request-entry-point code path includes the audit-emission call when the resolved generation is Deprecated. The check is BLOCKER day 1.

**Why per-call audit emission and not per-tenant-per-day aggregation at the µservice**: per-call emission is the only mechanism that survives tenant-pin changes, mid-day pin migrations, and intra-day SDK upgrades. Aggregation happens downstream in the audit-chain reader; emission is canonical at the call site.

**Volume management**: deprecation usage events are high-cardinality but low-fidelity. They are sealed into the audit chain (per ADR-0003) but the per-event payload is small (<1KB). Per-µservice rate limit: 100 events/second/tenant via the audit-chain client kernel; bursts are coalesced into per-second summary events when the rate is exceeded.

### D-12 — Webhook payload versioning

Webhook payloads (oyatie → tenant receivers) carry version metadata in **two** locations:

1. **Header**: `X-Oyatie-Webhook-Version: 2026-05-20` on the outbound HTTP request.
2. **Body**: `version: "2026-05-20"` as a top-level field in the JSON payload.

**Why two locations**: tenant webhook receivers may parse the header (for routing to a per-version handler) OR the body (for in-handler dispatch). Carrying both is cheap and supports both patterns.

**Version negotiation for webhooks**: the tenant configures their webhook endpoint with a target version, stored in the tenant's webhook subscription record:

```json
{
  "tenant_id": "tenant-abc",
  "webhook_id": "wh_xyz",
  "url": "https://tenant.example.com/oyatie-webhook",
  "target_version": "2026-05-20",
  "event_types": ["user.created", "user.updated"],
  "secret_ref": "kms-secret-arn-..."
}
```

The webhook dispatcher (per microservices/connector-events or per-µservice outbox; see ADR-0145) renders the payload at the target version. When a target version is sunset (per D-4), the dispatcher:

1. Emits `WebhookTargetVersionSunset` audit event.
2. Falls back to the µservice's `default_webhook_version` for that event type.
3. Emails the tenant admin with the migration notice.

**Webhook generation lifecycle**: webhook generations follow the same Pre-GA / Beta / Stable / Deprecated / Sunset states (D-10) as API generations but on a per-event-type basis. The `user.created` event may be on generation `2026-05-20` while the `payment.completed` event is on `2026-04-15`.

**Signature versioning**: webhook signatures use HMAC-SHA256 with the secret rotated per ADR-0043 secrets management. The signature header is `X-Oyatie-Webhook-Signature: t=<timestamp>,v1=<hex>`. The `v1` prefix is the signature algorithm version, not the payload version; payload version is independent.

## Alternatives considered

### Alternative A — URL-only versioning everywhere (public and internal)

**Pros**:
- Single mechanism; lower cognitive load for internal teams.
- Mesh routing is trivial.
- Matches the ADR-0037 §"API versioning convention" pre-existing convention for REST (already says `/api/v1/`).

**Cons**:
- Per-tenant pinning becomes URL-rewriting (`/v1/users` → `/2026-05-20/users`?) which is awkward and breaks RESTful resource identity (URLs should identify resources, not API generations).
- Stripe's empirical finding (from a decade of API evolution) is that URL versioning forces every breaking change to be a "wholesale move-to-v2" event, which clients resist; date-stamped header pinning lets clients adopt new generations incrementally per-endpoint.
- Tenant default-version-pinning requires storing per-tenant URL-rewrite tables, which doesn't compose with proxies, CDNs, or self-hosted-tenant gateways.
- URL versioning encourages "v3 is a new product"; date-stamped versioning encourages "v3 is the same product, evolved."
- Webhook URL-versioning would require the tenant to maintain N receiver endpoints (one per version) rather than a single endpoint that dispatches on a header.

**Rejected because**: external tenants benefit from Stripe-style per-account default-version pinning; URL-only on the public surface costs that benefit for no offsetting gain.

### Alternative B — Header-only versioning everywhere (public and internal)

**Pros**:
- Single mechanism; same.
- Per-tenant pinning is uniform across public and internal.

**Cons**:
- L7 mesh routing on header is brittle (per D-2 rationale): Cilium / Istio Ambient routing is built for path-based dispatch.
- Per-version canary deployment on a single URL requires HTTP request body inspection or header-based traffic split, both of which are mesh-supportable but operationally fragile.
- ADR-0011 cross-microservice contract registry would need per-(µservice, version-header-value) edge records, which is conceptually muddier than per-(µservice, URL-path) edges.
- Internal callers have no need for date-stamped pinning ergonomics; URL versioning is the canonical pattern for internal mesh in every hyperscaler we surveyed (AWS Twirp, Google Stubby, Stripe internal, Anthropic internal).

**Rejected because**: internal mesh routing is the dominant constraint for the internal surface; header-based dispatch is the wrong substrate.

### Alternative C — GraphQL Federation as the version-management substrate (historical, rejected)

**Pros**:
- GraphQL's deprecation directive `@deprecated(reason: "...")` is a first-class language feature.
- Field-level versioning is more granular than generation-level versioning; tenants only migrate the fields they use.
- Apollo Federation provides per-subgraph evolution with global schema composition.

**Cons**:
- At the time of the original decision, GraphQL surfaces were contemplated for Drive, Workspace
  search results, and Knowledge Graph. ADR-0565 later removed that planned surface entirely.
- GraphQL deprecation is per-field, not per-generation; tenants cannot pin to "the schema as of 2026-05-20" — they can only ignore the deprecation warnings.
- gRPC and AsyncAPI cannot ride the GraphQL deprecation directive; they would need a separate model anyway.
- Internal mesh is gRPC + REST, not GraphQL.
- Apollo Federation pricing and operational model is a separate dependency surface to take on.

**Rejected because**: the canonical surface is REST + gRPC + AsyncAPI. ADR-0565 subsequently
removed GraphQL entirely, so its deprecation directive is not available anywhere in the owned API
surface and cannot complement D-1.

### Alternative D — No versioning; every API change is breaking; tenants migrate on every release

**Pros**:
- Zero overhead.
- Forces tenants to stay current.

**Cons**:
- Violates `feedback_no_silent_regression` ("we don't break userspace") — every release would break every tenant.
- No tenant would adopt the platform at production scale.
- SOC2 / HIPAA / GDPR audits would fail (no change-management substrate).
- Korean public-sector procurement would reject (no commitment to compatibility).
- Stripe explicitly cites "tenants want to update on their own schedule" as the rationale for their date-stamped model.

**Rejected because**: violates the platform's primary commitment to tenants and the workspace-wide no-silent-regression directive.

### Alternative E — Calendar versioning in the URL (e.g. `/api/2026-05-20/users`)

**Pros**:
- Single mechanism for public and internal (calendar versioning everywhere).
- URL identifies both the resource AND the generation.
- AWS uses this pattern for some service date-stamped APIs (`ec2/2016-11-15`).

**Cons**:
- URL becomes verbose; per-call URL parsing on every request is heavier.
- Per-tenant default-version pinning still requires URL rewriting, which (per Alternative A) breaks RESTful resource identity.
- AWS's calendar-versioned URLs are stable per-service for years (EC2 has been on `2016-11-15` since 2016); the pattern doesn't accommodate per-µservice frequent generation minting that oyatie expects.
- AWS itself complements calendar-versioned URLs with header-based generation selection in the SDK layer; tenants don't write the URL directly.

**Rejected because**: it duplicates the date stamp into the URL without adding any property that the D-1 header mechanism doesn't already provide.

### Alternative F (CHOSEN) — Dual-mode: Stripe-style header pinning for public APIs, URL versioning for internal mesh

**Pros** (all of the above advantages, none of the disadvantages):
- Public surface: Stripe-style per-tenant pinning gives tenants the ergonomics they need.
- Internal surface: URL versioning gives the mesh deterministic L7 routing.
- The two mechanisms are layered on the same underlying contract artifacts (OpenAPI 3.2.0 + Protobuf), so the SDK auto-generation pipeline doesn't double its complexity.
- Date-stamped generations on the public surface translate naturally to dated audit-chain events; URL versions on the internal surface translate naturally to per-version Kubernetes Service objects.
- Per-µservice independent cadence (D-3) is preserved across both modes.

**Cons**:
- Two mechanisms instead of one; per-µservice contract authors must understand both.
- The boundary between public and internal surfaces must be unambiguous (resolved by D-1 declaration in `manifest.json`; see Implementation surface).

**Accepted**: the cons are tractable (one ADR, one canonical kernel, one BLOCKER lane); the pros match every hyperscaler we surveyed.

## Consequences

### Positive

1. **Tenant ergonomics match Stripe's**: tenants can pin a date stamp, integrate against that stamp, and migrate on their own schedule. Per-tenant default-version pinning makes "we're on the 2026-05-20 API" a stable claim for SOC2 / HIPAA / KR-public-sector compliance reviews.
2. **Internal mesh routing is deterministic**: Cilium / Istio Ambient L7 rules dispatch on URL path; per-version canary deployment is canonical Kubernetes substrate; per-version SLO accounting (ADR-0130) composes cleanly.
3. **Per-µservice independent cadence**: ADR-0001 cohesion thesis is preserved (single product), but each µservice's API evolves independently; no workspace-wide release-train coupling.
4. **No silent regression**: every breaking change mints a new generation; `oasdiff` classifies the diff at PR time; `oya-governance-api-semver` enforces; `oya-check-api-version-coverage` enforces manifest declaration; `oya-check-api-deprecation-emission` enforces audit emission. The Linus-mode directive is satisfied workspace-wide.
5. **SDK auto-generation pipeline is deterministic**: per-language SDKs derive from OpenAPI 3.2.0 / Protobuf / AsyncAPI 3.1.0 contracts; the `oya-check-sdk-contract-parity` lane forbids manual SDK drift.
6. **12-month sunset is industry-leading**: Stripe's 12-month-plus pattern is the benchmark; oyatie matches it without exception, including for the cross-cell compliance pack tenants.
7. **Webhook versioning is explicit**: tenant receivers pin a target version; the dispatcher renders at that version; sunset falls back with audit emission and admin notification.
8. **Cedar-gated per-tenant pinning composes with ADR-0243 / ADR-0244**: tenant pin is a Cedar policy fragment; tenant is the universal scope; no parallel authorization mechanism.

### Negative

1. **Two mechanisms to teach**: per-µservice contract authors learn both header-pinning (public) and URL-versioning (internal). Onboarding cost ~1 day per author per quarter as new patterns emerge. Mitigated by `docs/standards/api-versioning-canonical.md` reference + canonical kernel + per-lane CI gates.
2. **Per-µservice contract surface grows**: each µservice's `contracts/openapi/` directory carries one file per minted public-API generation. For a µservice that mints quarterly, that's 4-6 files in flight at any time. Mitigated by `contracts/openapi/<ms>/_index.yaml` aggregation generated by sdk-gen pipeline.
3. **Per-generation OpenAPI authoring discipline**: each new generation requires the OpenAPI file to be valid, ref-resolved, and oasdiff-clean vs the prior generation. Mitigated by `oya-check-openapi-validity` (existing) + `oya-governance-api-semver` (oasdiff classification at PR time).
4. **Per-tenant pin storage**: every tenant's Cedar policy carries a `pin_api_version` advice block per µservice they pin. Worst case: a tenant pinning every µservice independently has N entries in their Cedar policy. Mitigated by Cedar's per-tenant fragment compression (per ADR-0246) and by the empirical observation that most tenants pin 1-3 µservices, not all.
5. **Audit-chain volume increases**: deprecation-usage events emit per-call (D-11). For a µservice with 1000 calls/second to a deprecated generation, that's 1000 events/second. Mitigated by rate-limited coalescing (per D-11 implementation note) and by the audit-chain's per-tenant segment isolation (events to one tenant don't affect another's audit segment latency).
6. **Mesh-level v1/v2 coexistence operational cost**: maintaining v1 and v2 of an internal mesh service simultaneously means double the pod count, double the SLO accounting, double the per-version observability dashboards. Mitigated by integer-major-only versioning on the mesh (no v1.1, v1.2 — just v1 and v2) and by the fact that mesh v2 deployments are time-boxed (the mesh v1 is removed once all internal callers have migrated, which is faster than tenant migration because internal callers are oyatie code).

### Operational

1. **Per-µservice manifest update required**: every µservice's `manifest.json` gains `public_api_generations`, `default_public_version`, `mesh_api_major_versions`, `default_mesh_major_version`. Tooling: `oya-fix-manifest-api-versioning` (auto-add with sensible defaults; reviewed per-µservice).
2. **Per-µservice OpenAPI 3.2.0 contract migration**: µservices currently on OpenAPI 3.0.x or 3.1.0 must migrate to 3.2.0. Tooling: `openapi-3.2.0-migrate` per-file; semantic-equivalent rewrite; oasdiff-validated.
3. **Per-µservice deprecation calendar**: existing public-API endpoints that have no minted generation date stamp gain a synthetic `2026-05-20` stamp on ADR-0258 landing; they are NOT deprecated by this ADR (no behavior change); they simply gain the version-tag substrate.
4. **Per-tenant Cedar policy migration**: tenants who currently rely on implicit "latest API" gain an explicit `pin_api_version: 2026-05-20` (the synthetic stamp); they may change the pin at any time via the trust portal.
5. **SDK release re-pinning**: each language SDK gains the `--api-version=<date>` runtime parameter; default is the latest generation at SDK publish time. SDK consumers may override per-call (per D-1 header).
6. **Webhook dispatcher migration**: existing webhook subscriptions gain `target_version: 2026-05-20` (synthetic); tenants may change via the trust portal.
7. **`oya-check-api-version-coverage` lane wired into `gate run-all`**: BLOCKER day 1; verifies every µservice's `manifest.json` declares the versioning fields and every declared generation has a contract artifact.
8. **`oya-check-api-deprecation-emission` lane wired into `gate run-all`**: BLOCKER day 1; verifies every request-entry path emits the audit event when the resolved generation is Deprecated.
9. **`oya-check-sdk-contract-parity` lane wired into `gate run-all`**: BLOCKER day 1; verifies per-language SDKs are deterministic derivatives of the contracts.
10. **Per-µservice ops dashboard**: `ops.oyatie.com /microservice/<ms>/api` shows per-generation call volume, error rate, P99 latency, and tenant pinning distribution.
11. **Audit-chain reader index**: `ApiDeprecationUsed` events are indexed by `(tenant_id, microservice, generation)` for fast per-tenant deprecation-usage queries.

## Rollback

ADR-0258 is reversible. The revert path:

```bash
git revert <merge-commit-of-this-ADR-and-the-skeleton-PR>
```

State-change one-way analysis (per shipping-readiness checklist):

1. **`manifest.json` additions are reversible**: removing `public_api_generations`, `default_public_version`, `mesh_api_major_versions`, `default_mesh_major_version` produces the pre-ADR state. No persistent state depends on these fields.
2. **OpenAPI 3.2.0 migration is reversible**: contracts in `contracts/openapi/<ms>/<generation>.yaml` can be re-migrated to 3.1.0 or 3.0.x via `openapi-downgrade`. The pre-ADR contract files remain in git history.
3. **Cedar policy `pin_api_version` advice is reversible**: removing the advice block from the tenant's Cedar fragment removes the pin; the µservice falls back to default. No state depends on the pin existing.
4. **Audit-chain `ApiDeprecationUsed` events are append-only**: pre-existing events remain valid as Ed25519-signed leaves. Revert stops new emission; existing seals stay. No retroactive corruption.
5. **SDK auto-generation pipeline is reversible**: removing the per-language SDK release channel and reverting to the pre-ADR SDK release process is supported by the SDK monorepo's branching model.
6. **CI lane disablement is reversible**: `oya-check-api-version-coverage`, `oya-check-api-deprecation-emission`, `oya-check-sdk-contract-parity` are advisory-mode by default during the rollout window (W+0 to W+30 days); BLOCKER after W+30. Revert before W+30 has no enforcement impact.

No one-way state changes. The revert is operationally safe within the W+30-day rollout window.

After W+30, partial revert is supported per-µservice: the µservice's `manifest.json` may declare `api_versioning_enabled: false` to opt-out (advisory-mode treatment), pending the per-µservice migration. The opt-out is time-boxed (30 days from declaration) and requires an ADR amendment.

## Verification

### Verification matrix

| Verification target | Method | Owner | Frequency |
|---|---|---|---|
| Manifest field presence | `oya-check-api-version-coverage` lane | council-architecture | per-PR |
| OpenAPI 3.2.0 validity per generation | `oya-check-openapi-validity` lane (existing) | per-µservice | per-PR |
| oasdiff classification (Patch/Minor/Major) | `oya-governance-api-semver` lane | council-architecture | per-PR |
| SDK contract parity | `oya-check-sdk-contract-parity` lane | sdk-tooling | per-PR |
| Deprecation audit emission | `oya-check-api-deprecation-emission` lane | per-µservice | per-PR |
| RFC 8594 header rendering | integration test in `crates/oya-shared-api-version-kernel/tests/` | shared-kernel-owner | per-PR + nightly |
| Per-tenant Cedar pin propagation | integration test against canary tenant | council-architecture | per-PR + per-deploy |
| Webhook payload versioning | integration test in `crates/oya-shared-webhook-dispatcher-kernel/tests/` | shared-kernel-owner | per-PR + nightly |
| 12-month sunset window enforcement | nightly cron `oya-check-sunset-window` | ops-sre-reliability | nightly |
| Per-µservice independent cadence | structural check in `oya-check-api-cadence-independence` | council-architecture | weekly |

### Acceptance criteria

ADR-0258 is fully landed when:

1. Every µservice's `manifest.json` declares the four versioning fields with non-empty values.
2. Every declared `public_api_generations` entry has a corresponding contract artifact at `contracts/openapi/<ms>/<generation>.yaml` (or `.proto` / `asyncapi.yaml`).
3. `oya-check-api-version-coverage`, `oya-check-api-deprecation-emission`, `oya-check-sdk-contract-parity` are BLOCKER (not advisory).
4. Per-language SDKs are auto-generated from contracts; no hand-edited drift.
5. Every public-API response carries `X-Oyatie-API-Version` + `X-Oyatie-Default-Version` + `X-Oyatie-Latest-Version` headers.
6. Webhook payloads carry `X-Oyatie-Webhook-Version` header + `version` body field.
7. RFC 8594 `Sunset` and `Deprecation` headers render on every Deprecated generation response.
8. Per-tenant Cedar pin advice propagates through the version-negotiation algorithm with audit emission on fall-through.
9. The `oya-check-sunset-window` nightly cron is green workspace-wide.

### Rollout phases

| Phase | Date range | Enforcement | Tenant impact |
|---|---|---|---|
| W+0 to W+30 | 2026-05-20 → 2026-06-19 | All lanes advisory | None (silent ramp) |
| W+30 to W+90 | 2026-06-20 → 2026-08-18 | `oya-check-api-version-coverage` BLOCKER; others advisory | Manifest must declare versioning fields |
| W+90 to W+180 | 2026-08-19 → 2026-11-16 | `oya-check-api-deprecation-emission` + `oya-check-sdk-contract-parity` BLOCKER | Full versioning model live |
| W+180+ | 2026-11-17+ | All lanes BLOCKER; opt-out path closed | Steady state |

## Implementation surface

### Shared kernel crates

- `crates/oya-shared-api-version-kernel/` — request-entry version-resolution kernel (D-9 algorithm).
  - `src/negotiation.rs` — `resolve_request_version()` function.
  - `src/date_stamp.rs` — `DateStamp` newtype with parse/validate.
  - `src/manifest.rs` — `ServiceManifest` view over per-µservice `manifest.json`.
  - `src/cedar_advice.rs` — Cedar advice block parser for `pin_api_version`.
  - `src/headers.rs` — RFC 8594 `Deprecation`/`Sunset` header rendering.
  - `tests/` — integration tests covering header / pin / default / sunset / unknown paths.

- `crates/oya-shared-webhook-dispatcher-kernel/` — webhook payload version rendering (D-12).
  - `src/dispatcher.rs` — outbound webhook with version header + body.
  - `src/signature.rs` — HMAC-SHA256 signature with algorithm version prefix.
  - `src/subscription.rs` — webhook subscription record with `target_version`.

- `crates/oya-shared-audit-chain-client-kernel/` (existing, per ADR-0145) — gains `ApiDeprecationUsed` and `ApiVersionPinChanged` and `WebhookTargetVersionSunset` and `ApiGenerationDeprecationAnnounced` and `ApiGenerationRemoved` event types.

### CI lane crates

- `crates/oya-check-api-version-coverage/` — manifest declaration coverage (D-3 enforcement).
- `crates/oya-check-api-deprecation-emission/` — audit emission coverage (D-11 enforcement).
- `crates/oya-check-sdk-contract-parity/` — SDK auto-generation parity (D-6 enforcement).
- `crates/oya-check-sunset-window/` — 12-month minimum sunset enforcement (D-4 enforcement; nightly).
- `crates/oya-check-api-cadence-independence/` — per-µservice independence enforcement (D-3 structural check).
- `crates/oya-governance-api-semver/` (existing, per ADR-0037) — oasdiff classification at PR time (D-8 enforcement).

### SDK auto-generation pipeline

- `tools/sdk-gen/` — top-level orchestrator.
- `tools/sdk-gen/rust/` — Rust SDK generator (progenitor + prost + custom AsyncAPI macros).
- `tools/sdk-gen/typescript/` — TS SDK generator (openapi-typescript + @bufbuild/protobuf + @asyncapi/generator).
- `tools/sdk-gen/python/` — Python SDK generator (openapi-python-client + grpcio-tools + asyncapi-python).
- `tools/sdk-gen/go/` — Go SDK generator (oapi-codegen + protoc-gen-go-grpc + asyncapi-codegen-go).
- `tools/sdk-gen/java/` — Java SDK generator (openapi-generator-cli + protoc + asyncapi-jvm; GA only).
- `tools/sdk-gen/Makefile` — `make sdk-all-languages` regenerates every SDK.
- `tools/sdk-gen/parity-check.sh` — runs by `oya-check-sdk-contract-parity` lane.

### OpenAPI compatibility lane (oasdiff)

- Tool: `oasdiff` (https://github.com/oasdiff/oasdiff), MIT-licensed.
- Invoked per-PR by `oya-governance-api-semver`.
- Diff classification: Patch / Minor / Major per D-8.
- Output: `evidence/api-semver/<pr-number>/<microservice>/<old-gen>-to-<new-gen>.json` JSON report.

### Standards documents

- `docs/standards/api-versioning-canonical.md` — canonical reference for D-1 through D-12 with worked examples.
- `docs/standards/api-deprecation-runbook.md` — operator runbook for announcing, deprecating, and sunsetting a generation.
- `docs/standards/webhook-versioning-canonical.md` — webhook-specific patterns (D-12).
- `docs/api/migration/<old-gen>-to-<new-gen>.md` — per-(µservice, generation-pair) migration guides authored at deprecation announcement time.

### Trust portal surface (ADR-0038 integration)

- Per-tenant "API generations" panel showing pinned generations across µservices.
- Per-tenant "Deprecation usage" panel showing top-N deprecated endpoints by call volume.
- Per-tenant "Migration calendar" panel showing upcoming sunset dates and migration guide links.
- Tenant-admin self-service: change pinned generation per µservice via dropdown; Cedar advice fragment updated on save; audit event emitted.

### ops.oyatie.com surface (ADR-0067 integration)

- `/api-versioning` — workspace-wide view of generation states per µservice.
- `/api-versioning/sunset-calendar` — all generations approaching sunset within 90 days.
- `/api-versioning/tenant-pinning-distribution` — histogram of per-tenant pin choices.
- `/api-versioning/audit-deprecation-usage` — per-tenant deprecation event aggregation.

## References

### External references

- **Stripe API Versioning** (2024) — https://stripe.com/docs/api/versioning. Canonical Stripe model: date-stamped generations, per-account default pinning, request-time header override, monthly minting cadence at peak. Empirical foundation: a decade of API evolution at Stripe scale.
- **RFC 8594** — "The Sunset HTTP Header Field" (May 2019). Defines `Sunset` and `Deprecation` HTTP response headers. https://datatracker.ietf.org/doc/html/rfc8594.
- **Semantic Versioning 2.0.0** — https://semver.org/spec/v2.0.0.html. MAJOR.MINOR.PATCH semantics for API version bumps.
- **oasdiff** — https://github.com/oasdiff/oasdiff. MIT-licensed OpenAPI diff tool used by `oya-governance-api-semver`.
- **OpenAPI Specification 3.2.0** — https://spec.openapis.org/oas/v3.2.0. Canonical REST contract format.
- **AsyncAPI 3.1.0** — https://www.asyncapi.com/docs/reference/specification/v3.1.0. Canonical event contract format.
- **Protocol Buffers** — proto3 language guide + edition 2023 announcement. https://protobuf.dev/.
- **Cedar v4.2 LTS** — https://www.cedarpolicy.com/. Authorization policy substrate; advice extension for D-7 pinning.
- **GitHub REST API versioning** (2022 calendar model) — https://docs.github.com/en/rest/overview/api-versions. Reference for calendar-stamped public API.
- **Twilio API versioning** — https://www.twilio.com/docs/usage/api/version. Reference for header-driven versioning.
- **AWS service date-stamped APIs** — EC2 `2016-11-15`, S3 `2006-03-01`, et al. Reference for URL-embedded date stamps.
- **Square API versioning** — https://developer.squareup.com/docs/build-basics/versioning-overview. Reference copy of Stripe model.
- **Plaid API versioning** — https://plaid.com/docs/api/versioning/. Reference copy of Stripe model.
- **Apollo Federation** — https://www.apollographql.com/docs/federation/. Considered for Alternative C; per-field `@deprecated` directive context.
- **AWS Twirp / Stubby internal protocols** (public Google SRE Workbook) — internal mesh URL versioning context.

### Oyatie references

- **ADR-0001** — Cohesion thesis: one product across all microservices. https://github.com/oyatie/oyatie/blob/dev/docs/decisions/ADR-0001-cohesion-thesis-one-product-flat-catalog.md.
- **ADR-0003** — Audit chain and evidence emission. Per-tenant audit segment substrate used by D-11.
- **ADR-0011** — Cross-microservice contract registry. Records per-µservice contract edges that D-1 / D-2 surface.
- **ADR-0037** — Public API stability tiers — preview / stable / GA. Tier vocabulary referenced by D-10.
- **ADR-0038** — Trust portal. Per-tenant deprecation surface used by D-5 + D-7 + D-11.
- **ADR-0042** — Observability stack (OTel + in-house UI). Per-version SLO accounting integration.
- **ADR-0043** — Secrets management (OpenBao + HSM per cell). Webhook signature key rotation per D-12.
- **ADR-0064** — Regional pack architecture. Korean pack #1 as 24-month-extension tenant per D-4.
- **ADR-0067** — ops.oyatie.com manifest sections. `/api-versioning/*` surfaces.
- **ADR-0130** — Agentic SLO-gated promotion. Per-version SLO accounting per D-2.
- **ADR-0131** — Per-microservice flat layout. `microservices/<ms>/contracts/openapi/<generation>.yaml` location.
- **ADR-0145** — Inter-microservice communication: hyperscaler shape with opt-in Workflow + Ontology. Direct mesh substrate that D-2 URL-versions.
- **ADR-0150** — Cursor pagination canonical. Pattern reference for canonical-kernel + BLOCKER-day-1 lane + manifest declaration.
- **ADR-0243** — Cedar as universal gate. Per-tenant pin enforcement substrate per D-7.
- **ADR-0244** — Tenant as universal scoping primitive. Per-tenant pin scope per D-7.
- **ADR-0246** — Policy engine substrate promotion. Cedar fragment compression for D-7.
- **ADR-0251** — Compliance pack cell certification levels. SOC2 / HIPAA / GDPR per D-4.

### Auto-memory references

- **feedback_no_silent_regression** — "we don't break userspace" workspace-wide; `lean-a10-regression` BLOCKER day 1. ADR-0258 is the API-surface implementation of this directive.
- **feedback_quality_performance_scalability_bar** — industry-leader quality bar; Stripe API stability reference. ADR-0258 matches the Stripe bar.
- **feedback_autonomous_implementation_artifacts** — no stubs / no dead code / stale removed in reality. ADR-0258 forbids "we'll fix the docs later" deprecation announcements.
- **feedback_clean_architecture_requirements** — port-in-kernel pattern. `oya-shared-api-version-kernel` is a sealed kernel port.
- **feedback_workflow_objectgraph_adapter_layer** — superseded by ADR-0145; informational context only.

## Appendix A — Pattern attribution

### A.1 Stripe-style date-stamped versioning

Pattern: Each minted public-API generation is identified by a `YYYY-MM-DD` date stamp. Tenants pin a generation in their account settings and override per-request via header.

Attributed to: **Stripe** (https://stripe.com/docs/api/versioning). Documented since 2014. Adopted by Square, Plaid, Mercado Pago, and Adyen (in part).

oyatie-specific adaptations:
- Header name `X-Oyatie-API-Version` (Stripe uses `Stripe-Version`).
- Per-tenant pin stored as Cedar advice fragment (Stripe stores as account setting).
- Per-µservice independent generation list (Stripe has one generation list workspace-wide).
- Date-stamp date format is `YYYY-MM-DD` (Stripe uses the same).

### A.2 URL-path versioning for internal mesh

Pattern: Internal mesh services use URL paths `/v1/`, `/v2/` to identify integer-major API generations. Per-version Kubernetes Service objects enable canary deployment and per-version SLO accounting.

Attributed to: **AWS** (internal Twirp), **Google** (internal Stubby with versioned protobuf packages), **Stripe** (internal). Public reference: Google SRE Workbook chapter 1 + AWS Well-Architected Reliability Pillar 2024.

oyatie-specific adaptations:
- gRPC package path carries version (e.g. `oya.foundry.v1.CapabilityService`).
- Cilium Service Mesh / Istio Ambient L7 routing on URL path (per ADR-0145 + ADR-0148).
- Per-version SLO accounting via ADR-0130.

### A.3 RFC 8594 Sunset header

Pattern: HTTP response carries `Sunset: <timestamp>` and `Deprecation: <timestamp>` headers on responses from deprecated endpoints.

Attributed to: **IETF RFC 8594** (May 2019), Erik Wilde. Adopted by Stripe, GitHub, Microsoft Graph, Google Cloud APIs.

oyatie-specific adaptations:
- `Link: ...; rel="deprecation"` and `Link: ...; rel="sunset"` headers point at oyatie's migration guide and sunset documentation per generation.
- Headers render on every response from a Deprecated generation, not just on a first-call basis.

### A.4 OpenAPI 3.2.0 + AsyncAPI 3.1.0 as canonical contract substrate

Pattern: Canonical contract artifacts in standardized formats; SDKs derived deterministically; per-PR diff classification via `oasdiff` (or equivalent).

Attributed to: **OpenAPI Initiative** (Linux Foundation), **AsyncAPI Initiative** (Linux Foundation), **oasdiff project**.

oyatie-specific adaptations:
- OpenAPI 3.2.0 pinned (newer than the 3.1.0 default at many shops); contains the `examples` keyword improvements and standardized webhook surface.
- AsyncAPI 3.1.0 pinned (3.x major; multi-channel + per-channel-binding model).
- Per-language SDK generators selected for production-quality output (progenitor for Rust, openapi-typescript for TS, openapi-python-client for Python, oapi-codegen for Go).

### A.5 Webhook signature with versioned algorithm prefix

Pattern: Outbound webhook carries HMAC-SHA256 signature with algorithm version prefix (e.g. `v1=<hex>`) to enable signature algorithm rotation without breaking tenant receivers.

Attributed to: **Stripe** webhook signing (https://stripe.com/docs/webhooks/signatures). Adopted by Square, Plaid.

oyatie-specific adaptations:
- Signature header `X-Oyatie-Webhook-Signature: t=<timestamp>,v1=<hex>` matches Stripe shape.
- Algorithm version `v1` is the signature algorithm version, independent of the payload-version date stamp.
- Secret rotation per ADR-0043 (per-cell KMS).

### A.6 Per-tenant default pinning as authorization advice

Pattern: Per-tenant default API version is a permission, stored as a policy fragment in the authorization substrate.

Attributed to: novel oyatie adaptation. Stripe stores account-level pinning in a per-account settings table; AWS stores per-account region-default-API in IAM; oyatie unifies the storage in Cedar (the universal authorization substrate per ADR-0243).

Rationale for the adaptation: ADR-0243 (Cedar as universal gate) requires that all authorization-shaped decisions flow through Cedar. The per-tenant pin is a permission ("this tenant is allowed to receive responses on this API generation"), so it lives in Cedar advice. This is a synthesis of Stripe's per-account model + ADR-0243's universal-Cedar substrate; it is not directly attributable to any prior implementation.

### A.7 Per-generation Pre-GA / Beta / Stable / Deprecated / Sunset lifecycle

Pattern: Five-state generation lifecycle with explicit transition gates.

Attributed to: composite of **Kubernetes API versioning** (alpha/beta/stable/deprecated), **Google Cloud API lifecycle stages**, and **Microsoft Graph API lifecycle**. ADR-0258 adds the explicit `Sunset` (post-deprecation, terminal-410) state distinct from `Removed` (permanently absent from manifest).

oyatie-specific adaptations:
- Pre-GA gates require oyatie-internal-tenant integration (per ADR-0242 oyatie-is-a-tenant doctrine) before external Beta exposure.
- Beta → Stable gates require 30-day observation window with at least one external tenant.
- Sunset is explicitly a separate state from Removed; Sunset emits `410 Gone` with `Sunset` header (RFC 8594), Removed emits `404 Not Found` (no version-specific header).

## Appendix B — Worked example: v1.0 → v1.1 additive then v2.0 breaking

This appendix walks through a complete generation lifecycle for a hypothetical µservice (`oya-workspace-mail`) to demonstrate the model end-to-end.

### B.1 Initial state (2026-05-20)

Manifest:

```json
{
  "microservice_id": "oya-workspace-mail",
  "public_api_generations": ["2026-05-20"],
  "default_public_version": "2026-05-20",
  "mesh_api_major_versions": ["v1"],
  "default_mesh_major_version": "v1"
}
```

OpenAPI contract `microservices/oya-workspace-mail/contracts/openapi/2026-05-20.yaml`:

```yaml
openapi: 3.2.0
info:
  title: Oya Workspace Mail
  version: "2026-05-20"
paths:
  /messages:
    get:
      operationId: listMessages
      parameters:
        - $ref: '#/components/parameters/Cursor'    # per ADR-0150
        - $ref: '#/components/parameters/PageSize'  # per ADR-0150
      responses:
        '200':
          description: Page of messages
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MessagePage'
components:
  schemas:
    MessagePage:
      type: object
      required: [items, next_cursor]
      properties:
        items:
          type: array
          items:
            $ref: '#/components/schemas/Message'
        next_cursor:
          type: string
          nullable: true
    Message:
      type: object
      required: [id, subject, sender, sent_at]
      properties:
        id:
          type: string
        subject:
          type: string
        sender:
          type: string
          format: email
        sent_at:
          type: string
          format: date-time
```

Internal mesh gRPC: `oya.workspace.mail.v1.MailService`.

Tenant `tenant-abc` pins to `2026-05-20` via trust portal. Their Cedar policy gains:

```cedar
permit (
    principal,
    action == Action::"api.invoke",
    resource is Workspace::Mail
) when {
    principal.tenant.id == "tenant-abc"
} advice {
    pin_api_version: "2026-05-20"
};
```

Tenant `tenant-xyz` does NOT pin; they rely on the µservice default (also `2026-05-20`).

Both tenants' SDK packages: `@oyatie/sdk@1.0.0` configured with `--api-version=2026-05-20`.

### B.2 Additive change (2026-07-15): v1.1 — add `cc` field

The µservice team wants to expose the CC recipients on each message. They add an optional `cc` field to the `Message` schema.

Change at PR time:
- Author the new OpenAPI: `microservices/oya-workspace-mail/contracts/openapi/2026-07-15.yaml` with the `cc` field added to `Message` (optional, type array of email).
- `oasdiff 2026-05-20.yaml 2026-07-15.yaml` returns Minor (additive).
- `oya-governance-api-semver` lane classifies as `api-minor`. No ADR amendment required.

Update manifest:

```json
{
  "public_api_generations": ["2026-05-20", "2026-07-15"],
  "default_public_version": "2026-07-15",
  "mesh_api_major_versions": ["v1"],
  "default_mesh_major_version": "v1"
}
```

Behavior post-deploy:
- Tenants pinned to `2026-05-20` (e.g. `tenant-abc`) continue receiving responses WITHOUT the `cc` field. The µservice serializes the response with `cc` omitted per the `2026-05-20` schema.
- Tenants on default (`tenant-xyz`) now receive responses WITH the `cc` field, because the default has shifted to `2026-07-15`.
- Internal mesh is unchanged: still `v1`. The gRPC service adds the `cc` field as a new optional field (Protobuf field number 5, never reused).
- SDK release: `@oyatie/sdk@1.1.0` configured with `--api-version=2026-07-15` by default; consumers can pin to `2026-05-20` via SDK config.

No deprecation event fires. No audit-chain `ApiDeprecationUsed` events emitted (the `2026-05-20` generation is still Stable, not Deprecated).

### B.3 Breaking change announcement (2026-09-01): v2.0 — `sender` becomes object

The µservice team needs to expose the sender's display name alongside the email address. The clean schema change is:

- Before: `sender: string (email)`.
- After: `sender: { email: string, display_name: string }`.

This is a Field type change (D-8 case #3): the type of `sender` changes from string to object. This is a Major breaking change.

PR-flow at announcement time:
- Author the new OpenAPI: `microservices/oya-workspace-mail/contracts/openapi/2026-09-01.yaml` with `sender` as an object.
- `oasdiff 2026-07-15.yaml 2026-09-01.yaml` returns Major (sender type change).
- `oya-governance-api-semver` lane classifies as `api-major`. Requires:
  - ADR amendment: this is captured in a per-µservice ADR `docs/decisions/ADR-0XXX-oya-workspace-mail-2026-09-01-sender-shape.md` referencing ADR-0258.
  - Migration guide: `docs/api/migration/oya-workspace-mail-2026-07-15-to-2026-09-01.md` authored.
  - Reviewer pair from `council-architecture`.
- Internal mesh: a new mesh major version `v2` is minted. The gRPC service `oya.workspace.mail.v2.MailService` is deployed alongside `v1`. v1 internal callers continue routing to v1; v2 internal callers route to v2.

Update manifest:

```json
{
  "public_api_generations": ["2026-05-20", "2026-07-15", "2026-09-01"],
  "default_public_version": "2026-07-15",
  "mesh_api_major_versions": ["v1", "v2"],
  "default_mesh_major_version": "v1"
}
```

Note: `default_public_version` does NOT immediately advance to `2026-09-01`. The new generation is initially in **Pre-GA** state (D-10). Only after Beta observation and Stable promotion does the default advance.

Per D-5, the deprecation announcement for `2026-05-20` and `2026-07-15` is scheduled.

### B.4 Promotion to Stable (2026-10-01)

After 30 days of Beta observation with at least one external tenant on `2026-09-01`, the team promotes to Stable. Update manifest:

```json
{
  "public_api_generations": ["2026-05-20", "2026-07-15", "2026-09-01"],
  "default_public_version": "2026-09-01",
  "mesh_api_major_versions": ["v1", "v2"],
  "default_mesh_major_version": "v2"
}
```

Behavior post-promotion:
- Tenants pinned to `2026-05-20` continue receiving `sender: string` payload.
- Tenants pinned to `2026-07-15` continue receiving `sender: string` payload.
- Tenants on default now receive `sender: { email, display_name }` payload because the default has shifted to `2026-09-01`.
- Internal mesh default routes to `v2`.

### B.5 Deprecation announcement (2026-10-15)

Per D-5, the team announces deprecation of `2026-05-20` and `2026-07-15`. The deprecation calendar:

- `2026-05-20`: Deprecation announced 2026-10-15. Sunset begins 2027-04-15 (6 months announcement window). Generation removed 2028-04-15 (12 months after sunset begins).
- `2026-07-15`: Deprecation announced 2026-10-15. Sunset begins 2027-04-15 (same). Generation removed 2028-04-15.

Behavior post-announcement:
- Tenants pinned to either deprecated generation continue receiving responses, but every response carries:

```
Deprecation: Thu, 15 Oct 2026 00:00:00 GMT
Sunset: Thu, 15 Apr 2027 00:00:00 GMT
Link: <https://docs.oyatie.com/api/migration/oya-workspace-mail-2026-07-15-to-2026-09-01>; rel="deprecation"
```

- Every request resolved to a deprecated generation emits `ApiDeprecationUsed` audit-chain event.
- Tenant trust portal shows banner: "API generation 2026-07-15 is deprecated; sunset begins 2027-04-15. Migrate to 2026-09-01."
- Tenant admin monthly email includes the deprecation alert.

### B.6 Sunset begins (2027-04-15)

After the 6-month announcement window, requests pinned to `2026-05-20` or `2026-07-15` return `410 Gone`:

```http
HTTP/1.1 410 Gone
Content-Type: application/json
Sunset: Thu, 15 Apr 2027 00:00:00 GMT
Link: <https://docs.oyatie.com/api/sunset/oya-workspace-mail-2026-07-15>; rel="sunset"

{
  "error": {
    "code": "api_version_sunset",
    "message": "API version 2026-07-15 was sunset on 2027-04-15. Migrate to a supported generation.",
    "sunset_at": "2027-04-15T00:00:00Z",
    "supported_generations": ["2026-09-01"],
    "migration_guide": "https://docs.oyatie.com/api/migration/oya-workspace-mail-2026-07-15-to-2026-09-01"
  }
}
```

Tenants pinned to a sunset generation MUST update their pin OR start sending `X-Oyatie-API-Version: 2026-09-01` on requests. The µservice does NOT silently upgrade them; that would be a silent regression (forbidden by `feedback_no_silent_regression`).

### B.7 Generation removed (2028-04-15)

12 months after sunset began, the generation is removed from the manifest:

```json
{
  "public_api_generations": ["2026-09-01"],
  "default_public_version": "2026-09-01",
  "mesh_api_major_versions": ["v2"],
  "default_mesh_major_version": "v2"
}
```

Internal mesh v1 is also removed (no internal caller has used v1 in months; verified by mesh observability).

`ApiGenerationRemoved` audit-chain events emitted for `2026-05-20`, `2026-07-15`, and mesh `v1`.

OpenAPI contracts `2026-05-20.yaml` and `2026-07-15.yaml` are moved to `microservices/oya-workspace-mail/contracts/openapi/_retired/` for historical reference (per ADR-0019 doc catalog protocol).

### B.8 Timeline summary

| Date | Event | Generation state |
|---|---|---|
| 2026-05-20 | µservice ships with `2026-05-20` | Stable |
| 2026-07-15 | Additive `cc` field; new generation `2026-07-15` minted | Both Stable |
| 2026-09-01 | Breaking `sender` shape change; new generation minted | `2026-05-20` Stable, `2026-07-15` Stable, `2026-09-01` Pre-GA → Beta |
| 2026-10-01 | `2026-09-01` promoted to Stable; becomes default | All Stable |
| 2026-10-15 | `2026-05-20` and `2026-07-15` deprecation announced | `2026-05-20` Deprecated, `2026-07-15` Deprecated, `2026-09-01` Stable |
| 2027-04-15 | Sunset begins for the two deprecated generations | `2026-05-20` Sunset, `2026-07-15` Sunset, `2026-09-01` Stable |
| 2028-04-15 | Generations removed from manifest | Only `2026-09-01` Stable |

### B.9 Tenant migration cost analysis

The two tenants from B.1:

**tenant-abc** (pinned to `2026-05-20`):
- Receives deprecation announcement 2026-10-15.
- Has 18 months (until 2028-04-15) to migrate.
- Migration involves: SDK upgrade to `@oyatie/sdk@2.0.0` (or `@1.x` with pin override) + handler code to consume `sender` as object instead of string.
- Estimated effort: 1-2 engineering days per integration point.

**tenant-xyz** (no pin; on default):
- Auto-upgrades to `2026-07-15` on 2026-07-15 (additive; backward-compatible).
- Auto-upgrades to `2026-09-01` on 2026-10-01 (breaking; their handler MUST be ready).
- This is the canonical hazard of NOT pinning. The trust portal warns the tenant at onboarding: "Pinning a specific API version protects your integration from breaking changes. Recommended for production tenants."

The contrast illustrates the value of per-tenant pinning: `tenant-abc` controls their migration timing; `tenant-xyz` is at the mercy of the µservice's default rollover.

## End of ADR-0258
