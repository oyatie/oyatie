//! `oya-meter-rest` — axum REST adapter for the oya-meter metering surface (ADR-0479).
//!
//! Stub: HTTP routes for usage event ingestion and query will be implemented
//! per ADR-0479 D1-D5. This crate wires [`oya_meter_kernel`] types into axum
//! handlers and produces a [`axum::Router`] for the composition root to mount.

#![forbid(unsafe_code)]

// TODO: implement per ADR-0479 D1-D5

use axum::Router;

/// Build the metering REST router.
///
/// Routes (stubbed — full impl per ADR-0479 D1-D5):
/// - `POST /usage/events` — ingest a usage event
/// - `GET  /usage/tenants/{tenant_id}/totals` — query aggregated totals
#[must_use]
pub fn router() -> Router {
    // TODO: implement per ADR-0479 D1-D5
    Router::new()
}
