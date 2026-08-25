use cell_location::{AzCode, CellId, CellLocationError, RegionCode};

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
    assert_eq!(
        RegionCode::new("region_alpha1"),
        Err(CellLocationError::InvalidRegionCode)
    );
    assert_eq!(
        AzCode::new("region-alpha1-A"),
        Err(CellLocationError::InvalidAzCode)
    );
    assert_eq!(
        CellId::new("region-alpha1-a-001"),
        Err(CellLocationError::InvalidCellId)
    );
}

#[test]
fn agreed_location_contract_rejects_ambiguous_canonical_forms() {
    for value in ["", " region-alpha1", "region--alpha1", "-region", "region-"] {
        assert_eq!(
            RegionCode::new(value),
            Err(CellLocationError::InvalidRegionCode)
        );
    }
}
