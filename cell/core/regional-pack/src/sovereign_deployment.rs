//! Sovereign/air-gapped deployment model fixture manifest parsing.
//!
//! This module is deliberately fixture-scoped for SOV-001: it validates one
//! deployment-model manifest/evidence slice without claiming runtime deployment
//! activation, production readiness, measured SLOs, or a real signed bundle.
//! ADR-0164, ADR-0171, and ADR-0240 are accepted authority; ADR-0248,
//! ADR-0253, and ADR-0254 are planning context only.

use std::path::{Component, Path};

use serde::Deserialize;

pub const SOVEREIGN_DEPLOYMENT_MODEL_MANIFEST_SCHEMA_VERSION: u16 = 1;

const REQUIRED_ACCEPTED_ADRS: [&str; 3] = ["ADR-0164", "ADR-0171", "ADR-0240"];
const PLANNING_CONTEXT_ONLY_ADRS: [&str; 3] = ["ADR-0248", "ADR-0253", "ADR-0254"];
const REQUIRED_EXTERNAL_LLM_DENYLIST: [&str; 3] = ["Anthropic", "OpenAI", "Google Gemini"];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignDeploymentModelManifest {
    pub manifest_schema_version: u16, // data_class: INTERNAL_ONLY
    pub deployment_model: SovereignDeploymentModelIdentity, // data_class: INTERNAL_ONLY
    pub source_authority: SovereignDeploymentModelAuthority, // data_class: INTERNAL_ONLY
    pub pack_overlay: SovereignPackOverlayBinding, // data_class: INTERNAL_ONLY
    pub artifact_bundle: SovereignArtifactBundleEvidence, // data_class: INTERNAL_ONLY
    pub no_external_egress_validation: NoExternalEgressValidation, // data_class: INTERNAL_ONLY
    pub ownership: SovereignDeploymentOwnership, // data_class: INTERNAL_ONLY
    pub recovery_objectives: SovereignRecoveryObjectives, // data_class: INTERNAL_ONLY
    pub slo_targets: Vec<SovereignSloTarget>, // data_class: INTERNAL_ONLY
    pub claim_ceiling: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignDeploymentModelIdentity {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub kind: SovereignDeploymentModelKind, // data_class: INTERNAL_ONLY
    pub pack_id: String,                    // data_class: INTERNAL_ONLY
    pub jurisdiction: String,               // data_class: INTERNAL_ONLY
    pub home_region: String,                // data_class: INTERNAL_ONLY
    pub cell_topology: String,              // data_class: INTERNAL_ONLY
    pub status: String,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SovereignDeploymentModelKind {
    SovereignAirGapped,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignDeploymentModelAuthority {
    pub accepted_adrs: Vec<String>, // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub planning_context_adrs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignPackOverlayBinding {
    pub canonical_matrix_pack_id: String, // data_class: INTERNAL_ONLY
    pub air_gap: bool,                    // data_class: INTERNAL_ONLY
    pub sovereign_overlay_ref: String,    // data_class: INTERNAL_ONLY
    pub regulator: String,                // data_class: INTERNAL_ONLY
    pub data_classes: Vec<String>,        // data_class: INTERNAL_ONLY
    pub on_prem_substitutions: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignArtifactBundleEvidence {
    pub format: String,            // data_class: INTERNAL_ONLY
    pub bundle_ref: String,        // data_class: INTERNAL_ONLY
    pub digest: String,            // data_class: INTERNAL_ONLY
    pub signature_ref: String,     // data_class: INTERNAL_ONLY
    pub signature_profile: String, // data_class: INTERNAL_ONLY
    pub signing_status: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NoExternalEgressValidation {
    pub mode: String,                                  // data_class: INTERNAL_ONLY
    pub external_api_egress_policy: String,            // data_class: INTERNAL_ONLY
    pub allowed_external_hosts: Vec<String>,           // data_class: INTERNAL_ONLY
    pub service_entry_external_hosts_absent: bool,     // data_class: INTERNAL_ONLY
    pub cilium_l7_egress_deny: bool,                   // data_class: INTERNAL_ONLY
    pub forbidden_external_llm_providers: Vec<String>, // data_class: INTERNAL_ONLY
    pub validation_evidence_ref: String,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignDeploymentOwnership {
    pub service_owner: String,              // data_class: INTERNAL_ONLY
    pub operational_owner: String,          // data_class: INTERNAL_ONLY
    pub regulator_engagement_owner: String, // data_class: INTERNAL_ONLY
    pub evidence_owner: String,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignRecoveryObjectives {
    pub rto: String,     // data_class: INTERNAL_ONLY
    pub rpo: String,     // data_class: INTERNAL_ONLY
    pub dr_mode: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SovereignSloTarget {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub objective: String,       // data_class: INTERNAL_ONLY
    pub evidence_status: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SovereignDeploymentModelManifestError {
    ParseFailed { reason: String },
    Shape { field: &'static str, reason: String },
}

pub fn parse_sovereign_deployment_model_manifest(
    manifest_json: &str,
) -> Result<SovereignDeploymentModelManifest, SovereignDeploymentModelManifestError> {
    let manifest: SovereignDeploymentModelManifest =
        serde_json::from_str(manifest_json).map_err(|error| {
            SovereignDeploymentModelManifestError::ParseFailed {
                reason: error.to_string(),
            }
        })?;
    validate_sovereign_deployment_model_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_sovereign_deployment_model_manifest(
    manifest: &SovereignDeploymentModelManifest,
) -> Result<(), SovereignDeploymentModelManifestError> {
    if manifest.manifest_schema_version != SOVEREIGN_DEPLOYMENT_MODEL_MANIFEST_SCHEMA_VERSION {
        return shape_error(
            "manifest_schema_version",
            format!(
                "must equal {}",
                SOVEREIGN_DEPLOYMENT_MODEL_MANIFEST_SCHEMA_VERSION
            ),
        );
    }

    validate_deployment_model(&manifest.deployment_model)?;
    validate_authority(&manifest.source_authority)?;
    validate_pack_overlay(&manifest.deployment_model, &manifest.pack_overlay)?;
    validate_artifact_bundle(&manifest.artifact_bundle)?;
    validate_no_external_egress(&manifest.no_external_egress_validation)?;
    validate_ownership(&manifest.ownership)?;
    validate_recovery_objectives(&manifest.recovery_objectives)?;
    validate_slo_targets(&manifest.slo_targets)?;
    require_non_empty("claim_ceiling", &manifest.claim_ceiling)?;
    if !manifest.claim_ceiling.contains("fixture")
        || !manifest.claim_ceiling.contains("does not claim")
    {
        return shape_error(
            "claim_ceiling",
            "must state fixture scope and explicit non-claims",
        );
    }

    Ok(())
}

fn validate_deployment_model(
    deployment_model: &SovereignDeploymentModelIdentity,
) -> Result<(), SovereignDeploymentModelManifestError> {
    require_non_empty("deployment_model.id", &deployment_model.id)?;
    require_non_empty("deployment_model.pack_id", &deployment_model.pack_id)?;
    if !deployment_model.pack_id.starts_with("pack-") {
        return shape_error("deployment_model.pack_id", "must use canonical pack-* id");
    }
    require_non_empty(
        "deployment_model.jurisdiction",
        &deployment_model.jurisdiction,
    )?;
    if deployment_model.jurisdiction != deployment_model.jurisdiction.to_ascii_uppercase() {
        return shape_error(
            "deployment_model.jurisdiction",
            "must be uppercase jurisdiction code",
        );
    }
    require_non_empty(
        "deployment_model.home_region",
        &deployment_model.home_region,
    )?;
    require_non_empty(
        "deployment_model.cell_topology",
        &deployment_model.cell_topology,
    )?;
    require_exact(
        "deployment_model.status",
        &deployment_model.status,
        "fixture",
    )?;
    Ok(())
}

fn validate_authority(
    authority: &SovereignDeploymentModelAuthority,
) -> Result<(), SovereignDeploymentModelManifestError> {
    for adr in REQUIRED_ACCEPTED_ADRS {
        require_contains(
            "source_authority.accepted_adrs",
            &authority.accepted_adrs,
            adr,
        )?;
    }
    for adr in PLANNING_CONTEXT_ONLY_ADRS {
        require_contains(
            "source_authority.planning_context_adrs",
            &authority.planning_context_adrs,
            adr,
        )?;
        if authority.accepted_adrs.iter().any(|value| value == adr) {
            return shape_error(
                "source_authority.accepted_adrs",
                format!("{adr} must remain planning context only for this fixture"),
            );
        }
    }
    Ok(())
}

fn validate_pack_overlay(
    deployment_model: &SovereignDeploymentModelIdentity,
    overlay: &SovereignPackOverlayBinding,
) -> Result<(), SovereignDeploymentModelManifestError> {
    require_exact(
        "pack_overlay.canonical_matrix_pack_id",
        &overlay.canonical_matrix_pack_id,
        &deployment_model.pack_id,
    )?;
    if !overlay.air_gap {
        return shape_error(
            "pack_overlay.air_gap",
            "must be true for sovereign air-gap fixture",
        );
    }
    require_non_empty("pack_overlay.regulator", &overlay.regulator)?;
    ensure_repo_relative(
        "pack_overlay.sovereign_overlay_ref",
        &overlay.sovereign_overlay_ref,
    )?;
    if !overlay
        .sovereign_overlay_ref
        .ends_with("sovereign-cloud-overlay.yaml")
    {
        return shape_error(
            "pack_overlay.sovereign_overlay_ref",
            "must point to a sovereign-cloud-overlay.yaml source",
        );
    }
    require_non_empty_vec("pack_overlay.data_classes", &overlay.data_classes)?;
    require_contains(
        "pack_overlay.on_prem_substitutions",
        &overlay.on_prem_substitutions,
        "Harbor",
    )?;
    require_contains(
        "pack_overlay.on_prem_substitutions",
        &overlay.on_prem_substitutions,
        "OpenBao",
    )?;
    Ok(())
}

fn validate_artifact_bundle(
    bundle: &SovereignArtifactBundleEvidence,
) -> Result<(), SovereignDeploymentModelManifestError> {
    require_exact("artifact_bundle.format", &bundle.format, "oab")?;
    ensure_repo_relative("artifact_bundle.bundle_ref", &bundle.bundle_ref)?;
    if !bundle.bundle_ref.ends_with(".oab") {
        return shape_error("artifact_bundle.bundle_ref", "must end with .oab");
    }
    require_non_empty("artifact_bundle.digest", &bundle.digest)?;
    if !is_sha256_digest(&bundle.digest) {
        return shape_error(
            "artifact_bundle.digest",
            "must be a sha256:<64 lowercase hex> digest",
        );
    }
    ensure_repo_relative("artifact_bundle.signature_ref", &bundle.signature_ref)?;
    if !bundle.signature_ref.ends_with(".oab.sig") {
        return shape_error("artifact_bundle.signature_ref", "must end with .oab.sig");
    }
    require_non_empty(
        "artifact_bundle.signature_profile",
        &bundle.signature_profile,
    )?;
    require_non_empty("artifact_bundle.signing_status", &bundle.signing_status)?;
    Ok(())
}

fn validate_no_external_egress(
    validation: &NoExternalEgressValidation,
) -> Result<(), SovereignDeploymentModelManifestError> {
    require_exact(
        "no_external_egress_validation.mode",
        &validation.mode,
        "deny_by_default",
    )?;
    require_exact(
        "no_external_egress_validation.external_api_egress_policy",
        &validation.external_api_egress_policy,
        "DENY",
    )?;
    if !validation.allowed_external_hosts.is_empty() {
        return shape_error(
            "no_external_egress_validation.allowed_external_hosts",
            "must be empty for an air-gapped deployment fixture",
        );
    }
    if !validation.service_entry_external_hosts_absent {
        return shape_error(
            "no_external_egress_validation.service_entry_external_hosts_absent",
            "must be true",
        );
    }
    if !validation.cilium_l7_egress_deny {
        return shape_error(
            "no_external_egress_validation.cilium_l7_egress_deny",
            "must be true",
        );
    }
    for provider in REQUIRED_EXTERNAL_LLM_DENYLIST {
        require_contains(
            "no_external_egress_validation.forbidden_external_llm_providers",
            &validation.forbidden_external_llm_providers,
            provider,
        )?;
    }
    require_non_empty(
        "no_external_egress_validation.validation_evidence_ref",
        &validation.validation_evidence_ref,
    )?;
    Ok(())
}

fn validate_ownership(
    ownership: &SovereignDeploymentOwnership,
) -> Result<(), SovereignDeploymentModelManifestError> {
    require_non_empty("ownership.service_owner", &ownership.service_owner)?;
    require_non_empty("ownership.operational_owner", &ownership.operational_owner)?;
    require_non_empty(
        "ownership.regulator_engagement_owner",
        &ownership.regulator_engagement_owner,
    )?;
    require_non_empty("ownership.evidence_owner", &ownership.evidence_owner)?;
    Ok(())
}

fn validate_recovery_objectives(
    objectives: &SovereignRecoveryObjectives,
) -> Result<(), SovereignDeploymentModelManifestError> {
    require_duration("recovery_objectives.rto", &objectives.rto)?;
    require_duration("recovery_objectives.rpo", &objectives.rpo)?;
    require_non_empty("recovery_objectives.dr_mode", &objectives.dr_mode)?;
    Ok(())
}

fn validate_slo_targets(
    targets: &[SovereignSloTarget],
) -> Result<(), SovereignDeploymentModelManifestError> {
    if targets.is_empty() {
        return shape_error("slo_targets", "must contain at least one target");
    }
    for target in targets {
        require_non_empty("slo_targets.id", &target.id)?;
        require_non_empty("slo_targets.objective", &target.objective)?;
        require_exact(
            "slo_targets.evidence_status",
            &target.evidence_status,
            "target_fixture_only",
        )?;
    }
    Ok(())
}

fn require_duration(
    field: &'static str,
    value: &str,
) -> Result<(), SovereignDeploymentModelManifestError> {
    require_non_empty(field, value)?;
    if !value.starts_with("PT") {
        return shape_error(field, "must be an ISO-8601 time duration starting with PT");
    }
    Ok(())
}

fn require_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), SovereignDeploymentModelManifestError> {
    if value.trim().is_empty() {
        return shape_error(field, "must be non-empty");
    }
    Ok(())
}

fn require_non_empty_vec(
    field: &'static str,
    values: &[String],
) -> Result<(), SovereignDeploymentModelManifestError> {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        return shape_error(field, "must contain at least one non-empty value");
    }
    Ok(())
}

fn require_contains(
    field: &'static str,
    values: &[String],
    expected: &str,
) -> Result<(), SovereignDeploymentModelManifestError> {
    if !values.iter().any(|value| value == expected) {
        return shape_error(field, format!("must include {expected}"));
    }
    Ok(())
}

fn require_exact(
    field: &'static str,
    value: &str,
    expected: &str,
) -> Result<(), SovereignDeploymentModelManifestError> {
    if value != expected {
        return shape_error(field, format!("must equal {expected}"));
    }
    Ok(())
}

fn ensure_repo_relative(
    field: &'static str,
    path: &str,
) -> Result<(), SovereignDeploymentModelManifestError> {
    let path_ref = Path::new(path);
    if path.trim().is_empty() || path_ref.is_absolute() {
        return shape_error(field, "must be a non-empty repo-relative path");
    }
    if path_ref.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return shape_error(field, "must not escape the repository root");
    }
    Ok(())
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn shape_error<T>(
    field: &'static str,
    reason: impl Into<String>,
) -> Result<T, SovereignDeploymentModelManifestError> {
    Err(SovereignDeploymentModelManifestError::Shape {
        field,
        reason: reason.into(),
    })
}
