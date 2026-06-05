---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-014-ai-assist-bounds-and-eu-ai-act
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks + council-privacy + council-ai-safety
acceptance_lanes: [cargo-test, auto-assign-fairness, regulated-ai-refusal-grounding, cedar-policy-syntax]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: AI assist bounds — T0 next-task / T1 categorise + priority / T2 auto-assign with EU AI Act Annex III §4 refusal

## Intent

Wire the three autonomy-tier capabilities declared in
`capabilities/T0-suggest.yaml`, `T1-categorise.yaml`, `T2-auto.yaml`
to the task-store usecase. T0 next-task suggest is read-only;
T1 auto-categorise + priority suggestion presents a recommendation
that requires human acceptance; T2 auto-assign DIRECTLY mutates
assignment.

Per ADR-TASKS-0006: T2 auto-assign in **employment context** (the
default for any tenant whose Cedar `employment_context` claim is
true — KR 근로기준법 §17 + EEOC UGESP 1978 + UK Equality Act 2010 +
EU AI Act Annex III §4) is REFUSED at the Cedar policy layer until
the tenant has uploaded a per-pack conformity-assessment artefact.
PRD AC-12 codifies this refusal as a release gate.

The auto-assign-fairness lane runs synthetic demographic balance
checks against the assignment trace; cross-cohort assignment-rate
deltas beyond a configurable threshold trigger the fairness alert
wired in IP-001 PrometheusRule.

## ChangeSet boundary

`task-store-usecase` + `task-store-domain` + Cedar policies under
`microservices/tasks/policy/` + capability YAMLs (already authored).
Cross-link to `foundry-runtime` via Workflow events (the T1/T2
classifier itself runs in `foundry-runtime` per Bominal ADR-0166;
this µservice consumes results).

## Crate Naming

n/a — modifies existing task-store crates + Cedar policies.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-task-store-usecase/src/auto_assign.rs` | created | T2 orchestrator |
| `microservices/tasks/src/oya-tasks-task-store-domain/src/eu_ai_act_guard.rs` | created | refusal logic |
| `microservices/tasks/policy/auto-assign-employment-context.cedar` | created | Cedar refusal |
| `microservices/tasks/tests/integration/eu_ai_act_auto_assign_refusal.rs` | created | AC-12 |

## Acceptance Gates

```bash
cargo test -p oya-tasks-task-store-domain eu_ai_act_auto_assign_refusal
cargo test -p oya-tasks-task-store-usecase auto_assign
buck2 build //:quality-lane-registry-authority-check # lane=auto-assign-fairness --microservice tasks
buck2 build //:quality-lane-registry-authority-check # lane=regulated-ai-refusal-grounding --microservice tasks
buck2 build //:quality-lane-registry-authority-check # lane=cedar-policy-syntax --microservice tasks
```

## Test Plan

- T2 auto-assign in employment context without conformity assessment
  → 403 Cedar refusal (AC-12).
- T2 auto-assign in employment context WITH conformity assessment +
  per-pack ADR sign-off → 200; emits `TaskAssigned` with
  `via_auto_assign=true` + `ai_capability_id` populated.
- Fairness lane: synthetic skew injection → alert fires; fairness
  metric exposed at `/metrics`.

## Halt Conditions

- T2 auto-assign in employment context succeeds without conformity
  artefact — refuse; this is a regulatory hard-stop.

## Next IP

[`IP-015-hg-tasks-conformance.md`](IP-015-hg-tasks-conformance.md)

## References

- ADR-TASKS-0006 (auto-assign + EU AI Act Annex III §4).
- EU AI Act Annex III §4 — employment + worker management high-risk.
- EEOC UGESP 1978 — 4/5ths rule for adverse impact.
- KR 근로기준법 §17 — equal-treatment.
- workflow-studio ADR-WS-0005 (sibling EEOC pattern).
- network ADR-NET-0002 (sibling EEOC pattern).
