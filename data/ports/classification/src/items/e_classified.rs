#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Classified<T> {
    pub value: T, // data_class: CARRIED_BY_CLASSIFIED_FIELD
    /// Field-level classification. Kept as `data_class` for source
    /// compatibility while the bootstrap code migrates operational labels out
    /// of the privacy [`DataClass`] taxonomy.
    pub data_class: DataClassification,
}

impl<T> Classified<T> {
    pub fn new(value: T, data_class: impl Into<DataClassification>) -> Self {
        Self {
            value,
            data_class: data_class.into(),
        }
    }
}
