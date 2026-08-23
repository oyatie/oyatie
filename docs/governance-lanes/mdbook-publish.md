---
doc_status: published
---

# Fitness Lane: mdbook-publish

- status: Accepted
- date: 2026-05-12
- purpose: Verify every canonical standard/template/checklist appears in the mdbook `SUMMARY.md` and that mdbook build is green.
- enforces: STANDARD/doc-publish.
- kernel_crate: `governance-mdbook-publish-kernel` — `SummaryEntry { doc_path }`, `BuildStatus { ok, broken_links }`, verdict `MdbookPublishFitnessReport { entries_checked }`.
- runner_path: `tools/governance-mdbook-publish`
- inputs: `book/SUMMARY.md`, mdbook build report, canonical-doc list.
- failure_modes:
  - canonical doc not in SUMMARY
  - SUMMARY row points to missing doc
  - mdbook build reports broken link
- ci_invocation: `cargo run -p governance-mdbook-publish`
- runtime_budget: 1100 ms
- severity: HIGH
- kernel_sketch:
```rust
pub struct SummaryEntry { pub doc_path: String } // data_class: INTERNAL_ONLY
pub struct BuildStatus { pub ok: bool, pub broken_links: Vec<String> } // data_class: INTERNAL_ONLY
pub struct MdbookPublishFitnessReport { pub entries_checked: usize }

pub enum MdbookPublishFitnessError {
    UnpublishedCanonical { doc_path: String },
    OrphanSummaryRow { doc_path: String },
    BrokenLink { link: String },
}

pub fn validate_mdbook_publish_fitness(
    entries: &[SummaryEntry],
    canonical_docs: &[String],
    build: &BuildStatus,
) -> Result<MdbookPublishFitnessReport, MdbookPublishFitnessError> {
    let entry_set: std::collections::BTreeSet<&str> = entries.iter().map(|e| e.doc_path.as_str()).collect();
    let canonical: std::collections::BTreeSet<&str> = canonical_docs.iter().map(|s| s.as_str()).collect();
    for c in canonical_docs {
        if !entry_set.contains(c.as_str()) {
            return Err(MdbookPublishFitnessError::UnpublishedCanonical { doc_path: c.clone() });
        }
    }
    for e in entries {
        if !canonical.contains(e.doc_path.as_str()) {
            return Err(MdbookPublishFitnessError::OrphanSummaryRow { doc_path: e.doc_path.clone() });
        }
    }
    if !build.ok {
        if let Some(l) = build.broken_links.first() {
            return Err(MdbookPublishFitnessError::BrokenLink { link: l.clone() });
        }
    }
    Ok(MdbookPublishFitnessReport { entries_checked: entries.len() })
}
```
