---
doc_status: published
---

# Fitness Lane: changelog-row

- status: Accepted
- date: 2026-05-12
- purpose: Verify every PR that touches a canonical doc/standard/template/checklist has a corresponding CHANGELOG row.
- enforces: STANDARD/changelog-discipline.
- kernel_crate: `governance-changelog-row-kernel` — `ChangelogRow { pr_id, touches_canonical, row_present, row_classification }`, verdict `ChangelogRowFitnessReport { prs_checked }`.
- runner_path: `tools/governance-changelog-row`
- inputs: PR diff list, `docs/CHANGELOG.md`, canonical-path list.
- failure_modes:
  - canonical doc touched, no CHANGELOG row
  - row exists but missing classification (added/changed/removed/deprecated)
  - row classification not in canonical set
- ci_invocation: `cargo run -p governance-changelog-row`
- runtime_budget: 300 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct ChangelogRow {
    pub pr_id: String,                  // data_class: INTERNAL_ONLY
    pub touches_canonical: bool,        // data_class: INTERNAL_ONLY
    pub row_present: bool,              // data_class: INTERNAL_ONLY
    pub row_classification: Option<String>, // data_class: INTERNAL_ONLY
}
pub struct ChangelogRowFitnessReport { pub prs_checked: usize }

pub enum ChangelogRowFitnessError {
    MissingRow { pr_id: String },
    MissingClassification { pr_id: String },
    UnknownClassification { pr_id: String, classification: String },
}

pub fn validate_changelog_row_fitness(
    rows: &[ChangelogRow],
    valid_classes: &[String],
) -> Result<ChangelogRowFitnessReport, ChangelogRowFitnessError> {
    let valid: std::collections::BTreeSet<&str> = valid_classes.iter().map(|s| s.as_str()).collect();
    for r in rows {
        if !r.touches_canonical { continue; }
        if !r.row_present {
            return Err(ChangelogRowFitnessError::MissingRow { pr_id: r.pr_id.clone() });
        }
        let c = r.row_classification.as_ref().ok_or_else(|| ChangelogRowFitnessError::MissingClassification {
            pr_id: r.pr_id.clone(),
        })?;
        if !valid.contains(c.as_str()) {
            return Err(ChangelogRowFitnessError::UnknownClassification {
                pr_id: r.pr_id.clone(), classification: c.clone(),
            });
        }
    }
    Ok(ChangelogRowFitnessReport { prs_checked: rows.len() })
}
```
