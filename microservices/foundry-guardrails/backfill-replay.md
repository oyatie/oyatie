---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-guardrails
deciders: axis-foundry-guardrails, council-architecture, ops-sre-reliability
related_adrs: [ADR-0022, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/foundry-guardrails/PRD.md
  - microservices/foundry-guardrails/policy/guardrail-enforcement.md
  - microservices/foundry-guardrails/capacity-model.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (foundry-guardrails µservice)

## Purpose

Specify how foundry-guardrails handles:
1. **Backfill** — a new rule or classifier-model is deployed; can prior decisions be recomputed retroactively?
2. **Replay** — an existing decision needs re-computation (after a rule fix, classifier-model rollout, or post-incident analysis).

## Backfill

### Contract

When a new rule definition (or classifier model) lands in Postgres / object-storage:

1. `RuleStoreMutated` event emitted to AsyncAPI.
2. The rule / model is validated against Cedar v4 schema (rules) or Cosign signature (models) + golden-fixture test set.
3. Backfill computes the candidate-decision window:
   - Default = `min(rule.first_applicable_at, available_decision_history_in_foundry-evidence)`.
   - Per-pack retention applies (foundry-evidence owns the history; guardrails does not persist).
4. Backfill emits **`backfilled=true`** annotated decisions for the affected (invocation_id) tuples; these are NOT consumed by foundry-runtime (which only honours live decisions).
5. Decisions are stored in foundry-evidence under the original invocation's record, with a `backfilled_decision` sub-record + `backfilled_by_rule_id` + `backfill_at` timestamp.

### Constraints

- Backfill does NOT change live invocation behaviour. Live invocations already executed used the rule active at that time; backfill provides a counterfactual view.
- Backfill is cost-bounded per `capacity-model.md` formulae: `O(history_window × invocation_rate)` per re-evaluated rule.
- Per-tenant rate-limit: a tenant cannot trigger more than 1 backfill per rule per hour (anti-abuse).
- Classifier-model backfill is **mandatory** for shadow-mode rollouts (Sev-1 prevention; rollout LEAN refuses promote-to-enforce without ≥ 7d shadow-history backfill comparison).
- Rule-author backfill is **optional** for routine rule edits, **mandatory** for safety-bearing categories (toxicity / self-harm / sexual / violence / minors).

### Verification

- Integration test: deploy a new rule; verify backfill events for prior-7d invocations; verify foundry-evidence stores the counterfactual.
- Idempotency: re-running same backfill emits same decisions.

## Replay

### Contract

Replay recomputes a specific decision for a specific (invocation_id, decision_kind). Triggers:

- Tenant FP escalation: tenant marks block as false-positive; rule-author may replay with adjusted rule to verify.
- Sev-1 jailbreak success post-mortem: replay the failing invocation against improved classifier ensemble to verify retraining efficacy.
- Bug-fix in ensemble math: replay invalidates prior decisions; emits new ones.
- Post-incident analysis: replay against alternate rule overlays to test "would this rule have caught it?"

### Procedure

1. Operator invokes: `cargo run -p oya-foundry-guardrails-prompt-classifier-app -- replay --invocation-id <id> --decision-kind <prompt|output> --reason "<rfc>"`.
2. CLI requires 2-person rule + ops-security approval for replays affecting > 100 invocations (large-scale historical replay).
3. Engine recomputes verdict against current rule + current classifier-model.
4. Emits `GuardrailDecisionEmitted` event with `replay_of_decision_id=<original>`, `prior_verdict=<original>`, `reason=<rfc>`.
5. Audit-chain seal: replay event sealed; original event remains sealed; chain is reconstructable.

### Constraints

- Replay does NOT mutate the original decision in foundry-evidence; appends new record with `replay=true`.
- Replay cannot exceed foundry-evidence retention (default 30d; up to 6y for HIPAA).
- Replay output never retro-actively impacts foundry-runtime invocation flow (no "we now declare yesterday's allow was actually a block, so let's pretend the invocation never happened").
- For Sev-1 jailbreak: replay is **mandatory** as part of post-mortem; verify the retrained classifier would now block.

### Verification

- Integration test: induce a synthetic decision; replay with same inputs; verify identical decision (determinism).
- Audit-chain integrity: replay event sealed; original event sealed; chain reconstructable.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill on new rule | per-rule-change | ~$0.10-$5.00 (per pack; bounded by invocation history × rule applicability) |
| Backfill on new classifier model (shadow phase) | per-model-rollout | ~$50-$500 (7d history × all invocations in pack) |
| Replay on bug-fix | per-engine-deploy | ~$50-$200 (full re-eval) |
| Replay on FP escalation | per-escalation | ~$0.001 (single invocation) |
| Replay on Sev-1 post-mortem | per-incident | ~$0.001-$0.10 (single invocation + adjacent perturbations) |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- Backfill quality is bounded by foundry-evidence retention.
- Replay assumes deterministic classifier inference; if classifier-model RNG-state changed between deploys, replay may produce subtly different scores. Replay output carries `classifier_model_version` to surface this.
- LLM-judge fallback replay is non-deterministic by default (LLM outputs vary); replay uses seed-pinned temperature-0 mode where supported by foundry-providers.

## References

- `microservices/foundry-guardrails/PRD.md`.
- `microservices/foundry-guardrails/policy/guardrail-enforcement.md`.
- `microservices/foundry-guardrails/capacity-model.md`.
- `microservices/foundry-evidence/` (decision history authority).
- `microservices/observability/backfill-replay.md` (sibling shape).
