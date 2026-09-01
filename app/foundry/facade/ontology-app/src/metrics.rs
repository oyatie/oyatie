//! The Prometheus exposition surface, hand-rendered against the platform's
//! own precedent so the process carries no metrics dependency.
//!
//! Every counter here is one an SLO may name. The rule runs the other way
//! too: an SLI that names a metric this process does not export is a
//! declared objective with no signal behind it, which is worse than no
//! objective at all.

use crate::composition::AppState;

/// Render the current gauges in Prometheus text format.
pub fn prometheus_text(state: &AppState) -> String {
    let mut out = String::new();
    push_gauge(
        &mut out,
        "foundry_projection_lag",
        "Entries appended to a tenant's log that its projection has not yet consumed.",
        state
            .tenants
            .values()
            .map(|tenant| {
                tenant
                    .try_lock()
                    .map_or(0, |tenant| tenant.sync_status().lag)
            })
            .sum(),
    );
    push_gauge(
        &mut out,
        "foundry_poisoned_entries",
        "Log entries the fold consumed and deterministically refused.",
        state.poisoned_count(),
    );
    push_gauge(
        &mut out,
        "foundry_served_tenants",
        "Tenants this process serves; the configured roster is the served set.",
        state.tenant_count() as u64,
    );
    // Read accounting is born here with the probe surface so the read SLO
    // has a denominator from the first lane; the request surfaces increment
    // it as they land.
    push_counter(
        &mut out,
        "foundry_read_total",
        "Read requests served, by route and outcome.",
    );
    out
}

fn push_gauge(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!(
        "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}\n"
    ));
}

fn push_counter(out: &mut String, name: &str, help: &str) {
    out.push_str(&format!("# HELP {name} {help}\n# TYPE {name} counter\n"));
}
