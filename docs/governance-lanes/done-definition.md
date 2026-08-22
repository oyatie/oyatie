---
doc_status: published
---

# Fitness Lane: done-definition

- status: Accepted
- date: 2026-05-12
- purpose: Verify every PR proves all D-items (D1..D15) are checked, marked N/A with rationale, or deferred to an issue id.
- enforces: CHECKLIST/definition-of-done.
- kernel_crate: `governance-done-definition-kernel` — `DoneDefinition { pr_id, items }`, `DItem { id, state, deferred_issue }`, verdict `DoneDefinitionFitnessReport { pr_count }`.
- runner_path: `tools/governance-done-definition`
- inputs: PR body, definition-of-done schema (D1..D15).
- failure_modes:
  - D-item left unchecked
  - N/A row without rationale
  - deferred row without issue id
- ci_invocation: `cargo run -p governance-done-definition`
- runtime_budget: 250 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct DItem {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub state: String,                 // data_class: INTERNAL_ONLY
    pub rationale: Option<String>,     // data_class: INTERNAL_ONLY
    pub deferred_issue: Option<String>,// data_class: INTERNAL_ONLY
}
pub struct DoneDefinition {
    pub pr_id: String,         // data_class: INTERNAL_ONLY
    pub items: Vec<DItem>,     // data_class: INTERNAL_ONLY
}
pub struct DoneDefinitionFitnessReport { pub pr_count: usize }

pub enum DoneDefinitionFitnessError {
    UncheckedItem { pr_id: String, item: String },
    NaWithoutRationale { pr_id: String, item: String },
    DeferredWithoutIssue { pr_id: String, item: String },
    MissingDItem { pr_id: String, item: String },
}

pub fn validate_done_definition_fitness(
    defs: &[DoneDefinition],
    required: &[String],
) -> Result<DoneDefinitionFitnessReport, DoneDefinitionFitnessError> {
    for d in defs {
        let by_id: std::collections::BTreeMap<&str, &DItem> = d.items.iter().map(|i| (i.id.as_str(), i)).collect();
        for r in required {
            let it = by_id.get(r.as_str()).ok_or_else(|| DoneDefinitionFitnessError::MissingDItem {
                pr_id: d.pr_id.clone(), item: r.clone(),
            })?;
            match it.state.as_str() {
                "checked" => {}
                "n/a" => if it.rationale.is_none() {
                    return Err(DoneDefinitionFitnessError::NaWithoutRationale { pr_id: d.pr_id.clone(), item: r.clone() });
                },
                "deferred" => if it.deferred_issue.is_none() {
                    return Err(DoneDefinitionFitnessError::DeferredWithoutIssue { pr_id: d.pr_id.clone(), item: r.clone() });
                },
                _ => return Err(DoneDefinitionFitnessError::UncheckedItem { pr_id: d.pr_id.clone(), item: r.clone() }),
            }
        }
    }
    Ok(DoneDefinitionFitnessReport { pr_count: defs.len() })
}
```
