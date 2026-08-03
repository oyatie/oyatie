//! Quality management domain foundation.
//!
//! This crate owns pure quality-management invariants for inspection-plan
//! approval, inspection lot usage-decision metadata, quality certificate
//! preparation, and quality-notification opening. It does not perform durable
//! persistence, inventory blocking or release mutations, certificate PDF
//! rendering, email delivery, CAPA workflow execution, supplier-collaboration
//! network calls, runtime audit-chain emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const INSPECTION_PLAN_ID_PREFIX: &str = "qplan_";
const INSPECTION_LOT_ID_PREFIX: &str = "qlot_";
const QUALITY_CERTIFICATE_ID_PREFIX: &str = "qcert_";
const QUALITY_NOTIFICATION_ID_PREFIX: &str = "qnot_";
const PLANT_ID_PREFIX: &str = "plant_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const ITEM_ID_PREFIX: &str = "item_";
const BATCH_ID_PREFIX: &str = "batch_";
const CUSTOMER_ID_PREFIX: &str = "cust_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const BASIS_POINTS_DENOMINATOR: u32 = 10_000;
const QUALITY_MANAGEMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InspectionPlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InspectionLotId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct QualityCertificateId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct QualityNotificationId {
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
pub struct BatchId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CustomerId {
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
pub enum InspectionPlanType {
    Receiving,
    WorkInProcess,
    Inventory,
    Resource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InspectionPlanState {
    Approved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum UsageDecisionState {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CertificateState {
    Prepared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QualityNotificationState {
    Opened,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QualitySeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionPlanInput {
    pub inspection_plan_id: String,    // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub plant_id: String,              // data_class: INTERNAL_ONLY
    pub item_id: String,               // data_class: INTERNAL_ONLY
    pub plan_type: InspectionPlanType, // data_class: INTERNAL_ONLY
    pub characteristic_count: u32,     // data_class: FINANCIAL
    pub sample_size: u32,              // data_class: FINANCIAL
    pub acceptable_quality_limit_basis_points: u16, // data_class: FINANCIAL
    pub effective_from_yyyymmdd: u32,  // data_class: INTERNAL_ONLY
    pub specification_source_ref: String, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionPlanApproval {
    pub inspection_plan_id: Classified<InspectionPlanId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,       // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                    // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                      // data_class: INTERNAL_ONLY
    pub plan_type: Classified<InspectionPlanType>,        // data_class: INTERNAL_ONLY
    pub characteristic_count: Classified<u32>,            // data_class: FINANCIAL
    pub sample_size: Classified<u32>,                     // data_class: FINANCIAL
    pub acceptable_quality_limit_basis_points: Classified<u16>, // data_class: FINANCIAL
    pub effective_from_yyyymmdd: Classified<u32>,         // data_class: INTERNAL_ONLY
    pub specification_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>,   // data_class: INTERNAL_ONLY
    pub state: Classified<InspectionPlanState>,           // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,              // data_class: INTERNAL_ONLY
    pub inspection_runtime_attached: Classified<bool>,    // data_class: PUBLIC
    pub inventory_blocking_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>,    // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,      // data_class: PUBLIC
    pub schema_version: Classified<u32>,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionLotResultInput {
    pub inspection_lot_id: String,       // data_class: INTERNAL_ONLY
    pub inspection_plan_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub plant_id: String,                // data_class: INTERNAL_ONLY
    pub item_id: String,                 // data_class: INTERNAL_ONLY
    pub batch_id: String,                // data_class: INTERNAL_ONLY
    pub source_document_ref: String,     // data_class: INTERNAL_ONLY
    pub inspection_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub inspection_plan_approved: bool,  // data_class: INTERNAL_ONLY
    pub inspected_quantity: u32,         // data_class: FINANCIAL
    pub accepted_quantity: u32,          // data_class: FINANCIAL
    pub rejected_quantity: u32,          // data_class: FINANCIAL
    pub defect_count: u32,               // data_class: FINANCIAL
    pub acceptable_quality_limit_basis_points: u16, // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionLotUsageDecision {
    pub inspection_lot_id: Classified<InspectionLotId>, // data_class: INTERNAL_ONLY
    pub inspection_plan_id: Classified<InspectionPlanId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,     // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                  // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                    // data_class: INTERNAL_ONLY
    pub batch_id: Classified<BatchId>,                  // data_class: INTERNAL_ONLY
    pub source_document_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub inspection_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub inspected_quantity: Classified<u32>,            // data_class: FINANCIAL
    pub accepted_quantity: Classified<u32>,             // data_class: FINANCIAL
    pub rejected_quantity: Classified<u32>,             // data_class: FINANCIAL
    pub defect_count: Classified<u32>,                  // data_class: FINANCIAL
    pub acceptable_quality_limit_basis_points: Classified<u16>, // data_class: FINANCIAL
    pub max_rejected_quantity: Classified<u32>,         // data_class: FINANCIAL
    pub state: Classified<UsageDecisionState>,          // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,            // data_class: INTERNAL_ONLY
    pub inspection_passed: Classified<bool>,            // data_class: PUBLIC
    pub certificate_preparation_allowed: Classified<bool>, // data_class: PUBLIC
    pub quality_notification_required: Classified<bool>, // data_class: PUBLIC
    pub inventory_mutation_attached: Classified<bool>,  // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>,  // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,    // data_class: PUBLIC
    pub schema_version: Classified<u32>,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityCertificateInput {
    pub quality_certificate_id: String,   // data_class: INTERNAL_ONLY
    pub inspection_lot_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,          // data_class: INTERNAL_ONLY
    pub plant_id: String,                 // data_class: INTERNAL_ONLY
    pub item_id: String,                  // data_class: INTERNAL_ONLY
    pub batch_id: String,                 // data_class: INTERNAL_ONLY
    pub customer_id: String,              // data_class: INTERNAL_ONLY
    pub usage_decision_made: bool,        // data_class: INTERNAL_ONLY
    pub inspection_passed: bool,          // data_class: INTERNAL_ONLY
    pub characteristic_result_count: u32, // data_class: FINANCIAL
    pub certificate_profile_ref: String,  // data_class: INTERNAL_ONLY
    pub certificate_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityCertificatePreparation {
    pub quality_certificate_id: Classified<QualityCertificateId>, // data_class: INTERNAL_ONLY
    pub inspection_lot_id: Classified<InspectionLotId>,           // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                          // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,               // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                            // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                              // data_class: INTERNAL_ONLY
    pub batch_id: Classified<BatchId>,                            // data_class: INTERNAL_ONLY
    pub customer_id: Classified<CustomerId>,                      // data_class: INTERNAL_ONLY
    pub characteristic_result_count: Classified<u32>,             // data_class: FINANCIAL
    pub certificate_profile_ref: Classified<SourceDocumentRef>,   // data_class: INTERNAL_ONLY
    pub certificate_evidence_ref: Classified<EvidenceRef>,        // data_class: INTERNAL_ONLY
    pub state: Classified<CertificateState>,                      // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                      // data_class: INTERNAL_ONLY
    pub certificate_ready_for_output: Classified<bool>,           // data_class: PUBLIC
    pub pdf_rendering_attached: Classified<bool>,                 // data_class: PUBLIC
    pub email_delivery_attached: Classified<bool>,                // data_class: PUBLIC
    pub outbound_delivery_mutation_attached: Classified<bool>,    // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,              // data_class: PUBLIC
    pub schema_version: Classified<u32>,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityNotificationInput {
    pub quality_notification_id: String, // data_class: INTERNAL_ONLY
    pub inspection_lot_id: String,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub plant_id: String,                // data_class: INTERNAL_ONLY
    pub item_id: String,                 // data_class: INTERNAL_ONLY
    pub rejected_inspection: bool,       // data_class: INTERNAL_ONLY
    pub severity: QualitySeverity,       // data_class: INTERNAL_ONLY
    pub defect_count: u32,               // data_class: FINANCIAL
    pub corrective_action_due_days: u16, // data_class: INTERNAL_ONLY
    pub defect_source_ref: String,       // data_class: INTERNAL_ONLY
    pub notification_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityNotificationOpening {
    pub quality_notification_id: Classified<QualityNotificationId>, // data_class: INTERNAL_ONLY
    pub inspection_lot_id: Classified<InspectionLotId>,             // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                            // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,                 // data_class: INTERNAL_ONLY
    pub plant_id: Classified<PlantId>,                              // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                                // data_class: INTERNAL_ONLY
    pub severity: Classified<QualitySeverity>,                      // data_class: INTERNAL_ONLY
    pub defect_count: Classified<u32>,                              // data_class: FINANCIAL
    pub corrective_action_due_days: Classified<u16>,                // data_class: INTERNAL_ONLY
    pub defect_source_ref: Classified<SourceDocumentRef>,           // data_class: INTERNAL_ONLY
    pub notification_evidence_ref: Classified<EvidenceRef>,         // data_class: INTERNAL_ONLY
    pub state: Classified<QualityNotificationState>,                // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,                        // data_class: INTERNAL_ONLY
    pub corrective_action_required: Classified<bool>,               // data_class: PUBLIC
    pub capa_workflow_execution_attached: Classified<bool>,         // data_class: PUBLIC
    pub supplier_collaboration_network_attached: Classified<bool>,  // data_class: PUBLIC
    pub maintenance_notification_attached: Classified<bool>,        // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,                // data_class: PUBLIC
    pub schema_version: Classified<u32>,                            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualityManagementError {
    InvalidInspectionPlanId,
    InvalidInspectionLotId,
    InvalidQualityCertificateId,
    InvalidQualityNotificationId,
    InvalidPlantId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidItemId,
    InvalidBatchId,
    InvalidCustomerId,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidQuantity,
    InvalidAcceptableQualityLimit,
    InvalidEffectiveDate,
    InvalidInspectionResult,
    InspectionPlanApprovalRequired,
    UsageDecisionRequired,
    CertificateRequiresAcceptedInspection,
    RejectedInspectionRequired,
    InvalidCorrectiveActionDueDays,
}

pub fn approve_inspection_plan(
    input: InspectionPlanInput,
) -> Result<InspectionPlanApproval, QualityManagementError> {
    validate_inspection_plan_id(&input.inspection_plan_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_item_id(&input.item_id)?;
    validate_positive_quantity(input.characteristic_count)?;
    validate_positive_quantity(input.sample_size)?;
    validate_acceptable_quality_limit(input.acceptable_quality_limit_basis_points)?;
    validate_yyyymmdd(input.effective_from_yyyymmdd)?;
    validate_source_ref(&input.specification_source_ref)?;
    validate_evidence_ref(&input.approval_evidence_ref)?;
    let idempotency_key = format!(
        "quality-management:inspection-plan:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.inspection_plan_id
    );

    Ok(InspectionPlanApproval {
        inspection_plan_id: internal(InspectionPlanId {
            value: input.inspection_plan_id,
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
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        plan_type: internal(input.plan_type),
        characteristic_count: financial(input.characteristic_count),
        sample_size: financial(input.sample_size),
        acceptable_quality_limit_basis_points: financial(
            input.acceptable_quality_limit_basis_points,
        ),
        effective_from_yyyymmdd: internal(input.effective_from_yyyymmdd),
        specification_source_ref: internal(SourceDocumentRef {
            value: input.specification_source_ref,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        state: internal(InspectionPlanState::Approved),
        idempotency_key: internal(idempotency_key),
        inspection_runtime_attached: public(false),
        inventory_blocking_mutation_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(QUALITY_MANAGEMENT_SCHEMA_VERSION),
    })
}

pub fn record_inspection_lot_result(
    input: InspectionLotResultInput,
) -> Result<InspectionLotUsageDecision, QualityManagementError> {
    validate_inspection_lot_id(&input.inspection_lot_id)?;
    validate_inspection_plan_id(&input.inspection_plan_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_item_id(&input.item_id)?;
    validate_batch_id(&input.batch_id)?;
    validate_source_ref(&input.source_document_ref)?;
    validate_evidence_ref(&input.inspection_evidence_ref)?;
    if !input.inspection_plan_approved {
        return Err(QualityManagementError::InspectionPlanApprovalRequired);
    }
    validate_positive_quantity(input.inspected_quantity)?;
    validate_acceptable_quality_limit(input.acceptable_quality_limit_basis_points)?;
    let inspected_outcome_quantity = input
        .accepted_quantity
        .checked_add(input.rejected_quantity)
        .ok_or(QualityManagementError::InvalidInspectionResult)?;
    if inspected_outcome_quantity != input.inspected_quantity
        || input.defect_count > input.inspected_quantity
        || input.defect_count < input.rejected_quantity
    {
        return Err(QualityManagementError::InvalidInspectionResult);
    }
    let max_rejected_quantity = max_rejections_allowed(
        input.inspected_quantity,
        input.acceptable_quality_limit_basis_points,
    );
    let inspection_passed = input.rejected_quantity <= max_rejected_quantity;
    let state = if inspection_passed {
        UsageDecisionState::Accepted
    } else {
        UsageDecisionState::Rejected
    };
    let idempotency_key = format!(
        "quality-management:inspection-lot:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.inspection_lot_id
    );

    Ok(InspectionLotUsageDecision {
        inspection_lot_id: internal(InspectionLotId {
            value: input.inspection_lot_id,
        }),
        inspection_plan_id: internal(InspectionPlanId {
            value: input.inspection_plan_id,
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
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        batch_id: internal(BatchId {
            value: input.batch_id,
        }),
        source_document_ref: internal(SourceDocumentRef {
            value: input.source_document_ref,
        }),
        inspection_evidence_ref: internal(EvidenceRef {
            value: input.inspection_evidence_ref,
        }),
        inspected_quantity: financial(input.inspected_quantity),
        accepted_quantity: financial(input.accepted_quantity),
        rejected_quantity: financial(input.rejected_quantity),
        defect_count: financial(input.defect_count),
        acceptable_quality_limit_basis_points: financial(
            input.acceptable_quality_limit_basis_points,
        ),
        max_rejected_quantity: financial(max_rejected_quantity),
        state: internal(state),
        idempotency_key: internal(idempotency_key),
        inspection_passed: public(inspection_passed),
        certificate_preparation_allowed: public(inspection_passed),
        quality_notification_required: public(!inspection_passed),
        inventory_mutation_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(QUALITY_MANAGEMENT_SCHEMA_VERSION),
    })
}

pub fn prepare_quality_certificate(
    input: QualityCertificateInput,
) -> Result<QualityCertificatePreparation, QualityManagementError> {
    validate_quality_certificate_id(&input.quality_certificate_id)?;
    validate_inspection_lot_id(&input.inspection_lot_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_item_id(&input.item_id)?;
    validate_batch_id(&input.batch_id)?;
    validate_customer_id(&input.customer_id)?;
    if !input.usage_decision_made {
        return Err(QualityManagementError::UsageDecisionRequired);
    }
    if !input.inspection_passed {
        return Err(QualityManagementError::CertificateRequiresAcceptedInspection);
    }
    validate_positive_quantity(input.characteristic_result_count)?;
    validate_source_ref(&input.certificate_profile_ref)?;
    validate_evidence_ref(&input.certificate_evidence_ref)?;
    let idempotency_key = format!(
        "quality-management:certificate:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.quality_certificate_id
    );

    Ok(QualityCertificatePreparation {
        quality_certificate_id: internal(QualityCertificateId {
            value: input.quality_certificate_id,
        }),
        inspection_lot_id: internal(InspectionLotId {
            value: input.inspection_lot_id,
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
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        batch_id: internal(BatchId {
            value: input.batch_id,
        }),
        customer_id: internal(CustomerId {
            value: input.customer_id,
        }),
        characteristic_result_count: financial(input.characteristic_result_count),
        certificate_profile_ref: internal(SourceDocumentRef {
            value: input.certificate_profile_ref,
        }),
        certificate_evidence_ref: internal(EvidenceRef {
            value: input.certificate_evidence_ref,
        }),
        state: internal(CertificateState::Prepared),
        idempotency_key: internal(idempotency_key),
        certificate_ready_for_output: public(true),
        pdf_rendering_attached: public(false),
        email_delivery_attached: public(false),
        outbound_delivery_mutation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(QUALITY_MANAGEMENT_SCHEMA_VERSION),
    })
}

pub fn open_quality_notification(
    input: QualityNotificationInput,
) -> Result<QualityNotificationOpening, QualityManagementError> {
    validate_quality_notification_id(&input.quality_notification_id)?;
    validate_inspection_lot_id(&input.inspection_lot_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_plant_id(&input.plant_id)?;
    validate_item_id(&input.item_id)?;
    if !input.rejected_inspection {
        return Err(QualityManagementError::RejectedInspectionRequired);
    }
    validate_positive_quantity(input.defect_count)?;
    if !(1..=366).contains(&input.corrective_action_due_days) {
        return Err(QualityManagementError::InvalidCorrectiveActionDueDays);
    }
    validate_source_ref(&input.defect_source_ref)?;
    validate_evidence_ref(&input.notification_evidence_ref)?;
    let idempotency_key = format!(
        "quality-management:notification:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.plant_id, input.quality_notification_id
    );

    Ok(QualityNotificationOpening {
        quality_notification_id: internal(QualityNotificationId {
            value: input.quality_notification_id,
        }),
        inspection_lot_id: internal(InspectionLotId {
            value: input.inspection_lot_id,
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
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        severity: internal(input.severity),
        defect_count: financial(input.defect_count),
        corrective_action_due_days: internal(input.corrective_action_due_days),
        defect_source_ref: internal(SourceDocumentRef {
            value: input.defect_source_ref,
        }),
        notification_evidence_ref: internal(EvidenceRef {
            value: input.notification_evidence_ref,
        }),
        state: internal(QualityNotificationState::Opened),
        idempotency_key: internal(idempotency_key),
        corrective_action_required: public(true),
        capa_workflow_execution_attached: public(false),
        supplier_collaboration_network_attached: public(false),
        maintenance_notification_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(QUALITY_MANAGEMENT_SCHEMA_VERSION),
    })
}

fn validate_inspection_plan_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        INSPECTION_PLAN_ID_PREFIX,
        QualityManagementError::InvalidInspectionPlanId,
    )
}

fn validate_inspection_lot_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        INSPECTION_LOT_ID_PREFIX,
        QualityManagementError::InvalidInspectionLotId,
    )
}

fn validate_quality_certificate_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        QUALITY_CERTIFICATE_ID_PREFIX,
        QualityManagementError::InvalidQualityCertificateId,
    )
}

fn validate_quality_notification_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        QUALITY_NOTIFICATION_ID_PREFIX,
        QualityManagementError::InvalidQualityNotificationId,
    )
}

fn validate_plant_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        PLANT_ID_PREFIX,
        QualityManagementError::InvalidPlantId,
    )
}

fn validate_tenant_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        TENANT_ID_PREFIX,
        QualityManagementError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        QualityManagementError::InvalidLegalEntityId,
    )
}

fn validate_item_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(value, ITEM_ID_PREFIX, QualityManagementError::InvalidItemId)
}

fn validate_batch_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        BATCH_ID_PREFIX,
        QualityManagementError::InvalidBatchId,
    )
}

fn validate_customer_id(value: &str) -> Result<(), QualityManagementError> {
    validate_prefixed_identifier(
        value,
        CUSTOMER_ID_PREFIX,
        QualityManagementError::InvalidCustomerId,
    )
}

fn validate_prefixed_identifier(
    value: &str,
    prefix: &str,
    error: QualityManagementError,
) -> Result<(), QualityManagementError> {
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

fn validate_source_ref(value: &str) -> Result<(), QualityManagementError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        QualityManagementError::InvalidSourceDocumentRef,
    )
}

fn validate_evidence_ref(value: &str) -> Result<(), QualityManagementError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        QualityManagementError::InvalidEvidenceRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: QualityManagementError,
) -> Result<(), QualityManagementError> {
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

fn validate_positive_quantity(value: u32) -> Result<(), QualityManagementError> {
    if value == 0 {
        return Err(QualityManagementError::InvalidQuantity);
    }
    Ok(())
}

fn validate_acceptable_quality_limit(value: u16) -> Result<(), QualityManagementError> {
    if value == 0 || u32::from(value) > BASIS_POINTS_DENOMINATOR {
        return Err(QualityManagementError::InvalidAcceptableQualityLimit);
    }
    Ok(())
}

fn validate_yyyymmdd(value: u32) -> Result<(), QualityManagementError> {
    let year = value / 10_000;
    let month = (value / 100) % 100;
    let day = value % 100;
    if !(2020..=2100).contains(&year) || !(1..=12).contains(&month) {
        return Err(QualityManagementError::InvalidEffectiveDate);
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
        _ => return Err(QualityManagementError::InvalidEffectiveDate),
    };
    if day == 0 || day > max_day {
        return Err(QualityManagementError::InvalidEffectiveDate);
    }
    Ok(())
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn max_rejections_allowed(inspected_quantity: u32, aql_basis_points: u16) -> u32 {
    let product = u128::from(inspected_quantity) * u128::from(aql_basis_points);
    let quotient = product / u128::from(BASIS_POINTS_DENOMINATOR);
    quotient.min(u128::from(u32::MAX)) as u32
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
