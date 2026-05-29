//! Plant maintenance domain foundation.
//!
//! This crate owns pure plant-maintenance invariants for equipment asset
//! registration, preventive-maintenance plan approval, maintenance work-order
//! release metadata, and work-order completion metadata. It does not perform
//! durable persistence, IoT/SCADA ingestion, technician dispatch, spare-parts
//! inventory reservation, procurement requisition creation, accounting posting,
//! Workflow execution, runtime audit-chain emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const EQUIPMENT_ID_PREFIX: &str = "equip_";
const MAINTENANCE_PLAN_ID_PREFIX: &str = "mplan_";
const WORK_ORDER_ID_PREFIX: &str = "mwo_";
const FUNCTIONAL_LOCATION_ID_PREFIX: &str = "floc_";
const PLANT_ID_PREFIX: &str = "plant_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const PLANT_MAINTENANCE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EquipmentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaintenancePlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaintenanceWorkOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FunctionalLocationId {
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
pub struct SourceDocumentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaintenanceCriticality {
    Low,
    Medium,
    High,
    SafetyCritical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaintenancePriority {
    Low,
    Normal,
    Urgent,
    Emergency,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EquipmentAssetState {
    Registered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PreventiveMaintenancePlanState {
    Approved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaintenanceWorkOrderState {
    Released,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentAssetInput {
    pub equipment_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,              // data_class: INTERNAL_ONLY
    pub plant_id: String,                     // data_class: INTERNAL_ONLY
    pub functional_location_id: String,       // data_class: INTERNAL_ONLY
    pub criticality: MaintenanceCriticality,  // data_class: INTERNAL_ONLY
    pub installed_on_yyyymmdd: u32,           // data_class: INTERNAL_ONLY
    pub warranty_until_yyyymmdd: Option<u32>, // data_class: INTERNAL_ONLY
    pub asset_source_ref: String,             // data_class: INTERNAL_ONLY
    pub registration_evidence_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentAssetRegistration {
    pub equipment_id: Classified<EquipmentId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,       // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,         // data_class: INTERNAL_ONLY
    pub functional_location_id: Classified<FunctionalLocationId>, // data_class: INTERNAL_ONLY
    pub criticality: Classified<MaintenanceCriticality>, // data_class: INTERNAL_ONLY
    pub installed_on_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub warranty_until_yyyymmdd: Classified<Option<u32>>, // data_class: INTERNAL_ONLY
    pub asset_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub registration_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<EquipmentAssetState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,   // data_class: INTERNAL_ONLY
    pub durable_asset_registry_attached: Classified<bool>, // data_class: PUBLIC
    pub iot_or_scada_ingestion_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreventiveMaintenancePlanInput {
    pub maintenance_plan_id: String,    // data_class: INTERNAL_ONLY
    pub equipment_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,        // data_class: INTERNAL_ONLY
    pub plant_id: String,               // data_class: INTERNAL_ONLY
    pub functional_location_id: String, // data_class: INTERNAL_ONLY
    pub equipment_registered: bool,     // data_class: INTERNAL_ONLY
    pub interval_days: u16,             // data_class: FINANCIAL
    pub lead_time_days: u16,            // data_class: FINANCIAL
    pub estimated_labor_minutes: u32,   // data_class: FINANCIAL
    pub required_spare_part_count: u32, // data_class: FINANCIAL
    pub next_due_yyyymmdd: u32,         // data_class: INTERNAL_ONLY
    pub strategy_source_ref: String,    // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreventiveMaintenancePlanApproval {
    pub maintenance_plan_id: Classified<MaintenancePlanId>, // data_class: INTERNAL_ONLY
    pub equipment_id: Classified<EquipmentId>,              // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                    // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,         // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                      // data_class: INTERNAL_ONLY
    pub functional_location_id: Classified<FunctionalLocationId>, // data_class: INTERNAL_ONLY
    pub interval_days: Classified<u16>,                     // data_class: FINANCIAL
    pub lead_time_days: Classified<u16>,                    // data_class: FINANCIAL
    pub estimated_labor_minutes: Classified<u32>,           // data_class: FINANCIAL
    pub required_spare_part_count: Classified<u32>,         // data_class: FINANCIAL
    pub next_due_yyyymmdd: Classified<u32>,                 // data_class: INTERNAL_ONLY
    pub strategy_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>,     // data_class: INTERNAL_ONLY
    pub state: Classified<PreventiveMaintenancePlanState>,  // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                // data_class: INTERNAL_ONLY
    pub scheduler_runtime_attached: Classified<bool>,       // data_class: PUBLIC
    pub inventory_reservation_attached: Classified<bool>,   // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>,      // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,        // data_class: PUBLIC
    pub schema_version: Classified<u32>,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceWorkOrderInput {
    pub work_order_id: String,             // data_class: INTERNAL_ONLY
    pub maintenance_plan_id: String,       // data_class: INTERNAL_ONLY
    pub equipment_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub plant_id: String,                  // data_class: INTERNAL_ONLY
    pub functional_location_id: String,    // data_class: INTERNAL_ONLY
    pub maintenance_plan_approved: bool,   // data_class: INTERNAL_ONLY
    pub priority: MaintenancePriority,     // data_class: INTERNAL_ONLY
    pub planned_start_yyyymmdd: u32,       // data_class: INTERNAL_ONLY
    pub planned_labor_minutes: u32,        // data_class: FINANCIAL
    pub planned_spare_parts_quantity: u32, // data_class: FINANCIAL
    pub safety_permit_required: bool,      // data_class: INTERNAL_ONLY
    pub job_instruction_ref: String,       // data_class: INTERNAL_ONLY
    pub release_evidence_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceWorkOrderRelease {
    pub work_order_id: Classified<MaintenanceWorkOrderId>, // data_class: INTERNAL_ONLY
    pub maintenance_plan_id: Classified<MaintenancePlanId>, // data_class: INTERNAL_ONLY
    pub equipment_id: Classified<EquipmentId>,             // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,        // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                     // data_class: INTERNAL_ONLY
    pub functional_location_id: Classified<FunctionalLocationId>, // data_class: INTERNAL_ONLY
    pub priority: Classified<MaintenancePriority>,         // data_class: INTERNAL_ONLY
    pub planned_start_yyyymmdd: Classified<u32>,           // data_class: INTERNAL_ONLY
    pub planned_labor_minutes: Classified<u32>,            // data_class: FINANCIAL
    pub planned_spare_parts_quantity: Classified<u32>,     // data_class: FINANCIAL
    pub safety_permit_required: Classified<bool>,          // data_class: INTERNAL_ONLY
    pub job_instruction_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub release_evidence_ref: Classified<EvidenceRef>,     // data_class: INTERNAL_ONLY
    pub state: Classified<MaintenanceWorkOrderState>,      // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,               // data_class: INTERNAL_ONLY
    pub inventory_reservation_attached: Classified<bool>,  // data_class: PUBLIC
    pub procurement_requisition_attached: Classified<bool>, // data_class: PUBLIC
    pub technician_dispatch_attached: Classified<bool>,    // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>,     // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,       // data_class: PUBLIC
    pub schema_version: Classified<u32>,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceWorkOrderCompletionInput {
    pub work_order_id: String,             // data_class: INTERNAL_ONLY
    pub equipment_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub plant_id: String,                  // data_class: INTERNAL_ONLY
    pub work_order_released: bool,         // data_class: INTERNAL_ONLY
    pub completion_yyyymmdd: u32,          // data_class: INTERNAL_ONLY
    pub planned_labor_minutes: u32,        // data_class: FINANCIAL
    pub actual_labor_minutes: u32,         // data_class: FINANCIAL
    pub planned_spare_parts_quantity: u32, // data_class: FINANCIAL
    pub actual_spare_parts_quantity: u32,  // data_class: FINANCIAL
    pub downtime_minutes: u32,             // data_class: FINANCIAL
    pub measurement_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub completion_evidence_ref: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceWorkOrderCompletion {
    pub work_order_id: Classified<MaintenanceWorkOrderId>, // data_class: INTERNAL_ONLY
    pub equipment_id: Classified<EquipmentId>,             // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,        // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                     // data_class: INTERNAL_ONLY
    pub completion_yyyymmdd: Classified<u32>,              // data_class: INTERNAL_ONLY
    pub planned_labor_minutes: Classified<u32>,            // data_class: FINANCIAL
    pub actual_labor_minutes: Classified<u32>,             // data_class: FINANCIAL
    pub labor_variance_minutes: Classified<i64>,           // data_class: FINANCIAL
    pub planned_spare_parts_quantity: Classified<u32>,     // data_class: FINANCIAL
    pub actual_spare_parts_quantity: Classified<u32>,      // data_class: FINANCIAL
    pub spare_parts_remaining_quantity: Classified<u32>,   // data_class: FINANCIAL
    pub downtime_minutes: Classified<u32>,                 // data_class: FINANCIAL
    pub measurement_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub completion_evidence_ref: Classified<EvidenceRef>,  // data_class: INTERNAL_ONLY
    pub state: Classified<MaintenanceWorkOrderState>,      // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,               // data_class: INTERNAL_ONLY
    pub next_plan_recalculation_required: Classified<bool>, // data_class: PUBLIC
    pub accounting_posting_attached: Classified<bool>,     // data_class: PUBLIC
    pub equipment_meter_write_attached: Classified<bool>,  // data_class: PUBLIC
    pub runtime_audit_chain_emission_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,       // data_class: PUBLIC
    pub schema_version: Classified<u32>,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlantMaintenanceError {
    InvalidEquipmentId,
    InvalidMaintenancePlanId,
    InvalidWorkOrderId,
    InvalidFunctionalLocationId,
    InvalidPlantId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidDate,
    InvalidInterval,
    InvalidQuantity,
    EquipmentRegistrationRequired,
    MaintenancePlanApprovalRequired,
    WorkOrderReleaseRequired,
    SparePartOverConsumption,
}

pub fn register_equipment_asset(
    input: EquipmentAssetInput,
) -> Result<EquipmentAssetRegistration, PlantMaintenanceError> {
    validate_equipment_id(&input.equipment_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_functional_location_id(&input.functional_location_id)?;
    validate_yyyymmdd(input.installed_on_yyyymmdd)?;
    if let Some(warranty_until) = input.warranty_until_yyyymmdd {
        validate_yyyymmdd(warranty_until)?;
        if warranty_until < input.installed_on_yyyymmdd {
            return Err(PlantMaintenanceError::InvalidDate);
        }
    }
    validate_source_ref(&input.asset_source_ref)?;
    validate_evidence_ref(&input.registration_evidence_ref)?;
    let idempotency_key = format!(
        "plant-maintenance:equipment:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.equipment_id
    );

    Ok(EquipmentAssetRegistration {
        equipment_id: internal(EquipmentId {
            value: input.equipment_id,
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
        functional_location_id: internal(FunctionalLocationId {
            value: input.functional_location_id,
        }),
        criticality: internal(input.criticality),
        installed_on_yyyymmdd: internal(input.installed_on_yyyymmdd),
        warranty_until_yyyymmdd: internal(input.warranty_until_yyyymmdd),
        asset_source_ref: internal(SourceDocumentRef {
            value: input.asset_source_ref,
        }),
        registration_evidence_ref: internal(EvidenceRef {
            value: input.registration_evidence_ref,
        }),
        state: internal(EquipmentAssetState::Registered),
        idempotency_key: internal(idempotency_key),
        durable_asset_registry_attached: public(false),
        iot_or_scada_ingestion_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PLANT_MAINTENANCE_SCHEMA_VERSION),
    })
}

pub fn approve_preventive_maintenance_plan(
    input: PreventiveMaintenancePlanInput,
) -> Result<PreventiveMaintenancePlanApproval, PlantMaintenanceError> {
    validate_maintenance_plan_id(&input.maintenance_plan_id)?;
    validate_equipment_id(&input.equipment_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_functional_location_id(&input.functional_location_id)?;
    if !input.equipment_registered {
        return Err(PlantMaintenanceError::EquipmentRegistrationRequired);
    }
    validate_interval(input.interval_days, input.lead_time_days)?;
    validate_positive_quantity(input.estimated_labor_minutes)?;
    validate_positive_quantity(input.required_spare_part_count)?;
    validate_yyyymmdd(input.next_due_yyyymmdd)?;
    validate_source_ref(&input.strategy_source_ref)?;
    validate_evidence_ref(&input.approval_evidence_ref)?;
    let idempotency_key = format!(
        "plant-maintenance:plan:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.maintenance_plan_id
    );

    Ok(PreventiveMaintenancePlanApproval {
        maintenance_plan_id: internal(MaintenancePlanId {
            value: input.maintenance_plan_id,
        }),
        equipment_id: internal(EquipmentId {
            value: input.equipment_id,
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
        functional_location_id: internal(FunctionalLocationId {
            value: input.functional_location_id,
        }),
        interval_days: financial(input.interval_days),
        lead_time_days: financial(input.lead_time_days),
        estimated_labor_minutes: financial(input.estimated_labor_minutes),
        required_spare_part_count: financial(input.required_spare_part_count),
        next_due_yyyymmdd: internal(input.next_due_yyyymmdd),
        strategy_source_ref: internal(SourceDocumentRef {
            value: input.strategy_source_ref,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        state: internal(PreventiveMaintenancePlanState::Approved),
        idempotency_key: internal(idempotency_key),
        scheduler_runtime_attached: public(false),
        inventory_reservation_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PLANT_MAINTENANCE_SCHEMA_VERSION),
    })
}

pub fn release_maintenance_work_order(
    input: MaintenanceWorkOrderInput,
) -> Result<MaintenanceWorkOrderRelease, PlantMaintenanceError> {
    validate_work_order_id(&input.work_order_id)?;
    validate_maintenance_plan_id(&input.maintenance_plan_id)?;
    validate_equipment_id(&input.equipment_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_functional_location_id(&input.functional_location_id)?;
    if !input.maintenance_plan_approved {
        return Err(PlantMaintenanceError::MaintenancePlanApprovalRequired);
    }
    validate_yyyymmdd(input.planned_start_yyyymmdd)?;
    validate_positive_quantity(input.planned_labor_minutes)?;
    validate_positive_quantity(input.planned_spare_parts_quantity)?;
    validate_source_ref(&input.job_instruction_ref)?;
    validate_evidence_ref(&input.release_evidence_ref)?;
    let idempotency_key = format!(
        "plant-maintenance:work-order:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.work_order_id
    );

    Ok(MaintenanceWorkOrderRelease {
        work_order_id: internal(MaintenanceWorkOrderId {
            value: input.work_order_id,
        }),
        maintenance_plan_id: internal(MaintenancePlanId {
            value: input.maintenance_plan_id,
        }),
        equipment_id: internal(EquipmentId {
            value: input.equipment_id,
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
        functional_location_id: internal(FunctionalLocationId {
            value: input.functional_location_id,
        }),
        priority: internal(input.priority),
        planned_start_yyyymmdd: internal(input.planned_start_yyyymmdd),
        planned_labor_minutes: financial(input.planned_labor_minutes),
        planned_spare_parts_quantity: financial(input.planned_spare_parts_quantity),
        safety_permit_required: internal(input.safety_permit_required),
        job_instruction_ref: internal(SourceDocumentRef {
            value: input.job_instruction_ref,
        }),
        release_evidence_ref: internal(EvidenceRef {
            value: input.release_evidence_ref,
        }),
        state: internal(MaintenanceWorkOrderState::Released),
        idempotency_key: internal(idempotency_key),
        inventory_reservation_attached: public(false),
        procurement_requisition_attached: public(false),
        technician_dispatch_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PLANT_MAINTENANCE_SCHEMA_VERSION),
    })
}

pub fn complete_maintenance_work_order(
    input: MaintenanceWorkOrderCompletionInput,
) -> Result<MaintenanceWorkOrderCompletion, PlantMaintenanceError> {
    validate_work_order_id(&input.work_order_id)?;
    validate_equipment_id(&input.equipment_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    if !input.work_order_released {
        return Err(PlantMaintenanceError::WorkOrderReleaseRequired);
    }
    validate_yyyymmdd(input.completion_yyyymmdd)?;
    validate_positive_quantity(input.planned_labor_minutes)?;
    validate_positive_quantity(input.actual_labor_minutes)?;
    validate_positive_quantity(input.planned_spare_parts_quantity)?;
    if input.actual_spare_parts_quantity > input.planned_spare_parts_quantity {
        return Err(PlantMaintenanceError::SparePartOverConsumption);
    }
    validate_evidence_ref(&input.measurement_evidence_ref)?;
    validate_evidence_ref(&input.completion_evidence_ref)?;
    let spare_parts_remaining_quantity =
        input.planned_spare_parts_quantity - input.actual_spare_parts_quantity;
    let labor_variance_minutes =
        i64::from(input.actual_labor_minutes) - i64::from(input.planned_labor_minutes);
    let next_plan_recalculation_required = labor_variance_minutes > 0 || input.downtime_minutes > 0;
    let idempotency_key = format!(
        "plant-maintenance:completion:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.work_order_id
    );

    Ok(MaintenanceWorkOrderCompletion {
        work_order_id: internal(MaintenanceWorkOrderId {
            value: input.work_order_id,
        }),
        equipment_id: internal(EquipmentId {
            value: input.equipment_id,
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
        completion_yyyymmdd: internal(input.completion_yyyymmdd),
        planned_labor_minutes: financial(input.planned_labor_minutes),
        actual_labor_minutes: financial(input.actual_labor_minutes),
        labor_variance_minutes: financial(labor_variance_minutes),
        planned_spare_parts_quantity: financial(input.planned_spare_parts_quantity),
        actual_spare_parts_quantity: financial(input.actual_spare_parts_quantity),
        spare_parts_remaining_quantity: financial(spare_parts_remaining_quantity),
        downtime_minutes: financial(input.downtime_minutes),
        measurement_evidence_ref: internal(EvidenceRef {
            value: input.measurement_evidence_ref,
        }),
        completion_evidence_ref: internal(EvidenceRef {
            value: input.completion_evidence_ref,
        }),
        state: internal(MaintenanceWorkOrderState::Completed),
        idempotency_key: internal(idempotency_key),
        next_plan_recalculation_required: public(next_plan_recalculation_required),
        accounting_posting_attached: public(false),
        equipment_meter_write_attached: public(false),
        runtime_audit_chain_emission_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PLANT_MAINTENANCE_SCHEMA_VERSION),
    })
}

fn validate_equipment_id(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_prefixed_identifier(
        value,
        EQUIPMENT_ID_PREFIX,
        PlantMaintenanceError::InvalidEquipmentId,
    )
}

fn validate_maintenance_plan_id(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_prefixed_identifier(
        value,
        MAINTENANCE_PLAN_ID_PREFIX,
        PlantMaintenanceError::InvalidMaintenancePlanId,
    )
}

fn validate_work_order_id(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_prefixed_identifier(
        value,
        WORK_ORDER_ID_PREFIX,
        PlantMaintenanceError::InvalidWorkOrderId,
    )
}

fn validate_functional_location_id(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_prefixed_identifier(
        value,
        FUNCTIONAL_LOCATION_ID_PREFIX,
        PlantMaintenanceError::InvalidFunctionalLocationId,
    )
}

fn validate_plant_id(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_prefixed_identifier(
        value,
        PLANT_ID_PREFIX,
        PlantMaintenanceError::InvalidPlantId,
    )
}

fn validate_tenant_id(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_prefixed_identifier(
        value,
        TENANT_ID_PREFIX,
        PlantMaintenanceError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_prefixed_identifier(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        PlantMaintenanceError::InvalidLegalEntityId,
    )
}

fn validate_prefixed_identifier(
    value: &str,
    prefix: &str,
    error: PlantMaintenanceError,
) -> Result<(), PlantMaintenanceError> {
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

fn validate_source_ref(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        PlantMaintenanceError::InvalidSourceDocumentRef,
    )
}

fn validate_evidence_ref(value: &str) -> Result<(), PlantMaintenanceError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        PlantMaintenanceError::InvalidEvidenceRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: PlantMaintenanceError,
) -> Result<(), PlantMaintenanceError> {
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

fn validate_interval(interval_days: u16, lead_time_days: u16) -> Result<(), PlantMaintenanceError> {
    if interval_days == 0
        || interval_days > 3_650
        || lead_time_days == 0
        || lead_time_days >= interval_days
    {
        return Err(PlantMaintenanceError::InvalidInterval);
    }
    Ok(())
}

fn validate_positive_quantity(value: u32) -> Result<(), PlantMaintenanceError> {
    if value == 0 {
        return Err(PlantMaintenanceError::InvalidQuantity);
    }
    Ok(())
}

fn validate_yyyymmdd(value: u32) -> Result<(), PlantMaintenanceError> {
    let year = value / 10_000;
    let month = (value / 100) % 100;
    let day = value % 100;
    if !(2020..=2100).contains(&year) || !(1..=12).contains(&month) {
        return Err(PlantMaintenanceError::InvalidDate);
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
        _ => return Err(PlantMaintenanceError::InvalidDate),
    };
    if day == 0 || day > max_day {
        return Err(PlantMaintenanceError::InvalidDate);
    }
    Ok(())
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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
