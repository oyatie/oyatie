---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-012-per-tenant-namespace-controller
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [tenant-onboard-e2e]
---

# IP-012: per-tenant-namespace-controller

## Intent

React to `TenantRegistered` / `TenantDeprovisioned` / `MicroserviceRegistered` events; provision/seal OpenBao tenant namespaces; emit per-µservice scope policies; cryptographic-erasure on offboard.

## ChangeSet boundary

Six new crates: kernel, domain, usecase, api, adapter, app.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-per-tenant-namespace-controller-kernel/` | `TenantNamespace`, `MicroserviceScope`, `NamespacePolicy` |
| `…/oya-cloud-secrets-per-tenant-namespace-controller-domain/` | pure scope-policy generation logic |
| `…/oya-cloud-secrets-per-tenant-namespace-controller-usecase/` | orchestrate provision + seal + reconcile |
| `…/oya-cloud-secrets-per-tenant-namespace-controller-api/` | typed contracts |
| `…/oya-cloud-secrets-per-tenant-namespace-controller-adapter/` | OpenBao namespace API + event consumer |
| `…/oya-cloud-secrets-per-tenant-namespace-controller-app/` | controller binary |
| 6× catalog yamls | create |

## Acceptance Gates

```bash
cargo nextest run -p 'oya-cloud-secrets-per-tenant-namespace-controller-*'
# Tenant onboard e2e
cargo nextest run --features tenant-onboard-e2e
```

## Test Plan

- TenantRegistered → namespace provisioned within p99 ≤10s.
- TenantDeprovisioned → namespace sealed; cryptographic-erasure scheduled 30d out.
- MicroserviceRegistered → per-µservice scope policy emitted.
- Reconcile after region failover: missing namespaces re-provision.

## Halt Conditions

- Orphan namespaces auto-deleted (not flagged) — BLOCKER (manual review required for data preservation).

## Next IP

`IP-013-audit-emitter-bridge.md`
