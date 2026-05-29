---
doc_class: Implementation-Plan
ip_id: IP-journey-j136-open-enrollment-orchestration
journey_ref: docs/user-journeys/j136-hr-administers-benefits-open-enrollment/
status: draft
date: 2026-05-20
microservice: workflow-engine
related_adrs: [ADR-0311, ADR-0244, ADR-0263, ADR-0246, ADR-0249]
---

# IP — Workflow-Engine's role in j136 open enrollment orchestration

## Scope

Workflow-Engine orchestrates the 60-day annual benefits open-enrollment cycle for 5,000 employees:
plan-design → engagement → publish → announce → elections (5000 parallel) → late-reminder →
passive-defaults → provider-bulk-push → reconciliation → payroll-setup → confirmations →
ongoing-life-events (year-round) → year-end-aca.

Per ADR-0246 durable execution, all workflows survive pod restarts. Per ADR-0246, durable timers
trigger payroll deductions per pay-period from January 2027 onward.

## Acceptance criteria

1. Workflow definitions registered: `open-enrollment-cycle-v3`, `benefits-enrollment-v3`, `benefits-life-event-change-v1`, `aca-form-1095c-generate-v2`, `benefits-reconciliation-v1`, `payroll-deduction-execute-v3`.
2. Per-employee cycle: form-fill, submit, validate, archive, sync, set-payroll.
3. Per-jurisdiction timing: 38-day window for US-AUS; 38-day for others.
4. Year-end ACA Form 1095-C auto-trigger T+365d.
5. Per-pay-period durable-timer execution for payroll deductions.
6. SLO: P95 phase-advance ≤ 600ms; sustained 5000 concurrent workflows.

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Author open-enrollment-cycle-v3.yaml | Schema test |
| 2 | Author benefits-enrollment-v3.yaml (per-employee) | T-201..T-204 pass |
| 3 | Author benefits-life-event-change-v1.yaml | T-801..T-803 pass |
| 4 | Author aca-form-1095c-generate-v2.yaml (T+365d trigger) | T-701 passes |
| 5 | Author benefits-reconciliation-v1.yaml | T-501, T-502 pass |
| 6 | Author payroll-deduction-execute-v3.yaml (per pay-period) | T-602 passes |
| 7 | Implement passive-default logic at T+38d | T-302 passes |
| 8 | Implement late-reminder cascade | T-301 passes |
| 9 | Implement reconciliation discrepancy resolution | T-502 passes |
| 10 | Wire audit-chain | Registry green |

## State machine (benefits-enrollment-v3)

```
[pre-form] → [form-rendered] → [draft] → [submitted]
                                              │
                                              ▼
                                       [validated]
                                              │
                                              ▼
                                       [dependents-archived]
                                              │
                                              ▼
                                       [confirmed-mail-sent]
                                              │
                                              ▼
                                       [provider-bulk-included]
                                              │
                                              ▼
                                       [provider-acked]
                                              │
                                              ▼
                                       [payroll-deduction-set]
                                              │
                                              ▼
                                       [active]
```

## Cedar permits

```cedar
permit (
  principal,
  action == Action::"b2b.hr.open_enrollment_open",
  resource is OpenEnrollmentCycle
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  context.tenant.compliance_pack_active("pack-us-erisa-baseline") &&
  context.tenant.compliance_pack_active("pack-us-hipaa-baseline") &&
  context.tenant.compliance_pack_active("pack-eu-iorp-ii-baseline") &&
  context.tenant.compliance_pack_active("pack-kr-national-pension-baseline") &&
  context.tenant.compliance_pack_active("pack-in-epf-baseline") &&
  context.audit_session_open == true
};
```

## Dependencies

All 8 µservices (workflow-engine + forms + drive + connect + payments + mail + identity + tenancy).

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_open_enrollment_cycle_lifecycle_state` | gauge | tenant_id, state |
| `oya_open_enrollment_active_workflows_total` | gauge | jurisdiction |
| `oya_open_enrollment_late_filer_count` | gauge | jurisdiction |
| `oya_open_enrollment_passive_default_total` | counter | jurisdiction |
| `oya_open_enrollment_aca_form_generated_total` | counter | n/a |
| `oya_open_enrollment_reconciliation_discrepancy_total` | counter | provider |

## SLOs

- P50 phase: 240ms; P95: 600ms
- 5000 concurrent workflows sustained
- Durable timer accuracy: ±60s for pay-periods + T+365d

## Test gates

- All Test Set 3 + 4 + 5 + 6 + 7 + 8 + 9 tests

## Notes

- Per ADR-0246, durable execution + durable timers.
- Per ADR-0247, severance scorer + decision-support scorer are Foundry principals.
- Per ADR-0263, every phase-advance sealed.

— end of IP —

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j136-open-enrollment-orchestration.md` matched `SLO, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
