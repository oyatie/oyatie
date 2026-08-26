use network_residency_policy::{
    Classified, PerPackResidency, PerPackResidencyCreate, RegionJurisdiction, RegionRef,
    RegionRefCreate, RegulatorOverlay, RegulatorOverlayCreate, ResidencyClass, ResidencyError,
    parse_residency_class_label, residency_class_allows_home_region_label,
};

fn regulator_overlay() -> RegulatorOverlay {
    RegulatorOverlay::new(RegulatorOverlayCreate {
        regulator_refs: vec!["regulator/storage".to_string()],
        evidence_ref: "evidence/residency/storage".to_string(),
    })
    .expect("regulator overlay fixture should be valid")
}

fn per_pack_residency() -> PerPackResidency {
    PerPackResidency::new(PerPackResidencyCreate {
        allowed_primary_regions: vec!["region-alpha1".to_string()],
        allowed_replica_regions: vec!["region-beta1".to_string()],
        forbidden_regions: vec!["region-gamma1".to_string()],
        regulator_overlay: regulator_overlay(),
    })
    .expect("per-pack residency fixture should be valid")
}

#[test]
fn public_policy_surface_is_signature_closed() {
    let per_pack: PerPackResidency = per_pack_residency();
    let primary_regions: &Classified<Vec<String>> = &per_pack.allowed_primary_regions;
    assert_eq!(primary_regions.value, ["region-alpha1"]);

    let primary: RegionRef = RegionRef::new(RegionRefCreate {
        region_id: "region-alpha1".to_string(),
        jurisdiction: RegionJurisdiction::Home,
        cell_group_ref: "cell-group-alpha1".to_string(),
    })
    .expect("region fixture should be valid");
    assert!(per_pack.allows_primary_region(&primary));

    let policy: ResidencyClass = ResidencyClass::PerPack(Box::new(per_pack));
    assert_eq!(policy.label(), None);
}

#[test]
fn builtin_labels_and_parser_behavior_remain_exact() {
    for (label, expected) in [
        ("strict_home_region", ResidencyClass::StrictHomeRegion),
        (
            "home_with_recovery_failover",
            ResidencyClass::HomeWithRecoveryFailover,
        ),
        ("global", ResidencyClass::Global),
    ] {
        assert_eq!(parse_residency_class_label(label), Some(expected));
    }
    assert_eq!(
        parse_residency_class_label(" strict_home_region "),
        Some(ResidencyClass::StrictHomeRegion)
    );
    assert_eq!(parse_residency_class_label("per_pack"), None);
}

#[test]
fn home_region_predicate_behavior_remains_exact() {
    assert!(residency_class_allows_home_region_label(
        &ResidencyClass::StrictHomeRegion,
        "region-home-1"
    ));
    assert!(!residency_class_allows_home_region_label(
        &ResidencyClass::StrictHomeRegion,
        "region-homebrew"
    ));
    assert!(residency_class_allows_home_region_label(
        &ResidencyClass::Global,
        "region-anywhere"
    ));
    assert!(residency_class_allows_home_region_label(
        &ResidencyClass::PerPack(Box::new(per_pack_residency())),
        "REGION-ALPHA1"
    ));
}

#[test]
fn validation_failures_preserve_exact_typed_errors() {
    assert_eq!(
        RegulatorOverlay::new(RegulatorOverlayCreate {
            regulator_refs: Vec::new(),
            evidence_ref: "evidence/residency/storage".to_string(),
        }),
        Err(ResidencyError::EmptyRegulatorSet)
    );
    assert_eq!(
        PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec!["region-alpha1".to_string()],
            allowed_replica_regions: vec!["region-beta1".to_string()],
            forbidden_regions: vec!["region-alpha1".to_string()],
            regulator_overlay: regulator_overlay(),
        }),
        Err(ResidencyError::ForbiddenRegionOverlap)
    );
}
