---
doc_status: published
---

# Fitness Lane: pre-flight-checklist

- status: Accepted
- date: 2026-05-12
- purpose: Verify every PR includes a pre-flight checklist with every item marked checked or N/A (no unchecked rows).
- enforces: CHECKLIST/pre-flight.
- kernel_crate: `governance-pre-flight-checklist-kernel` — `PreFlightChecklist { pr_id, items }`, `ChecklistItem { id, state }`, verdict `PreFlightChecklistFitnessReport { pr_count }`.
- runner_path: `tools/governance-pre-flight-checklist`
- inputs: PR body, canonical pre-flight checklist template.
- failure_modes:
  - any row left unchecked
  - row state value invalid (not in {checked, n/a, blocked})
  - missing required row
- ci_invocation: `cargo run -p governance-pre-flight-checklist`
- runtime_budget: 200 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct ChecklistItem {
    pub id: String,     // data_class: INTERNAL_ONLY
    pub state: String,  // data_class: INTERNAL_ONLY
}
pub struct PreFlightChecklist {
    pub pr_id: String,             // data_class: INTERNAL_ONLY
    pub items: Vec<ChecklistItem>, // data_class: INTERNAL_ONLY
}
pub struct PreFlightChecklistFitnessReport { pub pr_count: usize }

pub enum PreFlightChecklistFitnessError {
    Unchecked { pr_id: String, item: String },
    InvalidState { pr_id: String, item: String, state: String },
    MissingRequired { pr_id: String, item: String },
}

pub fn validate_pre_flight_checklist_fitness(
    checklists: &[PreFlightChecklist],
    required_items: &[String],
) -> Result<PreFlightChecklistFitnessReport, PreFlightChecklistFitnessError> {
    let valid = ["checked", "n/a", "blocked"];
    for cl in checklists {
        let ids: std::collections::BTreeSet<&str> = cl.items.iter().map(|i| i.id.as_str()).collect();
        for req in required_items {
            if !ids.contains(req.as_str()) {
                return Err(PreFlightChecklistFitnessError::MissingRequired { pr_id: cl.pr_id.clone(), item: req.clone() });
            }
        }
        for i in &cl.items {
            if !valid.contains(&i.state.as_str()) {
                return Err(PreFlightChecklistFitnessError::InvalidState {
                    pr_id: cl.pr_id.clone(), item: i.id.clone(), state: i.state.clone(),
                });
            }
            if i.state == "blocked" {
                return Err(PreFlightChecklistFitnessError::Unchecked { pr_id: cl.pr_id.clone(), item: i.id.clone() });
            }
        }
    }
    Ok(PreFlightChecklistFitnessReport { pr_count: checklists.len() })
}
```
