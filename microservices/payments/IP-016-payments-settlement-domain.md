---
doc_class: ImplementationPlan
id: IP-016
title: "oya-payments-settlement-domain — SettlementBatch, Reconciliation, Discrepancy"
microservice: payments
bounded_context: settlement
layer: domain
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-treasury
pr_size_estimate: "≤500 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0244
  - ADR-0252
diataxis_quadrant: how-to
doc_status: published
---

# IP-016 — oya-payments-settlement-domain

## Purpose

Implement the `SettlementBatch` aggregate, `Reconciliation` entity, and `Discrepancy` value object. Settlement uses TrueTime monotonic timestamps per ADR-0252 (opt-in for financial reconciliation).

## Acceptance criteria

- [ ] `SettlementBatch` aggregate: `batch_id`, `tenant_id`, `psp`, `currency`, `period_start`, `period_end`, `expected_amount`, `settled_amount`, `state` (Pending | Settled | Reconciled | Discrepant).
- [ ] `Reconciliation` entity: matches internal charge/payout ledger against PSP settlement report; emits `DiscrepancyDetectedEvent` if `|expected_amount - settled_amount| > tolerance`.
- [ ] `Discrepancy` value object: `discrepancy_id`, `batch_id`, `type` (Underpayment | Overpayment | MissingCharge | ExtraCharge), `amount_delta`, `resolution`.
- [ ] TrueTime: `period_start`/`period_end` use `TrueTimePort` for monotonic cross-region correctness per ADR-0252.
- [ ] `SettlementRepository` port: `save`, `find_by_tenant_and_period`, `find_discrepant`.
- [ ] SOX-ITGC: discrepancies > $10k require dual-signoff before `Resolution::Accepted`.
- [ ] Domain events: `SettlementBatchCreatedEvent`, `DiscrepancyDetectedEvent`, `DiscrepancyResolvedEvent`.
- [ ] `cargo test -p oya-payments-settlement-domain` ≥ 12 tests: discrepancy detection, SOX dual-signoff, TrueTime mock.

## Dependencies

- IP-001 (kernel shared types).

## Cross-references

- `IP-017-payments-settlement-worker.md` — reconciliation CronJob.
- `runbooks/double-charge-detected.md` — discrepancy response.
- `ARCHITECTURE.md §time-coordination` — TrueTime opt-in for settlement.
- `compliance.md §11` — SOX-ITGC dual-signoff.

## Counterpart gap row

| Counterpart | Relevant behavior | Domain gap closed |
|---|---|---|
| Stripe | Balance transactions and settlement reconciliation | `SettlementBatch` lets Oyatie reconcile expected ledger entries against PSP settlement reports. |
| Adyen | Settlement detail reports and discrepancy handling | The domain captures discrepancies and SOX signoff without binding to one PSP report format. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-016-payments-settlement-domain.md` matched `financial, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
