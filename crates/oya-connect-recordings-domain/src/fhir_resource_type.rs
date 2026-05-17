//! FHIR R5 resource-type discriminant.
//!
//! Per M02-P16 (records merge-variant delta 1): backport of the `FhirResourceType`
//! discriminant from the planned `oya-records-fhir-kernel/src/types.rs`.
//! Placed here because `oya-connect-recordings-domain` is the live crate that owns
//! archive-record domain types; the full records-fhir kernel crate is deferred to a
//! later scaffold phase per execution_variant=merge-into-existing-crates
//! (user-directive-option-2).
//!
//! References: ADR-0016 (clinical canonical), ADR-0056 v4.1, ADR-0008 (DUB/PHI).

#![forbid(unsafe_code)]

/// The nine FHIR R5 resource types supported in the M02 records substrate.
///
/// All nine types carry PHI classification per ADR-0008 §"clinical records".
/// The ontology bridge uses `ontology_type()` as the `object_type` string when
/// storing resources as Ontology objects (ADR-0106).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FhirResourceType {
    AllergyIntolerance,
    Condition,
    DiagnosticReport,
    Encounter,
    Immunization,
    Medication,
    MedicationRequest,
    Observation,
    Procedure,
}

impl FhirResourceType {
    /// Returns the Ontology `object_type` string for this resource.
    ///
    /// Format: `"records.<VariantName>"` — e.g. `"records.Encounter"`.
    pub fn ontology_type(self) -> String {
        let name = match self {
            Self::AllergyIntolerance => "AllergyIntolerance",
            Self::Condition => "Condition",
            Self::DiagnosticReport => "DiagnosticReport",
            Self::Encounter => "Encounter",
            Self::Immunization => "Immunization",
            Self::Medication => "Medication",
            Self::MedicationRequest => "MedicationRequest",
            Self::Observation => "Observation",
            Self::Procedure => "Procedure",
        };
        format!("records.{name}")
    }

    /// All FHIR resource types in this kernel are PHI per ADR-0008.
    pub fn data_class(self) -> &'static str {
        "Phi"
    }

    /// Returns every variant in declaration order (alphabetical / Ord order).
    pub fn all() -> &'static [FhirResourceType] {
        &[
            Self::AllergyIntolerance,
            Self::Condition,
            Self::DiagnosticReport,
            Self::Encounter,
            Self::Immunization,
            Self::Medication,
            Self::MedicationRequest,
            Self::Observation,
            Self::Procedure,
        ]
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn ontology_type_returns_records_prefix() {
        assert_eq!(
            FhirResourceType::Encounter.ontology_type(),
            "records.Encounter"
        );
        assert_eq!(
            FhirResourceType::AllergyIntolerance.ontology_type(),
            "records.AllergyIntolerance"
        );
        assert_eq!(
            FhirResourceType::MedicationRequest.ontology_type(),
            "records.MedicationRequest"
        );
    }

    #[test]
    fn all_nine_types_return_phi_data_class() {
        for rt in FhirResourceType::all() {
            assert_eq!(rt.data_class(), "Phi", "{rt:?} must be PHI");
        }
    }

    #[test]
    fn all_returns_nine_variants() {
        assert_eq!(FhirResourceType::all().len(), 9);
    }

    #[test]
    fn variants_are_ord_sorted_alphabetically() {
        let all = FhirResourceType::all();
        for window in all.windows(2) {
            assert!(
                window[0] < window[1],
                "{:?} should be < {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn clone_and_copy_are_consistent() {
        let rt = FhirResourceType::Observation;
        let copied = rt;
        assert_eq!(rt, copied);
        assert_eq!(rt.clone(), copied);
    }

    #[test]
    fn ontology_type_covers_all_nine_variants() {
        let prefixed: Vec<String> = FhirResourceType::all()
            .iter()
            .map(|rt| rt.ontology_type())
            .collect();
        for s in &prefixed {
            assert!(s.starts_with("records."), "{s} must start with 'records.'");
            assert!(s.len() > "records.".len(), "{s} must have a suffix");
        }
        // All distinct — use a HashSet so non-adjacent duplicates are caught.
        let unique: std::collections::HashSet<&String> = prefixed.iter().collect();
        assert_eq!(
            prefixed.len(),
            unique.len(),
            "ontology_type values must be unique"
        );
    }
}
