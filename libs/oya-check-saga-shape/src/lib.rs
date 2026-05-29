//! Saga-shape validator (ADR-0222).
//!
//! Kernel-tier validator (ADR-0083 Tier 1): pure data + small validators;
//! no filesystem, no subprocess, no network. Callers supply the list of
//! `SagaDefinition` records read from `microservices/<axis>/specs/saga-*.json`
//! and the canonical schema (loaded from `/specs/saga-shape.json`).
//!
//! The validator checks the four invariants from ADR-0222:
//!
//! - I-1. Every saga has at least one step.
//! - I-2. Every step declares all six required fields
//!   (forward_action, compensation_action, idempotency_key_strategy,
//!   timeout_budget_ms, retry_policy, audit_class).
//! - I-3. If `audit_class != ReadOnly` then `compensation_action.kind` MUST
//!   NOT be `NoopWithEvidence` (per ADR-0222 D-1 paragraph 2).
//! - I-4. Step ids are unique within a saga.
//!
//! Non-validating (advisory) checks land as warnings:
//!
//! - W-1. `target_microservice` should appear in the canonical 32-µservice
//!   catalog. The catalog is supplied by the caller; this kernel
//!   doesn't read it from disk.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` / `panic!()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// One saga definition as ingested by the validator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaDefinition {
    pub saga_id: String,
    pub axis: String,
    pub owner_team: String,
    pub version: String,
    pub steps: Vec<SagaStep>,
    pub rollback_strategy: RollbackStrategy,
    pub audit_chain_emit: bool,
    /// Path the definition was loaded from. Used in violation messages.
    pub source_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaStep {
    pub step_id: String,
    pub target_microservice: String,
    pub forward_action_capability_ref: String,
    pub compensation_kind: CompensationKind,
    pub compensation_capability_ref: Option<String>,
    pub idempotency_key_strategy: IdempotencyKeyStrategy,
    pub timeout_budget_ms: u32,
    pub retry_max_attempts: u8,
    pub retry_backoff_ms: u32,
    pub audit_class: AuditClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompensationKind {
    Cancel,
    Refund,
    Retry,
    NoopWithEvidence,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdempotencyKeyStrategy {
    SagaStepAttempt,
    RequestBodyHash,
    ClientSupplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditClass {
    ReadOnly,
    WriteIdempotent,
    WriteNonIdempotent,
    SideEffectIrreversible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RollbackStrategy {
    ReverseOrderCompensation,
    NoCompensationReadOnly,
    BestEffortWithPage,
}

/// Validation outcome (success path).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaShapeReport {
    pub sagas_checked: usize,
    pub steps_checked: usize,
    pub advisories: Vec<SagaShapeAdvisory>,
}

/// Per-saga violation (failure path).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaShapeViolation {
    pub saga_id: String,
    pub source_path: String,
    pub step_id: Option<String>,
    pub kind: SagaShapeViolationKind,
    pub message: String,
    pub fix: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SagaShapeViolationKind {
    EmptyStepList,
    NoopCompensationOnWriteStep,
    DuplicateStepId,
    InvalidTimeoutBudget,
    InvalidRetryAttempts,
}

/// Advisory note (does not fail the gate).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SagaShapeAdvisory {
    pub saga_id: String,
    pub source_path: String,
    pub step_id: Option<String>,
    pub message: String,
}

impl fmt::Display for SagaShapeViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let step_label = self
            .step_id
            .as_deref()
            .map(|step| format!("[step={step}]"))
            .unwrap_or_default();
        write!(
            formatter,
            "{}:{} {:?}{}: {} fix: {}",
            self.source_path, self.saga_id, self.kind, step_label, self.message, self.fix
        )
    }
}

/// Canonical flattened µservice catalog (per microservices/ directory at ADR-0131
/// per-microservice flat layout). Callers may inject a different catalog
/// for tests.
pub fn canonical_microservice_catalog() -> BTreeSet<&'static str> {
    [
        "anonymous",
        "application",
        "audit-chain",
        "calendar",
        "cell",
        "cloud-iac",
        "cloud-k8s",
        "cloud-secrets",
        "community",
        "connector",
        "drive",
        "forms",
        "foundry",
        "governance",
        "mail",
        "meet",
        "messenger",
        "network",
        "notes",
        "observability",
        "ontology",
        "recordings",
        "sheets",
        "shorts",
        "sites",
        "slides",
        "social",
        "tasks",
        "tenancy",
        "translate",
        "workflow-engine",
        "workflow-studio",
    ]
    .into_iter()
    .collect()
}

/// Run the validator over the supplied saga definitions.
///
/// Returns `Ok(report)` if all I-1..I-4 invariants pass. Otherwise returns
/// the violation list. Advisories are always returned inside the report
/// (on the success path) or alongside violations (on the failure path is
/// indirected — callers see the violation list first; advisories surface
/// in the next run after fixes).
pub fn validate_saga_shape<S>(
    sagas: S,
    canonical_catalog: &BTreeSet<&str>,
) -> Result<SagaShapeReport, Vec<SagaShapeViolation>>
where
    S: IntoIterator<Item = SagaDefinition>,
{
    let mut violations = Vec::new();
    let mut advisories = Vec::new();
    let mut sagas_checked = 0usize;
    let mut steps_checked = 0usize;

    for saga in sagas {
        sagas_checked += 1;

        // I-1. Every saga has at least one step.
        if saga.steps.is_empty() {
            violations.push(SagaShapeViolation {
                saga_id: saga.saga_id.clone(),
                source_path: saga.source_path.clone(),
                step_id: None,
                kind: SagaShapeViolationKind::EmptyStepList,
                message: "saga has zero steps".into(),
                fix: "add at least one step per ADR-0222 D-1".into(),
            });
            continue;
        }

        // I-4. Step ids unique within saga.
        let mut seen_step_ids: BTreeSet<&str> = BTreeSet::new();
        for step in &saga.steps {
            if !seen_step_ids.insert(step.step_id.as_str()) {
                violations.push(SagaShapeViolation {
                    saga_id: saga.saga_id.clone(),
                    source_path: saga.source_path.clone(),
                    step_id: Some(step.step_id.clone()),
                    kind: SagaShapeViolationKind::DuplicateStepId,
                    message: format!("step_id {} is not unique within the saga", step.step_id),
                    fix: "rename one of the duplicate steps; step_ids must be unique within a saga"
                        .into(),
                });
            }
        }

        for step in &saga.steps {
            steps_checked += 1;

            // I-3. NoopWithEvidence requires ReadOnly audit class.
            if matches!(step.compensation_kind, CompensationKind::NoopWithEvidence)
                && !matches!(step.audit_class, AuditClass::ReadOnly)
            {
                violations.push(SagaShapeViolation {
                    saga_id: saga.saga_id.clone(),
                    source_path: saga.source_path.clone(),
                    step_id: Some(step.step_id.clone()),
                    kind: SagaShapeViolationKind::NoopCompensationOnWriteStep,
                    message: format!(
                        "step {} declares audit_class={:?} but compensation_action.kind=NoopWithEvidence (ADR-0222 D-1: only ReadOnly steps may compensate with Noop)",
                        step.step_id, step.audit_class
                    ),
                    fix: "either change audit_class to ReadOnly or change compensation_action.kind to Cancel/Refund/Retry/Custom".into(),
                });
            }

            // I-2. Required-field sanity (numeric ranges) — schema-level checks.
            if step.timeout_budget_ms < 100 || step.timeout_budget_ms > 600_000 {
                violations.push(SagaShapeViolation {
                    saga_id: saga.saga_id.clone(),
                    source_path: saga.source_path.clone(),
                    step_id: Some(step.step_id.clone()),
                    kind: SagaShapeViolationKind::InvalidTimeoutBudget,
                    message: format!(
                        "step {} timeout_budget_ms = {} (must be 100..=600_000 per /specs/saga-shape.json)",
                        step.step_id, step.timeout_budget_ms
                    ),
                    fix: "set timeout_budget_ms in the range 100..=600_000".into(),
                });
            }
            if step.retry_max_attempts < 1 || step.retry_max_attempts > 10 {
                violations.push(SagaShapeViolation {
                    saga_id: saga.saga_id.clone(),
                    source_path: saga.source_path.clone(),
                    step_id: Some(step.step_id.clone()),
                    kind: SagaShapeViolationKind::InvalidRetryAttempts,
                    message: format!(
                        "step {} retry_policy.max_attempts = {} (must be 1..=10 per /specs/saga-shape.json)",
                        step.step_id, step.retry_max_attempts
                    ),
                    fix: "set retry_policy.max_attempts in the range 1..=10".into(),
                });
            }

            // W-1. Advisory — target_microservice in canonical catalog.
            if !canonical_catalog.contains(step.target_microservice.as_str()) {
                advisories.push(SagaShapeAdvisory {
                    saga_id: saga.saga_id.clone(),
                    source_path: saga.source_path.clone(),
                    step_id: Some(step.step_id.clone()),
                    message: format!(
                        "step {} target_microservice={} is not in the canonical 32-µservice catalog",
                        step.step_id, step.target_microservice
                    ),
                });
            }
        }
    }

    if violations.is_empty() {
        Ok(SagaShapeReport {
            sagas_checked,
            steps_checked,
            advisories,
        })
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_only_step(step_id: &str, kind: CompensationKind) -> SagaStep {
        SagaStep {
            step_id: step_id.into(),
            target_microservice: "tenancy".into(),
            forward_action_capability_ref: "tenancy.read.foo".into(),
            compensation_kind: kind,
            compensation_capability_ref: None,
            idempotency_key_strategy: IdempotencyKeyStrategy::SagaStepAttempt,
            timeout_budget_ms: 10_000,
            retry_max_attempts: 3,
            retry_backoff_ms: 1000,
            audit_class: AuditClass::ReadOnly,
        }
    }

    fn write_step(step_id: &str, kind: CompensationKind) -> SagaStep {
        SagaStep {
            step_id: step_id.into(),
            target_microservice: "tenancy".into(),
            forward_action_capability_ref: "tenancy.write.foo".into(),
            compensation_kind: kind,
            compensation_capability_ref: Some("tenancy.write.foo_compensate".into()),
            idempotency_key_strategy: IdempotencyKeyStrategy::SagaStepAttempt,
            timeout_budget_ms: 10_000,
            retry_max_attempts: 3,
            retry_backoff_ms: 1000,
            audit_class: AuditClass::WriteIdempotent,
        }
    }

    fn saga(steps: Vec<SagaStep>) -> SagaDefinition {
        SagaDefinition {
            saga_id: "tenant-suspend-saga".into(),
            axis: "workspace".into(),
            owner_team: "ops-compliance".into(),
            version: "v1.0.0".into(),
            steps,
            rollback_strategy: RollbackStrategy::ReverseOrderCompensation,
            audit_chain_emit: true,
            source_path: "microservices/tenancy/specs/saga-suspend.json".into(),
        }
    }

    #[test]
    fn valid_saga_passes() {
        let catalog = canonical_microservice_catalog();
        let result = validate_saga_shape(
            vec![saga(vec![write_step(
                "revoke_tokens",
                CompensationKind::Custom,
            )])],
            &catalog,
        );
        let report = result.unwrap();
        assert_eq!(report.sagas_checked, 1);
        assert_eq!(report.steps_checked, 1);
    }

    #[test]
    fn read_only_step_with_noop_compensation_passes() {
        let catalog = canonical_microservice_catalog();
        let result = validate_saga_shape(
            vec![saga(vec![read_only_step(
                "verify_state",
                CompensationKind::NoopWithEvidence,
            )])],
            &catalog,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn write_step_with_noop_compensation_fails() {
        let catalog = canonical_microservice_catalog();
        let result = validate_saga_shape(
            vec![saga(vec![write_step(
                "revoke_tokens",
                CompensationKind::NoopWithEvidence,
            )])],
            &catalog,
        );
        let violations = result.unwrap_err();
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].kind,
            SagaShapeViolationKind::NoopCompensationOnWriteStep
        );
    }

    #[test]
    fn empty_step_list_fails() {
        let catalog = canonical_microservice_catalog();
        let result = validate_saga_shape(vec![saga(vec![])], &catalog);
        let violations = result.unwrap_err();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].kind, SagaShapeViolationKind::EmptyStepList);
    }

    #[test]
    fn duplicate_step_ids_fail() {
        let catalog = canonical_microservice_catalog();
        let result = validate_saga_shape(
            vec![saga(vec![
                write_step("revoke_tokens", CompensationKind::Custom),
                write_step("revoke_tokens", CompensationKind::Custom),
            ])],
            &catalog,
        );
        let violations = result.unwrap_err();
        assert!(
            violations
                .iter()
                .any(|v| v.kind == SagaShapeViolationKind::DuplicateStepId)
        );
    }

    #[test]
    fn invalid_timeout_budget_fails() {
        let catalog = canonical_microservice_catalog();
        let mut step = write_step("revoke_tokens", CompensationKind::Custom);
        step.timeout_budget_ms = 50; // below 100 ms floor
        let result = validate_saga_shape(vec![saga(vec![step])], &catalog);
        let violations = result.unwrap_err();
        assert!(
            violations
                .iter()
                .any(|v| v.kind == SagaShapeViolationKind::InvalidTimeoutBudget)
        );
    }

    #[test]
    fn invalid_retry_attempts_fail() {
        let catalog = canonical_microservice_catalog();
        let mut step = write_step("revoke_tokens", CompensationKind::Custom);
        step.retry_max_attempts = 11; // above 10 ceiling
        let result = validate_saga_shape(vec![saga(vec![step])], &catalog);
        let violations = result.unwrap_err();
        assert!(
            violations
                .iter()
                .any(|v| v.kind == SagaShapeViolationKind::InvalidRetryAttempts)
        );
    }

    #[test]
    fn unknown_target_microservice_produces_advisory_not_violation() {
        let catalog = canonical_microservice_catalog();
        let mut step = write_step("revoke_tokens", CompensationKind::Custom);
        step.target_microservice = "not-a-real-microservice".into();
        let result = validate_saga_shape(vec![saga(vec![step])], &catalog);
        let report = result.unwrap();
        assert_eq!(report.advisories.len(), 1);
    }

    #[test]
    fn canonical_catalog_has_32_microservices() {
        let catalog = canonical_microservice_catalog();
        assert_eq!(catalog.len(), 32);
    }

    #[test]
    fn violation_display_includes_path_and_step() {
        let v = SagaShapeViolation {
            saga_id: "demo-saga".into(),
            source_path: "microservices/foo/specs/saga-demo.json".into(),
            step_id: Some("step-a".into()),
            kind: SagaShapeViolationKind::EmptyStepList,
            message: "demo".into(),
            fix: "demo-fix".into(),
        };
        let rendered = format!("{v}");
        assert!(rendered.contains("microservices/foo/specs/saga-demo.json"));
        assert!(rendered.contains("step-a"));
    }
}
