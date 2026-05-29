---
doc_class: Implementation-Plan
ip_id: IP-journey-j134-engagement-workflow-and-replacement-guarantee
journey_ref: docs/user-journeys/j134-hr-cross-tenant-recruitment-via-staffing-agency/
status: draft
date: 2026-05-20
microservice: workflow-engine
related_adrs: [ADR-0311, ADR-0244, ADR-0249, ADR-0246]
---

# IP — Workflow-Engine's role in j134 engagement workflow + 90-day replacement guarantee

## Scope

Workflow-Engine orchestrates the staffing-agency engagement: post → shortlist → interview →
offer → escrow → start → 90-day check → guarantee/no-guarantee.
Per ADR-0246 durable execution, the 90-day check fires reliably regardless of engine restarts.

## Acceptance criteria

1. Workflow definitions registered: `staffing-engagement-v1`, `cross-tenant-candidate-triage-v1`, `placement-fee-90d-check-v1`, `replacement-guarantee-invoke-v1`.
2. Per-engagement durable state.
3. 90-day check trigger reliability > 99.99%.
4. Replacement-guarantee invokes reverse Stripe refund.
5. SLO: P95 phase-advance ≤ 600ms.

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Author staffing-engagement-v1.yaml | Schema test passes |
| 2 | Author cross-tenant-candidate-triage-v1.yaml | T-101 passes |
| 3 | Author placement-fee-90d-check-v1.yaml | T-401 passes |
| 4 | Author replacement-guarantee-invoke-v1.yaml | T-402 passes |
| 5 | Implement durable T+90d timer per placement | T-401 trigger reliability |
| 6 | Implement audience-type transition trigger | T-501 passes |
| 7 | Wire audit-chain | Registry green |

## State machine (staffing-engagement-v1)

```
[draft] → [signed] → [posted_to_agency] → [shortlist_arriving]
                                                │
                                                ▼
                                       [interviewing] → [offer_extended] → [signed_offer]
                                                                              │
                                                                              ▼
                                                                       [pre_start_escrow]
                                                                              │
                                                                              ▼
                                                                       [started] → [90d_window]
                                                                                      │
                                                                                      ▼
                                                                       [guarantee_passed] OR [guarantee_invoked]
                                                                                      │
                                                                                      ▼
                                                                              [engagement_closed]
```

## Cedar permits

```cedar
permit (
  principal,
  action == Action::"b2b.wf.staffing_engagement_start",
  resource is StaffingEngagement
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  resource.agency_tenant in principal.tenant.connect_trust_partners &&
  context.audit_session_open == true
};
```

```cedar
permit (
  principal == User::"oyatie:workflow-engine:internal:placement-90d-check",
  action == Action::"b2b.wf.placement_90d_check",
  resource is Placement
);
```

## Dependencies

- **community** (cross-tenant posting)
- **identity** (audience-type transition + cross-tenant principal)
- **payments** (Stripe escrow + refund)
- **workplace-integration** (offer letter)
- **tenancy** (engagement scope)
- **audit-chain** (EmitSealed)

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_wf_staffing_engagement_active_total` | gauge | n/a |
| `oya_wf_placement_90d_check_trigger_total` | counter | outcome |
| `oya_wf_replacement_guarantee_invoke_total` | counter | reason |
| `oya_wf_durable_timer_accuracy_seconds` | histogram | timer_type |

## SLOs

- P50 phase: 240ms; P95: 600ms
- 90d-timer accuracy: ±60s

## Test gates

- T-001..T-003 (engagement init)
- T-101..T-104 (cross-tenant interview)
- T-301..T-303 (escrow)
- T-401..T-403 (90d + replacement guarantee)
- T-501..T-502 (audience-type transition)

## Notes

- Per ADR-0246, the 90-day check is durable; restart-safe.
- Per ADR-0247, audience-type transition uses Foundry scorer.

— end of IP —

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j134-engagement-workflow-and-replacement-guarantee.md` matched `SLO, escrow, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
