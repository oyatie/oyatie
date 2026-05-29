---
doc_class: Tutorial
microservice: detection
persona: detection-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a payment-fraud Cedar rule with `rule-of-N` + `temporal-aggregate` extensions

You will: author a 3-rule Cedar policy that detects card-testing velocity bursts, deploy it to a running Flink streaming job, emit a synthetic attack, watch the mitigation fire, and audit the verdict. Total time ≤ 60 minutes.

## Pre-requisites

- A paid tenant_class detection cell (`capability-tiers/tier-matrix.md`).
- `oya-dev-cli` ≥ 1.42.0.
- Cedar CLI 4.4+ installed (`brew install cedar` or `cargo install cedar-cli`).
- A test tenant `drill-acme` with the `payments.transaction.v1` topic provisioned.

## Step 1 — Read the schema (≤ 5 min)

The Cedar schema defines what entity types + actions exist. Open `schemas/detection.cedarschema`:

```cedar
namespace Detection {
    entity User in [Tenant] {
        id: String,
        country_iso: String,
        account_age_days: Long,
        is_minor: Bool
    };

    entity Payment {
        id: String,
        amount_cents: Long,
        currency: String,
        card_bin: String,
        card_country_iso: String,
        merchant_country_iso: String,
        principal_country_iso: String,
        is_card_not_present: Bool
    };

    entity Tenant {
        id: String,
        pack_id: String
    };

    action "payment::charge" appliesTo {
        principal: [User],
        resource: [Payment],
        context: {
            event_time: Long,
            ip_address: String,
            device_fingerprint: String
        }
    };
}
```

## Step 2 — Author the rule (≤ 15 min)

Create `rules/payment-fraud-card-testing.cedar`:

```cedar
// Rule 1: Block charges from a customer who has made >= 4 attempts of <= $10 each
// in the last 600 seconds — classic card-testing fingerprint.
forbid (
    principal is Detection::User,
    action == Detection::Action::"payment::charge",
    resource is Detection::Payment
) when {
    rule_of_n(
        count: 4,
        window_seconds: 600,
        predicate: {
            event.action == "payment::charge"
            && event.principal == principal
            && event.resource.amount_cents <= 1000
        }
    )
};

// Rule 2: Block charges where the principal's country and the card's country diverge
// AND the resource is card-not-present AND the customer's account is < 30 days old.
forbid (
    principal is Detection::User,
    action == Detection::Action::"payment::charge",
    resource is Detection::Payment
) when {
    resource.is_card_not_present
    && resource.card_country_iso != context.principal_country_iso
    && principal.account_age_days < 30
};

// Rule 3: Block charges totaling >= $5000 in the last 600s for accounts < 7 days old.
forbid (
    principal is Detection::User,
    action == Detection::Action::"payment::charge",
    resource is Detection::Payment
) when {
    principal.account_age_days < 7
    && temporal_aggregate_gte(
        agg: "sum",
        field: "event.resource.amount_cents",
        threshold: 500000,    // 500_000 cents = $5000
        window_seconds: 600,
        predicate: {
            event.action == "payment::charge"
            && event.principal == principal
        }
    )
};
```

Notes:

- `rule_of_n` and `temporal_aggregate_gte` are extension constructs per ADR-DET-001 — they compile to bounded-state Flink operators. The Cedar validator proves the state bound.
- The three rules are independent; any one of them `forbid`-ing the action is sufficient to fire the mitigation. (Cedar default-deny — without a `permit` the action is denied; here we add `forbid`-with-condition to make the rationale auditable per ADR-0263.)
- The `event.action == "payment::charge"` predicate inside `rule_of_n` matches against the event stream (not the rule's current context); this is the streaming-aggregate primitive.

## Step 3 — Validate the rule (≤ 5 min)

```sh
oya detection rule validate \
    --rule rules/payment-fraud-card-testing.cedar \
    --schema schemas/detection.cedarschema
```

The validator runs:

1. **Typechecker** — confirms entity types match the schema; predicates reference valid fields.
2. **State-boundedness** — proves the `window_seconds × count` and `window_seconds` × cardinality(predicate) products are bounded.
3. **Termination** — confirms no recursion + no unbounded loops.
4. **Drift-resistance** — flags any predicate that references unstable fields (e.g., `event.fee_rate` if `fee_rate` is a model output rather than a raw event field).

Expected output:

```
Rule rules/payment-fraud-card-testing.cedar
  Typecheck: PASS
  State bound: 25 GiB max (worst-case, all 3 rules, 24h projected)
  Termination: PASS
  Drift-resistance: PASS
  Verdict: VALID
```

If the verdict is not `VALID`, fix the issue (the error message names the violating clause + schema-line) and re-validate.

## Step 4 — Deploy the rule to the running job (≤ 5 min)

The detection job is already running with a baseline rule set. We're hot-swapping with a richer set:

```sh
oya detection rule deploy \
    --family payment-fraud \
    --cell drill-syd-1 \
    --rule rules/payment-fraud-card-testing.cedar \
    --activation immediate \
    --shadow false
```

The deploy controller:

1. Re-runs validation (paranoid; the rule may have been edited).
2. Compiles the Cedar AST to Flink streaming operators.
3. Calls Cedar `detection::rule::deploy` itself — meta-gate; can the principal deploy this rule? (The principal must be in the `detection-engineer` role.)
4. Submits the operator graph to Flink with `--update-mode=hot-swap`. Flink saves the in-flight state + applies the new operator graph + restores state in < 30 s.
5. Emits `detection_rule_deployed` audit event.

Verify:

```sh
oya detection rule status --family payment-fraud --cell drill-syd-1
```

Expected:

```
Family: payment-fraud
Cell: drill-syd-1
Active rules: 4 (3 forbid + 1 permit base)
Rule set hash: sha256:c1d2e3f4...
Deploy event: detection_rule_deployed 2026-05-20T13:42:00Z
State size: 8.4 GiB (8.7% of bound)
```

## Step 5 — Emit a synthetic card-testing attack (≤ 5 min)

```sh
oya synthetic emit-attack \
    --tenant drill-acme \
    --shape card-testing \
    --attacker-user-id drill-attacker-w \
    --burst-pattern "5-charges-of-3-dollars-in-90-seconds"
```

The synthetic harness emits 5 `payment::charge` events for user `drill-attacker-w`, each $3.00, spaced ~ 18 s apart.

Watch the mitigation fire:

```sh
oya audit query --tenant drill-acme --since 2m --principal drill-attacker-w --event-class detection_*
```

Expected events:

- `detection_score_emitted` × 5 (one per event; the first 3 score below threshold; the 4th + 5th score above + fire the mitigation).
- `mitigation_action_emitted` × 2 (one for the 4th event with `action=block, rule=Rule 1`; one for the 5th with `action=block, rule=Rule 1`).
- `cedar_decision_audit` × 5 (the decision attribution for each event).

## Step 6 — Inspect the SHAP-class explanation (≤ 5 min)

```sh
oya detection decision explain \
    --tenant drill-acme \
    --decision-id <decision_id_from_audit>
```

Expected output (for the 4th event):

```
Decision: block
Confidence: 1.000 (deterministic; rule-based, not model-based)
Rule attribution: Rule 1 (rule_of_n: 4 charges <= $10 in 600s)
Cedar policy ID: payment-fraud-card-testing
Cedar policy hash: sha256:c1d2e3f4...
Counter-factuals:
  - If the 4th event was $50 instead of $3: rule 1 would NOT fire (predicate `amount_cents <= 1000` false).
  - If the window had been 10s longer: rule 1 would still fire (the 4-count threshold was met within 600s).
Audit-chain event: detection_decision_audit_001
```

The counter-factuals are auto-generated from the rule structure; they're useful for the appeal-workflow.

## Step 7 — Walk an appeal (≤ 10 min)

The attacker `drill-attacker-w` was malicious; the synthetic harness was attacking us legitimately. But let's pretend a real customer filed an appeal:

```sh
oya detection appeal file \
    --tenant drill-acme \
    --user drill-attacker-w \
    --decision-id <decision_id_of_4th_event> \
    --reason "I was buying 5 birthday gifts for my friend's kids — small denominations because they're for $1-store treats. Please review."
```

The appeal:

1. Cedar gate `detection::appeal::file` evaluates. Allow.
2. Case opened in the investigation case-management subsystem (per ADR-0310 + IP-013).
3. Independent reviewer assigned (defense-of-process invariant: the original adjudicator excluded).
4. Reviewer has 14 d to decide (EU DSA Art. 20 + ECOA Reg B notice requirements).

Walk the reviewer's diagnostic path:

```sh
oya detection case view --case-id <case_id> --as-role reviewer
```

The view shows:

- The 5 events that triggered the rule.
- The Cedar decision audit per event.
- The SHAP-class explanation + counter-factuals.
- The principal's prior 30-day activity (with PII redacted at the streaming layer per `tools/detection-shadow.sh`).
- The protected-class composition of similar past appeals + their disposition (for bias-pattern review).

The reviewer makes a decision; the audit chain emits.

## Step 8 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --since 30m --event-class detection_*,mitigation_*,cedar_*
oya audit verify-chain --tenant drill-acme --since 30m
```

Expected: ~ 15-20 audit events; chain verified; no signature gaps.

## What you've learned

- The Cedar schema + rule-authoring shape.
- The `rule_of_n` + `temporal_aggregate_gte` extension constructs + their state-boundedness proof.
- The validate → deploy → hot-swap pipeline.
- The Cedar decision audit + SHAP-class counter-factual explanation.
- The appeal-workflow defense-of-process invariant.

Next tutorial: `tutorials/train-and-promote-onnx-model.md` — author a payment-fraud ONNX model, generate the model card, run the fairness audit, promote to production.
