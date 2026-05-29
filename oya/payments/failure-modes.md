---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + ops-sre-reliability + ops-fraud
related_adrs: [ADR-0028, ADR-0145, ADR-0244, ADR-0248, ADR-0263]
companion_docs:
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/runbooks/psp-outage.md
  - microservices/payments/runbooks/double-charge-detected.md
  - microservices/payments/runbooks/payout-failed.md
  - microservices/payments/runbooks/dispute-escalation.md
  - microservices/payments/runbooks/fraud-spike-detected.md
  - microservices/payments/runbooks/refund-mismatch.md
diataxis_quadrant: reference
doc_status: published
---

# Failure Modes — payments µservice

> Per-failure-mode trigger, severity, blast radius, detection, mitigation, runbook ref. Hyperscaler-grade failure-mode tree per documentation-rigor.md §1.1 sub-test.

---

## FM-01 — PSP outage (Stripe / Adyen / Toss / KakaoPay / LINE Pay / WeChat Pay / Alipay)

| Attribute | Value |
|---|---|
| Trigger | PSP-side outage; success-rate <99% for ≥5 min |
| Severity | Sev-1 (single PSP) / Sev-1+ (multiple PSPs simultaneously) |
| Likelihood | High — every PSP has ≥1 publicly-disclosed outage/year |
| Blast radius | Per-region charges that route to the affected PSP; per-tenant if tenant pinned to single PSP |
| Detection | `payments_charge_total{outcome="errored",psp="stripe"} / payments_charge_total{psp="stripe"} > 0.01` for ≥5 min; PSP-status-page integration |
| Mitigation | Per-tenant fallback PSP (where tenant.payment_routing_policy permits); charge-queue + retry on recovery; degraded-mode UI on tenant surface |
| Runbook | [`runbooks/psp-outage.md`](runbooks/psp-outage.md) |
| Audit-event class | `oya.payments.charge.errored` |

## FM-02 — Double-charge from PSP-retry race

| Attribute | Value |
|---|---|
| Trigger | Idempotency-key collision; PSP times out → we retry → original succeeds |
| Severity | Sev-2 |
| Likelihood | Medium — observed ~0.001% of charges absent idempotency-key |
| Blast radius | Per-charge; specific consumer affected |
| Detection | `payments_charge_total` row count diverges from PSP-side charge-count for same idempotency-key; daily reconciliation worker flags |
| Mitigation | UNIQUE constraint on `(tenant_id, idempotency_key)`; idempotency-key derived deterministically from `(intent_id + amount + currency + payment_method_id)`; refund the duplicate within 24h via auto-refund |
| Runbook | [`runbooks/double-charge-detected.md`](runbooks/double-charge-detected.md) |
| Audit-event class | `oya.payments.charge.double-charge-detected` |

## FM-03 — Payout failure (bank rejects)

| Attribute | Value |
|---|---|
| Trigger | Bank returns the payout (insufficient funds at platform-master / invalid bank-account / closed-account / sanctions-screen hit) |
| Severity | Sev-2 |
| Likelihood | Medium — observed ~0.1% of payouts |
| Blast radius | Per-payout; recipient cannot withdraw funds |
| Detection | `payments_payout_failed_total` counter; PSP-payout-webhook with reason-code |
| Mitigation | Auto-retry with backoff (24h, 72h, 7d) for transient reasons; freeze + notify for compliance reasons (sanctions / KYB-revoked) |
| Runbook | [`runbooks/payout-failed.md`](runbooks/payout-failed.md) |
| Audit-event class | `oya.payments.payout.failed` |

## FM-04 — Partial refund mismatch

| Attribute | Value |
|---|---|
| Trigger | Refund-amount > original-charge-amount (math error or original-charge already partially refunded) |
| Severity | Sev-3 |
| Likelihood | Low |
| Blast radius | Per-charge |
| Detection | DDL-CHECK: `SUM(refunds.amount_minor) <= charges.amount_minor` per charge; pre-write validation |
| Mitigation | Refund rejected at usecase layer before PSP call; error returned to caller |
| Runbook | [`runbooks/refund-mismatch.md`](runbooks/refund-mismatch.md) |
| Audit-event class | `oya.payments.refund.rejected` |

## FM-05 — Dispute cascade (mass-chargeback storm)

| Attribute | Value |
|---|---|
| Trigger | Sub-merchant has a fraud-spike → cards being charged are stolen → many simultaneous chargebacks |
| Severity | Sev-1 (if sub-merchant > $100k/day) / Sev-2 (sub-merchant < $100k/day) |
| Likelihood | Medium (esp. in marketplace flows) |
| Blast radius | Per-sub-merchant; potentially per-platform if card-network monitoring threshold breached |
| Detection | `payments_dispute_open_total{sub_merchant_id=X}` rate-of-change >10x baseline; PSP `chargeback_threshold_warning` webhook |
| Mitigation | Auto-restrict sub-merchant (Cedar gate `sub-merchant.restricted=true`); freeze payouts pending review; notify ops-fraud |
| Runbook | [`runbooks/dispute-escalation.md`](runbooks/dispute-escalation.md) |
| Audit-event class | `oya.payments.sub-merchant.restricted` + `oya.payments.dispute.opened` |

## FM-06 — Currency exchange-rate divergence

| Attribute | Value |
|---|---|
| Trigger | Cross-currency charge: rate quoted at checkout vs rate at settlement diverges materially |
| Severity | Sev-3 (small) / Sev-2 (>1% divergence on >$10k charge) |
| Likelihood | Low — Stripe / Adyen handle rate-lock at authorize |
| Blast radius | Per-charge |
| Detection | Daily reconciliation; tenant ledger vs PSP settlement-report divergence |
| Mitigation | Per-tenant policy: tenant accepts FX risk by default; option to net-settle daily |
| Runbook | (inline in `runbooks/refund-mismatch.md` §FX) |
| Audit-event class | `oya.payments.settlement.fx-divergence` |

## FM-07 — Fraud false-positive (legitimate charge blocked)

| Attribute | Value |
|---|---|
| Trigger | Fraud-ML scores a legitimate charge above decline-threshold |
| Severity | Sev-3 |
| Likelihood | Medium — industry baseline ~5% false-positive rate at default thresholds |
| Blast radius | Per-consumer; lost-revenue per declined charge |
| Detection | Customer-support tickets; chargeback-reversal rate; consumer-survey CSAT |
| Mitigation | Per-tenant adjustable decline-threshold; human-review queue for borderline scores; auto-allowlist on repeat-customer signal |
| Runbook | (inline in [`runbooks/fraud-spike-detected.md`](runbooks/fraud-spike-detected.md) §False-positive recovery) |
| Audit-event class | `oya.payments.charge.declined-fraud` |

## FM-08 — Subscription dunning failure

| Attribute | Value |
|---|---|
| Trigger | Subscription renewal fails (expired card / insufficient funds / hard-decline); dunning retries exhaust |
| Severity | Sev-3 |
| Likelihood | High — industry baseline ~5-10% of renewals fail at least once |
| Blast radius | Per-subscription |
| Detection | `payments_subscription_dunning_total{status="failed"}`; per-subscription state machine |
| Mitigation | Smart-retry schedule (24h / 72h / 7d / 14d); pre-expiry card-update prompt; tenant-configurable dunning policy |
| Runbook | (inline in [`runbooks/payout-failed.md`](runbooks/payout-failed.md) §Subscription-dunning) |
| Audit-event class | `oya.payments.subscription.dunning-attempted` + `oya.payments.subscription.cancelled` |

## FM-09 — Webhook replay (PSP resends stale event)

| Attribute | Value |
|---|---|
| Trigger | PSP retries a webhook after our 5-min replay-window expired |
| Severity | Sev-4 (mostly benign; just rejected) |
| Likelihood | Medium |
| Blast radius | Per-event |
| Detection | Replay-window check on inbound webhook |
| Mitigation | HMAC + replay-window ≤5min + idempotency-key dedup; rejected events logged but not retried |
| Runbook | (inline in [`runbooks/psp-outage.md`](runbooks/psp-outage.md) §Webhook-recovery) |
| Audit-event class | `oya.payments.webhook.replay-rejected` |

## FM-10 — Webhook-storm (PSP-incident-driven)

| Attribute | Value |
|---|---|
| Trigger | PSP has internal incident → webhooks delayed → backlog floods us when their queue drains |
| Severity | Sev-2 |
| Likelihood | Medium |
| Blast radius | Webhook-handler worker pool saturation |
| Detection | Webhook-queue-depth alarm; HPA-trigger on queue-depth |
| Mitigation | HPA scales handler replicas; per-PSP-rate-limit on ingress; backpressure to PSP via 503 (PSP retries) |
| Runbook | (inline in [`runbooks/psp-outage.md`](runbooks/psp-outage.md) §Webhook-storm) |
| Audit-event class | `oya.payments.webhook.queue-pressure` |

## FM-11 — Audit-chain Merkle break

| Attribute | Value |
|---|---|
| Trigger | Per-µservice signing key mismatch between sealed chain and verifier |
| Severity | Sev-1 (security-critical) |
| Likelihood | Low |
| Blast radius | All payments audit-events emitted in the affected window |
| Detection | Daily seal-verification CronJob fails; `oya.payments.audit.chain-break-detected` event |
| Mitigation | Rotate signing key via [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md); rebuild chain from event-stream replay; assess whether tampering or legitimate rotation gap |
| Runbook | [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md) |
| Audit-event class | `oya.payments.audit.chain-break-detected` |

## FM-12 — Cedar fragment drift (live state diverges from Git)

| Attribute | Value |
|---|---|
| Trigger | Cluster live Cedar fragment differs from Git-of-record (e.g., manual `kubectl edit`) |
| Severity | Sev-1 (security-critical) |
| Likelihood | Low |
| Blast radius | All actions evaluated by the drifted fragment |
| Detection | PreCheck CronJob compares live-state vs Git; `oya.payments.policy.live-drift-detected` event |
| Mitigation | Auto-revert to Git-of-record; investigate root cause; assess whether tampering |
| Runbook | [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md) §Cedar-drift |
| Audit-event class | `oya.payments.policy.live-drift-detected` |

## FM-13 — Cross-tenant payout misroute

| Attribute | Value |
|---|---|
| Trigger | Bug in payout-engine sends Tenant A's funds to Tenant B's bank-account |
| Severity | Sev-0 (catastrophic; financial-liability + criminal exposure) |
| Likelihood | Negligible (multi-layer defence: Cedar + DDL CHECK + dual-signoff) |
| Blast radius | Per-payout; potentially per-tenant if pattern |
| Detection | DDL-CHECK: `payouts.tenant_id == bank_accounts.tenant_id`; Cedar `payout-authorization.cedar` FORBID; dual-signoff audit-event missing |
| Mitigation | Pre-execution: dual signoff for payouts >$10k or new-bank-account; post-execution: rapid-revoke via PSP; financial-reserve held for first 14d of new-bank-account |
| Runbook | [`runbooks/payout-failed.md`](runbooks/payout-failed.md) §Misroute-recovery |
| Audit-event class | `oya.payments.payout.misroute-detected` |

## FM-14 — Sub-merchant KYC revoked mid-stream

| Attribute | Value |
|---|---|
| Trigger | PSP revokes sub-merchant KYC (e.g., sanctions hit, fraud finding) |
| Severity | Sev-2 |
| Likelihood | Low |
| Blast radius | Per-sub-merchant; their pending payouts frozen |
| Detection | PSP webhook `sub_merchant.restricted`; per-sub-merchant Cedar flag |
| Mitigation | Auto-freeze + notify + offer re-onboarding path; freeze payouts; do not refund consumers (KYC-revocation is a sub-merchant issue, not a consumer issue) |
| Runbook | [`runbooks/dispute-escalation.md`](runbooks/dispute-escalation.md) §Sub-merchant-restricted |
| Audit-event class | `oya.payments.sub-merchant.restricted` |

## FM-15 — Idempotency-key store outage

| Attribute | Value |
|---|---|
| Trigger | CRDB cluster on which idempotency-key UNIQUE lives is partially down |
| Severity | Sev-1 |
| Likelihood | Low (CRDB RF-3 multi-AZ) |
| Blast radius | All charges that need idempotency-check |
| Detection | DB-availability monitor; cell-degraded alarm |
| Mitigation | DR-cell failover per [`multi-region.md`](multi-region.md); within-cell: HPA + alternate AZ |
| Runbook | (inline in [`runbooks/psp-outage.md`](runbooks/psp-outage.md) §Internal-DB-outage) |
| Audit-event class | `oya.payments.charge.errored-internal` |

## FM-16 — OpenBao credential outage

| Attribute | Value |
|---|---|
| Trigger | OpenBao seal-state stuck or unreachable |
| Severity | Sev-1 |
| Likelihood | Low |
| Blast radius | All PSP-adapter calls (credentials cannot be fetched) |
| Detection | OpenBao read-failure rate >0.1% for 1 min |
| Mitigation | Auto-unseal per cloud-secrets runbook (out-of-our-scope); circuit-breaker on adapter; degraded-mode UI |
| Runbook | (refers to `microservices/cloud-secrets/runbooks/openbao-outage.md`) |
| Audit-event class | `oya.payments.credential.fetch-failed` |

## FM-17 — Settlement reconciliation discrepancy

| Attribute | Value |
|---|---|
| Trigger | Daily reconciliation worker finds CRDB ledger != PSP settlement report |
| Severity | Sev-2 (small discrepancy) / Sev-1 (>$10k discrepancy) |
| Likelihood | Medium (industry baseline ~0.05% of charges have a reconciliation gap due to FX rounding or PSP-side restatement) |
| Blast radius | Per-(tenant, currency, day) |
| Detection | Daily CronJob; `payments_settlement_discrepancy_amount_total` gauge |
| Mitigation | Per-discrepancy triage: timing, FX rounding, PSP restatement; manual reconciliation entry signed by ops-treasury |
| Runbook | (inline in [`runbooks/refund-mismatch.md`](runbooks/refund-mismatch.md) §Settlement-reconciliation) |
| Audit-event class | `oya.payments.settlement.discrepancy-detected` |

## Severity-class summary

| Severity | Definition | Examples |
|---|---|---|
| Sev-0 | Multi-tenant data integrity / financial-liability | FM-13 cross-tenant payout misroute |
| Sev-1 | Single-tenant data integrity OR multi-tenant availability OR security incident | FM-01 PSP outage (cascading), FM-11 audit-chain break, FM-12 Cedar drift, FM-15 idempotency-store outage, FM-16 OpenBao outage, FM-05 mass-chargeback |
| Sev-2 | Single-tenant availability or correctness | FM-02 double-charge, FM-03 payout-failed, FM-10 webhook-storm, FM-17 reconciliation-discrepancy (large), FM-14 sub-merchant KYC revoked |
| Sev-3 | Per-charge correctness; user-visible but recoverable | FM-04, FM-06, FM-07, FM-08, FM-17 (small) |
| Sev-4 | Benign / metric-only | FM-09 webhook-replay-rejected |

## References

- [`ARCHITECTURE.md`](ARCHITECTURE.md).
- [`runbooks/`](runbooks/).
- [`slos/`](slos/).
- [ADR-0028 — Merkle-sealed audit chain](../../docs/decisions/ADR-0028-audit-chain.md).
- [ADR-0263 — observability emission](../../docs/decisions/ADR-0263-observability-emission-contract.md).
