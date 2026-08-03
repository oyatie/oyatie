use oya_plant_maintenance_domain::{
    EquipmentAssetInput, EquipmentAssetState, MaintenanceCriticality, MaintenancePriority,
    MaintenanceWorkOrderCompletionInput, MaintenanceWorkOrderInput, MaintenanceWorkOrderState,
    PlantMaintenanceError, PreventiveMaintenancePlanInput, PreventiveMaintenancePlanState,
    approve_preventive_maintenance_plan, complete_maintenance_work_order, register_equipment_asset,
    release_maintenance_work_order,
};

fn equipment_input() -> EquipmentAssetInput {
    EquipmentAssetInput {
        equipment_id: "equip_press_line_01".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        functional_location_id: "floc_assembly_line_01".to_owned(),
        criticality: MaintenanceCriticality::SafetyCritical,
        installed_on_yyyymmdd: 20250115,
        warranty_until_yyyymmdd: Some(20280115),
        asset_source_ref: "src/plant-maintenance/equipment/press-line-01".to_owned(),
        registration_evidence_ref: "audit/plant-maintenance/equip_press_line_01/register"
            .to_owned(),
    }
}

fn plan_input(equipment_registered: bool) -> PreventiveMaintenancePlanInput {
    PreventiveMaintenancePlanInput {
        maintenance_plan_id: "mplan_press_line_01_monthly".to_owned(),
        equipment_id: "equip_press_line_01".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        functional_location_id: "floc_assembly_line_01".to_owned(),
        equipment_registered,
        interval_days: 30,
        lead_time_days: 5,
        estimated_labor_minutes: 120,
        required_spare_part_count: 4,
        next_due_yyyymmdd: 20260615,
        strategy_source_ref: "src/plant-maintenance/strategy/monthly-press-inspection".to_owned(),
        approval_evidence_ref: "audit/plant-maintenance/mplan_press_line_01_monthly/approval"
            .to_owned(),
    }
}

fn work_order_input(maintenance_plan_approved: bool) -> MaintenanceWorkOrderInput {
    MaintenanceWorkOrderInput {
        work_order_id: "mwo_press_line_01_june".to_owned(),
        maintenance_plan_id: "mplan_press_line_01_monthly".to_owned(),
        equipment_id: "equip_press_line_01".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        functional_location_id: "floc_assembly_line_01".to_owned(),
        maintenance_plan_approved,
        priority: MaintenancePriority::Urgent,
        planned_start_yyyymmdd: 20260610,
        planned_labor_minutes: 120,
        planned_spare_parts_quantity: 4,
        safety_permit_required: true,
        job_instruction_ref: "src/plant-maintenance/job/press-line-monthly".to_owned(),
        release_evidence_ref: "audit/plant-maintenance/mwo_press_line_01_june/release".to_owned(),
    }
}

fn completion_input(work_order_released: bool) -> MaintenanceWorkOrderCompletionInput {
    MaintenanceWorkOrderCompletionInput {
        work_order_id: "mwo_press_line_01_june".to_owned(),
        equipment_id: "equip_press_line_01".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        work_order_released,
        completion_yyyymmdd: 20260610,
        planned_labor_minutes: 120,
        actual_labor_minutes: 150,
        planned_spare_parts_quantity: 4,
        actual_spare_parts_quantity: 3,
        downtime_minutes: 45,
        measurement_evidence_ref: "audit/plant-maintenance/mwo_press_line_01_june/measurement"
            .to_owned(),
        completion_evidence_ref: "audit/plant-maintenance/mwo_press_line_01_june/complete"
            .to_owned(),
    }
}

#[test]
fn asset_master_drives_preventive_plan_work_order_and_completion() {
    let equipment = register_equipment_asset(equipment_input()).unwrap();
    assert_eq!(equipment.state.value, EquipmentAssetState::Registered);
    assert_eq!(
        equipment.criticality.value,
        MaintenanceCriticality::SafetyCritical
    );
    assert!(!equipment.durable_asset_registry_attached.value);
    assert!(!equipment.iot_or_scada_ingestion_attached.value);

    let plan = approve_preventive_maintenance_plan(plan_input(true)).unwrap();
    assert_eq!(plan.state.value, PreventiveMaintenancePlanState::Approved);
    assert_eq!(plan.interval_days.value, 30);
    assert_eq!(plan.lead_time_days.value, 5);
    assert!(!plan.scheduler_runtime_attached.value);
    assert!(!plan.inventory_reservation_attached.value);
    assert!(!plan.workflow_execution_attached.value);

    let work_order = release_maintenance_work_order(work_order_input(true)).unwrap();
    assert_eq!(work_order.state.value, MaintenanceWorkOrderState::Released);
    assert_eq!(work_order.planned_labor_minutes.value, 120);
    assert!(work_order.safety_permit_required.value);
    assert!(!work_order.procurement_requisition_attached.value);
    assert!(!work_order.technician_dispatch_attached.value);
    assert!(!work_order.cloud_deployment_attached.value);

    let completion = complete_maintenance_work_order(completion_input(true)).unwrap();
    assert_eq!(completion.state.value, MaintenanceWorkOrderState::Completed);
    assert_eq!(completion.labor_variance_minutes.value, 30);
    assert_eq!(completion.spare_parts_remaining_quantity.value, 1);
    assert!(completion.next_plan_recalculation_required.value);
    assert!(!completion.accounting_posting_attached.value);
    assert!(!completion.equipment_meter_write_attached.value);
    assert!(!completion.runtime_audit_chain_emission_attached.value);
}

#[test]
fn plant_maintenance_preview_non_claim_contract_is_explicit() {
    let equipment = register_equipment_asset(equipment_input()).unwrap();
    assert_eq!(
        equipment.idempotency_key.value,
        "plant-maintenance:equipment:ten_enterprise:le_us001:plant_us001:equip_press_line_01"
    );
    assert_eq!(equipment.schema_version.value, 1);
    assert!(!equipment.durable_asset_registry_attached.value);
    assert!(!equipment.iot_or_scada_ingestion_attached.value);
    assert!(!equipment.cloud_deployment_attached.value);

    let plan = approve_preventive_maintenance_plan(plan_input(true)).unwrap();
    assert_eq!(
        plan.idempotency_key.value,
        "plant-maintenance:plan:ten_enterprise:le_us001:plant_us001:mplan_press_line_01_monthly"
    );
    assert_eq!(plan.schema_version.value, 1);
    assert!(!plan.scheduler_runtime_attached.value);
    assert!(!plan.inventory_reservation_attached.value);
    assert!(!plan.workflow_execution_attached.value);
    assert!(!plan.cloud_deployment_attached.value);

    let work_order = release_maintenance_work_order(work_order_input(true)).unwrap();
    assert_eq!(
        work_order.idempotency_key.value,
        "plant-maintenance:work-order:ten_enterprise:le_us001:plant_us001:mwo_press_line_01_june"
    );
    assert_eq!(work_order.schema_version.value, 1);
    assert!(!work_order.inventory_reservation_attached.value);
    assert!(!work_order.procurement_requisition_attached.value);
    assert!(!work_order.technician_dispatch_attached.value);
    assert!(!work_order.workflow_execution_attached.value);
    assert!(!work_order.cloud_deployment_attached.value);

    let completion = complete_maintenance_work_order(completion_input(true)).unwrap();
    assert_eq!(
        completion.idempotency_key.value,
        "plant-maintenance:completion:ten_enterprise:le_us001:plant_us001:mwo_press_line_01_june"
    );
    assert_eq!(completion.schema_version.value, 1);
    assert!(!completion.accounting_posting_attached.value);
    assert!(!completion.equipment_meter_write_attached.value);
    assert!(!completion.runtime_audit_chain_emission_attached.value);
    assert!(!completion.cloud_deployment_attached.value);
}

#[test]
fn plant_maintenance_refuses_unregistered_unapproved_and_unreleased_flow() {
    assert_eq!(
        approve_preventive_maintenance_plan(plan_input(false)),
        Err(PlantMaintenanceError::EquipmentRegistrationRequired)
    );
    assert_eq!(
        release_maintenance_work_order(work_order_input(false)),
        Err(PlantMaintenanceError::MaintenancePlanApprovalRequired)
    );
    assert_eq!(
        complete_maintenance_work_order(completion_input(false)),
        Err(PlantMaintenanceError::WorkOrderReleaseRequired)
    );
}

#[test]
fn plant_maintenance_validates_dates_refs_intervals_and_quantities() {
    let mut unsafe_equipment = equipment_input();
    unsafe_equipment.registration_evidence_ref = "audit/plant-maintenance/secret-token".to_owned();
    assert_eq!(
        register_equipment_asset(unsafe_equipment),
        Err(PlantMaintenanceError::InvalidEvidenceRef)
    );

    let mut bad_warranty = equipment_input();
    bad_warranty.warranty_until_yyyymmdd = Some(20240115);
    assert_eq!(
        register_equipment_asset(bad_warranty),
        Err(PlantMaintenanceError::InvalidDate)
    );

    let mut bad_interval = plan_input(true);
    bad_interval.lead_time_days = 30;
    assert_eq!(
        approve_preventive_maintenance_plan(bad_interval),
        Err(PlantMaintenanceError::InvalidInterval)
    );

    let mut bad_ref = work_order_input(true);
    bad_ref.job_instruction_ref = "src/../maintenance".to_owned();
    assert_eq!(
        release_maintenance_work_order(bad_ref),
        Err(PlantMaintenanceError::InvalidSourceDocumentRef)
    );

    let mut bad_labor = work_order_input(true);
    bad_labor.planned_labor_minutes = 0;
    assert_eq!(
        release_maintenance_work_order(bad_labor),
        Err(PlantMaintenanceError::InvalidQuantity)
    );
}

#[test]
fn plant_maintenance_refuses_ac05_boundary_values() {
    let mut prefix_only = equipment_input();
    prefix_only.equipment_id = "equip_".to_owned();
    assert_eq!(
        register_equipment_asset(prefix_only),
        Err(PlantMaintenanceError::InvalidEquipmentId)
    );

    let mut whitespace_tenant = equipment_input();
    whitespace_tenant.tenant_id = "ten_enterprise north".to_owned();
    assert_eq!(
        register_equipment_asset(whitespace_tenant),
        Err(PlantMaintenanceError::InvalidTenantId)
    );

    let mut impossible_date = equipment_input();
    impossible_date.installed_on_yyyymmdd = 20260231;
    assert_eq!(
        register_equipment_asset(impossible_date),
        Err(PlantMaintenanceError::InvalidDate)
    );

    let mut control_ref = equipment_input();
    control_ref.registration_evidence_ref = "audit/plant-maintenance/bad\nref".to_owned();
    assert_eq!(
        register_equipment_asset(control_ref),
        Err(PlantMaintenanceError::InvalidEvidenceRef)
    );
}

#[test]
fn plant_maintenance_refuses_spare_part_over_consumption() {
    let mut completion = completion_input(true);
    completion.actual_spare_parts_quantity = 5;
    assert_eq!(
        complete_maintenance_work_order(completion),
        Err(PlantMaintenanceError::SparePartOverConsumption)
    );
}
