---
doc_status: published
---

# Fitness Lane: agent-completion-checklist

- status: Accepted
- date: 2026-05-12
- enforces: CHECKLIST/agent-completion; ADR-0054.
- kernel_crate: `governance-agent-completion-checklist-kernel` — `CompletionTriple { grit_done_id, icm_store_id, audit_event_id }`, verdict `AgentCompletionChecklistFitnessReport { triples_checked }`.
- runner_path: `tools/governance-agent-completion-checklist`
- failure_modes:
- ci_invocation: `cargo run -p governance-agent-completion-checklist`
- runtime_budget: 600 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct CompletionTriple {
    pub grit_done_id: String,           // data_class: INTERNAL_ONLY
    pub icm_store_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub audit_event_id: Option<String>, // data_class: INTERNAL_ONLY
}
pub struct AgentCompletionChecklistFitnessReport { pub triples_checked: usize }

pub enum AgentCompletionChecklistFitnessError {
    MissingIcmStore { grit_done_id: String },
    MissingAuditEmission { grit_done_id: String, icm_store_id: String },
    DuplicateAuditEmission { audit_event_id: String, grit_done_ids: Vec<String> },
}

pub fn validate_agent_completion_checklist_fitness(
    triples: &[CompletionTriple],
) -> Result<AgentCompletionChecklistFitnessReport, AgentCompletionChecklistFitnessError> {
    use std::collections::BTreeMap;
    let mut audit_to_grit: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for t in triples {
            grit_done_id: t.grit_done_id.clone(),
        })?;
        let audit = t.audit_event_id.as_ref().ok_or_else(|| AgentCompletionChecklistFitnessError::MissingAuditEmission {
        })?;
        audit_to_grit.entry(audit.clone()).or_default().push(t.grit_done_id.clone());
    }
    for (event, grits) in &audit_to_grit {
        if grits.len() > 1 {
            return Err(AgentCompletionChecklistFitnessError::DuplicateAuditEmission {
                audit_event_id: event.clone(), grit_done_ids: grits.clone(),
            });
        }
    }
    Ok(AgentCompletionChecklistFitnessReport { triples_checked: triples.len() })
}
```
