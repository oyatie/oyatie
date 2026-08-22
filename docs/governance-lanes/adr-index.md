---
doc_status: published
---

# Fitness Lane: adr-index

- status: Accepted
- date: 2026-05-12
- purpose: Verify `docs/decisions/INDEX.md` enumerates every ADR file and rejects gaps in the numbering sequence.
- enforces: STANDARD/adr-index; existing crate `governance-adr-index-kernel` (EXISTING; extend with verdict).
- kernel_crate: `governance-adr-index-kernel` (EXISTING) — `AdrIndexRow { adr_id, path, status }`, verdict `AdrIndexFitnessReport { adrs_checked }`.
- runner_path: `tools/governance-adr-index`
- inputs: `docs/decisions/INDEX.md`, `docs/decisions/ADR-*.md`.
- failure_modes:
  - ADR file present but no index row
  - index row points at missing file
  - numbering gap (ADR-0050 -> ADR-0052)
- ci_invocation: `cargo run -p governance-adr-index`
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
    UnindexedAdr { adr_id: String, path: String },
    OrphanIndexRow { adr_id: String, path: String },
    NumberingGap { previous: String, next: String },
}

pub fn validate_adr_index_fitness(
    rows: &[AdrIndexRow],
    files: &[AdrFile],
) -> Result<AdrIndexFitnessReport, AdrIndexFitnessError> {
    let row_ids: std::collections::BTreeMap<&str, &AdrIndexRow> = rows.iter().map(|r| (r.adr_id.as_str(), r)).collect();
    let file_ids: std::collections::BTreeSet<&str> = files.iter().map(|f| f.adr_id.as_str()).collect();
    for f in files {
        if !row_ids.contains_key(f.adr_id.as_str()) {
            return Err(AdrIndexFitnessError::UnindexedAdr { adr_id: f.adr_id.clone(), path: f.path.clone() });
        }
    }
    for r in rows {
        if !file_ids.contains(r.adr_id.as_str()) {
            return Err(AdrIndexFitnessError::OrphanIndexRow { adr_id: r.adr_id.clone(), path: r.path.clone() });
        }
    }
    let mut sorted: Vec<&str> = file_ids.iter().copied().collect();
    sorted.sort();
    for w in sorted.windows(2) {
        let a: u32 = w[0].trim_start_matches("ADR-").parse().unwrap_or(0);
        let b: u32 = w[1].trim_start_matches("ADR-").parse().unwrap_or(0);
        if b != a + 1 {
            return Err(AdrIndexFitnessError::NumberingGap { previous: w[0].to_string(), next: w[1].to_string() });
        }
    }
    Ok(AdrIndexFitnessReport { adrs_checked: files.len() })
}
```
