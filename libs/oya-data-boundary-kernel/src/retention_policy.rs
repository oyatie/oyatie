//! Retention policy types for data-boundary classification.
//!
//! [`ClassificationLevel`] is an ordered severity tier that maps the raw
//! [`DataClass`] vocabulary to a 4-level operational sensitivity scale.
//! [`DataClassMatcher`] provides predicate logic for testing membership in
//! named class sets without importing the full privacy-evaluation graph.
//! [`RetentionPolicy`] captures the declared retention window and mandatory
//! purge action for a classified data object.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use crate::DataClass;
use std::time::Duration;

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

// ---------------------------------------------------------------------------
// DataClassMatcher
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// RetentionPolicy
// ---------------------------------------------------------------------------

/// Purge action that must be taken when the retention window expires.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PurgeAction {
    /// Cryptographically shred the encryption key; the ciphertext becomes
    /// irrecoverable. Required for [`ClassificationLevel::Critical`] data.
    CryptoShred,
    /// Overwrite the storage block with zeroes before deallocation.
    SecureErase,
    /// Standard logical deletion (sufficient for unrestricted data).
    LogicalDelete,
}

/// Declared retention window and mandatory purge action for a classified data
/// object.
///
/// [`RetentionPolicy`] does not enforce the purge itself — that is the
/// responsibility of the data-boundary purge executor. It is a value type
/// that carries the contract established at classification time so that the
/// executor can audit-log the correct action and duration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    /// Maximum wall-clock duration for which a data object at this
    /// classification level may be retained after its last legitimate use.
    // data_class: INTERNAL_ONLY
    pub retention_window: Duration,
    /// The purge action that must be taken when `retention_window` elapses.
    // data_class: INTERNAL_ONLY
    pub purge_action: PurgeAction,
    /// The classification level that governs this policy.
    // data_class: INTERNAL_ONLY
    pub level: ClassificationLevel,
}

impl RetentionPolicy {
    /// Construct a [`RetentionPolicy`] from a raw [`DataClass`].
    ///
    /// The defaults mirror the regulatory minimums codified in ADR-0008:
    ///
    /// | Level | Window | Action |
    /// |---|---|---|
    /// | Critical | 30 days | CryptoShred |
    /// | Sensitive | 90 days | SecureErase |
    /// | Restricted | 365 days | LogicalDelete |
    /// | Unrestricted | 730 days | LogicalDelete |
    pub fn from_data_class(data_class: DataClass) -> Self {
        let level = ClassificationLevel::from_data_class(data_class);
        Self::from_level(level)
    }

    /// Construct a [`RetentionPolicy`] directly from a [`ClassificationLevel`].
    pub fn from_level(level: ClassificationLevel) -> Self {
        let (retention_window, purge_action) = match level {
            ClassificationLevel::Critical => (
                Duration::from_secs(30 * 24 * 3600),
                PurgeAction::CryptoShred,
            ),
            ClassificationLevel::Sensitive => (
                Duration::from_secs(90 * 24 * 3600),
                PurgeAction::SecureErase,
            ),
            ClassificationLevel::Restricted => (
                Duration::from_secs(365 * 24 * 3600),
                PurgeAction::LogicalDelete,
            ),
            ClassificationLevel::Unrestricted => (
                Duration::from_secs(730 * 24 * 3600),
                PurgeAction::LogicalDelete,
            ),
        };
        Self {
            retention_window,
            purge_action,
            level,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{ClassificationLevel, DataClassMatcher, PurgeAction, RetentionPolicy};
    use crate::DataClass;

    #[test]
    fn classification_level_ordering_is_unrestricted_lt_critical() {
        assert!(ClassificationLevel::Unrestricted < ClassificationLevel::Restricted);
        assert!(ClassificationLevel::Restricted < ClassificationLevel::Sensitive);
        assert!(ClassificationLevel::Sensitive < ClassificationLevel::Critical);
    }

    #[test]
    fn hard_deny_data_classes_map_to_critical_level() {
        for data_class in [
            DataClass::Phi,
            DataClass::Pci,
            DataClass::PipaArticle23,
            DataClass::SensitivePipaArticle23,
            DataClass::Children,
        ] {
            assert_eq!(
                ClassificationLevel::from_data_class(data_class),
                ClassificationLevel::Critical,
                "{data_class:?} must map to Critical"
            );
            assert!(ClassificationLevel::from_data_class(data_class).is_hard_deny_tier());
        }
    }

    #[test]
    fn public_maps_to_unrestricted_level() {
        assert_eq!(
            ClassificationLevel::from_data_class(DataClass::Public),
            ClassificationLevel::Unrestricted
        );
        assert!(!ClassificationLevel::Unrestricted.is_hard_deny_tier());
    }

    #[test]
    fn classification_level_labels_are_stable() {
        assert_eq!(ClassificationLevel::Unrestricted.label(), "UNRESTRICTED");
        assert_eq!(ClassificationLevel::Restricted.label(), "RESTRICTED");
        assert_eq!(ClassificationLevel::Sensitive.label(), "SENSITIVE");
        assert_eq!(ClassificationLevel::Critical.label(), "CRITICAL");
    }

    #[test]
    fn data_class_matcher_hard_deny_set_covers_exactly_phi_pci_pipa_children() {
        let hard_deny = [
            DataClass::Phi,
            DataClass::Pci,
            DataClass::PipaArticle23,
            DataClass::SensitivePipaArticle23,
            DataClass::Children,
        ];
        for dc in hard_deny {
            assert!(
                DataClassMatcher::HardDenySet.matches(dc),
                "{dc:?} must be in HardDenySet"
            );
        }
        for dc in [
            DataClass::Public,
            DataClass::InternalOnly,
            DataClass::PiiIdentifying,
            DataClass::Financial,
        ] {
            assert!(
                !DataClassMatcher::HardDenySet.matches(dc),
                "{dc:?} must not be in HardDenySet"
            );
        }
    }

    #[test]
    fn data_class_matcher_regulated_financial_covers_financial_variants() {
        assert!(DataClassMatcher::RegulatedFinancial.matches(DataClass::Financial));
        assert!(DataClassMatcher::RegulatedFinancial.matches(DataClass::FinancialRegulatedCredit));
        assert!(!DataClassMatcher::RegulatedFinancial.matches(DataClass::Phi));
        assert!(!DataClassMatcher::RegulatedFinancial.matches(DataClass::Public));
    }

    #[test]
    fn data_class_matcher_direct_pii_covers_identifying_variants() {
        for dc in [
            DataClass::PiiIdentifying,
            DataClass::PiiSensitive,
            DataClass::PiiQuasiIdentifier,
        ] {
            assert!(
                DataClassMatcher::DirectPii.matches(dc),
                "{dc:?} must be DirectPii"
            );
        }
        assert!(!DataClassMatcher::DirectPii.matches(DataClass::Phi));
        assert!(!DataClassMatcher::DirectPii.matches(DataClass::BehavioralAds));
    }

    #[test]
    fn data_class_matcher_search_index_restricted_covers_phi_pci_pipa_financial() {
        for dc in [
            DataClass::Phi,
            DataClass::Pci,
            DataClass::PipaArticle23,
            DataClass::SensitivePipaArticle23,
            DataClass::Financial,
            DataClass::FinancialRegulatedCredit,
        ] {
            assert!(
                DataClassMatcher::SearchIndexRestricted.matches(dc),
                "{dc:?} must be SearchIndexRestricted"
            );
        }
        assert!(!DataClassMatcher::SearchIndexRestricted.matches(DataClass::Public));
        assert!(!DataClassMatcher::SearchIndexRestricted.matches(DataClass::PiiIdentifying));
    }

    #[test]
    fn retention_policy_critical_class_uses_crypto_shred_30_days() {
        let policy = RetentionPolicy::from_data_class(DataClass::Phi);
        assert_eq!(policy.level, ClassificationLevel::Critical);
        assert_eq!(policy.purge_action, PurgeAction::CryptoShred);
        assert_eq!(policy.retention_window.as_secs(), 30 * 24 * 3600);
    }

    #[test]
    fn retention_policy_sensitive_class_uses_secure_erase_90_days() {
        let policy = RetentionPolicy::from_data_class(DataClass::PiiIdentifying);
        assert_eq!(policy.level, ClassificationLevel::Sensitive);
        assert_eq!(policy.purge_action, PurgeAction::SecureErase);
        assert_eq!(policy.retention_window.as_secs(), 90 * 24 * 3600);
    }

    #[test]
    fn retention_policy_restricted_class_uses_logical_delete_365_days() {
        let policy = RetentionPolicy::from_data_class(DataClass::InternalOnly);
        assert_eq!(policy.level, ClassificationLevel::Restricted);
        assert_eq!(policy.purge_action, PurgeAction::LogicalDelete);
        assert_eq!(policy.retention_window.as_secs(), 365 * 24 * 3600);
    }

    #[test]
    fn retention_policy_unrestricted_class_uses_logical_delete_730_days() {
        let policy = RetentionPolicy::from_data_class(DataClass::Public);
        assert_eq!(policy.level, ClassificationLevel::Unrestricted);
        assert_eq!(policy.purge_action, PurgeAction::LogicalDelete);
        assert_eq!(policy.retention_window.as_secs(), 730 * 24 * 3600);
    }

    #[test]
    fn retention_policy_from_level_is_consistent_with_from_data_class() {
        for data_class in [
            DataClass::Phi,
            DataClass::PiiIdentifying,
            DataClass::InternalOnly,
            DataClass::Public,
        ] {
            let level = ClassificationLevel::from_data_class(data_class);
            assert_eq!(
                RetentionPolicy::from_data_class(data_class),
                RetentionPolicy::from_level(level)
            );
        }
    }

    const RETENTION_QK03_DSR_CONTRACT: &str =
        include_str!("../fixtures/retention_qk03_dsr_evidence_contract.txt");

    fn assert_contract_contains(marker: &str) {
        assert!(
            RETENTION_QK03_DSR_CONTRACT.contains(marker),
            "RETENTION-QK03 DSR contract is missing marker: {marker}"
        );
    }

    #[test]
    fn retention_qk03_contract_records_current_policy_defaults() {
        for (level, expected_days, expected_action) in [
            (ClassificationLevel::Critical, 30, PurgeAction::CryptoShred),
            (ClassificationLevel::Sensitive, 90, PurgeAction::SecureErase),
            (
                ClassificationLevel::Restricted,
                365,
                PurgeAction::LogicalDelete,
            ),
            (
                ClassificationLevel::Unrestricted,
                730,
                PurgeAction::LogicalDelete,
            ),
        ] {
            let policy = RetentionPolicy::from_level(level);
            assert_eq!(policy.level, level);
            assert_eq!(policy.purge_action, expected_action);
            assert_eq!(policy.retention_window.as_secs(), expected_days * 24 * 3600);
            assert_contract_contains(&format!(
                "retention.default.{}|window_days={expected_days}|purge_action={expected_action:?}",
                level.label()
            ));
        }
    }

    #[test]
    fn retention_qk03_contract_names_privacy_governance_evidence_markers() {
        for marker in [
            "contract_id=RETENTION-QK03-RED-001",
            "qk_id=QK-03-privacy-data-governance",
            "claim_status=blocked_until_future_runtime_evidence",
            "adr_0536_context=D-8_KMS_and_D-16_Audit_are_planning_context_only",
            "conflict_boundary=TRUST-005:t_bce85039:crypto_shred_kms_executor",
            "conflict_boundary=AUDIT-002:t_c157a3ae:retrieval_and_proof_emission",
            "conflict_boundary=PRIVACY-001:t_2fc04777:dub_cedar_matrix",
            "conflict_boundary=DATA-003:t_bbe5db45:data_boundary_storage_integration",
            "conflict_boundary=QK-EVIDENCE:t_cf995f91:quality_kit_evidence_decomposition",
            "evidence_output=data_flow_map",
            "evidence_output=retention_expiry_decision",
            "evidence_output=purge_action_selection",
            "evidence_output=dsr_delete_export_proof",
            "evidence_output=audit_proof_emission",
            "evidence_output=trust_portal_publication_semantics",
            "evidence_output=telemetry_redaction_check",
            "forbidden_claim=production_readiness_green",
            "forbidden_claim=runtime_harness_implemented",
            "forbidden_claim=measured_dogfood_receipt_exists",
            "forbidden_claim=trust_portal_publication_live",
            "forbidden_claim=hyperscaler_production_maturity",
        ] {
            assert_contract_contains(marker);
        }

        for scenario in [
            "scenario=QK-03-S01-personal-data-inventory|requires=data_flow_map",
            "scenario=QK-03-S02-residency-enforcement|requires=cedar_residency_constraint",
            "scenario=QK-03-S03-retention-expiry|requires=retention_expiry_decision",
            "scenario=QK-03-S04-deletion-erasure|requires=purge_action_selection",
            "scenario=QK-03-S05-export-portability|requires=dsr_export_receipt",
            "scenario=QK-03-S06-no-pii-telemetry-redaction|requires=telemetry_redaction_check",
            "scenario=QK-03-DSR-round-trip-proof|requires=dsr_delete_export_proof+audit_proof_emission+trust_portal_publication_semantics",
        ] {
            assert_contract_contains(scenario);
        }
    }
}
