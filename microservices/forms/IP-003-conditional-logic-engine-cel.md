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

## Next IP

[`IP-004-validation-engine.md`](IP-004-validation-engine.md)
