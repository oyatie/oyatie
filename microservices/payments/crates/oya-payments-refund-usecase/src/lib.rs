//! Payments refund-BC usecase — `IssueRefundUseCase`.
//!
//! Wave 15-IMPL-truth-up scaffold; full Cedar → charge load → window check →
//! PSP refund call → persist → audit emit pipeline in IP-006.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// IssueRefundUseCase orchestrates refund issuance.
#[allow(dead_code)]
pub struct IssueRefundUseCase {
    _placeholder: (),
}
