//! Production planning domain foundation.
//!
//! This crate owns pure production-planning invariants for approved work
//! definitions, MRP planned-order proposals, and production-release preparation
//! metadata. It does not perform durable persistence, procurement purchase-order
//! creation, inventory mutation, shop-floor execution, accounting posting,
//! Workflow dispatch, runtime audit-chain emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const WORK_DEFINITION_ID_PREFIX: &str = "wdef_";
const PLANNED_ORDER_ID_PREFIX: &str = "plord_";
const PRODUCTION_ORDER_ID_PREFIX: &str = "prod_";
const PLANT_ID_PREFIX: &str = "plant_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const ITEM_ID_PREFIX: &str = "item_";
const BOM_ID_PREFIX: &str = "bom_";
const ROUTE_ID_PREFIX: &str = "route_";
const WORK_CENTER_ID_PREFIX: &str = "wc_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const PRODUCTION_PLANNING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkDefinitionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlannedOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProductionOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlantId {
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
pub struct BillOfMaterialsId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RouteId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkCenterId {
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
pub enum WorkDefinitionState {
    Approved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MrpPlanState {
    PlannedOrderProposed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ProductionReleaseState {
    ReleasePrepared,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkDefinitionInput {
    pub work_definition_id: String,    // data_class: INTERNAL_ONLY
    pub plant_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub product_item_id: String,       // data_class: INTERNAL_ONLY
    pub bom_id: String,                // data_class: INTERNAL_ONLY
    pub route_id: String,              // data_class: INTERNAL_ONLY
    pub work_center_id: String,        // data_class: INTERNAL_ONLY
    pub component_count: u32,          // data_class: FINANCIAL
    pub total_component_quantity: u32, // data_class: FINANCIAL
    pub standard_run_minutes: u32,     // data_class: FINANCIAL
    pub effective_from_yyyymmdd: u32,  // data_class: INTERNAL_ONLY
    pub bom_source_ref: String,        // data_class: INTERNAL_ONLY
    pub routing_source_ref: String,    // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkDefinitionApproval {
    pub work_definition_id: Classified<WorkDefinitionId>, // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,       // data_class: INTERNAL_ONLY
    pub product_item_id: Classified<ItemId>,              // data_class: INTERNAL_ONLY
    pub bom_id: Classified<BillOfMaterialsId>,            // data_class: INTERNAL_ONLY
    pub route_id: Classified<RouteId>,                    // data_class: INTERNAL_ONLY
    pub work_center_id: Classified<WorkCenterId>,         // data_class: INTERNAL_ONLY
    pub component_count: Classified<u32>,                 // data_class: FINANCIAL
    pub total_component_quantity: Classified<u32>,        // data_class: FINANCIAL
    pub standard_run_minutes: Classified<u32>,            // data_class: FINANCIAL
    pub effective_from_yyyymmdd: Classified<u32>,         // data_class: INTERNAL_ONLY
    pub bom_source_ref: Classified<SourceDocumentRef>,    // data_class: INTERNAL_ONLY
    pub routing_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>,   // data_class: INTERNAL_ONLY
    pub state: Classified<WorkDefinitionState>,           // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,              // data_class: INTERNAL_ONLY
    pub shop_floor_execution_attached: Classified<bool>,  // data_class: PUBLIC
    pub inventory_mutation_attached: Classified<bool>,    // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,      // data_class: PUBLIC
    pub schema_version: Classified<u32>,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MrpPlanInput {
    pub planned_order_id: String,          // data_class: INTERNAL_ONLY
    pub work_definition_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub plant_id: String,                  // data_class: INTERNAL_ONLY
    pub product_item_id: String,           // data_class: INTERNAL_ONLY
    pub work_definition_approved: bool,    // data_class: INTERNAL_ONLY
    pub demand_quantity: u32,              // data_class: FINANCIAL
    pub on_hand_quantity: u32,             // data_class: FINANCIAL
    pub scheduled_receipt_quantity: u32,   // data_class: FINANCIAL
    pub safety_stock_quantity: u32,        // data_class: FINANCIAL
    pub lot_size_multiple: u32,            // data_class: FINANCIAL
    pub planning_horizon_days: u16,        // data_class: INTERNAL_ONLY
    pub demand_signal_ref: String,         // data_class: INTERNAL_ONLY
    pub planning_run_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MrpPlanProposal {
    pub planned_order_id: Classified<PlannedOrderId>, // data_class: INTERNAL_ONLY
    pub work_definition_id: Classified<WorkDefinitionId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,   // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                // data_class: INTERNAL_ONLY
    pub product_item_id: Classified<ItemId>,          // data_class: INTERNAL_ONLY
    pub demand_quantity: Classified<u32>,             // data_class: FINANCIAL
    pub on_hand_quantity: Classified<u32>,            // data_class: FINANCIAL
    pub scheduled_receipt_quantity: Classified<u32>,  // data_class: FINANCIAL
    pub safety_stock_quantity: Classified<u32>,       // data_class: FINANCIAL
    pub net_requirement_quantity: Classified<u32>,    // data_class: FINANCIAL
    pub planned_order_quantity: Classified<u32>,      // data_class: FINANCIAL
    pub lot_size_multiple: Classified<u32>,           // data_class: FINANCIAL
    pub planning_horizon_days: Classified<u16>,       // data_class: INTERNAL_ONLY
    pub demand_signal_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub planning_run_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<MrpPlanState>,              // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,          // data_class: INTERNAL_ONLY
    pub production_order_release_allowed: Classified<bool>, // data_class: PUBLIC
    pub procurement_purchase_order_attached: Classified<bool>, // data_class: PUBLIC
    pub inventory_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,  // data_class: PUBLIC
    pub schema_version: Classified<u32>,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionReleaseInput {
    pub production_order_id: String,       // data_class: INTERNAL_ONLY
    pub planned_order_id: String,          // data_class: INTERNAL_ONLY
    pub work_definition_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub plant_id: String,                  // data_class: INTERNAL_ONLY
    pub product_item_id: String,           // data_class: INTERNAL_ONLY
    pub planned_order_proposed: bool,      // data_class: INTERNAL_ONLY
    pub required_quantity: u32,            // data_class: FINANCIAL
    pub material_available_quantity: u32,  // data_class: FINANCIAL
    pub required_capacity_minutes: u32,    // data_class: FINANCIAL
    pub work_center_capacity_minutes: u32, // data_class: FINANCIAL
    pub schedule_evidence_ref: String,     // data_class: INTERNAL_ONLY
    pub material_availability_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub capacity_evidence_ref: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionReleasePlan {
    pub production_order_id: Classified<ProductionOrderId>, // data_class: INTERNAL_ONLY
    pub planned_order_id: Classified<PlannedOrderId>,       // data_class: INTERNAL_ONLY
    pub work_definition_id: Classified<WorkDefinitionId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                    // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,         // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                      // data_class: INTERNAL_ONLY
    pub product_item_id: Classified<ItemId>,                // data_class: INTERNAL_ONLY
    pub required_quantity: Classified<u32>,                 // data_class: FINANCIAL
    pub material_available_quantity: Classified<u32>,       // data_class: FINANCIAL
    pub material_remaining_quantity: Classified<u32>,       // data_class: FINANCIAL
    pub required_capacity_minutes: Classified<u32>,         // data_class: FINANCIAL
    pub work_center_capacity_minutes: Classified<u32>,      // data_class: FINANCIAL
    pub capacity_remaining_minutes: Classified<u32>,        // data_class: FINANCIAL
    pub schedule_evidence_ref: Classified<EvidenceRef>,     // data_class: INTERNAL_ONLY
    pub material_availability_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub capacity_evidence_ref: Classified<EvidenceRef>,     // data_class: INTERNAL_ONLY
    pub state: Classified<ProductionReleaseState>,          // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                // data_class: INTERNAL_ONLY
    pub release_allowed: Classified<bool>,                  // data_class: PUBLIC
    pub shop_floor_execution_attached: Classified<bool>,    // data_class: PUBLIC
    pub inventory_mutation_attached: Classified<bool>,      // data_class: PUBLIC
    pub accounting_posting_attached: Classified<bool>,      // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>,      // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,        // data_class: PUBLIC
    pub schema_version: Classified<u32>,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductionPlanningError {
    InvalidWorkDefinitionId,
    InvalidPlannedOrderId,
    InvalidProductionOrderId,
    InvalidPlantId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidItemId,
    InvalidBomId,
    InvalidRouteId,
    InvalidWorkCenterId,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidQuantity,
    InvalidEffectiveDate,
    InvalidPlanningHorizon,
    WorkDefinitionApprovalRequired,
    PlannedOrderRequired,
    InsufficientMaterialAvailability,
    InsufficientCapacity,
}

pub fn approve_work_definition(
    input: WorkDefinitionInput,
) -> Result<WorkDefinitionApproval, ProductionPlanningError> {
    validate_work_definition_id(&input.work_definition_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_item_id(&input.product_item_id)?;
    validate_bom_id(&input.bom_id)?;
    validate_route_id(&input.route_id)?;
    validate_work_center_id(&input.work_center_id)?;
    validate_positive_quantity(input.component_count)?;
    validate_positive_quantity(input.total_component_quantity)?;
    validate_positive_quantity(input.standard_run_minutes)?;
    validate_yyyymmdd(input.effective_from_yyyymmdd)?;
    validate_source_ref(&input.bom_source_ref)?;
    validate_source_ref(&input.routing_source_ref)?;
    validate_evidence_ref(&input.approval_evidence_ref)?;
    let idempotency_key = format!(
        "production-planning:work-definition:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.work_definition_id
    );

    Ok(WorkDefinitionApproval {
        work_definition_id: internal(WorkDefinitionId {
            value: input.work_definition_id,
        }),
        plant_id: internal(PlantId {
            value: input.plant_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        product_item_id: internal(ItemId {
            value: input.product_item_id,
        }),
        bom_id: internal(BillOfMaterialsId {
            value: input.bom_id,
        }),
        route_id: internal(RouteId {
            value: input.route_id,
        }),
        work_center_id: internal(WorkCenterId {
            value: input.work_center_id,
        }),
        component_count: financial(input.component_count),
        total_component_quantity: financial(input.total_component_quantity),
        standard_run_minutes: financial(input.standard_run_minutes),
        effective_from_yyyymmdd: internal(input.effective_from_yyyymmdd),
        bom_source_ref: internal(SourceDocumentRef {
            value: input.bom_source_ref,
        }),
        routing_source_ref: internal(SourceDocumentRef {
            value: input.routing_source_ref,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        state: internal(WorkDefinitionState::Approved),
        idempotency_key: internal(idempotency_key),
        shop_floor_execution_attached: public(false),
        inventory_mutation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PRODUCTION_PLANNING_SCHEMA_VERSION),
    })
}

pub fn plan_material_requirements(
    input: MrpPlanInput,
) -> Result<MrpPlanProposal, ProductionPlanningError> {
    validate_planned_order_id(&input.planned_order_id)?;
    validate_work_definition_id(&input.work_definition_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_item_id(&input.product_item_id)?;
    if !input.work_definition_approved {
        return Err(ProductionPlanningError::WorkDefinitionApprovalRequired);
    }
    validate_positive_quantity(input.demand_quantity)?;
    validate_positive_quantity(input.lot_size_multiple)?;
    if !(1..=366).contains(&input.planning_horizon_days) {
        return Err(ProductionPlanningError::InvalidPlanningHorizon);
    }
    validate_source_ref(&input.demand_signal_ref)?;
    validate_evidence_ref(&input.planning_run_evidence_ref)?;
    let gross_requirement = input.demand_quantity + input.safety_stock_quantity;
    let available_supply = input.on_hand_quantity + input.scheduled_receipt_quantity;
    let net_requirement_quantity = gross_requirement.saturating_sub(available_supply);
    let planned_order_quantity =
        round_up_to_multiple(net_requirement_quantity, input.lot_size_multiple);
    let idempotency_key = format!(
        "production-planning:mrp:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.planned_order_id
    );

    Ok(MrpPlanProposal {
        planned_order_id: internal(PlannedOrderId {
            value: input.planned_order_id,
        }),
        work_definition_id: internal(WorkDefinitionId {
            value: input.work_definition_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        plant_id: internal(PlantId {
            value: input.plant_id,
        }),
        product_item_id: internal(ItemId {
            value: input.product_item_id,
        }),
        demand_quantity: financial(input.demand_quantity),
        on_hand_quantity: financial(input.on_hand_quantity),
        scheduled_receipt_quantity: financial(input.scheduled_receipt_quantity),
        safety_stock_quantity: financial(input.safety_stock_quantity),
        net_requirement_quantity: financial(net_requirement_quantity),
        planned_order_quantity: financial(planned_order_quantity),
        lot_size_multiple: financial(input.lot_size_multiple),
        planning_horizon_days: internal(input.planning_horizon_days),
        demand_signal_ref: internal(SourceDocumentRef {
            value: input.demand_signal_ref,
        }),
        planning_run_evidence_ref: internal(EvidenceRef {
            value: input.planning_run_evidence_ref,
        }),
        state: internal(MrpPlanState::PlannedOrderProposed),
        idempotency_key: internal(idempotency_key),
        production_order_release_allowed: public(planned_order_quantity > 0),
        procurement_purchase_order_attached: public(false),
        inventory_mutation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PRODUCTION_PLANNING_SCHEMA_VERSION),
    })
}

pub fn prepare_production_release(
    input: ProductionReleaseInput,
) -> Result<ProductionReleasePlan, ProductionPlanningError> {
    validate_production_order_id(&input.production_order_id)?;
    validate_planned_order_id(&input.planned_order_id)?;
    validate_work_definition_id(&input.work_definition_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_item_id(&input.product_item_id)?;
    if !input.planned_order_proposed {
        return Err(ProductionPlanningError::PlannedOrderRequired);
    }
    validate_positive_quantity(input.required_quantity)?;
    validate_positive_quantity(input.material_available_quantity)?;
    validate_positive_quantity(input.required_capacity_minutes)?;
    validate_positive_quantity(input.work_center_capacity_minutes)?;
    if input.material_available_quantity < input.required_quantity {
        return Err(ProductionPlanningError::InsufficientMaterialAvailability);
    }
    if input.work_center_capacity_minutes < input.required_capacity_minutes {
        return Err(ProductionPlanningError::InsufficientCapacity);
    }
    validate_evidence_ref(&input.schedule_evidence_ref)?;
    validate_evidence_ref(&input.material_availability_evidence_ref)?;
    validate_evidence_ref(&input.capacity_evidence_ref)?;
    let material_remaining_quantity = input.material_available_quantity - input.required_quantity;
    let capacity_remaining_minutes =
        input.work_center_capacity_minutes - input.required_capacity_minutes;
    let idempotency_key = format!(
        "production-planning:release:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.production_order_id
    );

    Ok(ProductionReleasePlan {
        production_order_id: internal(ProductionOrderId {
            value: input.production_order_id,
        }),
        planned_order_id: internal(PlannedOrderId {
            value: input.planned_order_id,
        }),
        work_definition_id: internal(WorkDefinitionId {
            value: input.work_definition_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        plant_id: internal(PlantId {
            value: input.plant_id,
        }),
        product_item_id: internal(ItemId {
            value: input.product_item_id,
        }),
        required_quantity: financial(input.required_quantity),
        material_available_quantity: financial(input.material_available_quantity),
        material_remaining_quantity: financial(material_remaining_quantity),
        required_capacity_minutes: financial(input.required_capacity_minutes),
        work_center_capacity_minutes: financial(input.work_center_capacity_minutes),
        capacity_remaining_minutes: financial(capacity_remaining_minutes),
        schedule_evidence_ref: internal(EvidenceRef {
            value: input.schedule_evidence_ref,
        }),
        material_availability_evidence_ref: internal(EvidenceRef {
            value: input.material_availability_evidence_ref,
        }),
        capacity_evidence_ref: internal(EvidenceRef {
            value: input.capacity_evidence_ref,
        }),
        state: internal(ProductionReleaseState::ReleasePrepared),
        idempotency_key: internal(idempotency_key),
        release_allowed: public(true),
        shop_floor_execution_attached: public(false),
        inventory_mutation_attached: public(false),
        accounting_posting_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PRODUCTION_PLANNING_SCHEMA_VERSION),
    })
}

fn validate_work_definition_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        WORK_DEFINITION_ID_PREFIX,
        ProductionPlanningError::InvalidWorkDefinitionId,
    )
}

fn validate_planned_order_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        PLANNED_ORDER_ID_PREFIX,
        ProductionPlanningError::InvalidPlannedOrderId,
    )
}

fn validate_production_order_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        PRODUCTION_ORDER_ID_PREFIX,
        ProductionPlanningError::InvalidProductionOrderId,
    )
}

fn validate_plant_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        PLANT_ID_PREFIX,
        ProductionPlanningError::InvalidPlantId,
    )
}

fn validate_tenant_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        TENANT_ID_PREFIX,
        ProductionPlanningError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        ProductionPlanningError::InvalidLegalEntityId,
    )
}

fn validate_item_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        ITEM_ID_PREFIX,
        ProductionPlanningError::InvalidItemId,
    )
}

fn validate_bom_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(value, BOM_ID_PREFIX, ProductionPlanningError::InvalidBomId)
}

fn validate_route_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        ROUTE_ID_PREFIX,
        ProductionPlanningError::InvalidRouteId,
    )
}

fn validate_work_center_id(value: &str) -> Result<(), ProductionPlanningError> {
    validate_prefixed_identifier(
        value,
        WORK_CENTER_ID_PREFIX,
        ProductionPlanningError::InvalidWorkCenterId,
    )
}

fn validate_prefixed_identifier(
    value: &str,
    prefix: &str,
    error: ProductionPlanningError,
) -> Result<(), ProductionPlanningError> {
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

fn validate_source_ref(value: &str) -> Result<(), ProductionPlanningError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        ProductionPlanningError::InvalidSourceDocumentRef,
    )
}

fn validate_evidence_ref(value: &str) -> Result<(), ProductionPlanningError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        ProductionPlanningError::InvalidEvidenceRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: ProductionPlanningError,
) -> Result<(), ProductionPlanningError> {
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

fn validate_positive_quantity(value: u32) -> Result<(), ProductionPlanningError> {
    if value == 0 {
        return Err(ProductionPlanningError::InvalidQuantity);
    }
    Ok(())
}

fn validate_yyyymmdd(value: u32) -> Result<(), ProductionPlanningError> {
    let year = value / 10_000;
    let month = (value / 100) % 100;
    let day = value % 100;
    if !(2020..=2100).contains(&year) || !(1..=12).contains(&month) {
        return Err(ProductionPlanningError::InvalidEffectiveDate);
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
        _ => return Err(ProductionPlanningError::InvalidEffectiveDate),
    };
    if day == 0 || day > max_day {
        return Err(ProductionPlanningError::InvalidEffectiveDate);
    }
    Ok(())
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn round_up_to_multiple(value: u32, multiple: u32) -> u32 {
    if value == 0 {
        return 0;
    }
    value.div_ceil(multiple) * multiple
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
