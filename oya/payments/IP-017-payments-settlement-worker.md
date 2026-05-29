---
doc_class: ImplementationPlan
id: IP-017
title: "oya-payments-settlement-worker — daily reconciliation CronJob"
microservice: payments
bounded_context: settlement
layer: worker
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-treasury
pr_size_estimate: "≤400 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0248
  - ADR-0252
  - ADR-0254
diataxis_quadrant: how-to
doc_status: published
---

# IP-017 — oya-payments-settlement-worker

## Purpose

Implement `ReconciliationWorker` as a K8s CronJob (daily at 02:00 UTC). Fetches PSP settlement reports, reconciles against internal ledger, creates `SettlementBatch` aggregates, and surfaces discrepancies.

## Acceptance criteria

- [ ] CronJob `reconciliation-worker` runs at `0 2 * * *`; completes within 30 min for < 1M charges/day.
- [ ] `ReconciliationWorker::run()` steps: (1) fetch PSP settlement report via `PspAdapter::fetch_settlement_report(date)`, (2) load internal charges/payouts for the period from CRDB, (3) `Reconciliation::reconcile()` → produce `DiscrepancyDetectedEvent` per discrepancy, (4) persist `SettlementBatch`, (5) emit audit events.
- [ ] Per-PSP parallelism: runs Stripe / Adyen / Toss reconciliation concurrently (Tokio task per PSP); overall timeout 25 min.
- [ ] Discrepancies > $10k: emits `oya.payments.settlement.high-value-discrepancy` audit event + pages ops-treasury.
- [ ] Idempotent: if run twice for same date, second run is a no-op (checks `SettlementBatch` existence).
- [ ] Integration test: mock PSP settlement report with 1 missing charge → verifies `DiscrepancyDetectedEvent` emitted.
- [ ] Helm CronJob manifest in `iac/helm/payments-app/values.yaml` (already wired under `reconciliationWorker` key).

## Dependencies

- IP-016 (settlement domain), IP-001 (kernel).

## Cross-references

- `IP-016-payments-settlement-domain.md` — aggregate.
- `runbooks/double-charge-detected.md` — response if discrepancy found.
- `iac/helm/payments-app/values.yaml` — `reconciliationWorker` CronJob config.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-017-payments-settlement-worker.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
