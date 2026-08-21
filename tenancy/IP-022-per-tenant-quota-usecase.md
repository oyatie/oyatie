---
ip_id: IP-022
microservice: tenancy
bounded_context: per-tenant-quota
layer: usecase
status: in-progress
related_adrs: [ADR-0244, ADR-0248, ADR-0263, ADR-0243]
---

# IP-022 — per-tenant quota usecase

> **Delivery note (2026-08-20).** Implemented in tenancy/core/per-tenant-quota as `tenancy-per-tenant-quota`, collapsed into that ONE crate
> as a module tree rather than this plan's multi-crate fan-out: the capability is capped at 12 crates
> and `Cargo.lock` is a hub path owned by `integ/build`, so neither a new crate nor a new dependency
> was available to this lane. Landed: the class/pack/tenant precedence chain with honest provenance, hard-cap clamping and reserve/commit/release accounting. Deferred and named as a gap in the crate's `lib.rs` header:
> the REST surface and durable persistence. The crate names in the tables below are this plan's original
> proposal, not what shipped.


## A. Problem

Tenancy owns the canonical tenant record, but quota enforcement is currently implied rather than modeled. Downstream services need a single source for per-tenant request rate, storage, API-call, seat, and capability-invocation ceilings. If each service defines quota independently, sandbox tenants can exceed shared substrate limits, enterprise tenants can be throttled incorrectly, and audit events cannot explain why a request was refused.

## B. Approach

Create `oya-tenancy-per-tenant-quota-usecase` to read tenant class, pack, and lifecycle status, resolve quota defaults, apply overrides, and expose quota decisions to Cedar and REST. Enforcement remains local to each service, but the quota source of truth lives in tenancy.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-per-tenant-quota-usecase/Cargo.toml` | create | Usecase crate. |
| `src/resolve_quota.rs` | create | Resolve effective quota from tier defaults plus overrides. |
| `src/update_quota.rs` | create | Operator/tenant-admin bounded updates. |
| `src/decision.rs` | create | Soft/hard limit decision result. |
| `src/ports.rs` | create | `QuotaStore`, `TenantReadPort`, `AuditEmitPort`, `PolicyEvalPort`. |
| `tenancy/capabilities/quota-update.yaml` | align | Capability row for quota mutation. |
| `tenancy/catalog/oya-tenancy-per-tenant-quota-usecase.yaml` | update/create | Catalog evidence. |

## D. Implementation

1. Define quota classes: `RequestRatePerMinute`, `StorageBytes`, `ApiCallsPerDay`, `CapabilityInvocationsPerDay`, `SeatCount`, `WebhookFanoutPerMinute`.
2. Resolve defaults by `plan_tier` from `contracts/openapi/tenancy.yaml`: trial, production, sandbox, internal.
3. Apply pack-specific ceilings so US-HC and regulated packs can force stricter limits than generic production.
4. Implement `resolve_effective_quota(tenant_id, quota_class)` returning hard limit, soft threshold, reset window, source, and audit evidence.
5. Implement `update_quota` with Cedar gate, tier ceiling, reason code, actor, and idempotency key.
6. Emit `oya.tenancy.quota-updated`, `oya.tenancy.quota-breach`, and `oya.tenancy.quota-soft-threshold-crossed`.
7. Add tests for quota persistence across IP rotation, sandbox ceiling, enterprise override, hard-limit refusal, and soft-limit warning.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-per-tenant-quota-usecase --all-features`.
- `tenancy/capabilities/quota-update.yaml` names the usecase owner and audit events.
- Cedar tests cover tenant-admin self-read, substrate-principal update, and forbidden over-ceiling update.
- REST follow-up `IP-026` can expose `GET /v1/tenants/{tid}/quotas` without inventing quota semantics.

## F. Evidence

- `tenancy/contracts/openapi/tenancy.yaml` defines tenant `plan_tier` values consumed by quota defaults.
- `tenancy/manifest.json` lists `quota-update.yaml` and `oya-tenancy-per-tenant-quota-usecase.yaml`.
- `tenancy/dashboards/quota-utilisation.json` is the operational dashboard target.
- `tenancy/policy/action-authorization.cedar` is the mutation authorization surface.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| AWS Service Quotas | Per-account limits with override workflow | Makes quotas tenant-bound and auditable instead of service-local. |
| GCP Quotas | Project/tenant quotas by API and resource | Adds quota classes tied to tenant class and pack. |
| Stripe | Per-account API rate limits | Provides account-style rate-limit source of truth for Oyatie tenants. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `tenancy/IP-022-per-tenant-quota-usecase.md` matched `openapi`; contract files `tenancy/contracts/openapi/tenancy.yaml, tenancy/contracts/asyncapi/tenant-events.yaml, tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Pod runtime tier (per ADR-0338)
- `pod_runtime_tier: 0`
- Runtime: Kata Containers plus Cloud Hypervisor are REQUIRED for this tenant-customer execution path.
- Justification: this IP matched `sandbox`, so tenant-customer or third-party code can enter the execution path.
- Surface evidence: `tenancy/IP-022-per-tenant-quota-usecase.md` plus `crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
