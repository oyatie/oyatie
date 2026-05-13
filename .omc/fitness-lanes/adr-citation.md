# Fitness Lane: adr-citation

- purpose: Verify every architectural assertion in canonical docs cites an ADR by id.
- enforces: STANDARD/adr-citation; AGENTS.md fitness-lane `oya-foundry-fitness-adr-citation`.
- kernel_crate: `oya-foundry-adr-citation-kernel` (EXISTING crate; extend with verdict struct) — `AssertionCitation { document_path, line, adr_id }`, verdict `AdrCitationFitnessReport { assertions_checked, adrs_resolved }`.
- runner_path: `tools/oya-foundry-fitness-adr-citation`
- inputs: `docs/**/*.md`, `docs/decisions/ADR-*.md` index.
- failure_modes:
  - assertion line includes "MUST" without `ADR-####` citation
  - cited ADR id does not resolve to a file
  - cited ADR is in `superseded` state
- ci_invocation: `cargo run -p oya-foundry-fitness-adr-citation`
- runtime_budget: 800 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct AssertionCitation {
    pub document_path: String,  // data_class: INTERNAL_ONLY
    pub line: u32,              // data_class: INTERNAL_ONLY
    pub adr_id: Option<String>, // data_class: INTERNAL_ONLY
}

pub struct AdrRecord {
    pub adr_id: String,    // data_class: INTERNAL_ONLY
    pub status: String,    // data_class: INTERNAL_ONLY
}

pub struct AdrCitationFitnessReport {
    pub assertions_checked: usize,
    pub adrs_resolved: usize,
}

pub enum AdrCitationFitnessError {
    MissingCitation { document_path: String, line: u32 },
    UnresolvedAdr { document_path: String, adr_id: String },
    SupersededAdr { document_path: String, adr_id: String },
}

pub fn validate_adr_citation_fitness(
    assertions: &[AssertionCitation],
    adrs: &[AdrRecord],
) -> Result<AdrCitationFitnessReport, AdrCitationFitnessError> {
    let known: std::collections::BTreeMap<&str, &str> =
        adrs.iter().map(|a| (a.adr_id.as_str(), a.status.as_str())).collect();
    let mut resolved = 0;
    for a in assertions {
        let id = a.adr_id.as_ref().ok_or_else(|| AdrCitationFitnessError::MissingCitation {
            document_path: a.document_path.clone(),
            line: a.line,
        })?;
        match known.get(id.as_str()) {
            None => return Err(AdrCitationFitnessError::UnresolvedAdr {
                document_path: a.document_path.clone(),
                adr_id: id.clone(),
            }),
            Some(s) if *s == "superseded" => return Err(AdrCitationFitnessError::SupersededAdr {
                document_path: a.document_path.clone(),
                adr_id: id.clone(),
            }),
            Some(_) => resolved += 1,
        }
    }
    Ok(AdrCitationFitnessReport { assertions_checked: assertions.len(), adrs_resolved: resolved })
}
```
