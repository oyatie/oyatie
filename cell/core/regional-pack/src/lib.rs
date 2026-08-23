//! Regional pack kernel: canonical regulatory and residency pack metadata.
//!
//! Also hosts the M04-P01 vertical capability pack types
//! (`CapabilityPack`, `PackVersion`, `CapabilityPackError`) merged per
//! execution-variant decision 2026-05-17 (option 2 — merge-into-existing-crates).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod vertical_regulatory_profile;
pub use vertical_regulatory_profile::{
    AdVertical, VerticalRegulatoryProfile, VerticalRegulatoryProfileError,
};
pub mod capability_pack;
pub use capability_pack::{CapabilityPack, CapabilityPackError, PackVersion};
pub mod kr_regulatory;
pub use kr_regulatory::{KrRegulatoryBinding, KrRegulatoryBindingError, PipaDataClassification};
pub mod pack_onboarding_phase;
pub use pack_onboarding_phase::{
    PackInstallStatus, PackOnboardingPhase, RegionalRolloutGate, RegionalRolloutGateError,
};
pub mod manifest;
pub use manifest::{
    GateCheck, GateViolation, REGIONAL_PACK_MANIFEST_SCHEMA_VERSION, RegionalPackManifest,
    RegionalPackManifestError, RegionalPackManifestGateReport, RegionalPackSource,
    canonical_base_neutrality_check, cross_pack_refusal_check, evaluate_regional_pack_gates,
    load_regional_pack_manifest, parse_regional_pack_manifest,
};
pub mod sovereign_deployment;
pub use sovereign_deployment::{
    NoExternalEgressValidation, SOVEREIGN_DEPLOYMENT_MODEL_MANIFEST_SCHEMA_VERSION,
    SovereignArtifactBundleEvidence, SovereignDeploymentModelAuthority,
    SovereignDeploymentModelIdentity, SovereignDeploymentModelKind,
    SovereignDeploymentModelManifest, SovereignDeploymentModelManifestError,
    SovereignDeploymentOwnership, SovereignPackOverlayBinding, SovereignRecoveryObjectives,
    SovereignSloTarget, parse_sovereign_deployment_model_manifest,
};

use data_boundary_kernel::{Classified, DataClass};
use network_residency::{ResidencyClass, parse_residency_class_label};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPack {
    pub id: String, // data_class: INTERNAL_ONLY
    pub region: Classified<String>,
    pub residency_class: Classified<ResidencyClass>,
    pub controls: Classified<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionalPackError {
    InvalidPackId,
    EmptyRegion,
    EmptyResidencyClass,
    InvalidResidencyClass,
    MissingControls,
}

impl RegionalPack {
    pub fn new(
        id: String,
        region: String,
        residency_class: String,
        controls: Vec<String>,
    ) -> Result<Self, RegionalPackError> {
        if !id.starts_with("pack-") {
            return Err(RegionalPackError::InvalidPackId);
        }
        if region.trim().is_empty() {
            return Err(RegionalPackError::EmptyRegion);
        }
        if residency_class.trim().is_empty() {
            return Err(RegionalPackError::EmptyResidencyClass);
        }
        let residency_class = parse_residency_class_label(&residency_class)
            .ok_or(RegionalPackError::InvalidResidencyClass)?;
        if controls.is_empty() {
            return Err(RegionalPackError::MissingControls);
        }
        Ok(Self {
            id,
            region: Classified::new(region, DataClass::InternalOnly),
            residency_class: Classified::new(residency_class, DataClass::InternalOnly),
            controls: Classified::new(controls, DataClass::InternalOnly),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_residency_labels() {
        let pack = RegionalPack::new(
            "pack-alpha".to_string(),
            "region-home".to_string(),
            "strict_home_region".to_string(),
            vec!["PIPA".to_string()],
        )
        .expect("canonical residency label should be accepted");

        assert_eq!(
            pack.residency_class.value.label(),
            Some("strict_home_region")
        );
    }

    #[test]
    fn rejects_non_canonical_residency_labels() {
        let error = RegionalPack::new(
            "pack-alpha".to_string(),
            "region-home".to_string(),
            "KR_RESIDENT".to_string(),
            vec!["PIPA".to_string()],
        )
        .expect_err("regional packs use ADR-0049 residency labels");

        assert_eq!(error, RegionalPackError::InvalidResidencyClass);
    }

    // Crate-local fixtures (declared in BUCK srcs) keep these tests hermetic:
    // `include_str!` embeds the bytes at compile time, so nothing reads the
    // repo-root filesystem and `buck2 test` passes inside its sandbox.
    const KR_MANIFEST_JSON: &str = include_str!("../tests/fixtures/kr/manifest.json");
    const KR_CANONICAL_BASE: &str = include_str!("../tests/fixtures/kr/canonical-base.txt");
    const KR_PACK_IMPL: &str = include_str!("../tests/fixtures/kr/pack-impl.txt");
    const CANONICAL_BASE_WITH_MARKERS: &str =
        include_str!("../tests/fixtures/negative/canonical-base-with-jurisdiction-markers.txt");
    const PACK_IMPL_CROSS_PACK: &str =
        include_str!("../tests/fixtures/negative/pack-impl-cross-pack-reference.txt");

    fn kr_manifest() -> RegionalPackManifest {
        parse_regional_pack_manifest(KR_MANIFEST_JSON).expect("KR fixture manifest should parse")
    }

    fn source(manifest_path: &str, content: &str) -> RegionalPackSource {
        RegionalPackSource::new(manifest_path, content)
    }

    #[test]
    fn kr_pack_manifest_declares_minimal_regional_compliance_and_localization_shape() {
        let manifest = kr_manifest();

        assert_eq!(
            manifest.manifest_schema_version,
            REGIONAL_PACK_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest.pack.id, "kr-seoul");
        assert_eq!(manifest.pack.code, "kr");
        assert_eq!(manifest.source_authority.accepted_adrs, vec!["ADR-0064"]);
        assert_eq!(
            manifest.source_authority.planning_context_adrs,
            vec!["ADR-0010"]
        );
        assert_eq!(manifest.regional_pack.jurisdiction, "KR");
        assert!(
            manifest
                .regional_pack
                .residency_classes
                .contains(&"strict_home_region".to_string())
        );
        assert!(
            manifest
                .regional_pack
                .regulatory_controls
                .contains(&"PIPA".to_string())
        );
        assert_eq!(
            manifest.canonical_base.neutrality_gate,
            "canonical-base-neutrality"
        );
        assert_eq!(
            manifest.pack_impl.cross_pack_refusal_gate,
            "cross-pack-refusal"
        );
        assert!(manifest.pack_impl.allowed_pack_dependencies.is_empty());
    }

    #[test]
    fn kr_fixture_passes_both_pack_gates() {
        let manifest = kr_manifest();
        let canonical_sources = vec![source(
            &manifest.canonical_base.canonical_base_paths[0],
            KR_CANONICAL_BASE,
        )];
        let pack_sources = vec![source(&manifest.pack_impl.pack_paths[0], KR_PACK_IMPL)];

        let report = evaluate_regional_pack_gates(&manifest, &canonical_sources, &pack_sources);

        assert_eq!(report.pack_id, "kr-seoul");
        assert_eq!(report.pack_code, "kr");
        assert!(report.passed(), "fixture should pass every local pack gate");
        assert_eq!(
            report
                .check("canonical-base-neutrality")
                .expect("neutrality check should be reported")
                .violations,
            Vec::<GateViolation>::new()
        );
        assert_eq!(
            report
                .check("cross-pack-refusal")
                .expect("cross-pack refusal check should be reported")
                .violations,
            Vec::<GateViolation>::new()
        );
    }

    #[test]
    fn canonical_base_with_jurisdiction_markers_trips_neutrality_gate() {
        let manifest = kr_manifest();
        let leaking_base = vec![source(
            "tests/fixtures/negative/canonical-base-with-jurisdiction-markers.txt",
            CANONICAL_BASE_WITH_MARKERS,
        )];
        let clean_pack = vec![source(&manifest.pack_impl.pack_paths[0], KR_PACK_IMPL)];

        let report = evaluate_regional_pack_gates(&manifest, &leaking_base, &clean_pack);

        assert!(!report.passed(), "a leaking canonical base must fail");
        let neutrality = report
            .check("canonical-base-neutrality")
            .expect("neutrality check should be reported");
        assert!(!neutrality.passed);
        assert!(
            neutrality
                .violations
                .iter()
                .any(|violation| violation.reason.contains("jurisdiction marker")),
            "expected a jurisdiction-marker violation, got {:?}",
            neutrality.violations
        );
        // The cross-pack gate is orthogonal and must stay green here.
        assert!(
            report
                .check("cross-pack-refusal")
                .expect("cross-pack refusal check should be reported")
                .passed
        );
    }

    #[test]
    fn pack_impl_referencing_another_pack_trips_cross_pack_refusal_gate() {
        let manifest = kr_manifest();
        let clean_base = vec![source(
            &manifest.canonical_base.canonical_base_paths[0],
            KR_CANONICAL_BASE,
        )];
        let cross_pack = vec![source(
            "tests/fixtures/negative/pack-impl-cross-pack-reference.txt",
            PACK_IMPL_CROSS_PACK,
        )];

        let report = evaluate_regional_pack_gates(&manifest, &clean_base, &cross_pack);

        assert!(!report.passed(), "a cross-pack reference must fail");
        let refusal = report
            .check("cross-pack-refusal")
            .expect("cross-pack refusal check should be reported");
        assert!(!refusal.passed);
        assert!(
            refusal
                .violations
                .iter()
                .any(|violation| violation.reason.contains("packs/jp")),
            "expected a packs/jp cross-pack violation, got {:?}",
            refusal.violations
        );
        assert!(
            report
                .check("canonical-base-neutrality")
                .expect("neutrality check should be reported")
                .passed
        );
    }

    #[test]
    fn rejects_branded_pack_id() {
        let branded = KR_MANIFEST_JSON.replace("\"kr-seoul\"", "\"pack-kr\"");

        let error = parse_regional_pack_manifest(&branded)
            .expect_err("branded pack ids are de-branded out");

        assert!(matches!(
            error,
            RegionalPackManifestError::Shape {
                field: "pack.id",
                ..
            }
        ));
    }
}

#[cfg(test)]
mod sovereign_deployment_tests {
    use super::*;

    const KR_FSC_AIRGAP_DEPLOYMENT_MODEL_JSON: &str =
        include_str!("../tests/fixtures/sovereign-airgap/kr-fsc-deployment-model.json");

    #[test]
    fn sovereign_air_gapped_deployment_fixture_declares_atomic_exit_fields() {
        let manifest =
            parse_sovereign_deployment_model_manifest(KR_FSC_AIRGAP_DEPLOYMENT_MODEL_JSON)
                .expect("KR FSC air-gap deployment model fixture should parse");

        assert_eq!(manifest.deployment_model.id, "kr-fsc-airgap-single-cell");
        assert_eq!(
            manifest.deployment_model.kind,
            SovereignDeploymentModelKind::SovereignAirGapped
        );
        assert_eq!(
            manifest.pack_overlay.canonical_matrix_pack_id,
            "pack-kr-fsc"
        );
        assert!(manifest.pack_overlay.air_gap);
        assert!(manifest.artifact_bundle.bundle_ref.ends_with(".oab"));
        assert_eq!(
            manifest
                .no_external_egress_validation
                .allowed_external_hosts,
            Vec::<String>::new()
        );
        assert_eq!(
            manifest
                .no_external_egress_validation
                .external_api_egress_policy,
            "DENY"
        );
        assert_eq!(manifest.ownership.service_owner, "axis-cloud");
        assert_eq!(manifest.recovery_objectives.rto, "PT4H");
        assert!(
            manifest
                .slo_targets
                .iter()
                .any(|target| target.id == "zero-external-egress-violations")
        );
    }

    #[test]
    fn proposed_deployment_spectrum_adrs_remain_planning_context_only() {
        let manifest =
            parse_sovereign_deployment_model_manifest(KR_FSC_AIRGAP_DEPLOYMENT_MODEL_JSON)
                .expect("KR FSC air-gap deployment model fixture should parse");

        assert_eq!(
            manifest.source_authority.accepted_adrs,
            vec!["ADR-0164", "ADR-0171", "ADR-0240"]
        );
        assert_eq!(
            manifest.source_authority.planning_context_adrs,
            vec!["ADR-0248", "ADR-0253", "ADR-0254"]
        );
        assert!(
            !manifest
                .source_authority
                .accepted_adrs
                .contains(&"ADR-0248".to_string())
        );
        assert!(
            !manifest
                .source_authority
                .accepted_adrs
                .contains(&"ADR-0253".to_string())
        );
        assert!(
            !manifest
                .source_authority
                .accepted_adrs
                .contains(&"ADR-0254".to_string())
        );
    }

    #[test]
    fn rejects_airgap_fixture_with_external_egress_allowlist() {
        let candidate = KR_FSC_AIRGAP_DEPLOYMENT_MODEL_JSON.replace(
            "\"allowed_external_hosts\": []",
            "\"allowed_external_hosts\": [\"api.openai.com\"]",
        );

        let error = parse_sovereign_deployment_model_manifest(&candidate)
            .expect_err("air-gap fixtures must not allow external hosts");

        assert!(matches!(
            error,
            SovereignDeploymentModelManifestError::Shape {
                field: "no_external_egress_validation.allowed_external_hosts",
                ..
            }
        ));
    }

    #[test]
    fn rejects_unsigned_or_non_oab_bundle_reference() {
        let candidate = KR_FSC_AIRGAP_DEPLOYMENT_MODEL_JSON
            .replace("20260709.oab", "20260709.tar")
            .replace("20260709.oab.sig", "20260709.tar.sig");

        let error = parse_sovereign_deployment_model_manifest(&candidate)
            .expect_err("sovereign air-gap fixtures require signed .oab references");

        assert!(matches!(
            error,
            SovereignDeploymentModelManifestError::Shape {
                field: "artifact_bundle.bundle_ref",
                ..
            }
        ));
    }
}
