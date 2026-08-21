#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests for the saga-shape validator (ADR-0222).

use check_saga_shape::{
    AuditClass, CompensationKind, IdempotencyKeyStrategy, RollbackStrategy, SagaDefinition,
    SagaShapeViolationKind, SagaStep, canonical_microservice_catalog, validate_saga_shape,
};

fn fixture_saga(
    saga_id: &str,
    target: &str,
    compensation: CompensationKind,
    audit_class: AuditClass,
) -> SagaDefinition {
    SagaDefinition {
        saga_id: saga_id.into(),
        axis: "workspace".into(),
        owner_team: "ops-compliance".into(),
        version: "v1.0.0".into(),
        steps: vec![SagaStep {
            step_id: "step-a".into(),
            target_microservice: target.into(),
            forward_action_capability_ref: "demo.forward".into(),
            compensation_kind: compensation,
            compensation_capability_ref: match compensation {
                CompensationKind::NoopWithEvidence => None,
                _ => Some("demo.compensate".into()),
            },
            idempotency_key_strategy: IdempotencyKeyStrategy::SagaStepAttempt,
            timeout_budget_ms: 30_000,
            retry_max_attempts: 3,
            retry_backoff_ms: 1000,
            audit_class,
        }],
        rollback_strategy: RollbackStrategy::ReverseOrderCompensation,
        audit_chain_emit: true,
        source_path: format!("microservices/foo/specs/saga-{saga_id}.json"),
    }
}

#[test]
fn portfolio_passes_when_every_saga_well_shaped() {
    let catalog = canonical_microservice_catalog();
    let sagas = vec![
        fixture_saga(
            "tenant-onboard",
            "tenancy",
            CompensationKind::Cancel,
            AuditClass::WriteIdempotent,
        ),
        fixture_saga(
            "tenant-suspend",
            "tenancy",
            CompensationKind::Custom,
            AuditClass::WriteIdempotent,
        ),
        fixture_saga(
            "tenant-delete",
            "audit-chain",
            CompensationKind::Refund,
            AuditClass::SideEffectIrreversible,
        ),
        fixture_saga(
            "read-only-flow",
            "observability",
            CompensationKind::NoopWithEvidence,
            AuditClass::ReadOnly,
        ),
    ];
    let report = validate_saga_shape(sagas, &catalog).expect("portfolio sagas should validate");
    assert_eq!(report.sagas_checked, 4);
    assert_eq!(report.steps_checked, 4);
    assert!(report.advisories.is_empty());
}

#[test]
fn portfolio_fails_when_write_step_uses_noop_compensation() {
    let catalog = canonical_microservice_catalog();
    let sagas = vec![fixture_saga(
        "bad-tenant-write",
        "tenancy",
        CompensationKind::NoopWithEvidence,
        AuditClass::WriteIdempotent,
    )];
    let violations =
        validate_saga_shape(sagas, &catalog).expect_err("noop compensation on write must fail");
    assert!(
        violations
            .iter()
            .any(|v| v.kind == SagaShapeViolationKind::NoopCompensationOnWriteStep)
    );
}

#[test]
fn portfolio_advises_when_target_microservice_unknown() {
    let catalog = canonical_microservice_catalog();
    let sagas = vec![fixture_saga(
        "demo",
        "not-a-real-microservice",
        CompensationKind::Custom,
        AuditClass::WriteIdempotent,
    )];
    let report =
        validate_saga_shape(sagas, &catalog).expect("unknown target should not fail; only advise");
    assert_eq!(report.advisories.len(), 1);
    assert!(
        report.advisories[0]
            .message
            .contains("not-a-real-microservice")
    );
}
