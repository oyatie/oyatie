/// Non-privacy operational labels used to classify system metadata.
///
/// These labels are intentionally outside the canonical privacy-program
/// [`DataClass`] taxonomy. They remain accepted through legacy `DataClass`
/// variants while append-only bootstrap ledgers are still readable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum OperationalDataClass {
    Audit,
    Secret,
}

/// Subject-status markers that early bootstrap records expressed as data
/// classes before `SubjectClass` became the orthogonal policy input.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SubjectDataMarker {
    Children,
}

/// Broader field-level classification used by [`Classified`].
///
/// Privacy decisions use [`PrivacyDataClass`], while operational metadata and
/// subject-status markers cross field-level seams through their own variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DataClassification {
    Privacy(PrivacyDataClass),
    Operational(OperationalDataClass),
    SubjectMarker(SubjectDataMarker),
}

impl DataClassification {
    pub const fn from_data_class(data_class: DataClass) -> Self {
        match data_class {
            DataClass::Audit => Self::Operational(OperationalDataClass::Audit),
            DataClass::Secret => Self::Operational(OperationalDataClass::Secret),
            DataClass::Children => Self::SubjectMarker(SubjectDataMarker::Children),
            privacy_class => Self::Privacy(PrivacyDataClass {
                data_class: privacy_class,
            }),
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Privacy(data_class) => data_class.label(),
            Self::Operational(OperationalDataClass::Audit) => "AUDIT",
            Self::Operational(OperationalDataClass::Secret) => "SECRET",
            Self::SubjectMarker(SubjectDataMarker::Children) => "CHILDREN",
        }
    }

    pub const fn privacy_data_class(self) -> Option<PrivacyDataClass> {
        match self {
            Self::Privacy(data_class) => Some(data_class),
            Self::Operational(_) | Self::SubjectMarker(_) => None,
        }
    }

    /// Return the legacy [`DataClass`] label used by append-only ledgers and
    /// existing audit/evidence call sites while operational and subject markers
    /// migrate to the broader [`DataClassification`] wrapper.
    pub const fn compatibility_data_class(self) -> DataClass {
        match self {
            Self::Privacy(data_class) => data_class.data_class(),
            Self::Operational(OperationalDataClass::Audit) => DataClass::Audit,
            Self::Operational(OperationalDataClass::Secret) => DataClass::Secret,
            Self::SubjectMarker(SubjectDataMarker::Children) => DataClass::Children,
        }
    }

    pub const fn normalized(self) -> Self {
        self
    }
}

impl From<DataClass> for DataClassification {
    fn from(data_class: DataClass) -> Self {
        Self::from_data_class(data_class)
    }
}

impl From<PrivacyDataClass> for DataClassification {
    fn from(data_class: PrivacyDataClass) -> Self {
        Self::Privacy(data_class)
    }
}

impl From<OperationalDataClass> for DataClassification {
    fn from(operational_class: OperationalDataClass) -> Self {
        Self::Operational(operational_class)
    }
}

impl From<SubjectDataMarker> for DataClassification {
    fn from(subject_marker: SubjectDataMarker) -> Self {
        Self::SubjectMarker(subject_marker)
    }
}
