---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j142
microservice: workflow-engine
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0145, ADR-0244, ADR-0247, ADR-0299, ADR-0311]
---

# workflow-engine — IP slice for j142 (offboarding state machine)

## Scope

Deliver the 47-step `rif_offboarding_us_michigan_v3` template + companion templates for the 5 other jurisdictions, plus the cross-tenant emission primitives the offboarding needs.

## API surface (gRPC)

```proto
service Workflow {
  rpc Start(StartRequest) returns (StartResponse);
  rpc Pause(PauseRequest) returns (PauseResponse);
  rpc Resume(ResumeRequest) returns (ResumeResponse);
  rpc Close(CloseRequest) returns (CloseResponse);
  rpc Status(StatusRequest) returns (StatusResponse);
}

service Checkpoint {
  rpc Schedule(ScheduleRequest) returns (ScheduleResponse);
  rpc Fire(FireRequest) returns (FireResponse);
}
```

## Template authoring

Authored as YAML in `templates/`. Skeleton for `rif_offboarding_us_michigan_v3`:

```yaml
template_id: rif_offboarding_us_michigan_v3
version: 3.0.0
overlay_class: us_mi_layoff
required_inputs:
  - subject_principal
  - actor_principal
  - related_workflow
steps:
  - id: revoke_active_session_scopes
    µservice: identity
    rpc: Sessions.RevokeScopes
    payload_from_input: ...
    on_failure: retry_with_backoff
  - id: demote_work_mail
    µservice: mail
    rpc: Mailbox.Demote
    ...
  - id: demote_work_messenger
    ...
  - id: classify_and_demote_work_drive
    ...
  - id: cancel_future_calendar_events
    ...
  - id: emit_separation_packet_to_personal_mail
    µservice: mail
    rpc: OutboundMail.Send
    is_cross_tenant: true
    cross_tenant_purpose: mandated_layoff_communication
  - id: open_severance_payable
    µservice: payments
    rpc: Payable.OpenCrossTenant
    is_cross_tenant: true
    cross_tenant_purpose: severance_settlement
  - id: enroll_cobra_eligibility
    ...
  - id: emit_erisa_1132_notice
    ...
  - id: schedule_access_revocation_checkpoints
    µservice: workflow-engine
    rpc: Checkpoint.Schedule
    checkpoints:
      - at: +7d
        action: remind_chris_export_deadline
      - at: +14d
        action: remind_chris_export_deadline
      - at: +21d
        action: remind_chris_export_deadline
      - at: +30d
        action: revoke_all_remaining
  - id: request_audience_type_delegation
    µservice: identity
    rpc: AudienceType.RequestDelegation
    is_cross_tenant: true
  - id: emit_hrrp_signal
    µservice: detection-substrate (cross-tenant publish to personal-tenant)
    rpc: Signal.Publish
    is_cross_tenant: true
  - ... (35 more steps; hris-sync, payroll-cutoff, equipment-return, exit-interview-opt-in, references-policy, alumni-channel-opt-in, etc.)
```

## State machine semantics

- State `in_progress`: steps execute in declared order; failures retry up to per-step max.
- State `paused_on_audit_hold`: triggered by Sam-side audit hold; pauses revocation steps but allows already-completed emissions to stay.
- State `paused_on_leave`: protected-leave gate (FMLA, German Mutterschutz, etc.); legal pre-checks required to proceed.
- State `failed_recoverable`: step-level retry exhausted but workflow can be resumed manually.
- State `failed_terminal`: e.g., subject_principal does not exist; cannot proceed.
- State `completed_clean`: all 47 steps succeeded; T+30d revocation executed.

## Cross-tenant step semantics

For any step with `is_cross_tenant: true`:
- Workflow-engine fetches the source-tenant Cedar permit.
- Calls the target µservice's `*.CrossTenant` variant; the µservice handles dest-side Cedar + double-seal.
- Workflow stamps `cross_tenant_step_emitted=true` with both tenant_ids in audit.

## Cedar permits

| Permit | Granted to | Purpose |
|---|---|---|
| `b2b.workflow.offboarding.start` | HR-admin | Open the workflow |
| `b2b.workflow.offboarding.pause` | HR-admin + internal-audit | Pause for audit hold |
| `b2b.workflow.offboarding.resume` | HR-admin + internal-audit | Resume |
| `b2b.workflow.checkpoint.schedule` | workflow-engine (self) | Schedule the T+30d revocation |

## Audit emissions

- `WorkflowStarted{wf_id, actor, subject, overlay_hash, template_version}`
- For each step: `StepStarted`, `StepCompleted` or `StepFailed{reason, retry_count}`
- `CheckpointScheduled{at, action}`
- `CheckpointFired{at, action, result}`
- `WorkflowPaused{reason}`, `WorkflowResumed`
- `WorkflowCompleted{status}`

## Performance

- Workflow-start to phase-1-done (steps 1-6) p99 ≤ 30s.
- Cross-tenant steps p99 ≤ 60s end-to-end.
- Checkpoint scheduling p99 ≤ 100ms.

## Acceptance criteria

- [ ] Template `rif_offboarding_us_michigan_v3` authored and SHA-256 versioned.
- [ ] Companion templates for US-CA, US-NY, EU-DE, KR, IN authored.
- [ ] All 47 steps emit audit-trace under single `audit_trace_id`.
- [ ] B.10 audit-hold pause-resume works.
- [ ] B.9 KR variant runs with overlay swap transparent.
- [ ] Checkpoints fire at T+7d, T+14d, T+21d, T+30d via scheduler.

## Out of scope

- The individual µservices' demote/revoke endpoints (those are in their own IPs).
- The Cedar policy authoring.
- The works-council pre-consultation flow (EU-DE template references it; full flow is in a separate j-series journey not in this slice).

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j142-offboarding-state-machine.md` matched `p99, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j142-offboarding-state-machine.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
