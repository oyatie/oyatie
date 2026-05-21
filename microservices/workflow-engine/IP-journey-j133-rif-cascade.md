---
doc_class: Implementation-Plan
ip_id: IP-journey-j133-rif-cascade
journey_ref: docs/user-journeys/j133-hr-conducts-layoff-with-dignity-and-compliance/
status: draft
date: 2026-05-20
microservice: workflow-engine
related_adrs: [ADR-0311, ADR-0244, ADR-0263, ADR-0246, ADR-0247, ADR-0292]
---

# IP — Workflow-Engine's role in j133 RIF cascade

## Scope

Workflow-Engine orchestrates the full RIF lifecycle: **plan → analyze → works-council → execute → disburse → outplace → cohort → revoke → retro**.
200 parallel per-employee cascades; per-jurisdiction durable timers (final-pay timing differs); strict
Cedar gate enforcing Marcus approval + DEI green + works-council clearance.

## Acceptance criteria

1. Workflow definitions registered: `rif-event-v3`, `rif-employee-cascade-v3`, `severance-disbursement-v3`, `outplacement-enroll-v2`, `cohort-channel-provision-v1`, `access-revocation-v3`, `reference-letter-generate-v2`, `rif-retro-v1`.
2. Activation Cedar gate requires Marcus approval + DEI green + per-jurisdiction works-council clearance.
3. 200 concurrent per-employee cascades sustained without degradation.
4. Per-jurisdiction durable timers (US-AUS T+0; KR-SEO T+14d; IN-BLR T+last+2d; DE-BER T+8wk).
5. Per-employee cascade emits 17±2 events.
6. Personal-tenant continuity assured on every cascade (PersonalTenantContinuityAssured event).
7. SLO: P95 phase-advance ≤ 600ms (excluding human 1:1 wait).

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Author rif-event-v3.yaml + rif-employee-cascade-v3.yaml | Schema test passes |
| 2 | Author severance-disbursement-v3.yaml with per-jurisdiction durable timers | T-202 passes |
| 3 | Author outplacement-enroll-v2.yaml + cohort-channel-provision-v1.yaml + access-revocation-v3.yaml | T-301, T-401, T-501 pass |
| 4 | Author reference-letter-generate-v2.yaml | T-601 passes |
| 5 | Author rif-retro-v1.yaml (T+90d auto-trigger) | T+90d test passes |
| 6 | Implement DEI-gate enforcement (calls intelligence) | T-002 passes |
| 7 | Implement works-council clearance gate (calls tenancy) | T-003, T-004 pass |
| 8 | Implement per-jurisdiction sequencing (staggered by timezone) | T-101 sub-step passes |
| 9 | Implement personal-tenant-continuity-assured emit per cascade | T-502 passes |
| 10 | Implement 28 audit-event-class emits | Registry green |
| 11 | Wire severance computer (Foundry scorer) call | T-201 passes |
| 12 | Wire emergency-pause flow (Marcus + Priya dual-approval) | Pause test passes |

## State machines

### rif-event-v3

```
[approved] → [planning] → [analysis] → [works-council-consult] → [execute-ready]
                                                                       │ b2b.hr.rif_execute
                                                                       ▼
                                                                  [executing]
                                                                       │ all cascades complete
                                                                       ▼
                                                                  [disbursing]
                                                                       │ all disbursed
                                                                       ▼
                                                                  [revocation_pending]
                                                                       │ T+last_working_day_max
                                                                       ▼
                                                                  [completed]
                                                                       │ T+90d
                                                                       ▼
                                                                  [retrospective_complete]
```

### rif-employee-cascade-v3

```
[scheduled] → [manager-1on1-started] → [manager-1on1-completed]
   │
   ▼
[mail-sent] → [severance-computed] → [severance-scheduled]
   │
   ▼
[outplacement-enrolled] → [cohort-channel-enrolled]
   │
   ▼
[revocation-scheduled] → [(durable timer)] → [session-revoked]
   │
   ▼
[work-drive-transferred] → [work-mail-archived] → [work-messenger-archived]
   │
   ▼
[personal-tenant-continuity-assured] → [completed]
```

## Cedar fragments authored

```cedar
// b2b.hr.rif_plan.cedar
permit (
  principal,
  action == Action::"b2b.hr.rif_plan",
  resource is RifEvent
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  resource.approved_by in [User::"marcus-ceo@marcus-tenant"] &&
  context.audit_session_open == true
};
```

```cedar
// b2b.hr.rif_execute.cedar
permit (
  principal,
  action == Action::"b2b.hr.rif_execute",
  resource is RifEvent
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  resource.approved_by in [User::"marcus-ceo@marcus-tenant"] &&
  resource.disparate_impact_analysis.verdict == "green" &&
  resource.per_jurisdiction_works_council_clearance.all_jurisdictions_cleared == true &&
  context.tenant.compliance_pack_active("pack-us-warn-act") &&
  context.tenant.compliance_pack_active("pack-eu-anti-discrimination-baseline") &&
  context.tenant.compliance_pack_active("pack-kr-labor-standards-act-amendment") &&
  context.tenant.compliance_pack_active("pack-in-industrial-disputes-act") &&
  context.audit_session_open == true
};
```

```cedar
// b2b.hr.rif_emergency_pause.cedar
permit (
  principal,
  action == Action::"b2b.hr.rif_emergency_pause",
  resource is RifEvent
) when {
  (principal.audience_type == "B2B_HR_ADMIN" && principal == User::"priya-krishnan@marcus-tenant.hr") ||
  (principal == User::"marcus-ceo@marcus-tenant")
};
```

## Audit-event classes registered

- RifPlanned, RifExecutionStarted, RifCascadeCompleted (event-level)
- 200 × (ManagerRif1on1Started, TerminationMailSent, SeveranceComputed, OutplacementEnrolled, CohortChannelEnrolled, WorkTenantSessionRevoked, PersonalTenantContinuityAssured, ...) = 17+ per employee
- ReferenceLetterGenerated (on request)
- PersonalTenantRevokeAttempted (alert-only)

## Dependencies

- **identity** (session revoke + delegation)
- **intelligence** (DEI analysis + severance scorer)
- **finops-portal** (severance compute)
- **payments** (severance disburse)
- **messenger** (manager 1:1 DM)
- **mail** (termination + ref-letter + per-jurisdiction templates)
- **community** (outplacement + cohort channel)
- **drive** (work-Drive transfer + archival)
- **tenancy** (works-council notify + sub-tenant scope)
- **compliance** (DEI gate + pack overlay + litigation hold + OWBPA window)
- **audit-chain** (EmitSealed per event)

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_rif_event_lifecycle_state` | gauge | tenant_id, state |
| `oya_rif_cascade_active_total` | gauge | tenant_id, event_id |
| `oya_rif_cascade_phase_advance_ms` | histogram | phase |
| `oya_rif_personal_tenant_continuity_assured_total` | counter | jurisdiction |
| `oya_rif_personal_tenant_revoke_attempted_total` | counter | (alarm-on-nonzero) |
| `oya_rif_durable_timer_accuracy_seconds` | histogram | jurisdiction |

## SLOs

- P50 phase-advance: 250ms; P95: 600ms; P99: 1.4s
- 200 concurrent cascades sustained
- Durable timer accuracy: within ±60s of scheduled
- Personal-tenant-continuity-assured emit: 100% (zero misses)

## Failure modes

| Failure | Recovery |
|---|---|
| DEI analysis yellow/red | Halt cascade; require Priya + Marcus re-balance |
| Works-council clearance missing (DE-BER) | Halt DE-BER cohort cascade; other jurisdictions proceed |
| Per-employee Cedar permit chain partial fail | Per-step retry; ops surfaces partial cascade |
| Severance scorer down | Halt computation; ops alert |
| Disbursement rail down | Defer per ADR-0028; EmployeeFinalPayDeferred |
| Cohort channel provisioning fail | Retry; if persistent, manual provision |
| Workflow-engine restart | Per ADR-0246 resume from checkpoint |

## Test gates

- T-001..T-005 (planning)
- T-101..T-106 (execution)
- T-201..T-205 (severance + disbursement)
- T-301..T-303 (outplacement)
- T-401..T-404 (cohort channel)
- T-501..T-506 (revocation + boundary)
- T-601, T-701..T-703 (ref letter + litigation hold)
- T-801..T-804 (failure modes)

## Notes

- Per ADR-0246 durable-execution, all 200 cascades survive pod restart.
- Per ADR-0247, severance scorer runs as Foundry principal `oyatie:foundry:scorer-severance-computer-v3`.
- Per ADR-0292, accessibility floor applies to all employee-facing surfaces during the cascade.

— end of IP —

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j133-rif-cascade.md` matched `SLO, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j133-rif-cascade.md` matched `finops`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
