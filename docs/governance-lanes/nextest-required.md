---
doc_status: published
---

# Fitness Lane: nextest-required

- status: Accepted
- date: 2026-05-12
- purpose: Verify `cargo nextest run` is the test driver in CI and that every crate has at least one nextest-discovered test.
- enforces: hyperscaler-best-practices spec — Rust `cargo nextest` mandatory.
- kernel_crate: `governance-nextest-required-kernel` — `NextestRun { crate_id, test_count, used_nextest }`, verdict `NextestRequiredFitnessReport { crates_checked }`.
- runner_path: `tools/governance-nextest-required`
- inputs: `cargo nextest list --message-format=json`, CI workflow file.
- failure_modes:
  - CI job uses `cargo test` instead of `cargo nextest`
  - crate has zero tests
  - nextest binary missing from toolchain
- ci_invocation: `cargo run -p governance-nextest-required`
- runtime_budget: 1700 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct NextestRun {
    pub crate_id: String,     // data_class: INTERNAL_ONLY
    pub test_count: u32,      // data_class: INTERNAL_ONLY
    pub used_nextest: bool,   // data_class: INTERNAL_ONLY
}
pub struct NextestRequiredFitnessReport { pub crates_checked: usize }

pub enum NextestRequiredFitnessError {
    NotUsingNextest { crate_id: String },
    NoTests { crate_id: String },
}

pub fn validate_nextest_required_fitness(
    runs: &[NextestRun],
) -> Result<NextestRequiredFitnessReport, NextestRequiredFitnessError> {
    for r in runs {
        if !r.used_nextest { return Err(NextestRequiredFitnessError::NotUsingNextest { crate_id: r.crate_id.clone() }); }
        if r.test_count == 0 { return Err(NextestRequiredFitnessError::NoTests { crate_id: r.crate_id.clone() }); }
    }
    Ok(NextestRequiredFitnessReport { crates_checked: runs.len() })
}
```
