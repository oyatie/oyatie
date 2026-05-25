//! Supply-chain planning domain foundation.
//!
//! This crate owns pure supply-chain-planning invariants for consensus demand
//! planning, supply network plan proposal metadata, available-to-promise response
//! metadata, and distribution lane planning metadata. It does not perform durable
//! persistence, live planning solver execution, production order creation,
//! procurement requisition creation, inventory mutation, order-management
//! rescheduling, carrier booking, Workflow execution, runtime audit-chain
//! emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const DEMAND_PLAN_ID_PREFIX: &str = "dplan_";
const SUPPLY_PLAN_ID_PREFIX: &str = "splan_";
const ATP_RESPONSE_ID_PREFIX: &str = "atp_";
const DISTRIBUTION_PLAN_ID_PREFIX: &str = "distplan_";
const PLANNING_AREA_ID_PREFIX: &str = "pa_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const ITEM_ID_PREFIX: &str = "item_";
const LOCATION_ID_PREFIX: &str = "loc_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const SUPPLY_CHAIN_PLANNING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DemandPlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SupplyPlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AtpResponseId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DistributionPlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlanningAreaId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TenantId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LegalEntityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ItemId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceDocumentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DemandPlanningMethod {
    StatisticalForecast,
    SalesAndOperationsConsensus,
    DemandDrivenReplenishment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DemandPlanState {
    Approved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SupplyPlanState {
    Proposed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AtpResponseState {
    Prepared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DistributionPlanState {
    Prepared,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemandPlanInput {
    pub demand_plan_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,            // data_class: INTERNAL_ONLY
    pub planning_area_id: String,           // data_class: INTERNAL_ONLY
    pub item_id: String,                    // data_class: INTERNAL_ONLY
    pub demand_location_id: String,         // data_class: INTERNAL_ONLY
    pub method: DemandPlanningMethod,       // data_class: INTERNAL_ONLY
    pub baseline_forecast_quantity: u32,    // data_class: FINANCIAL
    pub firm_sales_order_quantity: u32,     // data_class: FINANCIAL
    pub consensus_adjustment_quantity: i32, // data_class: FINANCIAL
    pub planning_horizon_days: u16,         // data_class: INTERNAL_ONLY
    pub effective_from_yyyymmdd: u32,       // data_class: INTERNAL_ONLY
    pub forecast_source_ref: String,        // data_class: INTERNAL_ONLY
    pub consensus_evidence_ref: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemandPlanApproval {
    pub demand_plan_id: Classified<DemandPlanId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,          // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub planning_area_id: Classified<PlanningAreaId>, // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,              // data_class: INTERNAL_ONLY
    pub demand_location_id: Classified<LocationId>, // data_class: INTERNAL_ONLY
    pub method: Classified<DemandPlanningMethod>, // data_class: INTERNAL_ONLY
    pub baseline_forecast_quantity: Classified<u32>, // data_class: FINANCIAL
    pub firm_sales_order_quantity: Classified<u32>, // data_class: FINANCIAL
    pub consensus_adjustment_quantity: Classified<i32>, // data_class: FINANCIAL
    pub consensus_demand_quantity: Classified<u32>, // data_class: FINANCIAL
    pub planning_horizon_days: Classified<u16>,   // data_class: INTERNAL_ONLY
    pub effective_from_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub forecast_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub consensus_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<DemandPlanState>,       // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,      // data_class: INTERNAL_ONLY
    pub machine_learning_runtime_attached: Classified<bool>, // data_class: PUBLIC
    pub collaboration_workflow_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyNetworkPlanInput {
    pub supply_plan_id: String,            // data_class: INTERNAL_ONLY
    pub demand_plan_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub planning_area_id: String,          // data_class: INTERNAL_ONLY
    pub item_id: String,                   // data_class: INTERNAL_ONLY
    pub source_location_id: String,        // data_class: INTERNAL_ONLY
    pub demand_location_id: String,        // data_class: INTERNAL_ONLY
    pub demand_plan_approved: bool,        // data_class: INTERNAL_ONLY
    pub consensus_demand_quantity: u32,    // data_class: FINANCIAL
    pub on_hand_quantity: u32,             // data_class: FINANCIAL
    pub scheduled_receipt_quantity: u32,   // data_class: FINANCIAL
    pub in_transit_quantity: u32,          // data_class: FINANCIAL
    pub safety_stock_quantity: u32,        // data_class: FINANCIAL
    pub capacity_available_quantity: u32,  // data_class: FINANCIAL
    pub lot_size_multiple: u32,            // data_class: FINANCIAL
    pub lead_time_days: u16,               // data_class: INTERNAL_ONLY
    pub planning_run_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyNetworkPlanProposal {
    pub supply_plan_id: Classified<SupplyPlanId>, // data_class: INTERNAL_ONLY
    pub demand_plan_id: Classified<DemandPlanId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,          // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub planning_area_id: Classified<PlanningAreaId>, // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,              // data_class: INTERNAL_ONLY
    pub source_location_id: Classified<LocationId>, // data_class: INTERNAL_ONLY
    pub demand_location_id: Classified<LocationId>, // data_class: INTERNAL_ONLY
    pub consensus_demand_quantity: Classified<u32>, // data_class: FINANCIAL
    pub on_hand_quantity: Classified<u32>,        // data_class: FINANCIAL
    pub scheduled_receipt_quantity: Classified<u32>, // data_class: FINANCIAL
    pub in_transit_quantity: Classified<u32>,     // data_class: FINANCIAL
    pub safety_stock_quantity: Classified<u32>,   // data_class: FINANCIAL
    pub net_requirement_quantity: Classified<u32>, // data_class: FINANCIAL
    pub planned_supply_quantity: Classified<u32>, // data_class: FINANCIAL
    pub constrained_shortage_quantity: Classified<u32>, // data_class: FINANCIAL
    pub capacity_available_quantity: Classified<u32>, // data_class: FINANCIAL
    pub lot_size_multiple: Classified<u32>,       // data_class: FINANCIAL
    pub lead_time_days: Classified<u16>,          // data_class: INTERNAL_ONLY
    pub planning_run_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<SupplyPlanState>,       // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,      // data_class: INTERNAL_ONLY
    pub supply_plan_feasible: Classified<bool>,   // data_class: PUBLIC
    pub production_order_creation_attached: Classified<bool>, // data_class: PUBLIC
    pub procurement_requisition_attached: Classified<bool>, // data_class: PUBLIC
    pub inventory_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtpResponseInput {
    pub atp_response_id: String,            // data_class: INTERNAL_ONLY
    pub supply_plan_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,            // data_class: INTERNAL_ONLY
    pub planning_area_id: String,           // data_class: INTERNAL_ONLY
    pub item_id: String,                    // data_class: INTERNAL_ONLY
    pub request_location_id: String,        // data_class: INTERNAL_ONLY
    pub supply_plan_proposed: bool,         // data_class: INTERNAL_ONLY
    pub requested_quantity: u32,            // data_class: FINANCIAL
    pub available_to_promise_quantity: u32, // data_class: FINANCIAL
    pub allocation_priority_score: u16,     // data_class: FINANCIAL
    pub requested_ship_date_yyyymmdd: u32,  // data_class: INTERNAL_ONLY
    pub promise_evidence_ref: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtpResponsePreparation {
    pub atp_response_id: Classified<AtpResponseId>, // data_class: INTERNAL_ONLY
    pub supply_plan_id: Classified<SupplyPlanId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,            // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub planning_area_id: Classified<PlanningAreaId>, // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                // data_class: INTERNAL_ONLY
    pub request_location_id: Classified<LocationId>, // data_class: INTERNAL_ONLY
    pub requested_quantity: Classified<u32>,        // data_class: FINANCIAL
    pub available_to_promise_quantity: Classified<u32>, // data_class: FINANCIAL
    pub promised_quantity: Classified<u32>,         // data_class: FINANCIAL
    pub backorder_quantity: Classified<u32>,        // data_class: FINANCIAL
    pub allocation_priority_score: Classified<u16>, // data_class: FINANCIAL
    pub requested_ship_date_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub promise_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<AtpResponseState>,        // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,        // data_class: INTERNAL_ONLY
    pub backorder_required: Classified<bool>,       // data_class: PUBLIC
    pub order_management_reschedule_attached: Classified<bool>, // data_class: PUBLIC
    pub warehouse_reservation_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributionLanePlanInput {
    pub distribution_plan_id: String,      // data_class: INTERNAL_ONLY
    pub supply_plan_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub planning_area_id: String,          // data_class: INTERNAL_ONLY
    pub item_id: String,                   // data_class: INTERNAL_ONLY
    pub source_location_id: String,        // data_class: INTERNAL_ONLY
    pub destination_location_id: String,   // data_class: INTERNAL_ONLY
    pub supply_plan_proposed: bool,        // data_class: INTERNAL_ONLY
    pub transfer_quantity: u32,            // data_class: FINANCIAL
    pub lane_capacity_quantity: u32,       // data_class: FINANCIAL
    pub lead_time_days: u16,               // data_class: INTERNAL_ONLY
    pub estimated_freight_cost_cents: u64, // data_class: FINANCIAL
    pub lane_source_ref: String,           // data_class: INTERNAL_ONLY
    pub lane_evidence_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributionLanePlanPreparation {
    pub distribution_plan_id: Classified<DistributionPlanId>, // data_class: INTERNAL_ONLY
    pub supply_plan_id: Classified<SupplyPlanId>,             // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                      // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,           // data_class: INTERNAL_ONLY
    pub planning_area_id: Classified<PlanningAreaId>,         // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                          // data_class: INTERNAL_ONLY
    pub source_location_id: Classified<LocationId>,           // data_class: INTERNAL_ONLY
    pub destination_location_id: Classified<LocationId>,      // data_class: INTERNAL_ONLY
    pub transfer_quantity: Classified<u32>,                   // data_class: FINANCIAL
    pub lane_capacity_quantity: Classified<u32>,              // data_class: FINANCIAL
    pub capacity_remaining_quantity: Classified<u32>,         // data_class: FINANCIAL
    pub lead_time_days: Classified<u16>,                      // data_class: INTERNAL_ONLY
    pub estimated_freight_cost_cents: Classified<u64>,        // data_class: FINANCIAL
    pub lane_source_ref: Classified<SourceDocumentRef>,       // data_class: INTERNAL_ONLY
    pub lane_evidence_ref: Classified<EvidenceRef>,           // data_class: INTERNAL_ONLY
    pub state: Classified<DistributionPlanState>,             // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                  // data_class: INTERNAL_ONLY
    pub lane_capacity_sufficient: Classified<bool>,           // data_class: PUBLIC
    pub carrier_booking_attached: Classified<bool>,           // data_class: PUBLIC
    pub warehouse_transfer_order_attached: Classified<bool>,  // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,          // data_class: PUBLIC
    pub schema_version: Classified<u32>,                      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainPlanningError {
    InvalidDemandPlanId,
    InvalidSupplyPlanId,
    InvalidAtpResponseId,
    InvalidDistributionPlanId,
    InvalidPlanningAreaId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidItemId,
    InvalidLocationId,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidDate,
    InvalidPlanningHorizon,
    InvalidLeadTime,
    InvalidQuantity,
    InvalidPriority,
    DemandPlanApprovalRequired,
    SupplyPlanRequired,
    InsufficientLaneCapacity,
}

pub fn approve_demand_plan(
    input: DemandPlanInput,
) -> Result<DemandPlanApproval, SupplyChainPlanningError> {
    validate_demand_plan_id(&input.demand_plan_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_planning_area_id(&input.planning_area_id)?;
    validate_item_id(&input.item_id)?;
    validate_location_id(&input.demand_location_id)?;
    validate_positive_quantity(input.baseline_forecast_quantity)?;
    validate_horizon(input.planning_horizon_days)?;
    validate_yyyymmdd(input.effective_from_yyyymmdd)?;
    validate_source_ref(&input.forecast_source_ref)?;
    validate_evidence_ref(&input.consensus_evidence_ref)?;
    let consensus_demand_quantity = consensus_quantity(
        input.baseline_forecast_quantity,
        input.firm_sales_order_quantity,
        input.consensus_adjustment_quantity,
    )?;
    let idempotency_key = format!(
        "supply-chain-planning:demand:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.planning_area_id, input.demand_plan_id
    );

    Ok(DemandPlanApproval {
        demand_plan_id: internal(DemandPlanId {
            value: input.demand_plan_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        planning_area_id: internal(PlanningAreaId {
            value: input.planning_area_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        demand_location_id: internal(LocationId {
            value: input.demand_location_id,
        }),
        method: internal(input.method),
        baseline_forecast_quantity: financial(input.baseline_forecast_quantity),
        firm_sales_order_quantity: financial(input.firm_sales_order_quantity),
        consensus_adjustment_quantity: financial(input.consensus_adjustment_quantity),
        consensus_demand_quantity: financial(consensus_demand_quantity),
        planning_horizon_days: internal(input.planning_horizon_days),
        effective_from_yyyymmdd: internal(input.effective_from_yyyymmdd),
        forecast_source_ref: internal(SourceDocumentRef {
            value: input.forecast_source_ref,
        }),
        consensus_evidence_ref: internal(EvidenceRef {
            value: input.consensus_evidence_ref,
        }),
        state: internal(DemandPlanState::Approved),
        idempotency_key: internal(idempotency_key),
        machine_learning_runtime_attached: public(false),
        collaboration_workflow_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(SUPPLY_CHAIN_PLANNING_SCHEMA_VERSION),
    })
}

pub fn propose_supply_network_plan(
    input: SupplyNetworkPlanInput,
) -> Result<SupplyNetworkPlanProposal, SupplyChainPlanningError> {
    validate_supply_plan_id(&input.supply_plan_id)?;
    validate_demand_plan_id(&input.demand_plan_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_planning_area_id(&input.planning_area_id)?;
    validate_item_id(&input.item_id)?;
    validate_location_id(&input.source_location_id)?;
    validate_location_id(&input.demand_location_id)?;
    if !input.demand_plan_approved {
        return Err(SupplyChainPlanningError::DemandPlanApprovalRequired);
    }
    validate_positive_quantity(input.consensus_demand_quantity)?;
    validate_positive_quantity(input.lot_size_multiple)?;
    validate_lead_time(input.lead_time_days)?;
    validate_evidence_ref(&input.planning_run_evidence_ref)?;
    let gross_requirement = input
        .consensus_demand_quantity
        .checked_add(input.safety_stock_quantity)
        .ok_or(SupplyChainPlanningError::InvalidQuantity)?;
    let available_supply = input
        .on_hand_quantity
        .checked_add(input.scheduled_receipt_quantity)
        .and_then(|value| value.checked_add(input.in_transit_quantity))
        .ok_or(SupplyChainPlanningError::InvalidQuantity)?;
    let net_requirement_quantity = gross_requirement.saturating_sub(available_supply);
    let unconstrained_supply =
        round_up_to_multiple(net_requirement_quantity, input.lot_size_multiple)?;
    let planned_supply_quantity = unconstrained_supply.min(input.capacity_available_quantity);
    let constrained_shortage_quantity =
        unconstrained_supply.saturating_sub(planned_supply_quantity);
    let supply_plan_feasible = constrained_shortage_quantity == 0;
    let idempotency_key = format!(
        "supply-chain-planning:supply:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.planning_area_id, input.supply_plan_id
    );

    Ok(SupplyNetworkPlanProposal {
        supply_plan_id: internal(SupplyPlanId {
            value: input.supply_plan_id,
        }),
        demand_plan_id: internal(DemandPlanId {
            value: input.demand_plan_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        planning_area_id: internal(PlanningAreaId {
            value: input.planning_area_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        source_location_id: internal(LocationId {
            value: input.source_location_id,
        }),
        demand_location_id: internal(LocationId {
            value: input.demand_location_id,
        }),
        consensus_demand_quantity: financial(input.consensus_demand_quantity),
        on_hand_quantity: financial(input.on_hand_quantity),
        scheduled_receipt_quantity: financial(input.scheduled_receipt_quantity),
        in_transit_quantity: financial(input.in_transit_quantity),
        safety_stock_quantity: financial(input.safety_stock_quantity),
        net_requirement_quantity: financial(net_requirement_quantity),
        planned_supply_quantity: financial(planned_supply_quantity),
        constrained_shortage_quantity: financial(constrained_shortage_quantity),
        capacity_available_quantity: financial(input.capacity_available_quantity),
        lot_size_multiple: financial(input.lot_size_multiple),
        lead_time_days: internal(input.lead_time_days),
        planning_run_evidence_ref: internal(EvidenceRef {
            value: input.planning_run_evidence_ref,
        }),
        state: internal(SupplyPlanState::Proposed),
        idempotency_key: internal(idempotency_key),
        supply_plan_feasible: public(supply_plan_feasible),
        production_order_creation_attached: public(false),
        procurement_requisition_attached: public(false),
        inventory_mutation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(SUPPLY_CHAIN_PLANNING_SCHEMA_VERSION),
    })
}

pub fn prepare_available_to_promise_response(
    input: AtpResponseInput,
) -> Result<AtpResponsePreparation, SupplyChainPlanningError> {
    validate_atp_response_id(&input.atp_response_id)?;
    validate_supply_plan_id(&input.supply_plan_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_planning_area_id(&input.planning_area_id)?;
    validate_item_id(&input.item_id)?;
    validate_location_id(&input.request_location_id)?;
    if !input.supply_plan_proposed {
        return Err(SupplyChainPlanningError::SupplyPlanRequired);
    }
    validate_positive_quantity(input.requested_quantity)?;
    validate_priority(input.allocation_priority_score)?;
    validate_yyyymmdd(input.requested_ship_date_yyyymmdd)?;
    validate_evidence_ref(&input.promise_evidence_ref)?;
    let promised_quantity = input
        .requested_quantity
        .min(input.available_to_promise_quantity);
    let backorder_quantity = input.requested_quantity - promised_quantity;
    let idempotency_key = format!(
        "supply-chain-planning:atp:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.planning_area_id, input.atp_response_id
    );

    Ok(AtpResponsePreparation {
        atp_response_id: internal(AtpResponseId {
            value: input.atp_response_id,
        }),
        supply_plan_id: internal(SupplyPlanId {
            value: input.supply_plan_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        planning_area_id: internal(PlanningAreaId {
            value: input.planning_area_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        request_location_id: internal(LocationId {
            value: input.request_location_id,
        }),
        requested_quantity: financial(input.requested_quantity),
        available_to_promise_quantity: financial(input.available_to_promise_quantity),
        promised_quantity: financial(promised_quantity),
        backorder_quantity: financial(backorder_quantity),
        allocation_priority_score: financial(input.allocation_priority_score),
        requested_ship_date_yyyymmdd: internal(input.requested_ship_date_yyyymmdd),
        promise_evidence_ref: internal(EvidenceRef {
            value: input.promise_evidence_ref,
        }),
        state: internal(AtpResponseState::Prepared),
        idempotency_key: internal(idempotency_key),
        backorder_required: public(backorder_quantity > 0),
        order_management_reschedule_attached: public(false),
        warehouse_reservation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(SUPPLY_CHAIN_PLANNING_SCHEMA_VERSION),
    })
}

pub fn prepare_distribution_lane_plan(
    input: DistributionLanePlanInput,
) -> Result<DistributionLanePlanPreparation, SupplyChainPlanningError> {
    validate_distribution_plan_id(&input.distribution_plan_id)?;
    validate_supply_plan_id(&input.supply_plan_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_planning_area_id(&input.planning_area_id)?;
    validate_item_id(&input.item_id)?;
    validate_location_id(&input.source_location_id)?;
    validate_location_id(&input.destination_location_id)?;
    if !input.supply_plan_proposed {
        return Err(SupplyChainPlanningError::SupplyPlanRequired);
    }
    validate_positive_quantity(input.transfer_quantity)?;
    validate_positive_quantity(input.lane_capacity_quantity)?;
    if input.lane_capacity_quantity < input.transfer_quantity {
        return Err(SupplyChainPlanningError::InsufficientLaneCapacity);
    }
    validate_lead_time(input.lead_time_days)?;
    if input.estimated_freight_cost_cents == 0 {
        return Err(SupplyChainPlanningError::InvalidQuantity);
    }
    validate_source_ref(&input.lane_source_ref)?;
    validate_evidence_ref(&input.lane_evidence_ref)?;
    let capacity_remaining_quantity = input.lane_capacity_quantity - input.transfer_quantity;
    let idempotency_key = format!(
        "supply-chain-planning:distribution:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.planning_area_id, input.distribution_plan_id
    );

    Ok(DistributionLanePlanPreparation {
        distribution_plan_id: internal(DistributionPlanId {
            value: input.distribution_plan_id,
        }),
        supply_plan_id: internal(SupplyPlanId {
            value: input.supply_plan_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        planning_area_id: internal(PlanningAreaId {
            value: input.planning_area_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        source_location_id: internal(LocationId {
            value: input.source_location_id,
        }),
        destination_location_id: internal(LocationId {
            value: input.destination_location_id,
        }),
        transfer_quantity: financial(input.transfer_quantity),
        lane_capacity_quantity: financial(input.lane_capacity_quantity),
        capacity_remaining_quantity: financial(capacity_remaining_quantity),
        lead_time_days: internal(input.lead_time_days),
        estimated_freight_cost_cents: financial(input.estimated_freight_cost_cents),
        lane_source_ref: internal(SourceDocumentRef {
            value: input.lane_source_ref,
        }),
        lane_evidence_ref: internal(EvidenceRef {
            value: input.lane_evidence_ref,
        }),
        state: internal(DistributionPlanState::Prepared),
        idempotency_key: internal(idempotency_key),
        lane_capacity_sufficient: public(true),
        carrier_booking_attached: public(false),
        warehouse_transfer_order_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(SUPPLY_CHAIN_PLANNING_SCHEMA_VERSION),
    })
}

fn consensus_quantity(
    baseline_forecast_quantity: u32,
    firm_sales_order_quantity: u32,
    consensus_adjustment_quantity: i32,
) -> Result<u32, SupplyChainPlanningError> {
    let base = i64::from(baseline_forecast_quantity)
        + i64::from(firm_sales_order_quantity)
        + i64::from(consensus_adjustment_quantity);
    if base <= 0 || base > i64::from(u32::MAX) {
        return Err(SupplyChainPlanningError::InvalidQuantity);
    }
    Ok(base as u32)
}

fn validate_demand_plan_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        DEMAND_PLAN_ID_PREFIX,
        SupplyChainPlanningError::InvalidDemandPlanId,
    )
}

fn validate_supply_plan_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        SUPPLY_PLAN_ID_PREFIX,
        SupplyChainPlanningError::InvalidSupplyPlanId,
    )
}

fn validate_atp_response_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        ATP_RESPONSE_ID_PREFIX,
        SupplyChainPlanningError::InvalidAtpResponseId,
    )
}

fn validate_distribution_plan_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        DISTRIBUTION_PLAN_ID_PREFIX,
        SupplyChainPlanningError::InvalidDistributionPlanId,
    )
}

fn validate_planning_area_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        PLANNING_AREA_ID_PREFIX,
        SupplyChainPlanningError::InvalidPlanningAreaId,
    )
}

fn validate_tenant_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        TENANT_ID_PREFIX,
        SupplyChainPlanningError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        SupplyChainPlanningError::InvalidLegalEntityId,
    )
}

fn validate_item_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        ITEM_ID_PREFIX,
        SupplyChainPlanningError::InvalidItemId,
    )
}

fn validate_location_id(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_prefixed_identifier(
        value,
        LOCATION_ID_PREFIX,
        SupplyChainPlanningError::InvalidLocationId,
    )
}

fn validate_prefixed_identifier(
    value: &str,
    prefix: &str,
    error: SupplyChainPlanningError,
) -> Result<(), SupplyChainPlanningError> {
    if value == prefix
        || !value.starts_with(prefix)
        || has_unsafe_text(value)
        || value.contains('/')
        || value.contains("..")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_source_ref(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        SupplyChainPlanningError::InvalidSourceDocumentRef,
    )
}

fn validate_evidence_ref(value: &str) -> Result<(), SupplyChainPlanningError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        SupplyChainPlanningError::InvalidEvidenceRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: SupplyChainPlanningError,
) -> Result<(), SupplyChainPlanningError> {
    if value == prefix
        || !value.starts_with(prefix)
        || has_unsafe_text(value)
        || value.contains("..")
    {
        return Err(error);
    }
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("bearer")
        || lowered.contains("password")
        || lowered.contains("api-key")
        || lowered.contains("apikey")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_horizon(value: u16) -> Result<(), SupplyChainPlanningError> {
    if !(1..=1_095).contains(&value) {
        return Err(SupplyChainPlanningError::InvalidPlanningHorizon);
    }
    Ok(())
}

fn validate_lead_time(value: u16) -> Result<(), SupplyChainPlanningError> {
    if !(1..=366).contains(&value) {
        return Err(SupplyChainPlanningError::InvalidLeadTime);
    }
    Ok(())
}

fn validate_priority(value: u16) -> Result<(), SupplyChainPlanningError> {
    if !(1..=1_000).contains(&value) {
        return Err(SupplyChainPlanningError::InvalidPriority);
    }
    Ok(())
}

fn validate_positive_quantity(value: u32) -> Result<(), SupplyChainPlanningError> {
    if value == 0 {
        return Err(SupplyChainPlanningError::InvalidQuantity);
    }
    Ok(())
}

fn validate_yyyymmdd(value: u32) -> Result<(), SupplyChainPlanningError> {
    let year = value / 10_000;
    let month = (value / 100) % 100;
    let day = value % 100;
    if !(2020..=2100).contains(&year) || !(1..=12).contains(&month) {
        return Err(SupplyChainPlanningError::InvalidDate);
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return Err(SupplyChainPlanningError::InvalidDate),
    };
    if day == 0 || day > max_day {
        return Err(SupplyChainPlanningError::InvalidDate);
    }
    Ok(())
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn round_up_to_multiple(value: u32, multiple: u32) -> Result<u32, SupplyChainPlanningError> {
    if value == 0 {
        return Ok(0);
    }
    value
        .div_ceil(multiple)
        .checked_mul(multiple)
        .ok_or(SupplyChainPlanningError::InvalidQuantity)
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}
