//! Procurement source-to-pay domain foundation.
//!
//! This crate owns pure procurement invariants for supplier qualification,
//! purchase requisition approval, purchase order issuance, and three-way match
//! metadata for later accounting handoff. It does not perform persistence,
//! supplier portal/network calls, inventory mutation, payment execution,
//! workflow dispatch, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const SUPPLIER_ID_PREFIX: &str = "sup_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const EMPLOYEE_ID_PREFIX: &str = "emp_";
const REQUISITION_ID_PREFIX: &str = "preq_";
const PURCHASE_ORDER_ID_PREFIX: &str = "po_";
const RECEIPT_ID_PREFIX: &str = "gr_";
const INVOICE_ID_PREFIX: &str = "pinv_";
const MATCH_ID_PREFIX: &str = "pmatch_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const PROCUREMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SupplierId {
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
pub struct EmployeeId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PurchaseRequisitionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PurchaseOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoodsReceiptId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProcurementInvoiceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ThreeWayMatchId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceDocumentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoneyAmount {
    pub amount_minor: i64, // data_class: FINANCIAL
    pub currency: String,  // data_class: FINANCIAL
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SupplierQualificationStatus {
    Qualified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RequisitionState {
    Approved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PurchaseOrderState {
    Issued,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ThreeWayMatchState {
    Matched,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplierQualificationInput {
    pub supplier_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub supplier_profile_ref: String,        // data_class: INTERNAL_ONLY
    pub kyb_evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub risk_screening_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub approved_vendor_master_ref: String,  // data_class: INTERNAL_ONLY
    pub qualified_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplierQualification {
    pub supplier_id: Classified<SupplierId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,     // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub supplier_profile_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub kyb_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub risk_screening_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub approved_vendor_master_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub status: Classified<SupplierQualificationStatus>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub qualified_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurchaseRequisitionInput {
    pub requisition_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub requester_employee_id: String, // data_class: INTERNAL_ONLY
    pub approver_employee_id: String,  // data_class: INTERNAL_ONLY
    pub supplier_id: String,           // data_class: INTERNAL_ONLY
    pub supplier_qualified: bool,      // data_class: INTERNAL_ONLY
    pub amount: MoneyAmount,           // data_class: FINANCIAL
    pub budget_ref: String,            // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,   // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurchaseRequisitionApproval {
    pub requisition_id: Classified<PurchaseRequisitionId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,        // data_class: INTERNAL_ONLY
    pub requester_employee_id: Classified<EmployeeId>,     // data_class: INTERNAL_ONLY
    pub approver_employee_id: Classified<EmployeeId>,      // data_class: INTERNAL_ONLY
    pub supplier_id: Classified<SupplierId>,               // data_class: INTERNAL_ONLY
    pub amount: Classified<MoneyAmount>,                   // data_class: FINANCIAL
    pub budget_ref: Classified<SourceDocumentRef>,         // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: Classified<EvidenceRef>,      // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>,    // data_class: INTERNAL_ONLY
    pub state: Classified<RequisitionState>,               // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,               // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurchaseOrderInput {
    pub purchase_order_id: String,     // data_class: INTERNAL_ONLY
    pub requisition_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub supplier_id: String,           // data_class: INTERNAL_ONLY
    pub requisition_approved: bool,    // data_class: INTERNAL_ONLY
    pub amount: MoneyAmount,           // data_class: FINANCIAL
    pub contract_ref: String,          // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub issue_evidence_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurchaseOrder {
    pub purchase_order_id: Classified<PurchaseOrderId>, // data_class: INTERNAL_ONLY
    pub requisition_id: Classified<PurchaseRequisitionId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,     // data_class: INTERNAL_ONLY
    pub supplier_id: Classified<SupplierId>,            // data_class: INTERNAL_ONLY
    pub amount: Classified<MoneyAmount>,                // data_class: FINANCIAL
    pub contract_ref: Classified<SourceDocumentRef>,    // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub issue_evidence_ref: Classified<EvidenceRef>,    // data_class: INTERNAL_ONLY
    pub state: Classified<PurchaseOrderState>,          // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,            // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreeWayMatchInput {
    pub match_id: String,                        // data_class: INTERNAL_ONLY
    pub purchase_order_id: String,               // data_class: INTERNAL_ONLY
    pub receipt_id: String,                      // data_class: INTERNAL_ONLY
    pub invoice_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                 // data_class: INTERNAL_ONLY
    pub supplier_id: String,                     // data_class: INTERNAL_ONLY
    pub purchase_order_issued: bool,             // data_class: INTERNAL_ONLY
    pub ordered_quantity: u32,                   // data_class: FINANCIAL
    pub received_quantity: u32,                  // data_class: FINANCIAL
    pub invoiced_quantity: u32,                  // data_class: FINANCIAL
    pub ordered_amount: MoneyAmount,             // data_class: FINANCIAL
    pub invoiced_amount: MoneyAmount,            // data_class: FINANCIAL
    pub receipt_evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub invoice_evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub accounting_handoff_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreeWayMatchResult {
    pub match_id: Classified<ThreeWayMatchId>, // data_class: INTERNAL_ONLY
    pub purchase_order_id: Classified<PurchaseOrderId>, // data_class: INTERNAL_ONLY
    pub receipt_id: Classified<GoodsReceiptId>, // data_class: INTERNAL_ONLY
    pub invoice_id: Classified<ProcurementInvoiceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,       // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub supplier_id: Classified<SupplierId>,   // data_class: INTERNAL_ONLY
    pub matched_quantity: Classified<u32>,     // data_class: FINANCIAL
    pub matched_amount: Classified<MoneyAmount>, // data_class: FINANCIAL
    pub receipt_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub invoice_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub accounting_handoff_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<ThreeWayMatchState>, // data_class: INTERNAL_ONLY
    pub liability_draft_allowed: Classified<bool>, // data_class: PUBLIC
    pub payment_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub inventory_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub supplier_network_call_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcurementDomainError {
    InvalidSupplierId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidEmployeeId,
    InvalidPurchaseRequisitionId,
    InvalidPurchaseOrderId,
    InvalidGoodsReceiptId,
    InvalidInvoiceId,
    InvalidThreeWayMatchId,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidMoney,
    InvalidTimestamp,
    SupplierQualificationRequired,
    RequisitionApprovalRequired,
    PurchaseOrderIssueRequired,
    QuantityMismatch,
    AmountMismatch,
}

pub fn qualify_supplier(
    input: SupplierQualificationInput,
) -> Result<SupplierQualification, ProcurementDomainError> {
    validate_identifier(
        &input.supplier_id,
        SUPPLIER_ID_PREFIX,
        ProcurementDomainError::InvalidSupplierId,
    )?;
    validate_tenant_entity(&input.tenant_id, &input.legal_entity_id)?;
    validate_ref(
        &input.supplier_profile_ref,
        SOURCE_REF_PREFIX,
        ProcurementDomainError::InvalidSourceDocumentRef,
    )?;
    validate_evidence_ref(&input.kyb_evidence_ref)?;
    validate_evidence_ref(&input.risk_screening_evidence_ref)?;
    validate_ref(
        &input.approved_vendor_master_ref,
        SOURCE_REF_PREFIX,
        ProcurementDomainError::InvalidSourceDocumentRef,
    )?;
    if input.qualified_at_epoch_seconds == 0 {
        return Err(ProcurementDomainError::InvalidTimestamp);
    }
    let idempotency_key = format!(
        "{}:{}:{}",
        input.tenant_id, input.supplier_id, input.qualified_at_epoch_seconds
    );
    Ok(SupplierQualification {
        supplier_id: internal(SupplierId {
            value: input.supplier_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        supplier_profile_ref: internal(SourceDocumentRef {
            value: input.supplier_profile_ref,
        }),
        kyb_evidence_ref: internal(EvidenceRef {
            value: input.kyb_evidence_ref,
        }),
        risk_screening_evidence_ref: internal(EvidenceRef {
            value: input.risk_screening_evidence_ref,
        }),
        approved_vendor_master_ref: internal(SourceDocumentRef {
            value: input.approved_vendor_master_ref,
        }),
        status: internal(SupplierQualificationStatus::Qualified),
        idempotency_key: internal(idempotency_key),
        qualified_at_epoch_seconds: internal(input.qualified_at_epoch_seconds),
        schema_version: public(PROCUREMENT_SCHEMA_VERSION),
    })
}

pub fn approve_purchase_requisition(
    input: PurchaseRequisitionInput,
) -> Result<PurchaseRequisitionApproval, ProcurementDomainError> {
    validate_identifier(
        &input.requisition_id,
        REQUISITION_ID_PREFIX,
        ProcurementDomainError::InvalidPurchaseRequisitionId,
    )?;
    validate_tenant_entity(&input.tenant_id, &input.legal_entity_id)?;
    validate_identifier(
        &input.requester_employee_id,
        EMPLOYEE_ID_PREFIX,
        ProcurementDomainError::InvalidEmployeeId,
    )?;
    validate_identifier(
        &input.approver_employee_id,
        EMPLOYEE_ID_PREFIX,
        ProcurementDomainError::InvalidEmployeeId,
    )?;
    validate_identifier(
        &input.supplier_id,
        SUPPLIER_ID_PREFIX,
        ProcurementDomainError::InvalidSupplierId,
    )?;
    if !input.supplier_qualified {
        return Err(ProcurementDomainError::SupplierQualificationRequired);
    }
    validate_money(&input.amount)?;
    validate_ref(
        &input.budget_ref,
        SOURCE_REF_PREFIX,
        ProcurementDomainError::InvalidSourceDocumentRef,
    )?;
    validate_evidence_ref(&input.policy_evidence_ref)?;
    validate_evidence_ref(&input.approval_evidence_ref)?;
    let idempotency_key = format!("{}:{}:approved", input.tenant_id, input.requisition_id);
    Ok(PurchaseRequisitionApproval {
        requisition_id: internal(PurchaseRequisitionId {
            value: input.requisition_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        requester_employee_id: internal(EmployeeId {
            value: input.requester_employee_id,
        }),
        approver_employee_id: internal(EmployeeId {
            value: input.approver_employee_id,
        }),
        supplier_id: internal(SupplierId {
            value: input.supplier_id,
        }),
        amount: financial(input.amount),
        budget_ref: internal(SourceDocumentRef {
            value: input.budget_ref,
        }),
        policy_evidence_ref: internal(EvidenceRef {
            value: input.policy_evidence_ref,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        state: internal(RequisitionState::Approved),
        idempotency_key: internal(idempotency_key),
        schema_version: public(PROCUREMENT_SCHEMA_VERSION),
    })
}

pub fn issue_purchase_order(
    input: PurchaseOrderInput,
) -> Result<PurchaseOrder, ProcurementDomainError> {
    validate_identifier(
        &input.purchase_order_id,
        PURCHASE_ORDER_ID_PREFIX,
        ProcurementDomainError::InvalidPurchaseOrderId,
    )?;
    validate_identifier(
        &input.requisition_id,
        REQUISITION_ID_PREFIX,
        ProcurementDomainError::InvalidPurchaseRequisitionId,
    )?;
    validate_tenant_entity(&input.tenant_id, &input.legal_entity_id)?;
    validate_identifier(
        &input.supplier_id,
        SUPPLIER_ID_PREFIX,
        ProcurementDomainError::InvalidSupplierId,
    )?;
    if !input.requisition_approved {
        return Err(ProcurementDomainError::RequisitionApprovalRequired);
    }
    validate_money(&input.amount)?;
    validate_ref(
        &input.contract_ref,
        SOURCE_REF_PREFIX,
        ProcurementDomainError::InvalidSourceDocumentRef,
    )?;
    validate_evidence_ref(&input.approval_evidence_ref)?;
    validate_evidence_ref(&input.issue_evidence_ref)?;
    let idempotency_key = format!("{}:{}:issued", input.tenant_id, input.purchase_order_id);
    Ok(PurchaseOrder {
        purchase_order_id: internal(PurchaseOrderId {
            value: input.purchase_order_id,
        }),
        requisition_id: internal(PurchaseRequisitionId {
            value: input.requisition_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        supplier_id: internal(SupplierId {
            value: input.supplier_id,
        }),
        amount: financial(input.amount),
        contract_ref: internal(SourceDocumentRef {
            value: input.contract_ref,
        }),
        approval_evidence_ref: internal(EvidenceRef {
            value: input.approval_evidence_ref,
        }),
        issue_evidence_ref: internal(EvidenceRef {
            value: input.issue_evidence_ref,
        }),
        state: internal(PurchaseOrderState::Issued),
        idempotency_key: internal(idempotency_key),
        schema_version: public(PROCUREMENT_SCHEMA_VERSION),
    })
}

pub fn perform_three_way_match(
    input: ThreeWayMatchInput,
) -> Result<ThreeWayMatchResult, ProcurementDomainError> {
    validate_identifier(
        &input.match_id,
        MATCH_ID_PREFIX,
        ProcurementDomainError::InvalidThreeWayMatchId,
    )?;
    validate_identifier(
        &input.purchase_order_id,
        PURCHASE_ORDER_ID_PREFIX,
        ProcurementDomainError::InvalidPurchaseOrderId,
    )?;
    validate_identifier(
        &input.receipt_id,
        RECEIPT_ID_PREFIX,
        ProcurementDomainError::InvalidGoodsReceiptId,
    )?;
    validate_identifier(
        &input.invoice_id,
        INVOICE_ID_PREFIX,
        ProcurementDomainError::InvalidInvoiceId,
    )?;
    validate_tenant_entity(&input.tenant_id, &input.legal_entity_id)?;
    validate_identifier(
        &input.supplier_id,
        SUPPLIER_ID_PREFIX,
        ProcurementDomainError::InvalidSupplierId,
    )?;
    if !input.purchase_order_issued {
        return Err(ProcurementDomainError::PurchaseOrderIssueRequired);
    }
    if input.ordered_quantity == 0
        || input.ordered_quantity != input.received_quantity
        || input.ordered_quantity != input.invoiced_quantity
    {
        return Err(ProcurementDomainError::QuantityMismatch);
    }
    validate_money(&input.ordered_amount)?;
    validate_money(&input.invoiced_amount)?;
    if input.ordered_amount != input.invoiced_amount {
        return Err(ProcurementDomainError::AmountMismatch);
    }
    validate_evidence_ref(&input.receipt_evidence_ref)?;
    validate_evidence_ref(&input.invoice_evidence_ref)?;
    validate_evidence_ref(&input.accounting_handoff_evidence_ref)?;
    Ok(ThreeWayMatchResult {
        match_id: internal(ThreeWayMatchId {
            value: input.match_id,
        }),
        purchase_order_id: internal(PurchaseOrderId {
            value: input.purchase_order_id,
        }),
        receipt_id: internal(GoodsReceiptId {
            value: input.receipt_id,
        }),
        invoice_id: internal(ProcurementInvoiceId {
            value: input.invoice_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        supplier_id: internal(SupplierId {
            value: input.supplier_id,
        }),
        matched_quantity: financial(input.ordered_quantity),
        matched_amount: financial(input.ordered_amount),
        receipt_evidence_ref: internal(EvidenceRef {
            value: input.receipt_evidence_ref,
        }),
        invoice_evidence_ref: internal(EvidenceRef {
            value: input.invoice_evidence_ref,
        }),
        accounting_handoff_evidence_ref: internal(EvidenceRef {
            value: input.accounting_handoff_evidence_ref,
        }),
        state: internal(ThreeWayMatchState::Matched),
        liability_draft_allowed: public(true),
        payment_execution_attached: public(false),
        inventory_mutation_attached: public(false),
        supplier_network_call_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(PROCUREMENT_SCHEMA_VERSION),
    })
}

fn validate_tenant_entity(
    tenant_id: &str,
    legal_entity_id: &str,
) -> Result<(), ProcurementDomainError> {
    validate_identifier(
        tenant_id,
        TENANT_ID_PREFIX,
        ProcurementDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        ProcurementDomainError::InvalidLegalEntityId,
    )
}

fn validate_identifier(
    value: &str,
    prefix: &str,
    error: ProcurementDomainError,
) -> Result<(), ProcurementDomainError> {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return Err(error);
    };
    if suffix.is_empty()
        || has_unsafe_text(value)
        || suffix.contains("..")
        || !suffix
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(error);
    }
    Ok(())
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: ProcurementDomainError,
) -> Result<(), ProcurementDomainError> {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return Err(error);
    };
    if suffix.is_empty() || has_unsafe_text(value) || value.contains('\\') {
        return Err(error);
    }
    if value
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(error);
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.'))
    {
        return Err(error);
    }
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("bearer")
        || lowered.contains("password")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_evidence_ref(value: &str) -> Result<(), ProcurementDomainError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        ProcurementDomainError::InvalidEvidenceRef,
    )
}

fn validate_money(amount: &MoneyAmount) -> Result<(), ProcurementDomainError> {
    if amount.amount_minor <= 0
        || amount.currency.len() != 3
        || has_unsafe_text(&amount.currency)
        || !amount.currency.chars().all(|ch| ch.is_ascii_uppercase())
    {
        return Err(ProcurementDomainError::InvalidMoney);
    }
    Ok(())
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
