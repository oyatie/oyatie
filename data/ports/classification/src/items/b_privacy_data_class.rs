/// A data class that is guaranteed to belong to the privacy-program taxonomy.
///
/// Capability declarations, consent scopes, and public schema annotations use
/// privacy-program data classes. Operational labels such as `AUDIT` / `SECRET`
/// and subject markers such as `CHILDREN` must cross those seams through their
/// dedicated typed enums instead of being smuggled through [`DataClass`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PrivacyDataClass {
    data_class: DataClass, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NonPrivacyDataClass {
    pub data_class: DataClass, // data_class: INTERNAL_ONLY
}

impl PrivacyDataClass {
    pub fn new(data_class: DataClass) -> Result<Self, NonPrivacyDataClass> {
        if data_class.is_privacy_program_class() {
            Ok(Self { data_class })
        } else {
            Err(NonPrivacyDataClass { data_class })
        }
    }

    /// Infallible constructor for the `INTERNAL_ONLY` privacy-program data
    /// class.
    ///
    /// `INTERNAL_ONLY` is a statically known privacy-program member (see
    /// [`PRIVACY_PROGRAM_DATA_CLASS_LABELS`]), so this constructor returns
    /// [`Self`] directly without going through the fallible
    /// [`PrivacyDataClass::new`] path. Use this at every site that previously
    /// wrote `PrivacyDataClass::new(DataClass::InternalOnly).expect(...)` to
    /// satisfy the ADR-0083 Tier 1 ban on `.expect()` / `.unwrap()` in
    /// production code without `#[allow]` shortcuts.
    ///
    /// Naming justification (v4 BNF + 12-layer-enum):
    /// `data-boundary-kernel` is the canonical `kernel` layer that owns
    /// the `PrivacyDataClass` value type; an infallible constructor belongs
    /// here (not in a `domain` or `usecase` layer) because every caller in
    /// `*-domain` crates depends on the kernel for the type itself. The
    /// `internal_only` suffix matches the existing taxonomy label
    /// (`INTERNAL_ONLY`) and the 12-layer-enum `kernel` slot.
    pub const fn internal_only() -> Self {
        Self {
            data_class: DataClass::InternalOnly,
        }
    }

    /// Infallible constructor for the `PII_IDENTIFYING` privacy-program data
    /// class.
    ///
    /// Sibling of [`Self::internal_only`]; `PII_IDENTIFYING` is a statically
    /// known privacy-program member (see [`PRIVACY_PROGRAM_DATA_CLASS_LABELS`]),
    /// so this constructor returns [`Self`] directly without going through the
    /// fallible [`PrivacyDataClass::new`] path. Use this at every site that
    /// previously wrote `PrivacyDataClass::new(DataClass::PiiIdentifying)
    /// .expect(...)` to satisfy the ADR-0083 Tier 1 ban on `.expect()` /
    /// `.unwrap()` in production code without `#[allow]` shortcuts.
    ///
    /// Naming justification (v4 BNF + 12-layer-enum): identical to
    /// [`Self::internal_only`] — kernel-layer infallible constructor named
    /// after the privacy-program label (`PII_IDENTIFYING`).
    pub const fn pii_identifying() -> Self {
        Self {
            data_class: DataClass::PiiIdentifying,
        }
    }

    /// Infallible constructor for the `PII_QUASI_IDENTIFIER` privacy-program
    /// data class. Sibling of [`Self::pii_identifying`].
    pub const fn pii_quasi_identifier() -> Self {
        Self {
            data_class: DataClass::PiiQuasiIdentifier,
        }
    }

    pub const fn data_class(self) -> DataClass {
        self.data_class
    }

    pub const fn label(self) -> &'static str {
        self.data_class.label()
    }
}

impl TryFrom<DataClass> for PrivacyDataClass {
    type Error = NonPrivacyDataClass;

    fn try_from(data_class: DataClass) -> Result<Self, Self::Error> {
        Self::new(data_class)
    }
}

pub fn privacy_data_classes_from(
    data_classes: &[DataClass],
) -> Result<Vec<PrivacyDataClass>, NonPrivacyDataClass> {
    data_classes
        .iter()
        .copied()
        .map(PrivacyDataClass::try_from)
        .collect()
}

pub fn data_classes_from_privacy_data_classes(data_classes: &[PrivacyDataClass]) -> Vec<DataClass> {
    data_classes
        .iter()
        .map(|data_class| data_class.data_class())
        .collect()
}

pub fn most_restrictive_privacy_data_class(data_classes: &[PrivacyDataClass]) -> Option<DataClass> {
    data_classes
        .iter()
        .map(|data_class| data_class.data_class())
        .max()
}
