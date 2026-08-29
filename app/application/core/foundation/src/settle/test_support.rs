//! Shared fixtures for the settlement regression modules.

use crate::*;

pub(super) fn settlement_foundation() -> Foundation {
    let mut foundation = Foundation::default();
    foundation
        .capabilities
        .publish(
            Capability::new(
                "cap.demo.saga".into(),
                "demo".into(),
                AutonomyTier::T2Advisory,
                vec![PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()],
                "oya.foundry.capability.invoked".into(),
            )
            .unwrap(),
        )
        .unwrap();
    foundation
        .cost_budgets
        .configure_tenant_ceiling(
            "ten_saga".into(),
            "saga-window".into(),
            BudgetCeiling::new(1_000, 100, 80).unwrap(),
        )
        .unwrap();
    foundation
}

pub(super) fn settlement_request() -> CapabilityInvocationRequest {
    CapabilityInvocationRequest {
        tenant_id: "ten_saga".into(),
        user_id: "usr_saga".into(),
        capability_id: "cap.demo.saga".into(),
        purpose: Purpose::CapabilityInvocation,
        subject_class: SubjectClass::Adult,
        budget_window_id: "saga-window".into(),
        projected_cost_micros: 10,
        started_at_epoch_seconds: 1_000,
    }
}

pub(super) fn settlement_scope() -> BudgetScope {
    BudgetScope::new(
        "ten_saga".into(),
        "cap.demo.saga".into(),
        "saga-window".into(),
    )
    .unwrap()
}
