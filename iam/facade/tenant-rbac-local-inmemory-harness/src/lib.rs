//! Tenant RBAC local in-memory service harness.
//!
//! This crate composes the HR, Payroll, Accounting, and Tenant RBAC
//! in-memory adapter seams into one process-local harness for cloud-integration
//! rehearsal. It records real app-layer envelopes into service-specific
//! in-memory stores and queues Tenant RBAC Workflow dispatch metadata. It
//! does not deploy a listener, call downstream services over the network, attach a
//! durable backend or Postgres/RLS, execute Workflow, submit filings, disburse
//! funds, emit runtime audit-chain events, or deploy cloud infrastructure.
//! ADR-0083 Tier 3: tests legitimately use assertion helpers under the
//! `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use billing_accounting_app::AccountingPayrollPostingAuditEnvelope;
use billing_accounting_journal::{
    AccountingJournalStoragePort, AccountingStorageError, AccountingStoredRecord,
};
use billing_accounting_storage_inmemory_adapter::InMemoryAccountingJournalStore;
use hr_employment_app::HrLeavePayrollImpactEnvelope;
use hr_employment_storage_inmemory::{
    HrEmploymentStoragePort, HrStorageError, HrStoredRecord, InMemoryHrEmploymentStore,
};
use iam_tenant_rbac_storage_inmemory::{
    InMemoryTenantRbacStore, TenantRbacStorageError, TenantRbacStoragePort, TenantRbacStoredRecord,
};
use iam_tenant_rbac_usecase::CrossServiceWorkflowEnvelope;
use iam_tenant_rbac_workflow_inmemory::{
    InMemoryTenantRbacWorkflowQueue, TenantRbacWorkflowDispatchPort,
    TenantRbacWorkflowDispatchRecord, TenantRbacWorkflowQueueError,
};
use oya_payroll_run_app::{PayrollAccountingDispatchEnvelope, PayrollHrLeaveImpactEnvelope};
use oya_payroll_run_storage_adapter_inmemory::{
    InMemoryPayrollRunStore, PayrollRunStoragePort, PayrollStorageError, PayrollStoredRecord,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacLocalInMemoryHarnessCapabilities {
    pub harness: String,                              // data_class: PUBLIC
    pub in_memory_storage_integration_attached: bool, // data_class: PUBLIC
    pub durable_storage_attached: bool,               // data_class: PUBLIC
    pub postgres_rls_attached: bool,                  // data_class: PUBLIC
    pub deployed_listener_attached: bool,             // data_class: PUBLIC
    pub downstream_network_calls_attached: bool,      // data_class: PUBLIC
    pub workflow_engine_attached: bool,               // data_class: PUBLIC
    pub broker_publish_attached: bool,                // data_class: PUBLIC
    pub statutory_filing_rails_attached: bool,        // data_class: PUBLIC
    pub disbursement_rails_attached: bool,            // data_class: PUBLIC
    pub cloud_deployment_attached: bool,              // data_class: PUBLIC
    pub runtime_audit_chain_emission_attached: bool,  // data_class: PUBLIC
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantRbacLocalInMemoryHarnessSnapshot {
    pub hr_records: usize,                      // data_class: INTERNAL_ONLY
    pub payroll_records: usize,                 // data_class: INTERNAL_ONLY
    pub accounting_records: usize,              // data_class: INTERNAL_ONLY
    pub tenant_rbac_records: usize,             // data_class: INTERNAL_ONLY
    pub tenant_rbac_workflow_dispatches: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacWorkflowHarnessRecord {
    pub storage_record: TenantRbacStoredRecord, // data_class: INTERNAL_ONLY
    pub dispatch_record: TenantRbacWorkflowDispatchRecord, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacLocalInMemoryHarnessError {
    HrStorage(HrStorageError),
    PayrollStorage(PayrollStorageError),
    AccountingStorage(AccountingStorageError),
    TenantRbacStorage(TenantRbacStorageError),
    TenantRbacWorkflowQueue(TenantRbacWorkflowQueueError),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantRbacLocalInMemoryHarness {
    hr_store: InMemoryHrEmploymentStore,
    payroll_store: InMemoryPayrollRunStore,
    accounting_store: InMemoryAccountingJournalStore,
    tenant_rbac_store: InMemoryTenantRbacStore,
    tenant_rbac_workflow_queue: InMemoryTenantRbacWorkflowQueue,
}

impl TenantRbacLocalInMemoryHarness {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capabilities(&self) -> TenantRbacLocalInMemoryHarnessCapabilities {
        tenant_rbac_local_inmemory_harness_capabilities()
    }

    pub fn snapshot(&self) -> TenantRbacLocalInMemoryHarnessSnapshot {
        TenantRbacLocalInMemoryHarnessSnapshot {
            hr_records: self.hr_store.len(),
            payroll_records: self.payroll_store.len(),
            accounting_records: self.accounting_store.len(),
            tenant_rbac_records: self.tenant_rbac_store.len(),
            tenant_rbac_workflow_dispatches: self.tenant_rbac_workflow_queue.len(),
        }
    }

    pub fn record_hr_leave_payroll_impact(
        &mut self,
        envelope: &HrLeavePayrollImpactEnvelope,
    ) -> Result<HrStoredRecord, TenantRbacLocalInMemoryHarnessError> {
        self.hr_store
            .persist_leave_payroll_impact(envelope)
            .map_err(TenantRbacLocalInMemoryHarnessError::HrStorage)
    }

    pub fn record_payroll_hr_leave_impact_intake(
        &mut self,
        envelope: &PayrollHrLeaveImpactEnvelope,
    ) -> Result<PayrollStoredRecord, TenantRbacLocalInMemoryHarnessError> {
        self.payroll_store
            .persist_hr_leave_impact_intake(envelope)
            .map_err(TenantRbacLocalInMemoryHarnessError::PayrollStorage)
    }

    pub fn record_payroll_accounting_dispatch(
        &mut self,
        envelope: &PayrollAccountingDispatchEnvelope,
    ) -> Result<PayrollStoredRecord, TenantRbacLocalInMemoryHarnessError> {
        self.payroll_store
            .persist_accounting_dispatch(envelope)
            .map_err(TenantRbacLocalInMemoryHarnessError::PayrollStorage)
    }

    pub fn record_accounting_payroll_posting(
        &mut self,
        envelope: &AccountingPayrollPostingAuditEnvelope,
    ) -> Result<AccountingStoredRecord, TenantRbacLocalInMemoryHarnessError> {
        self.accounting_store
            .persist_payroll_posting_audit(envelope)
            .map_err(TenantRbacLocalInMemoryHarnessError::AccountingStorage)
    }

    pub fn record_tenant_rbac_workflow_dispatch(
        &mut self,
        envelope: &CrossServiceWorkflowEnvelope,
    ) -> Result<TenantRbacWorkflowHarnessRecord, TenantRbacLocalInMemoryHarnessError> {
        let storage_record = self
            .tenant_rbac_store
            .persist_cross_service_workflow(envelope)
            .map_err(TenantRbacLocalInMemoryHarnessError::TenantRbacStorage)?;
        let dispatch_record = self
            .tenant_rbac_workflow_queue
            .enqueue_dispatch(envelope)
            .map_err(TenantRbacLocalInMemoryHarnessError::TenantRbacWorkflowQueue)?;
        Ok(TenantRbacWorkflowHarnessRecord {
            storage_record,
            dispatch_record,
        })
    }
}

pub fn tenant_rbac_local_inmemory_harness_capabilities()
-> TenantRbacLocalInMemoryHarnessCapabilities {
    TenantRbacLocalInMemoryHarnessCapabilities {
        harness: "tenant-rbac-local-inmemory-harness".to_owned(),
        in_memory_storage_integration_attached: true,
        durable_storage_attached: false,
        postgres_rls_attached: false,
        deployed_listener_attached: false,
        downstream_network_calls_attached: false,
        workflow_engine_attached: false,
        broker_publish_attached: false,
        statutory_filing_rails_attached: false,
        disbursement_rails_attached: false,
        cloud_deployment_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: 1,
    }
}
