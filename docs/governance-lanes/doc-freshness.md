---
doc_status: published
---

# Fitness Lane: doc-freshness

- status: Accepted
- date: 2026-05-12
- purpose: Verify every doc respects its doc-class staleness budget (e.g., runbook <= 90d, standard <= 180d, ADR no budget).
- enforces: Directive 10 (MASTERPLAN) — staleness budget per doc-class.
- kernel_crate: `governance-doc-freshness-kernel` — `DocFreshness { path, doc_class, age_days }`, verdict `DocFreshnessFitnessReport { docs_checked }`.
- runner_path: `tools/governance-doc-freshness`
- inputs: catalog rows, git last-touched timestamp.
- failure_modes:
  - runbook last touched 200 days ago
  - standard older than 180d with no `reviewed:` extension
  - unknown doc-class (=> no budget to check against)
- ci_invocation: `cargo run -p governance-doc-freshness`
- runtime_budget: 700 ms
- severity: MED
- kernel_sketch:
```rust
pub struct DocFreshness {
    pub path: String,       // data_class: INTERNAL_ONLY
    pub doc_class: String,  // data_class: INTERNAL_ONLY
    pub age_days: u32,      // data_class: INTERNAL_ONLY
}
pub struct DocFreshnessFitnessReport { pub docs_checked: usize }

pub enum DocFreshnessFitnessError {
    Stale { path: String, doc_class: String, age_days: u32, budget: u32 },
    UnknownDocClass { path: String, doc_class: String },
}

pub fn validate_doc_freshness_fitness(
    docs: &[DocFreshness],
    budgets: &[(String, u32)], // (doc_class, days)
) -> Result<DocFreshnessFitnessReport, DocFreshnessFitnessError> {
    let by_class: std::collections::BTreeMap<&str, u32> =
        budgets.iter().map(|(c, b)| (c.as_str(), *b)).collect();
    for d in docs {
        let budget = by_class.get(d.doc_class.as_str())
            .ok_or_else(|| DocFreshnessFitnessError::UnknownDocClass {
                path: d.path.clone(), doc_class: d.doc_class.clone(),
            })?;
        if d.age_days > *budget {
            return Err(DocFreshnessFitnessError::Stale {
                path: d.path.clone(), doc_class: d.doc_class.clone(),
                age_days: d.age_days, budget: *budget,
            });
        }
    }
    Ok(DocFreshnessFitnessReport { docs_checked: docs.len() })
}
```
