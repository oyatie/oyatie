---
doc_class: Implementation-Plan
ip_id: IP-journey-j135-investigation-orchestration
journey_ref: docs/user-journeys/j135-hr-handles-harassment-complaint-with-dual-tenant-boundary/
status: draft
date: 2026-05-20
microservice: workflow-engine
related_adrs: [ADR-0311, ADR-0244, ADR-0263, ADR-0246]
---

# IP — Workflow-Engine's role in j135 investigation orchestration

## Scope

Workflow-Engine orchestrates the harassment-complaint-investigation-v2 workflow end-to-end:
**complaint → routing → unpseudonymize → investigation-open → work-data-read → 3rd-party-engage →
interviews → findings → outcome → remedy → close**.
Per ADR-0246 durable execution, the workflow survives any pod restart.

## Acceptance criteria

1. Workflow definitions registered: `harassment-complaint-investigation-v2`, `investigation-interview-schedule-v1`, `investigation-remedy-enact-v1`.
2. Per-investigation durable state.
3. Per-jurisdiction overlay enforcement.
4. Investigation timeline tracking (alarm if approaching jurisdiction-max-days).
5. SLO: P95 phase-advance ≤ 600ms.

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Author harassment-complaint-investigation-v2.yaml | Schema test passes |
| 2 | Author investigation-interview-schedule-v1.yaml | T-501 passes |
| 3 | Author investigation-remedy-enact-v1.yaml | T-502 passes |
| 4 | Implement per-jurisdiction overlay enforcement | T-601..T-603 pass |
| 5 | Implement timeline tracker + alarm | timeline alarm test passes |
| 6 | Wire audit-chain: InvestigationOpened + InvestigationInterviewScheduled + InvestigationInterviewCompleted + InvestigationReportFinalized + InvestigationOutcomeFinalized + InvestigationClosed | Registry green |

## State machine (harassment-complaint-investigation-v2)

```
[opened]
   │
   ▼
[work_data_review]
   │
   ▼
[third_party_engaged]
   │
   ▼
[interview_phase]
   │
   ▼
[report_drafted]
   │
   ▼
[outcome_pending]
   │
   ▼
[remedy_implementing]
   │
   ▼
[closed]
```

## Cedar permits

```cedar
permit (
  principal,
  action == Action::"b2b.wf.investigation_orchestrate",
  resource is Investigation
) when {
  principal.audience_type in ["B2B_HR_ADMIN", "B2B_LEGAL_ADMIN"] &&
  context.audit_session_open == true
};
```

## Dependencies

- **community** (whistleblower)
- **messenger** (work-DM read)
- **identity** (pseudonymize + cross-tenant resolve)
- **tenancy** (investigation engagement)
- **compliance** (per-jurisdiction overlay + timeline)
- **mail** (interview invites)
- **calendar** (interview booking)
- **meet** (interview rooms)
- **workplace-integration** (remedy enactment)
- **audit-chain** (EmitSealed)

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_wf_investigation_active_total` | gauge | jurisdiction |
| `oya_wf_investigation_phase_advance_ms` | histogram | phase |
| `oya_wf_investigation_timeline_remaining_days` | gauge | investigation_id |
| `oya_wf_investigation_close_total` | counter | outcome |

## SLOs

- P50 phase: 240ms; P95: 600ms; P99: 1.4s
- Timeline tracker: 100% alarm-before-violation

## Failure modes

| Failure | Recovery |
|---|---|
| Investigation timeline approaching limit | Auto-escalate to Marcus + Priya |
| Pod restart mid-investigation | Per ADR-0246 resume from checkpoint |
| 3rd-party report delivery delayed | Auto-extend deadline; alert Priya |

## Test gates

- All Test Set 5 + 6 + 7 tests
- T-001..T-503 (investigation orchestration end-to-end)

## Notes

- Per ADR-0246, investigation state is durable.
- Per ADR-0263, every phase-advance sealed.
- Per ADR-0244, B2B_HR_ADMIN + B2B_LEGAL_ADMIN audience-types orchestrate.

— end of IP —

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j135-investigation-orchestration.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
