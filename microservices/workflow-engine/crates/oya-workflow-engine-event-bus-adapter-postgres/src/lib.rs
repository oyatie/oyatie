//! Workflow-engine event-bus Postgres adapter foundation.
//!
//! This crate provides source-level, plan-only Postgres semantics for the
//! workflow event-bus adapter seam. It defines tenant-scoped outbox, inbox, and
//! offset-observation SQL plans with Row-Level Security posture, idempotency
//! keys, `ON CONFLICT` guards, and `FOR UPDATE SKIP LOCKED` queue-claim SQL.
//! It never opens database connections, executes SQL, performs network I/O,
//! materializes payloads, coordinates consumer groups, commits offsets, signs
//! events, deploys to Kubernetes/cloud, schedules tenant workloads, or claims
//! durable event-bus runtime behavior.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_event_bus_adapter::{
    WORKFLOW_EVENT_BUS_ADAPTER_SURFACE, WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
    WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION, WorkflowEventBusAdapterDeliveryEnvelope,
    WorkflowEventBusAdapterDeliveryReceipt, WorkflowEventBusAdapterPublishEnvelope,
    WorkflowEventBusAdapterPublishReceipt, WorkflowEventBusEventKind,
};

pub const POSTGRES_EVENT_BUS_ADAPTER_SURFACE: &str = "workflow-engine.event-bus.adapter.postgres";
pub const POSTGRES_EVENT_BUS_ADAPTER_MODE_REF: &str =
    "workflow-event-bus-adapter-postgres:plan-only-preview";
pub const POSTGRES_EVENT_BUS_MAX_CLAIM_BATCH_SIZE: u32 = 1000;

pub const POSTGRES_EVENT_BUS_ADAPTER_NON_CLAIMS: [&str; 8] = [
    "workflow-event-bus-adapter-postgres:no-database-connection",
    "workflow-event-bus-adapter-postgres:no-sql-execution",
    "workflow-event-bus-adapter-postgres:no-broker-runtime",
    "workflow-event-bus-adapter-postgres:no-consumer-group-runtime",
    "workflow-event-bus-adapter-postgres:no-offset-commit-runtime",
    "workflow-event-bus-adapter-postgres:no-payload-materialization",
    "workflow-event-bus-adapter-postgres:no-cloud-runtime",
    "workflow-event-bus-adapter-postgres:no-hyperscaler-claim",
];

pub const POSTGRES_EVENT_BUS_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS workflow_event_bus_outbox (
  tenant_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  cell_id TEXT NOT NULL,
  channel_address TEXT NOT NULL,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  source_ref TEXT NOT NULL,
  subject_ref TEXT NULL,
  partition_key_ref TEXT NOT NULL,
  payload_ref TEXT NOT NULL,
  trace_context_ref TEXT NOT NULL,
  audit_chain_ref TEXT NOT NULL,
  asyncapi_channel_ref TEXT NOT NULL,
  cloudevents_specversion TEXT NOT NULL,
  evidence_refs TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, idempotency_key),
  UNIQUE (tenant_id, channel_address, event_id)
);

CREATE INDEX IF NOT EXISTS workflow_event_bus_outbox_pending_idx
ON workflow_event_bus_outbox (tenant_id, channel_address, created_at, event_id)
WHERE status = 'pending';

CREATE TABLE IF NOT EXISTS workflow_event_bus_inbox_deliveries (
  tenant_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  cell_id TEXT NOT NULL,
  channel_address TEXT NOT NULL,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  consumer_ref TEXT NOT NULL,
  offset_ref TEXT NOT NULL,
  payload_ref TEXT NOT NULL,
  replay_cursor_ref TEXT NULL,
  trace_context_ref TEXT NOT NULL,
  audit_chain_ref TEXT NOT NULL,
  delivery_status TEXT NOT NULL,
  offset_commit_planned BOOLEAN NOT NULL DEFAULT FALSE,
  evidence_refs TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, consumer_ref, idempotency_key),
  UNIQUE (tenant_id, consumer_ref, channel_address, offset_ref)
);

CREATE TABLE IF NOT EXISTS workflow_event_bus_offset_observations (
  tenant_id TEXT NOT NULL,
  consumer_ref TEXT NOT NULL,
  channel_address TEXT NOT NULL,
  offset_ref TEXT NOT NULL,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  commit_planned BOOLEAN NOT NULL DEFAULT FALSE,
  evidence_refs TEXT NOT NULL,
  observed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, consumer_ref, channel_address, offset_ref)
);

ALTER TABLE workflow_event_bus_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_event_bus_outbox FORCE ROW LEVEL SECURITY;
ALTER TABLE workflow_event_bus_inbox_deliveries ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_event_bus_inbox_deliveries FORCE ROW LEVEL SECURITY;
ALTER TABLE workflow_event_bus_offset_observations ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_event_bus_offset_observations FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS workflow_event_bus_outbox_tenant_isolation ON workflow_event_bus_outbox;
CREATE POLICY workflow_event_bus_outbox_tenant_isolation ON workflow_event_bus_outbox
USING (tenant_id = current_setting('oyatie.tenant_id', true))
WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

DROP POLICY IF EXISTS workflow_event_bus_inbox_tenant_isolation ON workflow_event_bus_inbox_deliveries;
CREATE POLICY workflow_event_bus_inbox_tenant_isolation ON workflow_event_bus_inbox_deliveries
USING (tenant_id = current_setting('oyatie.tenant_id', true))
WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

DROP POLICY IF EXISTS workflow_event_bus_offsets_tenant_isolation ON workflow_event_bus_offset_observations;
CREATE POLICY workflow_event_bus_offsets_tenant_isolation ON workflow_event_bus_offset_observations
USING (tenant_id = current_setting('oyatie.tenant_id', true))
WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
"#;

pub const POSTGRES_EVENT_BUS_OUTBOX_INSERT_SQL: &str = r#"
INSERT INTO workflow_event_bus_outbox (
  tenant_id, idempotency_key, cell_id, channel_address, event_id, event_type,
  source_ref, subject_ref, partition_key_ref, payload_ref, trace_context_ref,
  audit_chain_ref, asyncapi_channel_ref, cloudevents_specversion, evidence_refs, status
)
VALUES ($1, $2, $3, $4, $5, $6, $7, NULLIF($8, ''), $9, $10, $11, $12, $13, $14, $15, $16)
ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
RETURNING tenant_id, idempotency_key, status
"#;

pub const POSTGRES_EVENT_BUS_OUTBOX_LOAD_SQL: &str = r#"
SELECT tenant_id, idempotency_key, cell_id, channel_address, event_id, event_type,
       source_ref, subject_ref, partition_key_ref, payload_ref, trace_context_ref,
       audit_chain_ref, asyncapi_channel_ref, cloudevents_specversion, evidence_refs, status
FROM workflow_event_bus_outbox
WHERE tenant_id = $1 AND idempotency_key = $2
LIMIT 1
"#;

pub const POSTGRES_EVENT_BUS_OUTBOX_CLAIM_PENDING_SQL: &str = r#"
SELECT tenant_id, idempotency_key, cell_id, channel_address, event_id, event_type,
       source_ref, subject_ref, partition_key_ref, payload_ref, trace_context_ref,
       audit_chain_ref, asyncapi_channel_ref, cloudevents_specversion, evidence_refs, status
FROM workflow_event_bus_outbox
WHERE tenant_id = $1 AND channel_address = $2 AND status = 'pending'
ORDER BY created_at ASC, event_id ASC
FOR UPDATE SKIP LOCKED
LIMIT $3
"#;

pub const POSTGRES_EVENT_BUS_OUTBOX_MARK_STATUS_SQL: &str = r#"
UPDATE workflow_event_bus_outbox
SET status = $3, updated_at = now()
WHERE tenant_id = $1 AND idempotency_key = $2 AND status = $4
RETURNING status
"#;

pub const POSTGRES_EVENT_BUS_INBOX_INSERT_SQL: &str = r#"
INSERT INTO workflow_event_bus_inbox_deliveries (
  tenant_id, idempotency_key, cell_id, channel_address, event_id, event_type,
  consumer_ref, offset_ref, payload_ref, replay_cursor_ref, trace_context_ref,
  audit_chain_ref, delivery_status, offset_commit_planned, evidence_refs
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NULLIF($10, ''), $11, $12, $13, FALSE, $14)
ON CONFLICT (tenant_id, consumer_ref, idempotency_key) DO NOTHING
RETURNING tenant_id, consumer_ref, idempotency_key, delivery_status, offset_commit_planned
"#;

pub const POSTGRES_EVENT_BUS_OFFSET_OBSERVATION_SQL: &str = r#"
INSERT INTO workflow_event_bus_offset_observations (
  tenant_id, consumer_ref, channel_address, offset_ref, event_id, event_type,
  commit_planned, evidence_refs
)
VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7)
ON CONFLICT (tenant_id, consumer_ref, channel_address, offset_ref) DO UPDATE
SET event_id = EXCLUDED.event_id,
    event_type = EXCLUDED.event_type,
    commit_planned = FALSE,
    evidence_refs = EXCLUDED.evidence_refs,
    updated_at = now()
RETURNING tenant_id, consumer_ref, channel_address, offset_ref, commit_planned
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresEventBusQueryPlan {
    pub statement_name: String,              // data_class: INTERNAL_ONLY
    pub sql: String,                         // data_class: INTERNAL_ONLY
    pub params: Vec<String>,                 // data_class: INTERNAL_ONLY
    pub expected_status: Option<String>,     // data_class: INTERNAL_ONLY
    pub offset_commit_planned: Option<bool>, // data_class: INTERNAL_ONLY
}

pub type PostgresSqlPlan = PostgresEventBusQueryPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PostgresEventBusApplyOutcome {
    Applied,
    IdempotentNoop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostgresEventBusPlanFailure {
    InvalidBatchSize,
    InvalidStatus,
    PlanOnly { evidence_ref: String },
    TooManyRows { evidence_ref: String },
    UnsafeMetadata,
}

#[derive(Clone, Eq, PartialEq)]
pub struct PostgresEventBusPublishOutboxRow {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub cell_id: String,                 // data_class: INTERNAL_ONLY
    pub channel_address: String,         // data_class: PUBLIC
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub event_type: String,              // data_class: PUBLIC
    pub source_ref: String,              // data_class: INTERNAL_ONLY
    pub subject_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub partition_key_ref: String,       // data_class: INTERNAL_ONLY
    pub payload_ref: String,             // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,       // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,         // data_class: INTERNAL_ONLY
    pub asyncapi_channel_ref: String,    // data_class: INTERNAL_ONLY
    pub cloudevents_specversion: String, // data_class: PUBLIC
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
    pub status: String,                  // data_class: PUBLIC
}

impl std::fmt::Debug for PostgresEventBusPublishOutboxRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresEventBusPublishOutboxRow")
            .field("channel_address", &self.channel_address)
            .field("event_id", &self.event_id)
            .field("event_type", &self.event_type)
            .field("status", &self.status)
            .field("evidence_ref_count", &self.evidence_refs.len())
            .finish()
    }
}

impl PostgresEventBusPublishOutboxRow {
    pub fn from_envelope(envelope: &WorkflowEventBusAdapterPublishEnvelope) -> Self {
        Self {
            tenant_id: envelope.tenant_id.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
            cell_id: envelope.cell_id.clone(),
            channel_address: envelope.channel_address.clone(),
            event_id: envelope.event_id.clone(),
            event_type: envelope.event_type.clone(),
            source_ref: envelope.source_ref.clone(),
            subject_ref: envelope.subject_ref.clone(),
            partition_key_ref: envelope.partition_key_ref.clone(),
            payload_ref: envelope.payload_ref.clone(),
            trace_context_ref: envelope.trace_context_ref.clone(),
            audit_chain_ref: envelope.audit_chain_ref.clone(),
            asyncapi_channel_ref: envelope.asyncapi_channel_ref.clone().unwrap_or_default(),
            cloudevents_specversion: envelope.cloudevents_specversion.clone(),
            evidence_refs: sorted_unique(envelope.evidence_refs.clone()),
            status: "pending".to_owned(),
        }
    }

    pub fn to_envelope(
        &self,
    ) -> Result<WorkflowEventBusAdapterPublishEnvelope, PostgresEventBusPlanFailure> {
        validate_publish_row(self)?;
        Ok(WorkflowEventBusAdapterPublishEnvelope {
            tenant_id: self.tenant_id.clone(),
            cell_id: self.cell_id.clone(),
            channel_address: self.channel_address.clone(),
            event_id: self.event_id.clone(),
            event_type: self.event_type.clone(),
            source_ref: self.source_ref.clone(),
            subject_ref: self.subject_ref.clone(),
            partition_key_ref: self.partition_key_ref.clone(),
            payload_ref: self.payload_ref.clone(),
            idempotency_key: self.idempotency_key.clone(),
            trace_context_ref: self.trace_context_ref.clone(),
            audit_chain_ref: self.audit_chain_ref.clone(),
            asyncapi_channel_ref: Some(self.asyncapi_channel_ref.clone()),
            cloudevents_specversion: self.cloudevents_specversion.clone(),
            evidence_refs: sorted_unique(self.evidence_refs.clone()),
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PostgresEventBusDeliveryInboxRow {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub cell_id: String,                   // data_class: INTERNAL_ONLY
    pub channel_address: String,           // data_class: PUBLIC
    pub event_id: String,                  // data_class: INTERNAL_ONLY
    pub event_type: String,                // data_class: PUBLIC
    pub consumer_ref: String,              // data_class: INTERNAL_ONLY
    pub offset_ref: String,                // data_class: INTERNAL_ONLY
    pub payload_ref: String,               // data_class: INTERNAL_ONLY
    pub replay_cursor_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,         // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,           // data_class: INTERNAL_ONLY
    pub delivery_status: String,           // data_class: PUBLIC
    pub offset_commit_planned: bool,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for PostgresEventBusDeliveryInboxRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresEventBusDeliveryInboxRow")
            .field("channel_address", &self.channel_address)
            .field("event_id", &self.event_id)
            .field("event_type", &self.event_type)
            .field("consumer_ref", &self.consumer_ref)
            .field("delivery_status", &self.delivery_status)
            .field("offset_commit_planned", &self.offset_commit_planned)
            .field("evidence_ref_count", &self.evidence_refs.len())
            .finish()
    }
}

impl PostgresEventBusDeliveryInboxRow {
    pub fn from_envelope(
        envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
        delivery_status: &str,
    ) -> Self {
        Self {
            tenant_id: envelope.tenant_id.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
            cell_id: envelope.cell_id.clone(),
            channel_address: envelope.channel_address.clone(),
            event_id: envelope.event_id.clone(),
            event_type: envelope.event_type.clone(),
            consumer_ref: envelope.consumer_ref.clone(),
            offset_ref: envelope.offset_ref.clone(),
            payload_ref: envelope.payload_ref.clone(),
            replay_cursor_ref: envelope.replay_cursor_ref.clone(),
            trace_context_ref: envelope.trace_context_ref.clone(),
            audit_chain_ref: envelope.audit_chain_ref.clone(),
            delivery_status: delivery_status.to_owned(),
            offset_commit_planned: false,
            evidence_refs: sorted_unique(envelope.evidence_refs.clone()),
        }
    }

    pub fn to_envelope(
        &self,
    ) -> Result<WorkflowEventBusAdapterDeliveryEnvelope, PostgresEventBusPlanFailure> {
        validate_delivery_row(self)?;
        Ok(WorkflowEventBusAdapterDeliveryEnvelope {
            tenant_id: self.tenant_id.clone(),
            cell_id: self.cell_id.clone(),
            channel_address: self.channel_address.clone(),
            event_id: self.event_id.clone(),
            event_type: self.event_type.clone(),
            consumer_ref: self.consumer_ref.clone(),
            offset_ref: self.offset_ref.clone(),
            payload_ref: self.payload_ref.clone(),
            idempotency_key: self.idempotency_key.clone(),
            replay_cursor_ref: self.replay_cursor_ref.clone(),
            trace_context_ref: self.trace_context_ref.clone(),
            audit_chain_ref: self.audit_chain_ref.clone(),
            evidence_refs: sorted_unique(self.evidence_refs.clone()),
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PostgresEventBusOffsetObservationRow {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub consumer_ref: String,       // data_class: INTERNAL_ONLY
    pub channel_address: String,    // data_class: PUBLIC
    pub offset_ref: String,         // data_class: INTERNAL_ONLY
    pub event_id: String,           // data_class: INTERNAL_ONLY
    pub event_type: String,         // data_class: PUBLIC
    pub commit_planned: bool,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

impl std::fmt::Debug for PostgresEventBusOffsetObservationRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresEventBusOffsetObservationRow")
            .field("channel_address", &self.channel_address)
            .field("event_id", &self.event_id)
            .field("consumer_ref", &self.consumer_ref)
            .field("commit_planned", &self.commit_planned)
            .field("evidence_ref_count", &self.evidence_refs.len())
            .finish()
    }
}

impl PostgresEventBusOffsetObservationRow {
    pub fn from_envelope(envelope: &WorkflowEventBusAdapterDeliveryEnvelope) -> Self {
        Self {
            tenant_id: envelope.tenant_id.clone(),
            consumer_ref: envelope.consumer_ref.clone(),
            channel_address: envelope.channel_address.clone(),
            offset_ref: envelope.offset_ref.clone(),
            event_id: envelope.event_id.clone(),
            event_type: envelope.event_type.clone(),
            commit_planned: false,
            evidence_refs: sorted_unique(envelope.evidence_refs.clone()),
        }
    }
}

#[derive(Default)]
pub struct PostgresEventBusAdapter {
    generated_plans: Vec<PostgresEventBusQueryPlan>,
}

impl PostgresEventBusAdapter {
    pub fn publish_outbox_insert_plan(
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
    ) -> Result<PostgresEventBusQueryPlan, PostgresEventBusPlanFailure> {
        validate_publish_envelope(envelope)?;
        let row = PostgresEventBusPublishOutboxRow::from_envelope(envelope);
        validate_publish_row(&row)?;
        Ok(PostgresEventBusQueryPlan {
            statement_name: "workflow_event_bus_outbox_insert_idempotent".to_owned(),
            sql: POSTGRES_EVENT_BUS_OUTBOX_INSERT_SQL.to_owned(),
            params: vec![
                row.tenant_id,
                row.idempotency_key,
                row.cell_id,
                row.channel_address,
                row.event_id,
                row.event_type,
                row.source_ref,
                row.subject_ref.unwrap_or_default(),
                row.partition_key_ref,
                row.payload_ref,
                row.trace_context_ref,
                row.audit_chain_ref,
                row.asyncapi_channel_ref,
                row.cloudevents_specversion,
                row.evidence_refs.join("|"),
                row.status,
            ],
            expected_status: Some("pending".to_owned()),
            offset_commit_planned: None,
        })
    }

    pub fn outbox_load_plan(
        tenant_id: &str,
        idempotency_key: &str,
    ) -> Result<PostgresEventBusQueryPlan, PostgresEventBusPlanFailure> {
        if !is_safe_tenant(tenant_id) || !is_safe_ref(idempotency_key) {
            return Err(PostgresEventBusPlanFailure::UnsafeMetadata);
        }
        Ok(PostgresEventBusQueryPlan {
            statement_name: "workflow_event_bus_outbox_load_by_idempotency".to_owned(),
            sql: POSTGRES_EVENT_BUS_OUTBOX_LOAD_SQL.to_owned(),
            params: vec![tenant_id.to_owned(), idempotency_key.to_owned()],
            expected_status: None,
            offset_commit_planned: None,
        })
    }

    pub fn claim_pending_outbox_plan(
        tenant_id: &str,
        channel_address: &str,
        batch_size: u32,
    ) -> Result<PostgresEventBusQueryPlan, PostgresEventBusPlanFailure> {
        if batch_size == 0 || batch_size > POSTGRES_EVENT_BUS_MAX_CLAIM_BATCH_SIZE {
            return Err(PostgresEventBusPlanFailure::InvalidBatchSize);
        }
        if !is_safe_tenant(tenant_id) || !is_safe_metadata(channel_address) {
            return Err(PostgresEventBusPlanFailure::UnsafeMetadata);
        }
        Ok(PostgresEventBusQueryPlan {
            statement_name: "workflow_event_bus_outbox_claim_pending_skip_locked".to_owned(),
            sql: POSTGRES_EVENT_BUS_OUTBOX_CLAIM_PENDING_SQL.to_owned(),
            params: vec![
                tenant_id.to_owned(),
                channel_address.to_owned(),
                batch_size.to_string(),
            ],
            expected_status: Some("pending".to_owned()),
            offset_commit_planned: None,
        })
    }

    pub fn mark_outbox_status_plan(
        tenant_id: &str,
        idempotency_key: &str,
        expected_status: &str,
        next_status: &str,
        evidence_ref: &str,
    ) -> Result<PostgresEventBusQueryPlan, PostgresEventBusPlanFailure> {
        if !is_safe_tenant(tenant_id)
            || !is_safe_ref(idempotency_key)
            || !is_safe_ref(evidence_ref)
            || !is_valid_outbox_status(expected_status)
            || !is_valid_outbox_status(next_status)
        {
            return Err(PostgresEventBusPlanFailure::UnsafeMetadata);
        }
        Ok(PostgresEventBusQueryPlan {
            statement_name: "workflow_event_bus_outbox_mark_status_guarded".to_owned(),
            sql: POSTGRES_EVENT_BUS_OUTBOX_MARK_STATUS_SQL.to_owned(),
            params: vec![
                tenant_id.to_owned(),
                idempotency_key.to_owned(),
                next_status.to_owned(),
                expected_status.to_owned(),
                evidence_ref.to_owned(),
            ],
            expected_status: Some(expected_status.to_owned()),
            offset_commit_planned: None,
        })
    }

    pub fn delivery_inbox_insert_plan(
        envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
        delivery_status: &str,
    ) -> Result<PostgresEventBusQueryPlan, PostgresEventBusPlanFailure> {
        validate_delivery_envelope(envelope)?;
        if !is_valid_delivery_status(delivery_status) {
            return Err(PostgresEventBusPlanFailure::InvalidStatus);
        }
        let row = PostgresEventBusDeliveryInboxRow::from_envelope(envelope, delivery_status);
        validate_delivery_row(&row)?;
        Ok(PostgresEventBusQueryPlan {
            statement_name: "workflow_event_bus_inbox_insert_idempotent".to_owned(),
            sql: POSTGRES_EVENT_BUS_INBOX_INSERT_SQL.to_owned(),
            params: vec![
                row.tenant_id,
                row.idempotency_key,
                row.cell_id,
                row.channel_address,
                row.event_id,
                row.event_type,
                row.consumer_ref,
                row.offset_ref,
                row.payload_ref,
                row.replay_cursor_ref.unwrap_or_default(),
                row.trace_context_ref,
                row.audit_chain_ref,
                row.delivery_status,
                row.evidence_refs.join("|"),
            ],
            expected_status: Some(delivery_status.to_owned()),
            offset_commit_planned: Some(false),
        })
    }

    pub fn offset_observation_plan(
        envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
    ) -> Result<PostgresEventBusQueryPlan, PostgresEventBusPlanFailure> {
        validate_delivery_envelope(envelope)?;
        let row = PostgresEventBusOffsetObservationRow::from_envelope(envelope);
        validate_offset_row(&row)?;
        Ok(PostgresEventBusQueryPlan {
            statement_name: "workflow_event_bus_offset_observation_upsert_no_commit".to_owned(),
            sql: POSTGRES_EVENT_BUS_OFFSET_OBSERVATION_SQL.to_owned(),
            params: vec![
                row.tenant_id,
                row.consumer_ref,
                row.channel_address,
                row.offset_ref,
                row.event_id,
                row.event_type,
                row.evidence_refs.join("|"),
            ],
            expected_status: None,
            offset_commit_planned: Some(false),
        })
    }

    pub fn map_idempotent_insert_result(
        affected_rows: u64,
        evidence_ref: &str,
    ) -> Result<PostgresEventBusApplyOutcome, PostgresEventBusPlanFailure> {
        match affected_rows {
            1 => Ok(PostgresEventBusApplyOutcome::Applied),
            0 => Ok(PostgresEventBusApplyOutcome::IdempotentNoop),
            _ => Err(PostgresEventBusPlanFailure::TooManyRows {
                evidence_ref: safe_evidence_ref(
                    evidence_ref,
                    "workflow-event-bus-postgres-adapter:too-many-rows",
                ),
            }),
        }
    }

    pub fn plan_publish_outbox(
        &mut self,
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
    ) -> Result<(), PostgresEventBusPlanFailure> {
        let plan = Self::publish_outbox_insert_plan(envelope)?;
        self.generated_plans.push(plan);
        Err(PostgresEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-postgres-adapter:plan-only-publish-outbox".to_owned(),
        })
    }

    pub fn plan_delivery_inbox_and_offset(
        &mut self,
        envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
        delivery_status: &str,
    ) -> Result<(), PostgresEventBusPlanFailure> {
        let inbox = Self::delivery_inbox_insert_plan(envelope, delivery_status)?;
        let offset = Self::offset_observation_plan(envelope)?;
        self.generated_plans.push(inbox);
        self.generated_plans.push(offset);
        Err(PostgresEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-postgres-adapter:plan-only-delivery-inbox-offset"
                .to_owned(),
        })
    }

    pub fn generated_plans(&self) -> &[PostgresEventBusQueryPlan] {
        &self.generated_plans
    }
}

fn validate_publish_envelope(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), PostgresEventBusPlanFailure> {
    let asyncapi_safe = envelope
        .asyncapi_channel_ref
        .as_deref()
        .is_some_and(|value| {
            is_safe_metadata(value)
                && value.starts_with(WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF)
                && value.contains("#/channels/")
        });
    let valid = is_safe_tenant(&envelope.tenant_id)
        && is_safe_ref(&envelope.cell_id)
        && is_safe_metadata(&envelope.channel_address)
        && is_safe_ref(&envelope.event_id)
        && is_safe_metadata(&envelope.event_type)
        && is_safe_ref(&envelope.source_ref)
        && is_safe_optional_ref(envelope.subject_ref.as_deref())
        && is_safe_ref(&envelope.partition_key_ref)
        && is_safe_ref(&envelope.payload_ref)
        && is_safe_ref(&envelope.idempotency_key)
        && is_safe_ref(&envelope.trace_context_ref)
        && is_safe_ref(&envelope.audit_chain_ref)
        && asyncapi_safe
        && envelope.cloudevents_specversion == WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION
        && envelope
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value));
    if valid {
        Ok(())
    } else {
        Err(PostgresEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_delivery_envelope(
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> Result<(), PostgresEventBusPlanFailure> {
    let valid = is_safe_tenant(&envelope.tenant_id)
        && is_safe_ref(&envelope.cell_id)
        && is_safe_metadata(&envelope.channel_address)
        && is_safe_ref(&envelope.event_id)
        && is_safe_metadata(&envelope.event_type)
        && is_safe_ref(&envelope.consumer_ref)
        && is_safe_ref(&envelope.offset_ref)
        && is_safe_ref(&envelope.payload_ref)
        && is_safe_ref(&envelope.idempotency_key)
        && is_safe_optional_ref(envelope.replay_cursor_ref.as_deref())
        && is_safe_ref(&envelope.trace_context_ref)
        && is_safe_ref(&envelope.audit_chain_ref)
        && envelope
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value));
    if valid {
        Ok(())
    } else {
        Err(PostgresEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_publish_row(
    row: &PostgresEventBusPublishOutboxRow,
) -> Result<(), PostgresEventBusPlanFailure> {
    if is_safe_tenant(&row.tenant_id)
        && is_safe_ref(&row.idempotency_key)
        && is_safe_ref(&row.cell_id)
        && is_safe_metadata(&row.channel_address)
        && is_safe_ref(&row.event_id)
        && is_safe_metadata(&row.event_type)
        && is_safe_ref(&row.source_ref)
        && is_safe_optional_ref(row.subject_ref.as_deref())
        && is_safe_ref(&row.partition_key_ref)
        && is_safe_ref(&row.payload_ref)
        && is_safe_ref(&row.trace_context_ref)
        && is_safe_ref(&row.audit_chain_ref)
        && is_safe_metadata(&row.asyncapi_channel_ref)
        && row
            .asyncapi_channel_ref
            .starts_with(WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF)
        && row.cloudevents_specversion == WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION
        && is_valid_outbox_status(&row.status)
        && row.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_delivery_row(
    row: &PostgresEventBusDeliveryInboxRow,
) -> Result<(), PostgresEventBusPlanFailure> {
    if is_safe_tenant(&row.tenant_id)
        && is_safe_ref(&row.idempotency_key)
        && is_safe_ref(&row.cell_id)
        && is_safe_metadata(&row.channel_address)
        && is_safe_ref(&row.event_id)
        && is_safe_metadata(&row.event_type)
        && is_safe_ref(&row.consumer_ref)
        && is_safe_ref(&row.offset_ref)
        && is_safe_ref(&row.payload_ref)
        && is_safe_optional_ref(row.replay_cursor_ref.as_deref())
        && is_safe_ref(&row.trace_context_ref)
        && is_safe_ref(&row.audit_chain_ref)
        && is_valid_delivery_status(&row.delivery_status)
        && !row.offset_commit_planned
        && row.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_offset_row(
    row: &PostgresEventBusOffsetObservationRow,
) -> Result<(), PostgresEventBusPlanFailure> {
    if is_safe_tenant(&row.tenant_id)
        && is_safe_ref(&row.consumer_ref)
        && is_safe_metadata(&row.channel_address)
        && is_safe_ref(&row.offset_ref)
        && is_safe_ref(&row.event_id)
        && is_safe_metadata(&row.event_type)
        && !row.commit_planned
        && row.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(PostgresEventBusPlanFailure::UnsafeMetadata)
    }
}

fn is_valid_outbox_status(value: &str) -> bool {
    matches!(
        value,
        "pending" | "broker-publish-planned" | "published" | "failed"
    )
}

fn is_valid_delivery_status(value: &str) -> bool {
    matches!(value, "delivery-accepted" | "delivery-denied")
}

fn safe_evidence_ref(value: &str, fallback: &str) -> String {
    if is_safe_ref(value) {
        value.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_ref)
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
        || lower.contains("raw payload")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| {
        !value.trim().is_empty()
            && !contains_raw_secret_material(value)
            && !contains_raw_content_material(value)
    });
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_workflow_engine_event_bus_adapter::{
        WORKFLOW_EVENT_BUS_API_DECLARED_VERSION, WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
        WORKFLOW_EVENT_BUS_API_METHOD, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
        WORKFLOW_EVENT_BUS_API_SURFACE, WorkflowEventBusApi, WorkflowEventBusApiAuthorization,
        WorkflowEventBusApiBoundaryContext, WorkflowEventBusApiDeliveryBody,
        WorkflowEventBusApiDeliveryRequest, WorkflowEventBusApiPrincipal,
        WorkflowEventBusApiPublishBody, WorkflowEventBusApiPublishRequest,
        WorkflowEventBusApiSuccessResponse, WorkflowEventBusMemoryAdapter,
    };

    #[test]
    fn ddl_uses_tenant_keys_rls_idempotency_and_skip_locked_claims() {
        assert!(POSTGRES_EVENT_BUS_DDL.contains("PRIMARY KEY (tenant_id, idempotency_key)"));
        assert!(POSTGRES_EVENT_BUS_DDL.contains("UNIQUE (tenant_id, channel_address, event_id)"));
        assert!(POSTGRES_EVENT_BUS_DDL.contains("ENABLE ROW LEVEL SECURITY"));
        assert!(POSTGRES_EVENT_BUS_DDL.contains("FORCE ROW LEVEL SECURITY"));
        assert!(POSTGRES_EVENT_BUS_DDL.contains("current_setting('oyatie.tenant_id', true)"));
        assert!(POSTGRES_EVENT_BUS_OUTBOX_CLAIM_PENDING_SQL.contains("FOR UPDATE SKIP LOCKED"));
        assert!(
            POSTGRES_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-postgres:no-sql-execution")
        );
    }

    #[test]
    fn publish_outbox_plan_is_parameterized_idempotent_and_plan_only() {
        let envelope = publish_envelope();
        let plan = PostgresEventBusAdapter::publish_outbox_insert_plan(&envelope).unwrap();

        assert_eq!(
            plan.statement_name,
            "workflow_event_bus_outbox_insert_idempotent"
        );
        assert_eq!(plan.params.len(), 16);
        assert_eq!(plan.params[0], "ten_workflow_event_bus");
        assert_eq!(plan.params[3], "workflow.runs.events.v1");
        assert_eq!(plan.params[15], "pending");
        assert!(
            plan.sql
                .contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING")
        );
        assert!(
            plan.sql
                .contains("RETURNING tenant_id, idempotency_key, status")
        );

        let mut adapter = PostgresEventBusAdapter::default();
        let err = adapter.plan_publish_outbox(&envelope).unwrap_err();
        assert_eq!(adapter.generated_plans().len(), 1);
        assert_eq!(
            err,
            PostgresEventBusPlanFailure::PlanOnly {
                evidence_ref: "workflow-event-bus-postgres-adapter:plan-only-publish-outbox"
                    .to_owned(),
            }
        );
    }

    #[test]
    fn delivery_inbox_and_offset_plans_preserve_no_offset_commit() {
        let envelope = delivery_envelope("delivery-accepted");
        let inbox =
            PostgresEventBusAdapter::delivery_inbox_insert_plan(&envelope, "delivery-accepted")
                .unwrap();
        let offset = PostgresEventBusAdapter::offset_observation_plan(&envelope).unwrap();

        assert_eq!(inbox.params.len(), 14);
        assert_eq!(inbox.expected_status.as_deref(), Some("delivery-accepted"));
        assert_eq!(inbox.offset_commit_planned, Some(false));
        assert!(inbox.sql.contains("offset_commit_planned, evidence_refs"));
        assert!(inbox.sql.contains("FALSE"));
        assert!(!inbox.sql.to_ascii_lowercase().contains("commit offset"));
        assert_eq!(offset.offset_commit_planned, Some(false));
        assert!(offset.sql.contains("commit_planned = FALSE"));
        assert!(
            offset
                .sql
                .contains("ON CONFLICT (tenant_id, consumer_ref, channel_address, offset_ref)")
        );

        let mut adapter = PostgresEventBusAdapter::default();
        assert!(matches!(
            adapter.plan_delivery_inbox_and_offset(&envelope, "delivery-accepted"),
            Err(PostgresEventBusPlanFailure::PlanOnly { .. })
        ));
        assert_eq!(adapter.generated_plans().len(), 2);
    }

    #[test]
    fn claim_and_mark_plans_are_tenant_channel_and_status_guarded() {
        let claim = PostgresEventBusAdapter::claim_pending_outbox_plan(
            "ten_workflow_event_bus",
            "workflow.runs.events.v1",
            25,
        )
        .unwrap();
        assert_eq!(claim.params[2], "25");
        assert!(
            claim
                .sql
                .contains("WHERE tenant_id = $1 AND channel_address = $2")
        );
        assert!(claim.sql.contains("status = 'pending'"));
        assert!(claim.sql.contains("FOR UPDATE SKIP LOCKED"));

        let mark = PostgresEventBusAdapter::mark_outbox_status_plan(
            "ten_workflow_event_bus",
            "idem:event-bus-adapter:publish:1",
            "pending",
            "broker-publish-planned",
            "evidence:event-bus-pg:mark",
        )
        .unwrap();
        assert_eq!(mark.params[2], "broker-publish-planned");
        assert_eq!(mark.params[3], "pending");
        assert!(mark.sql.contains("AND status = $4"));
        assert_eq!(
            PostgresEventBusAdapter::claim_pending_outbox_plan(
                "ten_workflow_event_bus",
                "workflow.runs.events.v1",
                0,
            )
            .unwrap_err(),
            PostgresEventBusPlanFailure::InvalidBatchSize
        );
    }

    #[test]
    fn row_mapping_round_trips_publish_and_delivery_without_payload_debug() {
        let publish_row = PostgresEventBusPublishOutboxRow::from_envelope(&publish_envelope());
        assert_eq!(publish_row.to_envelope().unwrap(), publish_envelope());
        assert!(
            !format!("{publish_row:?}")
                .to_ascii_lowercase()
                .contains("payload")
        );
        assert!(
            !format!("{publish_row:?}")
                .to_ascii_lowercase()
                .contains("secret")
        );

        let delivery = delivery_envelope("delivery-denied");
        let delivery_row =
            PostgresEventBusDeliveryInboxRow::from_envelope(&delivery, "delivery-denied");
        assert_eq!(delivery_row.to_envelope().unwrap(), delivery);
        assert!(
            !format!("{delivery_row:?}")
                .to_ascii_lowercase()
                .contains("payload")
        );
        assert!(
            !format!("{delivery_row:?}")
                .to_ascii_lowercase()
                .contains("secret")
        );

        let offset_row = PostgresEventBusOffsetObservationRow::from_envelope(&delivery);
        assert!(!offset_row.commit_planned);
        assert!(
            !format!("{offset_row:?}")
                .to_ascii_lowercase()
                .contains("payload")
        );
    }

    #[test]
    fn unsafe_raw_metadata_is_rejected_before_sql_plan_without_echo() {
        let mut envelope = publish_envelope();
        envelope.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();
        let err = PostgresEventBusAdapter::publish_outbox_insert_plan(&envelope).unwrap_err();

        assert_eq!(err, PostgresEventBusPlanFailure::UnsafeMetadata);
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
    }

    #[test]
    fn affected_rows_mapping_distinguishes_applied_noop_and_too_many_rows() {
        assert_eq!(
            PostgresEventBusAdapter::map_idempotent_insert_result(1, "pg:event-bus:applied"),
            Ok(PostgresEventBusApplyOutcome::Applied)
        );
        assert_eq!(
            PostgresEventBusAdapter::map_idempotent_insert_result(0, "pg:event-bus:noop"),
            Ok(PostgresEventBusApplyOutcome::IdempotentNoop)
        );
        assert_eq!(
            PostgresEventBusAdapter::map_idempotent_insert_result(
                2,
                "raw payload Authorization: Bearer sk-test",
            )
            .unwrap_err(),
            PostgresEventBusPlanFailure::TooManyRows {
                evidence_ref: "workflow-event-bus-postgres-adapter:too-many-rows".to_owned(),
            }
        );
    }

    #[test]
    fn api_generic_adapter_and_postgres_plans_integrate_without_runtime_claims() {
        let mut api = WorkflowEventBusApi::default();
        let publish_success = api
            .publish_event(publish_request("idem:event-bus-pg:publish"))
            .unwrap();
        let delivery_success = api
            .evaluate_delivery(delivery_request("idem:event-bus-pg:delivery"))
            .unwrap();
        let mut memory_adapter = WorkflowEventBusMemoryAdapter::default();
        let publish_receipt = memory_adapter
            .record_publish_from_api_success(
                &publish_success,
                publish_envelope_from_api(&publish_success),
            )
            .unwrap();
        let delivery_receipt = memory_adapter
            .record_delivery_from_api_success(
                &delivery_success,
                delivery_envelope_from_api(&delivery_success),
            )
            .unwrap();

        let publish_plan = PostgresEventBusAdapter::publish_outbox_insert_plan(
            &publish_envelope_from_api(&publish_success),
        )
        .unwrap();
        let delivery_plan = PostgresEventBusAdapter::delivery_inbox_insert_plan(
            &delivery_envelope_from_api(&delivery_success),
            &delivery_receipt.delivery_status,
        )
        .unwrap();
        let offset_plan = PostgresEventBusAdapter::offset_observation_plan(
            &delivery_envelope_from_api(&delivery_success),
        )
        .unwrap();

        assert!(
            publish_receipt
                .non_claim_refs
                .iter()
                .any(|value| value.contains("no-broker"))
        );
        assert!(
            delivery_receipt
                .non_claim_refs
                .iter()
                .any(|value| value.contains("no-offset-commit"))
        );
        assert_eq!(publish_plan.expected_status.as_deref(), Some("pending"));
        assert_eq!(delivery_plan.offset_commit_planned, Some(false));
        assert_eq!(offset_plan.offset_commit_planned, Some(false));
    }

    fn publish_envelope() -> WorkflowEventBusAdapterPublishEnvelope {
        WorkflowEventBusAdapterPublishEnvelope {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            channel_address: "workflow.runs.events.v1".to_owned(),
            event_id: "event:workflow-run-started:001".to_owned(),
            event_type: WorkflowEventBusEventKind::WorkflowRunStarted
                .event_type()
                .to_owned(),
            source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
            subject_ref: Some("subject:workflow-run:001".to_owned()),
            partition_key_ref: "partition:tenant-workflow-run".to_owned(),
            payload_ref: "body-ref:workflow-run-started".to_owned(),
            idempotency_key: "idem:event-bus-adapter:publish:1".to_owned(),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            asyncapi_channel_ref: Some(format!(
                "{}#/channels/workflow_runs_events_v1",
                WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF
            )),
            cloudevents_specversion: WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION.to_owned(),
            evidence_refs: vec!["evidence:event-bus-pg:publish".to_owned()],
        }
    }

    fn delivery_envelope(delivery_status: &str) -> WorkflowEventBusAdapterDeliveryEnvelope {
        let event_kind = if delivery_status == "delivery-denied" {
            WorkflowEventBusEventKind::WorkflowRunStarted
        } else {
            WorkflowEventBusEventKind::WorkflowStateTransitioned
        };
        WorkflowEventBusAdapterDeliveryEnvelope {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            channel_address: event_kind.channel().address().to_owned(),
            event_id: "event:workflow-state:001".to_owned(),
            event_type: event_kind.event_type().to_owned(),
            consumer_ref: "consumer:workflow-state-machine".to_owned(),
            offset_ref: "offset:partition-0:42".to_owned(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            idempotency_key: "idem:event-bus-adapter:delivery:1".to_owned(),
            replay_cursor_ref: Some("cursor:event-bus-pg:state".to_owned()),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            evidence_refs: vec!["evidence:event-bus-pg:delivery".to_owned()],
        }
    }

    fn publish_envelope_from_api(
        success: &WorkflowEventBusApiSuccessResponse,
    ) -> WorkflowEventBusAdapterPublishEnvelope {
        let mut envelope = publish_envelope();
        envelope.idempotency_key = success.metadata.idempotency_key.clone();
        envelope.trace_context_ref = success.metadata.trace_context_ref.clone();
        envelope.tenant_id = success.event.tenant_id.clone();
        envelope.cell_id = success.event.cell_id.clone();
        envelope.channel_address = success.event.channel_address.clone().unwrap();
        envelope.event_type = success.event.event_type.clone();
        envelope.asyncapi_channel_ref = success.event.asyncapi_channel_ref.clone();
        envelope
    }

    fn delivery_envelope_from_api(
        success: &WorkflowEventBusApiSuccessResponse,
    ) -> WorkflowEventBusAdapterDeliveryEnvelope {
        let mut envelope = delivery_envelope(&success.event.usecase_status);
        envelope.idempotency_key = success.metadata.idempotency_key.clone();
        envelope.trace_context_ref = success.metadata.trace_context_ref.clone();
        envelope.tenant_id = success.event.tenant_id.clone();
        envelope.cell_id = success.event.cell_id.clone();
        envelope.channel_address = success.event.channel_address.clone().unwrap();
        envelope.event_type = success.event.event_type.clone();
        envelope.consumer_ref = success.event.consumer_ref.clone().unwrap();
        envelope.offset_ref = success.event.offset_ref.clone().unwrap();
        envelope
    }

    fn publish_request(idempotency_key: &str) -> WorkflowEventBusApiPublishRequest {
        WorkflowEventBusApiPublishRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE.to_owned(),
            body: WorkflowEventBusApiPublishBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-api".to_owned(),
                event_kind: "workflow-run-started".to_owned(),
                producer_ref: "producer:workflow-engine:execution".to_owned(),
                event_id: "event:workflow-run-started:001".to_owned(),
                source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
                subject_ref: Some("subject:workflow-run:001".to_owned()),
                time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
                dataschema_ref: Some("schema:workflow-event-run-started".to_owned()),
                partition_key_ref: "partition:tenant-workflow-run".to_owned(),
                publish_idempotency_key: "idem:event-bus-domain:publish:001".to_owned(),
                causation_ref: "cause:execution-engine:start-run".to_owned(),
                correlation_ref: "corr:workflow-run:001".to_owned(),
                payload_ref: "body-ref:workflow-run-started".to_owned(),
                evidence_refs: vec!["evidence:event-bus-api:publish".to_owned()],
            },
        }
    }

    fn delivery_request(idempotency_key: &str) -> WorkflowEventBusApiDeliveryRequest {
        WorkflowEventBusApiDeliveryRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE.to_owned(),
            body: WorkflowEventBusApiDeliveryBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-api".to_owned(),
                subscription_channel: "workflow-state".to_owned(),
                consumer_ref: "consumer:workflow-state-machine".to_owned(),
                subscription_event_types: vec![
                    WorkflowEventBusEventKind::WorkflowStateTransitioned
                        .event_type()
                        .to_owned(),
                ],
                replay_cursor_ref: Some("cursor:event-bus-api:state".to_owned()),
                max_batch_size: 100,
                subscription_authorization_evidence_ref: "authz:event-bus-api:consume".to_owned(),
                candidate_channel: "workflow-state".to_owned(),
                candidate_event_id: "event:workflow-state:001".to_owned(),
                candidate_event_type: WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                candidate_idempotency_key: "idem:event-bus-domain:delivery:001".to_owned(),
                candidate_payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
                candidate_offset_ref: "offset:partition-0:42".to_owned(),
                candidate_evidence_refs: vec!["evidence:event-bus-api:delivery".to_owned()],
            },
        }
    }

    fn boundary(idempotency_key: &str) -> WorkflowEventBusApiBoundaryContext {
        WorkflowEventBusApiBoundaryContext {
            request_id: format!("request:event-bus-pg:{idempotency_key}"),
            tenant_id: "ten_workflow_event_bus".to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
            oyatie_version: WORKFLOW_EVENT_BUS_API_DECLARED_VERSION.to_owned(),
        }
    }

    fn principal() -> WorkflowEventBusApiPrincipal {
        WorkflowEventBusApiPrincipal {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
        }
    }

    fn authorization() -> WorkflowEventBusApiAuthorization {
        WorkflowEventBusApiAuthorization {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
            decision_id: "policy-decision:event-bus-allow".to_owned(),
            evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            allowed_surfaces: vec![WORKFLOW_EVENT_BUS_API_SURFACE.to_owned()],
            allowed_channels: vec![
                "workflow-runs".to_owned(),
                "workflow-state".to_owned(),
                "trigger-events".to_owned(),
                "intelligence-requests".to_owned(),
                "ontology-projections".to_owned(),
            ],
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowRunStarted
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::TriggerEvaluated
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::IntelligenceDraftRequested
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::OntologyProjectionUpdated
                    .event_type()
                    .to_owned(),
            ],
        }
    }
}
