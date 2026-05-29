//! Payments charge-BC gRPC surface — implements `PaymentsService` from
//! `contracts/payments-v1.proto`.
//!
//! Wave 15-IMPL-truth-up scaffold; full tonic + proto3 wiring in IP-015.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// gRPC service stub.
#[allow(dead_code)]
pub struct PaymentsGrpcService {
    _placeholder: (),
}
