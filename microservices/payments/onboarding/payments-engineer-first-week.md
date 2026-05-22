---
doc_class: Onboarding
microservice: payments
persona: payments-engineer + fintech-platform-engineer
related_adrs: [ADR-0263, ADR-0251, ADR-0131, ADR-0316]
date: 2026-05-20
doc_status: published
---

# Payments Engineer onboarding — first 5 working days on `payments`

Audience: a new payments engineer or fintech-platform engineer joining the `payments` rotation. By Day-5 they will have: stood up a DemoTrial cell, processed sandbox charges via Stripe + Adyen, exercised a refund + chargeback workflow, executed a multi-currency settlement drill, and walked the PSP-failover runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Focus on the per-tenant fee schedule + double-entry ledger sections.
2. Read `ARCHITECTURE.md` § PCI-DSS-CDE-isolation + § PSP-routing-policy + § currency-arithmetic (∼ 60 min).
3. Open the Grafana folder `payments`. Identify the seven paiden boards: `payments-auth-latency`, `payments-settlement-lag`, `payments-psp-error-rate`, `payments-currency-conversion-margin`, `payments-chargeback-rate`, `payments-fraud-flag-rate`, `payments-ledger-imbalance`.
4. Walk `runbooks/README.md`. The on-call runbooks: `psp-outage-failover.md`, `ledger-imbalance.md`, `chargeback-stuck-evidence.md`, `fx-rate-stale.md`, `settlement-stuck.md`, `webhook-replay.md`, `card-vault-decrypt-failure.md`, `pci-scope-violation.md`, `fincen-sar-flag.md`, `gdpr-payment-data-deletion.md`.
5. Sit in on the Wednesday payments-substrate handoff. Watch how the outgoing rotation reads the past-week chargeback ledger, the FX-margin board, and the PSP-error-rate split.

Acceptance: you can sketch the auth path: tenant API → Cedar gate → routing policy (choose PSP) → PSP SDK call → idempotency-key Valkey insert → ledger debit/credit posting → audit-chain emit → response.

## Day 2 — DemoTrial payments cell bootstrap

```sh
cargo run -p oya-dev-cli -- payments bootstrap \
    --profile demo_trial \
    --cell drill-syd-1 \
    --psp-mode sandbox \
    --stripe-secret-key-secret-name stripe-sandbox-test \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/payments \
    --valkey-endpoint valkey://drill-valkey-syd-1:6379 \
    --pulsar-endpoint pulsar://drill-pulsar-syd-1:6650 \
    --audit-chain-endpoint http://drill-audit-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 12 min. Verify after bootstrap:

```sh
oya payments health --cell drill-syd-1
# Expected:
#   psp.stripe.sandbox: connected (api-version=2025-09-30.basil)
#   ledger.postgres:    up (lag_ms=12 from primary)
#   audit_chain.emit:   up (last_event=2026-05-20T14:00:01Z)
#   valkey.idempotency: up (3 nodes, 0 keys cached)
#   cilium.policy:      applied (pci_scope=l0_sandbox)
```

`pci_scope=l0_sandbox` is DemoTrial-only; it does NOT permit production CHD.

Acceptance: cell live, you can describe why we don't run production PSPs at DemoTrial (no PCI-DSS attestation; Cilium policy blocks production PSP egress).

## Day 3 — First sandbox charge + ledger walk

Create a tenant + customer in sandbox:

```sh
oya payments tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --fee-schedule-template default \
    --currencies-allowed USD \
    --pci-scope l0_sandbox

oya payments customer create \
    --tenant drill-acme \
    --customer-id cust-001 \
    --email test@example.com \
    --idempotency-key onboard-cust-001
```

Tokenize a Stripe-test card (the famous 4242…):

```sh
oya payments method tokenize \
    --tenant drill-acme \
    --customer cust-001 \
    --psp stripe \
    --card-number 4242424242424242 \
    --exp-month 12 --exp-year 2030 --cvc 123 \
    --idempotency-key tok-cust-001-card-1
# Returns: payment_method_id=pm_drill_001, psp_token=pm_1QabcDEF...
```

Create a charge:

```sh
oya payments charge create \
    --tenant drill-acme \
    --customer cust-001 \
    --payment-method-id pm_drill_001 \
    --amount-minor-units 12500 \
    --currency USD \
    --description "Day-3 onboarding test charge" \
    --idempotency-key charge-day3-001
# Returns: charge_id=ch_drill_001, status=succeeded, fee_minor_units=305 (= 2.9% + 30¢ stripe fee), settlement_amount_minor_units=12195
```

Inspect the ledger:

```sh
psql postgres://drill-pg-syd-1:5432/payments -c "
    SELECT posting_id, account_path, debit_minor, credit_minor, ts
    FROM payments.ledger_posting
    WHERE transaction_id = 'ch_drill_001'
    ORDER BY ts, posting_id"
```

Expected:

| account_path | debit_minor | credit_minor |
|---|---:|---:|
| tenant.drill-acme.receivable.stripe | 12500 | 0 |
| tenant.drill-acme.revenue | 0 | 12500 |
| tenant.drill-acme.expense.psp_fee | 305 | 0 |
| tenant.drill-acme.payable.stripe_fee | 0 | 305 |

Note: each transaction produces multiple postings (gross charge + PSP fee split). Sum of debits = sum of credits. The `ledger-imbalance` Grafana panel alerts within 60 s if this invariant breaks.

Audit-chain verification:

```sh
oya audit query --tenant drill-acme --event-class payments.charge.created --since 5m
# Expected: 1 event with the charge_id, amount, currency, PSP, principal.
```

Acceptance: charge succeeds, ledger balances, audit event emitted.

## Day 4 — Refund + chargeback workflow

Issue a partial refund:

```sh
oya payments refund create \
    --tenant drill-acme \
    --charge ch_drill_001 \
    --amount-minor-units 5000 \
    --reason customer_requested \
    --idempotency-key refund-day4-001
# Returns: refund_id=re_drill_001, status=succeeded
```

Ledger should now show:

| account_path | debit_minor | credit_minor |
|---|---:|---:|
| tenant.drill-acme.revenue | 5000 | 0 |
| tenant.drill-acme.receivable.stripe | 0 | 5000 |
| tenant.drill-acme.expense.psp_fee | 0 | 30 |
| tenant.drill-acme.payable.stripe_fee | 30 | 0 |

(Stripe refunds the full PSP fee on a partial refund — the 30¢ flat is refunded, the 2.9% is pro-rated; this is a Stripe-specific behavior the routing policy normalizes.)

Now simulate a chargeback. Stripe's test-mode chargeback trigger:

```sh
oya payments simulate chargeback \
    --tenant drill-acme \
    --charge ch_drill_001 \
    --reason fraudulent \
    --network visa
# Returns: chargeback_id=cb_drill_001, status=open, evidence_deadline=2026-06-13T14:00:00Z
```

This emits `payments.chargeback.opened` to audit-chain + Pulsar. The merchant has 21 d (Visa rules; 14 d for Mastercard) to upload evidence.

Upload evidence:

```sh
oya payments chargeback evidence-upload \
    --tenant drill-acme \
    --chargeback cb_drill_001 \
    --evidence-file ./tx-screenshot.jpg \
    --evidence-class transaction_screenshot

oya payments chargeback evidence-upload \
    --tenant drill-acme \
    --chargeback cb_drill_001 \
    --evidence-file ./ship-confirmation.pdf \
    --evidence-class shipping_documentation

oya payments chargeback submit --tenant drill-acme --chargeback cb_drill_001
# Returns: submitted_at=2026-05-20T16:14:23Z, awaiting_network_decision_p99=14_days
```

Acceptance: refund flow + chargeback evidence upload + submission. You can articulate the 14/21-day window difference + the per-network rules.

## Day 5 — Multi-currency settlement + PSP failover drill

Cross-currency charge (USD-quoted, EUR-settled; only available when promoted to paid, but the API surface works at DemoTrial sandbox):

```sh
oya payments charge create \
    --tenant drill-acme \
    --customer cust-001 \
    --payment-method-id pm_drill_001 \
    --amount-minor-units 12500 \
    --currency-presented USD \
    --currency-settled EUR \
    --idempotency-key charge-fx-day5-001 \
    --fx-rate-source stripe-test-fx
# Returns: charge_id=ch_drill_002, fx_rate=0.9120, settlement_amount_minor_units=11400 (= 12500 * 0.9120)
```

Note the FX margin: at DemoTrial it's the Stripe test-FX which is the same as Stripe production-FX. At paid, you configure per-tenant FX margin atop a wholesale rate (Refinitiv / OANDA / Bloomberg BFIX).

Now the PSP failover drill. Read `runbooks/psp-outage-failover.md` first.

```sh
oya payments drill psp-outage \
    --cell drill-syd-1 \
    --target-psp stripe \
    --duration 5m
```

The drill:

1. Blocks egress to `api.stripe.com` (via Cilium NetworkPolicy `block-stripe-egress`).
2. Continues issuing charges.
3. Charges should route to the fallback PSP (Adyen, then Checkout.com, then PayPal-Braintree as configured in `routing-policy.yaml`).
4. Observes the `payments-psp-error-rate` panel — Stripe error rate should hit 100 %; fallback PSPs should hit < 1 %.
5. Removes the block after 5 min.

Expected result: zero customer-visible failures (all charges succeed via fallback), Stripe errors recorded in audit-chain, alerts page on-call.

Walk the runbook end-to-end: identify the offending PSP, decide whether to demote it permanently (e.g., Stripe just announced an outage in their status page → demote until status-page green), monitor for ledger consistency (a charge that started on Stripe and fell back to Adyen MUST appear in the ledger as one transaction with a `routed_via_fallback=true` attribute).

Acceptance: drill executed, you understand the multi-PSP routing policy, you can recover from a sustained PSP outage.

## What you've learned

- DemoTrial bootstrap + sandbox charge end-to-end with double-entry ledger walk.
- Refund + chargeback evidence workflow + per-network rule differences (Visa 21 d, MC 14 d).
- Multi-currency settlement + FX margin model.
- PSP failover drill + routing-policy.yaml mechanics.

Next week: Paid promotion (PCI-DSS L1 attestation walkthrough), Paid promotion (RTGS correspondent-bank onboarding + ISO 20022 message authoring), Paid tour (per-pack PSP allowlists + regulator-direct reporting), and your first production shadow.
