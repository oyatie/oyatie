---
doc_status: published
---

# Fitness Lane: raci-completeness

- status: Accepted
- date: 2026-05-12
- purpose: Verify every doc-class in the catalog has an owner row in `docs/RACI-OWNERSHIP.md`.
- enforces: STANDARD/raci-ownership.
- kernel_crate: `governance-raci-completeness-kernel` — `RaciRow { doc_class, responsible, accountable }`, verdict `RaciCompletenessFitnessReport { rows_checked }`.
- runner_path: `tools/governance-raci-completeness`
- inputs: `docs/RACI-OWNERSHIP.md`, doc-class registry.
- failure_modes:
  - doc-class with no RACI row
  - row missing accountable owner
  - duplicated row for same doc-class
- ci_invocation: `cargo run -p governance-raci-completeness`
- runtime_budget: 200 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct RaciRow {
    pub doc_class: String,    // data_class: INTERNAL_ONLY
    pub responsible: String,  // data_class: INTERNAL_ONLY
    pub accountable: String,  // data_class: INTERNAL_ONLY
}
pub struct RaciCompletenessFitnessReport { pub rows_checked: usize }

pub enum RaciCompletenessFitnessError {
    MissingClass { doc_class: String },
    MissingAccountable { doc_class: String },
    DuplicateRow { doc_class: String },
}

pub fn validate_raci_completeness_fitness(
    rows: &[RaciRow],
    doc_classes: &[String],
) -> Result<RaciCompletenessFitnessReport, RaciCompletenessFitnessError> {
    let mut seen = std::collections::BTreeSet::new();
    let by_class: std::collections::BTreeMap<&str, &RaciRow> = rows.iter().map(|r| (r.doc_class.as_str(), r)).collect();
    for r in rows {
        if !seen.insert(r.doc_class.clone()) {
            return Err(RaciCompletenessFitnessError::DuplicateRow { doc_class: r.doc_class.clone() });
        }
        if r.accountable.trim().is_empty() {
            return Err(RaciCompletenessFitnessError::MissingAccountable { doc_class: r.doc_class.clone() });
        }
    }
    for c in doc_classes {
        if !by_class.contains_key(c.as_str()) {
            return Err(RaciCompletenessFitnessError::MissingClass { doc_class: c.clone() });
        }
    }
    Ok(RaciCompletenessFitnessReport { rows_checked: rows.len() })
}
```
