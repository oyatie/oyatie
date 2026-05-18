---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-application
deciders: axis-application, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/PRD.md
  - microservices/application/capacity-model.md
  - microservices/application/policy/data-residency.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (application µservice)

## Purpose

Specify how Application Shell handles:

1. **Backfill** — onboarding a new tenant or product: route registrations,
   module manifests, and ontology bindings retroactively reach a
   consistent state.
2. **Replay** — re-processing a stream of Workflow events (session
   lifecycle, module load, route deny) after schema migration, evaluator
   defect, or pack relocation.

## Backfill

### Tenant onboarding backfill

When a new tenant is provisioned in `tenancy`:

1. `TenantProvisioned` event consumed by Application Shell's
   `tenant-context` BC.
2. Application Shell:
   - Creates Postgres tenant_id row in shell-state DB (RLS scope established).
   - Pre-registers the tenant's pack-residency binding.
   - Allocates a Valkey session-namespace.
   - Subscribes to the tenant's enabled-product set; pre-fetches module
     manifests for each enabled product.
3. No retroactive session creation (sessions are user-driven).

### Product enablement backfill

When a tenant enables a new product:

1. `ProductEnabled` event consumed by `module-loader`.
2. Application Shell:
   - Fetches the product's current module manifest (signed).
   - Validates SRI + Ed25519 signature.
   - Registers product routes in `RouteRegistration` Postgres table
     (Cedar scope binding).
   - Pre-warms CDN origin for the product's bundle.
   - Emits `ModuleEnabled` audit event.
3. No retroactive ModuleLoaded events for the tenant — modules load on
   first user navigation.

### Backfill rate-limits

- 1 tenant onboarding / minute / pack (to avoid Postgres ingester spike).
- 5 product enablements / minute / tenant (to avoid CDN origin overload).

## Replay

### Triggers

| Trigger | Re-processing required |
|---|---|
| Audit-chain seal worker outage (gap in seal record) | Re-seal all unsealed session / module / route events since gap |
| Cedar policy schema migration | Re-evaluate access for any in-flight session (forced re-sign-in is the simpler answer) |
| Module-loader signature-verify defect found (FN: valid manifest rejected) | Re-emit ModuleLoaded for previously rejected loads |
| Module-loader signature-verify defect found (FP: invalid manifest accepted) | Mass force-rollback + audit |
| Pack relocation (tenant moves jurisdiction) | Re-write tenant_id binding; force re-sign-in; emit residency-change audit |

### Replay procedure

1. Identify scope: which event-kind + time range + tenant/pack scope.
2. Drain consumer: pause downstream consumers (audit-chain seal worker) to
   prevent double-emit.
3. Replay: read events from event-store; re-apply use-case orchestrator
   in idempotent mode (`replay=true` tag).
4. Verify: assert post-replay state matches expected invariants (e.g.,
   `route_audit` row count matches).
5. Resume consumers.
6. Audit: emit `ReplayExecuted` event with scope + reason + operator.

### Idempotency

All event handlers in Application Shell carry an `event_id` UUIDv7 and
write to event-store with `INSERT … ON CONFLICT (event_id) DO NOTHING`.
Replay never produces duplicate effects.

## Cost

| Operation | Cost class |
|---|---|
| Tenant backfill | O(1) per tenant; ≤ 200 ms |
| Product enablement | O(routes_in_product) per enable; ≤ 1 s |
| Replay (audit re-seal) | O(events_in_window) ≤ 10 k/s per worker |
| Replay (forced re-sign-in) | O(active_sessions); user-visible |

## Verification

| AC | Test |
|---|---|
| Backfill-AC-01 | New tenant reaches consistent state in ≤ 5 s | drill |
| Backfill-AC-02 | Product enablement registers all routes + manifest in ≤ 10 s | drill |
| Replay-AC-01 | Re-seal of 100 k events completes ≤ 60 s | benchmark |
| Replay-AC-02 | Idempotent: replay produces same final state as live | property test |

## References

- ADR-0028 audit chain.
- ADR-0123 cross-product auth.
- `microservices/observability/backfill-replay.md` (precedent).
