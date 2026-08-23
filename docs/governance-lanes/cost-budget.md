---
doc_status: published
---

# Fitness Lane: cost-budget

- status: Accepted
- date: 2026-05-12
- purpose: Verify projected per-cell unit-cost stays within the declared cost budget (FinOps signal).
- enforces: STANDARD/cost-budget; existing crate `intelligence-cost-budget-kernel` (EXISTING).
- kernel_crate: `intelligence-cost-budget-kernel` (EXISTING) — `CostProjection { cell_id, unit_cost, currency }`, verdict `CostBudgetFitnessReport { cells_checked }`.
- runner_path: `tools/governance-cost-budget`
- inputs: cost-model output JSON, per-cell budget registry.
- failure_modes:
  - cell projection > budget
  - currency mismatch
  - missing projection for declared cell
- ci_invocation: `cargo run -p governance-cost-budget`
- runtime_budget: 600 ms
- severity: MED
- kernel_sketch:
```rust
pub struct CostProjection {
    pub cell_id: String,  // data_class: INTERNAL_ONLY
    pub unit_cost: f64,   // data_class: INTERNAL_ONLY
    pub currency: String, // data_class: INTERNAL_ONLY
}
pub struct CellBudget {
    pub cell_id: String,  // data_class: INTERNAL_ONLY
    pub max_cost: f64,    // data_class: INTERNAL_ONLY
    pub currency: String, // data_class: INTERNAL_ONLY
}
pub struct CostBudgetFitnessReport { pub cells_checked: usize }

pub enum CostBudgetFitnessError {
    OverBudget { cell_id: String, unit_cost: f64, max_cost: f64 },
    CurrencyMismatch { cell_id: String, projected: String, expected: String },
    MissingProjection { cell_id: String },
}

pub fn validate_cost_budget_fitness(
    projections: &[CostProjection],
    budgets: &[CellBudget],
) -> Result<CostBudgetFitnessReport, CostBudgetFitnessError> {
    let by_cell: std::collections::BTreeMap<&str, &CostProjection> =
        projections.iter().map(|p| (p.cell_id.as_str(), p)).collect();
    for b in budgets {
        let p = by_cell.get(b.cell_id.as_str()).ok_or_else(|| CostBudgetFitnessError::MissingProjection {
            cell_id: b.cell_id.clone(),
        })?;
        if p.currency != b.currency {
            return Err(CostBudgetFitnessError::CurrencyMismatch {
                cell_id: b.cell_id.clone(), projected: p.currency.clone(), expected: b.currency.clone(),
            });
        }
        if p.unit_cost > b.max_cost {
            return Err(CostBudgetFitnessError::OverBudget {
                cell_id: b.cell_id.clone(), unit_cost: p.unit_cost, max_cost: b.max_cost,
            });
        }
    }
    Ok(CostBudgetFitnessReport { cells_checked: budgets.len() })
}
```
