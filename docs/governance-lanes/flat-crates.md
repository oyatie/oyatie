---
doc_status: published
---

# Fitness Lane: flat-crates

- status: Accepted
- date: 2026-05-12
- purpose: Verify workspace stays flat (no nested `crates/foo/sub/`) and every crate has the `oyatie-` prefix.
- enforces: STANDARD/flat-workspace; AGENTS.md fitness-lane `governance-flat-crates`.
- kernel_crate: `governance-flat-crates-kernel` — `CrateLocation { crate_id, manifest_path }`, verdict `FlatCratesFitnessReport { crates_checked }`.
- runner_path: `tools/governance-flat-crates`
- inputs: workspace `Cargo.toml` member list.
- failure_modes:
  - crate manifest path has depth > `crates/<name>/Cargo.toml`
  - crate id missing `oyatie-` prefix
  - crate id with `_` instead of `-`
- ci_invocation: `cargo run -p governance-flat-crates`
- runtime_budget: 100 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct CrateLocation {
    pub crate_id: String,      // data_class: INTERNAL_ONLY
    pub manifest_path: String, // data_class: INTERNAL_ONLY
}

pub struct FlatCratesFitnessReport {
    pub crates_checked: usize,
}

pub enum FlatCratesFitnessError {
    NestedCrate { crate_id: String, manifest_path: String },
    MissingOyaPrefix { crate_id: String },
    UnderscoreInId { crate_id: String },
}

pub fn validate_flat_crates_fitness(
    crates: &[CrateLocation],
) -> Result<FlatCratesFitnessReport, FlatCratesFitnessError> {
    for c in crates {
        let depth = c.manifest_path.split('/').count();
        if depth != 3 {
            return Err(FlatCratesFitnessError::NestedCrate {
                crate_id: c.crate_id.clone(),
                manifest_path: c.manifest_path.clone(),
            });
        }
        if !c.crate_id.starts_with("oyatie-") {
            return Err(FlatCratesFitnessError::MissingOyaPrefix { crate_id: c.crate_id.clone() });
        }
        if c.crate_id.contains('_') {
            return Err(FlatCratesFitnessError::UnderscoreInId { crate_id: c.crate_id.clone() });
        }
    }
    Ok(FlatCratesFitnessReport { crates_checked: crates.len() })
}
```
