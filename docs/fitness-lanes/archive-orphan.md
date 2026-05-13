# Fitness Lane: archive-orphan

- status: Accepted
- date: 2026-05-12
- purpose: Verify archived paths (`docs/archive/**`, `crates/_attic/**`) have zero inbound references from living code, docs, or configs.
- enforces: Directive A3 P7 gate (MASTERPLAN).
- kernel_crate: `oya-foundry-fitness-archive-orphan-kernel` — `ArchivedPath { path }`, `InboundRef { source_path, target_path, line }`, verdict `ArchiveOrphanFitnessReport { archives_checked }`.
- runner_path: `tools/oya-foundry-fitness-archive-orphan`
- inputs: archive path list, repo-wide rg index of refs.
- failure_modes:
  - active doc links to archived path
  - active crate `use` references archived crate
  - CI config references archived runbook
- ci_invocation: `cargo run -p oya-foundry-fitness-archive-orphan`
- runtime_budget: 1200 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct ArchivedPath { pub path: String } // data_class: INTERNAL_ONLY
pub struct InboundRef {
    pub source_path: String, // data_class: INTERNAL_ONLY
    pub target_path: String, // data_class: INTERNAL_ONLY
    pub line: u32,           // data_class: INTERNAL_ONLY
}
pub struct ArchiveOrphanFitnessReport { pub archives_checked: usize }

pub enum ArchiveOrphanFitnessError {
    LiveRefToArchived { source_path: String, target_path: String, line: u32 },
}

pub fn validate_archive_orphan_fitness(
    archived: &[ArchivedPath],
    refs: &[InboundRef],
) -> Result<ArchiveOrphanFitnessReport, ArchiveOrphanFitnessError> {
    let archived_set: std::collections::BTreeSet<&str> = archived.iter().map(|a| a.path.as_str()).collect();
    for r in refs {
        if archived_set.contains(r.target_path.as_str()) && !archived_set.contains(r.source_path.as_str()) {
            return Err(ArchiveOrphanFitnessError::LiveRefToArchived {
                source_path: r.source_path.clone(),
                target_path: r.target_path.clone(),
                line: r.line,
            });
        }
    }
    Ok(ArchiveOrphanFitnessReport { archives_checked: archived.len() })
}
```
