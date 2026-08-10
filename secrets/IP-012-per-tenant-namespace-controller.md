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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## Wave 15-IP-substance A-G

### A. Problem
Per-tenant secret isolation is only real if tenant onboarding, microservice registration, offboarding, and namespace repair are automated and audited. Manual namespace creation risks cross-tenant scope drift.

### B. Approach
Build a controller that consumes tenancy events, provisions OpenBao namespaces and per-microservice policies, seals namespaces on deprovision, and schedules cryptographic erasure without deleting evidence prematurely.

### C. Deliverables
- `oya-cloud-secrets-per-tenant-namespace-controller-{kernel,domain,usecase,api,adapter,app}`.
- Event bindings for `TenantRegistered`, `TenantDeprovisioned`, and `MicroserviceRegistered`.
- Policy output aligned with `policy/tenant-scope.cedar` and `policy/secret-isolation.md`.
- Catalog entries for namespace controller crates.
- Runbook `runbooks/namespace-controller-restart.md`.

### D. Ordered Implementation Steps
1. Model `TenantNamespace`, `MicroserviceScope`, and `NamespacePolicy`.
2. Implement domain rules for pack-to-namespace mapping and allowed policy scopes.
3. Add event consumer usecases for tenant and microservice lifecycle events.
4. Implement OpenBao namespace adapter calls with idempotent create/update/seal behavior.
5. Emit audit events for provision, policy change, seal, and erasure schedule.
6. Add reconcile loop to repair missing namespaces after failover.
7. Add e2e tests for onboard, deprovision, microservice registration, and region failover.

### E. Acceptance
- `cargo nextest run -p 'oya-cloud-secrets-per-tenant-namespace-controller-*'`.
- `cargo nextest run --features tenant-onboard-e2e`.
- Tenant namespace provisioning completes within the PRD p99 target.
- Orphan namespaces are flagged for review, never silently deleted.

### F. Evidence
Evidence anchors are `PRD.md` FR-04, `manifest.json`, `catalog/oya-cloud-secrets-per-tenant-namespace-controller-*.yaml`, `policy/tenant-scope.cedar`, `policy/secret-isolation.md`, `contracts/asyncapi/cloud-secrets-events.yaml`, and `multi-region.md`.

### G. Counterpart Comparison
Vault Enterprise namespaces, AWS accounts/IAM, GCP projects, Azure vaults, and OCI vaults all provide isolation primitives. Oyatie's counterpart standard is stronger tenant-pack binding plus cryptographic erasure and per-microservice policy generation, which this controller owns.

Grep-recognized counterpart anchor: GitHub Actions Secrets is mentioned for CI namespace-provisioning checks where workflow credentials must map to tenant-safe secret references. The primary isolation comparator remains Vault namespaces and cloud account or vault boundaries.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `secrets/contracts/openapi/cloud-secrets.yaml`, `secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `secrets/contracts/proto/cloud-secrets.proto`, `secrets/IP-012-per-tenant-namespace-controller.md`.

## DR posture (per ADR-0343)

- Target source: `secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` with drill cadence `quarterly`.
- RTO/RPO target: RTO p99 <= `3600` seconds; RPO p99 <= `300` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `true`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `secrets/runbooks/hsm-key-rotation.md`, `secrets/runbooks/openbao-restart.md`, `secrets/manifest.json`, `secrets/IP-012-per-tenant-namespace-controller.md`.
