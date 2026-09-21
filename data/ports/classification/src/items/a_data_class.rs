/// Data-class vocabulary owned by the Data Use Boundary.
///
/// The first bootstrap variants are retained as migration compatibility for
/// already-written code and local ledger records; they are not additional
/// privacy-program classes. New catalog/OpenAPI/telemetry surfaces should use
/// [`PRIVACY_PROGRAM_DATA_CLASS_LABELS`] and [`DataClass::label`], while file
/// ledgers keep using [`DataClass::pascal_label`] until the bootstrap records
/// can be rewritten under an explicit migration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DataClass {
    Public,
    InternalOnly,
    PiiIdentifying,
    /// Compatibility label for the privacy-program `PII_QUASI_IDENTIFIER` class.
    PiiSensitive,
    Phi,
    Pci,
    /// Compatibility label for `SENSITIVE_PIPA_ART23`.
    PipaArticle23,
    /// Transitional conservative marker until the full orthogonal
    /// `SubjectClass::Minor` model lands; treated as hard-denied for
    /// search/ads in this bootstrap slice.
    Children,
    /// Compatibility label for the generic `FINANCIAL` privacy class.
    Financial,
    /// Compatibility label for tenant-product behavioral usage.
    Usage,
    Secret,
    Audit,
    PiiQuasiIdentifier,
    FinancialRegulatedCredit,
    BehavioralTenantProduct,
    BehavioralAds,
    DeclaredPreference,
    SearchQuery,
    SensitivePipaArticle23,
}

/// Canonical privacy-program data class labels.
pub const PRIVACY_PROGRAM_DATA_CLASS_LABELS: [&str; 13] = [
    "INTERNAL_ONLY",
    "PHI",
    "PII_IDENTIFYING",
    "PII_QUASI_IDENTIFIER",
    "PCI",
    "FINANCIAL",
    "FINANCIAL_REGULATED_CREDIT",
    "BEHAVIORAL_TENANT_PRODUCT",
    "BEHAVIORAL_ADS",
    "DECLARED_PREFERENCE",
    "SEARCH_QUERY",
    "PUBLIC",
    "SENSITIVE_PIPA_ART23",
];

impl DataClass {
    /// Source-compatibility alias for pre-migration code paths; public labels use FINANCIAL_REGULATED_CREDIT.
    #[allow(non_upper_case_globals)]
    pub const FinancialCredit: Self = Self::FinancialRegulatedCredit;

    /// Stable label for catalog, OpenAPI, and telemetry surfaces.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Public => "PUBLIC",
            Self::InternalOnly => "INTERNAL_ONLY",
            Self::PiiIdentifying => "PII_IDENTIFYING",
            Self::PiiSensitive => "PII_SENSITIVE",
            Self::Phi => "PHI",
            Self::Pci => "PCI",
            Self::PipaArticle23 => "PIPA_ARTICLE_23",
            Self::Children => "CHILDREN",
            Self::Financial => "FINANCIAL",
            Self::Usage => "USAGE",
            Self::Secret => "SECRET",
            Self::Audit => "AUDIT",
            Self::PiiQuasiIdentifier => "PII_QUASI_IDENTIFIER",
            Self::FinancialRegulatedCredit => "FINANCIAL_REGULATED_CREDIT",
            Self::BehavioralTenantProduct => "BEHAVIORAL_TENANT_PRODUCT",
            Self::BehavioralAds => "BEHAVIORAL_ADS",
            Self::DeclaredPreference => "DECLARED_PREFERENCE",
            Self::SearchQuery => "SEARCH_QUERY",
            Self::SensitivePipaArticle23 => "SENSITIVE_PIPA_ART23",
        }
    }

    /// Historical PascalCase label used by existing file-ledger records.
    ///
    /// This is intentionally separate from [`Self::label`] so public/catalog
    /// surfaces can migrate to the privacy-program labels without rewriting
    /// append-only local ledgers.
    pub const fn pascal_label(self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::InternalOnly => "InternalOnly",
            Self::PiiIdentifying => "PiiIdentifying",
            Self::PiiSensitive => "PiiSensitive",
            Self::Phi => "Phi",
            Self::Pci => "Pci",
            Self::PipaArticle23 => "PipaArticle23",
            Self::Children => "Children",
            Self::Financial => "Financial",
            Self::Usage => "Usage",
            Self::Secret => "Secret",
            Self::Audit => "Audit",
            Self::PiiQuasiIdentifier => "PiiQuasiIdentifier",
            Self::FinancialRegulatedCredit => "FinancialRegulatedCredit",
            Self::BehavioralTenantProduct => "BehavioralTenantProduct",
            Self::BehavioralAds => "BehavioralAds",
            Self::DeclaredPreference => "DeclaredPreference",
            Self::SearchQuery => "SearchQuery",
            Self::SensitivePipaArticle23 => "SensitivePipaArticle23",
        }
    }

    /// Canonical privacy-program label when this class has a direct mapping.
    pub const fn privacy_program_label(self) -> Option<&'static str> {
        match self {
            Self::InternalOnly => Some("INTERNAL_ONLY"),
            Self::Phi => Some("PHI"),
            Self::PiiIdentifying => Some("PII_IDENTIFYING"),
            Self::PiiSensitive | Self::PiiQuasiIdentifier => Some("PII_QUASI_IDENTIFIER"),
            Self::Pci => Some("PCI"),
            Self::Financial => Some("FINANCIAL"),
            Self::FinancialRegulatedCredit => Some("FINANCIAL_REGULATED_CREDIT"),
            Self::Usage | Self::BehavioralTenantProduct => Some("BEHAVIORAL_TENANT_PRODUCT"),
            Self::BehavioralAds => Some("BEHAVIORAL_ADS"),
            Self::DeclaredPreference => Some("DECLARED_PREFERENCE"),
            Self::SearchQuery => Some("SEARCH_QUERY"),
            Self::Public => Some("PUBLIC"),
            Self::PipaArticle23 | Self::SensitivePipaArticle23 => Some("SENSITIVE_PIPA_ART23"),
            Self::Children | Self::Secret | Self::Audit => None,
        }
    }

    pub fn is_privacy_program_class(self) -> bool {
        self.privacy_program_label().is_some()
    }
}
