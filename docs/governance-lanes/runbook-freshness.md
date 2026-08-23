---
doc_status: published
---

# Fitness Lane: runbook-freshness

- status: Accepted
- date: 2026-05-12
- purpose: Verify runbooks have a recent-incident link within the last 180d or are explicitly marked advisory.
- enforces: Directive 10 (MASTERPLAN) — runbooks have recent-incident links or marked advisory.
- kernel_crate: `governance-runbook-freshness-kernel` — `RunbookEvidence { runbook_id, latest_incident_age_days, advisory }`, verdict `RunbookFreshnessFitnessReport { runbooks_checked }`.
- runner_path: `tools/governance-runbook-freshness`
- inputs: runbook front-matter `incidents:` list, advisory flag.
- failure_modes:
  - runbook claims operational but no incidents in last 180d
  - incident link 404
  - missing advisory flag on stale runbook
- ci_invocation: `cargo run -p governance-runbook-freshness`
- runtime_budget: 600 ms
- severity: MED
- kernel_sketch:
```rust
pub struct RunbookEvidence {
    pub runbook_id: String,                   // data_class: INTERNAL_ONLY
    pub latest_incident_age_days: Option<u32>,// data_class: INTERNAL_ONLY
    pub advisory: bool,                       // data_class: INTERNAL_ONLY
}
pub struct RunbookFreshnessFitnessReport { pub runbooks_checked: usize }

pub enum RunbookFreshnessFitnessError {
    StaleNonAdvisory { runbook_id: String, age_days: u32 },
    NoIncidentsNotAdvisory { runbook_id: String },
}

pub fn validate_runbook_freshness_fitness(
    evidence: &[RunbookEvidence],
    budget_days: u32,
) -> Result<RunbookFreshnessFitnessReport, RunbookFreshnessFitnessError> {
    for r in evidence {
        if r.advisory { continue; }
        match r.latest_incident_age_days {
            None => return Err(RunbookFreshnessFitnessError::NoIncidentsNotAdvisory { runbook_id: r.runbook_id.clone() }),
            Some(d) if d > budget_days => return Err(RunbookFreshnessFitnessError::StaleNonAdvisory {
                runbook_id: r.runbook_id.clone(), age_days: d,
            }),
            _ => {}
        }
    }
    Ok(RunbookFreshnessFitnessReport { runbooks_checked: evidence.len() })
}
```
