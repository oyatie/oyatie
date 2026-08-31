use super::support::*;
use crate::*;

#[test]
fn enforces_equipment_lifecycle_and_capacity_without_overlap() {
    let mut catalog = active_catalog();
    let equipment_id = received_equipment(&mut catalog, EQUIP_ID);
    catalog
        .install_equipment(&equipment_id, install_plan(1, 4_000))
        .expect("install first server");
    catalog
        .transition_equipment(&equipment_id, EquipmentLifecycle::InService, 40)
        .expect("in service");
    let capacity = catalog
        .rack_capacity(&RackId::new(RACK_ID_VALUE).expect("rack id"))
        .expect("capacity");
    assert_eq!(capacity.used_u, 2);
    assert_eq!(capacity.remaining_power_watts, 8_000);

    let other_id = received_equipment(&mut catalog, EQUIP_ID_B);
    assert_eq!(
        catalog
            .install_equipment(&other_id, install_plan(2, 1_000))
            .expect_err("U ranges overlap"),
        CloudDcopsError::RackUnitOverlap
    );
    assert_eq!(
        catalog
            .install_equipment(&other_id, install_plan(3, 9_000))
            .expect_err("rack power budget exceeded"),
        CloudDcopsError::RackCapacityExceeded
    );
}

#[test]
fn ewaste_transferred_equipment_releases_installation_capacity() {
    let mut catalog = active_catalog();
    let retired_id = received_equipment(&mut catalog, EQUIP_ID);
    catalog
        .install_equipment(&retired_id, install_plan(1, 10_000))
        .expect("install retired server");
    catalog
        .transition_equipment(&retired_id, EquipmentLifecycle::InService, 40)
        .expect("retired in service");
    catalog
        .transition_equipment(&retired_id, EquipmentLifecycle::Decommissioning, 41)
        .expect("retired decommissioning");
    catalog
        .transition_equipment(&retired_id, EquipmentLifecycle::Sanitized, 42)
        .expect("retired sanitized");
    catalog
        .transition_equipment(&retired_id, EquipmentLifecycle::EwasteTransferred, 43)
        .expect("retired transferred");

    let replacement_id = received_equipment(&mut catalog, EQUIP_ID_B);
    let mut replacement_plan = install_plan(1, 12_000);
    replacement_plan.installed_at_epoch_seconds = 44;
    catalog
        .install_equipment(&replacement_id, replacement_plan)
        .expect("ewaste-transferred equipment releases rack, power, and cooling budgets");

    let capacity = catalog
        .rack_capacity(&RackId::new(RACK_ID_VALUE).expect("rack id"))
        .expect("capacity");
    assert_eq!(capacity.used_u, 2);
    assert_eq!(capacity.remaining_power_watts, 0);
    assert_eq!(capacity.remaining_heat_watts, 0);
}

#[test]
fn rejects_installing_equipment_before_receipt() {
    let mut catalog = active_catalog();
    let equipment = catalog
        .order_equipment(equipment_create(EQUIP_ID))
        .expect("ordered");
    assert_eq!(
        catalog
            .install_equipment(&equipment.id.value, install_plan(1, 1_000))
            .expect_err("ordered equipment is not installable"),
        CloudDcopsError::InvalidStateTransition
    );
}
