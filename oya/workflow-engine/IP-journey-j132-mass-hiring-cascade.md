---
doc_class: Implementation-Plan
ip_id: IP-journey-j132-mass-hiring-cascade
journey_ref: docs/user-journeys/j132-hr-mass-hiring-event-100-roles/
status: draft
date: 2026-05-20
microservice: workflow-engine
related_adrs: [ADR-0311, ADR-0308, ADR-0244, ADR-0292, ADR-0246, ADR-0263]
---

# IP — Workflow-Engine's role in j132 mass-hiring cascade

## Scope

Workflow-Engine orchestrates the 7-phase mass-hiring cascade end-to-end:
**activation → posting → application-triage → AI-screening → interview → offer → post-hire-audit**.
Per ADR-0246 durable-execution, all phases survive crash + restart. Per ADR-0263, all 26 audit-event classes are emitted from this µservice or delegated to peer µservices but coordinated here.

## Acceptance criteria

1. Workflow definitions registered:
   - `hiring-event-v2` (event-level state machine)
   - `requisition-activation-v2` (per-req state machine)
   - `application-triage-v3` (per-applicant state machine)
   - `interview-execute-v1` (per-interview round)
   - `offer-extension-v2` (per-offer)
   - `new-hire-provision-v2` (per-new-hire)
   - `post-hire-fairness-audit-v1` (T+90d trigger)
2. Each phase advances on Cedar PERMIT; reject terminates with structured outcome code.
3. EU-AI-Act preflight gate is enforced before AI-screening phase advances.
4. Fairness gate is enforced before per-applicant decision is accepted.
5. State conforms to `schemas/hiring-event.json` + `schemas/job-application.json`.
6. Crash + restart resumes any in-flight workflow without audit-event loss (ADR-0028).
7. SLO: P95 phase-advance latency (excluding human wait) ≤ 600ms.
8. SLO: 5,000 concurrent application-triage workflows sustained per cell.
9. T+90d post-hire audit is auto-scheduled with reliable wakeup.

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Author workflow def `hiring-event-v2.yaml` (top-level FSM) | Workflow-def schema test passes |
| 2 | Author workflow def `requisition-activation-v2.yaml` | Schema test + integration T-001 passes |
| 3 | Author workflow def `application-triage-v3.yaml` | Schema test + T-201 passes |
| 4 | Author workflow def `interview-execute-v1.yaml` | Schema test + T-401 passes |
| 5 | Author workflow def `offer-extension-v2.yaml` | Schema test + T-501 passes |
| 6 | Author workflow def `new-hire-provision-v2.yaml` | Schema test + T-504 passes |
| 7 | Author workflow def `post-hire-fairness-audit-v1.yaml` | Schema test + T-601 passes |
| 8 | Implement EU-AI-Act preflight gate (calls compliance) | Integration T-302 passes |
| 9 | Implement fairness gate enforcement (reads intelligence.fairness_audit verdict) | T-303, T-304 pass |
| 10 | Wire ResolveRoleHolder dispatch to identity for per-jurisdiction delegate resolution | T-103, T-103-delegate passes |
| 11 | Add Cedar fragments: hiring-event-activate, requisition-activate, ai-screen-phase-activate, offer-extend, fairness-yellow-requires-manual-final-review | Cedar evaluator tests pass |
| 12 | Add audit-event classes (26 classes) to ADR-0263 registry | Registry lane green |
| 13 | Add durable-resume test (kill engine mid-1040-screening) | T-903 passes |
| 14 | Add T+90d-auto-schedule durable timer | T-601 passes |
| 15 | Add per-jurisdiction overlay hash-pinning at activation time | T-002 passes |

## State machines (key)

### `hiring-event-v2`

```
[awaiting_hr_activation]
   │ b2b.hr.event_activate
   ▼
[active]
   │ all_reqs_finalized
   ▼
[in_post_hire_window]
   │ T+90d_elapsed
   ▼
[post_hire_audit_complete]
   │ archive
   ▼
[archived]
```

### `application-triage-v3`

```
[received]
   │ ai_screen_request
   ▼
[ai_screened]
   │ human_review
   ▼
[human_reviewed]
   ├─→ [invited_to_interview]
   │        │
   │        ▼
   │      [in_interview]
   │        │
   │        ▼
   │      [scorecards_complete]
   │        │
   │        ▼
   │      [offer_decision]
   │        ├─→ [offer_extended]
   │        │      │
   │        │      ▼
   │        │    [signed] OR [declined] OR [expired]
   │        │      │
   │        │      ▼
   │        │   [provisioning]
   │        │      │
   │        │      ▼
   │        │   [hired]
   │        └─→ [rejected]
   └─→ [rejected]
```

### `offer-extension-v2`

State machine includes:
- Per-jurisdiction template selection (compliance overlay-driven)
- E-sign cycle with 7-day expiry
- Decline → keeps req open for next candidate
- Expire → auto-reminds candidate, escalates after 5d

## Dependencies

- **identity**: ResolveSubject + ResolveRoleHolder + ApplicantPseudonymized + ProvisionNewPrincipal + SCIM bulk push (Identity IP separate, see `IP-journey-j132-applicant-pseudonymization-and-provisioning.md`).
- **community**: PublishHandshakeBatch + PublishLinkedInBatch + RecordApplication.
- **intelligence**: ScreenApplicantBatch + RunFairnessAudit + RunPostHireFairnessAudit + per-applicant explanation persistence.
- **mail**: ComposeAndSend (multiple templates).
- **calendar**: BookCrossTenantSlot + RescheduleSlot.
- **meet**: CreateInterviewRoom (with closed-captions per-jurisdiction).
- **workplace-integration**: GenerateOfferLetter + EsignTracker + OnboardingCascade.
- **tenancy**: ResolveTenantScope + WorksCouncilNotify + jurisdiction overlay propagation.
- **compliance**: CheckEUAIActPreflight + ResolvePerReqOverlay + FileArticle86Record + PublishNYAEDTReport.
- **audit-chain**: EmitSealed (per phase + per audit-event class).
- **finops-portal** (read-only): salary-ceiling lookup for offer-extension Cedar gate.

## Cedar fragments authored

```cedar
// hiring-event-activate.cedar
permit (
  principal == User,
  action == Action::"b2b.hr.event_activate",
  resource is HiringEvent
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  resource.requested_by in principal.delegated_authority_chain &&
  context.tenant.compliance_pack_active("pack-eu-ai-act-2026-baseline") &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true
};
```

```cedar
// ai-screen-phase-activate.cedar
permit (
  principal == User,
  action == Action::"b2b.intelligence.applicant_screening.activate",
  resource is HiringEvent
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  context.tenant.compliance_pack_active("pack-eu-ai-act-2026-baseline") &&
  context.tenant.eu_ai_act_conformity_certificate_valid == true &&
  resource.fairness_gate_active == true &&
  context.applicant_count <= 5000 &&
  context.audit_session_open == true
};
```

```cedar
// fairness-yellow-requires-manual-final-review.cedar
forbid (
  principal == User,
  action == Action::"b2b.hr.offer_decision_finalize",
  resource is OfferDecision
) when {
  resource.fairness_band == "yellow" &&
  !resource.has_manual_secondary_review &&
  context.tenant.compliance_pack_active("pack-eu-ai-act-2026-baseline")
};
```

```cedar
// offer-extend-budget-gate.cedar
permit (
  principal == User,
  action == Action::"b2b.hr.offer_extend",
  resource is Offer
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  resource.req_id.has_hiring_committee_decision_finalized == true &&
  resource.salary <= principal.tenant.budget.salary_ceiling_for_role(resource.req_id) &&
  context.tenant.compliance_pack_active("pack-eu-ai-act-2026-baseline") &&
  context.audit_session_open == true
};
```

## Audit-event classes registered (per ADR-0263)

Workflow-engine emits or coordinates the emission of:
- HiringEventActivated
- RequisitionActivated (×100)
- AiScreeningPhaseStarted
- AiScreeningPhaseCompleted
- InterviewInviteStarted
- InterviewInviteCompleted
- OfferExtensionStarted
- OfferExtensionFinalized
- NewHireProvisioningStarted
- NewHireProvisioningCompleted
- PostHireAuditPhaseScheduled
- PostHireAuditPhaseCompleted

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_hiring_event_activation_latency_ms` | histogram | tenant_id, jurisdiction |
| `oya_application_triage_active_total` | gauge | tenant_id, event_id |
| `oya_workflow_phase_advance_latency_ms` | histogram | phase, outcome |
| `oya_workflow_fairness_gate_yellow_total` | counter | tenant_id, event_id |
| `oya_workflow_fairness_gate_red_total` | counter | tenant_id, event_id |
| `oya_workflow_durable_resume_count` | counter | reason |
| `oya_workflow_post_hire_audit_scheduled_total` | counter | tenant_id |

## SLOs

- P50 phase-advance: 280ms
- P95 phase-advance: 600ms
- P99 phase-advance: 1.4s
- Durable-resume success rate: > 99.99%
- 5k concurrent application-triage workflows per cell, P99 advance ≤ 1.4s
- T+90d auto-trigger reliability: > 99.99% within ±60s of T+90d

## Failure modes

| Failure | Recovery |
|---|---|
| Compliance preflight FAIL | Phase aborts; Priya notified; workflow stays in `awaiting_preflight_recheck` |
| Intelligence scorer unavailable | Wait 60s; fall back to v1 with Priya consent; if v1 also down, halt phase |
| Audit-chain degraded | Local WAL per ADR-0028; flush when audit recovers |
| Workflow-engine pod crash mid-screening | Per ADR-0246 durable execution, resume from checkpoint |
| Per-req overlay missing | Per-req fail; other reqs proceed; banner to Priya |

## Per-jurisdiction overlay coverage

| Jurisdiction | Overlay version pinned at activation | Audit-event class added |
|---|---|---|
| IN-BLR | pack-in-industrial-disputes-act:v3 | OverlayResolved-IN-BLR |
| US-AUS | pack-us-title-vii-baseline:v2 + pack-us-adea:v2 + pack-us-ny-aedt-local-law-144:v1 | OverlayResolved-US-AUS |
| DE-BER | pack-eu-anti-discrimination-baseline:v1 + pack-eu-pay-transparency-2023-970:v1 + pack-eu-ai-act-2026-baseline:v1 | OverlayResolved-DE-BER |
| KR-SEO | pack-kr-equal-employment-opportunity-act:v2 + pack-kr-labor-standards-act-2026-amendment:v1 | OverlayResolved-KR-SEO |

## Migration / rollout

- Lane: workflow-engine-rollout-j132 on dev → staging → production cells
- Pre-roll: backfill `hiring-event-v2` def into spec-store; smoke-run T-001
- Roll: enable `j132_workflow_set` feature flag (per ADR-0292) for marcus-tenant only
- Validate: 1 week with marcus-tenant; bias-flag rate ≤ 8%
- Promote: enable for all B2B tenants with `pack-eu-ai-act-2026-baseline` active

## Test gates

- T-001 through T-904 (per `integration-test-plan.md`)
- All Cedar fragments compile in policy-engine
- All audit-event classes registered in ADR-0263 audit-class-registry.yaml

## Notes

- Per ADR-0247 self-modification, the scorer principal is `oyatie:foundry:scorer-applicant-screening-v2`; workflow-engine treats it as a Cedar-permitted internal principal.
- Per ADR-0311, the workflow-engine does NOT cross the dual-tenant boundary; cross-tenant calendar invites use a Cedar-permitted cross-tenant-invite primitive that the candidate's personal tenant can accept or refuse.
- This IP is intern-buildable: clear acceptance criteria, atomic deliverables, canonical fixture files, and reproducible local-cell test environment.

— end of IP —

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j132-mass-hiring-cascade.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j132-mass-hiring-cascade.md` matched `emission, finops`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
