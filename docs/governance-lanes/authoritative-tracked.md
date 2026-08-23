---
doc_status: published
---

# Fitness Lane: authoritative-tracked

- status: Accepted
- date: 2026-05-12
- purpose: Verify every authoritative artifact (ADR, standard, runbook, template, checklist) is tracked in git (no untracked or .gitignored authoritative paths).
- enforces: Directive A8 (MASTERPLAN).
- kernel_crate: `governance-authoritative-tracked-kernel` — `AuthoritativeArtifact { path, tracked }`, verdict `AuthoritativeTrackedFitnessReport { artifacts_checked }`.
- runner_path: `tools/governance-authoritative-tracked`
- inputs: catalog rows with `authoritative: true`, `git ls-files` snapshot.
- failure_modes:
  - authoritative doc under `.gitignore`
  - authoritative path exists on disk but not in git index
  - catalog marks authoritative but file missing
- ci_invocation: `cargo run -p governance-authoritative-tracked`
- runtime_budget: 400 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct AuthoritativeArtifact {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub tracked: bool,   // data_class: INTERNAL_ONLY
    pub on_disk: bool,   // data_class: INTERNAL_ONLY
    pub gitignored: bool,// data_class: INTERNAL_ONLY
}

pub struct AuthoritativeTrackedFitnessReport { pub artifacts_checked: usize }

pub enum AuthoritativeTrackedFitnessError {
    Untracked { path: String },
    Gitignored { path: String },
    Missing { path: String },
}

pub fn validate_authoritative_tracked_fitness(
    artifacts: &[AuthoritativeArtifact],
) -> Result<AuthoritativeTrackedFitnessReport, AuthoritativeTrackedFitnessError> {
    for a in artifacts {
        if !a.on_disk { return Err(AuthoritativeTrackedFitnessError::Missing { path: a.path.clone() }); }
        if a.gitignored { return Err(AuthoritativeTrackedFitnessError::Gitignored { path: a.path.clone() }); }
        if !a.tracked { return Err(AuthoritativeTrackedFitnessError::Untracked { path: a.path.clone() }); }
    }
    Ok(AuthoritativeTrackedFitnessReport { artifacts_checked: artifacts.len() })
}
```
