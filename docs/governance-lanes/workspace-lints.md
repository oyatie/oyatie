---
doc_status: published
---

# Fitness Lane: workspace-lints

- status: Accepted
- date: 2026-05-12
- purpose: Verify workspace `Cargo.toml` has a `[workspace.lints]` table and every member crate inherits via `lints.workspace = true`.
- enforces: hyperscaler-best-practices spec — workspace lints inherited.
- kernel_crate: `governance-workspace-lints-kernel` — `MemberManifest { crate_id, inherits_lints }`, verdict `WorkspaceLintsFitnessReport { crates_checked }`.
- runner_path: `tools/governance-workspace-lints`
- inputs: workspace + crate `Cargo.toml`s.
- failure_modes:
  - workspace lacks `[workspace.lints]`
  - member crate sets its own `[lints]` instead of inheriting
  - inherited table empty
- ci_invocation: `cargo run -p governance-workspace-lints`
- runtime_budget: 250 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct MemberManifest {
    pub crate_id: String,         // data_class: INTERNAL_ONLY
    pub inherits_lints: bool,     // data_class: INTERNAL_ONLY
    pub has_local_lints: bool,    // data_class: INTERNAL_ONLY
}
pub struct WorkspaceLintsFitnessReport { pub crates_checked: usize }

pub enum WorkspaceLintsFitnessError {
    WorkspaceTableMissing,
    LocalOverridesInherited { crate_id: String },
    NotInherited { crate_id: String },
}

pub fn validate_workspace_lints_fitness(
    members: &[MemberManifest],
    workspace_table_present: bool,
) -> Result<WorkspaceLintsFitnessReport, WorkspaceLintsFitnessError> {
    if !workspace_table_present {
        return Err(WorkspaceLintsFitnessError::WorkspaceTableMissing);
    }
    for m in members {
        if m.has_local_lints {
            return Err(WorkspaceLintsFitnessError::LocalOverridesInherited { crate_id: m.crate_id.clone() });
        }
        if !m.inherits_lints {
            return Err(WorkspaceLintsFitnessError::NotInherited { crate_id: m.crate_id.clone() });
        }
    }
    Ok(WorkspaceLintsFitnessReport { crates_checked: members.len() })
}
```
