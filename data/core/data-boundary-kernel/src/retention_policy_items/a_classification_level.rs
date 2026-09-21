/// Four-level operational sensitivity tier derived from the [`DataClass`]
/// vocabulary.
///
/// The ordering is `Unrestricted < Restricted < Sensitive < Critical`, which
/// matches the tightest-first purge-delay budget applied by [`RetentionPolicy`].
/// Callers that only need to know "how regulated is this field?" should use
/// [`ClassificationLevel`] instead of inspecting the full [`DataClass`] enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ClassificationLevel {
    /// Non-regulated, publicly shareable data (e.g. [`DataClass::Public`]).
    Unrestricted,
    /// Internal-only data that must not leave the tenant boundary but is not
    /// directly regulated (e.g. [`DataClass::InternalOnly`],
    /// [`DataClass::Usage`]).
    Restricted,
    /// Directly regulated or quasi-identifying data (e.g. PII, financial,
    /// behavioral ad-targeting data).
    Sensitive,
    /// Hardest-regulated classes whose misuse triggers a HARD_DENY
    /// (PHI, PCI, PIPA Article 23, children's data).
    Critical,
}

impl ClassificationLevel {
    /// Derive the operational sensitivity level from a raw [`DataClass`].
    ///
    /// This mapping is intentionally conservative: ambiguous bootstrap
    /// variants (e.g. [`DataClass::Usage`]) are placed at the lower bound of
    /// the range they could possibly occupy.
    pub const fn from_data_class(data_class: DataClass) -> Self {
        match data_class {
            DataClass::Public => Self::Unrestricted,
            DataClass::InternalOnly | DataClass::Usage | DataClass::Audit | DataClass::Secret => {
                Self::Restricted
            }
            DataClass::PiiIdentifying
            | DataClass::PiiSensitive
            | DataClass::PiiQuasiIdentifier
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
            | DataClass::BehavioralTenantProduct
            | DataClass::BehavioralAds
            | DataClass::DeclaredPreference
            | DataClass::SearchQuery => Self::Sensitive,
            DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Children => Self::Critical,
        }
    }

    /// Stable wire label used by telemetry and catalog surfaces.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unrestricted => "UNRESTRICTED",
            Self::Restricted => "RESTRICTED",
            Self::Sensitive => "SENSITIVE",
            Self::Critical => "CRITICAL",
        }
    }

    /// Whether this level requires HARD_DENY treatment on regulated operations.
    pub const fn is_hard_deny_tier(self) -> bool {
        matches!(self, Self::Critical)
    }
}
