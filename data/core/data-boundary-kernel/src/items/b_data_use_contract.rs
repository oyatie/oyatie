#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DataUseAttributes {
    pub purpose: Purpose,                        // data_class: INTERNAL_ONLY
    pub data_classification: DataClassification, // data_class: INTERNAL_ONLY
    pub subject_class: SubjectClass,             // data_class: INTERNAL_ONLY
}

/// Compatibility input for callers that still carry raw `DataClass` labels.
///
/// Canonical data-use policy evaluation takes [`DataUseAttributes`] with a
/// typed [`DataClassification`]. This shape is reserved for ledger/bootstrap
/// replay seams that have not split operational and subject markers yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LegacyDataUseAttributes {
    pub purpose: Purpose,            // data_class: INTERNAL_ONLY
    pub data_class: DataClass,       // data_class: INTERNAL_ONLY
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY
}

/// Historical name for the canonical typed evaluator input.
pub type DataUseClassificationAttributes = DataUseAttributes;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DataUseDenialReason {
    HardDeniedDataClass,
    MinorSubjectAds,
}
