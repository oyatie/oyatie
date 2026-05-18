---
doc_class: Runbook
title: Tenant deletion + DSR cascade (proof-of-erasure)
microservice: tenancy
severity: "Sev-2 (DSR SLA at risk) / Sev-1 (regulator escalation)"
status: Accepted
owner_team: council-privacy + axis-tenancy
date: 2026-05-17
related_artifacts:
  - microservices/tenancy/failure-modes.md (FM-06 DSR cascade incomplete)
  - microservices/tenancy/dpia.md (R-05, R-12)
  - microservices/tenancy/policy/data-residency.md (DSR Cascade)
  - microservices/tenancy/incident-response.md
  - microservices/tenancy/PRD.md (FR-05, FR-09)
doc_status: published
---

# Runbook: Tenant deletion + DSR cascade

## Purpose

Tenant deletion triggers a **compliant cross-µservice erasure cascade** producing a cryptographic proof-of-erasure certificate. This is the load-bearing GDPR Art. 17 / KR PIPA Art. 36 / DPDPA §12 / LGPD Art. 18 fulfilment path.

## Trigger

ONE of:
1. **Tenant-operator self-serve deletion**: tenant operator submits `POST /tenants/<id>` `DELETE` action (with DPO confirmation flow).
2. **Operator-initiated deletion**: platform-operator + ops-security 2-person rule (e.g., contract termination, non-payment).
3. **DSR submission**: end-user-of-tenant submits DSR via the tenant operator's joint-controllership channel; tenant operator forwards to oyatie DPO.
4. **Regulator-compelled deletion**: legal process; council-privacy chair authorises.

## Severity

- Routine DSR within SLA: Sev-3 (or not an incident; tracked as ordinary lifecycle).
- DSR SLA at risk (80% of per-pack legal window consumed): Sev-2.
- Missing receipt / handler gap discovered: Sev-2 escalating to Sev-1 if SLA breach imminent.
- Regulator escalation: Sev-1.

## Steps (normal cascade)

| Step | Action | Time budget |
|---|---|---|
| 1 | DSR submission to `dsr-cascade-rest` (`POST /dsr-requests`); 2-person rule for non-DSR-driven deletes | – |
| 2 | dsr-cascade-rest: Cedar policy verifies submitter; DPO sign-off recorded | ≤ 1 min |
| 3 | dsr-cascade-usecase: creates DsrRequest entity; emits `TenantDeletionRequested` Workflow event | ≤ 5 s |
| 4 | Every µservice with tenant-scoped data receives the event; executes its DSR handler | hours-days |
| 5 | Each µservice emits `ErasureReceipt{microservice, tenant_id, data_classes_erased, residual_data_basis_if_any, signed_at}` | per-µservice |
| 6 | dsr-cascade-worker aggregates receipts; tracks `received_n` vs `expected_n` (from MicroserviceRegistered registry) | continuous |
| 7 | When `received_n == expected_n` (or DPO-signed-off equivalence): aggregate Merkle root; emit `TenantDeletionCompleted` | ≤ 1 min after final receipt |
| 8 | dsr-cascade-adapter (ProofOfErasureSigner): seals Merkle root via audit-chain Ed25519; emits ProofOfErasure certificate | ≤ 5 s |
| 9 | tenant-lifecycle-usecase: terminal state `Activated|Suspended → Deleted` (soft-delete grace window 30d for accidental-recovery before hard-delete) | – |
| 10 | Tenant notified per pack legal SLA (30d GDPR / 30d KR PIPA / 15d LGPD / 30d DPDPA); regulator notified if applicable | per SLA |

## Pre-checks (recovery)

1. Identify the DSR: `cargo run -p oya-dev-cli -- tenancy dsr status --dsr-id <id>` → returns request, received receipts, missing µservices, SLA timer.
2. Verify expected µservice count: read `MicroserviceRegistered` registry.
3. Check for per-µservice handler exceptions in receipt records.

## Recovery Path A — Missing receipt (single µservice timeout)

Cause: a µservice's DSR handler crashed / is slow / is missing.

| Step | Action |
|---|---|
| 1 | Identify the missing µservice from the receipt registry. |
| 2 | Engage that µservice's on-call: verify handler health. |
| 3 | If handler bug: emergency hotfix; re-trigger event consumption (idempotent). |
| 4 | If handler missing (rare; LEAN check `oya-governance-dsr-handler-conformance` prevents new µservices without handlers): file Sev-2 incident; DPO + ops-security 2-person manual-override; document alternative-measure (e.g., µservice deployed post-tenant-creation has no tenant data, so receipt is vacuous). |
| 5 | Re-aggregate receipts; proceed if complete. |

## Recovery Path B — SLA timer at 80% of window

Cause: legitimate tenant DSR taking longer than expected.

| Step | Action |
|---|---|
| 1 | Escalate to council-privacy: review which µservices have not emitted. |
| 2 | If receipts pending due to operational delay (not legal-hold): expedite via on-call engagement. |
| 3 | If legal-hold prevents erasure: document the legal basis (LEGAL_HOLD residual_data); proceed with proof-of-erasure marking the residual; tenant + regulator notified of legal-hold basis. |

## Recovery Path C — Soft-delete recovery (within 30d grace)

Cause: erroneous deletion (e.g., wrong tenant ID); soft-delete grace not yet elapsed.

| Step | Action |
|---|---|
| 1 | Operator + DPO + ops-security agree to undo within 30d grace. |
| 2 | tenant-lifecycle-usecase: terminal state `Deleted-Soft → Activated`; revert state machine. |
| 3 | DSR cascade NOT reversed (some µservices may have already hard-deleted; tenant accepts partial data loss); operator informed. |
| 4 | Post-incident: tighten 2-person rule + add UI confirmation. |

## Recovery Path D — Hard-delete (post-grace)

After 30d soft-delete grace:

| Step | Action |
|---|---|
| 1 | tenant-lifecycle-worker schedules hard-delete; emits `TenantDeletionFinalised` event. |
| 2 | Per-µservice handlers that supported soft-delete grace finalise their erasure. |
| 3 | Proof-of-erasure updated to mark `finalised_at`. |
| 4 | Tenant metadata retained for 7y after finalisation (per `policy/data-residency.md` retention table) for DSR audit horizon. |

## Recovery Path E — Regulator audit of proof-of-erasure

Cause: regulator (e.g., EU DPA, KR PIPC) requests verification of a tenant's deletion.

| Step | Action |
|---|---|
| 1 | Auditor / regulator JIT token issued (Cedar `regulator` scope); engagement window bound. |
| 2 | Regulator queries `GET /proof-of-erasure/<dsr_id>`; returns Merkle root + receipt envelopes. |
| 3 | Regulator independently verifies Merkle proof against per-µservice signed receipts. |
| 4 | Audit-chain seal trail confirms emission timestamps. |

## Verification

After cascade completion:
- DsrRequest status = `Completed`; ProofOfErasure record emitted with Merkle root signed.
- All expected µservices' ErasureReceipts received + signed.
- Tenant operator notified within per-pack SLA.
- Regulator notification (if applicable) within per-pack timeline.
- Tenant metadata in 30d soft-delete grace; hard-delete scheduled.

## Post-incident updates

- Postmortem within 5 business days if Sev-2+.
- If receipt gap: tighten `oya-governance-dsr-handler-conformance` lane; require µservice handler registration in catalog.
- If SLA breach: regulator notification + breach-report; review per-pack SLA timer thresholds.

## References

- `microservices/tenancy/PRD.md` FR-05 + FR-09.
- `microservices/tenancy/failure-modes.md` FM-06.
- `microservices/tenancy/dpia.md` R-05 + R-12.
- `microservices/tenancy/policy/data-residency.md` §"DSR Cascade".
- `microservices/tenancy/incident-response.md` §"Regulatory Notifications".
- GDPR Art. 17 + Art. 12(3); KR PIPA Art. 36; DPDPA 2023 §12; LGPD Art. 18.
- ICO right-to-erasure guidance — `ico.org.uk`.
