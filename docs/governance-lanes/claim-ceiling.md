---
doc_status: published
---

# Fitness Lane: claim-ceiling

- status: Accepted
- date: 2026-05-12
- enforces: STANDARD/claim-ceiling; existing crate `governance-claim-ceiling-kernel` (EXISTING).
- kernel_crate: `governance-claim-ceiling-kernel` (EXISTING) — `ClaimSnapshot { agent_id, active_claims }`, verdict `ClaimCeilingFitnessReport { agents_checked }`.
- runner_path: `tools/governance-claim-ceiling`
- failure_modes:
  - agent holds claims above ceiling
  - claim with no expiry
  - duplicate claim id
- ci_invocation: `cargo run -p governance-claim-ceiling`
- runtime_budget: 300 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct ClaimSnapshot {
    pub agent_id: String,           // data_class: INTERNAL_ONLY
    pub active_claims: Vec<String>, // data_class: INTERNAL_ONLY
}
pub struct ClaimCeilingFitnessReport { pub agents_checked: usize }

pub enum ClaimCeilingFitnessError {
    AboveCeiling { agent_id: String, active: usize, ceiling: usize },
    DuplicateClaim { agent_id: String, claim_id: String },
}

pub fn validate_claim_ceiling_fitness(
    snapshots: &[ClaimSnapshot],
    ceiling: usize,
) -> Result<ClaimCeilingFitnessReport, ClaimCeilingFitnessError> {
    for s in snapshots {
        if s.active_claims.len() > ceiling {
            return Err(ClaimCeilingFitnessError::AboveCeiling {
                agent_id: s.agent_id.clone(), active: s.active_claims.len(), ceiling,
            });
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in &s.active_claims {
            if !seen.insert(c.clone()) {
                return Err(ClaimCeilingFitnessError::DuplicateClaim {
                    agent_id: s.agent_id.clone(), claim_id: c.clone(),
                });
            }
        }
    }
    Ok(ClaimCeilingFitnessReport { agents_checked: snapshots.len() })
}
```
