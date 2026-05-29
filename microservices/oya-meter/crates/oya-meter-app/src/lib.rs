//! `oya-meter-app` — composition root for oya-meter (ADR-0479).
//!
//! Wires [`oya_meter_kernel`] + [`oya_meter_rest`] into a runnable metering
//! µservice. Owns the tokio runtime lifecycle, tracing init, and TCP listener.
//!
//! # Layering invariant
//! Path-deps inward on `-kernel` and `-rest` only. No business logic lives
//! here; this crate is exclusively wiring + lifecycle.

#![forbid(unsafe_code)]

// TODO: implement per ADR-0479 D1-D5

/// Run the oya-meter service until the process is signalled.
///
/// Binds on the address supplied via `OYA_METER_ADDR` (default `0.0.0.0:8080`).
pub async fn run() {
    // TODO: implement per ADR-0479 D1-D5
    let router = oya_meter_rest::router();
    let addr = std::env::var("OYA_METER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind OYA_METER_ADDR");
    tracing::info!(addr, "oya-meter listening");
    axum::serve(listener, router)
        .await
        .expect("oya-meter serve error");
}
