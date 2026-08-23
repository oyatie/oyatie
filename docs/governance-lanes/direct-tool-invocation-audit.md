---
doc_status: published
---

# Fitness Lane: direct-tool-invocation-audit

- status: Accepted
- date: 2026-05-12
- kernel_crate: `governance-direct-tool-invocation-kernel` — `ToolInvocation { session_id, tool, command, icm_record_id }`, verdict `DirectToolInvocationFitnessReport { invocations_checked }`.
- runner_path: `tools/governance-direct-tool-invocation-audit`
- failure_modes:
- ci_invocation: `cargo run -p governance-direct-tool-invocation-audit`
- runtime_budget: 800 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct ToolInvocation {
    pub session_id: String,        // data_class: INTERNAL_ONLY
    pub tool: String,              // data_class: INTERNAL_ONLY
    pub command: String,           // data_class: INTERNAL_ONLY
    pub icm_record_id: Option<String>, // data_class: INTERNAL_ONLY
}
pub struct IcmRecord { pub id: String, pub session_id: String } // data_class: INTERNAL_ONLY
pub struct DirectToolInvocationFitnessReport { pub invocations_checked: usize }

pub enum DirectToolInvocationFitnessError {
    MissingRecord { session_id: String, command: String },
    UnknownRecord { record_id: String },
    DuplicateRecordForDistinctCommands { record_id: String },
}

pub fn validate_direct_tool_invocation_fitness(
    invocations: &[ToolInvocation],
    records: &[IcmRecord],
) -> Result<DirectToolInvocationFitnessReport, DirectToolInvocationFitnessError> {
    let known: std::collections::BTreeSet<&str> = records.iter().map(|r| r.id.as_str()).collect();
    let mut seen: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    for i in invocations {
        let r = i.icm_record_id.as_ref().ok_or_else(|| DirectToolInvocationFitnessError::MissingRecord {
            session_id: i.session_id.clone(), command: i.command.clone(),
        })?;
        if !known.contains(r.as_str()) {
            return Err(DirectToolInvocationFitnessError::UnknownRecord { record_id: r.clone() });
        }
        if let Some(existing) = seen.get(r.as_str()) {
            if *existing != i.command { return Err(DirectToolInvocationFitnessError::DuplicateRecordForDistinctCommands { record_id: r.clone() }); }
        } else { seen.insert(r.as_str(), i.command.as_str()); }
    }
    Ok(DirectToolInvocationFitnessReport { invocations_checked: invocations.len() })
}
```
