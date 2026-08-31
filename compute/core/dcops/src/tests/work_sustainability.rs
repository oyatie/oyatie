use super::support::*;
use crate::*;
use data_boundary_kernel::DataClass;

#[test]
fn work_orders_require_safe_state_machine_and_privacy_class() {
    let mut catalog = active_catalog();
    let equipment_id = received_equipment(&mut catalog, EQUIP_ID);
    let work_order = catalog
        .open_work_order(WorkOrderCreate {
            id: "wo/dc/region-alpha1/site-a/wo-a".to_string(),
            site_id: SITE_ID.to_string(),
            equipment_id: Some(equipment_id.value.clone()),
            kind: WorkOrderKind::Install,
            priority: WorkOrderPriority::P1,
            state: WorkOrderState::Open,
            opened_by: "usr_operator".to_string(),
            assigned_to: None,
            safety_plan_ref: "safety/site-a/install".to_string(),
            data_class: DataClass::PiiQuasiIdentifier,
            opened_at_epoch_seconds: 70,
        })
        .expect("work order");
    assert_eq!(
        WorkOrder::new(WorkOrderCreate {
            id: "wo/dc/region-alpha1/site-a/wo-b".to_string(),
            site_id: SITE_ID.to_string(),
            equipment_id: None,
            kind: WorkOrderKind::Audit,
            priority: WorkOrderPriority::P3,
            state: WorkOrderState::Completed,
            opened_by: "usr_operator".to_string(),
            assigned_to: None,
            safety_plan_ref: "safety/site-a/audit".to_string(),
            data_class: DataClass::Audit,
            opened_at_epoch_seconds: 70,
        })
        .expect_err("state and data class are not accepted"),
        CloudDcopsError::InvalidInitialState
    );
    let assigned = catalog
        .assign_work_order(&work_order.id.value, "usr_tech".to_string(), 71)
        .expect("assigned");
    assert_eq!(assigned.state.value, WorkOrderState::Assigned);
    catalog
        .start_work_order(&work_order.id.value, 72)
        .expect("started");
    let completed = catalog
        .complete_work_order(
            &work_order.id.value,
            WorkOrderResolution {
                completed_by: "usr_tech".to_string(),
                resolution_ref: "resolution/site-a/wo-a".to_string(),
                completed_at_epoch_seconds: 73,
            },
        )
        .expect("completed");
    assert_eq!(completed.state.value, WorkOrderState::Completed);
}

#[test]
fn sustainability_snapshot_verifies_exact_ratios_and_targets() {
    let mut catalog = active_catalog();
    let snapshot = catalog
        .record_sustainability_snapshot(SustainabilitySnapshotCreate {
            id: "sustainability/dc/region-alpha1/site-a/day-1".to_string(),
            site_id: SITE_ID.to_string(),
            period_start_epoch_seconds: 100,
            period_end_epoch_seconds: 200,
            it_energy_kwh_milli: 1_000,
            facility_energy_kwh_milli: 1_500,
            water_liters_milli: 2_000,
            carbon_grams: 1_000,
            pue_milli: 1_500,
            wue_milli: 2_000,
            cue_milli: 1_000,
            data_class: DataClass::InternalOnly,
        })
        .expect("snapshot");
    assert_eq!(snapshot.pue_milli.value, 1_500);
    assert_eq!(
        SustainabilitySnapshot::new(SustainabilitySnapshotCreate {
            id: "sustainability/dc/region-alpha1/site-a/day-2".to_string(),
            site_id: SITE_ID.to_string(),
            period_start_epoch_seconds: 100,
            period_end_epoch_seconds: 200,
            it_energy_kwh_milli: 1_000,
            facility_energy_kwh_milli: 1_400,
            water_liters_milli: 2_000,
            carbon_grams: 1_000,
            pue_milli: 1_500,
            wue_milli: 2_000,
            cue_milli: 1_000,
            data_class: DataClass::InternalOnly,
        })
        .expect_err("provided ratios must equal source measurements"),
        CloudDcopsError::InvalidTargetRatio
    );
}
