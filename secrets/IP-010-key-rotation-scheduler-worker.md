---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-010-key-rotation-scheduler-worker
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [rotation-e2e, cascade-e2e]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: key-rotation-scheduler worker

## Intent

Ship the rotation scheduler: cron-driven rotation of secrets per declared policy; cascade rotation of dependents; stuck-rotation detection.

## ChangeSet boundary

Seven new crates: kernel, domain, usecase, api, adapter, worker, app.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-key-rotation-scheduler-kernel/` | `RotationPolicy`, `RotationJob`, `CascadeDependency`, `RotationStateMachine` |
| `…/oya-cloud-secrets-key-rotation-scheduler-domain/` | pure topo-sort over cascade DAG; jitter arithmetic |
| `…/oya-cloud-secrets-key-rotation-scheduler-usecase/` | orchestrators: schedule, execute, cascade |
| `…/oya-cloud-secrets-key-rotation-scheduler-api/` | typed contracts |
| `…/oya-cloud-secrets-key-rotation-scheduler-adapter/` | OpenBao + audit-emitter adapter wiring |
| `…/oya-cloud-secrets-key-rotation-scheduler-worker/` | long-lived worker binary |
| `…/oya-cloud-secrets-key-rotation-scheduler-app/` | composition root |
| 7× catalog yamls | create |

## Rotation State Machine

```text
Scheduled → InProgress → Rotated → CascadeQueued → CascadeInProgress → Completed
                ↓                          ↓
             Failed (retry ×3 → Overdue → Page)
```

## Acceptance Gates

```bash
cargo nextest run -p 'oya-cloud-secrets-key-rotation-scheduler-*'
# Rotation e2e
cargo nextest run -p oya-cloud-secrets-key-rotation-scheduler-worker --features e2e
# Cascade e2e (chain of 3 dependents)
cargo nextest run --features cascade-e2e
```

## Test Plan

- Single-rotation: schedule → complete within SLA.
- Cascade DAG: rotate root → leaves rotate in topo order.
- Stuck detection: simulated HSM unavailability → RotationOverdue event after T+1d.
- Storm: 100 concurrent rotations → throttle respected.

## Halt Conditions

- Cascade rotation breaks dependent µservices — BLOCKER.

## Next IP

`IP-011-hsm-integration-adapter-hsm.md`

## Wave 15-IP-substance A-G

### A. Problem
Rotation cannot be advisory because stale credentials invalidate the PRD's ISO 27001 and incident-response posture. The service needs deterministic cadence rotation, cascade rotation of dependents, stuck detection, and audit evidence.

### B. Approach
Implement the key-rotation scheduler as a worker-backed bounded context over declared kernel/domain/usecase/adapter/api/app crates. The worker evaluates `RotationPolicy`, emits `RotationJob`, invokes OpenBao/HSM adapters, rotates dependents in topological order, and records outcomes in audit-chain.

### C. Deliverables
- `oya-cloud-secrets-key-rotation-scheduler-{kernel,domain,usecase,api,adapter,worker,app}`.
- Catalog files for each scheduler crate.
- Capability linkage to `capabilities/secret-rotate.yaml`.
- SLO linkage to `slos/key-rotation-correctness.openslo.yaml`.
- Runbooks `runbooks/hsm-key-rotation.md` and `runbooks/rotation-cascade-recovery.md`.

### D. Ordered Implementation Steps
1. Model `RotationPolicy`, `RotationJob`, and `CascadeDependency` in the kernel crate.
2. Implement domain cadence and topological-sort rules.
3. Implement usecase orchestration for single rotate, cascade rotate, and emergency revoke.
4. Add adapter calls to OpenBao, HSM, and audit-emitter ports.
5. Build the worker loop with lease/lock semantics and backpressure.
6. Emit `SecretRotated`, `SecretRevoked`, and `RotationOverdue` events.
7. Add e2e tests for normal rotation, cascade, stuck detection, and storm throttling.

### E. Acceptance
- `cargo nextest run -p 'oya-cloud-secrets-key-rotation-scheduler-*'`.
- `cargo nextest run -p oya-cloud-secrets-key-rotation-scheduler-worker --features e2e`.
- `cargo nextest run --features cascade-e2e`.
- `key-rotation-correctness` SLO records on-cadence completion and pages on overdue jobs.

### F. Evidence
Evidence anchors are `PRD.md` FR-03/FR-08, `manifest.json`, `capabilities/secret-rotate.yaml`, `catalog/oya-cloud-secrets-key-rotation-scheduler-worker.yaml`, `contracts/asyncapi/cloud-secrets-events.yaml`, `slos/key-rotation-correctness.openslo.yaml`, and `runbooks/rotation-cascade-recovery.md`.

### G. Counterpart Comparison
AWS Secrets Manager supports scheduled and Lambda-backed rotation; Vault supports dynamic leases and revocation; GCP/Azure have weaker automatic rotation surfaces. Oyatie's counterpart advantage is cascade rotation plus revocation push, so this worker must prove dependency-aware rotation instead of simple periodic rewrite.

Grep-recognized counterpart anchor: GitHub Actions Secrets is cited for CI rotation drills where test credentials are distributed to workflow jobs and must be rotated through cloud-secrets handles. The runtime comparator remains scheduled rotation in Vault and managed secret stores.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `secrets/contracts/openapi/cloud-secrets.yaml`, `secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `secrets/contracts/proto/cloud-secrets.proto`, `secrets/IP-010-key-rotation-scheduler-worker.md`.

## DR posture (per ADR-0343)

- Target source: `secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `secrets/runbooks/hsm-key-rotation.md`, `secrets/runbooks/openbao-restart.md`, `secrets/manifest.json`, `secrets/IP-010-key-rotation-scheduler-worker.md`.
