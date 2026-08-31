use super::support::*;
use crate::*;

#[test]
fn maps_and_certifies_network_cables_with_loss_budget() {
    let mut catalog = active_catalog();
    let first = received_equipment(&mut catalog, EQUIP_ID);
    let second = received_equipment(&mut catalog, EQUIP_ID_B);
    catalog
        .install_equipment(&first, install_plan(1, 2_000))
        .expect("first install");
    catalog
        .install_equipment(&second, install_plan(3, 2_000))
        .expect("second install");
    let cable = catalog
        .add_cable_run(CableRunCreate {
            id: "cable/dc/region-alpha1/site-a/cable-a".to_string(),
            site_id: SITE_ID.to_string(),
            from: CableEndpoint {
                equipment_id: EQUIP_ID.to_string(),
                port_name: "eth0".to_string(),
            },
            to: CableEndpoint {
                equipment_id: EQUIP_ID_B.to_string(),
                port_name: "eth0".to_string(),
            },
            media: CableMedia::SingleModeFiber,
            state: CableState::Planned,
            measured_loss_milli_db: 1_000,
            loss_budget_milli_db: 3_000,
            created_at_epoch_seconds: 50,
        })
        .expect("cable");
    let cable = catalog
        .transition_cable_run(&cable.id.value, CableState::Installed, 51)
        .expect("installed");
    assert_eq!(cable.state.value, CableState::Installed);
    assert_eq!(
        CableRun::new(CableRunCreate {
            id: "cable/dc/region-alpha1/site-a/cable-b".to_string(),
            site_id: SITE_ID.to_string(),
            from: CableEndpoint {
                equipment_id: EQUIP_ID.to_string(),
                port_name: "eth1".to_string(),
            },
            to: CableEndpoint {
                equipment_id: EQUIP_ID_B.to_string(),
                port_name: "eth1".to_string(),
            },
            media: CableMedia::SingleModeFiber,
            state: CableState::Planned,
            measured_loss_milli_db: 4_000,
            loss_budget_milli_db: 3_000,
            created_at_epoch_seconds: 50,
        })
        .expect_err("measured loss must fit budget"),
        CloudDcopsError::InvalidCableLoss
    );
}

#[test]
fn records_bms_readings_only_for_enabled_points_once() {
    let mut catalog = active_catalog();
    let point = catalog
        .add_bms_point(BmsPointCreate {
            id: "bms/dc/region-alpha1/site-a/temp-a".to_string(),
            site_id: SITE_ID.to_string(),
            equipment_id: None,
            kind: BmsPointKind::Temperature,
            state: BmsPointState::Commissioning,
            unit: "milli-celsius".to_string(),
            created_at_epoch_seconds: 60,
        })
        .expect("point");
    assert_eq!(
        catalog
            .record_bms_reading(BmsReadingCreate {
                point_id: point.id.value.value.clone(),
                site_id: SITE_ID.to_string(),
                observed_at_epoch_seconds: 61,
                milli_value: 22_000,
            })
            .expect_err("disabled point rejects reading"),
        CloudDcopsError::InactiveParent
    );
    catalog
        .transition_bms_point(&point.id.value, BmsPointState::Enabled, 61)
        .expect("enabled");
    catalog
        .record_bms_reading(BmsReadingCreate {
            point_id: point.id.value.value.clone(),
            site_id: SITE_ID.to_string(),
            observed_at_epoch_seconds: 62,
            milli_value: 22_000,
        })
        .expect("reading");
    assert_eq!(
        catalog
            .record_bms_reading(BmsReadingCreate {
                point_id: point.id.value.value,
                site_id: SITE_ID.to_string(),
                observed_at_epoch_seconds: 62,
                milli_value: 22_100,
            })
            .expect_err("duplicate point timestamp rejected"),
        CloudDcopsError::DuplicateBmsReading
    );
}

#[test]
fn bms_reading_store_enforces_bounded_retention() {
    let mut catalog = active_catalog();
    catalog.bms_reading_retention_limit = 1;
    let point = catalog
        .add_bms_point(BmsPointCreate {
            id: "bms/dc/region-alpha1/site-a/temp-retention".to_string(),
            site_id: SITE_ID.to_string(),
            equipment_id: None,
            kind: BmsPointKind::Temperature,
            state: BmsPointState::Commissioning,
            unit: "milli-celsius".to_string(),
            created_at_epoch_seconds: 71,
        })
        .expect("point");
    catalog
        .transition_bms_point(&point.id.value, BmsPointState::Enabled, 72)
        .expect("point enabled");

    catalog
        .record_bms_reading(BmsReadingCreate {
            point_id: point.id.value.value.clone(),
            site_id: SITE_ID.to_string(),
            observed_at_epoch_seconds: 73,
            milli_value: 22_000,
        })
        .expect("first reading");
    catalog
        .record_bms_reading(BmsReadingCreate {
            point_id: point.id.value.value.clone(),
            site_id: SITE_ID.to_string(),
            observed_at_epoch_seconds: 74,
            milli_value: 22_100,
        })
        .expect("second reading");

    assert_eq!(catalog.bms_reading_count(), 1);
}
