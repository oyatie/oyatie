//! Accounting journal application layer.
//!
//! This crate turns pure accounting-domain outcomes into metadata-only audit
//! and Workflow dispatch envelopes for later cloud/runtime adapters. It does
//! not persist ledgers, call Workflow, file taxes, execute payments, or perform
//! network I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_accounting_journal_domain::{
    AccountingDomainError, EvidenceDigest, EvidenceRef, JournalId, JournalPostInput,
    JournalVoucher, LegalEntityId, PayrollPostingEvidence, PayrollPostingInput, PeriodState,
    TenantId, VatDeadlineInput, VatReturnId, VatReturnWorkflow, VatWorkflowStep, WorkflowRef,
    evaluate_vat_deadline, payroll_posting, payroll_posting_for_period, post_journal,
};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const ACCOUNTING_JOURNAL_TOPIC: &str = "audit.accounting.journal.posted";
const ACCOUNTING_PAYROLL_POSTING_TOPIC: &str = "audit.accounting.payroll.posted";
const ACCOUNTING_VAT_WORKFLOW_TOPIC: &str = "workflow.accounting.vat.dispatch";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingAuditEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub journal_id: Classified<JournalId>, // data_class: INTERNAL_ONLY
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingWorkflowDispatchEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub workflow_ref: Classified<WorkflowRef>, // data_class: INTERNAL_ONLY
    pub return_id: Classified<VatReturnId>, // data_class: INTERNAL_ONLY
    pub hometax_export_hash: Classified<EvidenceDigest>, // data_class: FINANCIAL
    pub required_steps: Classified<Vec<VatWorkflowStep>>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Classified<Vec<EvidenceRef>>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingPayrollPostingAuditEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub journal_id: Classified<JournalId>, // data_class: INTERNAL_ONLY
    pub source_payroll_digest: Classified<EvidenceDigest>, // data_class: FINANCIAL
    pub approval_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub reversal_path_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub wage_ledger_refs: Classified<Vec<EvidenceRef>>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalPostOutcome {
    pub journal: JournalVoucher,                 // data_class: FINANCIAL
    pub audit_envelope: AccountingAuditEnvelope, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VatWorkflowOutcome {
    pub workflow: Option<VatReturnWorkflow>, // data_class: FINANCIAL
    pub dispatch_envelope: Option<AccountingWorkflowDispatchEnvelope>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingPayrollPostingOutcome {
    pub evidence: PayrollPostingEvidence, // data_class: FINANCIAL
    pub audit_envelope: AccountingPayrollPostingAuditEnvelope, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountingAppError {
    Domain(AccountingDomainError),
}

impl From<AccountingDomainError> for AccountingAppError {
    fn from(error: AccountingDomainError) -> Self {
        Self::Domain(error)
    }
}

pub fn post_journal_with_audit(
    input: JournalPostInput,
) -> Result<JournalPostOutcome, AccountingAppError> {
    let journal = post_journal(input)?;
    let audit_envelope = journal_audit_envelope(&journal);
    Ok(JournalPostOutcome {
        journal,
        audit_envelope,
    })
}

pub fn plan_vat_workflow(
    input: VatDeadlineInput,
) -> Result<VatWorkflowOutcome, AccountingAppError> {
    let workflow = evaluate_vat_deadline(input)?;
    let dispatch_envelope = workflow.as_ref().map(vat_dispatch_envelope);
    Ok(VatWorkflowOutcome {
        workflow,
        dispatch_envelope,
    })
}

pub fn record_payroll_posting(
    input: PayrollPostingInput,
) -> Result<AccountingPayrollPostingOutcome, AccountingAppError> {
    let evidence = payroll_posting(input)?;
    let audit_envelope = payroll_posting_audit_envelope(&evidence);
    Ok(AccountingPayrollPostingOutcome {
        evidence,
        audit_envelope,
    })
}

pub fn record_payroll_posting_for_period(
    input: PayrollPostingInput,
    period_state: PeriodState,
) -> Result<AccountingPayrollPostingOutcome, AccountingAppError> {
    let evidence = payroll_posting_for_period(input, period_state)?;
    let audit_envelope = payroll_posting_audit_envelope(&evidence);
    Ok(AccountingPayrollPostingOutcome {
        evidence,
        audit_envelope,
    })
}

fn journal_audit_envelope(journal: &JournalVoucher) -> AccountingAuditEnvelope {
    AccountingAuditEnvelope {
        topic: internal(ACCOUNTING_JOURNAL_TOPIC.to_owned()),
        tenant_id: internal(journal.tenant_id.value.clone()),
        legal_entity_id: internal(journal.legal_entity_id.value.clone()),
        journal_id: internal(journal.journal_id.value.clone()),
        approval_evidence_ref: internal(journal.approval_evidence_ref.value.clone()),
        idempotency_key: internal(format!("{}:{}:posted", journal.journal_id.value.value, 1)),
        payload_data_class: internal(DataClass::Financial),
        schema_version: public(1),
    }
}

fn vat_dispatch_envelope(workflow: &VatReturnWorkflow) -> AccountingWorkflowDispatchEnvelope {
    AccountingWorkflowDispatchEnvelope {
        topic: internal(ACCOUNTING_VAT_WORKFLOW_TOPIC.to_owned()),
        tenant_id: internal(workflow.tenant_id.value.clone()),
        legal_entity_id: internal(workflow.legal_entity_id.value.clone()),
        workflow_ref: internal(workflow.workflow_ref.value.clone()),
        return_id: internal(workflow.return_id.value.clone()),
        hometax_export_hash: financial(workflow.hometax_export_hash.value.clone()),
        required_steps: internal(workflow.required_steps.value.clone()),
        evidence_refs: internal(workflow.evidence_paths.value.clone()),
        idempotency_key: internal(format!(
            "{}:{}:vat-workflow",
            workflow.tenant_id.value.value, workflow.return_id.value.value
        )),
        payload_data_class: internal(DataClass::Financial),
        schema_version: public(1),
    }
}

fn payroll_posting_audit_envelope(
    evidence: &PayrollPostingEvidence,
) -> AccountingPayrollPostingAuditEnvelope {
    AccountingPayrollPostingAuditEnvelope {
        topic: internal(ACCOUNTING_PAYROLL_POSTING_TOPIC.to_owned()),
        tenant_id: internal(evidence.journal.tenant_id.value.clone()),
        legal_entity_id: internal(evidence.journal.legal_entity_id.value.clone()),
        journal_id: internal(evidence.journal.journal_id.value.clone()),
        source_payroll_digest: financial(evidence.source_payroll_digest.value.clone()),
        approval_evidence_ref: internal(evidence.journal.approval_evidence_ref.value.clone()),
        reversal_path_ref: internal(evidence.reversal_path_ref.value.clone()),
        wage_ledger_refs: internal(evidence.wage_ledger_refs.value.clone()),
        idempotency_key: internal(format!(
            "{}:{}:payroll-posted",
            evidence.journal.tenant_id.value.value, evidence.journal.journal_id.value.value
        )),
        payload_data_class: internal(DataClass::Financial),
        schema_version: public(1),
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}
