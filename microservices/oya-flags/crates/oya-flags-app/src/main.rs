//! oya-flags binary entrypoint (ADR-0481).
//!
//! Starts the OpenFeature flag server on `OYA_FLAGS_ADDR` (default: `0.0.0.0:8080`).

#![forbid(unsafe_code)]

fn main() {
    // Stage-1 scaffold: binary compiles and exits cleanly.
    // Stage-2 will wire tokio + axum router + DefaultFlagResolver.
    // Tracked: registry/placeholder-debt/adr-follow-ups.yaml#oya-flags-stage-2
    tracing_subscriber::fmt().json().init();
    tracing::info!(service = "oya-flags", "scaffold placeholder — Stage-2 wires HTTP runtime");
}
