---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-003-conditional-logic-engine-cel
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test, oya-forms-conditional-logic-parity, oya-forms-branching-static-analysis, oya-forms-skip-logic-pii-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: Conditional-logic engine (CEL declarative DAG)

## Intent

Implement the conditional-logic + branching engine per ADR-FORMS-0004. CEL evaluation server-side via `cel-rust`; client-side via `cel-js` (Leptos WASM bundle includes the JS evaluator). Static-analysis lane validates DAG acyclicity, reachability, type-soundness, predicate cost cap, and data-class-flow safety.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/domain/branching/predicate.rs` | create |
| `microservices/forms/src/domain/branching/dag.rs` | create |
| `microservices/forms/src/domain/branching/static_analysis.rs` | create |
| `microservices/forms/src/domain/branching/runtime.rs` | create — `cel-rust` integration |
| `microservices/forms/src/domain/branching/parity_test_corpus.rs` | create |
| `microservices/forms/tests/branching_parity.rs` | create — 1000-case corpus |
| `microservices/forms/tests/branching_static.rs` | create |

## Code Shape

```rust
pub struct BranchPredicate {
    pub when_cel: CelExpression,    // parsed at load
    pub show_field_ids: Vec<FieldId>,
    pub hide_field_ids: Vec<FieldId>,
    pub next_page_id: Option<PageId>,
}

pub fn evaluate_branching(
    spec: &FormSpecV1,
    submission: &FieldValues,
) -> Result<Visibility, BranchingError> { /* … */ }
```

## Acceptance Gates

- `oya-forms-conditional-logic-parity` 1000-case corpus identical server + client.
- `oya-forms-branching-static-analysis` rejects: cyclic DAG, unreachable page, type mismatch, over-cap predicate count.
- `oya-forms-skip-logic-pii-correctness`: hidden-field with `data_class=PII_*` is NOT persisted in submission.

## References

- ADR-FORMS-0004 CEL branching.
- `cel-rust` + `cel-js` + Google CEL spec.
- PRD FR-02 / FR-12 and AC-03 / AC-15.
- `microservices/forms/contracts/openapi/forms.openapi.yaml` branching and form-definition paths.
- `microservices/forms/contracts/proto/forms.proto` internal branching projection.
- `microservices/forms/slos/form-render-latency.openslo.yaml` and `field-validate-latency.openslo.yaml`.
- `microservices/forms/runbooks/pii-leak-incident-p0.md` for hidden-field persistence failure handling.
- `microservices/forms/decisions/ADR-FORMS-0004-conditional-logic-and-branching-engine.md`.

## Foundation A-G Substance

- A. Product scope: branch predicates are part of form definition authoring, not an analytics-only convenience.
- B. Domain model: `BranchPredicate`, `BranchDag`, `BranchEdge`, `VisibilityState`, and `BranchDecisionTrace` are clean-domain types.
- C. Contracts: REST and proto surfaces must expose predicate diagnostics without leaking submitter field values.
- D. Policy: branch evaluation inherits tenant scope and data-class rules from `policy/tenant-scope.cedar` and `policy/data-residency.md`.
- E. Operations: failed predicate compile blocks publish; failed runtime evaluation blocks submit with a typed diagnostic and audit event.
- F. Observability: emit branch-evaluation latency, skipped-PII-field count, and static-analysis rejection counters.
- G. Promotion: publish only after parity corpus, static-analysis gate, hidden-PII non-persistence, and renderer smoke tests are green.

## Counterpart Benchmark

- Counterpart: Typeform Logic Jumps, HubSpot Forms dependent fields, and Salesforce Web-to-Lead assignment-style routing.
- Defensible parity claim: Oyatie must support show/hide, skip-to-page, and server-authoritative validation rather than client-only branching.
- Differentiator: hidden PII values are discarded before persistence, closing the gap left by common web-form builders.
- Grep counterpart names: HubSpot Forms; Salesforce Web-to-Lead; Typeform Logic Jumps.

## Remediation Notes

- Expanded this IP from a compact implementation stub into a foundation plan tied to PRD, ADR, contracts, SLOs, policy, and runbook evidence.
- Added A-G delivery substance so a reviewer can trace product, domain, contract, policy, operations, observability, and promotion boundaries.
- Added grep-recognized counterpart names to make competitive parity review mechanically discoverable.

## Next IP

[`IP-004-validation-engine.md`](IP-004-validation-engine.md)
