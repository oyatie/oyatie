use oya_data_boundary_kernel::{Classified, DataClass};
use oya_supply_chain_planning_domain::{
    AtpResponseInput, AtpResponseState, DemandPlanInput, DemandPlanState, DemandPlanningMethod,
    DistributionLanePlanInput, DistributionPlanState, SupplyChainPlanningError,
    SupplyNetworkPlanInput, SupplyPlanState, approve_demand_plan,
    prepare_available_to_promise_response, prepare_distribution_lane_plan,
    propose_supply_network_plan,
};

fn demand_plan_input() -> DemandPlanInput {
    DemandPlanInput {
        demand_plan_id: "dplan_laptop_q3_consensus".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        planning_area_id: "pa_global_laptops".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        demand_location_id: "loc_us_east_dc".to_owned(),
        method: DemandPlanningMethod::SalesAndOperationsConsensus,
        baseline_forecast_quantity: 100,
        firm_sales_order_quantity: 25,
        consensus_adjustment_quantity: -5,
        planning_horizon_days: 90,
        effective_from_yyyymmdd: 20260701,
        forecast_source_ref: "src/supply-chain-planning/forecast/laptop-q3".to_owned(),
        consensus_evidence_ref: "audit/supply-chain-planning/dplan_laptop_q3_consensus/approval"
            .to_owned(),
    }
}

fn supply_plan_input(demand_plan_approved: bool) -> SupplyNetworkPlanInput {
    SupplyNetworkPlanInput {
        supply_plan_id: "splan_laptop_q3_network".to_owned(),
        demand_plan_id: "dplan_laptop_q3_consensus".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        planning_area_id: "pa_global_laptops".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        source_location_id: "loc_us_plant".to_owned(),
        demand_location_id: "loc_us_east_dc".to_owned(),
        demand_plan_approved,
        consensus_demand_quantity: 120,
        on_hand_quantity: 20,
        scheduled_receipt_quantity: 10,
        in_transit_quantity: 5,
        safety_stock_quantity: 10,
        capacity_available_quantity: 125,
        lot_size_multiple: 25,
        lead_time_days: 7,
        planning_run_evidence_ref: "audit/supply-chain-planning/splan_laptop_q3_network/run"
            .to_owned(),
    }
}

fn atp_input(supply_plan_proposed: bool) -> AtpResponseInput {
    AtpResponseInput {
        atp_response_id: "atp_laptop_order_01".to_owned(),
        supply_plan_id: "splan_laptop_q3_network".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        planning_area_id: "pa_global_laptops".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        request_location_id: "loc_us_east_dc".to_owned(),
        supply_plan_proposed,
        requested_quantity: 80,
        available_to_promise_quantity: 75,
        allocation_priority_score: 900,
        requested_ship_date_yyyymmdd: 20260715,
        promise_evidence_ref: "audit/supply-chain-planning/atp_laptop_order_01/promise".to_owned(),
    }
}

fn distribution_plan_input(supply_plan_proposed: bool) -> DistributionLanePlanInput {
    DistributionLanePlanInput {
        distribution_plan_id: "distplan_laptop_us_lane".to_owned(),
        supply_plan_id: "splan_laptop_q3_network".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        planning_area_id: "pa_global_laptops".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        source_location_id: "loc_us_plant".to_owned(),
        destination_location_id: "loc_us_east_dc".to_owned(),
        supply_plan_proposed,
        transfer_quantity: 100,
        lane_capacity_quantity: 120,
        lead_time_days: 3,
        estimated_freight_cost_cents: 125_000,
        lane_source_ref: "src/supply-chain-planning/lane/us-plant-to-east-dc".to_owned(),
        lane_evidence_ref: "audit/supply-chain-planning/distplan_laptop_us_lane/prepared"
            .to_owned(),
    }
}

fn assert_data_class<T>(field: &Classified<T>, expected: DataClass) {
    assert_eq!(field.data_class.compatibility_data_class(), expected);
}

#[test]
fn prd_ac_01_demand_plan_approval_metadata_and_non_claims_are_explicit() {
    let demand = approve_demand_plan(demand_plan_input()).unwrap();

    assert_eq!(
        demand.demand_plan_id.value.value,
        "dplan_laptop_q3_consensus"
    );
    assert_eq!(demand.tenant_id.value.value, "ten_enterprise");
    assert_eq!(demand.legal_entity_id.value.value, "le_us001");
    assert_eq!(demand.planning_area_id.value.value, "pa_global_laptops");
    assert_eq!(demand.item_id.value.value, "item_laptop_finished");
    assert_eq!(demand.demand_location_id.value.value, "loc_us_east_dc");
    assert_eq!(
        demand.method.value,
        DemandPlanningMethod::SalesAndOperationsConsensus
    );
    assert_eq!(demand.baseline_forecast_quantity.value, 100);
    assert_eq!(demand.firm_sales_order_quantity.value, 25);
    assert_eq!(demand.consensus_adjustment_quantity.value, -5);
    assert_eq!(demand.consensus_demand_quantity.value, 120);
    assert_eq!(demand.planning_horizon_days.value, 90);
    assert_eq!(demand.effective_from_yyyymmdd.value, 20260701);
    assert_eq!(
        demand.forecast_source_ref.value.value,
        "src/supply-chain-planning/forecast/laptop-q3"
    );
    assert_eq!(
        demand.consensus_evidence_ref.value.value,
        "audit/supply-chain-planning/dplan_laptop_q3_consensus/approval"
    );
    assert_eq!(demand.state.value, DemandPlanState::Approved);
    assert_eq!(
        demand.idempotency_key.value,
        "supply-chain-planning:demand:ten_enterprise:le_us001:pa_global_laptops:dplan_laptop_q3_consensus"
    );
    assert!(!demand.machine_learning_runtime_attached.value);
    assert!(!demand.collaboration_workflow_attached.value);
    assert!(!demand.cloud_deployment_attached.value);
    assert_eq!(demand.schema_version.value, 1);
    assert_data_class(&demand.tenant_id, DataClass::InternalOnly);
    assert_data_class(&demand.consensus_demand_quantity, DataClass::Financial);
    assert_data_class(&demand.machine_learning_runtime_attached, DataClass::Public);
}

#[test]
fn prd_ac_02_supply_network_proposal_metadata_and_non_claims_are_explicit() {
    let supply = propose_supply_network_plan(supply_plan_input(true)).unwrap();

    assert_eq!(supply.supply_plan_id.value.value, "splan_laptop_q3_network");
    assert_eq!(
        supply.demand_plan_id.value.value,
        "dplan_laptop_q3_consensus"
    );
    assert_eq!(supply.tenant_id.value.value, "ten_enterprise");
    assert_eq!(supply.legal_entity_id.value.value, "le_us001");
    assert_eq!(supply.planning_area_id.value.value, "pa_global_laptops");
    assert_eq!(supply.item_id.value.value, "item_laptop_finished");
    assert_eq!(supply.source_location_id.value.value, "loc_us_plant");
    assert_eq!(supply.demand_location_id.value.value, "loc_us_east_dc");
    assert_eq!(supply.consensus_demand_quantity.value, 120);
    assert_eq!(supply.on_hand_quantity.value, 20);
    assert_eq!(supply.scheduled_receipt_quantity.value, 10);
    assert_eq!(supply.in_transit_quantity.value, 5);
    assert_eq!(supply.safety_stock_quantity.value, 10);
    assert_eq!(supply.net_requirement_quantity.value, 95);
    assert_eq!(supply.planned_supply_quantity.value, 100);
    assert_eq!(supply.constrained_shortage_quantity.value, 0);
    assert_eq!(supply.capacity_available_quantity.value, 125);
    assert_eq!(supply.lot_size_multiple.value, 25);
    assert_eq!(supply.lead_time_days.value, 7);
    assert_eq!(
        supply.planning_run_evidence_ref.value.value,
        "audit/supply-chain-planning/splan_laptop_q3_network/run"
    );
    assert_eq!(supply.state.value, SupplyPlanState::Proposed);
    assert_eq!(
        supply.idempotency_key.value,
        "supply-chain-planning:supply:ten_enterprise:le_us001:pa_global_laptops:splan_laptop_q3_network"
    );
    assert!(supply.supply_plan_feasible.value);
    assert!(!supply.production_order_creation_attached.value);
    assert!(!supply.procurement_requisition_attached.value);
    assert!(!supply.inventory_mutation_attached.value);
    assert!(!supply.cloud_deployment_attached.value);
    assert_eq!(supply.schema_version.value, 1);
    assert_data_class(&supply.tenant_id, DataClass::InternalOnly);
    assert_data_class(&supply.planned_supply_quantity, DataClass::Financial);
    assert_data_class(
        &supply.production_order_creation_attached,
        DataClass::Public,
    );
}

#[test]
fn prd_ac_03_atp_response_metadata_and_non_claims_are_explicit() {
    let atp = prepare_available_to_promise_response(atp_input(true)).unwrap();

    assert_eq!(atp.atp_response_id.value.value, "atp_laptop_order_01");
    assert_eq!(atp.supply_plan_id.value.value, "splan_laptop_q3_network");
    assert_eq!(atp.tenant_id.value.value, "ten_enterprise");
    assert_eq!(atp.legal_entity_id.value.value, "le_us001");
    assert_eq!(atp.planning_area_id.value.value, "pa_global_laptops");
    assert_eq!(atp.item_id.value.value, "item_laptop_finished");
    assert_eq!(atp.request_location_id.value.value, "loc_us_east_dc");
    assert_eq!(atp.requested_quantity.value, 80);
    assert_eq!(atp.available_to_promise_quantity.value, 75);
    assert_eq!(atp.promised_quantity.value, 75);
    assert_eq!(atp.backorder_quantity.value, 5);
    assert_eq!(atp.allocation_priority_score.value, 900);
    assert_eq!(atp.requested_ship_date_yyyymmdd.value, 20260715);
    assert_eq!(
        atp.promise_evidence_ref.value.value,
        "audit/supply-chain-planning/atp_laptop_order_01/promise"
    );
    assert_eq!(atp.state.value, AtpResponseState::Prepared);
    assert_eq!(
        atp.idempotency_key.value,
        "supply-chain-planning:atp:ten_enterprise:le_us001:pa_global_laptops:atp_laptop_order_01"
    );
    assert!(atp.backorder_required.value);
    assert!(!atp.order_management_reschedule_attached.value);
    assert!(!atp.warehouse_reservation_attached.value);
    assert!(!atp.cloud_deployment_attached.value);
    assert_eq!(atp.schema_version.value, 1);
    assert_data_class(&atp.tenant_id, DataClass::InternalOnly);
    assert_data_class(&atp.promised_quantity, DataClass::Financial);
    assert_data_class(&atp.order_management_reschedule_attached, DataClass::Public);
}

#[test]
fn prd_ac_04_distribution_lane_metadata_and_non_claims_are_explicit() {
    let distribution = prepare_distribution_lane_plan(distribution_plan_input(true)).unwrap();

    assert_eq!(
        distribution.distribution_plan_id.value.value,
        "distplan_laptop_us_lane"
    );
    assert_eq!(
        distribution.supply_plan_id.value.value,
        "splan_laptop_q3_network"
    );
    assert_eq!(distribution.tenant_id.value.value, "ten_enterprise");
    assert_eq!(distribution.legal_entity_id.value.value, "le_us001");
    assert_eq!(
        distribution.planning_area_id.value.value,
        "pa_global_laptops"
    );
    assert_eq!(distribution.item_id.value.value, "item_laptop_finished");
    assert_eq!(distribution.source_location_id.value.value, "loc_us_plant");
    assert_eq!(
        distribution.destination_location_id.value.value,
        "loc_us_east_dc"
    );
    assert_eq!(distribution.transfer_quantity.value, 100);
    assert_eq!(distribution.lane_capacity_quantity.value, 120);
    assert_eq!(distribution.capacity_remaining_quantity.value, 20);
    assert_eq!(distribution.lead_time_days.value, 3);
    assert_eq!(distribution.estimated_freight_cost_cents.value, 125_000);
    assert_eq!(
        distribution.lane_source_ref.value.value,
        "src/supply-chain-planning/lane/us-plant-to-east-dc"
    );
    assert_eq!(
        distribution.lane_evidence_ref.value.value,
        "audit/supply-chain-planning/distplan_laptop_us_lane/prepared"
    );
    assert_eq!(distribution.state.value, DistributionPlanState::Prepared);
    assert_eq!(
        distribution.idempotency_key.value,
        "supply-chain-planning:distribution:ten_enterprise:le_us001:pa_global_laptops:distplan_laptop_us_lane"
    );
    assert!(distribution.lane_capacity_sufficient.value);
    assert!(!distribution.carrier_booking_attached.value);
    assert!(!distribution.warehouse_transfer_order_attached.value);
    assert!(!distribution.cloud_deployment_attached.value);
    assert_eq!(distribution.schema_version.value, 1);
    assert_data_class(&distribution.tenant_id, DataClass::InternalOnly);
    assert_data_class(&distribution.transfer_quantity, DataClass::Financial);
    assert_data_class(&distribution.carrier_booking_attached, DataClass::Public);
}

#[test]
fn demand_plan_drives_supply_network_and_atp_response() {
    let demand = approve_demand_plan(demand_plan_input()).unwrap();
    assert_eq!(demand.state.value, DemandPlanState::Approved);
    assert_eq!(demand.consensus_demand_quantity.value, 120);
    assert_eq!(demand.effective_from_yyyymmdd.value, 20260701);
    assert!(!demand.machine_learning_runtime_attached.value);
    assert!(!demand.collaboration_workflow_attached.value);

    let supply = propose_supply_network_plan(supply_plan_input(true)).unwrap();
    assert_eq!(supply.state.value, SupplyPlanState::Proposed);
    assert_eq!(supply.net_requirement_quantity.value, 95);
    assert_eq!(supply.planned_supply_quantity.value, 100);
    assert_eq!(supply.constrained_shortage_quantity.value, 0);
    assert!(supply.supply_plan_feasible.value);
    assert!(!supply.production_order_creation_attached.value);
    assert!(!supply.procurement_requisition_attached.value);
    assert!(!supply.inventory_mutation_attached.value);

    let atp = prepare_available_to_promise_response(atp_input(true)).unwrap();
    assert_eq!(atp.state.value, AtpResponseState::Prepared);
    assert_eq!(atp.promised_quantity.value, 75);
    assert_eq!(atp.backorder_quantity.value, 5);
    assert!(atp.backorder_required.value);
    assert!(!atp.order_management_reschedule_attached.value);
    assert!(!atp.warehouse_reservation_attached.value);

    let distribution = prepare_distribution_lane_plan(distribution_plan_input(true)).unwrap();
    assert_eq!(distribution.state.value, DistributionPlanState::Prepared);
    assert_eq!(distribution.capacity_remaining_quantity.value, 20);
    assert!(distribution.lane_capacity_sufficient.value);
    assert!(!distribution.carrier_booking_attached.value);
    assert!(!distribution.warehouse_transfer_order_attached.value);
    assert!(!distribution.cloud_deployment_attached.value);
}

#[test]
fn supply_chain_planning_refuses_unapproved_and_unproposed_flow() {
    assert_eq!(
        propose_supply_network_plan(supply_plan_input(false)),
        Err(SupplyChainPlanningError::DemandPlanApprovalRequired)
    );
    assert_eq!(
        prepare_available_to_promise_response(atp_input(false)),
        Err(SupplyChainPlanningError::SupplyPlanRequired)
    );
    assert_eq!(
        prepare_distribution_lane_plan(distribution_plan_input(false)),
        Err(SupplyChainPlanningError::SupplyPlanRequired)
    );
}

#[test]
fn supply_chain_planning_validates_refs_dates_horizons_and_quantities() {
    let mut prefix_only_demand_id = demand_plan_input();
    prefix_only_demand_id.demand_plan_id = "dplan_".to_owned();
    assert_eq!(
        approve_demand_plan(prefix_only_demand_id),
        Err(SupplyChainPlanningError::InvalidDemandPlanId)
    );

    let mut prefix_only_supply_id = supply_plan_input(true);
    prefix_only_supply_id.supply_plan_id = "splan_".to_owned();
    assert_eq!(
        propose_supply_network_plan(prefix_only_supply_id),
        Err(SupplyChainPlanningError::InvalidSupplyPlanId)
    );

    let mut prefix_only_atp_id = atp_input(true);
    prefix_only_atp_id.atp_response_id = "atp_".to_owned();
    assert_eq!(
        prepare_available_to_promise_response(prefix_only_atp_id),
        Err(SupplyChainPlanningError::InvalidAtpResponseId)
    );

    let mut prefix_only_distribution_id = distribution_plan_input(true);
    prefix_only_distribution_id.distribution_plan_id = "distplan_".to_owned();
    assert_eq!(
        prepare_distribution_lane_plan(prefix_only_distribution_id),
        Err(SupplyChainPlanningError::InvalidDistributionPlanId)
    );

    let mut whitespace_tenant = demand_plan_input();
    whitespace_tenant.tenant_id = "ten_enter prise".to_owned();
    assert_eq!(
        approve_demand_plan(whitespace_tenant),
        Err(SupplyChainPlanningError::InvalidTenantId)
    );

    let mut control_character_location = supply_plan_input(true);
    control_character_location.source_location_id = "loc_us\nplant".to_owned();
    assert_eq!(
        propose_supply_network_plan(control_character_location),
        Err(SupplyChainPlanningError::InvalidLocationId)
    );

    let mut path_traversal_location = supply_plan_input(true);
    path_traversal_location.source_location_id = "loc_../plant".to_owned();
    assert_eq!(
        propose_supply_network_plan(path_traversal_location),
        Err(SupplyChainPlanningError::InvalidLocationId)
    );

    let mut unsafe_demand = demand_plan_input();
    unsafe_demand.consensus_evidence_ref = "audit/supply-chain-planning/secret-token".to_owned();
    assert_eq!(
        approve_demand_plan(unsafe_demand),
        Err(SupplyChainPlanningError::InvalidEvidenceRef)
    );

    let mut credential_shaped_source = demand_plan_input();
    credential_shaped_source.forecast_source_ref =
        "src/supply-chain-planning/forecast/api-key".to_owned();
    assert_eq!(
        approve_demand_plan(credential_shaped_source),
        Err(SupplyChainPlanningError::InvalidSourceDocumentRef)
    );

    let mut negative_consensus = demand_plan_input();
    negative_consensus.consensus_adjustment_quantity = -200;
    assert_eq!(
        approve_demand_plan(negative_consensus),
        Err(SupplyChainPlanningError::InvalidQuantity)
    );

    let mut bad_horizon = demand_plan_input();
    bad_horizon.planning_horizon_days = 0;
    assert_eq!(
        approve_demand_plan(bad_horizon),
        Err(SupplyChainPlanningError::InvalidPlanningHorizon)
    );

    let mut excessive_horizon = demand_plan_input();
    excessive_horizon.planning_horizon_days = 1_096;
    assert_eq!(
        approve_demand_plan(excessive_horizon),
        Err(SupplyChainPlanningError::InvalidPlanningHorizon)
    );

    let mut bad_effective_date = demand_plan_input();
    bad_effective_date.effective_from_yyyymmdd = 20260230;
    assert_eq!(
        approve_demand_plan(bad_effective_date),
        Err(SupplyChainPlanningError::InvalidDate)
    );

    let mut bad_lot = supply_plan_input(true);
    bad_lot.lot_size_multiple = 0;
    assert_eq!(
        propose_supply_network_plan(bad_lot),
        Err(SupplyChainPlanningError::InvalidQuantity)
    );

    let mut bad_lead_time = supply_plan_input(true);
    bad_lead_time.lead_time_days = 0;
    assert_eq!(
        propose_supply_network_plan(bad_lead_time),
        Err(SupplyChainPlanningError::InvalidLeadTime)
    );

    let mut bad_date = atp_input(true);
    bad_date.requested_ship_date_yyyymmdd = 20260230;
    assert_eq!(
        prepare_available_to_promise_response(bad_date),
        Err(SupplyChainPlanningError::InvalidDate)
    );

    let mut zero_requested_quantity = atp_input(true);
    zero_requested_quantity.requested_quantity = 0;
    assert_eq!(
        prepare_available_to_promise_response(zero_requested_quantity),
        Err(SupplyChainPlanningError::InvalidQuantity)
    );

    let mut invalid_priority = atp_input(true);
    invalid_priority.allocation_priority_score = 1_001;
    assert_eq!(
        prepare_available_to_promise_response(invalid_priority),
        Err(SupplyChainPlanningError::InvalidPriority)
    );

    let mut bad_ref = distribution_plan_input(true);
    bad_ref.lane_source_ref = "src/../lane".to_owned();
    assert_eq!(
        prepare_distribution_lane_plan(bad_ref),
        Err(SupplyChainPlanningError::InvalidSourceDocumentRef)
    );

    let mut bad_distribution_lead_time = distribution_plan_input(true);
    bad_distribution_lead_time.lead_time_days = 367;
    assert_eq!(
        prepare_distribution_lane_plan(bad_distribution_lead_time),
        Err(SupplyChainPlanningError::InvalidLeadTime)
    );
}

#[test]
fn supply_chain_planning_records_capacity_shortage_without_execution_claim() {
    let mut constrained = supply_plan_input(true);
    constrained.capacity_available_quantity = 50;
    let supply = propose_supply_network_plan(constrained).unwrap();
    assert_eq!(supply.planned_supply_quantity.value, 50);
    assert_eq!(supply.constrained_shortage_quantity.value, 50);
    assert!(!supply.supply_plan_feasible.value);

    let mut insufficient_lane = distribution_plan_input(true);
    insufficient_lane.lane_capacity_quantity = 99;
    assert_eq!(
        prepare_distribution_lane_plan(insufficient_lane),
        Err(SupplyChainPlanningError::InsufficientLaneCapacity)
    );
}
