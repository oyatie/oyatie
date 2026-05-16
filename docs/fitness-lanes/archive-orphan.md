---
doc_status: published
---

# Fitness Lane: archive-orphan

- status: Accepted
- date: 2026-05-12
- purpose: Verify Bominal ultragoal orchestration-glue ARCHIVE rows were moved under `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/`, active originals are absent, and living files do not reference the archived runtime paths except explicit authority/provenance docs.
- enforces: Directive A3 P6/P7 gate (MASTERPLAN) — archive-before-delete rollback boundary.
- activation: Scaffolded and active at M-CC-P01-IP-008 / P6; used as a required P7 deletion precondition.
- kernel_crate: `oya-foundry-fitness-archive-orphan-kernel` — `ArchivedPath { original_path, archive_path, original_exists, archive_exists }`, `InboundRef { source_path, target_path, line, context }`, verdict `ArchiveOrphanFitnessReport { archives_checked, archive_files_present, originals_absent, inbound_refs_checked }`.
- runner_path: `tools/oya-foundry-fitness-archive-orphan`
- inputs: `docs/decisions/ADR-0052-inventory-grit-cutover.md` ARCHIVE rows, filesystem state rooted at `..`, living scan roots under Oyatie `docs/`, `.omc/`, `crates/`, `tools/`, root manifests/instructions, plus `../bominal/agents/ultragoal`, and authority-source allowlist.
- failure_modes:
  - ARCHIVE row missing its archived copy
  - ARCHIVE row still exists at its active pre-cutover path
  - archive copy lands outside `archive/pre-grit-cutover-2026-05-12/`
  - living code/docs/config reference an archived runtime-glue path outside authority/provenance docs
- ci_invocation: `cargo run -p oya-foundry-fitness-archive-orphan`
- runtime_budget: 1200 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct ArchivedPath {
    pub original_path: String, // data_class: INTERNAL_ONLY
    pub archive_path: String,  // data_class: INTERNAL_ONLY
    pub original_exists: bool, // data_class: INTERNAL_ONLY
    pub archive_exists: bool,  // data_class: INTERNAL_ONLY
}

pub struct InboundRef {
    pub source_path: String, // data_class: INTERNAL_ONLY
    pub target_path: String, // data_class: INTERNAL_ONLY
    pub line: u32,           // data_class: INTERNAL_ONLY
    pub context: String,     // data_class: INTERNAL_ONLY
}

pub fn check(
    archived: &[ArchivedPath],
    inbound_refs: &[InboundRef],
) -> Result<ArchiveOrphanFitnessReport, ArchiveOrphanFitnessError> { /* implemented in Rust */ }
```
