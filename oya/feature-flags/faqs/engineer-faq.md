---
doc_class: FAQ
microservice: feature-flags
persona: engineer
date: 2026-05-20
doc_status: published
---

# Engineer FAQ — feature-flags

## Why is my flag returning the default variant even though the dashboard shows the rule is "on"?

Three common causes, in order of frequency. (1) Your SDK client was built before the rule was published; the in-process cache TTL is 60 s, so wait one minute and re-evaluate. (2) Your evaluation context is missing `pack_id`; the Cedar fragment guards on `pack_id == "us-default"` and missing-attribute evaluations short-circuit to the default per ADR-0145 §"deny-on-attribute-missing". (3) Your `(tenant_id, flag_key)` lands outside the percentage-rollout bucket. The bucket is deterministic — same tenant always lands in the same bucket — so flipping the percent from 5 % to 50 % may not include your test tenant. Use `oya ff explain --tenant <id> --flag <key>` to see the exact reasoning chain.

## Can I add a new evaluation-context attribute that's not in the schema?

No — context attributes are closed-enum per ADR-0263. The seven attributes are `tenant_id`, `persona_tier`, `pack_id`, `cohort_ids[]`, `user_id_hash`, `request_id`, `provider_credential_mode`. Adding an eighth requires a feature-flags ADR amendment, a `feature-flags-v2.proto` revision (additive only), and a 30-day sunset on the v1 evaluator. We've held this discipline since 2026-01 — open-enum context attributes were a major audit liability under SOC 2 §CC7.2 because they broke retroactive replay.

## How do I run an A/B experiment? feature-flags is OpenFeature-compliant — why doesn't it surface a Bayesian winner?

By policy. The PRD §Scope excludes statistics + winner-selection — that's the future `experiments` µservice. `feature-flags` only does the variant assignment + evaluation-event emission. Pipe the `flag_evaluation` audit-chain events to `analytics` and run your statistics there. Today the most common pattern: a Materialise view on `audit_chain.flag_evaluation` joined with `events.product_outcome` on `(tenant_id, user_id_hash, time_bucket)`, then a Wilson interval at α=0.05 in the analytics notebook.

## What's the difference between `kill_switch`, `release_toggle`, `experiment`, `permission_toggle`?

Lifecycle and sunset semantics:

- `kill_switch`: emergency-disable for an existing feature. No sunset — lives until the feature is amputated from the code. P0 mutation rights only.
- `release_toggle`: gates a new feature during rollout. Sunset within ≤ 90 days of GA (lane BLOCKER thereafter).
- `experiment`: A/B variant assignment. Sunset within ≤ 60 days of experiment readout.
- `permission_toggle`: gates a per-pack or per-persona-tier capability. May be long-lived (e.g., `pack.hipaa.surface_dx_codes` is permanent).

The lane `oya-governance-feature-flag-sunset` enforces sunset; misclassifying a `release_toggle` as a `permission_toggle` to escape the sunset clock is detected at PR review time and is a §3.2.1 Axis B finding.

## My on-call paged me at 3 AM to flip a kill-switch. What did they expect me to do?

Run `oya ff flip <flag-key> --variant off --tenant <id-or-all> --reason "<incident-id>"`. The command requires `--reason` and emits an Ed25519-signed audit-chain `kill_switch_flipped` event per ADR-0263. If the flag is at `paid compliance_pack` tier (PCI / HIPAA), the command additionally requires a second approver via `--approver <on-call-2-handle>`. Post-incident, the kill-switch stays off until the incident report APPROVEs flip-back; do not flip back during the post-mortem call without an explicit go from the incident commander.

## How do I test a flag locally without going through staging?

Two options. (1) Use the local evaluator side-car: `cargo run -p feature-flags-local -- --flags-file ./local-flags.yaml`. Your SDK points at `localhost:50500` and reads YAML-defined flags. Used in unit tests and `cargo nextest` runs. (2) Use the SDK's `with_provider(MockProvider::new())` for in-process mocks where you don't want the side-car overhead. Both paths are covered in `reference-implementations/rust-mock-provider.rs`.

## Are flag evaluations themselves audit-emitted?

Only when the flag is tagged `audit_required: true`. Default is false because audit emission per-evaluation costs ~ 800 µs additional latency and ~ 12 % of cluster CPU at our 12 000 RPS paid tenant_class budget. We mark `audit_required: true` only on compliance-class flags (`pack.*` flags governing pack toggles; cardholder-data flags; PHI-exposure flags). Definition-mutation events (create / edit / archive / delete) are always audit-emitted regardless of `audit_required`.

## My PR is failing the `oya-governance-feature-flag-cedar-fragment-shape` lane. What does it want?

Cedar fragments must follow the shape `permit(principal, action == Action::"evaluate-flag-<flag-key>", resource) when {...}`. Common violations: using `forbid` instead of `permit` (we are default-deny per ADR-0007; flags are additive permits over the deny floor); using `action in [...]` instead of `action ==` (single-action permits are the convention, eases predicate reading); resource clause missing (must be present even if vacuous — see ADR-0145 amendment). Run `oya gate validate feature-flag-cedar-fragment-shape --flag <key>` for a per-flag report.
