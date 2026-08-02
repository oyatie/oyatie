#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests for the vendor-lockin discipline validator.
//!
//! These tests use the real registry file shipped at
//! `registry/vendor-lockin-phaseout/index.json` to ensure the validator
//! passes on the current state of the codebase (per ADR-0173).

use check_vendor_lockin_discipline::{
    VendorEntry, VendorLockinError, VendorTier, parse_registry_json, validate_registry,
};

const REGISTRY_PATH: &str = "../../registry/vendor-lockin-phaseout/index.json";

#[test]
fn real_registry_parses_and_validates() {
    let source = std::fs::read_to_string(REGISTRY_PATH).expect("registry must be readable");
    let entries = parse_registry_json(&source).expect("registry must parse");
    assert!(
        entries.len() >= 30,
        "vendor inventory must have >=30 entries (ADR-0173 quality bar); got {}",
        entries.len()
    );
    let report = validate_registry(&entries).expect("registry must validate");
    assert!(
        report.tier_i_count >= 10,
        "expected >=10 Tier-I entries; got {}",
        report.tier_i_count
    );
    assert!(
        report.tier_ii_count >= 5,
        "expected >=5 Tier-II entries; got {}",
        report.tier_ii_count
    );
    assert!(
        report.tier_iii_count >= 5,
        "expected >=5 Tier-III entries; got {}",
        report.tier_iii_count
    );
    assert!(
        report.seam_impls_total >= report.tier_ii_count,
        "every Tier-II entry must declare >=1 seam impl"
    );
}

#[test]
fn real_registry_anthropic_api_has_multi_impl_seam() {
    let source = std::fs::read_to_string(REGISTRY_PATH).expect("registry must be readable");
    let entries = parse_registry_json(&source).expect("registry must parse");
    let anthropic = entries
        .iter()
        .find(|entry| entry.name == "anthropic-api")
        .expect("anthropic-api entry must exist");
    assert_eq!(anthropic.tier, VendorTier::TierII);
    assert!(
        anthropic.seam_adapter_impls.len() >= 2,
        "anthropic-api seam must register >=2 impls (multi-vendor); got {}",
        anthropic.seam_adapter_impls.len()
    );
    assert!(
        anthropic
            .seam_adapter_trait
            .as_ref()
            .is_some_and(|trait_path| {
                trait_path.starts_with("crates/") || trait_path.starts_with("microservices/")
            }),
        "seam trait must point into the workspace"
    );
}

#[test]
fn real_registry_no_forbidden_tier_iii_silently_promoted() {
    let source = std::fs::read_to_string(REGISTRY_PATH).expect("registry must be readable");
    let entries = parse_registry_json(&source).expect("registry must parse");
    for entry in entries
        .iter()
        .filter(|entry| entry.tier == VendorTier::TierIII)
    {
        assert!(
            entry.adoption_rationale.to_uppercase().contains("REFUSED"),
            "Tier III entry {} must carry REFUSED rationale",
            entry.name
        );
    }
}

#[test]
fn validate_rejects_tier_ii_without_seam_trait() {
    let bad = VendorEntry {
        name: "phantom-vendor".to_owned(),
        tier: VendorTier::TierII,
        license: Some("proprietary".to_owned()),
        steward: Some("v".to_owned()),
        adoption_rationale: "x".to_owned(),
        replacement_path: Some("y".to_owned()),
        replacement_readiness_gate: Some("z".to_owned()),
        seam_adapter_trait: None,
        seam_adapter_impls: vec!["crates/x".to_owned()],
        phase_out_target_date_or_signal: Some("signal".to_owned()),
    };
    assert_eq!(
        validate_registry(&[bad]),
        Err(VendorLockinError::TierIIMissingSeamTrait(
            "phantom-vendor".to_owned()
        ))
    );
}

#[test]
fn validate_rejects_tier_ii_with_zero_impls() {
    let bad = VendorEntry {
        name: "phantom-vendor".to_owned(),
        tier: VendorTier::TierII,
        license: Some("proprietary".to_owned()),
        steward: Some("v".to_owned()),
        adoption_rationale: "x".to_owned(),
        replacement_path: Some("y".to_owned()),
        replacement_readiness_gate: Some("z".to_owned()),
        seam_adapter_trait: Some("crates/oya-x-kernel".to_owned()),
        seam_adapter_impls: vec![],
        phase_out_target_date_or_signal: Some("signal".to_owned()),
    };
    assert_eq!(
        validate_registry(&[bad]),
        Err(VendorLockinError::TierIIMissingSeamImpl(
            "phantom-vendor".to_owned()
        ))
    );
}

#[test]
fn validate_distinguishes_pre_classified_from_adopted() {
    let pre = VendorEntry {
        name: "cloudflare-cdn".to_owned(),
        tier: VendorTier::TierIIPreClassified,
        license: Some("proprietary".to_owned()),
        steward: Some("Cloudflare".to_owned()),
        adoption_rationale: "NOT ADOPTED. Pre-classified.".to_owned(),
        replacement_path: Some("self-hosted Envoy".to_owned()),
        replacement_readiness_gate: Some("edge POPs".to_owned()),
        seam_adapter_trait: Some("crates/oya-edge-cdn-kernel".to_owned()),
        seam_adapter_impls: vec![],
        phase_out_target_date_or_signal: Some("NOT ADOPTED".to_owned()),
    };
    let report = validate_registry(&[pre]).unwrap();
    assert_eq!(report.tier_ii_pre_count, 1);
    assert_eq!(report.tier_ii_count, 0);
}

#[test]
fn parse_registry_json_handles_extra_top_level_fields() {
    let json = r#"{
      "$schema": "vendor-lockin-phaseout/v1",
      "adr": "ADR-0173",
      "schema_version": "1.0.0",
      "last_audited": "2026-05-18",
      "tier_definitions": {"I": "...", "II": "...", "III": "..."},
      "entries": [
        {
          "name": "postgres",
          "tier": "I",
          "license": "PostgreSQL License",
          "steward": "PGDG",
          "adoption_rationale": "canonical OLTP"
        }
      ]
    }"#;
    let parsed = parse_registry_json(json).unwrap();
    assert_eq!(parsed.len(), 1);
}

#[test]
fn parse_registry_json_rejects_truncated_input() {
    let json = r#"{ "entries": [ { "name": "x", "tier": "I", "#;
    assert!(parse_registry_json(json).is_err());
}
