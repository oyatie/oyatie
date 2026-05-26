---
doc_status: published
---

# Fitness Lane: traceability-validator

- status: Accepted
- date: 2026-05-12
- purpose: Verify the PRD->ADR->crate->test traceability chain resolves end-to-end for every accepted requirement.
- enforces: STANDARD/traceability-chain; AGENTS.md fitness-lane `oya-governance-traceability-validator`.
- kernel_crate: `oya-governance-traceability-kernel` — `TraceLink { requirement_id, adr_ids, crate_ids, test_ids }`, verdict `TraceabilityFitnessReport { links_checked }`.
- runner_path: `tools/oya-governance-traceability-validator`
- inputs: PRD index, ADR index, workspace crate list, `cargo nextest` test list.
- failure_modes:
  - accepted requirement with no ADR
  - ADR cited but no implementing crate
  - crate cited but no covering test
- ci_invocation: `cargo run -p oya-governance-traceability-validator`
- runtime_budget: 1500 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct TraceLink {
    pub requirement_id: String, // data_class: INTERNAL_ONLY
    pub adr_ids: Vec<String>,   // data_class: INTERNAL_ONLY
    pub crate_ids: Vec<String>, // data_class: INTERNAL_ONLY
    pub test_ids: Vec<String>,  // data_class: INTERNAL_ONLY
}

pub struct TraceabilityFitnessReport { pub links_checked: usize }

pub enum TraceabilityFitnessError {
    NoAdr { requirement_id: String },
    NoCrate { requirement_id: String, adr_id: String },
    NoTest { requirement_id: String, crate_id: String },
}

pub fn validate_traceability_fitness(
    links: &[TraceLink],
) -> Result<TraceabilityFitnessReport, TraceabilityFitnessError> {
    for l in links {
        if l.adr_ids.is_empty() {
            return Err(TraceabilityFitnessError::NoAdr { requirement_id: l.requirement_id.clone() });
        }
        if l.crate_ids.is_empty() {
            return Err(TraceabilityFitnessError::NoCrate {
                requirement_id: l.requirement_id.clone(),
                adr_id: l.adr_ids[0].clone(),
            });
        }
        if l.test_ids.is_empty() {
            return Err(TraceabilityFitnessError::NoTest {
                requirement_id: l.requirement_id.clone(),
                crate_id: l.crate_ids[0].clone(),
            });
        }
    }
    Ok(TraceabilityFitnessReport { links_checked: links.len() })
}
```
