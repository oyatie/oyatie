---
doc_class: ImplementationPlan
id: IP-011
title: "oya-payments-subscription-domain — Subscription, BillingCycle, DunningStep, Trial"
microservice: payments
bounded_context: subscription-lifecycle
layer: domain
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤600 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0244
  - ADR-0252
diataxis_quadrant: how-to
doc_status: published
---

# IP-011 — oya-payments-subscription-domain

## Purpose

Implement the `Subscription` aggregate with billing-cycle management, dunning steps, trial periods, and usage-based billing accumulation.

## Acceptance criteria

- [ ] `Subscription` aggregate states: `Trialing | Active | PastDue | Unpaid | Cancelled | Paused`.
- [ ] `BillingCycle` entity: `period_start`, `period_end`, `amount_due`, `invoice_id`, `payment_attempt_count`.
- [ ] `DunningStep` value object: sequence of retry attempts (Day 1: retry; Day 3: retry; Day 7: downgrade + notify; Day 14: cancel). Steps configurable per tenant dunning policy.
- [ ] `Trial` entity: `trial_end`, `trial_type` (Free | Paid | Freemium), `convert_to_paid_at`.
- [ ] `UsageRecord` entity: `usage_quantity`, `recorded_at`, `billing_period`, `unit_price` — for usage-based billing accumulation.
- [ ] COPPA: `Subscription::new()` refuses `plan.is_recurring = true` if `audience_context.age_class == AgeClass::Kosa` and `!parent_consent_token.is_some()` per ADR-0292.
- [ ] Financial inclusion: `Subscription` supports `pay_as_you_go` billing model with no minimum commitment per §3.2.5 row 15.
- [ ] `SubscriptionRepository` port: `save`, `find_by_id`, `find_active_for_renewal`, `find_past_due`.
- [ ] Domain events: `SubscriptionCreatedEvent`, `SubscriptionRenewedEvent`, `SubscriptionDunningAttemptedEvent`, `SubscriptionCancelledEvent`.
- [ ] `cargo test -p oya-payments-subscription-domain` ≥ 18 tests: dunning sequence, trial conversion, usage accumulation, COPPA/KOSA refusal, pay-as-you-go.

## Dependencies

- IP-001 (kernel shared types).

## Cross-references

- `IP-012-payments-usecase-subscription.md` — orchestrates this aggregate.
- `capabilities/subscription-lifecycle.yaml` — capability record.
- `ARCHITECTURE.md §C` — subscription-lifecycle BC.

## Counterpart gap row

| Counterpart | Relevant behavior | Domain gap closed |
|---|---|---|
| Stripe | Billing subscription states, trials, invoices, and dunning | Oyatie owns subscription state and dunning transitions instead of outsourcing them to Stripe Billing. |
| Chargebee | Subscription and dunning product depth | The aggregate carries billing-cycle and usage records needed for Chargebee-grade lifecycle parity. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-011-payments-domain-subscription.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
