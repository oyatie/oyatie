---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j129-warrant-orchestrator
journey_id: j129-court-warrant-pierces-personal-tenant-with-judicial-oversight
microservice: workflow-engine
role: warrant-orchestrator
status: draft
related_adrs:
  - ADR-0312-court-warrant-scoped-piercing
date: 2026-05-20
owner_team: axis-workflow-engine + axis-governance
parallel_work_compatibility: Independent of j127 offboarding orchestrator
---

# IP-journey-j129-warrant-orchestrator — Workflow-engine µservice: warrant lifecycle workflow from submit to expiry

## Goal

Implement workflow that orchestrates the warrant lifecycle:

1. Submit → validate → grant permit (60s soak) → notify subject →
   permit active → permit exercised (across multiple queries) →
   permit expires → audit-chain seal.

## Data model

```sql
CREATE TABLE warrant_lifecycle_workflows (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  warrant_docket TEXT NOT NULL UNIQUE,
  state TEXT NOT NULL CHECK (state IN (
    'SUBMITTED','VALIDATING','VALIDATED','GRANTING','SOAK','ACTIVE','EXERCISED','EXPIRED','REJECTED'
  )),
  permit_id UUID,
  expires_at TIMESTAMPTZ NOT NULL,
  last_state_transition_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## API surface

```protobuf
service WorkflowEngineWarrant {
  rpc SubmitWarrant (SubmitWarrantRequest) returns (SubmitWarrantResponse);
  rpc GetWarrantStatus (GetWarrantStatusRequest) returns (GetWarrantStatusResponse);
}
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/workflow-engine/src/warrant/orchestrator.rs` | Lifecycle orchestration | ~340 |
| `microservices/workflow-engine/src/warrant/state_machine.rs` | State machine impl | ~280 |
| `microservices/workflow-engine/src/warrant/expiry_worker.rs` | Cron worker for expiry | ~180 |
| `microservices/workflow-engine/workflow-templates/warrant-lifecycle-v1.yaml` | Workflow YAML | ~140 |
| `microservices/workflow-engine/policy/warrant-orchestrator.cedar` | Cedar permit | ~30 |
| `microservices/workflow-engine/contracts/proto/warrant_orchestration.proto` | gRPC defs | ~100 |
| `microservices/workflow-engine/db/migrations/2026-05-20-001-warrant-lifecycle.sql` | DDL | ~40 |
| `microservices/workflow-engine/runbooks/warrant-stuck-in-state.md` | Runbook | ~140 |
| `microservices/workflow-engine/tests/integration/warrant_lifecycle_test.rs` | Tests | ~400 |
| `microservices/workflow-engine/dashboards/warrant-lifecycle-health.json` | Grafana | ~80 |
| `microservices/workflow-engine/slos/warrant-lifecycle-completion.openslo.yaml` | SLO | ~40 |

Total approximate: ~1,770 lines.

## Cedar fragments

```cedar
// warrant-orchestrator.cedar
permit (
  principal == Service::"legal-process-api",
  action == Action::"workflow_engine.start_warrant_lifecycle",
  resource is Warrant
);
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| `governance.ValidateWarrant` | workflow-engine → governance | Validation phase |
| `tenancy.GrantCrossTenantPermit` | workflow-engine → tenancy | Permit creation |
| `identity.DispatchWarrantSubjectNotification` | workflow-engine → identity | Notification |
| `tenancy.ExpireCrossTenantPermit` | workflow-engine → tenancy | Expiry cron |
| `audit-chain.EmitSealedDualTenant` | workflow-engine → audit-chain | Per state transition |

## Latency budget

- Full lifecycle (excluding active window): ≤2 minutes
- Active window: warrant-defined (5 days in test fixture)

## Test plan

- Test A.1, A.2, C.1 — full lifecycle

## Observability emissions

- `oya_workflow_engine_warrant_state_transitions_total{from_state, to_state}`
- `oya_workflow_engine_warrant_duration_seconds{phase}`

## Acceptance criteria

- State machine correctly transitions.
- Expiry cron triggers on time.

## Cross-references

- `docs/user-journeys/j129-*/handshake.md`

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
- Trigger evidence: `microservices/workflow-engine/IP-journey-j129-warrant-orchestrator.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j129-warrant-orchestrator.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
