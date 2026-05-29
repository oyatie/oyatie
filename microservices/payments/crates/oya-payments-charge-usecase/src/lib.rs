//! Payments charge-BC usecase — `CreateChargeUseCase`, `CaptureChargeUseCase`,
//! `VoidChargeUseCase` application services.
//!
//! Wave 15-IMPL-truth-up scaffold; full Cedar-first orchestration in IP-003.
//! Steps: Cedar eval → fraud-score → PSP routing → domain aggregate → audit emit.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// CreateChargeUseCase orchestrates Cedar + fraud + PSP + persist + audit.
#[allow(dead_code)]
pub struct CreateChargeUseCase {
    _placeholder: (),
}

/// CaptureChargeUseCase advances `Authorized → Captured`.
#[allow(dead_code)]
pub struct CaptureChargeUseCase {
    _placeholder: (),
}

/// VoidChargeUseCase advances `Authorized → Voided`.
#[allow(dead_code)]
pub struct VoidChargeUseCase {
    _placeholder: (),
}
