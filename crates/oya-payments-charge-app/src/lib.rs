//! Payments charge-BC composition root — K8s binary entry point that wires
//! REST + gRPC surfaces against the usecase layer.
//!
//! Wave 15-IMPL-truth-up scaffold; full Tokio multi-thread runtime +
//! graceful shutdown + Cloud Hypervisor / Kata pod shape per ADR-0254 in
//! IP-015.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Application composition root.
#[allow(dead_code)]
pub struct ChargeApp {
    _placeholder: (),
}
