---
doc_status: published
---

# Fitness Lane: glossary

- status: Accepted
- date: 2026-05-12
- purpose: Verify every domain term used in canonical docs has a glossary entry.
- enforces: STANDARD/glossary-required-terms; AGENTS.md fitness-lane `governance-glossary`.
- kernel_crate: `governance-glossary-kernel` — `TermOccurrence { term, document_path, line }`, `GlossaryEntry { term, definition_present }`, verdict `GlossaryFitnessReport { terms_checked, documents_checked }`.
- runner_path: `tools/governance-glossary`
- inputs: `docs/glossary.md`, every `docs/**/*.md` canonical doc, registry of required-defined terms.
- failure_modes:
  - capitalized term `TenantId` appears in standard but no glossary row exists
  - glossary row exists but has empty definition
  - duplicate glossary rows
- ci_invocation: `cargo run -p governance-glossary`
- runtime_budget: 700 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct TermOccurrence {
    pub term: String,           // data_class: INTERNAL_ONLY
    pub document_path: String,  // data_class: INTERNAL_ONLY
    pub line: u32,              // data_class: INTERNAL_ONLY
}

pub struct GlossaryEntry {
    pub term: String,                 // data_class: INTERNAL_ONLY
    pub definition_present: bool,     // data_class: INTERNAL_ONLY
}

pub struct GlossaryFitnessReport {
    pub terms_checked: usize,
    pub documents_checked: usize,
}

pub enum GlossaryFitnessError {
    MissingDefinition { term: String, first_seen: String },
    EmptyDefinition { term: String },
    DuplicateGlossaryRow { term: String },
}

pub fn validate_glossary_fitness(
    occurrences: &[TermOccurrence],
    entries: &[GlossaryEntry],
) -> Result<GlossaryFitnessReport, GlossaryFitnessError> {
    let mut seen = std::collections::BTreeSet::new();
    for e in entries {
        if !seen.insert(e.term.clone()) {
            return Err(GlossaryFitnessError::DuplicateGlossaryRow { term: e.term.clone() });
        }
        if !e.definition_present {
            return Err(GlossaryFitnessError::EmptyDefinition { term: e.term.clone() });
        }
    }
    for o in occurrences {
        if !seen.contains(&o.term) {
            return Err(GlossaryFitnessError::MissingDefinition {
                term: o.term.clone(),
                first_seen: o.document_path.clone(),
            });
        }
    }
    Ok(GlossaryFitnessReport {
        terms_checked: occurrences.len(),
        documents_checked: occurrences.iter().map(|o| o.document_path.clone()).collect::<std::collections::BTreeSet<_>>().len(),
    })
}
```
