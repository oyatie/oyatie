---
doc_status: published
---

# Fitness Lane: runbook-index-resolves

- status: Accepted
- date: 2026-05-12
- purpose: Verify every runbook listed in `docs/runbooks/INDEX.md` resolves to an existing file and every runbook file appears in INDEX.
- enforces: STANDARD/runbook-index; AGENTS.md fitness-lane `governance-runbook-index-resolves`.
- kernel_crate: `governance-runbook-index-kernel` — `RunbookEntry { runbook_id, path }`, `RunbookFile { path }`, verdict `RunbookIndexFitnessReport { entries_checked, files_checked }`.
- runner_path: `tools/governance-runbook-index-resolves`
- inputs: `docs/runbooks/INDEX.md`, `docs/runbooks/**/*.md`.
- failure_modes:
  - INDEX row points at missing file
  - runbook file exists but not listed in INDEX
  - duplicate runbook id
- ci_invocation: `cargo run -p governance-runbook-index-resolves`
- runtime_budget: 250 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct RunbookEntry { pub runbook_id: String, pub path: String } // data_class: INTERNAL_ONLY
pub struct RunbookFile { pub path: String } // data_class: INTERNAL_ONLY
pub struct RunbookIndexFitnessReport { pub entries_checked: usize, pub files_checked: usize }
pub enum RunbookIndexFitnessError {
    EntryUnresolved { runbook_id: String, path: String },
    FileUnindexed { path: String },
    DuplicateId { runbook_id: String },
}

pub fn validate_runbook_index_fitness(
    entries: &[RunbookEntry],
    files: &[RunbookFile],
) -> Result<RunbookIndexFitnessReport, RunbookIndexFitnessError> {
    let mut ids = std::collections::BTreeSet::new();
    let file_paths: std::collections::BTreeSet<&str> = files.iter().map(|f| f.path.as_str()).collect();
    let entry_paths: std::collections::BTreeSet<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    for e in entries {
        if !ids.insert(e.runbook_id.clone()) {
            return Err(RunbookIndexFitnessError::DuplicateId { runbook_id: e.runbook_id.clone() });
        }
        if !file_paths.contains(e.path.as_str()) {
            return Err(RunbookIndexFitnessError::EntryUnresolved {
                runbook_id: e.runbook_id.clone(),
                path: e.path.clone(),
            });
        }
    }
    for f in files {
        if !entry_paths.contains(f.path.as_str()) {
            return Err(RunbookIndexFitnessError::FileUnindexed { path: f.path.clone() });
        }
    }
    Ok(RunbookIndexFitnessReport { entries_checked: entries.len(), files_checked: files.len() })
}
```
