# P07 — Workflow Studio Visual Editor: Implementation Plan

## Metadata
- phase: P07-workflow-studio-editor
- milestone: M03-first-tenant
- depends_on: [M02-workflow-engine]
- parallel_with: [P01-hr, P02-payroll, P03-accounting, P04-connect-pro-mail, P05-connect-pro-messenger]
- wave: 2 (parallel; only needs M02 engine substrate, not M03 product µservices)
- grit_claim_symbols: [m03.p07.workflow.studio, m03.p07.workflow.engine, m03.p07.workflow.templates, m03.p07.workflow.agentic, m03.p07.workflow.leptos-canvas]
- icm_topics: [context-oyatie, decisions-oyatie, errors-resolved]
- icm_keywords: [workflow,studio,editor,leptos,canvas,durable-execution,temporal,agentic,llm,n8n]

---

## 0. Crate Inventory

```
crates/
  oya-workflow-studio-kernel/        # port traits: WorkflowStateStore, TransitionEngine, EventBus,
                                     #   ApprovalStore, SlaTimerStore, TemplateRegistry, AgentNodeRunner
  oya-workflow-studio-domain/        # DefinitionAggregate, RunAggregate, ApprovalAggregate, SlaTimer
  oya-workflow-studio-application/   # use-cases: CreateDefinition, TriggerRun, AdvanceTransition,
                                     #   SubmitApproval, EscalateSlaBreach, RegisterTemplate
  oya-workflow-studio-adapter/       # PostgresWorkflowStateStore, KafkaEventBus, TemporalRunAdapter,
                                     #   LlmAgentNodeAdapter, ValkeySlaTimerStore
  oya-workflow-studio-rest/          # Axum REST: /api/workflow/*
  oya-workflow-studio-grpc/          # tonic gRPC: WorkflowEngineService
  oya-workflow-studio-worker/        # Kafka consumers: inbound domain events → trigger workflows
  oya-workflow-studio-app/           # composition root
  oya-workflow-leptos-canvas/        # Leptos WASM canvas SDK — drag-drop node editor
  oya-workflow-template-fixtures/    # 10 M03 domain template YAML/JSON fixtures (lib crate, no DB)
```

---

## 1. Full DDL

```sql
-- migrations/20260513_000001_workflow_studio.sql

CREATE SCHEMA IF NOT EXISTS workflow;

-- ── Enums ─────────────────────────────────────────────────────────────
CREATE TYPE workflow.definition_status AS ENUM (
  'draft',
  'published',
  'archived'
);

CREATE TYPE workflow.run_status AS ENUM (
  'pending',
  'running',
  'waiting_approval',
  'waiting_timer',
  'completed',
  'failed',
  'cancelled'
);

CREATE TYPE workflow.node_kind AS ENUM (
  'trigger',
  'action',
  'condition',
  'approval',
  'timer',
  'agentic',
  'integration',
  'sub_workflow'
);

CREATE TYPE workflow.approval_status AS ENUM (
  'pending',
  'approved',
  'rejected',
  'escalated',
  'expired'
);

CREATE TYPE workflow.trigger_kind AS ENUM (
  'event',        -- Kafka event consumed
  'schedule',     -- cron
  'webhook',      -- inbound HTTP
  'manual'        -- user-initiated from Studio UI
);

-- ── workflow_definitions ──────────────────────────────────────────────
CREATE TABLE workflow.definitions (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  name            TEXT NOT NULL,
  description     TEXT,
  version         INTEGER NOT NULL DEFAULT 1,
  status          workflow.definition_status NOT NULL DEFAULT 'draft',
  graph           JSONB NOT NULL DEFAULT '{"nodes":[],"edges":[]}',
  -- graph: { nodes: [{id, kind, config}], edges: [{from, to, condition}] }
  template_id     UUID,               -- null if custom, set if instantiated from template
  created_by      UUID NOT NULL,
  published_at    TIMESTAMPTZ,
  archived_at     TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, name, version)
);

SELECT create_distributed_table('workflow.definitions', 'tenant_id');
ALTER TABLE workflow.definitions ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.definitions
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_definitions_status ON workflow.definitions (tenant_id, status)
  WHERE status = 'published';
CREATE INDEX idx_definitions_template ON workflow.definitions (template_id)
  WHERE template_id IS NOT NULL;

-- ── workflow_runs ─────────────────────────────────────────────────────
CREATE TABLE workflow.runs (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id           UUID NOT NULL,
  definition_id       UUID NOT NULL,
  definition_version  INTEGER NOT NULL,
  status              workflow.run_status NOT NULL DEFAULT 'pending',
  trigger_kind        workflow.trigger_kind NOT NULL,
  trigger_payload     JSONB NOT NULL DEFAULT '{}',
  current_node_id     TEXT,           -- node id within graph
  context             JSONB NOT NULL DEFAULT '{}',
  -- durable execution: journal of completed steps for deterministic replay
  step_journal        JSONB NOT NULL DEFAULT '[]',
  started_at          TIMESTAMPTZ,
  completed_at        TIMESTAMPTZ,
  failed_at           TIMESTAMPTZ,
  failure_reason      TEXT,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('workflow.runs', 'tenant_id');
ALTER TABLE workflow.runs ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.runs
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_runs_definition ON workflow.runs (tenant_id, definition_id, status);
CREATE INDEX idx_runs_active ON workflow.runs (tenant_id, status, created_at)
  WHERE status IN ('running', 'waiting_approval', 'waiting_timer');

-- ── workflow_transitions ──────────────────────────────────────────────
-- Append-only state transition log (durable execution journal)
CREATE TABLE workflow.transitions (
  id              BIGSERIAL,
  tenant_id       UUID NOT NULL,
  run_id          UUID NOT NULL,
  from_node_id    TEXT,
  to_node_id      TEXT NOT NULL,
  node_kind       workflow.node_kind NOT NULL,
  input           JSONB NOT NULL DEFAULT '{}',
  output          JSONB NOT NULL DEFAULT '{}',
  error           TEXT,
  event_hash      BYTEA NOT NULL,   -- Ed25519 sealed per ADR-0028
  transitioned_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('workflow.transitions', 'tenant_id');
ALTER TABLE workflow.transitions ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.transitions
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Append-only: no updates or deletes
CREATE RULE no_update_transitions AS ON UPDATE TO workflow.transitions DO INSTEAD NOTHING;
CREATE RULE no_delete_transitions AS ON DELETE TO workflow.transitions DO INSTEAD NOTHING;

CREATE INDEX idx_transitions_run ON workflow.transitions (tenant_id, run_id, transitioned_at);

-- ── workflow_approvals ────────────────────────────────────────────────
CREATE TABLE workflow.approvals (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  run_id          UUID NOT NULL,
  node_id         TEXT NOT NULL,
  status          workflow.approval_status NOT NULL DEFAULT 'pending',
  assignee_role   TEXT NOT NULL,        -- Cedar role expression
  assignee_user_id UUID,               -- resolved at runtime
  requested_by    UUID NOT NULL,
  response_by     UUID,
  response_note   TEXT,
  -- PQXDH approval chain: signature over (run_id, node_id, decision)
  approval_signature BYTEA,
  escalated_to    UUID,
  deadline_at     TIMESTAMPTZ NOT NULL,
  responded_at    TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('workflow.approvals', 'tenant_id');
ALTER TABLE workflow.approvals ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.approvals
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_approvals_run ON workflow.approvals (tenant_id, run_id, status);
CREATE INDEX idx_approvals_pending ON workflow.approvals (tenant_id, assignee_user_id, deadline_at)
  WHERE status = 'pending';

-- ── workflow_sla_timers ───────────────────────────────────────────────
CREATE TABLE workflow.sla_timers (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  run_id          UUID NOT NULL,
  node_id         TEXT NOT NULL,
  fire_at         TIMESTAMPTZ NOT NULL,
  fired_at        TIMESTAMPTZ,
  cancelled_at    TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('workflow.sla_timers', 'tenant_id');
ALTER TABLE workflow.sla_timers ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.sla_timers
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_sla_timers_pending ON workflow.sla_timers (fire_at)
  WHERE fired_at IS NULL AND cancelled_at IS NULL;

-- ── workflow_automations ──────────────────────────────────────────────
-- Saved automation rules: "when X happens, do Y" shortcuts
CREATE TABLE workflow.automations (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  name            TEXT NOT NULL,
  trigger_event   TEXT NOT NULL,    -- Kafka topic pattern
  filter_expr     JSONB,            -- JSONata/JMESPath filter
  definition_id   UUID NOT NULL,
  is_active       BOOLEAN NOT NULL DEFAULT true,
  created_by      UUID NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('workflow.automations', 'tenant_id');
ALTER TABLE workflow.automations ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.automations
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_automations_trigger ON workflow.automations (trigger_event)
  WHERE is_active = true;

-- ── workflow_integrations ─────────────────────────────────────────────
CREATE TABLE workflow.integrations (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  integration_key TEXT NOT NULL,    -- e.g. "slack", "jira", "webhook"
  display_name    TEXT NOT NULL,
  credential_ref  TEXT NOT NULL,    -- OpenBao path
  config          JSONB NOT NULL DEFAULT '{}',
  is_active       BOOLEAN NOT NULL DEFAULT true,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, integration_key, display_name)
);

SELECT create_distributed_table('workflow.integrations', 'tenant_id');
ALTER TABLE workflow.integrations ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.integrations
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- ── workflow_triggers (registered inbound event handlers) ─────────────
CREATE TABLE workflow.triggers (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  definition_id   UUID NOT NULL,
  trigger_kind    workflow.trigger_kind NOT NULL,
  -- event trigger
  kafka_topic     TEXT,
  event_filter    JSONB,
  -- schedule trigger
  cron_expr       TEXT,
  -- webhook trigger
  webhook_path    TEXT UNIQUE,
  webhook_secret_ref TEXT,          -- OpenBao path
  is_active       BOOLEAN NOT NULL DEFAULT true,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('workflow.triggers', 'tenant_id');
ALTER TABLE workflow.triggers ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON workflow.triggers
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- ── workflow_outbox ───────────────────────────────────────────────────
CREATE TABLE workflow.workflow_outbox (
  id              BIGSERIAL PRIMARY KEY,
  tenant_id       UUID NOT NULL,
  aggregate_type  TEXT NOT NULL,
  aggregate_id    UUID NOT NULL,
  event_type      TEXT NOT NULL,
  payload         JSONB NOT NULL,
  kafka_topic     TEXT NOT NULL,
  published_at    TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('workflow.workflow_outbox', 'tenant_id');
CREATE INDEX idx_workflow_outbox_unpublished
  ON workflow.workflow_outbox (tenant_id, created_at)
  WHERE published_at IS NULL;
```

---

## 2. Kernel Port Traits

```rust
// crates/oya-workflow-studio-kernel/src/ports.rs

use uuid::Uuid;
use async_trait::async_trait;
use crate::sealed;

// ── WorkflowStateStore ────────────────────────────────────────────────

pub struct WorkflowDefinition {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub version: i32,
    pub status: DefinitionStatus,
    pub graph: serde_json::Value,
    pub template_id: Option<Uuid>,
    pub created_by: Uuid,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct WorkflowRun {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub definition_id: Uuid,
    pub definition_version: i32,
    pub status: RunStatus,
    pub trigger_kind: TriggerKind,
    pub trigger_payload: serde_json::Value,
    pub current_node_id: Option<String>,
    pub context: serde_json::Value,
    pub step_journal: Vec<StepJournalEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepJournalEntry {
    pub node_id: String,
    pub node_kind: NodeKind,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub transitioned_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait WorkflowStateStore: Send + Sync + sealed::Sealed {
    async fn save_definition(&self, def: &WorkflowDefinition)
        -> Result<(), StateStoreError>;
    async fn load_definition(&self, tenant_id: Uuid, id: Uuid)
        -> Result<Option<WorkflowDefinition>, StateStoreError>;
    async fn list_definitions(
        &self,
        tenant_id: Uuid,
        status: Option<DefinitionStatus>,
    ) -> Result<Vec<WorkflowDefinition>, StateStoreError>;
    async fn save_run(&self, run: &WorkflowRun) -> Result<(), StateStoreError>;
    async fn load_run(&self, tenant_id: Uuid, run_id: Uuid)
        -> Result<Option<WorkflowRun>, StateStoreError>;
    async fn list_active_runs(&self, tenant_id: Uuid)
        -> Result<Vec<WorkflowRun>, StateStoreError>;
}

// ── TransitionEngine ──────────────────────────────────────────────────
// Deterministic replay: given a step journal, re-derives current state
// without re-executing side effects (Temporal-parity durable execution).

pub struct TransitionInput {
    pub run_id: Uuid,
    pub tenant_id: Uuid,
    pub node_id: String,
    pub input: serde_json::Value,
}

pub struct TransitionOutput {
    pub next_node_id: Option<String>,
    pub output: serde_json::Value,
    pub new_status: RunStatus,
    pub event_hash: [u8; 64],  // Ed25519 signature
}

#[async_trait]
pub trait TransitionEngine: Send + Sync + sealed::Sealed {
    /// Execute a single node transition.  Side effects are journaled BEFORE
    /// execution so that replay skips already-completed nodes.
    async fn advance(
        &self,
        input: TransitionInput,
    ) -> Result<TransitionOutput, TransitionError>;

    /// Replay from step_journal to verify determinism (no side effects).
    async fn replay(
        &self,
        tenant_id: Uuid,
        run_id: Uuid,
    ) -> Result<WorkflowRun, TransitionError>;
}

// ── EventBus ──────────────────────────────────────────────────────────

pub struct OutboundEvent {
    pub topic: String,
    pub key: String,
    pub payload: Vec<u8>,
}

#[async_trait]
pub trait EventBus: Send + Sync + sealed::Sealed {
    async fn publish(&self, event: OutboundEvent) -> Result<(), EventBusError>;
    async fn subscribe(
        &self,
        topics: &[String],
        group_id: &str,
    ) -> Result<Box<dyn EventStream>, EventBusError>;
}

#[async_trait]
pub trait EventStream: Send {
    async fn next(&mut self) -> Option<Result<InboundEvent, EventBusError>>;
    async fn ack(&mut self, offset: i64) -> Result<(), EventBusError>;
}

pub struct InboundEvent {
    pub topic: String,
    pub key: String,
    pub payload: Vec<u8>,
    pub offset: i64,
}

// ── ApprovalStore ─────────────────────────────────────────────────────

pub struct ApprovalRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub run_id: Uuid,
    pub node_id: String,
    pub status: ApprovalStatus,
    pub assignee_role: String,
    pub assignee_user_id: Option<Uuid>,
    pub requested_by: Uuid,
    pub deadline_at: chrono::DateTime<chrono::Utc>,
    pub approval_signature: Option<Vec<u8>>,
}

#[async_trait]
pub trait ApprovalStore: Send + Sync + sealed::Sealed {
    async fn create(&self, record: &ApprovalRecord) -> Result<(), ApprovalStoreError>;
    async fn load(&self, tenant_id: Uuid, id: Uuid)
        -> Result<Option<ApprovalRecord>, ApprovalStoreError>;
    async fn pending_for_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ApprovalRecord>, ApprovalStoreError>;
    async fn respond(
        &self,
        id: Uuid,
        status: ApprovalStatus,
        response_by: Uuid,
        signature: Vec<u8>,
        note: Option<String>,
    ) -> Result<(), ApprovalStoreError>;
    async fn escalate(
        &self,
        id: Uuid,
        escalated_to: Uuid,
    ) -> Result<(), ApprovalStoreError>;
}

// ── SlaTimerStore ─────────────────────────────────────────────────────

pub struct SlaTimer {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub run_id: Uuid,
    pub node_id: String,
    pub fire_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait SlaTimerStore: Send + Sync + sealed::Sealed {
    async fn schedule(&self, timer: &SlaTimer) -> Result<(), SlaTimerError>;
    async fn cancel(&self, id: Uuid) -> Result<(), SlaTimerError>;
    async fn due_timers(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        limit: u32,
    ) -> Result<Vec<SlaTimer>, SlaTimerError>;
    async fn mark_fired(&self, id: Uuid) -> Result<(), SlaTimerError>;
}

// ── TemplateRegistry ──────────────────────────────────────────────────

pub struct WorkflowTemplate {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub graph: serde_json::Value,
    pub domain: String,   // "hr" | "payroll" | "accounting" | "connect" | "cross-domain"
}

#[async_trait]
pub trait TemplateRegistry: Send + Sync + sealed::Sealed {
    async fn list_templates(
        &self,
        domain: Option<&str>,
    ) -> Result<Vec<WorkflowTemplate>, TemplateError>;
    async fn find_by_slug(&self, slug: &str)
        -> Result<Option<WorkflowTemplate>, TemplateError>;
}

// ── AgentNodeRunner (ADR-0107 agentic LLM node) ───────────────────────

pub struct AgentNodeInput {
    pub node_id: String,
    pub tenant_id: Uuid,
    pub run_id: Uuid,
    pub prompt_template: String,
    pub context: serde_json::Value,
    pub output_schema: serde_json::Value,  // JSON Schema for validated output
}

pub struct AgentNodeOutput {
    pub structured_output: serde_json::Value,
    pub token_usage: u32,
    pub model_id: String,
}

#[async_trait]
pub trait AgentNodeRunner: Send + Sync + sealed::Sealed {
    /// Invoke LLM with prompt template + context; validates output against JSON Schema.
    /// Tenant data NEVER leaves the tenant's data residency region.
    async fn run(
        &self,
        input: AgentNodeInput,
    ) -> Result<AgentNodeOutput, AgentNodeError>;
}

// ── Errors ────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum StateStoreError {
    #[error("database error: {0}")]
    Database(String),
    #[error("run not found: {0}")]
    RunNotFound(Uuid),
    #[error("definition version conflict")]
    VersionConflict,
}

#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("node not found in graph: {0}")]
    NodeNotFound(String),
    #[error("guard condition not satisfied: {0}")]
    GuardFailed(String),
    #[error("side effect failed: {0}")]
    SideEffectFailed(String),
    #[error("replay mismatch at node {node}: expected {expected:?}, got {actual:?}")]
    ReplayMismatch {
        node: String,
        expected: serde_json::Value,
        actual: serde_json::Value,
    },
    #[error("state store: {0}")]
    StateStore(#[from] StateStoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("kafka error: {0}")]
    Kafka(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ApprovalStoreError {
    #[error("database error: {0}")]
    Database(String),
    #[error("approval not found: {0}")]
    NotFound(Uuid),
    #[error("approval already responded")]
    AlreadyResponded,
}

#[derive(Debug, thiserror::Error)]
pub enum SlaTimerError {
    #[error("database error: {0}")]
    Database(String),
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("fixture load error: {0}")]
    Load(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AgentNodeError {
    #[error("LLM call failed: {0}")]
    LlmCall(String),
    #[error("output schema validation failed: {0}")]
    SchemaValidation(String),
    #[error("tenant data residency violation")]
    DataResidencyViolation,
}
```

---

## 3. Domain Aggregates

```rust
// crates/oya-workflow-studio-domain/src/run.rs

use uuid::Uuid;
use crate::kernel::{WorkflowRun, RunStatus, StepJournalEntry, TransitionError};

pub struct RunAggregate {
    run: WorkflowRun,
    pending_events: Vec<RunDomainEvent>,
}

#[derive(Debug)]
pub enum RunDomainEvent {
    RunStarted { run_id: Uuid, definition_id: Uuid, tenant_id: Uuid },
    StepCompleted { run_id: Uuid, node_id: String, output: serde_json::Value },
    RunCompleted { run_id: Uuid },
    RunFailed { run_id: Uuid, reason: String },
    ApprovalRequested { run_id: Uuid, node_id: String, assignee_role: String },
}

impl RunAggregate {
    pub fn start(
        id: Uuid,
        tenant_id: Uuid,
        definition_id: Uuid,
        definition_version: i32,
        trigger_kind: crate::kernel::TriggerKind,
        trigger_payload: serde_json::Value,
        first_node_id: String,
    ) -> Self {
        let run = WorkflowRun {
            id,
            tenant_id,
            definition_id,
            definition_version,
            status: RunStatus::Running,
            trigger_kind,
            trigger_payload,
            current_node_id: Some(first_node_id.clone()),
            context: serde_json::Value::Object(Default::default()),
            step_journal: vec![],
        };
        let mut agg = Self { run, pending_events: vec![] };
        agg.pending_events.push(RunDomainEvent::RunStarted {
            run_id: id,
            definition_id,
            tenant_id,
        });
        agg
    }

    /// Journal a completed step. Idempotent: if node already in journal, skip.
    pub fn record_step(
        &mut self,
        entry: StepJournalEntry,
    ) -> Result<(), TransitionError> {
        if self.run.step_journal.iter().any(|e| e.node_id == entry.node_id) {
            // Already journaled — replay path; do not re-emit event.
            return Ok(());
        }
        let node_id = entry.node_id.clone();
        let output = entry.output.clone();
        self.run.step_journal.push(entry);
        self.run.current_node_id = Some(node_id.clone());
        self.pending_events.push(RunDomainEvent::StepCompleted {
            run_id: self.run.id,
            node_id,
            output,
        });
        Ok(())
    }

    pub fn complete(&mut self) {
        self.run.status = RunStatus::Completed;
        self.pending_events.push(RunDomainEvent::RunCompleted {
            run_id: self.run.id,
        });
    }

    pub fn fail(&mut self, reason: String) {
        self.run.status = RunStatus::Failed;
        self.pending_events.push(RunDomainEvent::RunFailed {
            run_id: self.run.id,
            reason,
        });
    }

    pub fn request_approval(&mut self, node_id: String, assignee_role: String) {
        self.run.status = RunStatus::WaitingApproval;
        self.pending_events.push(RunDomainEvent::ApprovalRequested {
            run_id: self.run.id,
            node_id,
            assignee_role,
        });
    }

    pub fn run(&self) -> &WorkflowRun { &self.run }
    pub fn take_events(&mut self) -> Vec<RunDomainEvent> {
        std::mem::take(&mut self.pending_events)
    }
}
```

---

## 4. Application Use-Cases

```rust
// crates/oya-workflow-studio-application/src/use_cases.rs

use uuid::Uuid;
use crate::kernel::{
    WorkflowStateStore, TransitionEngine, EventBus, ApprovalStore,
    SlaTimerStore, TemplateRegistry, AgentNodeRunner,
    WorkflowDefinition, WorkflowRun, RunStatus, TriggerKind,
    OutboundEvent, SlaTimer, ApprovalRecord, ApprovalStatus,
};

// ── CreateDefinition ──────────────────────────────────────────────────

pub struct CreateDefinitionCommand {
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub graph: serde_json::Value,
    pub template_id: Option<Uuid>,
    pub created_by: Uuid,
}

pub struct CreateDefinitionUseCase<S> {
    state_store: S,
}

impl<S: WorkflowStateStore> CreateDefinitionUseCase<S> {
    pub fn new(state_store: S) -> Self { Self { state_store } }

    pub async fn execute(
        &self,
        cmd: CreateDefinitionCommand,
    ) -> Result<Uuid, CreateDefinitionError> {
        validate_graph(&cmd.graph)?;
        let def = WorkflowDefinition {
            id: Uuid::new_v4(),
            tenant_id: cmd.tenant_id,
            name: cmd.name,
            version: 1,
            status: crate::kernel::DefinitionStatus::Draft,
            graph: cmd.graph,
            template_id: cmd.template_id,
            created_by: cmd.created_by,
            published_at: None,
            description: cmd.description,
        };
        let id = def.id;
        self.state_store.save_definition(&def).await?;
        Ok(id)
    }
}

fn validate_graph(graph: &serde_json::Value) -> Result<(), CreateDefinitionError> {
    let nodes = graph.get("nodes").and_then(|n| n.as_array())
        .ok_or(CreateDefinitionError::InvalidGraph("missing nodes array".into()))?;
    if nodes.is_empty() {
        return Err(CreateDefinitionError::InvalidGraph("empty graph".into()));
    }
    // Exactly one trigger node required.
    let trigger_count = nodes.iter()
        .filter(|n| n.get("kind").and_then(|k| k.as_str()) == Some("trigger"))
        .count();
    if trigger_count != 1 {
        return Err(CreateDefinitionError::InvalidGraph(
            format!("expected 1 trigger node, found {trigger_count}")
        ));
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateDefinitionError {
    #[error("invalid graph: {0}")]
    InvalidGraph(String),
    #[error("state store: {0}")]
    StateStore(#[from] crate::kernel::StateStoreError),
}

// ── TriggerRun ────────────────────────────────────────────────────────

pub struct TriggerRunCommand {
    pub tenant_id: Uuid,
    pub definition_id: Uuid,
    pub trigger_kind: TriggerKind,
    pub trigger_payload: serde_json::Value,
}

pub struct TriggerRunUseCase<S, E, T> {
    state_store: S,
    engine: E,
    event_bus: T,
}

impl<S, E, T> TriggerRunUseCase<S, E, T>
where
    S: WorkflowStateStore,
    E: TransitionEngine,
    T: EventBus,
{
    pub fn new(state_store: S, engine: E, event_bus: T) -> Self {
        Self { state_store, engine, event_bus }
    }

    pub async fn execute(
        &self,
        cmd: TriggerRunCommand,
    ) -> Result<Uuid, TriggerRunError> {
        let def = self.state_store
            .load_definition(cmd.tenant_id, cmd.definition_id)
            .await?
            .ok_or(TriggerRunError::DefinitionNotFound(cmd.definition_id))?;

        if def.status != crate::kernel::DefinitionStatus::Published {
            return Err(TriggerRunError::DefinitionNotPublished);
        }

        let run_id = Uuid::new_v4();
        let run = WorkflowRun {
            id: run_id,
            tenant_id: cmd.tenant_id,
            definition_id: cmd.definition_id,
            definition_version: def.version,
            status: RunStatus::Pending,
            trigger_kind: cmd.trigger_kind,
            trigger_payload: cmd.trigger_payload,
            current_node_id: None,
            context: serde_json::Value::Object(Default::default()),
            step_journal: vec![],
        };
        self.state_store.save_run(&run).await?;

        self.event_bus.publish(OutboundEvent {
            topic: "oyatie.workflow.run-triggered.v1".into(),
            key: run_id.to_string(),
            payload: serde_json::to_vec(&serde_json::json!({
                "run_id": run_id,
                "tenant_id": cmd.tenant_id,
                "definition_id": cmd.definition_id,
            })).unwrap(),
        }).await?;

        Ok(run_id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TriggerRunError {
    #[error("definition not found: {0}")]
    DefinitionNotFound(Uuid),
    #[error("definition must be published before triggering")]
    DefinitionNotPublished,
    #[error("state store: {0}")]
    StateStore(#[from] crate::kernel::StateStoreError),
    #[error("event bus: {0}")]
    EventBus(#[from] crate::kernel::EventBusError),
}

// ── SubmitApproval ────────────────────────────────────────────────────

pub struct SubmitApprovalCommand {
    pub tenant_id: Uuid,
    pub approval_id: Uuid,
    pub responder_user_id: Uuid,
    pub approved: bool,
    pub note: Option<String>,
    /// Ed25519 signature over (approval_id || decision_byte || run_id)
    pub signature: Vec<u8>,
}

pub struct SubmitApprovalUseCase<A, S, E> {
    approval_store: A,
    state_store: S,
    event_bus: E,
}

impl<A, S, E> SubmitApprovalUseCase<A, S, E>
where
    A: ApprovalStore,
    S: WorkflowStateStore,
    E: EventBus,
{
    pub fn new(approval_store: A, state_store: S, event_bus: E) -> Self {
        Self { approval_store, state_store, event_bus }
    }

    pub async fn execute(
        &self,
        cmd: SubmitApprovalCommand,
    ) -> Result<(), SubmitApprovalError> {
        let approval = self.approval_store
            .load(cmd.tenant_id, cmd.approval_id)
            .await?
            .ok_or(SubmitApprovalError::NotFound(cmd.approval_id))?;

        if approval.status != ApprovalStatus::Pending {
            return Err(SubmitApprovalError::AlreadyResponded);
        }

        let status = if cmd.approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };

        self.approval_store.respond(
            cmd.approval_id,
            status,
            cmd.responder_user_id,
            cmd.signature,
            cmd.note,
        ).await?;

        // Resume workflow run if approved; fail it if rejected.
        self.event_bus.publish(OutboundEvent {
            topic: "oyatie.workflow.approval-responded.v1".into(),
            key: approval.run_id.to_string(),
            payload: serde_json::to_vec(&serde_json::json!({
                "approval_id": cmd.approval_id,
                "run_id": approval.run_id,
                "tenant_id": cmd.tenant_id,
                "approved": cmd.approved,
            })).unwrap(),
        }).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubmitApprovalError {
    #[error("approval not found: {0}")]
    NotFound(Uuid),
    #[error("approval already responded")]
    AlreadyResponded,
    #[error("approval store: {0}")]
    ApprovalStore(#[from] crate::kernel::ApprovalStoreError),
    #[error("event bus: {0}")]
    EventBus(#[from] crate::kernel::EventBusError),
}
```

---

## 5. Adapter Scaffolds

```rust
// crates/oya-workflow-studio-adapter/src/postgres_state_store.rs

use sqlx::PgPool;
use crate::kernel::{WorkflowStateStore, WorkflowRun, WorkflowDefinition, StateStoreError};

pub struct PostgresWorkflowStateStore { pool: PgPool }
impl PostgresWorkflowStateStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

// Implements WorkflowStateStore — full CRUD over workflow.definitions + workflow.runs.
// step_journal serialized as JSONB array of StepJournalEntry.

// crates/oya-workflow-studio-adapter/src/kafka_event_bus.rs

use rdkafka::producer::{FutureProducer, FutureRecord};
use crate::kernel::{EventBus, OutboundEvent, EventBusError, EventStream, InboundEvent};

pub struct KafkaEventBus { producer: FutureProducer }
impl KafkaEventBus {
    pub fn new(brokers: &str) -> Result<Self, EventBusError> {
        use rdkafka::config::ClientConfig;
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| EventBusError::Kafka(e.to_string()))?;
        Ok(Self { producer })
    }
}

// crates/oya-workflow-studio-adapter/src/valkey_sla_timer_store.rs
// SLA timers: sorted set in Valkey (score = fire_at unix timestamp).
// Background poller queries due timers every 1s; on fire, emits to Kafka.
// Persisted fallback: workflow.sla_timers (single source of truth).

pub struct ValkeySlaTimerStore {
    redis: redis::aio::ConnectionManager,
    pool: sqlx::PgPool,
}

// crates/oya-workflow-studio-adapter/src/llm_agent_node_adapter.rs
// Implements AgentNodeRunner using Anthropic Claude API.
// Validates structured output against JSON Schema (jsonschema crate).
// Tenant prompt context always injected with tenant_id; no cross-tenant bleed.

pub struct LlmAgentNodeAdapter {
    api_key_ref: String,   // OpenBao path; resolved at call time
    model_id: String,      // e.g. "claude-sonnet-4-5"
    http_client: reqwest::Client,
}

// crates/oya-workflow-studio-adapter/src/temporal_run_adapter.rs
// Wraps Temporal workflow SDK for long-running durable execution.
// Short-lived runs (< 5 steps, no timer) execute directly in-process via TransitionEngine.
// Long-running runs (timers, waits, sub-workflows) delegated to Temporal worker.

pub struct TemporalRunAdapter {
    temporal_client: temporal_sdk::WorkflowClient,
}
```

---

## 6. 10 M03 Domain Template Fixtures

```
crates/oya-workflow-template-fixtures/templates/
  01-hr-employee-onboarding.json
  02-hr-employee-offboarding.json
  03-payroll-run-approval.json
  04-payroll-year-end-settlement-review.json
  05-accounting-period-close-checklist.json
  06-connect-legal-hold-four-eyes-release.json
  07-connect-mail-data-export-approval.json
  08-cross-domain-new-hire-full-provisioning.json
  09-cross-domain-employee-termination-cascade.json
  10-cross-domain-payroll-variance-escalation.json
```

Sample fixture (01-hr-employee-onboarding.json):

```json
{
  "id": "00000000-0001-0001-0001-000000000001",
  "slug": "hr-employee-onboarding",
  "name": "HR: Employee Onboarding",
  "description": "Full onboarding flow from EmployeeHired event through product provisioning, equipment request, and buddy assignment.",
  "domain": "hr",
  "graph": {
    "nodes": [
      { "id": "trigger-1", "kind": "trigger", "config": {
          "trigger_kind": "event",
          "kafka_topic": "oyatie.hr.employee-hired.v1"
      }},
      { "id": "action-provision-user", "kind": "action", "config": {
          "action_type": "provision_tenant_user",
          "input_mapping": { "employee_id": "$.trigger.employee_id", "email": "$.trigger.work_email" }
      }},
      { "id": "action-send-welcome", "kind": "action", "config": {
          "action_type": "send_mail",
          "template": "welcome-new-hire",
          "to": "$.trigger.work_email"
      }},
      { "id": "approval-equipment", "kind": "approval", "config": {
          "assignee_role": "Role::HrAdmin",
          "deadline_hours": 48,
          "prompt": "Approve equipment request for new hire"
      }},
      { "id": "action-assign-buddy", "kind": "agentic", "config": {
          "prompt_template": "Select the most suitable buddy for {{employee_name}} in {{department}} based on current workload.",
          "output_schema": { "type": "object", "properties": { "buddy_employee_id": { "type": "string" } }, "required": ["buddy_employee_id"] }
      }},
      { "id": "end", "kind": "action", "config": {
          "action_type": "mark_onboarding_complete"
      }}
    ],
    "edges": [
      { "from": "trigger-1", "to": "action-provision-user" },
      { "from": "action-provision-user", "to": "action-send-welcome" },
      { "from": "action-send-welcome", "to": "approval-equipment" },
      { "from": "approval-equipment", "to": "action-assign-buddy", "condition": "approved" },
      { "from": "action-assign-buddy", "to": "end" }
    ]
  }
}
```

---

## 7. Cedar Policy Fragments

```cedar
// policies/workflow-studio.cedar

// WorkflowAdmin: full CRUD on definitions + runs for own tenant
permit (
  principal in Role::"WorkflowAdmin",
  action in [Action::"CreateDefinition", Action::"PublishDefinition",
             Action::"ArchiveDefinition", Action::"TriggerRun",
             Action::"CancelRun", Action::"ReadRun", Action::"ReadDefinition"],
  resource
)
when { principal.tenant_id == resource.tenant_id };

// WorkflowUser: can trigger runs and read own-triggered runs
permit (
  principal in Role::"WorkflowUser",
  action in [Action::"TriggerRun", Action::"ReadRun"],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  (action == Action::"TriggerRun" || resource.triggered_by == principal.user_id)
};

// Approval responder: only respond to approvals assigned to their role
permit (
  principal in Role::"WorkflowApprover",
  action == Action::"SubmitApproval",
  resource in Resource::"Approval"
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal has role &&
  resource.assignee_role == principal.role
};

// Auditor: read-only transitions log
permit (
  principal in Role::"Auditor",
  action == Action::"ReadTransitions",
  resource in Resource::"WorkflowRun"
)
when { principal.tenant_id == resource.tenant_id };

// Agentic node: only runs may invoke agentic actions; not users directly
permit (
  principal in Role::"WorkflowEngine",
  action == Action::"InvokeAgentNode",
  resource in Resource::"AgentNode"
)
when { principal.tenant_id == resource.tenant_id };

// Forbid cross-tenant
forbid (principal, action, resource)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};
```

---

## 8. Protobuf Event Schemas + Kafka Topics

```protobuf
// proto/workflow/v1/events.proto
syntax = "proto3";
package workflow.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

// Kafka topic: oyatie.workflow.run-triggered.v1
message WorkflowRunTriggered {
  string run_id          = 1;
  string tenant_id       = 2;
  string definition_id   = 3;
  string trigger_kind    = 4;
  google.protobuf.Struct trigger_payload = 5;
  google.protobuf.Timestamp triggered_at = 6;
  string event_id        = 7;
}

// Kafka topic: oyatie.workflow.run-completed.v1
message WorkflowRunCompleted {
  string run_id          = 1;
  string tenant_id       = 2;
  string definition_id   = 3;
  google.protobuf.Struct final_context = 4;
  google.protobuf.Timestamp completed_at = 5;
  string event_id        = 6;
}

// Kafka topic: oyatie.workflow.run-failed.v1
message WorkflowRunFailed {
  string run_id          = 1;
  string tenant_id       = 2;
  string failure_reason  = 3;
  string failed_node_id  = 4;
  google.protobuf.Timestamp failed_at = 5;
  string event_id        = 6;
}

// Kafka topic: oyatie.workflow.approval-requested.v1
message ApprovalRequested {
  string approval_id     = 1;
  string run_id          = 2;
  string tenant_id       = 3;
  string node_id         = 4;
  string assignee_role   = 5;
  google.protobuf.Timestamp deadline_at = 6;
  string event_id        = 7;
}

// Kafka topic: oyatie.workflow.approval-responded.v1
message ApprovalResponseSubmitted {
  string approval_id     = 1;
  string run_id          = 2;
  string tenant_id       = 3;
  bool   approved        = 4;
  string responder_id    = 5;
  bytes  approval_signature = 6;
  google.protobuf.Timestamp responded_at = 7;
  string event_id        = 8;
}

// Kafka topic: oyatie.workflow.sla-breached.v1
message SlaBreached {
  string timer_id        = 1;
  string run_id          = 2;
  string tenant_id       = 3;
  string node_id         = 4;
  google.protobuf.Timestamp breached_at = 5;
  string event_id        = 6;
}

// Kafka topic: oyatie.workflow.definition-published.v1
message WorkflowDefinitionPublished {
  string definition_id   = 1;
  string tenant_id       = 2;
  string name            = 3;
  int32  version         = 4;
  string published_by    = 5;
  google.protobuf.Timestamp published_at = 6;
  string event_id        = 7;
}
```

---

## 9. OpenAPI / gRPC Contracts

```yaml
# openapi/workflow-studio.yaml (condensed)
openapi: "3.1.0"
info:
  title: Workflow Studio API
  version: "1.0.0"
paths:
  /api/workflow/definitions:
    post:
      operationId: createDefinition
    get:
      operationId: listDefinitions

  /api/workflow/definitions/{definitionId}/publish:
    post:
      operationId: publishDefinition

  /api/workflow/definitions/{definitionId}/runs:
    post:
      operationId: triggerRun

  /api/workflow/runs/{runId}:
    get:
      operationId: getRun

  /api/workflow/runs/{runId}/transitions:
    get:
      operationId: listTransitions
      summary: Append-only transition log (audit chain)

  /api/workflow/approvals:
    get:
      operationId: listPendingApprovals

  /api/workflow/approvals/{approvalId}/respond:
    post:
      operationId: submitApproval

  /api/workflow/templates:
    get:
      operationId: listTemplates

  /api/workflow/templates/{slug}/instantiate:
    post:
      operationId: instantiateTemplate
```

```protobuf
// proto/workflow/v1/service.proto
service WorkflowEngineService {
  rpc TriggerRun(TriggerRunRequest) returns (TriggerRunResponse);
  rpc AdvanceRun(AdvanceRunRequest) returns (AdvanceRunResponse);
  rpc GetRunState(GetRunStateRequest) returns (GetRunStateResponse);
  rpc SubmitApproval(SubmitApprovalRequest) returns (SubmitApprovalResponse);
  rpc ListPendingApprovals(ListApprovalsRequest) returns (ListApprovalsResponse);
  rpc StreamRunEvents(StreamRunEventsRequest) returns (stream RunEvent);
}
```

---

## 10. Leptos Canvas SDK (oya-workflow-leptos-canvas)

```rust
// crates/oya-workflow-leptos-canvas/src/lib.rs
// WASM island compiled with trunk; loaded lazily in oya-application-leptos.

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasGraph {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
}

#[component]
pub fn WorkflowCanvas(
    #[prop(into)] initial_graph: Signal<CanvasGraph>,
    on_change: Callback<CanvasGraph>,
) -> impl IntoView {
    let (graph, set_graph) = create_signal(initial_graph.get_untracked());

    // Pointer-event handlers use wasm-bindgen to attach to canvas element.
    // Node drop performance target: p99 ≤ 16ms (1 rAF budget).
    // Implemented with direct DOM manipulation via web-sys; no re-render on drag.

    view! {
        <div class="workflow-canvas" id="workflow-canvas-root">
            <NodePalette />
            <CanvasSurface graph=graph set_graph=set_graph on_change=on_change />
            <PropertyPanel graph=graph />
        </div>
    }
}

/// NodePalette: lists all node kinds available for drag-drop.
#[component]
fn NodePalette() -> impl IntoView {
    view! {
        <aside class="node-palette">
            <PaletteItem kind="trigger" label="Trigger" />
            <PaletteItem kind="action" label="Action" />
            <PaletteItem kind="condition" label="Condition" />
            <PaletteItem kind="approval" label="Approval" />
            <PaletteItem kind="timer" label="Timer" />
            <PaletteItem kind="agentic" label="AI Agent" />
            <PaletteItem kind="integration" label="Integration" />
            <PaletteItem kind="sub_workflow" label="Sub-Workflow" />
        </aside>
    }
}
```

---

## 11. k6 Load Tests

```javascript
// tests/load/workflow-studio.k6.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import ws from 'k6/ws';

export const options = {
  scenarios: {
    concurrent_runs: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 5000 },
        { duration: '5m', target: 10000 },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    // PRD-workflow Performance Targets
    'http_req_duration{name:trigger_run}': ['p(99)<200'],      // ≤200ms @ 10k concurrent
    'http_req_duration{name:get_run_state}': ['p(99)<50'],     // ≤50ms read
    'http_req_duration{name:submit_approval}': ['p(99)<150'],  // ≤150ms
    'http_req_failed': ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'https://api.oyatie.local';
const TENANT_TOKEN = __ENV.TENANT_TOKEN || '';

export default function () {
  const headers = {
    'Authorization': `Bearer ${TENANT_TOKEN}`,
    'Content-Type': 'application/json',
    'X-Oyatie-Tenant-Id': __ENV.TENANT_ID,
  };

  // Trigger a workflow run
  const triggerRes = http.post(
    `${BASE_URL}/api/workflow/definitions/${__ENV.DEFINITION_ID}/runs`,
    JSON.stringify({ trigger_kind: 'manual', trigger_payload: {} }),
    { headers, tags: { name: 'trigger_run' } }
  );
  check(triggerRes, { 'trigger run 201': (r) => r.status === 201 });

  if (triggerRes.status === 201) {
    const runId = JSON.parse(triggerRes.body).run_id;
    sleep(0.05);

    const stateRes = http.get(
      `${BASE_URL}/api/workflow/runs/${runId}`,
      { headers, tags: { name: 'get_run_state' } }
    );
    check(stateRes, { 'get run state 200': (r) => r.status === 200 });
  }

  sleep(0.1);
}
```

```javascript
// tests/load/workflow-canvas-node-drop.playwright.js
// Playwright perf test: node-drop latency ≤ 16ms p99
const { test, expect } = require('@playwright/test');

test('node drop p99 ≤ 16ms', async ({ page }) => {
  await page.goto('/workflow/studio/new');
  const canvas = page.locator('#workflow-canvas-root');

  const latencies = [];
  for (let i = 0; i < 200; i++) {
    const start = Date.now();
    // Drag trigger node from palette to canvas
    await page.dragAndDrop('.palette-item[data-kind="action"]', '#workflow-canvas-root', {
      targetPosition: { x: 200 + i * 5, y: 200 },
    });
    latencies.push(Date.now() - start);
  }

  latencies.sort((a, b) => a - b);
  const p99 = latencies[Math.floor(latencies.length * 0.99)];
  expect(p99).toBeLessThanOrEqual(16);
});
```

---

## 12. Acceptance Gates

```
GATE WF-01: 10k concurrent workflow runs p99 trigger latency ≤ 200ms (k6)
GATE WF-02: Run state read p99 ≤ 50ms at 10k concurrent (k6)
GATE WF-03: Node-drop on Leptos canvas p99 ≤ 16ms (Playwright perf)
GATE WF-04: Approval round-trip (request → respond → run resume) < 500ms (integration test)
GATE WF-05: Deterministic replay: replay(run_id) produces identical step_journal as live run (property test with proptest)
GATE WF-06: All 10 M03 domain templates load and instantiate without error (fixture smoke test)
GATE WF-07: Agentic node output validated against JSON Schema; invalid output returns 422 (unit test)
GATE WF-08: Cross-tenant Cedar forbid: attempt to read other tenant's run returns 403
GATE WF-09: Transition log append-only: UPDATE/DELETE return 0 rows (DB invariant test)
GATE WF-10: Ed25519 seal on every transition record verifiable by audit-chain script (ADR-0028)
GATE WF-11: SLA timer fires within 2s of fire_at (timer accuracy test with mocked clock)
GATE WF-12: EmployeeHired → onboarding workflow run triggered within 5s of Kafka publish (E2E)
```

---

## 13. Grit Claim Symbols

```
grit session start m03-p07-workflow-studio-2026-05-13
grit claim m03.p07.workflow.studio
grit claim m03.p07.workflow.engine
grit claim m03.p07.workflow.templates
grit claim m03.p07.workflow.agentic
grit claim m03.p07.workflow.leptos-canvas
# ... implement ...
grit done --agent m03-p07-workflow-studio-2026-05-13
```

---

## 14. ICM Payload

```bash
icm store \
  -t context-oyatie \
  -c "M03-P07 Workflow Studio visual editor impl-plan complete: 8-BC DDL (definitions/runs/transitions/approvals/sla_timers/automations/triggers/integrations), all kernel port traits (WorkflowStateStore/TransitionEngine/EventBus/ApprovalStore/SlaTimerStore/TemplateRegistry/AgentNodeRunner), durable execution with deterministic replay, 10 M03 domain template JSON fixtures, PQXDH approval chain signatures, Valkey SLA timers, Temporal run adapter, LLM agentic nodes (ADR-0107), Leptos WASM canvas SDK, 10k-concurrent-runs p99 ≤200ms k6 + node-drop ≤16ms Playwright tests" \
  -i high \
  -k "workflow,studio,editor,leptos,canvas,durable-execution,temporal,agentic,llm,n8n,approval,sla"
```
