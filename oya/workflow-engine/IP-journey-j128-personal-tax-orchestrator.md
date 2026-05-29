---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j128-personal-tax-orchestrator
journey_id: j128-auditor-personal-side-uses-workflow-studio-for-family-taxes
microservice: workflow-engine
role: personal-tax-orchestrator
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0245-substrate-vs-product-layering
date: 2026-05-20
owner_team: axis-workflow-engine + axis-personal-tenant
parallel_work_compatibility: Independent of j126/j127 workflows
---

# IP-journey-j128-personal-tax-orchestrator — Workflow Engine: per-step orchestration with pause-for-review for personal-tenant tax workflow

## Goal

Implement workflow-engine surfaces specific to personal-tenant tax
workflows:

1. **Per-step Cedar evaluation** — every step in the DAG re-evaluates
   tenant-scope per ADR-0246 amendment defense-in-depth.
2. **Pause-for-review checkpoint primitive** — workflow halts at
   designated step; resumes only on user input.
3. **Per-step audit-chain emission** — every step's success/failure
   sealed to the personal-tenant chain.
4. **Idempotency** — re-running a tax workflow does NOT double-submit
   to IRS.

## Data model

```sql
CREATE TABLE workflow_run_pause_points (
  run_id UUID NOT NULL,
  step_id TEXT NOT NULL,
  paused_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  pause_reason TEXT NOT NULL,
  review_payload_ref TEXT,
  resumed_at TIMESTAMPTZ,
  resume_choice TEXT CHECK (resume_choice IN ('approve','cancel','edit')),
  PRIMARY KEY (run_id, step_id)
);

CREATE TABLE workflow_submission_idempotency (
  workflow_id TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  submission_kind TEXT NOT NULL CHECK (submission_kind IN ('irs-mef','va-dor','ca-ftb')),
  tax_year INT NOT NULL,
  submitted_at TIMESTAMPTZ NOT NULL,
  confirmation_hash TEXT NOT NULL,
  PRIMARY KEY (workflow_id, tenant_id, submission_kind, tax_year)
);
```

## API surface

```protobuf
service WorkflowEnginePersonal {
  rpc StartWorkflow (StartWorkflowRequest)
      returns (StartWorkflowResponse);
  rpc ResumeFromPause (ResumeFromPauseRequest)
      returns (ResumeFromPauseResponse);
  rpc GetRunStatus (GetRunStatusRequest)
      returns (GetRunStatusResponse);
}
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/workflow-engine/src/personal/orchestrator.rs` | Orchestration | ~340 |
| `microservices/workflow-engine/src/personal/pause_for_review.rs` | Pause primitive | ~220 |
| `microservices/workflow-engine/src/personal/idempotency.rs` | Idempotency guard | ~200 |
| `microservices/workflow-engine/policy/personal-workflow-run.cedar` | Cedar permit | ~30 |
| `microservices/workflow-engine/contracts/proto/personal.proto` | gRPC defs | ~120 |
| `microservices/workflow-engine/db/migrations/2026-05-20-001-workflow-pause-and-idempotency.sql` | DDL | ~50 |
| `microservices/workflow-engine/runbooks/tax-workflow-resume-from-pause.md` | Runbook | ~140 |
| `microservices/workflow-engine/tests/integration/personal_tax_test.rs` | Tests | ~440 |
| `microservices/workflow-engine/dashboards/personal-workflow-pause-rates.json` | Grafana | ~80 |
| `microservices/workflow-engine/slos/personal-workflow-step-latency.openslo.yaml` | SLO | ~40 |

Total approximate: ~1,660 lines.

## Cedar fragments

```cedar
// personal-workflow-run.cedar
permit (
  principal is User,
  action == Action::"workflow_engine.run_workflow",
  resource is Workflow
) when {
  principal.tenant == resource.tenant &&
  principal.id == resource.owner_principal_id
};
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| Per-step gRPC calls | workflow-engine → various | Each downstream µservice |
| `audit-chain.EmitSealed` | workflow-engine → audit-chain | Per step |
| `mail.SendNotification` | workflow-engine → mail | Pause + completion |

## Latency budget

- Step transition: ≤80ms p99
- Pause persistence: ≤50ms p99
- Resume from pause: ≤200ms p99

## Test plan

- Test A.1 — workflow completes
- Test E.1 — Vanguard failure pauses for retry

## Observability emissions

- `oya_workflow_engine_personal_tax_steps_total{tenant_id, step, outcome}`
- `oya_workflow_engine_personal_tax_pause_duration_ms`
- `oya_workflow_engine_submission_idempotency_hit_total{submission_kind}`

## Acceptance criteria

- Idempotency prevents double-submit.
- Pause primitive works.
- Per-step Cedar evaluation passes.

## Cross-references

- `docs/user-journeys/j128-*/handshake.md`
- ADR-0246 amendment

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
- Trigger evidence: `microservices/workflow-engine/IP-journey-j128-personal-tax-orchestrator.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j128-personal-tax-orchestrator.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.

## Pod runtime tier (per ADR-0338)

- Authority: ADR-0338.
- `pod_runtime_tier`: `0`.
- Justification: tenant-customer code exists in this IP execution path; Kata Containers + Cloud Hypervisor are required.
- Surface evidence: `microservices/workflow-engine/IP-journey-j128-personal-tax-orchestrator.md`, `microservices/workflow-engine/manifest.json`; trigger terms `workflow-studio`.
