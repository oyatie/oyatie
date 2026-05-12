//! Foundry data-class fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldIdentity {
    pub path: String,        // data_class: INTERNAL_ONLY
    pub struct_name: String, // data_class: INTERNAL_ONLY
    pub field_name: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelField {
    pub identity: FieldIdentity,         // data_class: INTERNAL_ONLY
    pub has_data_class_annotation: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyUnannotatedField {
    pub identity: FieldIdentity, // data_class: INTERNAL_ONLY
    pub rationale: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataClassFitnessReport {
    pub fields_checked: usize,            // data_class: INTERNAL_ONLY
    pub annotated_fields: usize,          // data_class: INTERNAL_ONLY
    pub legacy_unannotated_fields: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataClassFitnessError {
    EmptyFieldPath,
    EmptyStructName,
    EmptyFieldName,
    DuplicateField { field: FieldIdentity },
    UnknownUnannotatedField { field: FieldIdentity },
    DuplicateLegacyAllowance { field: FieldIdentity },
    StaleLegacyAllowance { field: FieldIdentity },
    EmptyLegacyRationale { field: FieldIdentity },
}

pub fn validate_data_class_fitness(
    fields: &[KernelField],
    legacy_allowances: &[LegacyUnannotatedField],
) -> Result<DataClassFitnessReport, DataClassFitnessError> {
    let mut all_fields = BTreeSet::new();
    let mut unannotated_fields = BTreeSet::new();
    let mut annotated_fields = 0;

    for field in fields {
        validate_identity(&field.identity)?;
        if !all_fields.insert(field.identity.clone()) {
            return Err(DataClassFitnessError::DuplicateField {
                field: field.identity.clone(),
            });
        }
        if field.has_data_class_annotation {
            annotated_fields += 1;
        } else {
            unannotated_fields.insert(field.identity.clone());
        }
    }

    let mut legacy_fields = BTreeSet::new();
    for allowance in legacy_allowances {
        validate_identity(&allowance.identity)?;
        if allowance.rationale.trim().is_empty() {
            return Err(DataClassFitnessError::EmptyLegacyRationale {
                field: allowance.identity.clone(),
            });
        }
        if !legacy_fields.insert(allowance.identity.clone()) {
            return Err(DataClassFitnessError::DuplicateLegacyAllowance {
                field: allowance.identity.clone(),
            });
        }
    }

    for field in &unannotated_fields {
        if !legacy_fields.contains(field) {
            return Err(DataClassFitnessError::UnknownUnannotatedField {
                field: field.clone(),
            });
        }
    }

    for field in &legacy_fields {
        if !unannotated_fields.contains(field) {
            return Err(DataClassFitnessError::StaleLegacyAllowance {
                field: field.clone(),
            });
        }
    }

    Ok(DataClassFitnessReport {
        fields_checked: fields.len(),
        annotated_fields,
        legacy_unannotated_fields: unannotated_fields.len(),
    })
}

fn validate_identity(identity: &FieldIdentity) -> Result<(), DataClassFitnessError> {
    if identity.path.trim().is_empty() {
        return Err(DataClassFitnessError::EmptyFieldPath);
    }
    if identity.struct_name.trim().is_empty() {
        return Err(DataClassFitnessError::EmptyStructName);
    }
    if identity.field_name.trim().is_empty() {
        return Err(DataClassFitnessError::EmptyFieldName);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_unannotated_kernel_field() {
        let field = kernel_field(
            "crates/example-kernel/src/lib.rs",
            "Example",
            "tenant_id",
            false,
        );
        let identity = field.identity.clone();

        assert_eq!(
            validate_data_class_fitness(std::slice::from_ref(&field), &[]),
            Err(DataClassFitnessError::UnknownUnannotatedField { field: identity })
        );
    }

    #[test]
    fn accepts_annotated_fields_and_tracked_legacy_fields() {
        let annotated = kernel_field("crates/example-kernel/src/lib.rs", "Example", "name", true);
        let legacy = kernel_field(
            "crates/example-kernel/src/lib.rs",
            "Example",
            "tenant_id",
            false,
        );

        assert_eq!(
            validate_data_class_fitness(
                &[annotated, legacy.clone()],
                &[LegacyUnannotatedField {
                    identity: legacy.identity,
                    rationale: "MFL-0008 bootstrap debt; new fields must be annotated".into(),
                }]
            ),
            Ok(DataClassFitnessReport {
                fields_checked: 2,
                annotated_fields: 1,
                legacy_unannotated_fields: 1,
            })
        );
    }

    #[test]
    fn rejects_stale_legacy_allowance() {
        let allowance = LegacyUnannotatedField {
            identity: field_identity("crates/example-kernel/src/lib.rs", "Example", "removed"),
            rationale: "MFL-0008 bootstrap debt; new fields must be annotated".into(),
        };
        let identity = allowance.identity.clone();

        assert_eq!(
            validate_data_class_fitness(&[], std::slice::from_ref(&allowance)),
            Err(DataClassFitnessError::StaleLegacyAllowance { field: identity })
        );
    }

    #[test]
    fn rejects_legacy_allowance_without_rationale() {
        let field = kernel_field(
            "crates/example-kernel/src/lib.rs",
            "Example",
            "tenant_id",
            false,
        );
        let identity = field.identity.clone();

        assert_eq!(
            validate_data_class_fitness(
                std::slice::from_ref(&field),
                &[LegacyUnannotatedField {
                    identity: field.identity.clone(),
                    rationale: " ".into(),
                }]
            ),
            Err(DataClassFitnessError::EmptyLegacyRationale { field: identity })
        );
    }

    fn kernel_field(
        path: &str,
        struct_name: &str,
        field_name: &str,
        has_data_class_annotation: bool,
    ) -> KernelField {
        KernelField {
            identity: field_identity(path, struct_name, field_name),
            has_data_class_annotation,
        }
    }

    fn field_identity(path: &str, struct_name: &str, field_name: &str) -> FieldIdentity {
        FieldIdentity {
            path: path.into(),
            struct_name: struct_name.into(),
            field_name: field_name.into(),
        }
    }
}
