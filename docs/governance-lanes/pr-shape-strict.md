---
doc_status: published
---

# Fitness Lane: pr-shape-strict

- status: Accepted
- date: 2026-05-12
- purpose: Verify every PR body has the 5 required H2 sections in canonical order (Summary, Evidence, Cross-Axis-Notify, Risk, Rollback).
- enforces: STANDARD/pr-shape; supersedes "D15-lite" laxity.
- kernel_crate: `governance-pr-shape-strict-kernel` — `PrBody { pr_id, h2_sections }`, verdict `PrShapeStrictFitnessReport { prs_checked }`.
- runner_path: `tools/governance-pr-shape-strict`
- inputs: PR body, canonical-section list.
- failure_modes:
  - "Rollback" section absent
  - sections present but order wrong
  - extra unknown H2 between required sections
- ci_invocation: `cargo run -p governance-pr-shape-strict`
- runtime_budget: 150 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct PrBody {
    pub pr_id: String,                // data_class: INTERNAL_ONLY
    pub h2_sections: Vec<String>,     // data_class: INTERNAL_ONLY
}
pub struct PrShapeStrictFitnessReport { pub prs_checked: usize }

pub enum PrShapeStrictFitnessError {
    MissingSection { pr_id: String, section: String },
    OutOfOrder { pr_id: String, expected: String, actual: String },
}

pub fn validate_pr_shape_strict_fitness(
    prs: &[PrBody],
    canonical_order: &[String],
) -> Result<PrShapeStrictFitnessReport, PrShapeStrictFitnessError> {
    for p in prs {
        let observed: Vec<&str> = p.h2_sections.iter()
            .filter(|s| canonical_order.iter().any(|c| c == *s))
            .map(|s| s.as_str()).collect();
        for (i, c) in canonical_order.iter().enumerate() {
            if i >= observed.len() {
                return Err(PrShapeStrictFitnessError::MissingSection { pr_id: p.pr_id.clone(), section: c.clone() });
            }
            if observed[i] != c.as_str() {
                return Err(PrShapeStrictFitnessError::OutOfOrder {
                    pr_id: p.pr_id.clone(), expected: c.clone(), actual: observed[i].to_string(),
                });
            }
        }
    }
    Ok(PrShapeStrictFitnessReport { prs_checked: prs.len() })
}
```
