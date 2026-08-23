---
doc_status: published
---

# Fitness Lane: semver-checks

- status: Accepted
- date: 2026-05-12
- purpose: Verify public APIs across `*-api` crates pass `cargo-semver-checks` against the last released minor.
- enforces: hyperscaler-best-practices spec — Rust `cargo-semver-checks` on public APIs.
- kernel_crate: `intelligence-api-semver-kernel` (EXISTING; extend with verdict) — `SemverDelta { crate_id, breaking, change_class }`, verdict `SemverChecksFitnessReport { crates_checked }`.
- runner_path: `tools/governance-semver-checks`
- inputs: `cargo semver-checks` JSON report, version-bump policy.
- failure_modes:
  - breaking change without major bump
  - removed item without deprecation cycle
  - added required trait method
- ci_invocation: `cargo run -p governance-semver-checks`
- runtime_budget: 2500 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct SemverDelta {
    pub crate_id: String,         // data_class: INTERNAL_ONLY
    pub current_version: String,  // data_class: INTERNAL_ONLY
    pub baseline_version: String, // data_class: INTERNAL_ONLY
    pub breaking: bool,           // data_class: INTERNAL_ONLY
    pub change_class: String,     // data_class: INTERNAL_ONLY (major|minor|patch)
}
pub struct SemverChecksFitnessReport { pub crates_checked: usize }

pub enum SemverChecksFitnessError {
    BreakingWithoutMajor { crate_id: String, current: String, baseline: String },
    InvalidChangeClass { crate_id: String, class: String },
}

pub fn validate_semver_checks_fitness(
    deltas: &[SemverDelta],
) -> Result<SemverChecksFitnessReport, SemverChecksFitnessError> {
    let valid = ["major", "minor", "patch"];
    for d in deltas {
        if !valid.contains(&d.change_class.as_str()) {
            return Err(SemverChecksFitnessError::InvalidChangeClass {
                crate_id: d.crate_id.clone(), class: d.change_class.clone(),
            });
        }
        if d.breaking && d.change_class != "major" {
            return Err(SemverChecksFitnessError::BreakingWithoutMajor {
                crate_id: d.crate_id.clone(), current: d.current_version.clone(), baseline: d.baseline_version.clone(),
            });
        }
    }
    Ok(SemverChecksFitnessReport { crates_checked: deltas.len() })
}
```
