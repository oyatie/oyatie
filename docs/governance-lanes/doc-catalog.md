---
doc_status: published
---

# Fitness Lane: doc-catalog

- status: Accepted
- date: 2026-05-12
- purpose: Verify every canonical doc has a row in `docs/CATALOG.md` with declared doc-class, owner, last-reviewed, and that no orphan doc-class rows exist.
- enforces: STANDARD/doc-catalog; AGENTS.md fitness-lane `governance-doc-catalog`.
- kernel_crate: `intelligence-catalog-kernel` (EXISTING; extend with verdict) — `CatalogRow { path, doc_class, owner_axis, last_reviewed }`, verdict `DocCatalogFitnessReport { rows_checked, docs_checked }`.
- runner_path: `tools/governance-doc-catalog`
- inputs: `docs/CATALOG.md`, `docs/**/*.md`.
- failure_modes:
  - canonical doc exists but no catalog row
  - catalog row references missing file
  - row has unknown doc_class value
- ci_invocation: `cargo run -p governance-doc-catalog`
- runtime_budget: 400 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct CatalogRow {
    pub path: String,           // data_class: INTERNAL_ONLY
    pub doc_class: String,      // data_class: INTERNAL_ONLY
    pub owner_axis: String,     // data_class: INTERNAL_ONLY
    pub last_reviewed: String,  // data_class: INTERNAL_ONLY (ISO date)
}

pub struct DocFile { pub path: String, pub is_canonical: bool } // data_class: INTERNAL_ONLY
pub struct DocCatalogFitnessReport { pub rows_checked: usize, pub docs_checked: usize }

pub enum DocCatalogFitnessError {
    UncatalogedDoc { path: String },
    OrphanRow { path: String },
    UnknownDocClass { path: String, doc_class: String },
    MissingOwner { path: String },
}

pub fn validate_doc_catalog_fitness(
    rows: &[CatalogRow],
    docs: &[DocFile],
    known_classes: &[String],
) -> Result<DocCatalogFitnessReport, DocCatalogFitnessError> {
    let known: std::collections::BTreeSet<&str> = known_classes.iter().map(|s| s.as_str()).collect();
    let row_paths: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.path.as_str()).collect();
    let doc_paths: std::collections::BTreeSet<&str> = docs.iter().map(|d| d.path.as_str()).collect();
    for r in rows {
        if !doc_paths.contains(r.path.as_str()) {
            return Err(DocCatalogFitnessError::OrphanRow { path: r.path.clone() });
        }
        if !known.contains(r.doc_class.as_str()) {
            return Err(DocCatalogFitnessError::UnknownDocClass { path: r.path.clone(), doc_class: r.doc_class.clone() });
        }
        if r.owner_axis.trim().is_empty() {
            return Err(DocCatalogFitnessError::MissingOwner { path: r.path.clone() });
        }
    }
    for d in docs {
        if d.is_canonical && !row_paths.contains(d.path.as_str()) {
            return Err(DocCatalogFitnessError::UncatalogedDoc { path: d.path.clone() });
        }
    }
    Ok(DocCatalogFitnessReport { rows_checked: rows.len(), docs_checked: docs.len() })
}
```
