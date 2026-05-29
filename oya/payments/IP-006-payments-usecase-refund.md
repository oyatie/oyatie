---
doc_class: ImplementationPlan
id: IP-006
title: "oya-payments-refund-usecase — IssueRefund orchestration"
microservice: payments
bounded_context: refund
layer: usecase
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤400 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0246
diataxis_quadrant: how-to
doc_status: published
---

# IP-006 — oya-payments-refund-usecase

## Purpose

Implement `IssueRefundUseCase` application service: Cedar → original charge lookup → refund window check → PSP refund call → aggregate persist → audit emit.

## Acceptance criteria

- [ ] `IssueRefundUseCase::execute(cmd)` steps: (1) Cedar eval `policy/refund-authorization.cedar`, (2) load original `Charge` via `ChargeRepository`, (3) load tenant refund window from Ontology, (4) `Refund::new()` with invariant checks, (5) call `PspAdapter::refund()`, (6) persist via `RefundRepository`, (7) emit `RefundRequestedEvent`.
- [ ] Cedar `DENY` → `RefundError::AuthorizationDenied`.
- [ ] Idempotency: if `find_by_idempotency_key` returns existing refund in terminal state, return cached; if in-flight, return `RefundError::ConflictInFlight`.
- [ ] If PSP returns `PspError::AlreadyRefunded`, mark refund `Succeeded` (idempotent PSP response).
- [ ] Unit tests ≥ 15: Cedar deny, window expired, over-refund, PSP error, idempotency replay.

## Dependencies

- IP-005 (refund domain), IP-001 (kernel), IP-004 (Stripe adapter for integration test).

## Cross-references

- `IP-005-payments-domain-refund.md` — aggregate.
- `policy/refund-authorization.cedar` — Cedar gate.
- `runbooks/refund-mismatch.md` — operational response to refund discrepancies.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-006-payments-usecase-refund.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
