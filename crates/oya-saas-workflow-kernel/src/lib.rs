//! SaaS workflow kernel — final-shape identity and contract primitives.
//!
//! Owns the workflow definition / run / event identity types shared across the
//! workflow domain, workflow application, plugin runtime, and bench harness.
//! Per ADR-0015 (flat-crates) this kernel takes no cross-context dependencies;
//! every downstream layer flows inward from kernel -> domain -> app.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod priority;
pub use priority::WorkflowPriority;

pub mod trigger;
pub use trigger::{StepStatus, TriggerKind};

use std::fmt;

const WORKFLOW_DEFINITION_ID_PREFIX: &str = "wfd_";
const WORKFLOW_RUN_ID_PREFIX: &str = "wfr_";
const WORKFLOW_STEP_ID_PREFIX: &str = "wfs_";
const WORKFLOW_EVENT_ID_PREFIX: &str = "wfe_";
const WORKFLOW_DEFINITION_SCHEMA_VERSION: u32 = 1;
const WORKFLOW_RUN_SCHEMA_VERSION: u32 = 1;
const WORKFLOW_EVENT_SCHEMA_VERSION: u32 = 1;

/// Errors the kernel surfaces while validating workflow identity primitives.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowKernelError {
    InvalidDefinitionId,
    InvalidRunId,
    InvalidStepId,
    InvalidEventId,
    InvalidTenantId,
    InvalidRegionalPack,
    EmptySteps,
    DuplicateStepId,
    InvalidScheduleAt,
    InvalidStepOrder,
    UnknownStep,
}

/// Globally unique workflow definition identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkflowDefinitionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl WorkflowDefinitionId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkflowKernelError> {
        prefixed(
            value.into(),
            WORKFLOW_DEFINITION_ID_PREFIX,
            WorkflowKernelError::InvalidDefinitionId,
        )
        .map(|value| Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for WorkflowDefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// Globally unique workflow run identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkflowRunId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl WorkflowRunId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkflowKernelError> {
        prefixed(
            value.into(),
            WORKFLOW_RUN_ID_PREFIX,
            WorkflowKernelError::InvalidRunId,
        )
        .map(|value| Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Identifier for a workflow step within a definition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkflowStepId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl WorkflowStepId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkflowKernelError> {
        prefixed(
            value.into(),
            WORKFLOW_STEP_ID_PREFIX,
            WorkflowKernelError::InvalidStepId,
        )
        .map(|value| Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Identifier for an emitted workflow audit event.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkflowEventId {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl WorkflowEventId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkflowKernelError> {
        prefixed(
            value.into(),
            WORKFLOW_EVENT_ID_PREFIX,
            WorkflowKernelError::InvalidEventId,
        )
        .map(|value| Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// Workflow step kinds supported by the preview kernel.
///
/// Final-shape kernel exposes the minimal closed-set per ADR-0023 (plugin
/// sandbox); plugin steps are dispatched through the runtime crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorkflowStepKind {
    Plugin,
    HumanReview,
    Webhook,
    Schedule,
}

/// Lifecycle phase of a workflow run.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorkflowRunState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Workflow event kinds emitted per step.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorkflowEventKind {
    DefinitionPublished,
    RunStarted,
    StepStarted,
    StepCompleted,
    StepFailed,
    RunCompleted,
    RunFailed,
    RunCancelled,
}

/// A single step in a workflow definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStep {
    pub id: WorkflowStepId,      // data_class: INTERNAL_ONLY
    pub kind: WorkflowStepKind,  // data_class: INTERNAL_ONLY
    pub order: u32,              // data_class: INTERNAL_ONLY
    pub plugin_manifest: String, // data_class: INTERNAL_ONLY
}

/// Workflow definition contract — versioned, tenant-bound, regional-pack-bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowDefinition {
    pub id: WorkflowDefinitionId,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub regional_pack: String,           // data_class: INTERNAL_ONLY
    pub steps: Vec<WorkflowStep>,        // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,             // data_class: INTERNAL_ONLY
}

/// Workflow run contract — instantiated from a definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRun {
    pub id: WorkflowRunId,                   // data_class: INTERNAL_ONLY
    pub definition_id: WorkflowDefinitionId, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub regional_pack: String,               // data_class: INTERNAL_ONLY
    pub state: WorkflowRunState,             // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub schema_version: u32,                 // data_class: INTERNAL_ONLY
}

/// Per-step audit event emitted by the workflow engine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEvent {
    pub id: WorkflowEventId,             // data_class: INTERNAL_ONLY
    pub run_id: WorkflowRunId,           // data_class: INTERNAL_ONLY
    pub step_id: Option<WorkflowStepId>, // data_class: INTERNAL_ONLY
    pub kind: WorkflowEventKind,         // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub schema_version: u32,             // data_class: INTERNAL_ONLY
}

impl WorkflowStep {
    pub fn new(
        id: WorkflowStepId,
        kind: WorkflowStepKind,
        order: u32,
        plugin_manifest: impl Into<String>,
    ) -> Self {
        Self {
            id,
            kind,
            order,
            plugin_manifest: plugin_manifest.into(),
        }
    }
}

impl WorkflowDefinition {
    /// Construct + validate a workflow definition.
    pub fn new(
        id: WorkflowDefinitionId,
        tenant_id: impl Into<String>,
        regional_pack: impl Into<String>,
        steps: Vec<WorkflowStep>,
        published_at_epoch_seconds: u64,
    ) -> Result<Self, WorkflowKernelError> {
        let tenant_id = tenant_id.into();
        if !is_tenant(&tenant_id) {
            return Err(WorkflowKernelError::InvalidTenantId);
        }
        let regional_pack = regional_pack.into();
        if !is_regional_pack(&regional_pack) {
            return Err(WorkflowKernelError::InvalidRegionalPack);
        }
        if steps.is_empty() {
            return Err(WorkflowKernelError::EmptySteps);
        }
        let mut last_order: Option<u32> = None;
        let mut seen: Vec<&str> = Vec::with_capacity(steps.len());
        for step in &steps {
            if seen.iter().any(|existing| *existing == step.id.as_str()) {
                return Err(WorkflowKernelError::DuplicateStepId);
            }
            seen.push(step.id.as_str());
            match last_order {
                None => last_order = Some(step.order),
                Some(prev) if step.order > prev => last_order = Some(step.order),
                _ => return Err(WorkflowKernelError::InvalidStepOrder),
            }
        }
        Ok(Self {
            id,
            tenant_id,
            regional_pack,
            steps,
            published_at_epoch_seconds,
            schema_version: WORKFLOW_DEFINITION_SCHEMA_VERSION,
        })
    }

    pub fn step(&self, id: &WorkflowStepId) -> Option<&WorkflowStep> {
        self.steps.iter().find(|step| step.id == *id)
    }
}

impl WorkflowRun {
    /// Instantiate a run from a published definition.
    pub fn start(
        run_id: WorkflowRunId,
        definition: &WorkflowDefinition,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, WorkflowKernelError> {
        if started_at_epoch_seconds < definition.published_at_epoch_seconds {
            return Err(WorkflowKernelError::InvalidScheduleAt);
        }
        Ok(Self {
            id: run_id,
            definition_id: definition.id.clone(),
            tenant_id: definition.tenant_id.clone(),
            regional_pack: definition.regional_pack.clone(),
            state: WorkflowRunState::Running,
            started_at_epoch_seconds,
            schema_version: WORKFLOW_RUN_SCHEMA_VERSION,
        })
    }

    pub fn transition(&mut self, next: WorkflowRunState) -> Result<(), WorkflowKernelError> {
        let allowed = matches!(
            (self.state, next),
            (WorkflowRunState::Pending, WorkflowRunState::Running)
                | (WorkflowRunState::Running, WorkflowRunState::Succeeded)
                | (WorkflowRunState::Running, WorkflowRunState::Failed)
                | (WorkflowRunState::Running, WorkflowRunState::Cancelled)
        );
        if !allowed {
            return Err(WorkflowKernelError::InvalidStepOrder);
        }
        self.state = next;
        Ok(())
    }
}

impl WorkflowEvent {
    pub fn new(
        id: WorkflowEventId,
        run_id: WorkflowRunId,
        step_id: Option<WorkflowStepId>,
        kind: WorkflowEventKind,
        occurred_at_epoch_seconds: u64,
    ) -> Self {
        Self {
            id,
            run_id,
            step_id,
            kind,
            occurred_at_epoch_seconds,
            schema_version: WORKFLOW_EVENT_SCHEMA_VERSION,
        }
    }
}

fn prefixed(
    value: String,
    prefix: &str,
    error: WorkflowKernelError,
) -> Result<String, WorkflowKernelError> {
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

fn is_tenant(value: &str) -> bool {
    value.starts_with("ten_") && value.len() > "ten_".len()
}

fn is_regional_pack(value: &str) -> bool {
    value.starts_with("oya-pack-") && value.len() > "oya-pack-".len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(id: &str, order: u32) -> WorkflowStep {
        WorkflowStep::new(
            WorkflowStepId::new(id).expect("step id"),
            WorkflowStepKind::Plugin,
            order,
            "oya:plugin:summarizer@1",
        )
    }

    #[test]
    fn definition_validates_ids_pack_and_step_order() {
        let def = WorkflowDefinition::new(
            WorkflowDefinitionId::new("wfd_summarize_v1").expect("definition id"),
            "ten_acme",
            "oya-pack-kr",
            vec![step("wfs_extract", 1), step("wfs_summarize", 2)],
            1_700_000_000,
        )
        .expect("valid definition");

        assert_eq!(def.steps.len(), 2);
        assert_eq!(def.schema_version, WORKFLOW_DEFINITION_SCHEMA_VERSION);
        assert!(
            def.step(&WorkflowStepId::new("wfs_extract").unwrap())
                .is_some()
        );
    }

    #[test]
    fn definition_rejects_empty_duplicate_and_unordered_steps() {
        let empty = WorkflowDefinition::new(
            WorkflowDefinitionId::new("wfd_empty").unwrap(),
            "ten_acme",
            "oya-pack-kr",
            vec![],
            1_700_000_000,
        )
        .expect_err("empty steps rejected");
        assert_eq!(empty, WorkflowKernelError::EmptySteps);

        let dup = WorkflowDefinition::new(
            WorkflowDefinitionId::new("wfd_dup").unwrap(),
            "ten_acme",
            "oya-pack-kr",
            vec![step("wfs_a", 1), step("wfs_a", 2)],
            1_700_000_000,
        )
        .expect_err("duplicate step rejected");
        assert_eq!(dup, WorkflowKernelError::DuplicateStepId);

        let unordered = WorkflowDefinition::new(
            WorkflowDefinitionId::new("wfd_unord").unwrap(),
            "ten_acme",
            "oya-pack-kr",
            vec![step("wfs_b", 2), step("wfs_a", 1)],
            1_700_000_000,
        )
        .expect_err("step order strictly increasing");
        assert_eq!(unordered, WorkflowKernelError::InvalidStepOrder);
    }

    #[test]
    fn run_starts_and_transitions_through_lifecycle() {
        let def = WorkflowDefinition::new(
            WorkflowDefinitionId::new("wfd_run").unwrap(),
            "ten_acme",
            "oya-pack-kr",
            vec![step("wfs_a", 1)],
            1_700_000_000,
        )
        .unwrap();
        let mut run =
            WorkflowRun::start(WorkflowRunId::new("wfr_001").unwrap(), &def, 1_700_000_100)
                .expect("run starts after publish");
        assert_eq!(run.state, WorkflowRunState::Running);
        run.transition(WorkflowRunState::Succeeded).unwrap();
        assert_eq!(run.state, WorkflowRunState::Succeeded);

        let backdate = WorkflowRun::start(
            WorkflowRunId::new("wfr_002").unwrap(),
            &def,
            1_700_000_000 - 1,
        )
        .expect_err("run cannot precede publish");
        assert_eq!(backdate, WorkflowKernelError::InvalidScheduleAt);
    }

    #[test]
    fn id_constructors_enforce_prefixes() {
        assert!(WorkflowDefinitionId::new("nope").is_err());
        assert!(WorkflowRunId::new("nope").is_err());
        assert!(WorkflowStepId::new("nope").is_err());
        assert!(WorkflowEventId::new("nope").is_err());
        assert_eq!(
            WorkflowDefinitionId::new("wfd_x").unwrap().as_str(),
            "wfd_x"
        );
    }

    #[test]
    fn event_records_kind_and_step_pointer() {
        let event = WorkflowEvent::new(
            WorkflowEventId::new("wfe_001").unwrap(),
            WorkflowRunId::new("wfr_001").unwrap(),
            Some(WorkflowStepId::new("wfs_a").unwrap()),
            WorkflowEventKind::StepCompleted,
            1_700_000_200,
        );
        assert_eq!(event.kind, WorkflowEventKind::StepCompleted);
        assert_eq!(event.schema_version, WORKFLOW_EVENT_SCHEMA_VERSION);
        assert!(event.step_id.is_some());
    }
}
