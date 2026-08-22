---
doc_status: published
---

# Fitness Lane: mistakes-ledger-cite

- status: Accepted
- date: 2026-05-12
- purpose: Verify every "do-not-repeat" style guardrail in canonical docs cites a row in `docs/MISTAKES-LEDGER.md`.
- enforces: STANDARD/mistakes-ledger; AGENTS.md fitness-lane `governance-mistakes-ledger-cite`.
- kernel_crate: `governance-mistakes-ledger-kernel` — `MistakeCitation { path, line, mistake_id }`, `LedgerRow { mistake_id, status }`, verdict `MistakesLedgerFitnessReport { citations_checked, ledger_rows }`.
- runner_path: `tools/governance-mistakes-ledger-cite`
- inputs: `docs/**/*.md`, `docs/MISTAKES-LEDGER.md`.
- failure_modes:
  - "do not repeat" sentence without a ledger id
  - cited id not in ledger
  - ledger row marked `retracted` still cited as live
- ci_invocation: `cargo run -p governance-mistakes-ledger-cite`
- runtime_budget: 600 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct MistakeCitation {
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub line: u32,                     // data_class: INTERNAL_ONLY
    pub mistake_id: Option<String>,    // data_class: INTERNAL_ONLY
}
pub struct LedgerRow {
    pub mistake_id: String,            // data_class: INTERNAL_ONLY
    pub status: String,                // data_class: INTERNAL_ONLY
}
pub struct MistakesLedgerFitnessReport { pub citations_checked: usize, pub ledger_rows: usize }

pub enum MistakesLedgerFitnessError {
    MissingCitation { path: String, line: u32 },
    UnknownLedgerId { path: String, mistake_id: String },
    RetractedLedgerCited { path: String, mistake_id: String },
}

pub fn validate_mistakes_ledger_fitness(
    citations: &[MistakeCitation],
    rows: &[LedgerRow],
) -> Result<MistakesLedgerFitnessReport, MistakesLedgerFitnessError> {
    let by_id: std::collections::BTreeMap<&str, &str> =
        rows.iter().map(|r| (r.mistake_id.as_str(), r.status.as_str())).collect();
    for c in citations {
        let id = c.mistake_id.as_ref().ok_or_else(|| MistakesLedgerFitnessError::MissingCitation {
            path: c.path.clone(), line: c.line,
        })?;
        match by_id.get(id.as_str()) {
            None => return Err(MistakesLedgerFitnessError::UnknownLedgerId {
                path: c.path.clone(), mistake_id: id.clone(),
            }),
            Some(s) if *s == "retracted" => return Err(MistakesLedgerFitnessError::RetractedLedgerCited {
                path: c.path.clone(), mistake_id: id.clone(),
            }),
            Some(_) => {}
        }
    }
    Ok(MistakesLedgerFitnessReport { citations_checked: citations.len(), ledger_rows: rows.len() })
}
```
