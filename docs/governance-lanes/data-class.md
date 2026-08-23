---
doc_status: published
---

# Fitness Lane: data-class

- status: Accepted
- date: 2026-05-12
- purpose: Verify every public/struct field in `*-kernel` crates carries a `data_class:` annotation matching the registry.
- enforces: STANDARD/data-class-tagging; AGENTS.md fitness-lane `governance-data-class`.
- kernel_crate: `governance-data-class-kernel` — `FieldAnnotation { crate_id, type_name, field_name, data_class }`, verdict `DataClassFitnessReport { fields_checked, public_fields_checked }`.
- runner_path: `tools/governance-data-class`
- inputs: every `*-kernel/src/**/*.rs`, registry `docs/standards/data-class-registry.md`.
- failure_modes:
  - public field with no `data_class:` comment
  - `data_class:` value not in registry (e.g., `SECRET_TYPO`)
  - mixed casing (`internal_only` vs `INTERNAL_ONLY`)
- ci_invocation: `cargo run -p governance-data-class`
- runtime_budget: 600 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct FieldAnnotation {
    pub crate_id: String,    // data_class: INTERNAL_ONLY
    pub type_name: String,   // data_class: INTERNAL_ONLY
    pub field_name: String,  // data_class: INTERNAL_ONLY
    pub data_class: String,  // data_class: INTERNAL_ONLY
    pub visibility: String,  // data_class: INTERNAL_ONLY ("pub"|"crate"|"priv")
}

pub struct DataClassFitnessReport {
    pub fields_checked: usize,
    pub public_fields_checked: usize,
}

pub enum DataClassFitnessError {
    MissingAnnotation { crate_id: String, type_name: String, field_name: String },
    UnknownClass { crate_id: String, field_name: String, class: String },
    NonCanonicalCasing { class: String },
}

pub fn validate_data_class_fitness<R>(
    fields: &[FieldAnnotation],
    registry: R,
) -> Result<DataClassFitnessReport, DataClassFitnessError>
where
    R: IntoIterator,
    R::Item: AsRef<str>,
{
    let known = registry.into_iter().map(|s| s.as_ref().to_string()).collect::<std::collections::BTreeSet<_>>();
    let mut public_checked = 0;
    for f in fields {
        if f.visibility == "pub" {
            public_checked += 1;
            if f.data_class.trim().is_empty() {
                return Err(DataClassFitnessError::MissingAnnotation {
                    crate_id: f.crate_id.clone(),
                    type_name: f.type_name.clone(),
                    field_name: f.field_name.clone(),
                });
            }
        }
        if !f.data_class.is_empty() && f.data_class != f.data_class.to_uppercase() {
            return Err(DataClassFitnessError::NonCanonicalCasing { class: f.data_class.clone() });
        }
        if !f.data_class.is_empty() && !known.contains(&f.data_class) {
            return Err(DataClassFitnessError::UnknownClass {
                crate_id: f.crate_id.clone(),
                field_name: f.field_name.clone(),
                class: f.data_class.clone(),
            });
        }
    }
    Ok(DataClassFitnessReport { fields_checked: fields.len(), public_fields_checked: public_checked })
}
```
