---
ip_id: IP-026
microservice: tenancy
bounded_context: per-tenant-quota
layer: rest
status: planned
related_adrs: [ADR-0253, ADR-0258, ADR-0243, ADR-0263]
---

# IP-026 — quota REST + SDK

## A. Problem

`IP-022` defines quota decisions, but tenant admins and substrate services need a stable API to inspect effective quotas, request bounded updates, and understand refusals. Without REST and SDK contracts, callers will either cache stale limits or create service-local quota mutation paths that bypass tenancy audit and Cedar policy.

## B. Approach

Extend tenancy REST and Rust SDK surfaces with quota read/update operations backed by `oya-tenancy-per-tenant-quota-usecase`. The API returns both configured and effective quota values, explains tier and pack sources, and emits audit-chain events for every mutation or hard-limit breach.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/contracts/openapi/tenancy.yaml` | update | Add quota routes and schemas. |
| `microservices/tenancy/src/crates/oya-tenancy-quota-rest/Cargo.toml` | create | REST crate. |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-sdk/src/quota.rs` | update/create | Rust SDK quota client. |
| `src/routes.rs` | create | REST handlers for read/update. |
| `src/sdk_contract_tests.rs` | create | SDK/OpenAPI compatibility tests. |
| `microservices/tenancy/capabilities/quota-update.yaml` | align | Capability references API and SDK. |

## D. Implementation

1. Add `GET /v1/tenants/{tenant_id}/quotas` returning all effective quota classes.
2. Add `GET /v1/tenants/{tenant_id}/quotas/{quota_id}` for a single class.
3. Add `PATCH /v1/tenants/{tenant_id}/quotas/{quota_id}` with idempotency key, reason, requested limit, and actor context.
4. Return `effective_limit`, `soft_threshold`, `hard_limit`, `reset_window`, `source`, `pack_override`, and `last_updated_by`.
5. Enforce Cedar: tenant admins can read own quota; substrate principals can update inside tier ceiling; over-ceiling requests return 403 with audit evidence.
6. Add SDK methods `list_quotas`, `get_quota`, and `request_quota_update` using the existing tenancy SDK auth pattern.
7. Add contract tests to ensure OpenAPI enum values match `IP-022` quota classes.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-quota-rest --all-features`.
- OpenAPI validates with quota routes and schemas.
- SDK contract tests prove no mismatch between OpenAPI quota class enum and Rust SDK enum.
- Cedar-denied update emits `oya.tenancy.quota-update-denied`; accepted update emits `oya.tenancy.quota-updated`.
- `microservices/tenancy/dashboards/quota-utilisation.json` can consume the route names and quota class labels.

## F. Evidence

- `microservices/tenancy/IP-022-per-tenant-quota-usecase.md` owns quota semantics.
- `microservices/tenancy/contracts/openapi/tenancy.yaml` is the REST authority.
- `microservices/tenancy/manifest.json` lists `quota-update.yaml`.
- `microservices/tenancy/dashboards/quota-utilisation.json` is the quota operations dashboard.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| AWS Service Quotas | API-visible account quotas and increase workflow | Gives tenants and operators a stable quota read/update surface. |
| GCP Quotas | Project quota API and override visibility | Returns effective limit plus source, not only a raw number. |
| Stripe | Dashboard and API-visible rate-limit behavior | Lets tenants understand and audit quota decisions before requests fail. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/tenancy/IP-026-quota-rest-and-sdk.md` matched `openapi`; contract files `microservices/tenancy/contracts/openapi/tenancy.yaml, microservices/tenancy/contracts/asyncapi/tenant-events.yaml, microservices/tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
