//! Tenant RBAC in-memory Workflow dispatch queue reference.
//!
//! SECURITY/OPERATIONS: NOT FOR PRODUCTION. This adapter is a volatile,
//! process-local reference implementation for the Tenant RBAC Workflow
//! dispatch seam. It records dispatch intents prepared by the app layer and can
//! deterministically close the required gates in-process so later Workflow-engine,
//! broker, and cloud adapters have a tested execution contract. It does not
//! publish to a broker, call downstream services, persist to a durable queue, emit
//! audit-chain events, or deploy cloud I/O.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use oya_tenant_rbac_usecase::CrossServiceWorkflowEnvelope;

const IN_MEMORY_WORKFLOW_QUEUE_LABEL: &str = "in-memory-workflow-reference";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TenantRbacWorkflowDispatchStatus {
    QueuedMetadataOnly,
    ExecutedInMemoryReference,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TenantRbacWorkflowExecutionStatus {
    CompletedInMemoryReference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacWorkflowExecutionGateRecord {
    pub gate_name: String,    // data_class: INTERNAL_ONLY
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
    pub gate_satisfied: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacWorkflowExecutionRecord {
    pub topic: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub workflow_ref: String,                  // data_class: INTERNAL_ONLY
    pub object_graph_relationship_ref: String, // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub dispatch_idempotency_key: String,      // data_class: INTERNAL_ONLY
    pub executed_gate_count: usize,            // data_class: INTERNAL_ONLY
    pub gate_records: Vec<TenantRbacWorkflowExecutionGateRecord>, // data_class: INTERNAL_ONLY
    pub execution_status: TenantRbacWorkflowExecutionStatus, // data_class: INTERNAL_ONLY
    pub queue_backend: String,                 // data_class: PUBLIC
    pub downstream_service_calls_attached: bool, // data_class: PUBLIC
    pub broker_publish_attached: bool,         // data_class: PUBLIC
    pub durable_queue_attached: bool,          // data_class: PUBLIC
    pub audit_chain_emission_attached: bool,   // data_class: PUBLIC
    pub schema_version: u32,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacWorkflowDispatchRecord {
    pub topic: String,                                     // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                 // data_class: INTERNAL_ONLY
    pub workflow_ref: String,                              // data_class: INTERNAL_ONLY
    pub object_graph_relationship_ref: String,             // data_class: INTERNAL_ONLY
    pub required_gate_count: usize,                        // data_class: INTERNAL_ONLY
    pub gate_evidence_count: usize,                        // data_class: INTERNAL_ONLY
    pub ai_suggestion_ref: Option<String>,                 // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                           // data_class: INTERNAL_ONLY
    pub payload_data_class: String,                        // data_class: INTERNAL_ONLY
    pub dispatch_status: TenantRbacWorkflowDispatchStatus, // data_class: INTERNAL_ONLY
    pub queue_backend: String,                             // data_class: PUBLIC
    pub schema_version: u32,                               // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacWorkflowQueueCapabilities {
    pub adapter: String,                              // data_class: PUBLIC
    pub durable_queue_attached: bool,                 // data_class: PUBLIC
    pub workflow_engine_attached: bool,               // data_class: PUBLIC
    pub broker_publish_attached: bool,                // data_class: PUBLIC
    pub runtime_execution_attached: bool,             // data_class: PUBLIC
    pub in_memory_execution_reference_attached: bool, // data_class: PUBLIC
    pub downstream_service_calls_attached: bool,      // data_class: PUBLIC
    pub audit_chain_emission_attached: bool,          // data_class: PUBLIC
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacWorkflowQueueError {
    DuplicateDispatch(String),
    InvalidIdempotencyKey(String),
    MissingGateEvidence,
    MissingDispatch(String),
    DuplicateExecution(String),
    AlreadyExecuted(String),
    GateEvidenceMismatch,
}

pub trait TenantRbacWorkflowDispatchPort {
    fn reserve_dispatch_key(&mut self, key: &str) -> Result<(), TenantRbacWorkflowQueueError>;
    fn enqueue_dispatch(
        &mut self,
        envelope: &CrossServiceWorkflowEnvelope,
    ) -> Result<TenantRbacWorkflowDispatchRecord, TenantRbacWorkflowQueueError>;
    fn get_dispatch(&self, idempotency_key: &str) -> Option<&TenantRbacWorkflowDispatchRecord>;
    fn require_dispatch(
        &self,
        idempotency_key: &str,
    ) -> Result<&TenantRbacWorkflowDispatchRecord, TenantRbacWorkflowQueueError>;
    fn execute_dispatch(
        &mut self,
        envelope: &CrossServiceWorkflowEnvelope,
    ) -> Result<TenantRbacWorkflowExecutionRecord, TenantRbacWorkflowQueueError>;
    fn get_execution(&self, idempotency_key: &str) -> Option<&TenantRbacWorkflowExecutionRecord>;
    fn list_dispatches(&self) -> Vec<&TenantRbacWorkflowDispatchRecord>;
    fn list_executions(&self) -> Vec<&TenantRbacWorkflowExecutionRecord>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryTenantRbacWorkflowQueue {
    dispatches_by_idempotency_key: BTreeMap<String, TenantRbacWorkflowDispatchRecord>,
    executions_by_idempotency_key: BTreeMap<String, TenantRbacWorkflowExecutionRecord>,
    reserved_dispatch_keys: BTreeSet<String>,
}

impl InMemoryTenantRbacWorkflowQueue {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TenantRbacWorkflowDispatchPort for InMemoryTenantRbacWorkflowQueue {
    fn reserve_dispatch_key(&mut self, key: &str) -> Result<(), TenantRbacWorkflowQueueError> {
        validate_idempotency_key(key)?;
        if self.dispatches_by_idempotency_key.contains_key(key)
            || !self.reserved_dispatch_keys.insert(key.to_owned())
        {
            return Err(TenantRbacWorkflowQueueError::DuplicateDispatch(
                key.to_owned(),
            ));
        }
        Ok(())
    }

    fn enqueue_dispatch(
        &mut self,
        envelope: &CrossServiceWorkflowEnvelope,
    ) -> Result<TenantRbacWorkflowDispatchRecord, TenantRbacWorkflowQueueError> {
        validate_idempotency_key(&envelope.idempotency_key.value)?;
        if envelope.required_gates.value.is_empty() || envelope.gate_evidence_refs.value.is_empty()
        {
            return Err(TenantRbacWorkflowQueueError::MissingGateEvidence);
        }
        if self
            .dispatches_by_idempotency_key
            .contains_key(&envelope.idempotency_key.value)
        {
            return Err(TenantRbacWorkflowQueueError::DuplicateDispatch(
                envelope.idempotency_key.value.clone(),
            ));
        }
        self.reserved_dispatch_keys
            .remove(&envelope.idempotency_key.value);
        let record = TenantRbacWorkflowDispatchRecord {
            topic: envelope.topic.value.clone(),
            tenant_id: envelope.tenant_id.value.value.clone(),
            workflow_ref: envelope.workflow_ref.value.value.clone(),
            object_graph_relationship_ref: envelope
                .object_graph_relationship_ref
                .value
                .value
                .clone(),
            required_gate_count: envelope.required_gates.value.len(),
            gate_evidence_count: envelope.gate_evidence_refs.value.len(),
            ai_suggestion_ref: envelope
                .ai_suggestion_ref
                .value
                .as_ref()
                .map(|suggestion| suggestion.value.clone()),
            idempotency_key: envelope.idempotency_key.value.clone(),
            payload_data_class: format!("{:?}", envelope.payload_data_class.value),
            dispatch_status: TenantRbacWorkflowDispatchStatus::QueuedMetadataOnly,
            queue_backend: IN_MEMORY_WORKFLOW_QUEUE_LABEL.to_owned(),
            schema_version: envelope.schema_version.value,
        };
        self.dispatches_by_idempotency_key
            .insert(record.idempotency_key.clone(), record.clone());
        Ok(record)
    }

    fn get_dispatch(&self, idempotency_key: &str) -> Option<&TenantRbacWorkflowDispatchRecord> {
        self.dispatches_by_idempotency_key.get(idempotency_key)
    }

    fn require_dispatch(
        &self,
        idempotency_key: &str,
    ) -> Result<&TenantRbacWorkflowDispatchRecord, TenantRbacWorkflowQueueError> {
        self.get_dispatch(idempotency_key).ok_or_else(|| {
            TenantRbacWorkflowQueueError::MissingDispatch(idempotency_key.to_owned())
        })
    }

    fn execute_dispatch(
        &mut self,
        envelope: &CrossServiceWorkflowEnvelope,
    ) -> Result<TenantRbacWorkflowExecutionRecord, TenantRbacWorkflowQueueError> {
        validate_idempotency_key(&envelope.idempotency_key.value)?;
        if self
            .executions_by_idempotency_key
            .contains_key(&envelope.idempotency_key.value)
        {
            return Err(TenantRbacWorkflowQueueError::DuplicateExecution(
                envelope.idempotency_key.value.clone(),
            ));
        }
        let mut dispatch = self
            .dispatches_by_idempotency_key
            .get(&envelope.idempotency_key.value)
            .cloned()
            .ok_or_else(|| {
                TenantRbacWorkflowQueueError::MissingDispatch(
                    envelope.idempotency_key.value.clone(),
                )
            })?;
        if dispatch.dispatch_status == TenantRbacWorkflowDispatchStatus::ExecutedInMemoryReference {
            return Err(TenantRbacWorkflowQueueError::AlreadyExecuted(
                envelope.idempotency_key.value.clone(),
            ));
        }
        if envelope.required_gates.value.len() != envelope.gate_evidence_refs.value.len()
            || envelope.required_gates.value.len() != dispatch.required_gate_count
            || envelope.gate_evidence_refs.value.len() != dispatch.gate_evidence_count
        {
            return Err(TenantRbacWorkflowQueueError::GateEvidenceMismatch);
        }
        let gate_records = envelope
            .required_gates
            .value
            .iter()
            .zip(envelope.gate_evidence_refs.value.iter())
            .map(|(gate, evidence)| TenantRbacWorkflowExecutionGateRecord {
                gate_name: format!("{gate:?}"),
                evidence_ref: evidence.value.clone(),
                gate_satisfied: true,
                schema_version: envelope.schema_version.value,
            })
            .collect::<Vec<_>>();
        let record = TenantRbacWorkflowExecutionRecord {
            topic: envelope.topic.value.clone(),
            tenant_id: envelope.tenant_id.value.value.clone(),
            workflow_ref: envelope.workflow_ref.value.value.clone(),
            object_graph_relationship_ref: envelope
                .object_graph_relationship_ref
                .value
                .value
                .clone(),
            idempotency_key: format!("{}:execution", envelope.idempotency_key.value),
            dispatch_idempotency_key: envelope.idempotency_key.value.clone(),
            executed_gate_count: gate_records.len(),
            gate_records,
            execution_status: TenantRbacWorkflowExecutionStatus::CompletedInMemoryReference,
            queue_backend: IN_MEMORY_WORKFLOW_QUEUE_LABEL.to_owned(),
            downstream_service_calls_attached: false,
            broker_publish_attached: false,
            durable_queue_attached: false,
            audit_chain_emission_attached: false,
            schema_version: envelope.schema_version.value,
        };
        dispatch.dispatch_status = TenantRbacWorkflowDispatchStatus::ExecutedInMemoryReference;
        self.dispatches_by_idempotency_key
            .insert(envelope.idempotency_key.value.clone(), dispatch);
        self.executions_by_idempotency_key
            .insert(envelope.idempotency_key.value.clone(), record.clone());
        Ok(record)
    }

    fn get_execution(&self, idempotency_key: &str) -> Option<&TenantRbacWorkflowExecutionRecord> {
        self.executions_by_idempotency_key.get(idempotency_key)
    }

    fn list_dispatches(&self) -> Vec<&TenantRbacWorkflowDispatchRecord> {
        self.dispatches_by_idempotency_key.values().collect()
    }

    fn list_executions(&self) -> Vec<&TenantRbacWorkflowExecutionRecord> {
        self.executions_by_idempotency_key.values().collect()
    }

    fn len(&self) -> usize {
        self.dispatches_by_idempotency_key.len()
    }

    fn is_empty(&self) -> bool {
        self.dispatches_by_idempotency_key.is_empty()
    }
}

pub fn tenant_rbac_workflow_queue_capabilities() -> TenantRbacWorkflowQueueCapabilities {
    TenantRbacWorkflowQueueCapabilities {
        adapter: IN_MEMORY_WORKFLOW_QUEUE_LABEL.to_owned(),
        durable_queue_attached: false,
        workflow_engine_attached: false,
        broker_publish_attached: false,
        runtime_execution_attached: false,
        in_memory_execution_reference_attached: true,
        downstream_service_calls_attached: false,
        audit_chain_emission_attached: false,
        schema_version: 1,
    }
}

fn validate_idempotency_key(key: &str) -> Result<(), TenantRbacWorkflowQueueError> {
    if key.trim().is_empty()
        || key.trim() != key
        || key.contains("..")
        || key.chars().any(char::is_whitespace)
    {
        return Err(TenantRbacWorkflowQueueError::InvalidIdempotencyKey(
            key.to_owned(),
        ));
    }
    Ok(())
}
