---
doc_class: ImplementationPlan
id: IP-012
title: "oya-payments-subscription-usecase — CreateSubscription, RenewSubscription, DunningWorker"
microservice: payments
bounded_context: subscription-lifecycle
layer: usecase
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤600 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0243
  - ADR-0246
  - ADR-0252
diataxis_quadrant: how-to
doc_status: published
---

# IP-012 — oya-payments-subscription-usecase

## Purpose

Implement `CreateSubscriptionUseCase`, `RenewSubscriptionUseCase`, and `DunningWorker` (scheduled job). The dunning worker runs as a K8s CronJob and drives the dunning-step sequence.

## Acceptance criteria

- [ ] `CreateSubscriptionUseCase::execute(cmd)` steps: (1) Cedar eval, (2) COPPA/KOSA age check, (3) `Subscription::new()`, (4) if `trial_end` set, schedule trial-conversion job, (5) persist, (6) emit `SubscriptionCreatedEvent`.
- [ ] `RenewSubscriptionUseCase::execute(cmd)` steps: (1) advance `BillingCycle`, (2) call `CreateChargeUseCase` (reuse charge pipeline), (3) on success → `Subscription::renew()`, on failure → `Subscription::mark_past_due()` + trigger dunning step 1.
- [ ] `DunningWorker::run()` (CronJob, runs every 6h): load `past_due` subscriptions; for each, apply next `DunningStep` (retry charge or send notification or cancel); emit `SubscriptionDunningAttemptedEvent`.
- [ ] Usage-based billing: `RecordUsageUseCase` accumulates `UsageRecord` entries; `BillingCycle` metering sums usage at period end.
- [ ] Pause/resume: `PauseSubscriptionUseCase` / `ResumeSubscriptionUseCase` with prorated billing.
- [ ] Financial inclusion: `pay_as_you_go` plan path supported — no upfront charge; metered at end of period per §3.2.5 row 15.
- [ ] Unit tests ≥ 20: trial conversion, renewal failure → dunning, dunning escalation to cancel, usage metering, pause/resume proration.

## Dependencies

- IP-011 (subscription domain), IP-003 (charge usecase — reused for renewal charge).

## Cross-references

- `IP-011-payments-domain-subscription.md` — aggregate.
- `capabilities/subscription-lifecycle.yaml` — capability record.
- `iac/helm/payments-app/values.yaml` — CronJob config for DunningWorker.

## Counterpart gap row

| Counterpart | Relevant behavior | Usecase gap closed |
|---|---|---|
| Stripe | Billing renewal and smart-retry orchestration | `DunningWorker` gives Oyatie a PSP-independent renewal and retry path. |
| Chargebee | Dunning workflow and subscription operations | The usecase coordinates renewal, usage metering, pause/resume, and cancellation inside Oyatie. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-012-payments-usecase-subscription.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/payments/IP-012-payments-usecase-subscription.md` matched `metered`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/payments/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
