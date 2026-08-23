---
doc_status: published
---

# Fitness Lane: orphan-detection

- status: Accepted
- date: 2026-05-12
- purpose: Verify every artifact has a declared purpose (front-matter `purpose:` or catalog row) and is reachable from a registered entry point.
- enforces: Directive 10 (MASTERPLAN) — every artifact has declared purpose.
- kernel_crate: `governance-orphan-detection-kernel` — `Artifact { path, has_purpose, reachable }`, verdict `OrphanDetectionFitnessReport { artifacts_checked }`.
- runner_path: `tools/governance-orphan-detection`
- inputs: artifact list, registered entry points, ref-graph snapshot.
- failure_modes:
  - doc with no `purpose:` front-matter
  - artifact present but not reachable from any entry point
  - dead crate (no rdeps, no entry)
- ci_invocation: `cargo run -p governance-orphan-detection`
- runtime_budget: 1100 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct Artifact {
    pub path: String,       // data_class: INTERNAL_ONLY
    pub has_purpose: bool,  // data_class: INTERNAL_ONLY
    pub reachable: bool,    // data_class: INTERNAL_ONLY
}

pub struct OrphanDetectionFitnessReport { pub artifacts_checked: usize }

pub enum OrphanDetectionFitnessError {
    MissingPurpose { path: String },
    Unreachable { path: String },
}

pub fn validate_orphan_detection_fitness(
    artifacts: &[Artifact],
) -> Result<OrphanDetectionFitnessReport, OrphanDetectionFitnessError> {
    for a in artifacts {
        if !a.has_purpose { return Err(OrphanDetectionFitnessError::MissingPurpose { path: a.path.clone() }); }
        if !a.reachable { return Err(OrphanDetectionFitnessError::Unreachable { path: a.path.clone() }); }
    }
    Ok(OrphanDetectionFitnessReport { artifacts_checked: artifacts.len() })
}
```
