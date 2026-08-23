---
doc_status: published
---

# Fitness Lane: glossary-vocabulary

- status: Accepted
- date: 2026-05-12
- purpose: Verify retired terms ("M0", "M3", "MVP") never appear in active canonical docs (only in archives + retired-vocab notes).
- enforces: STANDARD/vocabulary-retirement.
- kernel_crate: `governance-glossary-vocabulary-kernel` — `RetiredTermOccurrence { token, path, line, archived }`, verdict `GlossaryVocabularyFitnessReport { occurrences_checked }`.
- runner_path: `tools/governance-glossary-vocabulary`
- inputs: docs tree, retired-vocab registry.
- failure_modes:
  - "M0" appears in live PRD
  - "MVP" used in active doc title
  - "M3" appears in standard or runbook
- ci_invocation: `cargo run -p governance-glossary-vocabulary`
- runtime_budget: 500 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct RetiredTermOccurrence {
    pub token: String,   // data_class: INTERNAL_ONLY
    pub path: String,    // data_class: INTERNAL_ONLY
    pub line: u32,       // data_class: INTERNAL_ONLY
    pub archived: bool,  // data_class: INTERNAL_ONLY
}
pub struct GlossaryVocabularyFitnessReport { pub occurrences_checked: usize }

pub enum GlossaryVocabularyFitnessError {
    LiveUsage { token: String, path: String, line: u32 },
}

pub fn validate_glossary_vocabulary_fitness(
    occurrences: &[RetiredTermOccurrence],
    retired: &[String],
) -> Result<GlossaryVocabularyFitnessReport, GlossaryVocabularyFitnessError> {
    let r: std::collections::BTreeSet<&str> = retired.iter().map(|s| s.as_str()).collect();
    for o in occurrences {
        if !o.archived && r.contains(o.token.as_str()) {
            return Err(GlossaryVocabularyFitnessError::LiveUsage {
                token: o.token.clone(), path: o.path.clone(), line: o.line,
            });
        }
    }
    Ok(GlossaryVocabularyFitnessReport { occurrences_checked: occurrences.len() })
}
```
