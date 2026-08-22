---
doc_status: published
---

# Fitness Lane: lts-dependency

- status: Accepted
- date: 2026-05-12
- purpose: Verify every workspace dep is LTS-pinned per `lts-versions-verified` spec and `cargo deny` policy passes.
- enforces: Directive 8 (MASTERPLAN), `.omc/scratch/lts-versions-verified-2026-05-12.md`.
- kernel_crate: `governance-lts-dependency-kernel` — `DepPin { crate_id, dep_name, requested, verified_lts }`, verdict `LtsDependencyFitnessReport { deps_checked }`.
- runner_path: `tools/governance-lts-dependency`
- inputs: `Cargo.toml` workspace dep table, lts-verified registry, `cargo deny` report.
- failure_modes:
  - dep request differs from LTS pin
  - cargo deny reports advisory/license violation
  - dep on yanked version
- ci_invocation: `cargo run -p governance-lts-dependency`
- runtime_budget: 1800 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct DepPin {
    pub crate_id: String,        // data_class: INTERNAL_ONLY
    pub dep_name: String,        // data_class: INTERNAL_ONLY
    pub requested: String,       // data_class: INTERNAL_ONLY (semver req)
    pub verified_lts: String,    // data_class: INTERNAL_ONLY
    pub deny_advisory: Option<String>, // data_class: INTERNAL_ONLY
    pub yanked: bool,            // data_class: INTERNAL_ONLY
}

pub struct LtsDependencyFitnessReport { pub deps_checked: usize }

pub enum LtsDependencyFitnessError {
    NotLts { dep: String, requested: String, verified: String },
    DenyViolation { dep: String, advisory: String },
    Yanked { dep: String, requested: String },
}

pub fn validate_lts_dependency_fitness(
    deps: &[DepPin],
) -> Result<LtsDependencyFitnessReport, LtsDependencyFitnessError> {
    for d in deps {
        if let Some(a) = &d.deny_advisory {
            return Err(LtsDependencyFitnessError::DenyViolation { dep: d.dep_name.clone(), advisory: a.clone() });
        }
        if d.yanked {
            return Err(LtsDependencyFitnessError::Yanked { dep: d.dep_name.clone(), requested: d.requested.clone() });
        }
        if d.requested != d.verified_lts {
            return Err(LtsDependencyFitnessError::NotLts {
                dep: d.dep_name.clone(), requested: d.requested.clone(), verified: d.verified_lts.clone(),
            });
        }
    }
    Ok(LtsDependencyFitnessReport { deps_checked: deps.len() })
}
```
