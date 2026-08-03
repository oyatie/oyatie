//! Workspace tasks kernel.
//!
//! Typed kernel records for the W-Workspace-GA Tasks surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns task-store
//! metadata, task graph validation, and the Foundry agent task-runtime binding
//! without owning workflow execution, storage adapters, or UI dispatch.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use intelligence_capability_domain::AutonomyTier;

const TASK_STORE_SCHEMA_VERSION: u32 = 1;
const TASK_SCHEMA_VERSION: u32 = 1;
const TASK_GRAPH_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskError {
    InvalidStoreId,
    InvalidTaskId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidOwnerRef,
    InvalidTitle,
    InvalidDescription,
    InvalidAssigneeRef,
    DuplicateAssigneeRef,
    InvalidCapabilityId,
    InvalidAgentRef,
    InvalidPolicyDecisionId,
    InvalidEvidenceTopic,
    InvalidAutoExecuteGrant,
    MissingAutoExecuteGrant,
    MissingCompletionTime,
    UnexpectedCompletionTime,
    EmptyTaskGraph,
    DuplicateTaskId,
    UnknownTaskParent,
    SelfParentTask,
    TaskParentCycle,
    MissingDependencyEndpoint,
    SelfDependency,
    DuplicateDependency,
    TaskDependencyCycle,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    Done,
    Canceled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaskDependencyKind {
    Blocks,
    MustFinishBefore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskStoreCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub owner_ref: String,                    // data_class: PII_IDENTIFYING
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskStore {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub owner_ref: Classified<String>,             // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCreate {
    pub id: String,                                       // data_class: INTERNAL_ONLY
    pub store_id: String,                                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                // data_class: INTERNAL_ONLY
    pub region: String,                                   // data_class: INTERNAL_ONLY
    pub cell_id: String,                                  // data_class: INTERNAL_ONLY
    pub title: String,                                    // data_class: PII_QUASI_IDENTIFIER
    pub description: Option<String>,                      // data_class: PII_IDENTIFYING
    pub assignee_refs: Vec<String>,                       // data_class: PII_IDENTIFYING
    pub status: TaskStatus,                               // data_class: INTERNAL_ONLY
    pub priority: TaskPriority,                           // data_class: INTERNAL_ONLY
    pub parent_task_id: Option<String>,                   // data_class: INTERNAL_ONLY
    pub due_at_epoch_seconds: Option<u64>,                // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Option<u64>,          // data_class: INTERNAL_ONLY
    pub foundry_binding: Option<FoundryAgentTaskBinding>, // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,             // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,                    // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceTask {
    pub id: Classified<String>,                  // data_class: INTERNAL_ONLY
    pub store_id: Classified<String>,            // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,           // data_class: INTERNAL_ONLY
    pub region: Classified<String>,              // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub title: Classified<String>,               // data_class: PII_QUASI_IDENTIFIER
    pub description: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub assignee_refs: Classified<Vec<String>>,  // data_class: PII_IDENTIFYING
    pub status: Classified<TaskStatus>,          // data_class: INTERNAL_ONLY
    pub priority: Classified<TaskPriority>,      // data_class: INTERNAL_ONLY
    pub parent_task_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub due_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub foundry_binding: Classified<Option<FoundryAgentTaskBinding>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryAgentTaskBinding {
    pub capability_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub agent_ref: Classified<String>,     // data_class: PII_IDENTIFYING
    pub requested_tier: Classified<AutonomyTier>, // data_class: INTERNAL_ONLY
    pub policy_decision_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub evidence_topic: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_auto_execute_grant_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaskDependency {
    pub before_task_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub after_task_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub kind: Classified<TaskDependencyKind>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskGraph {
    pub tasks: Vec<WorkspaceTask>,         // data_class: PII_IDENTIFYING
    pub dependencies: Vec<TaskDependency>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

pub trait TaskStoreReader {
    fn read_task(
        &self,
        tenant_id: &str,
        store_id: &str,
        task_id: &str,
    ) -> Result<Option<WorkspaceTask>, TaskError>;
}

impl TaskStore {
    pub fn new(input: TaskStoreCreate) -> Result<Self, TaskError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_task_data_class());
        validate_non_empty(&input.id, TaskError::InvalidStoreId)?;
        validate_non_empty(&input.tenant_id, TaskError::InvalidTenantId)?;
        validate_non_empty(&input.region, TaskError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, TaskError::InvalidCellId)?;
        validate_non_empty(&input.owner_ref, TaskError::InvalidOwnerRef)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            owner_ref: Classified::new(input.owner_ref, task_assignee_data_class()),
            data_class: internal(data_class),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(TASK_STORE_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl WorkspaceTask {
    pub fn new(input: TaskCreate, store: &TaskStore) -> Result<Self, TaskError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_task_data_class());
        validate_non_empty(&input.id, TaskError::InvalidTaskId)?;
        validate_store_binding(&input, store)?;
        validate_text(&input.title, TaskError::InvalidTitle)?;
        validate_optional_text(input.description.as_deref(), TaskError::InvalidDescription)?;
        validate_assignees(&input.assignee_refs)?;
        validate_status_completion(input.status, input.completed_at_epoch_seconds)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        if let Some(parent_task_id) = input.parent_task_id.as_deref() {
            validate_non_empty(parent_task_id, TaskError::InvalidTaskId)?;
            if parent_task_id == input.id {
                return Err(TaskError::SelfParentTask);
            }
        }
        if let Some(binding) = &input.foundry_binding {
            binding.validate()?;
        }

        Ok(Self {
            id: internal(input.id),
            store_id: internal(input.store_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            title: Classified::new(input.title, task_metadata_data_class()),
            description: Classified::new(input.description, task_content_data_class()),
            assignee_refs: Classified::new(input.assignee_refs, task_assignee_data_class()),
            status: internal(input.status),
            priority: internal(input.priority),
            parent_task_id: internal(input.parent_task_id),
            due_at_epoch_seconds: internal(input.due_at_epoch_seconds),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            foundry_binding: internal(input.foundry_binding),
            data_class: internal(data_class),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(TASK_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl FoundryAgentTaskBinding {
    pub fn new(
        capability_id: String,
        agent_ref: String,
        requested_tier: AutonomyTier,
        policy_decision_id: String,
        evidence_topic: String,
        tenant_auto_execute_grant_id: Option<String>,
    ) -> Result<Self, TaskError> {
        validate_capability_id(&capability_id)?;
        validate_non_empty(&agent_ref, TaskError::InvalidAgentRef)?;
        validate_non_empty(&policy_decision_id, TaskError::InvalidPolicyDecisionId)?;
        validate_non_empty(&evidence_topic, TaskError::InvalidEvidenceTopic)?;
        validate_auto_execute_grant(requested_tier, tenant_auto_execute_grant_id.as_deref())?;
        Ok(Self {
            capability_id: internal(capability_id),
            agent_ref: Classified::new(agent_ref, task_assignee_data_class()),
            requested_tier: internal(requested_tier),
            policy_decision_id: internal(policy_decision_id),
            evidence_topic: internal(evidence_topic),
            tenant_auto_execute_grant_id: internal(tenant_auto_execute_grant_id),
        })
    }

    fn validate(&self) -> Result<(), TaskError> {
        validate_capability_id(&self.capability_id.value)?;
        validate_non_empty(&self.agent_ref.value, TaskError::InvalidAgentRef)?;
        validate_non_empty(
            &self.policy_decision_id.value,
            TaskError::InvalidPolicyDecisionId,
        )?;
        validate_non_empty(&self.evidence_topic.value, TaskError::InvalidEvidenceTopic)?;
        validate_auto_execute_grant(
            self.requested_tier.value,
            self.tenant_auto_execute_grant_id.value.as_deref(),
        )
    }
}

impl TaskDependency {
    pub fn new(
        before_task_id: String,
        after_task_id: String,
        kind: TaskDependencyKind,
    ) -> Result<Self, TaskError> {
        validate_non_empty(&before_task_id, TaskError::InvalidTaskId)?;
        validate_non_empty(&after_task_id, TaskError::InvalidTaskId)?;
        if before_task_id == after_task_id {
            return Err(TaskError::SelfDependency);
        }
        Ok(Self {
            before_task_id: internal(before_task_id),
            after_task_id: internal(after_task_id),
            kind: internal(kind),
        })
    }
}

impl TaskGraph {
    pub fn new(
        tasks: Vec<WorkspaceTask>,
        dependencies: Vec<TaskDependency>,
    ) -> Result<Self, TaskError> {
        let graph = Self {
            tasks,
            dependencies,
            schema_version: internal(TASK_GRAPH_SCHEMA_VERSION),
        };
        graph.validate()?;
        Ok(graph)
    }

    pub fn validate(&self) -> Result<(), TaskError> {
        if self.tasks.is_empty() {
            return Err(TaskError::EmptyTaskGraph);
        }
        let mut ids = BTreeSet::new();
        let mut parent_by_id = BTreeMap::new();
        for task in &self.tasks {
            validate_non_empty(&task.id.value, TaskError::InvalidTaskId)?;
            if !ids.insert(task.id.value.clone()) {
                return Err(TaskError::DuplicateTaskId);
            }
            if let Some(parent_task_id) = task.parent_task_id.value.as_deref()
                && parent_task_id == task.id.value
            {
                return Err(TaskError::SelfParentTask);
            }
            parent_by_id.insert(task.id.value.clone(), task.parent_task_id.value.clone());
        }
        validate_parent_graph(&parent_by_id)?;
        validate_dependencies(&ids, &self.dependencies)
    }
}

pub fn default_workspace_task_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn task_content_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn task_assignee_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn task_metadata_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn workspace_task_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, TaskError> {
    PrivacyDataClass::new(data_class).map_err(|_| TaskError::InvalidDataClass)
}

fn validate_store_binding(input: &TaskCreate, store: &TaskStore) -> Result<(), TaskError> {
    validate_non_empty(&input.store_id, TaskError::InvalidStoreId)?;
    validate_non_empty(&input.tenant_id, TaskError::InvalidTenantId)?;
    validate_non_empty(&input.region, TaskError::InvalidRegion)?;
    validate_non_empty(&input.cell_id, TaskError::InvalidCellId)?;
    if input.store_id != store.id.value {
        return Err(TaskError::InvalidStoreId);
    }
    if input.tenant_id != store.tenant_id.value {
        return Err(TaskError::InvalidTenantId);
    }
    if input.region != store.region.value {
        return Err(TaskError::InvalidRegion);
    }
    if input.cell_id != store.cell_id.value {
        return Err(TaskError::InvalidCellId);
    }
    Ok(())
}

fn validate_capability_id(capability_id: &str) -> Result<(), TaskError> {
    if capability_id.starts_with("cap.") && capability_id.len() > "cap.".len() {
        Ok(())
    } else {
        Err(TaskError::InvalidCapabilityId)
    }
}

fn validate_auto_execute_grant(
    requested_tier: AutonomyTier,
    tenant_auto_execute_grant_id: Option<&str>,
) -> Result<(), TaskError> {
    match (requested_tier, tenant_auto_execute_grant_id) {
        (AutonomyTier::T4AutoExecute, Some(grant_id)) => {
            validate_non_empty(grant_id, TaskError::InvalidAutoExecuteGrant)
        }
        (AutonomyTier::T4AutoExecute, None) => Err(TaskError::MissingAutoExecuteGrant),
        (_, Some(_)) => Err(TaskError::InvalidAutoExecuteGrant),
        _ => Ok(()),
    }
}

fn validate_assignees(assignee_refs: &[String]) -> Result<(), TaskError> {
    let mut seen = BTreeSet::new();
    for assignee_ref in assignee_refs {
        validate_non_empty(assignee_ref, TaskError::InvalidAssigneeRef)?;
        if !seen.insert(assignee_ref) {
            return Err(TaskError::DuplicateAssigneeRef);
        }
    }
    Ok(())
}

fn validate_status_completion(
    status: TaskStatus,
    completed_at_epoch_seconds: Option<u64>,
) -> Result<(), TaskError> {
    match (status, completed_at_epoch_seconds) {
        (TaskStatus::Done, Some(_)) => Ok(()),
        (TaskStatus::Done, None) => Err(TaskError::MissingCompletionTime),
        (_, Some(_)) => Err(TaskError::UnexpectedCompletionTime),
        _ => Ok(()),
    }
}

fn validate_parent_graph(parent_by_id: &BTreeMap<String, Option<String>>) -> Result<(), TaskError> {
    for (task_id, parent_task_id) in parent_by_id {
        let Some(parent_task_id) = parent_task_id else {
            continue;
        };
        if !parent_by_id.contains_key(parent_task_id) {
            return Err(TaskError::UnknownTaskParent);
        }
        if task_id == parent_task_id {
            return Err(TaskError::SelfParentTask);
        }
        let mut seen = BTreeSet::new();
        let mut current = Some(task_id.clone());
        while let Some(current_id) = current {
            if !seen.insert(current_id.clone()) {
                return Err(TaskError::TaskParentCycle);
            }
            current = parent_by_id.get(&current_id).and_then(Clone::clone);
        }
    }
    Ok(())
}

fn validate_dependencies(
    task_ids: &BTreeSet<String>,
    dependencies: &[TaskDependency],
) -> Result<(), TaskError> {
    let mut seen = BTreeSet::new();
    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for dependency in dependencies {
        let before = &dependency.before_task_id.value;
        let after = &dependency.after_task_id.value;
        if before == after {
            return Err(TaskError::SelfDependency);
        }
        if !task_ids.contains(before) || !task_ids.contains(after) {
            return Err(TaskError::MissingDependencyEndpoint);
        }
        if !seen.insert((before.clone(), after.clone(), dependency.kind.value)) {
            return Err(TaskError::DuplicateDependency);
        }
        adjacency
            .entry(before.clone())
            .or_default()
            .push(after.clone());
    }
    validate_dependency_acyclic(&adjacency)
}

fn validate_dependency_acyclic(adjacency: &BTreeMap<String, Vec<String>>) -> Result<(), TaskError> {
    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    for task_id in adjacency.keys() {
        visit_dependency(task_id, adjacency, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit_dependency(
    task_id: &str,
    adjacency: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<(), TaskError> {
    if visited.contains(task_id) {
        return Ok(());
    }
    if !visiting.insert(task_id.to_owned()) {
        return Err(TaskError::TaskDependencyCycle);
    }
    if let Some(next_tasks) = adjacency.get(task_id) {
        for next_task in next_tasks {
            visit_dependency(next_task, adjacency, visiting, visited)?;
        }
    }
    visiting.remove(task_id);
    visited.insert(task_id.to_owned());
    Ok(())
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), TaskError> {
    if updated_at < created_at {
        Err(TaskError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_optional_text(value: Option<&str>, error: TaskError) -> Result<(), TaskError> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_text(value, error)
}

fn validate_text(value: &str, error: TaskError) -> Result<(), TaskError> {
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: TaskError) -> Result<(), TaskError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn store() -> TaskStore {
        TaskStore::new(TaskStoreCreate {
            id: "tasks-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            owner_ref: "user:owner@example.com".into(),
            data_class: None,
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        })
        .unwrap()
    }

    fn binding(tier: AutonomyTier, grant: Option<&str>) -> FoundryAgentTaskBinding {
        FoundryAgentTaskBinding::new(
            "cap.workspace.tasks.summarize".into(),
            "agent:task-planner".into(),
            tier,
            "policy-decision-1".into(),
            "oya.workspace.tasks.agent".into(),
            grant.map(str::to_owned),
        )
        .unwrap()
    }

    fn task_input(task_id: &str) -> TaskCreate {
        TaskCreate {
            id: task_id.into(),
            store_id: "tasks-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            title: format!("Task {task_id}"),
            description: Some("Coordinate rollout".into()),
            assignee_refs: vec!["user:owner@example.com".into()],
            status: TaskStatus::InProgress,
            priority: TaskPriority::High,
            parent_task_id: None,
            due_at_epoch_seconds: Some(1_700_086_400),
            completed_at_epoch_seconds: None,
            foundry_binding: Some(binding(AutonomyTier::T3ExecuteWithApproval, None)),
            data_class: None,
            created_at_epoch_seconds: 1_700_000_020,
            updated_at_epoch_seconds: 1_700_000_030,
        }
    }

    fn task(task_id: &str) -> WorkspaceTask {
        WorkspaceTask::new(task_input(task_id), &store()).unwrap()
    }

    #[test]
    fn task_store_defaults_to_identifying_and_classifies_owner() {
        let store = store();

        assert_eq!(
            store.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            store.owner_ref.data_class,
            DataClassification::Privacy(task_assignee_data_class())
        );
        assert_eq!(store.schema_version.value, 1);
    }

    #[test]
    fn foundry_agent_binding_requires_policy_and_auto_execute_grant() {
        assert_eq!(
            FoundryAgentTaskBinding::new(
                "workspace.tasks.bad".into(),
                "agent:task-planner".into(),
                AutonomyTier::T2Advisory,
                "policy-decision-1".into(),
                "oya.workspace.tasks.agent".into(),
                None,
            ),
            Err(TaskError::InvalidCapabilityId)
        );

        assert_eq!(
            FoundryAgentTaskBinding::new(
                "cap.workspace.tasks.close".into(),
                "agent:task-planner".into(),
                AutonomyTier::T4AutoExecute,
                "policy-decision-1".into(),
                "oya.workspace.tasks.agent".into(),
                None,
            ),
            Err(TaskError::MissingAutoExecuteGrant)
        );

        assert_eq!(
            FoundryAgentTaskBinding::new(
                "cap.workspace.tasks.advise".into(),
                "agent:task-planner".into(),
                AutonomyTier::T2Advisory,
                "policy-decision-1".into(),
                "oya.workspace.tasks.agent".into(),
                Some("grant-not-allowed".into()),
            ),
            Err(TaskError::InvalidAutoExecuteGrant)
        );

        assert!(
            binding(AutonomyTier::T4AutoExecute, Some("grant-1"))
                .tenant_auto_execute_grant_id
                .value
                .is_some()
        );
    }

    #[test]
    fn task_graph_rejects_parent_and_dependency_cycles() {
        let mut child_input = task_input("task-b");
        child_input.parent_task_id = Some("task-a".into());
        let child = WorkspaceTask::new(child_input, &store()).unwrap();
        let graph = TaskGraph::new(
            vec![task("task-a"), child],
            vec![
                TaskDependency::new("task-a".into(), "task-b".into(), TaskDependencyKind::Blocks)
                    .unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(graph.schema_version.value, 1);

        let dependency_cycle = TaskGraph::new(
            vec![task("task-a"), task("task-b")],
            vec![
                TaskDependency::new("task-a".into(), "task-b".into(), TaskDependencyKind::Blocks)
                    .unwrap(),
                TaskDependency::new("task-b".into(), "task-a".into(), TaskDependencyKind::Blocks)
                    .unwrap(),
            ],
        );
        assert_eq!(dependency_cycle, Err(TaskError::TaskDependencyCycle));

        let mut unknown_parent_input = task_input("task-c");
        unknown_parent_input.parent_task_id = Some("missing".into());
        let unknown_parent = WorkspaceTask::new(unknown_parent_input, &store()).unwrap();
        assert_eq!(
            TaskGraph::new(vec![task("task-a"), unknown_parent], Vec::new()),
            Err(TaskError::UnknownTaskParent)
        );
    }

    #[test]
    fn status_completion_invariants_are_explicit() {
        let mut done_missing = task_input("task-done");
        done_missing.status = TaskStatus::Done;
        assert_eq!(
            WorkspaceTask::new(done_missing, &store()),
            Err(TaskError::MissingCompletionTime)
        );

        let mut todo_completed = task_input("task-todo");
        todo_completed.completed_at_epoch_seconds = Some(1_700_000_040);
        assert_eq!(
            WorkspaceTask::new(todo_completed, &store()),
            Err(TaskError::UnexpectedCompletionTime)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_task_data_class_from_legacy(DataClass::Audit),
            Err(TaskError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.tasks STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksSurfaceStaging {
    pub task_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub status: Classified<String>,    // data_class: INTERNAL_ONLY
}

impl TasksSurfaceStaging {
    pub fn new(task_id: String, tenant_id: String, status: String) -> Self {
        Self {
            task_id: Classified::new(task_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            status: Classified::new(status, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> TasksSurfaceStaging {
        TasksSurfaceStaging::new("tasks-1".into(), "tasks-1".into(), "tasks-1".into())
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.task_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
