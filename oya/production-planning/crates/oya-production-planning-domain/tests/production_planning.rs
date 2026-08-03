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
    assert_eq!(
        work_definition.work_definition_id.value.value,
        "wdef_laptop_standard"
    );
    assert_eq!(work_definition.tenant_id.value.value, "ten_enterprise");
    assert_eq!(work_definition.legal_entity_id.value.value, "le_us001");
    assert_eq!(work_definition.plant_id.value.value, "plant_us001");
    assert_eq!(
        work_definition.product_item_id.value.value,
        "item_laptop_finished"
    );
    assert_eq!(work_definition.bom_id.value.value, "bom_laptop_standard");
    assert_eq!(
        work_definition.route_id.value.value,
        "route_laptop_assembly"
    );
    assert_eq!(
        work_definition.work_center_id.value.value,
        "wc_assembly_line_1"
    );
    assert_eq!(work_definition.state.value, WorkDefinitionState::Approved);
    assert_eq!(work_definition.component_count.value, 12);
    assert_eq!(work_definition.total_component_quantity.value, 36);
    assert_eq!(work_definition.standard_run_minutes.value, 45);
    assert_eq!(work_definition.effective_from_yyyymmdd.value, 20260523);
    assert_eq!(
        work_definition.bom_source_ref.value.value,
        "src/production-planning/bom/laptop-standard"
    );
    assert_eq!(
        work_definition.routing_source_ref.value.value,
        "src/production-planning/route/laptop-assembly"
    );
    assert_eq!(
        work_definition.approval_evidence_ref.value.value,
        "audit/production-planning/wdef_laptop_standard/approval"
    );
    assert_eq!(
        work_definition.idempotency_key.value,
        "production-planning:work-definition:ten_enterprise:le_us001:plant_us001:wdef_laptop_standard"
    );
    assert!(!work_definition.shop_floor_execution_attached.value);
    assert!(!work_definition.inventory_mutation_attached.value);
    assert!(!work_definition.cloud_deployment_attached.value);
    assert_eq!(work_definition.schema_version.value, 1);

    let mrp = plan_material_requirements(mrp_input(true)).unwrap();
    assert_eq!(mrp.planned_order_id.value.value, "plord_laptop_june");
    assert_eq!(mrp.work_definition_id.value.value, "wdef_laptop_standard");
    assert_eq!(mrp.tenant_id.value.value, "ten_enterprise");
    assert_eq!(mrp.legal_entity_id.value.value, "le_us001");
    assert_eq!(mrp.plant_id.value.value, "plant_us001");
    assert_eq!(mrp.product_item_id.value.value, "item_laptop_finished");
    assert_eq!(mrp.state.value, MrpPlanState::PlannedOrderProposed);
    assert_eq!(mrp.demand_quantity.value, 100);
    assert_eq!(mrp.on_hand_quantity.value, 20);
    assert_eq!(mrp.scheduled_receipt_quantity.value, 10);
    assert_eq!(mrp.safety_stock_quantity.value, 10);
    assert_eq!(mrp.net_requirement_quantity.value, 80);
    assert_eq!(mrp.planned_order_quantity.value, 100);
    assert_eq!(mrp.lot_size_multiple.value, 25);
    assert_eq!(mrp.planning_horizon_days.value, 30);
    assert_eq!(
        mrp.demand_signal_ref.value.value,
        "src/demand/laptops/2026-06"
    );
    assert_eq!(
        mrp.planning_run_evidence_ref.value.value,
        "audit/production-planning/plord_laptop_june/mrp"
    );
    assert_eq!(
        mrp.idempotency_key.value,
        "production-planning:mrp:ten_enterprise:le_us001:plant_us001:plord_laptop_june"
    );
    assert!(mrp.production_order_release_allowed.value);
    assert!(!mrp.procurement_purchase_order_attached.value);
    assert!(!mrp.inventory_mutation_attached.value);
    assert!(!mrp.cloud_deployment_attached.value);
    assert_eq!(mrp.schema_version.value, 1);

    let release = prepare_production_release(release_input(true)).unwrap();
    assert_eq!(release.production_order_id.value.value, "prod_laptop_june");
    assert_eq!(release.planned_order_id.value.value, "plord_laptop_june");
    assert_eq!(
        release.work_definition_id.value.value,
        "wdef_laptop_standard"
    );
    assert_eq!(release.tenant_id.value.value, "ten_enterprise");
    assert_eq!(release.legal_entity_id.value.value, "le_us001");
    assert_eq!(release.plant_id.value.value, "plant_us001");
    assert_eq!(release.product_item_id.value.value, "item_laptop_finished");
    assert_eq!(release.state.value, ProductionReleaseState::ReleasePrepared);
    assert_eq!(release.required_quantity.value, 100);
    assert_eq!(release.material_available_quantity.value, 100);
    assert_eq!(release.material_remaining_quantity.value, 0);
    assert_eq!(release.required_capacity_minutes.value, 900);
    assert_eq!(release.work_center_capacity_minutes.value, 1_000);
    assert_eq!(release.capacity_remaining_minutes.value, 100);
    assert_eq!(
        release.schedule_evidence_ref.value.value,
        "audit/production-planning/prod_laptop_june/schedule"
    );
    assert_eq!(
        release.material_availability_evidence_ref.value.value,
        "audit/production-planning/prod_laptop_june/material-availability"
    );
    assert_eq!(
        release.capacity_evidence_ref.value.value,
        "audit/production-planning/prod_laptop_june/capacity"
    );
    assert_eq!(
        release.idempotency_key.value,
        "production-planning:release:ten_enterprise:le_us001:plant_us001:prod_laptop_june"
    );
    assert!(release.release_allowed.value);
    assert!(!release.shop_floor_execution_attached.value);
    assert!(!release.inventory_mutation_attached.value);
    assert!(!release.accounting_posting_attached.value);
    assert!(!release.workflow_execution_attached.value);
    assert!(!release.cloud_deployment_attached.value);
    assert_eq!(release.schema_version.value, 1);
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
    let mut prefix_only_id = work_definition_input();
    prefix_only_id.work_definition_id = "wdef_".to_owned();
    assert_eq!(
        approve_work_definition(prefix_only_id),
        Err(ProductionPlanningError::InvalidWorkDefinitionId)
    );

    let mut whitespace_identifier = work_definition_input();
    whitespace_identifier.plant_id = "plant_us 001".to_owned();
    assert_eq!(
        approve_work_definition(whitespace_identifier),
        Err(ProductionPlanningError::InvalidPlantId)
    );

    let mut control_identifier = work_definition_input();
    control_identifier.legal_entity_id = "le_us001\u{0007}".to_owned();
    assert_eq!(
        approve_work_definition(control_identifier),
        Err(ProductionPlanningError::InvalidLegalEntityId)
    );

    let mut prefix_only_source_ref = work_definition_input();
    prefix_only_source_ref.bom_source_ref = "src/".to_owned();
    assert_eq!(
        approve_work_definition(prefix_only_source_ref),
        Err(ProductionPlanningError::InvalidSourceDocumentRef)
    );

    let mut prefix_only_evidence_ref = work_definition_input();
    prefix_only_evidence_ref.approval_evidence_ref = "audit/".to_owned();
    assert_eq!(
        approve_work_definition(prefix_only_evidence_ref),
        Err(ProductionPlanningError::InvalidEvidenceRef)
    );

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

    let mut credential_source_ref = mrp_input(true);
    credential_source_ref.demand_signal_ref = "src/demand/api-key".to_owned();
    assert_eq!(
        plan_material_requirements(credential_source_ref),
        Err(ProductionPlanningError::InvalidSourceDocumentRef)
    );

    let mut bad_horizon = mrp_input(true);
    bad_horizon.planning_horizon_days = 0;
    assert_eq!(
        plan_material_requirements(bad_horizon),
        Err(ProductionPlanningError::InvalidPlanningHorizon)
    );

    let mut bad_lot = mrp_input(true);
    bad_lot.lot_size_multiple = 0;
    assert_eq!(
        plan_material_requirements(bad_lot),
        Err(ProductionPlanningError::InvalidQuantity)
    );

    let mut bad_work_definition_quantity = work_definition_input();
    bad_work_definition_quantity.component_count = 0;
    assert_eq!(
        approve_work_definition(bad_work_definition_quantity),
        Err(ProductionPlanningError::InvalidQuantity)
    );

    let mut bad_release_quantity = release_input(true);
    bad_release_quantity.required_quantity = 0;
    assert_eq!(
        prepare_production_release(bad_release_quantity),
        Err(ProductionPlanningError::InvalidQuantity)
    );
}
