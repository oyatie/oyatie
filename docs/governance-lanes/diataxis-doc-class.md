---
doc_status: published
---

# Fitness Lane: diataxis-doc-class

- status: Accepted
- date: 2026-05-12
- purpose: Verify every doc matches its declared Diataxis class shape (tutorial, how-to, reference, explanation).
- enforces: Directive 10 (MASTERPLAN) — docs match their declared class shape.
- kernel_crate: `governance-diataxis-doc-class-kernel` — `ClassifiedDoc { path, declared_class, has_required_sections }`, verdict `DiataxisDocClassFitnessReport { docs_checked }`.
- runner_path: `tools/governance-diataxis-doc-class`
- inputs: catalog rows with `doc_class`, doc parse, per-class section schema.
- failure_modes:
  - declared `tutorial` but no `Steps` section
  - `reference` doc contains opinion/narrative
  - declared class unknown
- ci_invocation: `cargo run -p governance-diataxis-doc-class`
- runtime_budget: 700 ms
- severity: MED
- kernel_sketch:
```rust
pub struct ClassifiedDoc {
    pub path: String,             // data_class: INTERNAL_ONLY
    pub declared_class: String,   // data_class: INTERNAL_ONLY
    pub sections: Vec<String>,    // data_class: INTERNAL_ONLY
}
pub struct DiataxisDocClassFitnessReport { pub docs_checked: usize }

pub enum DiataxisDocClassFitnessError {
    UnknownClass { path: String, class: String },
    MissingSection { path: String, class: String, section: String },
}

pub fn validate_diataxis_doc_class_fitness(
    docs: &[ClassifiedDoc],
    schema: &[(String, Vec<String>)], // (class, required sections)
) -> Result<DiataxisDocClassFitnessReport, DiataxisDocClassFitnessError> {
    let by_class: std::collections::BTreeMap<&str, &Vec<String>> =
        schema.iter().map(|(c, s)| (c.as_str(), s)).collect();
    for d in docs {
        let required = by_class.get(d.declared_class.as_str()).ok_or_else(|| {
            DiataxisDocClassFitnessError::UnknownClass {
                path: d.path.clone(), class: d.declared_class.clone(),
            }
        })?;
        for r in required.iter() {
            if !d.sections.contains(r) {
                return Err(DiataxisDocClassFitnessError::MissingSection {
                    path: d.path.clone(), class: d.declared_class.clone(), section: r.clone(),
                });
            }
        }
    }
    Ok(DiataxisDocClassFitnessReport { docs_checked: docs.len() })
}
```
