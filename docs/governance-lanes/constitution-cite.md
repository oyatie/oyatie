---
doc_status: published
---

# Fitness Lane: constitution-cite

- status: Accepted
- date: 2026-05-12
- purpose: Verify every standard cites the constitutional principle it derives from.
- enforces: STANDARD/constitution-derivation; existing crate `governance-constitution-cite-kernel` (EXISTING).
- kernel_crate: `governance-constitution-cite-kernel` (EXISTING) — `StandardCite { standard_id, principle_id }`, verdict `ConstitutionCiteFitnessReport { citations_checked }`.
- runner_path: `tools/governance-constitution-cite`
- inputs: `docs/standards/**/*.md` front-matter `derives_from:`, constitution principle registry.
- failure_modes:
  - standard with no `derives_from:`
  - principle id unresolved
  - principle id retracted
- ci_invocation: `cargo run -p governance-constitution-cite`
- runtime_budget: 350 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct StandardCite {
    pub standard_id: String,         // data_class: INTERNAL_ONLY
    pub principle_id: Option<String>,// data_class: INTERNAL_ONLY
}
pub struct Principle { pub id: String, pub status: String } // data_class: INTERNAL_ONLY
pub struct ConstitutionCiteFitnessReport { pub citations_checked: usize }

pub enum ConstitutionCiteFitnessError {
    MissingCite { standard_id: String },
    UnknownPrinciple { standard_id: String, principle_id: String },
    RetractedPrinciple { standard_id: String, principle_id: String },
}

pub fn validate_constitution_cite_fitness(
    cites: &[StandardCite],
    principles: &[Principle],
) -> Result<ConstitutionCiteFitnessReport, ConstitutionCiteFitnessError> {
    let by_id: std::collections::BTreeMap<&str, &Principle> = principles.iter().map(|p| (p.id.as_str(), p)).collect();
    for c in cites {
        let id = c.principle_id.as_ref().ok_or_else(|| ConstitutionCiteFitnessError::MissingCite {
            standard_id: c.standard_id.clone(),
        })?;
        let p = by_id.get(id.as_str()).ok_or_else(|| ConstitutionCiteFitnessError::UnknownPrinciple {
            standard_id: c.standard_id.clone(), principle_id: id.clone(),
        })?;
        if p.status == "retracted" {
            return Err(ConstitutionCiteFitnessError::RetractedPrinciple {
                standard_id: c.standard_id.clone(), principle_id: id.clone(),
            });
        }
    }
    Ok(ConstitutionCiteFitnessReport { citations_checked: cites.len() })
}
```
