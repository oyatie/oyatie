use oya_production_planning_domain::{
    MrpPlanInput, MrpPlanState, ProductionPlanningError, ProductionReleaseInput,
    ProductionReleaseState, WorkDefinitionInput, WorkDefinitionState, approve_work_definition,
    plan_material_requirements, prepare_production_release,
};

fn work_definition_input() -> WorkDefinitionInput {
    WorkDefinitionInput {
        work_definition_id: "wdef_laptop_standard".to_owned(),
        plant_id: "plant_us001".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        product_item_id: "item_laptop_finished".to_owned(),
        bom_id: "bom_laptop_standard".to_owned(),
        route_id: "route_laptop_assembly".to_owned(),
        work_center_id: "wc_assembly_line_1".to_owned(),
        component_count: 12,
        total_component_quantity: 36,
        standard_run_minutes: 45,
        effective_from_yyyymmdd: 20260523,
        bom_source_ref: "src/production-planning/bom/laptop-standard".to_owned(),
        routing_source_ref: "src/production-planning/route/laptop-assembly".to_owned(),
        approval_evidence_ref: "audit/production-planning/wdef_laptop_standard/approval".to_owned(),
    }
}

fn mrp_input(work_definition_approved: bool) -> MrpPlanInput {
    MrpPlanInput {
        planned_order_id: "plord_laptop_june".to_owned(),
        work_definition_id: "wdef_laptop_standard".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        product_item_id: "item_laptop_finished".to_owned(),
        work_definition_approved,
        demand_quantity: 100,
        on_hand_quantity: 20,
        scheduled_receipt_quantity: 10,
        safety_stock_quantity: 10,
        lot_size_multiple: 25,
        planning_horizon_days: 30,
        demand_signal_ref: "src/demand/laptops/2026-06".to_owned(),
        planning_run_evidence_ref: "audit/production-planning/plord_laptop_june/mrp".to_owned(),
    }
}

fn release_input(planned_order_proposed: bool) -> ProductionReleaseInput {
    ProductionReleaseInput {
        production_order_id: "prod_laptop_june".to_owned(),
        planned_order_id: "plord_laptop_june".to_owned(),
        work_definition_id: "wdef_laptop_standard".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        product_item_id: "item_laptop_finished".to_owned(),
        planned_order_proposed,
        required_quantity: 100,
        material_available_quantity: 100,
        required_capacity_minutes: 900,
        work_center_capacity_minutes: 1_000,
        schedule_evidence_ref: "audit/production-planning/prod_laptop_june/schedule".to_owned(),
        material_availability_evidence_ref:
            "audit/production-planning/prod_laptop_june/material-availability".to_owned(),
        capacity_evidence_ref: "audit/production-planning/prod_laptop_june/capacity".to_owned(),
    }
}

#[test]
fn work_definition_drives_mrp_and_release_plan() {
    let work_definition = approve_work_definition(work_definition_input()).unwrap();
    assert_eq!(work_definition.state.value, WorkDefinitionState::Approved);
    assert_eq!(work_definition.component_count.value, 12);
    assert!(!work_definition.shop_floor_execution_attached.value);
    assert!(!work_definition.inventory_mutation_attached.value);

    let mrp = plan_material_requirements(mrp_input(true)).unwrap();
    assert_eq!(mrp.state.value, MrpPlanState::PlannedOrderProposed);
    assert_eq!(mrp.net_requirement_quantity.value, 80);
    assert_eq!(mrp.planned_order_quantity.value, 100);
    assert!(mrp.production_order_release_allowed.value);
    assert!(!mrp.procurement_purchase_order_attached.value);
    assert!(!mrp.cloud_deployment_attached.value);

    let release = prepare_production_release(release_input(true)).unwrap();
    assert_eq!(release.state.value, ProductionReleaseState::ReleasePrepared);
    assert_eq!(release.capacity_remaining_minutes.value, 100);
    assert!(release.release_allowed.value);
    assert!(!release.shop_floor_execution_attached.value);
    assert!(!release.inventory_mutation_attached.value);
    assert!(!release.accounting_posting_attached.value);
}

#[test]
fn production_planning_refuses_unapproved_work_definition_and_unproposed_order() {
    assert_eq!(
        plan_material_requirements(mrp_input(false)),
        Err(ProductionPlanningError::WorkDefinitionApprovalRequired)
    );
    assert_eq!(
        prepare_production_release(release_input(false)),
        Err(ProductionPlanningError::PlannedOrderRequired)
    );
}

#[test]
fn production_planning_refuses_material_or_capacity_shortage() {
    let mut material_short = release_input(true);
    material_short.material_available_quantity = 99;
    assert_eq!(
        prepare_production_release(material_short),
        Err(ProductionPlanningError::InsufficientMaterialAvailability)
    );

    let mut capacity_short = release_input(true);
    capacity_short.work_center_capacity_minutes = 899;
    assert_eq!(
        prepare_production_release(capacity_short),
        Err(ProductionPlanningError::InsufficientCapacity)
    );
}

#[test]
fn production_planning_validates_refs_dates_and_quantities() {
    let mut unsafe_work_definition = work_definition_input();
    unsafe_work_definition.approval_evidence_ref =
        "audit/production-planning/secret-token".to_owned();
    assert_eq!(
        approve_work_definition(unsafe_work_definition),
        Err(ProductionPlanningError::InvalidEvidenceRef)
    );

    let mut bad_date = work_definition_input();
    bad_date.effective_from_yyyymmdd = 20261340;
    assert_eq!(
        approve_work_definition(bad_date),
        Err(ProductionPlanningError::InvalidEffectiveDate)
    );

    let mut bad_ref = mrp_input(true);
    bad_ref.demand_signal_ref = "src/../demand".to_owned();
    assert_eq!(
        plan_material_requirements(bad_ref),
        Err(ProductionPlanningError::InvalidSourceDocumentRef)
    );

    let mut bad_lot = mrp_input(true);
    bad_lot.lot_size_multiple = 0;
    assert_eq!(
        plan_material_requirements(bad_lot),
        Err(ProductionPlanningError::InvalidQuantity)
    );
}
