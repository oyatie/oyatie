/// Named predicate sets for [`DataClass`] membership tests.
///
/// Rather than duplicating `matches!(data_class, DataClass::Phi | ...)` across
/// every policy call-site, callers use [`DataClassMatcher`] variants to express
/// intent clearly. The variants correspond to the sets already defined by the
/// Cedar DUB policy fragment (`cedar/data_boundary.cedar`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DataClassMatcher {
    /// Matches exactly the HARD_DENY set: PHI, PCI, PIPA Article 23 /
    /// Sensitive PIPA Art 23, and Children.
    HardDenySet,
    /// Matches the regulated-financial set: Financial and FinancialRegulatedCredit.
    RegulatedFinancial,
    /// Matches any directly identifying PII variant.
    DirectPii,
    /// Matches any class that must not appear in a public search index.
    SearchIndexRestricted,
}

impl DataClassMatcher {
    /// Returns `true` if `data_class` is a member of this matcher's set.
    pub const fn matches(self, data_class: DataClass) -> bool {
        match self {
            Self::HardDenySet => matches!(
                data_class,
                DataClass::Phi
                    | DataClass::Pci
                    | DataClass::PipaArticle23
                    | DataClass::SensitivePipaArticle23
                    | DataClass::Children
            ),
            Self::RegulatedFinancial => matches!(
                data_class,
                DataClass::Financial | DataClass::FinancialRegulatedCredit
            ),
            Self::DirectPii => matches!(
                data_class,
                DataClass::PiiIdentifying | DataClass::PiiSensitive | DataClass::PiiQuasiIdentifier
            ),
            Self::SearchIndexRestricted => matches!(
                data_class,
                DataClass::Phi
                    | DataClass::Pci
                    | DataClass::PipaArticle23
                    | DataClass::SensitivePipaArticle23
                    | DataClass::Financial
                    | DataClass::FinancialRegulatedCredit
            ),
        }
    }
}
