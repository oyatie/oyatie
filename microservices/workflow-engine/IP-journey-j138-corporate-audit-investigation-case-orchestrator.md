---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j138-workflow-engine-investigation-case-orchestrator
journey_id: j138-corporate-audit-fraud-investigation-via-pattern-detection
microservice: workflow-engine
role: investigation-case-orchestrator
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-workflow-engine + axis-internal-audit
parallel_work_compatibility: extends j137 audit-sample-planner with investigation lifecycle
related_adrs: [ADR-0310, ADR-0311, ADR-0307, ADR-0243, ADR-0028, ADR-0145]
depends_on:
  - microservices/workflow-engine/IP-journey-j137-corporate-internal-audit-sox-controls-test-execution-log-reader.md
  - microservices/identity/IP-journey-j137-corporate-internal-audit-sox-controls-test-permit-resolver.md
---

# IP-journey-j138-workflow-engine-investigation-case-orchestrator — Workflow Engine: investigation lifecycle orchestrator

## Goal

Implement `workflow-engine.investigation_orchestrator` — the
workflow template + driver that handles investigation cases per
ADR-0310. Distinct from j137's `audit_sample_planner` because:

- Triggered by detection signals OR by Sam's manual creation.
- Longer time horizon (14d vs 5d).
- Includes ACTIONS phase (suspend, freeze, notify-HR, subpoena-request).
- State machine matches ADR-0310 investigation lifecycle.

## Data model

| Object | Storage | Schema | TTL |
|---|---|---|---|
| `InvestigationCase` | Postgres `workflow_engine.investigation_cases` | `schemas/investigation-case.json` | 7y |
| `InvestigationEvidenceRequest` | Postgres `workflow_engine.investigation_evidence_requests` | per-request | 7y |
| `InvestigationFinding` | Postgres `workflow_engine.investigation_findings` | per-finding | 7y |
| `InvestigationAction` | Postgres `workflow_engine.investigation_actions` | per-action | 7y |

## Schema mapping

```sql
CREATE TABLE workflow_engine.investigation_cases (
  case_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  originating_alert_id TEXT,
  requestor_principal TEXT NOT NULL,
  permit_batch_ref TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('ALERT','TRIAGE','PERMIT_PENDING','ACTIVE','EVIDENCE','INTERVIEW','REMEDIATION','EXTERNAL','CLOSED','EXPIRED')),
  classification_window_start TIMESTAMPTZ,
  classification_window_end TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  closed_at TIMESTAMPTZ,
  personal_tenant_deny_count INTEGER DEFAULT 0
);

CREATE TABLE workflow_engine.investigation_findings (
  finding_id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL REFERENCES workflow_engine.investigation_cases(case_id),
  finding_type TEXT NOT NULL,
  severity TEXT NOT NULL,
  confidence_pct INTEGER,
  description TEXT NOT NULL,
  evidence_refs TEXT[],
  status TEXT NOT NULL CHECK (status IN ('OPEN','RESOLVED','ESCALATED')),
  sealed_at TIMESTAMPTZ,
  audit_seal_id TEXT
);

CREATE TABLE workflow_engine.investigation_actions (
  action_id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL,
  action_type TEXT NOT NULL CHECK (action_type IN ('suspend_vendor','freeze_invoice','suspend_principal_role','notify_hr','request_subpoena_preparation','unsuspend_vendor')),
  target TEXT NOT NULL,
  executed_by_principal TEXT NOT NULL,
  dual_control_signer TEXT,
  executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  audit_seal_id TEXT NOT NULL,
  reversible BOOLEAN DEFAULT true,
  reversed_at TIMESTAMPTZ
);
```

## API surface (gRPC)

```protobuf
service InvestigationOrchestrator {
  rpc CreateInvestigationCase (CreateInvestigationCaseRequest) returns (CreateInvestigationCaseResponse);
  rpc TransitionState (TransitionStateRequest) returns (TransitionStateResponse);
  rpc FileFinding (FileFindingRequest) returns (FileFindingResponse);
  rpc ExecuteAction (ExecuteActionRequest) returns (ExecuteActionResponse);
  rpc HandToExternalCounsel (HandToExternalCounselRequest) returns (HandToExternalCounselResponse);
  rpc CloseCase (CloseCaseRequest) returns (CloseCaseResponse);
}
```

## Cedar policy

```cedar
@id("workflow-engine-investigation-create-v1")
permit (
  principal,
  action == Action::"workflow_engine.investigation_orchestrator.create_case",
  resource is Tenant
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  resource.tenant_id == principal.tenant_id
};

@id("workflow-engine-investigation-execute-action-v1")
permit (
  principal,
  action == Action::"workflow_engine.investigation_orchestrator.execute_action",
  resource is InvestigationAction
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  resource.case.requestor_principal == principal.id &&
  (resource.action_type in ["suspend_vendor","freeze_invoice"] ? context.investigation_severity in ["HIGH","CRITICAL"] : true)
};
```

## State machine

```
       ┌────────┐
       │ ALERT  │ ──(triage)──► TRIAGE ──(open)──► PERMIT_PENDING ──(co-sign)──► ACTIVE
       └────────┘                                                                  │
                                                                                   ▼
              ┌────────────────────────────────────────────────────────────► EVIDENCE
              │                                                                    │
              │                                                                    ▼
              │                                                              INTERVIEW
              │                                                                    │
              │                                                                    ▼
              │                                                            REMEDIATION
              │                                                                    │
              ┴──(escalate)──► EXTERNAL ◄───────(handoff)─────────────────────────┤
                                  │                                                ▼
                                  ▼                                            CLOSED
                                                                               (or)
                                                                            EXPIRED (timeout)
```

## Integration contracts

### Upstream

- `ops-dashboard.audit-pane` (Sam's entry point).
- `observability.detection.SignalDispatcher` (auto-trigger).

### Downstream

- `payments.VendorAdministration` (suspend / freeze).
- `identity.PrincipalRoleManagement` (suspend role).
- `community.HRReporting` (notify HR).
- `legal-counsel-gateway` (subpoena request).
- `audit-chain.SealLeaf`.

## Performance budget

- Case creation p95 ≤ 1s.
- Evidence-pull p95 ≤ 60s per source.
- Action-execute p95 ≤ 3s.

## Test plan

See integration-test-plan.md §9, §6.

Unit tests:
- `test_state_transitions_monotonic`
- `test_action_dual_control_required_for_suspend`
- `test_close_case_with_pending_actions_rejected`
- `test_finding_required_for_action`
- `test_personal_tenant_deny_count_propagated`

## Build sequence

1. Schema migrations.
2. State machine implementation.
3. Cedar policies.
4. gRPC services.
5. Workflow DSL template `investigation-orchestrator-v1`.
6. Wire downstream actions.
7. Audit-chain seal per transition.
8. Tests.

## Acceptance gates

- All tests PASS; state machine matches ADR-0310.
- Cedar lint clean.
- Migrations applied.
- Code review: axis-workflow-engine + axis-internal-audit.

## Operational notes

- Owner: axis-workflow-engine.
- Pager: `oya-workflow-engine-investigation`.
- Dashboards: `investigation-case-lifecycle-state-rate`,
  `action-execute-latency`.

## Compliance / packs

- `pack-corporate-internal-audit-baseline` + investigation-specific
  overlay `pack-investigation-protocol-v1`.

## Cross-microservice port declaration

Per ADR-0145, `InvestigationOrchestrator` in
`oyatie.workflow_engine.investigation.v1`.

## Roll-out plan

Same five-phase rollout.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| State-machine inconsistency | CRITICAL | Property test on transitions |
| Wrong action executed (suspend wrong vendor) | CRITICAL | Confirmation modal + dual-control |
| Case not closing properly | MEDIUM | Auto-expire + reminder cron |
| Permit leak across cases | CRITICAL | Per-case permit batch + revocation on close |

## Definition of done

- Service live in production behind flag.
- All tests PASS.
- AcmeWire investigation end-to-end verified.
- All actions trigger correct downstream effects.
- Personal-tenant deny propagates through full lifecycle.

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j138-corporate-audit-investigation-case-orchestrator.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
