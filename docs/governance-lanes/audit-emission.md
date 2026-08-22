---
doc_status: published
---

# Fitness Lane: audit-emission

- status: Accepted
- date: 2026-05-12
- purpose: Verify every kernel function that mutates state declares its audit-emission contract in the registry and that runtime adapters emit the declared event.
- enforces: STANDARD/audit-chain; AGENTS.md fitness-lane `governance-audit-emission`.
- kernel_crate: `governance-audit-emission-kernel` — `MutationFn { crate_id, fn_path, declared_event }`, verdict `AuditEmissionFitnessReport { fns_checked, events_resolved }`.
- runner_path: `tools/governance-audit-emission`
- inputs: kernel source files, `docs/contracts/audit-events.md` registry.
- failure_modes:
  - function with `mutate_` prefix has no declared event
  - declared event id not in registry
  - duplicate fn->event mapping
- ci_invocation: `cargo run -p governance-audit-emission`
- runtime_budget: 700 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct MutationFn {
    pub crate_id: String,           // data_class: INTERNAL_ONLY
    pub fn_path: String,            // data_class: INTERNAL_ONLY
    pub declared_event: Option<String>, // data_class: INTERNAL_ONLY
}

pub struct AuditEmissionFitnessReport { pub fns_checked: usize, pub events_resolved: usize }

pub enum AuditEmissionFitnessError {
    MissingDeclaration { crate_id: String, fn_path: String },
    UnknownEvent { fn_path: String, event: String },
    DuplicateMapping { fn_path: String },
}

pub fn validate_audit_emission_fitness(
    fns: &[MutationFn],
    known_events: &[String],
) -> Result<AuditEmissionFitnessReport, AuditEmissionFitnessError> {
    let known: std::collections::BTreeSet<&str> = known_events.iter().map(|s| s.as_str()).collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut resolved = 0;
    for f in fns {
        let e = f.declared_event.as_ref().ok_or_else(|| AuditEmissionFitnessError::MissingDeclaration {
            crate_id: f.crate_id.clone(), fn_path: f.fn_path.clone(),
        })?;
        if !known.contains(e.as_str()) {
            return Err(AuditEmissionFitnessError::UnknownEvent {
                fn_path: f.fn_path.clone(), event: e.clone(),
            });
        }
        if !seen.insert(f.fn_path.clone()) {
            return Err(AuditEmissionFitnessError::DuplicateMapping { fn_path: f.fn_path.clone() });
        }
        resolved += 1;
    }
    Ok(AuditEmissionFitnessReport { fns_checked: fns.len(), events_resolved: resolved })
}
```
