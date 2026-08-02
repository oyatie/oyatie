#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests for the vendor-lockin discipline validator.
//!
//! These tests use the real registry file shipped at
//! `registry/vendor-lockin-phaseout/index.json` to ensure the validator
//! passes on the current state of the codebase (per ADR-0173).

use check_vendor_lockin_discipline::{
    VendorEntry, VendorLockinError, VendorTier, parse_registry_json, validate_registry,
};

const REGISTRY_REL_PATH: &str = "registry/vendor-lockin-phaseout/index.json";

/// Walk up from `start` to the repo root (the standing live-corpus pattern).
///
/// Replaces a hardcoded `../../registry/...` that silently went stale when this
/// crate moved from `crates/<name>/` to `governance/check/<name>/` — one level
/// deeper, so the relative path resolved to `governance/registry/...`, which does
/// not exist. Nothing caught it because this file had no Buck2 target at all.
fn repo_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn read_registry() -> String {
    let path = repo_root().join(REGISTRY_REL_PATH);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("registry must be readable at {}: {error}", path.display()))
}

/// A gate that observes an EMPTY corpus is RED, never green.
///
/// `N == 0` means the probe went blind — moved corpus, stale path, renamed root —
/// which is precisely the failure this file sat in, undetected, while unwired.
/// The floor is reported on every call so the observed population is visible in
/// the test log instead of being implied by a bare pass.
fn population_floor(observed: usize, floor: usize, label: &str) -> Result<String, String> {
    if observed == 0 {
        return Err(format!(
            "{label}: observed ZERO — the probe is blind, not the corpus clean"
        ));
    }
    if observed < floor {
        return Err(format!("{label}: observed {observed}, below floor {floor}"));
    }
    Ok(format!("{label}: observed {observed} (floor {floor})"))
}

/// RED fixture for the rule above: an empty probe MUST fail.
#[test]
fn population_floor_refuses_an_empty_probe() {
    assert!(
        population_floor(0, 30, "vendor entries").is_err(),
        "N==0 must be RED — a blind probe cannot report green"
    );
    assert!(
        population_floor(29, 30, "vendor entries").is_err(),
        "below-floor must be RED"
    );
    assert!(population_floor(30, 30, "vendor entries").is_ok());
}

#[test]
fn real_registry_parses_and_validates() {
    let source = read_registry();
    let entries = parse_registry_json(&source).expect("registry must parse");
    let report = validate_registry(&entries).expect("registry must validate");
    // Every population this gate observes is asserted non-zero AND printed, so a
    // probe that goes blind reports RED instead of a vacuous green.
    for observation in [
        population_floor(entries.len(), 30, "vendor entries (ADR-0173 quality bar)"),
        population_floor(report.tier_i_count, 10, "Tier-I entries"),
        population_floor(report.tier_ii_count, 5, "Tier-II entries"),
        population_floor(report.tier_iii_count, 5, "Tier-III entries"),
    ] {
        println!("{}", observation.unwrap_or_else(|error| panic!("{error}")));
    }
    assert!(
        report.seam_impls_total >= report.tier_ii_count,
        "every Tier-II entry must declare >=1 seam impl"
    );
}

#[test]
fn real_registry_anthropic_api_has_multi_impl_seam() {
    let source = read_registry();
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
    let source = read_registry();
    let entries = parse_registry_json(&source).expect("registry must parse");
    let tier_iii: Vec<_> = entries
        .iter()
        .filter(|entry| entry.tier == VendorTier::TierIII)
        .collect();
    // Without this floor the loop below passes VACUOUSLY when the filter matches
    // nothing — green precisely because it checked nothing.
    println!(
        "{}",
        population_floor(tier_iii.len(), 5, "Tier-III entries scanned")
            .unwrap_or_else(|error| panic!("{error}"))
    );
    for entry in tier_iii {
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
