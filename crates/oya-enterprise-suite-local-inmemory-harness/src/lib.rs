//! Enterprise Suite local in-memory service harness.
//!
//! This crate composes the HR, Payroll, Accounting, and Enterprise Suite
//! in-memory adapter seams into one process-local harness for cloud-integration
//! rehearsal. It records real app-layer envelopes into service-specific
//! in-memory stores and queues Enterprise Suite Workflow dispatch metadata. It
//! does not deploy a listener, call child services over the network, attach a
//! durable backend or Postgres/RLS, execute Workflow, submit filings, disburse
//! funds, emit runtime audit-chain events, or deploy cloud infrastructure.
//! ADR-0083 Tier 3: tests legitimately use assertion helpers under the
//! `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_accounting_journal_app::AccountingPayrollPostingAuditEnvelope;
use oya_accounting_journal_storage_adapter_inmemory::{
    AccountingJournalStoragePort, AccountingStorageError, AccountingStoredRecord,
    InMemoryAccountingJournalStore,
};
use oya_enterprise_suite_app::CrossProductWorkflowEnvelope;
use oya_enterprise_suite_storage_adapter_inmemory::{
    EnterpriseSuiteStorageError, EnterpriseSuiteStoragePort, EnterpriseSuiteStoredRecord,
    InMemoryEnterpriseSuiteStore,
};
use oya_enterprise_suite_workflow_adapter_inmemory::{
    EnterpriseSuiteWorkflowDispatchPort, EnterpriseSuiteWorkflowDispatchRecord,
    EnterpriseSuiteWorkflowQueueError, InMemoryEnterpriseSuiteWorkflowQueue,
};
use oya_hr_employment_app::HrLeavePayrollImpactEnvelope;
use oya_hr_employment_storage_adapter_inmemory::{
    HrEmploymentStoragePort, HrStorageError, HrStoredRecord, InMemoryHrEmploymentStore,
};
use oya_payroll_run_app::{PayrollAccountingDispatchEnvelope, PayrollHrLeaveImpactEnvelope};
use oya_payroll_run_storage_adapter_inmemory::{
    InMemoryPayrollRunStore, PayrollRunStoragePort, PayrollStorageError, PayrollStoredRecord,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseLocalInMemoryHarnessCapabilities {
    pub harness: String,                              // data_class: PUBLIC
    pub in_memory_storage_integration_attached: bool, // data_class: PUBLIC
    pub durable_storage_attached: bool,               // data_class: PUBLIC
    pub postgres_rls_attached: bool,                  // data_class: PUBLIC
    pub deployed_listener_attached: bool,             // data_class: PUBLIC
    pub child_network_calls_attached: bool,           // data_class: PUBLIC
    pub workflow_engine_attached: bool,               // data_class: PUBLIC
    pub broker_publish_attached: bool,                // data_class: PUBLIC
    pub statutory_filing_rails_attached: bool,        // data_class: PUBLIC
    pub disbursement_rails_attached: bool,            // data_class: PUBLIC
    pub cloud_deployment_attached: bool,              // data_class: PUBLIC
    pub runtime_audit_chain_emission_attached: bool,  // data_class: PUBLIC
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnterpriseLocalInMemoryHarnessSnapshot {
    pub hr_records: usize,                     // data_class: INTERNAL_ONLY
    pub payroll_records: usize,                // data_class: INTERNAL_ONLY
    pub accounting_records: usize,             // data_class: INTERNAL_ONLY
    pub enterprise_suite_records: usize,       // data_class: INTERNAL_ONLY
    pub enterprise_workflow_dispatches: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseSuiteWorkflowHarnessRecord {
    pub storage_record: EnterpriseSuiteStoredRecord, // data_class: INTERNAL_ONLY
    pub dispatch_record: EnterpriseSuiteWorkflowDispatchRecord, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnterpriseLocalInMemoryHarnessError {
    HrStorage(HrStorageError),
    PayrollStorage(PayrollStorageError),
    AccountingStorage(AccountingStorageError),
    EnterpriseSuiteStorage(EnterpriseSuiteStorageError),
    EnterpriseSuiteWorkflowQueue(EnterpriseSuiteWorkflowQueueError),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnterpriseLocalInMemoryHarness {
    hr_store: InMemoryHrEmploymentStore,
    payroll_store: InMemoryPayrollRunStore,
    accounting_store: InMemoryAccountingJournalStore,
    enterprise_suite_store: InMemoryEnterpriseSuiteStore,
    enterprise_workflow_queue: InMemoryEnterpriseSuiteWorkflowQueue,
}

impl EnterpriseLocalInMemoryHarness {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capabilities(&self) -> EnterpriseLocalInMemoryHarnessCapabilities {
        enterprise_local_inmemory_harness_capabilities()
    }

    pub fn snapshot(&self) -> EnterpriseLocalInMemoryHarnessSnapshot {
        EnterpriseLocalInMemoryHarnessSnapshot {
            hr_records: self.hr_store.len(),
            payroll_records: self.payroll_store.len(),
            accounting_records: self.accounting_store.len(),
            enterprise_suite_records: self.enterprise_suite_store.len(),
            enterprise_workflow_dispatches: self.enterprise_workflow_queue.len(),
        }
    }

    pub fn record_hr_leave_payroll_impact(
        &mut self,
        envelope: &HrLeavePayrollImpactEnvelope,
    ) -> Result<HrStoredRecord, EnterpriseLocalInMemoryHarnessError> {
        self.hr_store
            .persist_leave_payroll_impact(envelope)
            .map_err(EnterpriseLocalInMemoryHarnessError::HrStorage)
    }

    pub fn record_payroll_hr_leave_impact_intake(
        &mut self,
        envelope: &PayrollHrLeaveImpactEnvelope,
    ) -> Result<PayrollStoredRecord, EnterpriseLocalInMemoryHarnessError> {
        self.payroll_store
            .persist_hr_leave_impact_intake(envelope)
            .map_err(EnterpriseLocalInMemoryHarnessError::PayrollStorage)
    }

    pub fn record_payroll_accounting_dispatch(
        &mut self,
        envelope: &PayrollAccountingDispatchEnvelope,
    ) -> Result<PayrollStoredRecord, EnterpriseLocalInMemoryHarnessError> {
        self.payroll_store
            .persist_accounting_dispatch(envelope)
            .map_err(EnterpriseLocalInMemoryHarnessError::PayrollStorage)
    }

    pub fn record_accounting_payroll_posting(
        &mut self,
        envelope: &AccountingPayrollPostingAuditEnvelope,
    ) -> Result<AccountingStoredRecord, EnterpriseLocalInMemoryHarnessError> {
        self.accounting_store
            .persist_payroll_posting_audit(envelope)
            .map_err(EnterpriseLocalInMemoryHarnessError::AccountingStorage)
    }

    pub fn record_enterprise_suite_workflow_dispatch(
        &mut self,
        envelope: &CrossProductWorkflowEnvelope,
    ) -> Result<EnterpriseSuiteWorkflowHarnessRecord, EnterpriseLocalInMemoryHarnessError> {
        let storage_record = self
            .enterprise_suite_store
            .persist_cross_product_workflow(envelope)
            .map_err(EnterpriseLocalInMemoryHarnessError::EnterpriseSuiteStorage)?;
        let dispatch_record = self
            .enterprise_workflow_queue
            .enqueue_dispatch(envelope)
            .map_err(EnterpriseLocalInMemoryHarnessError::EnterpriseSuiteWorkflowQueue)?;
        Ok(EnterpriseSuiteWorkflowHarnessRecord {
            storage_record,
            dispatch_record,
        })
    }
}

pub fn enterprise_local_inmemory_harness_capabilities() -> EnterpriseLocalInMemoryHarnessCapabilities
{
    EnterpriseLocalInMemoryHarnessCapabilities {
        harness: "enterprise-local-inmemory-harness".to_owned(),
        in_memory_storage_integration_attached: true,
        durable_storage_attached: false,
        postgres_rls_attached: false,
        deployed_listener_attached: false,
        child_network_calls_attached: false,
        workflow_engine_attached: false,
        broker_publish_attached: false,
        statutory_filing_rails_attached: false,
        disbursement_rails_attached: false,
        cloud_deployment_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: 1,
    }
}
