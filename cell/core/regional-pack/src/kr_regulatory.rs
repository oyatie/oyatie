//! KR regulatory binding types for PIPA Article-23 sensitive data classification
//! and CSAP control evidence references.
//!
//! Implements M04-P02-IP-001 (merge-variant delta-1): smallest net-new types
//! merged into `regional-pack-domain`; no new crate scaffolding.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// PIPA (개인정보 보호법) Article-23 sensitive personal data classification.
///
/// Art-23 prohibits processing of sensitive categories without explicit consent
/// or a specific statutory basis. Each variant maps to one Art-23 sensitive
/// category. The `General` variant covers non-sensitive personal data governed
/// by the general PIPA provisions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PipaDataClassification {
    /// Non-sensitive personal data (PIPA general provisions).
    General,
    /// Ideology or belief (사상·신념) — Art-23(1).
    IdeologyOrBelief,
    /// Trade-union membership or political party affiliation (노동조합·정당의 가입·탈퇴) — Art-23(1).
    UnionOrPartyMembership,
    /// Political views (정치적 견해) — Art-23(1).
    PoliticalViews,
    /// Health or medical information (건강·성생활 등에 관한 정보) — Art-23(1).
    HealthOrMedical,
    /// Biometric or genetic data (생체인식정보·유전정보) — Art-23(1).
    BiometricOrGenetic,
    /// Criminal record (범죄경력자료) — Art-23(1).
    CriminalRecord,
    /// Race or ethnicity (인종·민족) — Art-23(1) sensitive scope per KR policy baseline.
    RaceOrEthnicity,
}

impl PipaDataClassification {
    /// Returns `true` when the classification requires explicit consent or a
    /// statutory exception under PIPA Art-23.
    pub fn is_sensitive(self) -> bool {
        !matches!(self, Self::General)
    }

    /// Returns the canonical Korean statutory label for this classification.
    pub fn statutory_label(self) -> &'static str {
        match self {
            Self::General => "일반개인정보",
            Self::IdeologyOrBelief => "사상·신념",
            Self::UnionOrPartyMembership => "노동조합·정당의 가입·탈퇴",
            Self::PoliticalViews => "정치적 견해",
            Self::HealthOrMedical => "건강·성생활 등에 관한 정보",
            Self::BiometricOrGenetic => "생체인식정보·유전정보",
            Self::CriminalRecord => "범죄경력자료",
            Self::RaceOrEthnicity => "인종·민족",
        }
    }
}

/// Binding of a regional pack to KR-specific regulatory controls.
///
/// A `KrRegulatoryBinding` asserts that the named `pack_id` has been audited
/// against PIPA Art-23 and CSAP, and records the CSAP evidence reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KrRegulatoryBinding {
    /// Pack identifier (must match `RegionalPack::id` prefix `pack-`).
    pack_id: String,
    /// PIPA classification for the primary data processed by this pack.
    pipa_classification: PipaDataClassification,
    /// CSAP control-evidence reference (non-empty).
    csap_evidence_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KrRegulatoryBindingError {
    InvalidPackId,
    EmptyCsapEvidenceRef,
}

impl KrRegulatoryBinding {
    /// Create and validate a new `KrRegulatoryBinding`.
    pub fn new(
        pack_id: String,
        pipa_classification: PipaDataClassification,
        csap_evidence_ref: String,
    ) -> Result<Self, KrRegulatoryBindingError> {
        if !pack_id.starts_with("pack-") {
            return Err(KrRegulatoryBindingError::InvalidPackId);
        }
        if csap_evidence_ref.trim().is_empty() {
            return Err(KrRegulatoryBindingError::EmptyCsapEvidenceRef);
        }
        Ok(Self {
            pack_id,
            pipa_classification,
            csap_evidence_ref,
        })
    }

    /// Returns the pack identifier.
    pub fn pack_id(&self) -> &str {
        &self.pack_id
    }

    /// Returns the PIPA data classification.
    pub fn pipa_classification(&self) -> PipaDataClassification {
        self.pipa_classification
    }

    /// Returns the CSAP control-evidence reference.
    pub fn csap_evidence_ref(&self) -> &str {
        &self.csap_evidence_ref
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_is_not_sensitive_all_others_are() {
        assert!(!PipaDataClassification::General.is_sensitive());
        for cls in [
            PipaDataClassification::IdeologyOrBelief,
            PipaDataClassification::UnionOrPartyMembership,
            PipaDataClassification::PoliticalViews,
            PipaDataClassification::HealthOrMedical,
            PipaDataClassification::BiometricOrGenetic,
            PipaDataClassification::CriminalRecord,
            PipaDataClassification::RaceOrEthnicity,
        ] {
            assert!(cls.is_sensitive(), "{cls:?} must be sensitive");
        }
    }

    #[test]
    fn statutory_labels_are_non_empty() {
        for cls in [
            PipaDataClassification::General,
            PipaDataClassification::IdeologyOrBelief,
            PipaDataClassification::UnionOrPartyMembership,
            PipaDataClassification::PoliticalViews,
            PipaDataClassification::HealthOrMedical,
            PipaDataClassification::BiometricOrGenetic,
            PipaDataClassification::CriminalRecord,
            PipaDataClassification::RaceOrEthnicity,
        ] {
            assert!(
                !cls.statutory_label().is_empty(),
                "{cls:?} label must be non-empty"
            );
        }
    }

    /// Synthetic-violation test: validates the RaceOrEthnicity variant is correctly
    /// classified as sensitive and returns the canonical Art-23 statutory label.
    /// Without this variant, callers must misclassify race/ethnicity data as General,
    /// producing incorrect consent/audit labeling.
    #[test]
    fn race_or_ethnicity_is_sensitive_with_canonical_label() {
        let cls = PipaDataClassification::RaceOrEthnicity;
        assert!(
            cls.is_sensitive(),
            "RaceOrEthnicity must be PIPA Art-23 sensitive"
        );
        assert_eq!(
            cls.statutory_label(),
            "인종·민족",
            "RaceOrEthnicity must map to canonical KR statutory label"
        );
    }

    #[test]
    fn kr_regulatory_binding_accepts_valid_inputs() {
        let binding = KrRegulatoryBinding::new(
            "pack-alpha".to_string(),
            PipaDataClassification::HealthOrMedical,
            "csap-ctrl-kr-2026-001".to_string(),
        )
        .expect("valid binding should be accepted");

        assert_eq!(binding.pack_id(), "pack-alpha");
        assert!(binding.pipa_classification().is_sensitive());
        assert_eq!(binding.csap_evidence_ref(), "csap-ctrl-kr-2026-001");
    }

    #[test]
    fn kr_regulatory_binding_rejects_invalid_pack_id() {
        let err = KrRegulatoryBinding::new(
            "regional-kr".to_string(),
            PipaDataClassification::General,
            "csap-ctrl-kr-2026-001".to_string(),
        )
        .expect_err("non-pack- prefix must be rejected");

        assert_eq!(err, KrRegulatoryBindingError::InvalidPackId);
    }

    #[test]
    fn kr_regulatory_binding_rejects_empty_csap_ref() {
        let err = KrRegulatoryBinding::new(
            "pack-alpha".to_string(),
            PipaDataClassification::General,
            "   ".to_string(),
        )
        .expect_err("blank csap_evidence_ref must be rejected");

        assert_eq!(err, KrRegulatoryBindingError::EmptyCsapEvidenceRef);
    }
}
