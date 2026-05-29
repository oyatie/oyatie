---
doc_class: ImplementationPlan
id: IP-008
title: "oya-payments-payout-usecase — SchedulePayout, InitiatePayout orchestration"
microservice: payments
bounded_context: payout
layer: usecase
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-treasury
pr_size_estimate: "≤500 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0246
  - ADR-0252
diataxis_quadrant: how-to
doc_status: published
---

# IP-008 — oya-payments-payout-usecase

## Purpose

Implement `SchedulePayoutUseCase` and `InitiatePayoutUseCase`. Handles cooling-period evaluation, step-up auth for large payouts, and PSP-specific payout flow (Stripe Express vs Standard vs Adyen MarketPay).

## Acceptance criteria

- [ ] `SchedulePayoutUseCase::execute(cmd)` steps: (1) Cedar eval `policy/payout-authorization.cedar`, (2) load `BankAccount` + verify state, (3) evaluate cooling period via TrueTime, (4) persist `Payout` in `Pending` state, (5) emit `PayoutScheduledEvent`.
- [ ] `InitiatePayoutUseCase::execute(cmd)` steps: (1) step-up auth check for `amount_minor > 1_000_000_00` (abuse-defence gate), (2) PSP payout call, (3) advance aggregate to `Initiated`, (4) emit `PayoutInitiatedEvent`.
- [ ] PSP routing: Stripe Express (`Stripe.payouts.create` on connected account) vs Stripe Standard (`Stripe.transfers.create` → `payouts.create`) vs Adyen MarketPay (`POST /fund/v6/transferFunds`).
- [ ] KR payout: Toss Payments settlement API; KRW-only; KR-FSS audit event on every payout.
- [ ] Unit tests ≥ 18: cooling-period block, bank-account unverified, Cedar deny, step-up required, PSP routing by psp_id, KR-FSS audit emit.

## Dependencies

- IP-007 (payout domain), IP-001 (kernel), IP-004 (Stripe adapter).

## Cross-references

- `IP-007-payments-domain-payout.md` — aggregate.
- `policy/payout-authorization.cedar` — Cedar gate.
- `runbooks/payout-failed.md` — failure handling.
- `compliance.md §2` — KR-FSS audit requirements.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-008-payments-usecase-payout.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
