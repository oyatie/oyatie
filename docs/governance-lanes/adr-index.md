---
doc_status: published
---

# Fitness Lane: adr-index

- status: Accepted
- date: 2026-05-12
- purpose: Verify `docs/ADR-INDEX.md` and `docs/machine-readable/decisions.json` are generated from every `docs/decisions/ADR-*.md` file.
- enforces: STANDARD/adr-index; Rust/Buck2 regenerator target `//:adr-index-regeneration-check`.
- kernel_crate: `libs/oya-check-adr-index` plus `tools/oya-adr-index-regenerator-app`.
- runner_path: `tools/oya-adr-index-regenerator-app`
- inputs: `docs/decisions/ADR-*.md`, `docs/ADR-INDEX.md`, `docs/machine-readable/decisions.json`.
- failure_modes:
  - generated Markdown differs from committed `docs/ADR-INDEX.md`
  - generated JSON differs from committed `docs/machine-readable/decisions.json`
  - ADR file cannot be parsed into the index record shape
- ci_invocation: `buck2 build //tools/oya-adr-index-regenerator-app:adr-index-regenerator-unit-tests //:adr-index-regeneration-check`
- runtime_budget: 250 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct AdrIndexRow {
    pub adr_id: String, // data_class: INTERNAL_ONLY
    pub path: String,   // data_class: INTERNAL_ONLY
    pub status: String, // data_class: INTERNAL_ONLY
}
pub struct AdrFile { pub adr_id: String, pub path: String } // data_class: INTERNAL_ONLY
pub struct AdrIndexFitnessReport { pub adrs_checked: usize }

pub enum AdrIndexFitnessError {
    InvalidRecord { adr_id: String, reason: String },
    MarkdownDrift,
    JsonDrift,
}

pub fn validate_adr_index_fitness(
    rows: &[AdrIndexRow],
    files: &[AdrFile],
) -> Result<AdrIndexFitnessReport, AdrIndexFitnessError> {
    // Production implementation parses the corpus, renders both artifacts, and
    // byte-compares them against the committed generated outputs.
    Ok(AdrIndexFitnessReport { adrs_checked: files.len() })
}
```
