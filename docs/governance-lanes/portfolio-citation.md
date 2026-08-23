---
doc_status: published
---

# Fitness Lane: portfolio-citation

- status: Accepted
- date: 2026-05-12
- purpose: Verify bidirectional cross-citations between `bominal/` and `oyatie/` PRDs.
- enforces: Directive A1 (MASTERPLAN) — bidirectional bominal<->oyatie PRD cite.
- kernel_crate: `governance-portfolio-citation-kernel` — `PortfolioCitation { source_repo, source_prd, target_repo, target_prd }`, verdict `PortfolioCitationFitnessReport { citations_checked }`.
- runner_path: `tools/governance-portfolio-citation`
- inputs: `bominal/docs/prd/**/*.md` (read-only mirror), `oyatie/docs/prd/**/*.md`.
- failure_modes:
  - oyatie PRD references a bominal PRD that does not reciprocate
  - bominal PRD missing oyatie back-cite
  - cited PRD path unresolved
- adr_citations: ADR-0052 (inventory — the bidirectional citation requirement is grounded in the cross-repo artifact inventory that classifies PRDs as authoritative artifacts requiring back-links)
- ci_invocation: `cargo run -p governance-portfolio-citation`
- runtime_budget: 800 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct PortfolioCitation {
    pub source_repo: String,  // data_class: INTERNAL_ONLY
    pub source_prd: String,   // data_class: INTERNAL_ONLY
    pub target_repo: String,  // data_class: INTERNAL_ONLY
    pub target_prd: String,   // data_class: INTERNAL_ONLY
}

pub struct PortfolioCitationFitnessReport { pub citations_checked: usize }

pub enum PortfolioCitationFitnessError {
    MissingReciprocal { source_prd: String, target_prd: String },
    UnresolvedTarget { source_prd: String, target_prd: String },
}

pub fn validate_portfolio_citation_fitness(
    citations: &[PortfolioCitation],
    known_prds: &[(String, String)], // (repo, prd)
) -> Result<PortfolioCitationFitnessReport, PortfolioCitationFitnessError> {
    let known: std::collections::BTreeSet<(&str, &str)> =
        known_prds.iter().map(|(r, p)| (r.as_str(), p.as_str())).collect();
    let set: std::collections::BTreeSet<(&str, &str, &str, &str)> =
        citations.iter().map(|c| (c.source_repo.as_str(), c.source_prd.as_str(), c.target_repo.as_str(), c.target_prd.as_str())).collect();
    for c in citations {
        if !known.contains(&(c.target_repo.as_str(), c.target_prd.as_str())) {
            return Err(PortfolioCitationFitnessError::UnresolvedTarget {
                source_prd: c.source_prd.clone(), target_prd: c.target_prd.clone(),
            });
        }
        let reciprocal = (c.target_repo.as_str(), c.target_prd.as_str(), c.source_repo.as_str(), c.source_prd.as_str());
        if !set.contains(&reciprocal) {
            return Err(PortfolioCitationFitnessError::MissingReciprocal {
                source_prd: c.source_prd.clone(), target_prd: c.target_prd.clone(),
            });
        }
    }
    Ok(PortfolioCitationFitnessReport { citations_checked: citations.len() })
}
```
