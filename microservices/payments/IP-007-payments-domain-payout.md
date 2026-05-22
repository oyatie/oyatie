---
doc_class: ImplementationPlan
id: IP-007
title: "oya-payments-payout-domain — Payout aggregate, BankAccount, CoolingPeriod"
microservice: payments
bounded_context: payout
layer: domain
status: accepted
date: 2026-05-20
owner_team: axis-payments + ops-treasury
pr_size_estimate: "≤500 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0252
diataxis_quadrant: how-to
doc_status: published
---

# IP-007 — oya-payments-payout-domain

## Purpose

Implement the `Payout` aggregate with cooling-period enforcement, bank-account verification invariant, and TrueTime-aware timestamp for monotonic financial reconciliation per ADR-0252.

## Acceptance criteria

- [ ] `Payout` aggregate states: `Pending | Scheduled | Initiated | Completed | Failed | Reversed`.
- [ ] `CoolingPeriod` value object: configurable per sub-merchant tier (default 7 days for new sub-merchants, 2 days for verified, 0 for Stripe Express accounts with instant-payout enabled).
- [ ] Invariant: `BankAccount` must be in `Verified` state before payout initiates; else `PayoutError::BankAccountNotVerified`.
- [ ] Invariant: cooling-period check uses **TrueTime** (monotonic) per ADR-0252 `TrueTimePort`; HLC fallback only in dev/test.
- [ ] SOX dual-signoff invariant: `amount_minor > 1_000_000_00` (> $1M USD-equivalent) requires `dual_signoff_token` per `pack-sox-itgc`.
- [ ] `PayoutSchedule` entity: `frequency` (daily | weekly | on-demand), `anchor_day`, `currency`, `psp`.
- [ ] `BankAccount` entity: `id`, `tenant_id`, `routing_number_encrypted`, `account_number_last4`, `verified_at`, `country`, `currency`.
- [ ] Domain events: `PayoutScheduledEvent`, `PayoutInitiatedEvent`, `PayoutCompletedEvent`, `PayoutFailedEvent`, `PayoutReversedEvent`.
- [ ] `cargo test -p oya-payments-payout-domain` ≥ 15 tests: cooling period, bank account unverified, SOX dual-signoff, TrueTime mock.

## Dependencies

- IP-001 (kernel shared types).

## Cross-references

- `IP-008-payments-usecase-payout.md` — orchestrates this aggregate.
- `policy/payout-authorization.cedar` — Cedar gate.
- `runbooks/payout-failed.md` — operational response.
- `ARCHITECTURE.md §time-coordination` — TrueTime opt-in for payout BC.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-007-payments-domain-payout.md` matched `financial, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
