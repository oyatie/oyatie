---
doc_status: published
---

# Fitness Lane: scaffold-claim-pattern

- status: Accepted
- date: 2026-05-12
- enforces: ADR-0054 — scaffold claim pattern.
- kernel_crate: `governance-scaffold-claim-pattern-kernel` — `ScaffoldClaim { crate_id, claim_id, completion_stamp }`, verdict `ScaffoldClaimPatternFitnessReport { claims_checked }`.
- runner_path: `tools/governance-scaffold-claim-pattern`
- failure_modes:
  - new crate added with no claim_id
  - claim never stamped complete
  - two crates share same claim_id
- ci_invocation: `cargo run -p governance-scaffold-claim-pattern`
- runtime_budget: 400 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct ScaffoldClaim {
    pub crate_id: String,               // data_class: INTERNAL_ONLY
    pub claim_id: Option<String>,       // data_class: INTERNAL_ONLY
    pub completion_stamp: Option<String>,// data_class: INTERNAL_ONLY
}
pub struct ScaffoldClaimPatternFitnessReport { pub claims_checked: usize }

pub enum ScaffoldClaimPatternFitnessError {
    MissingClaim { crate_id: String },
    Unstamped { crate_id: String, claim_id: String },
    DuplicateClaim { claim_id: String, crate_ids: Vec<String> },
}

pub fn validate_scaffold_claim_pattern_fitness(
    claims: &[ScaffoldClaim],
) -> Result<ScaffoldClaimPatternFitnessReport, ScaffoldClaimPatternFitnessError> {
    use std::collections::BTreeMap;
    let mut by_claim: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in claims {
        let id = c.claim_id.as_ref().ok_or_else(|| ScaffoldClaimPatternFitnessError::MissingClaim {
            crate_id: c.crate_id.clone(),
        })?;
        if c.completion_stamp.is_none() {
            return Err(ScaffoldClaimPatternFitnessError::Unstamped {
                crate_id: c.crate_id.clone(), claim_id: id.clone(),
            });
        }
        by_claim.entry(id.clone()).or_default().push(c.crate_id.clone());
    }
    for (claim, ids) in &by_claim {
        if ids.len() > 1 {
            return Err(ScaffoldClaimPatternFitnessError::DuplicateClaim {
                claim_id: claim.clone(), crate_ids: ids.clone(),
            });
        }
    }
    Ok(ScaffoldClaimPatternFitnessReport { claims_checked: claims.len() })
}
```
