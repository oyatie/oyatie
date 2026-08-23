---
doc_status: published
---

# Fitness Lane: foundry-corpus-citation

- status: Accepted
- date: 2026-05-12
- purpose: Verify every foundry-internal artifact that depends on the salvaged corpus cites a corpus row id.
- enforces: MASTERPLAN P3.5 — foundry corpus cross-cite.
- kernel_crate: `governance-foundry-corpus-citation-kernel` — `CorpusCitation { path, line, corpus_id }`, verdict `FoundryCorpusCitationFitnessReport { citations_checked }`.
- runner_path: `tools/governance-foundry-corpus-citation`
- inputs: `.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md`, every doc/code marked `corpus-dependent`.
- failure_modes:
  - missing corpus id citation
  - cited id unresolved
  - duplicate citation collision in same doc
- ci_invocation: `cargo run -p governance-foundry-corpus-citation`
- runtime_budget: 500 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct CorpusCitation {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub line: u32,       // data_class: INTERNAL_ONLY
    pub corpus_id: Option<String>, // data_class: INTERNAL_ONLY
}
pub struct CorpusRow { pub corpus_id: String } // data_class: INTERNAL_ONLY
pub struct FoundryCorpusCitationFitnessReport { pub citations_checked: usize }

pub enum FoundryCorpusCitationFitnessError {
    MissingId { path: String, line: u32 },
    Unresolved { path: String, corpus_id: String },
}

pub fn validate_foundry_corpus_citation_fitness(
    citations: &[CorpusCitation],
    rows: &[CorpusRow],
) -> Result<FoundryCorpusCitationFitnessReport, FoundryCorpusCitationFitnessError> {
    let known: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.corpus_id.as_str()).collect();
    for c in citations {
        let id = c.corpus_id.as_ref().ok_or_else(|| FoundryCorpusCitationFitnessError::MissingId {
            path: c.path.clone(), line: c.line,
        })?;
        if !known.contains(id.as_str()) {
            return Err(FoundryCorpusCitationFitnessError::Unresolved {
                path: c.path.clone(), corpus_id: id.clone(),
            });
        }
    }
    Ok(FoundryCorpusCitationFitnessReport { citations_checked: citations.len() })
}
```
