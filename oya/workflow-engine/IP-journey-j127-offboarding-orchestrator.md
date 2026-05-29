---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j127-offboarding-orchestrator
journey_id: j127-dual-tenant-identity-employee-resigns-and-keeps-personal
microservice: workflow-engine
role: offboarding-orchestrator
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0263-observability-emission-contract
depends_on:
  - microservices/tenancy/IP-journey-j127-offboarding-cascade.md
  - microservices/identity/IP-journey-j127-tenant-membership-revocation.md
date: 2026-05-20
owner_team: axis-workflow-engine + axis-hr
parallel_work_compatibility: |
  Independent of j126 evidence-pull orchestration (different
  workflow), j128 personal-tax-workflow, j131 cross-jurisdiction
  audit workflow.
---

# IP-journey-j127-offboarding-orchestrator — Workflow-engine µservice: offboarding cascade orchestrator

## Goal

Implement workflow-engine surfaces that orchestrate the 11-step
offboarding cascade per `handshake.md` §3:

1. RevokeMembership (identity)
2. ArchiveMessenger (messenger)
3. ArchiveMail (mail)
4. TransferDrive (drive)
5. CancelCalendar (calendar)
6. CancelMeet (meet)
7. RevokeWorkplaceIntegration (workplace-integration)
8. RemoteWipeDevice (workplace-integration)
9. RevokeCedarAttributions (policy-engine)
10. RevokeOAuth (identity)
11. EmitCascadeComplete (audit-chain)

Steps 2-10 run in parallel after step 1 completes. Step 11 is gate.

## Data model

Workflow-engine's standard workflow-state model is extended with:

```sql
-- Migration: 2026-05-20-001-offboarding-workflow-template.sql

INSERT INTO workflow_templates (id, name, version, definition_yaml_ref)
VALUES ('offboarding-v1', 'offboarding-cascade', '1.0.0', 'workflow-templates/offboarding-v1.yaml');
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/workflow-engine/src/offboarding/orchestrator.rs` | Cascade orchestration logic | ~340 |
| `microservices/workflow-engine/src/offboarding/parallel_step_runner.rs` | Parallel step execution | ~220 |
| `microservices/workflow-engine/src/offboarding/failure_recovery.rs` | Per-step retry + escalation | ~240 |
| `microservices/workflow-engine/workflow-templates/offboarding-v1.yaml` | Workflow YAML definition | ~180 |
| `microservices/workflow-engine/policy/offboarding-orchestrator.cedar` | Cedar permit | ~30 |
| `microservices/workflow-engine/contracts/proto/offboarding.proto` | gRPC defs | ~100 |
| `microservices/workflow-engine/db/migrations/2026-05-20-001-offboarding-workflow-template.sql` | DDL | ~30 |
| `microservices/workflow-engine/runbooks/offboarding-cascade-failure.md` | Runbook | ~160 |
| `microservices/workflow-engine/runbooks/offboarding-step-rollback.md` | Runbook | ~140 |
| `microservices/workflow-engine/tests/integration/offboarding_orchestrator_test.rs` | Tests | ~440 |
| `microservices/workflow-engine/dashboards/offboarding-cascade-health.json` | Grafana | ~100 |
| `microservices/workflow-engine/slos/offboarding-cascade-completion.openslo.yaml` | SLO | ~40 |

Total approximate: ~2,020 lines.

## Workflow YAML

```yaml
# microservices/workflow-engine/workflow-templates/offboarding-v1.yaml
apiVersion: oya.workflow/v1
kind: WorkflowTemplate
metadata:
  name: offboarding-cascade
  version: 1.0.0
spec:
  inputs:
    - tenant_id
    - principal_id
    - drive_transfer_target
    - reason
  steps:
    - id: revoke_membership
      call: identity.RevokeTenantMembership
      timeout: 5s
      retry: 3
    - id: parallel_cascade
      parallel:
        - call: messenger.ArchiveAllForPrincipal
        - call: mail.ArchiveAllForPrincipal
        - call: drive.TransferOwnership
        - call: calendar.CancelFutureEventsForPrincipal
        - call: meet.CancelFutureSessionsForPrincipal
        - call: workplace-integration.RevokeAllBridges
        - call: workplace-integration.RemoteWipeDevice
        - call: policy-engine.RevokePermitAttributions
        - call: identity.RevokeWorkTenantOAuth
      timeout: 20s
      on_partial_failure: continue
    - id: complete
      call: audit-chain.EmitSealed
      payload:
        class: OffboardingCascadeCompleted
```

## Cedar fragments

```cedar
// offboarding-orchestrator.cedar
permit (
  principal is User,
  action == Action::"workflow_engine.start_workflow",
  resource is WorkflowTemplate
) when {
  resource.id == "offboarding-v1" &&
  principal.audience_type == "B2B_HR_ADMIN"
};
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| Each downstream µservice gRPC call | workflow-engine → various | Per step |
| `audit-chain.EmitSealed` | workflow-engine → audit-chain | Step-level events + completion |

## Latency budget

- Step 1 (sequential): ≤5s
- Steps 2-10 (parallel): ≤20s p99
- Step 11 (sequential): ≤2s
- Total: ≤30s p99

## Test plan

- Test A.1, A.2, A.3 — cascade completes
- Test E.1 — identity revoke retry
- Test E.2 — drive transfer failure doesn't block other steps

## Observability emissions

- `oya_workflow_engine_offboarding_started_total`
- `oya_workflow_engine_offboarding_completed_total{outcome}`
- `oya_workflow_engine_offboarding_step_latency_ms{step}`
- `oya_workflow_engine_offboarding_step_failed_total{step,reason}`

## Acceptance criteria

- Cascade completes ≤30s p99.
- Per-step retry works.
- Partial-failure path is tested.

## Cross-references

- `docs/user-journeys/j127-*/handshake.md` §3
- ADR-0311

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j127-offboarding-orchestrator.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j127-offboarding-orchestrator.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
