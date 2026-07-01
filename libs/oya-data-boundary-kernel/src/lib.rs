//! Data Use Boundary kernel.
//!
//! Pure value types for classifying fields and checking purpose-bound use.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod policy_gate;
pub mod retention_policy;

pub use retention_policy::{ClassificationLevel, DataClassMatcher, PurgeAction, RetentionPolicy};

use std::collections::BTreeSet;

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
    /// `oya-data-boundary-kernel` is the canonical `kernel` layer that owns
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

pub fn parse_data_class_pascal_label(label: &str) -> Option<DataClass> {
    match label.trim() {
        "Public" => Some(DataClass::Public),
        "InternalOnly" => Some(DataClass::InternalOnly),
        "PiiIdentifying" => Some(DataClass::PiiIdentifying),
        "PiiSensitive" => Some(DataClass::PiiSensitive),
        "Phi" => Some(DataClass::Phi),
        "Pci" => Some(DataClass::Pci),
        "PipaArticle23" => Some(DataClass::PipaArticle23),
        "Children" => Some(DataClass::Children),
        "Financial" => Some(DataClass::Financial),
        "Usage" => Some(DataClass::Usage),
        "Secret" => Some(DataClass::Secret),
        "Audit" => Some(DataClass::Audit),
        "PiiQuasiIdentifier" => Some(DataClass::PiiQuasiIdentifier),
        "FinancialRegulatedCredit" => Some(DataClass::FinancialRegulatedCredit),
        "BehavioralTenantProduct" => Some(DataClass::BehavioralTenantProduct),
        "BehavioralAds" => Some(DataClass::BehavioralAds),
        "DeclaredPreference" => Some(DataClass::DeclaredPreference),
        "SearchQuery" => Some(DataClass::SearchQuery),
        "SensitivePipaArticle23" => Some(DataClass::SensitivePipaArticle23),
        _ => None,
    }
}

pub fn parse_operational_data_class_label(label: &str) -> Option<OperationalDataClass> {
    match label.trim() {
        "AUDIT" => Some(OperationalDataClass::Audit),
        "SECRET" => Some(OperationalDataClass::Secret),
        _ => None,
    }
}

pub fn parse_subject_data_marker_label(label: &str) -> Option<SubjectDataMarker> {
    match label.trim() {
        "CHILDREN" => Some(SubjectDataMarker::Children),
        _ => None,
    }
}

/// Parse a privacy data-class label from catalog/OpenAPI surfaces.
///
/// Privacy surfaces are strict about operational/subject labels: `AUDIT`,
/// `SECRET`, and `CHILDREN` are accepted only by the explicitly named legacy
/// PascalCase ledger parser or the operational/subject marker parsers.
pub fn parse_data_class_label(label: &str) -> Option<DataClass> {
    match label.trim() {
        "PUBLIC" => Some(DataClass::Public),
        "INTERNAL_ONLY" => Some(DataClass::InternalOnly),
        "PII_IDENTIFYING" => Some(DataClass::PiiIdentifying),
        "PII_SENSITIVE" => Some(DataClass::PiiSensitive),
        "PII_QUASI_IDENTIFIER" => Some(DataClass::PiiQuasiIdentifier),
        "PHI" => Some(DataClass::Phi),
        "PCI" => Some(DataClass::Pci),
        "PIPA_ARTICLE_23" | "PIPA_ARTICLE23" => Some(DataClass::PipaArticle23),
        "SENSITIVE_PIPA_ART23" => Some(DataClass::SensitivePipaArticle23),
        "FINANCIAL" => Some(DataClass::Financial),
        "FINANCIAL_REGULATED_CREDIT" => Some(DataClass::FinancialRegulatedCredit),
        "USAGE" => Some(DataClass::Usage),
        "BEHAVIORAL_TENANT_PRODUCT" => Some(DataClass::BehavioralTenantProduct),
        "BEHAVIORAL_ADS" => Some(DataClass::BehavioralAds),
        "DECLARED_PREFERENCE" => Some(DataClass::DeclaredPreference),
        "SEARCH_QUERY" => Some(DataClass::SearchQuery),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Purpose {
    CoreService,
    CapabilityInvocation,
    SearchIndex,
    AdsTargeting,
    Analytics,
    Support,
    TenantAnalyticsFirstParty,
    CrossTenantAggregateAnonymous,
    PersonalizationInProduct,
    SearchIndexPrivate,
    SearchIndexPublic,
    AdTargetingDeclared,
    AdTargetingBehavioral,
    ModelTrainingOya,
    ModelTrainingThirdParty,
}

impl Purpose {
    /// Historical PascalCase label used by existing file-ledger records.
    pub const fn pascal_label(self) -> &'static str {
        match self {
            Self::CoreService => "CoreService",
            Self::CapabilityInvocation => "CapabilityInvocation",
            Self::SearchIndex => "SearchIndex",
            Self::AdsTargeting => "AdsTargeting",
            Self::Analytics => "Analytics",
            Self::Support => "Support",
            Self::TenantAnalyticsFirstParty => "TenantAnalyticsFirstParty",
            Self::CrossTenantAggregateAnonymous => "CrossTenantAggregateAnonymous",
            Self::PersonalizationInProduct => "PersonalizationInProduct",
            Self::SearchIndexPrivate => "SearchIndexPrivate",
            Self::SearchIndexPublic => "SearchIndexPublic",
            Self::AdTargetingDeclared => "AdTargetingDeclared",
            Self::AdTargetingBehavioral => "AdTargetingBehavioral",
            Self::ModelTrainingOya => "ModelTrainingOya",
            Self::ModelTrainingThirdParty => "ModelTrainingThirdParty",
        }
    }
}

pub fn parse_purpose_pascal_label(label: &str) -> Option<Purpose> {
    match label.trim() {
        "CoreService" => Some(Purpose::CoreService),
        "CapabilityInvocation" => Some(Purpose::CapabilityInvocation),
        "SearchIndex" => Some(Purpose::SearchIndex),
        "AdsTargeting" => Some(Purpose::AdsTargeting),
        "Analytics" => Some(Purpose::Analytics),
        "Support" => Some(Purpose::Support),
        "TenantAnalyticsFirstParty" => Some(Purpose::TenantAnalyticsFirstParty),
        "CrossTenantAggregateAnonymous" => Some(Purpose::CrossTenantAggregateAnonymous),
        "PersonalizationInProduct" => Some(Purpose::PersonalizationInProduct),
        "SearchIndexPrivate" => Some(Purpose::SearchIndexPrivate),
        "SearchIndexPublic" => Some(Purpose::SearchIndexPublic),
        "AdTargetingDeclared" => Some(Purpose::AdTargetingDeclared),
        "AdTargetingBehavioral" => Some(Purpose::AdTargetingBehavioral),
        "ModelTrainingOya" => Some(Purpose::ModelTrainingOya),
        "ModelTrainingThirdParty" => Some(Purpose::ModelTrainingThirdParty),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AgeBand {
    Under13,
    Under14,
    Under16,
    Under19,
    UnknownMinor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SubjectClass {
    Adult,
    Minor { age_band: AgeBand },
    Elderly,
    Vulnerable,
    Authority,
}

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsentScope {
    grants: BTreeSet<(Purpose, PrivacyDataClass)>,
}

impl ConsentScope {
    /// Record a purpose-bound privacy-program grant.
    pub fn allow_privacy_data_class(
        mut self,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> Self {
        self.grants.insert((purpose, data_class));
        self
    }

    /// Record a purpose-bound privacy-program grant.
    pub fn allow(self, purpose: Purpose, data_class: PrivacyDataClass) -> Self {
        self.allow_privacy_data_class(purpose, data_class)
    }

    /// Compatibility constructor for bootstrap/import seams that still carry
    /// raw `DataClass` labels. Canonical consent grants take
    /// `PrivacyDataClass`; this path fails closed for operational markers and
    /// subject markers by returning [`NonPrivacyDataClass`].
    pub fn try_allow_legacy_data_class(
        self,
        purpose: Purpose,
        data_class: DataClass,
    ) -> Result<Self, NonPrivacyDataClass> {
        let data_class = PrivacyDataClass::try_from(data_class)?;
        Ok(self.allow_privacy_data_class(purpose, data_class))
    }

    pub fn allows_privacy_data_class(
        &self,
        purpose: Purpose,
        data_class: PrivacyDataClass,
    ) -> bool {
        if is_hard_denied_classification(purpose, DataClassification::Privacy(data_class)) {
            return false;
        }
        self.grants.contains(&(purpose, data_class))
    }

    pub fn allows(&self, purpose: Purpose, data_class: PrivacyDataClass) -> bool {
        self.allows_privacy_data_class(purpose, data_class)
    }

    pub fn allows_legacy_data_class(&self, purpose: Purpose, data_class: DataClass) -> bool {
        let Ok(data_class) = PrivacyDataClass::try_from(data_class) else {
            return false;
        };
        self.allows_privacy_data_class(purpose, data_class)
    }

    pub fn allows_all(&self, purpose: Purpose, classes: &[PrivacyDataClass]) -> bool {
        classes.iter().all(|class| self.allows(purpose, *class))
    }

    pub fn allows_all_legacy_data_classes(&self, purpose: Purpose, classes: &[DataClass]) -> bool {
        classes
            .iter()
            .all(|class| self.allows_legacy_data_class(purpose, *class))
    }

    pub fn allows_classification(
        &self,
        purpose: Purpose,
        classification: impl Into<DataClassification>,
    ) -> bool {
        let classification = normalize_classification(classification.into());
        if is_hard_denied_classification(purpose, classification) {
            return false;
        }
        classification
            .privacy_data_class()
            .is_some_and(|data_class| self.allows_privacy_data_class(purpose, data_class))
    }

    pub fn allows_all_classifications(
        &self,
        purpose: Purpose,
        classifications: &[DataClassification],
    ) -> bool {
        classifications
            .iter()
            .all(|classification| self.allows_classification(purpose, *classification))
    }
}

pub fn is_hard_denied(purpose: Purpose, data_class: DataClass) -> bool {
    is_hard_denied_classification(purpose, data_class)
}

pub fn is_hard_denied_classification(
    purpose: Purpose,
    classification: impl Into<DataClassification>,
) -> bool {
    match normalize_classification(classification.into()) {
        DataClassification::Privacy(data_class) => {
            is_hard_denied_privacy_class(purpose, data_class.data_class())
        }
        DataClassification::Operational(OperationalDataClass::Audit) => {
            is_audit_marker_hard_denied(purpose)
        }
        DataClassification::Operational(OperationalDataClass::Secret) => {
            is_secret_marker_hard_denied(purpose)
        }
        DataClassification::SubjectMarker(SubjectDataMarker::Children) => {
            is_child_subject_marker_hard_denied(purpose)
        }
    }
}

fn normalize_classification(classification: DataClassification) -> DataClassification {
    classification
}

fn is_hard_denied_privacy_class(purpose: Purpose, data_class: DataClass) -> bool {
    match purpose {
        Purpose::SearchIndex | Purpose::SearchIndexPrivate => {
            is_search_index_privacy_hard_denied(data_class)
        }
        Purpose::SearchIndexPublic => !matches!(data_class, DataClass::Public),
        Purpose::AdsTargeting => is_ads_targeting_privacy_hard_denied(data_class),
        Purpose::AdTargetingDeclared => !matches!(
            data_class,
            DataClass::DeclaredPreference | DataClass::Public
        ),
        Purpose::AdTargetingBehavioral => !matches!(
            data_class,
            DataClass::BehavioralAds | DataClass::DeclaredPreference | DataClass::Public
        ),
        Purpose::Analytics | Purpose::TenantAnalyticsFirstParty => {
            matches!(data_class, DataClass::Pci)
        }
        Purpose::CrossTenantAggregateAnonymous => {
            matches!(data_class, DataClass::Pci | DataClass::SearchQuery)
        }
        Purpose::PersonalizationInProduct => is_regulated_privacy_class(data_class),
        Purpose::ModelTrainingOya => is_model_training_privacy_hard_denied(data_class),
        Purpose::ModelTrainingThirdParty => is_model_training_privacy_hard_denied(data_class),
        Purpose::CoreService | Purpose::CapabilityInvocation | Purpose::Support => false,
    }
}

fn is_search_index_privacy_hard_denied(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
    )
}

fn is_ads_targeting_privacy_hard_denied(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::InternalOnly
            | DataClass::PiiIdentifying
            | DataClass::PiiSensitive
            | DataClass::PiiQuasiIdentifier
            | DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
            | DataClass::Usage
            | DataClass::BehavioralTenantProduct
            | DataClass::SearchQuery
    )
}

fn is_regulated_privacy_class(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
    )
}

fn is_model_training_privacy_hard_denied(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::PiiIdentifying
            | DataClass::PiiSensitive
            | DataClass::PiiQuasiIdentifier
            | DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
            | DataClass::SearchQuery
    )
}

fn is_audit_marker_hard_denied(purpose: Purpose) -> bool {
    matches!(
        purpose,
        Purpose::SearchIndexPublic
            | Purpose::AdsTargeting
            | Purpose::AdTargetingDeclared
            | Purpose::AdTargetingBehavioral
            | Purpose::ModelTrainingOya
            | Purpose::ModelTrainingThirdParty
    )
}

fn is_secret_marker_hard_denied(purpose: Purpose) -> bool {
    matches!(
        purpose,
        Purpose::SearchIndex
            | Purpose::SearchIndexPrivate
            | Purpose::SearchIndexPublic
            | Purpose::AdsTargeting
            | Purpose::AdTargetingDeclared
            | Purpose::AdTargetingBehavioral
            | Purpose::Analytics
            | Purpose::TenantAnalyticsFirstParty
            | Purpose::CrossTenantAggregateAnonymous
            | Purpose::PersonalizationInProduct
            | Purpose::ModelTrainingOya
            | Purpose::ModelTrainingThirdParty
    )
}

fn is_child_subject_marker_hard_denied(purpose: Purpose) -> bool {
    matches!(
        purpose,
        Purpose::SearchIndex
            | Purpose::SearchIndexPrivate
            | Purpose::SearchIndexPublic
            | Purpose::AdsTargeting
            | Purpose::AdTargetingDeclared
            | Purpose::AdTargetingBehavioral
            | Purpose::PersonalizationInProduct
            | Purpose::ModelTrainingOya
            | Purpose::ModelTrainingThirdParty
    )
}

pub fn evaluate_data_use(attributes: DataUseAttributes) -> Result<(), DataUseDenialReason> {
    if is_hard_denied_classification(attributes.purpose, attributes.data_classification) {
        return Err(DataUseDenialReason::HardDeniedDataClass);
    }
    if is_subject_hard_denied(attributes.purpose, attributes.subject_class) {
        return Err(DataUseDenialReason::MinorSubjectAds);
    }
    Ok(())
}

pub fn evaluate_data_use_classification(
    attributes: DataUseClassificationAttributes,
) -> Result<(), DataUseDenialReason> {
    evaluate_data_use(attributes)
}

pub fn evaluate_legacy_data_use(
    attributes: LegacyDataUseAttributes,
) -> Result<(), DataUseDenialReason> {
    evaluate_data_use(DataUseAttributes {
        purpose: attributes.purpose,
        data_classification: attributes.data_class.into(),
        subject_class: attributes.subject_class,
    })
}

pub fn is_subject_hard_denied(purpose: Purpose, subject_class: SubjectClass) -> bool {
    matches!(purpose, Purpose::AdsTargeting) && matches!(subject_class, SubjectClass::Minor { .. })
}

#[cfg(test)]
mod tests {
    use super::{
        AgeBand, Classified, DataClass, DataClassification, DataUseAttributes,
        DataUseClassificationAttributes, DataUseDenialReason, LegacyDataUseAttributes,
        OperationalDataClass, PrivacyDataClass, Purpose, SubjectClass, SubjectDataMarker,
        evaluate_data_use, evaluate_data_use_classification, evaluate_legacy_data_use,
    };

    fn privacy(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::try_from(data_class).expect("test fixture must use privacy data class")
    }

    #[test]
    fn privacy_program_data_class_labels_are_parseable() {
        for label in super::PRIVACY_PROGRAM_DATA_CLASS_LABELS {
            assert!(
                super::parse_data_class_label(label).is_some(),
                "privacy-program label must parse: {label}"
            );
        }
        assert_eq!(DataClass::Financial.label(), "FINANCIAL");
        assert_eq!(
            DataClass::Financial.privacy_program_label(),
            Some("FINANCIAL")
        );
        assert_eq!(
            super::PrivacyDataClass::new(DataClass::Financial)
                .expect("financial compatibility class is a privacy class")
                .label(),
            "FINANCIAL"
        );
        assert_eq!(DataClass::Audit.privacy_program_label(), None);
        for operational_or_subject_label in ["AUDIT", "SECRET", "CHILDREN"] {
            assert_eq!(
                super::parse_data_class_label(operational_or_subject_label),
                None,
                "non-privacy label must not parse as a privacy data class"
            );
        }
        assert_eq!(
            super::parse_operational_data_class_label("AUDIT"),
            Some(OperationalDataClass::Audit)
        );
        assert_eq!(
            super::parse_subject_data_marker_label("CHILDREN"),
            Some(SubjectDataMarker::Children)
        );
    }

    #[test]
    fn privacy_data_class_newtype_rejects_operational_and_subject_markers() {
        for data_class in [
            DataClass::Public,
            DataClass::InternalOnly,
            DataClass::PiiIdentifying,
            DataClass::PiiQuasiIdentifier,
            DataClass::FinancialRegulatedCredit,
            DataClass::BehavioralAds,
            DataClass::SensitivePipaArticle23,
        ] {
            let privacy_class = super::PrivacyDataClass::try_from(data_class)
                .expect("privacy-program data classes should construct");
            assert_eq!(privacy_class.data_class(), data_class);
            assert_eq!(privacy_class.label(), data_class.label());
        }

        for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
            assert_eq!(
                super::PrivacyDataClass::try_from(data_class),
                Err(super::NonPrivacyDataClass { data_class })
            );
        }
        assert_eq!(
            super::privacy_data_classes_from(&[DataClass::InternalOnly, DataClass::Audit]),
            Err(super::NonPrivacyDataClass {
                data_class: DataClass::Audit
            })
        );
        let privacy_classes = super::privacy_data_classes_from(&[
            DataClass::Public,
            DataClass::Phi,
            DataClass::BehavioralAds,
        ])
        .expect("privacy classes construct");
        assert_eq!(
            super::data_classes_from_privacy_data_classes(&privacy_classes),
            vec![DataClass::Public, DataClass::Phi, DataClass::BehavioralAds]
        );
        assert_eq!(
            super::most_restrictive_privacy_data_class(&privacy_classes),
            Some(DataClass::BehavioralAds)
        );
    }

    #[test]
    fn data_class_pascal_labels_round_trip_for_file_ledger_compatibility() {
        for data_class in [
            DataClass::Public,
            DataClass::InternalOnly,
            DataClass::PiiIdentifying,
            DataClass::PiiSensitive,
            DataClass::Phi,
            DataClass::Pci,
            DataClass::PipaArticle23,
            DataClass::Children,
            DataClass::Financial,
            DataClass::Usage,
            DataClass::Secret,
            DataClass::Audit,
            DataClass::PiiQuasiIdentifier,
            DataClass::FinancialRegulatedCredit,
            DataClass::BehavioralTenantProduct,
            DataClass::BehavioralAds,
            DataClass::DeclaredPreference,
            DataClass::SearchQuery,
            DataClass::SensitivePipaArticle23,
        ] {
            assert_eq!(
                super::parse_data_class_pascal_label(data_class.pascal_label()),
                Some(data_class)
            );
        }
    }

    #[test]
    fn classified_metadata_splits_operational_and_subject_markers_from_privacy_classes() {
        let privacy = Classified::new("tenant-id", DataClass::InternalOnly);
        assert_eq!(
            privacy.data_class,
            DataClassification::from(DataClass::InternalOnly)
        );
        assert_eq!(
            privacy.data_class.privacy_data_class(),
            PrivacyDataClass::try_from(DataClass::InternalOnly).ok()
        );

        let audit = Classified::new("audit-hash", OperationalDataClass::Audit);
        assert_eq!(
            audit.data_class,
            DataClassification::Operational(OperationalDataClass::Audit)
        );
        assert_eq!(audit.data_class.label(), "AUDIT");
        assert_eq!(audit.data_class.privacy_data_class(), None);
        assert_eq!(
            audit.data_class.compatibility_data_class(),
            DataClass::Audit
        );

        let secret = Classified::new("secret-ref", OperationalDataClass::Secret);
        assert_eq!(
            secret.data_class,
            DataClassification::Operational(OperationalDataClass::Secret)
        );
        assert_eq!(secret.data_class.label(), "SECRET");
        assert_eq!(
            secret.data_class.compatibility_data_class(),
            DataClass::Secret
        );

        let child_subject_marker = Classified::new("minor", SubjectDataMarker::Children);
        assert_eq!(
            child_subject_marker.data_class,
            DataClassification::SubjectMarker(SubjectDataMarker::Children)
        );
        assert_eq!(child_subject_marker.data_class.label(), "CHILDREN");
        assert_eq!(
            child_subject_marker.data_class.compatibility_data_class(),
            DataClass::Children
        );
        assert_eq!(
            DataClassification::from(DataClass::Audit),
            DataClassification::from(OperationalDataClass::Audit)
        );
        assert_eq!(
            DataClassification::from(DataClass::Secret),
            DataClassification::from(OperationalDataClass::Secret)
        );
        assert_eq!(
            DataClassification::from(DataClass::Children),
            DataClassification::from(SubjectDataMarker::Children)
        );
        assert_eq!(
            DataClassification::from(DataClass::PiiIdentifying).normalized(),
            DataClassification::from(DataClass::PiiIdentifying)
        );
    }

    #[test]
    fn purpose_pascal_labels_round_trip_for_file_ledger_compatibility() {
        for purpose in [
            Purpose::CoreService,
            Purpose::CapabilityInvocation,
            Purpose::SearchIndex,
            Purpose::AdsTargeting,
            Purpose::Analytics,
            Purpose::Support,
            Purpose::TenantAnalyticsFirstParty,
            Purpose::CrossTenantAggregateAnonymous,
            Purpose::PersonalizationInProduct,
            Purpose::SearchIndexPrivate,
            Purpose::SearchIndexPublic,
            Purpose::AdTargetingDeclared,
            Purpose::AdTargetingBehavioral,
            Purpose::ModelTrainingOya,
            Purpose::ModelTrainingThirdParty,
        ] {
            assert_eq!(
                super::parse_purpose_pascal_label(purpose.pascal_label()),
                Some(purpose)
            );
        }
    }

    #[test]
    fn ad_targeting_purpose_blocks_non_ad_safe_classes_even_with_grants() {
        let scope = super::ConsentScope::default()
            .allow(Purpose::AdsTargeting, privacy(DataClass::PiiIdentifying))
            .allow(Purpose::AdsTargeting, privacy(DataClass::SearchQuery))
            .allow(Purpose::AdsTargeting, privacy(DataClass::InternalOnly));

        for data_class in [
            DataClass::PiiIdentifying,
            DataClass::SearchQuery,
            DataClass::InternalOnly,
        ] {
            assert!(!scope.allows(Purpose::AdsTargeting, privacy(data_class)));
            assert_eq!(
                evaluate_legacy_data_use(LegacyDataUseAttributes {
                    purpose: Purpose::AdsTargeting,
                    data_class,
                    subject_class: SubjectClass::Adult,
                }),
                Err(DataUseDenialReason::HardDeniedDataClass)
            );
        }
    }

    #[test]
    fn declared_preference_is_the_only_declared_ad_targeting_class() {
        let scope = super::ConsentScope::default().allow(
            Purpose::AdTargetingDeclared,
            privacy(DataClass::DeclaredPreference),
        );

        assert!(scope.allows(
            Purpose::AdTargetingDeclared,
            privacy(DataClass::DeclaredPreference)
        ));
        assert_eq!(
            evaluate_legacy_data_use(LegacyDataUseAttributes {
                purpose: Purpose::AdTargetingDeclared,
                data_class: DataClass::PiiIdentifying,
                subject_class: SubjectClass::Adult,
            }),
            Err(DataUseDenialReason::HardDeniedDataClass)
        );
    }

    #[test]
    fn public_search_index_accepts_only_public_class() {
        assert_eq!(
            evaluate_legacy_data_use(LegacyDataUseAttributes {
                purpose: Purpose::SearchIndexPublic,
                data_class: DataClass::Public,
                subject_class: SubjectClass::Adult,
            }),
            Ok(())
        );
        assert_eq!(
            evaluate_legacy_data_use(LegacyDataUseAttributes {
                purpose: Purpose::SearchIndexPublic,
                data_class: DataClass::PiiIdentifying,
                subject_class: SubjectClass::Adult,
            }),
            Err(DataUseDenialReason::HardDeniedDataClass)
        );
    }

    #[test]
    fn model_training_purposes_block_direct_and_regulated_classes_even_with_grants() {
        let denied_classes = [
            DataClass::PiiIdentifying,
            DataClass::PiiQuasiIdentifier,
            DataClass::Phi,
            DataClass::Pci,
            DataClass::SensitivePipaArticle23,
            DataClass::FinancialRegulatedCredit,
            DataClass::SearchQuery,
        ];
        let mut scope = super::ConsentScope::default();
        for purpose in [Purpose::ModelTrainingOya, Purpose::ModelTrainingThirdParty] {
            for data_class in denied_classes {
                scope = scope.allow(purpose, privacy(data_class));
                assert!(!scope.allows(purpose, privacy(data_class)));
                assert_eq!(
                    evaluate_legacy_data_use(LegacyDataUseAttributes {
                        purpose,
                        data_class,
                        subject_class: SubjectClass::Adult,
                    }),
                    Err(DataUseDenialReason::HardDeniedDataClass)
                );
            }
            assert!(
                scope
                    .clone()
                    .try_allow_legacy_data_class(purpose, DataClass::Audit)
                    .is_err()
            );
            assert!(!scope.allows_legacy_data_class(purpose, DataClass::Audit));
        }

        let allowed_scope = super::ConsentScope::default()
            .allow(Purpose::ModelTrainingOya, privacy(DataClass::Public))
            .allow(Purpose::ModelTrainingThirdParty, privacy(DataClass::Public));
        assert!(allowed_scope.allows(Purpose::ModelTrainingOya, privacy(DataClass::Public)));
        assert!(allowed_scope.allows(Purpose::ModelTrainingThirdParty, privacy(DataClass::Public)));
    }

    #[test]
    fn typed_classification_policy_preserves_legacy_marker_denials() {
        let cases = [
            (
                Purpose::Analytics,
                DataClass::Secret,
                DataClassification::from(OperationalDataClass::Secret),
            ),
            (
                Purpose::ModelTrainingOya,
                DataClass::Audit,
                DataClassification::from(OperationalDataClass::Audit),
            ),
            (
                Purpose::SearchIndex,
                DataClass::Children,
                DataClassification::from(SubjectDataMarker::Children),
            ),
            (
                Purpose::CapabilityInvocation,
                DataClass::Audit,
                DataClassification::from(OperationalDataClass::Audit),
            ),
        ];

        for (purpose, legacy_class, classification) in cases {
            assert_eq!(
                super::is_hard_denied(purpose, legacy_class),
                super::is_hard_denied_classification(purpose, classification)
            );
        }

        assert_eq!(
            evaluate_data_use(DataUseAttributes {
                purpose: Purpose::Analytics,
                data_classification: DataClassification::from(OperationalDataClass::Secret),
                subject_class: SubjectClass::Adult,
            }),
            Err(DataUseDenialReason::HardDeniedDataClass)
        );
        assert_eq!(
            evaluate_data_use_classification(DataUseClassificationAttributes {
                purpose: Purpose::ModelTrainingThirdParty,
                data_classification: DataClassification::from(OperationalDataClass::Audit),
                subject_class: SubjectClass::Adult,
            }),
            Err(DataUseDenialReason::HardDeniedDataClass)
        );
        assert_eq!(
            evaluate_data_use_classification(DataUseClassificationAttributes {
                purpose: Purpose::SearchIndexPrivate,
                data_classification: DataClassification::from(SubjectDataMarker::Children),
                subject_class: SubjectClass::Adult,
            }),
            Err(DataUseDenialReason::HardDeniedDataClass)
        );
        assert_eq!(
            evaluate_data_use_classification(DataUseClassificationAttributes {
                purpose: Purpose::Analytics,
                data_classification: DataClassification::from(OperationalDataClass::Audit),
                subject_class: SubjectClass::Adult,
            }),
            Ok(())
        );
    }

    #[test]
    fn typed_consent_scope_grants_only_privacy_classifications() {
        let scope = super::ConsentScope::default()
            .allow(Purpose::Analytics, privacy(DataClass::InternalOnly));

        assert!(
            scope
                .clone()
                .try_allow_legacy_data_class(Purpose::Analytics, DataClass::Audit)
                .is_err()
        );

        assert!(scope.allows_classification(
            Purpose::Analytics,
            DataClassification::from(DataClass::InternalOnly)
        ));
        assert!(!scope.allows_classification(
            Purpose::Analytics,
            DataClassification::from(OperationalDataClass::Audit)
        ));
        assert!(!scope.allows_classification(
            Purpose::Analytics,
            DataClassification::from(SubjectDataMarker::Children)
        ));
    }

    #[test]
    fn legacy_consent_scope_allow_fails_closed_for_non_privacy_markers() {
        let scope = super::ConsentScope::default()
            .try_allow_legacy_data_class(Purpose::CapabilityInvocation, DataClass::InternalOnly)
            .expect("internal-only is a privacy data class");

        assert!(
            scope.allows_legacy_data_class(Purpose::CapabilityInvocation, DataClass::InternalOnly)
        );
        for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
            assert!(
                scope
                    .clone()
                    .try_allow_legacy_data_class(Purpose::CapabilityInvocation, data_class)
                    .is_err()
            );
            assert!(!scope.allows_legacy_data_class(Purpose::CapabilityInvocation, data_class));
        }
    }

    #[test]
    fn data_use_evaluator_composes_data_class_and_subject_class_denies() {
        assert_eq!(
            evaluate_legacy_data_use(LegacyDataUseAttributes {
                purpose: Purpose::AdsTargeting,
                data_class: DataClass::Public,
                subject_class: SubjectClass::Minor {
                    age_band: AgeBand::Under14
                },
            }),
            Err(DataUseDenialReason::MinorSubjectAds)
        );
        assert_eq!(
            evaluate_legacy_data_use(LegacyDataUseAttributes {
                purpose: Purpose::AdsTargeting,
                data_class: DataClass::Phi,
                subject_class: SubjectClass::Adult,
            }),
            Err(DataUseDenialReason::HardDeniedDataClass)
        );
        assert_eq!(
            evaluate_legacy_data_use(LegacyDataUseAttributes {
                purpose: Purpose::AdsTargeting,
                data_class: DataClass::Public,
                subject_class: SubjectClass::Adult,
            }),
            Ok(())
        );
    }
}
