use cell_location::{AzCode, CellId, RegionCode};

#[test]
fn agreed_location_contract_preserves_validated_identity_shapes() {
    let region = RegionCode::new("region-alpha1").expect("region fixture is canonical");
    let az = AzCode::new("region-alpha1-a").expect("AZ fixture is canonical");
    let cell = CellId::new("cell-region-alpha1-a-001").expect("cell fixture is canonical");

    assert_eq!(region.value, "region-alpha1");
    assert_eq!(az.value, "region-alpha1-a");
    assert_eq!(cell.value, "cell-region-alpha1-a-001");
}

#[test]
fn agreed_location_contract_preserves_fail_closed_validation() {
    assert!(RegionCode::new("region_alpha1").is_err());
    assert!(AzCode::new("region-alpha1-A").is_err());
    assert!(CellId::new("region-alpha1-a-001").is_err());
}
