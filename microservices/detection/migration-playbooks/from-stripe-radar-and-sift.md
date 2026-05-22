---
doc_class: MigrationPlaybook
microservice: detection
vendor: Stripe Radar + Sift + Adyen RevenueProtect (consolidated migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Stripe Radar / Sift / Adyen RevenueProtect → oyatie detection

Audience: an oyatie tenant moving payment-fraud + account-takeover + chargeback-prediction from a managed SaaS fraud platform (Stripe Radar, Sift, or Adyen RevenueProtect) to oyatie's native `detection` µservice.

## Why this migration is non-trivial

Managed fraud SaaS platforms hold the *training data* + *learned models* + *historical decisions* — these are not portable. What IS portable:

- The historical decisions ledger (which transactions fired, what action, what reason code) — exportable.
- The chargeback labels (which transactions were later disputed by the cardholder + the chargeback reason code) — exportable from the issuer/acquirer side.
- The tenant's own raw transaction data — already in the tenant's database; not held by the SaaS.

What IS NOT portable:

- The vendor's learned models (proprietary).
- The vendor's global cross-tenant signals (e.g., Sift's "this device fingerprint appears across N tenants" signal).
- The vendor's accumulated false-positive labels from the tenant's appeal queue.

The migration approach: bootstrap with rules-only (the 8 detection families' rule sets, plus tenant-authored Cedar rules), then train models on the tenant's own decision ledger over 90-180 d. Bridge the gap with a more-aggressive rule set during the model training period.

## Step 1 — Export historical decisions from the source vendor (≤ 1-3 days)

Stripe Radar:

```sh
# Stripe Radar evaluations export
oya detection migrate inventory \
    --source stripe-radar \
    --stripe-api-key "$STRIPE_API_KEY" \
    --window-days 365 \
    --out inventory/stripe-radar-365d.jsonl
```

Stripe's API rate-limits at ~ 100 requests/sec; for a 365-day, 100M-transaction tenant, plan 24-48 h export.

Sift:

```sh
oya detection migrate inventory \
    --source sift \
    --sift-account-id "$SIFT_ACCOUNT_ID" \
    --sift-api-key "$SIFT_API_KEY" \
    --window-days 365 \
    --out inventory/sift-365d.jsonl
```

Adyen RevenueProtect:

```sh
oya detection migrate inventory \
    --source adyen \
    --adyen-merchant-account "$ADYEN_MERCHANT_ACCOUNT" \
    --adyen-api-key "$ADYEN_API_KEY" \
    --window-days 365 \
    --out inventory/adyen-365d.jsonl
```

Each export contains: transaction_id, evaluation_id, score, action_taken (block / allow / step-up / review), risk_reason_codes, evaluator_version, timestamp.

## Step 2 — Acquire chargeback labels (≤ 1-2 days)

Chargebacks are reported by the issuing bank ~ 30-90 d after the transaction. Acquire from your acquirer (Adyen, Stripe, Worldpay, etc.):

```sh
oya detection migrate chargeback-labels \
    --acquirer stripe \
    --window-days 540 \    # 365d + 180d chargeback lag
    --out inventory/chargeback-labels-540d.jsonl
```

The labels contain: transaction_id, chargeback_filed_date, chargeback_reason_code (Reason 10.4 fraud, 13.1 services not provided, etc.), final_disposition (won_dispute, lost_dispute, accepted_chargeback).

Cross-join the evaluations + chargebacks to compute the true-positive / false-positive / true-negative / false-negative per the source vendor's decision ledger.

## Step 3 — Validate vendor decision quality + build the gap baseline (≤ 1-2 days)

```sh
oya detection migrate validate-baseline \
    --evaluations inventory/stripe-radar-365d.jsonl \
    --labels inventory/chargeback-labels-540d.jsonl \
    --out baseline/stripe-radar-baseline.yaml
```

Expected output (illustrative):

```yaml
source: stripe-radar
window_days: 365
total_evaluated: 102_487_213
total_blocked: 287_412
chargeback_rate_evaluated: 0.0042
chargeback_rate_blocked: 0.0001
chargeback_rate_allowed: 0.0044
false_positive_rate: 0.0091
false_negative_rate: 0.0044
disparate_impact_4_5ths: 0.87  # 4/5ths rule borderline; investigate
```

Use this as the floor — oyatie must match or exceed these numbers within 6 months of migration to declare success.

## Step 4 — Author tenant-specific Cedar rules (≤ 1-2 weeks per family)

Work with the tenant's BSA + fraud teams to author the initial rule set. For payment-fraud, the rule set should cover:

1. Card-testing velocity (per `tutorials/build-payment-fraud-cedar-rule.md`).
2. Geo-velocity (transactions in 2 countries in < 1 h).
3. Cross-currency divergence (card-currency vs principal-country).
4. New-account-with-high-velocity (account < 30 d + > 3 transactions / day).
5. Known-bad-bin + known-bad-issuer denylists.
6. Step-up-auth-on-marginal-score (model scores 0.5-0.8 → 3D-Secure challenge).

Each rule deploys via `oya detection rule deploy` (per the tutorial). Validate against the historical decision ledger:

```sh
oya detection rule shadow-replay \
    --rule-set rules/payment-fraud-card-testing.cedar \
    --historical inventory/stripe-radar-365d.jsonl \
    --chargeback-labels inventory/chargeback-labels-540d.jsonl \
    --out shadow/cedar-replay.yaml
```

The shadow-replay shows: which historical transactions our rules would have blocked, which we'd have allowed, the resulting chargeback-rate, the false-positive-rate. Compare against the vendor baseline.

## Step 5 — Train tenant-specific ONNX models (≤ 4-12 weeks)

Once the rule set is deployed in shadow mode (per Step 6), accumulate the tenant's own decision ledger + chargeback labels for 90-180 d. Then train:

```sh
oya detection model train \
    --family payment-fraud \
    --tenant drill-acme \
    --training-data ./oyatie-decisions-90d.parquet \
    --label-source chargeback \
    --output models/payment-fraud-v1.onnx
```

The training pipeline (per ADR-0308):

1. Splits 80/10/10 train / validation / holdout.
2. Trains an ONNX-compatible model (Random Forest / XGBoost / a small MLP).
3. Computes drift baseline + fairness audit on holdout.
4. Generates model card (per IP-017).

The CI gate `detection-model-card-acceptance` blocks deploy if fairness audit fails any pack.

## Step 6 — Shadow + cutover (≤ 6-12 months)

Run oyatie + vendor in parallel:

- Vendor remains the "decision" source for 30-180 d.
- oyatie shadow-scores every transaction; emits `mitigation_action_would_be` events without enforcing.
- Daily cron job compares oyatie's would-be-action vs the vendor's actual-action. Drift > 5 % triggers human review.
- After 90 d of < 5 % drift + better false-positive rate + equivalent chargeback-rate, cut over to oyatie as the source of truth.

Monitor the cutover via:

```sh
oya detection migrate cutover-status \
    --tenant drill-acme \
    --source stripe-radar
```

The dashboard surfaces: shadow-vs-vendor drift, chargeback-rate delta, false-positive-rate delta, appeal-throughput delta.

## Step 7 — Decommission the vendor

After the cutover period + a 30-d-buffer, decommission the vendor:

```sh
oya detection migrate decommission \
    --tenant drill-acme \
    --source stripe-radar \
    --evidence-out evidence/migrations/stripe-radar-to-oyatie-acme.json
```

The evidence file enumerates: inventory size, baseline metrics, rule-deploy timeline, model-train + promote evidence, shadow-period drift, cutover decision, vendor cancellation date.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Vendor API rate-limit slows export | Medium | Schedule export over weekends; budget 3× expected wall-clock. |
| Chargeback labels lag 30-90 d | High | Wait for label maturity before evaluating; do NOT cut over without ≥ 180 d of labeled decisions. |
| Vendor's cross-tenant signals not portable | Critical | Accept the gap; the shadow period quantifies the gap; if oyatie can't close the gap with tenant-scoped models, the migration fails. |
| Disparate-impact ratio worse on oyatie than vendor | Critical | Fairness audit blocks deploy; investigate + retrain with fairness-aware loss. |
| Vendor contract cancellation lock-in | Medium | Check contract; cancel with 30-90 d notice before cutover. |
| Acquirer chargeback feed format mismatch | Medium | The `inventory/chargeback-labels.jsonl` schema is normalized; if your acquirer doesn't match, write an adapter. |
| Step-up-auth UX regression (oyatie's 3DS challenge has different UX than vendor's) | High | UX test on a pilot tenant before cutover; tune challenge-rate to match vendor's. |
| Stripe Radar's "Block all" customer rules silently superseded | Medium | Migrate customer rules to oyatie Cedar BEFORE cutover; do not assume vendor rules transfer automatically. |
