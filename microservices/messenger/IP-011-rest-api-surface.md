---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-011-rest-api-surface
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, openapi-conformance, contract-test-pact]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: REST API surface (OpenAPI 3.2.0 conformance)

## Intent

Land the `-rest` crates per BC, bound to the OpenAPI 3.2.0 contract at
`contracts/openapi/messenger.yaml`. Per-endpoint Cedar evaluation; OIDC bearer
authn; X-Scope-OrgID enforcement.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-*-rest/src/{handlers,middleware}.rs` | create per BC |
| `tests/openapi_conformance.rs` | create — schemars-derived schema validated against contracts/openapi/ |

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-channel-store-rest
cargo nextest run -p oya-messenger-message-stream-rest
cargo nextest run --test openapi_conformance
oya gate validate openapi-spec --microservice messenger
```

## Test Plan

- For every `paths:` entry: at least 1 success + 1 401 + 1 403 + 1 404 case.
- Pact contract test against TypeScript SDK consumer (M01+1 SDK).
- Negative: missing X-Scope-OrgID → 401; wrong tenant → 401; Cedar deny → 403.

## Next IP

[`IP-012-websocket-frame-protocol.md`](IP-012-websocket-frame-protocol.md)

## Wave 15 substance conversion — REST API surface

### §A Problem

The messenger service has rich domain slices, but customers and SDKs need one coherent OpenAPI surface with tenant,
context, authn, and Cedar-deny semantics.
This IP closes the gap between domain/usecase crates and `contracts/openapi/messenger.yaml`.

### §B Approach

Generate no business logic from the schema; implement REST handlers that call the kernel/usecase ports and validate
request/response conformance against OpenAPI 3.2.
Every handler enforces OIDC bearer auth, `X-Scope-OrgID`, tenant claims, and channel/personal-DM Cedar rules.

### §C Deliverables

- `src/crates/oya-messenger-*-rest/src/handlers.rs`
- `src/crates/oya-messenger-*-rest/src/middleware.rs`
- `tests/openapi_conformance.rs`
- Pact fixtures for TypeScript SDK consumers

### §D Implementation

1. Bind every `paths:` entry in `contracts/openapi/messenger.yaml` to a handler.
2. Require tenant and active-context headers before reaching usecases.
3. Map auth failures to 401 and Cedar denials to 403 consistently.
4. Keep personal-DM plaintext out of REST responses.
5. Add schema assertions for success and error envelopes.
6. Emit HTTP metrics into messenger SLO series.

### §E Acceptance

Tests must include success, 401, 403, and 404 for each endpoint class, plus a negative cross-tenant read and a
personal-DM admin-disclosure denial.

### §F Evidence

Local anchors: `contracts/openapi/messenger.yaml`, `policy/tenant-scope.cedar`,
`policy/personal-dm-scope.cedar`, `slos/message-send-latency.openslo.yaml`.

### §G Counterparts

Slack Web API and Microsoft Teams Graph-style APIs define enterprise expectations; oyatie closes parity with
contract-tested REST plus stricter dual-context and Cedar guarantees.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/messenger/contracts/openapi/messenger.yaml`, `microservices/messenger/contracts/asyncapi/messenger-events.yaml`, `microservices/messenger/contracts/proto/messenger.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-011-rest-api-surface.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
