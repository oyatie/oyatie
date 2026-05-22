---
doc_class: Onboarding
microservice: detection
persona: detection-engineer + ml-fairness-engineer
related_adrs: [ADR-0307, ADR-0308, ADR-0309, ADR-0310]
date: 2026-05-20
doc_status: published
---

# Detection Engineer onboarding — first 5 working days on `detection`

Audience: a new detection engineer or ML fairness engineer joining the `detection` rotation. By Day-5 they will have: deployed a streaming detection job, authored a Cedar rule + a small ONNX model, walked a model-drift incident, exercised the fairness-audit pipeline, and shadowed an investigation case adjudication.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Detection families + § Compliance (∼ 60 min) + `decisions/ADR-DET-001-streaming-vs-batch-substrate-split.md` (∼ 30 min) + skim docs/decisions/ADR-0307 + ADR-0308 + ADR-0309 + ADR-0310 (∼ 90 min).
2. Open the Grafana folder `detection`. Identify the boards: `detection-streaming-latency`, `detection-mitigation-action-rate`, `detection-model-drift`, `detection-fairness-audit-status`, `detection-investigation-case-flow`, `detection-appeal-throughput`.
3. Walk the runbook index. The on-call runbooks: `flink-checkpoint-stalled.md`, `model-drift-alert.md`, `fairness-threshold-breached.md`, `investigation-case-overflow.md`, `appeal-tied-vote-tiebreak.md`, `cedar-rule-runaway.md`, `feature-store-skew-detected.md`, `sar-candidate-pipeline-lag.md`.
4. Sit in on Wed's detection-on-call handoff. Watch how the outgoing rotation reviews the past-week mitigation-action rate + appeal-throughput + drift-alert resolution.

Acceptance: you can articulate the 8 detection families + the per-family mitigation path (block / step-up auth / human-review / log-only).

## Day 2 — Deploy a streaming detection job

```sh
cargo run -p oya-dev-cli -- detection deploy \
    --family payment-fraud \
    --cell drill-syd-1 \
    --shape streaming \
    --rule-set rules/payment-fraud-baseline.cedar \
    --model-card model-cards/payment-fraud-v0.json
```

The deploy controller:

1. Validates the rule set against the Cedar schema (typechecking).
2. Validates the model card against ADR-0308 schema (model_card_id, training_data_provenance, fairness_audit_signoff_id, drift_baseline_distribution_id).
3. Submits the Flink job with the rule + model artifacts.
4. Provisions Pulsar consumer slots for the source topic (`payments.transaction.v1`).
5. Emits `detection_job_deployed` audit event.

Verify the job:

```sh
oya detection job status --family payment-fraud --cell drill-syd-1
```

Should show `RUNNING` within ~ 60 s. The first checkpoint completes within ~ 2 min (Flink incremental checkpointing).

Emit synthetic events:

```sh
oya synthetic emit \
    --tenant drill-acme \
    --topic payments.transaction.v1 \
    --shape risk-mixed \
    --rate 50/sec \
    --duration 5m
```

Watch the streaming-latency dashboard. p99 should be ≤ 200 ms throughout the burst.

Acceptance: job deployed, synthetic events scored, you can describe the back-pressure path (Pulsar → Flink source → state-backend write → ONNX inference → rule eval → mitigation emit).

## Day 3 — Author a Cedar rule + extension construct

Read `decisions/ADR-DET-001-streaming-vs-batch-substrate-split.md` § "Why Cedar rules". The extension constructs are `rule-of-N (count, window, predicate)` and `temporal-aggregate (sum/avg, window, predicate)`.

Author a baseline payment-fraud rule:

```cedar
// rules/payment-fraud-velocity.cedar
permit (
    principal == User::"customer",
    action == Action::"payment::charge",
    resource is Payment
) when {
    // rule-of-N: if the customer has made >= 4 payments in the last 600 seconds, flag.
    !rule_of_n(
        count: 4,
        window_seconds: 600,
        predicate: { event.action == "payment::charge" && event.principal == principal }
    )
    // temporal-aggregate: AND the sum of those payments is >= $5000
    && !temporal_aggregate_gte(
        agg: "sum",
        field: "event.amount_cents",
        threshold: 500000,
        window_seconds: 600,
        predicate: { event.action == "payment::charge" && event.principal == principal }
    )
};
```

Validate the rule:

```sh
oya detection rule validate \
    --rule rules/payment-fraud-velocity.cedar \
    --schema schemas/detection.cedarschema
```

The validator checks: typechecking (resource shapes match), bounded state (the `window_seconds` × `count` product is bounded), no recursion. Expect `VALID` on a well-formed rule.

Deploy the rule to the running job (zero-downtime config update):

```sh
oya detection rule deploy \
    --family payment-fraud \
    --cell drill-syd-1 \
    --rule rules/payment-fraud-velocity.cedar \
    --activation immediate
```

Emit a velocity-burst test:

```sh
oya synthetic emit-velocity \
    --tenant drill-acme \
    --user drill-customer-z \
    --shape 5-payments-in-90-seconds-each-2000-dollars
```

Within ~ 200 ms of the 4th synthetic payment, you should see a `mitigation_action_step_up_auth` event in the audit chain.

Acceptance: rule deployed, velocity-burst test fires the mitigation, you can articulate why `rule-of-N` is statically bounded (the state size is `count × window_seconds × avg_event_size`, all known at deploy-time).

## Day 4 — Model drift drill + fairness audit walkthrough

Read `runbooks/model-drift-alert.md` and skim `decisions/ADR-0308`.

Run the drift drill:

```sh
oya detection drill model-drift \
    --family payment-fraud \
    --cell drill-syd-1 \
    --shape feature-distribution-shift-cardholder-country
```

The drill emits synthetic events with a feature-distribution shift (cardholder-country mix changes from baseline 80/15/5 US/EU/other to 60/30/10). After ~ 5 min, the `detection-model-drift` Grafana panel should fire an alert.

Walk the resolution:

1. Confirm the drift is data-shift (not adversarial). The runbook enumerates the diagnostic queries.
2. Decide: temporary auto-retrain trigger OR human-review hold? For this drill, the answer is `human-review hold` because the shift is small enough that auto-retrain risks over-fitting on a noisy window.
3. Emit `model_drift_acknowledged` audit event.

Now walk the fairness-audit pipeline:

```sh
oya detection fairness-audit run \
    --family payment-fraud \
    --cell drill-syd-1 \
    --shape weekly-eu-ai-act-annex-iii-section-1
```

The audit pipeline (per IP-019):

1. Sample N decisions from the last 7 days.
2. Stratify by protected class (race, ethnicity, gender, age-bracket, marital-status — pack-specific).
3. Compute disparate-impact ratio (4/5ths rule per EEOC UGESP § 1607.4(D)).
4. Bootstrap confidence interval (10 000 samples, 95 % CI).
5. Emit pass/fail verdict + evidence bundle.

Expected output:

```
Audit complete.
  Sample size: 12 478 decisions
  Protected class: race
  Reference group: White (denial rate 4.2 %)
  Comparison: Asian (denial rate 3.8 %)    ratio 0.91    PASS (≥ 0.80 threshold)
  Comparison: Black (denial rate 4.7 %)    ratio 1.12    PASS (≤ 1.25 threshold; reverse-test)
  Comparison: Hispanic (denial rate 4.1 %) ratio 0.98    PASS
  Verdict: PASS for race; see fairness-audit-evidence-2026-05-20.json
```

Acceptance: drift drill fired + resolved; fairness-audit run; you can articulate the difference between "disparate-impact ratio" (4/5ths rule) and "disparate-treatment" (intent-based; not auditable from data alone).

## Day 5 — Investigation case adjudication shadow

Read `decisions/ADR-0310` + skim `IP-013-investigation-bridge-usecase.md` + `IP-014-investigation-rest.md`.

Shadow a senior detection engineer adjudicating 3-5 real cases from the past week (with PII redacted via `tools/detection-shadow.sh`). For each case:

1. Read the evidence bundle: features at time of decision, model card, rule output, prior cases by the same principal.
2. Watch the senior engineer's diagnostic path: confirm the score, drill into the features, decide (`uphold`, `reverse`, `escalate`).
3. Watch the appeal-workflow if the case is an appeal: independent reviewer, defense-of-process invariant (original adjudicator excluded), tied-vote escalation.

Now walk a synthetic adjudication end-to-end:

```sh
oya detection drill adjudicate \
    --cell drill-syd-1 \
    --case-id drill-case-acct-takeover-456 \
    --as-role senior-adjudicator
```

The drill loads a synthetic account-takeover case. Walk the evidence: login-attempt features, device-fingerprint trajectory, IP-geolocation history. Decide.

Acceptance: you can articulate the appeal-workflow's defense-of-process invariant + the audit-chain shape for an adjudication (decision-event + evidence-bundle pin + appeal-deadline-set event).

## What you've learned

- The Flink streaming + Spark batch substrate split.
- Cedar rule authoring with `rule-of-N` + `temporal-aggregate` extensions.
- Model-drift detection + the auto-retrain-vs-human-review decision.
- Fairness-audit + 4/5ths rule + bootstrap CI.
- Investigation case adjudication + appeal-workflow defense-of-process.

Next week: graph-traversal community-detection shadow, SAR-candidate pipeline walk, EU AI Act Annex III conformity assessment shadow.
