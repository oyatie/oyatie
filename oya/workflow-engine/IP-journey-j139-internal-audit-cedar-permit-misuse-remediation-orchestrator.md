---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j139-workflow-engine-remediation-orchestrator
journey_id: j139-internal-audit-policy-violation-cedar-permit-misuse
microservice: workflow-engine
role: remediation-orchestrator
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-workflow-engine + axis-internal-audit
parallel_work_compatibility: extends j138 investigation-orchestrator with policy-violation-specific remediation
related_adrs: [ADR-0310, ADR-0311, ADR-0307, ADR-0243, ADR-0028, ADR-0145]
depends_on:
  - microservices/workflow-engine/IP-journey-j138-corporate-audit-investigation-case-orchestrator.md
---

# IP-journey-j139-workflow-engine-remediation-orchestrator — Workflow Engine: policy-violation remediation orchestrator

## Goal

Extend the j138 investigation-orchestrator with a new remediation
workflow template specific to policy-violation investigations. The
template orchestrates:

1. Revoke offending permit overlays (atomic batch).
2. Update Cedar policy (with dual-control + audit-committee co-sign).
3. Suspend offending principal's role (per ADR-0311).
4. Notify HR (need-to-know).
5. Engage outside counsel.

## Data model

Reuses j138 schemas. Adds:

```sql
CREATE TABLE workflow_engine.remediation_workflows (
  workflow_id TEXT PRIMARY KEY,
  investigation_case_id TEXT NOT NULL,
  remediation_type TEXT NOT NULL,        -- 'policy_violation', 'fraud', 'dlp_egress'
  actions JSONB NOT NULL,
  atomic_batch BOOLEAN DEFAULT true,
  executed_at TIMESTAMPTZ,
  reverted_at TIMESTAMPTZ,
  audit_seal_id TEXT NOT NULL
);
```

## API surface (gRPC)

```protobuf
service RemediationOrchestrator {
  rpc CreateRemediationWorkflow (CreateRemediationWorkflowRequest) returns (CreateRemediationWorkflowResponse);
  rpc ExecuteRemediation (ExecuteRemediationRequest) returns (ExecuteRemediationResponse);
  rpc RevertRemediation (RevertRemediationRequest) returns (RevertRemediationResponse);
  rpc ListAtomicBatch (ListAtomicBatchRequest) returns (ListAtomicBatchResponse);
}
```

## Cedar policy

```cedar
@id("workflow-engine-execute-remediation-v1")
permit (
  principal,
  action == Action::"workflow_engine.execute_remediation",
  resource is RemediationWorkflow
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  resource.investigation_case.requestor_principal == principal.id &&
  resource.investigation_case.severity in ["HIGH", "CRITICAL"] &&
  context.dual_control_approval_at != null &&
  context.outside_counsel_concurrence != null
};
```

## Workflow template

```yaml
workflow:
  id: policy-violation-remediation-v1
  stages:
    - id: 1-validate-prereqs
      action: validate_investigation_state
      required_state: REMEDIATION
    - id: 2-revoke-overlays
      action: governance.revoke_permit_overlay_batch
      atomic: true
    - id: 3-update-cedar-policy
      action: governance.update_cedar_policy
      dual_control: true
      cosign: audit_committee_chair
    - id: 4-suspend-principal-role
      action: identity.suspend_principal_role
      dual_control: true
    - id: 5-notify-hr
      action: community.hr_reporting.post_suspension_ticket
    - id: 6-engage-counsel
      action: legal-counsel-gateway.request_subpoena_preparation
      optional: false
    - id: 7-seal-remediation-batch
      action: audit-chain.batch_seal_remediation_actions
    - id: 8-transition-case-to-external
      action: workflow-engine.investigation_orchestrator.transition_state(EXTERNAL)
```

Stages 2-6 are an ATOMIC BATCH: if any fails, all roll back. Stage 7
seals the batch. Stage 8 transitions the case.

## Integration contracts

### Upstream

- ops-dashboard.audit-pane (Sam triggers remediation).

### Downstream

- governance.PermitOverlayRegistry.RevokePermitOverlay (x5).
- governance.CedarPolicy.UpdateCedarPolicy.
- identity.PrincipalRoleManagement.SuspendPrincipalRole.
- community.HRReporting.PostSuspensionTicket.
- legal-counsel-gateway.RequestSubpoenaPreparation.
- audit-chain.BatchSealLeaves.

## Implementation notes

### Atomic batch semantics

The workflow-engine implements two-phase commit across the
downstream services. Phase 1: each service ACK's the prepare-call.
Phase 2: each service commits. If any service returns NACK on
prepare, all are rolled back. Each service implements idempotent
prepare/commit/rollback semantics.

### Reversibility window

Remediation is reversible for 30 days post-execution (via
RevertRemediation RPC). Reversal requires:
- Audit-committee dual-control.
- Outside-counsel attestation that the matter is resolved.
- Audit-chain sealed reversal events.

### Performance budget

- `ExecuteRemediation` p95 ≤ 10s for 6-action batch.
- `RevertRemediation` p95 ≤ 8s.

## Test plan

Unit tests:
- `test_remediation_atomic_batch_succeeds_when_all_actions_succeed`
- `test_remediation_atomic_batch_rolls_back_on_any_failure`
- `test_reversibility_within_30d`
- `test_reversibility_blocked_post_criminal_referral`
- `test_dual_control_plus_counsel_required`

## Build sequence

1. Schema migration.
2. Cedar policy.
3. Two-phase commit logic across downstream services.
4. Workflow DSL template.
5. Audit-chain seal.
6. Reversibility flow.
7. Tests.

## Acceptance gates

All tests PASS; Cedar lint clean; schema applied; code review by
axis-workflow-engine + axis-internal-audit + axis-governance.

## Operational notes

Owner: axis-workflow-engine.

## Compliance / packs

Same as j138.

## Cross-microservice port declaration

Per ADR-0145, `RemediationOrchestrator` in
`oyatie.workflow_engine.remediation.v1`.

## Roll-out plan

Five-phase rollout.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Atomic batch partial commit | CRITICAL | 2PC with prepare/commit/rollback semantics |
| Reversibility window race | HIGH | Per-action TTL + audit log |
| Wrong principal targeted in remediation | CRITICAL | Confirmation modal + dual-control |
| Cedar policy update propagation lag | HIGH | Atomic deploy + post-deploy validation |

## Definition of done

- Service live behind flag.
- Kemi-remediation 6-action batch verified atomic.
- Roll-back verified within 30d.
- Personal-tenant boundary maintained.
- All actions audit-sealed.

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.
