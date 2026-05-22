---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j140-workflow-engine-dlp-investigation
journey_id: j140-internal-audit-data-loss-prevention-egress-trip
microservice: workflow-engine
role: dlp-investigation
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-workflow-engine + axis-internal-audit
parallel_work_compatibility: extends j138/j139 investigation-orchestrator with DLP-specific evidence sources + light-touch remediation
related_adrs: [ADR-0310, ADR-0311, ADR-0307, ADR-0243, ADR-0145]
depends_on:
  - microservices/workflow-engine/IP-journey-j138-corporate-audit-investigation-case-orchestrator.md
  - microservices/workflow-engine/IP-journey-j139-internal-audit-cedar-permit-misuse-remediation-orchestrator.md
---

# IP-journey-j140-workflow-engine-dlp-investigation — Workflow Engine: DLP investigation orchestrator + light-touch remediation template

## Goal

Add a new workflow template `dlp-investigation-v1` that orchestrates
investigation of DLP trips with both heavy-touch (suspension + counsel)
and light-touch (training + UX) remediation paths based on evidence.

## Data model

Reuses j138 schemas. New:

```sql
CREATE TABLE workflow_engine.dlp_investigation_workflows (
  workflow_id TEXT PRIMARY KEY,
  investigation_case_id TEXT NOT NULL,
  egress_event_id TEXT NOT NULL,
  evidence_pull_status TEXT NOT NULL,
  interview_outcome TEXT,
  remediation_class TEXT CHECK (remediation_class IN ('LIGHT_TOUCH', 'HEAVY_TOUCH')),
  counsel_attestation TEXT,
  audit_seal_id TEXT NOT NULL
);
```

## API surface (gRPC)

```protobuf
service DLPInvestigationOrchestrator {
  rpc OpenDLPInvestigation (OpenDLPInvestigationRequest) returns (OpenDLPInvestigationResponse);
  rpc DetermineRemediationClass (DetermineRemediationClassRequest) returns (DetermineRemediationClassResponse);
  rpc ExecuteLightTouchRemediation (ExecuteLightTouchRemediationRequest) returns (ExecuteLightTouchRemediationResponse);
  rpc ExecuteHeavyTouchRemediation (ExecuteHeavyTouchRemediationRequest) returns (ExecuteHeavyTouchRemediationResponse);
}
```

## Workflow template

```yaml
workflow:
  id: dlp-investigation-v1
  stages:
    - id: 1-open-case
      action: investigation_orchestrator.create_case
    - id: 2-cosign
      action: wait_for_dual_control
    - id: 3-pull-dlp-event
      action: drive.read_dlp_event
    - id: 4-pull-drive-activity
      action: drive.read_drive_activity (30d window)
    - id: 5-pull-cross-tenant-trace
      action: workplace_integration.cross_tenant_egress_trace_read
    - id: 6-pull-mail-context
      action: mail.read_keyword_search
    - id: 7-pull-workflow-logs
      action: workflow_engine.read_execution_logs
    - id: 8-interview
      action: interview_workbook
    - id: 9-determine-remediation-class
      action: determine_remediation_class
      input: [evidence_strength, counsel_attestation, interview_outcome]
    - id: 10-execute-remediation
      action: execute_remediation (light_touch OR heavy_touch)
    - id: 11-close
      action: investigation_orchestrator.close_case
```

## Cedar policy

```cedar
@id("workflow-engine-dlp-investigation-v1")
permit (
  principal,
  action == Action::"workflow_engine.dlp_investigation.open",
  resource is DLPEgressEvent
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  resource.tenant_id == principal.tenant_id
};
```

## Implementation notes

### Remediation-class determination

The `DetermineRemediationClass` algorithm:
```python
def determine_remediation_class(evidence, counsel_attestation, interview_outcome) -> str:
    if interview_outcome == 'malicious_intent':
        return 'HEAVY_TOUCH'
    if evidence.includes_home_ip_export and evidence.no_business_ticket:
        return 'HEAVY_TOUCH'
    if counsel_attestation == 'honest_mistake':
        return 'LIGHT_TOUCH'
    if evidence.coherent_business_narrative:
        return 'LIGHT_TOUCH'
    return 'HEAVY_TOUCH'  # default conservative
```

### Light-touch remediation

Atomic batch:
- Refresh DLP training.
- Update picker UI.
- Create pre-approved folder.
- Broadcast team channel.

### Heavy-touch remediation

Inherits from j138 investigation-orchestrator remediation.

## Performance budget

- DLP investigation case open ≤ 1s.
- Evidence-pull full set ≤ 60s.
- Light-touch remediation ≤ 5s.
- Heavy-touch remediation ≤ 10s.

## Test plan

Unit tests:
- `test_remediation_class_determination_correct`
- `test_light_touch_remediation_atomic`
- `test_heavy_touch_inherits_j138_safety`
- `test_dlp_investigation_state_machine`

## Build sequence

Standard. Inherits j138/j139.

## Acceptance gates

All tests PASS; Cedar lint clean; code review.

## Operational notes

Owner: axis-workflow-engine. Pager: `oya-workflow-engine-dlp-investigation`.

## Compliance / packs

Same as j138/j139.

## Cross-microservice port declaration

Per ADR-0145, `DLPInvestigationOrchestrator` in
`oyatie.workflow_engine.dlp_investigation.v1`.

## Roll-out plan

Five-phase.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Wrong remediation class | HIGH | Counsel attestation required + audit log |
| Light-touch ignores malicious | CRITICAL | Default-conservative rule |
| Evidence-pull misses key source | MED | Required-field validation |

## Definition of done

- Service live in production behind flag.
- Olusegun fixture goes through light-touch path correctly.
- Heavy-touch path verified separately with malicious fixture.
- Investigation conclusion sealed.

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.
