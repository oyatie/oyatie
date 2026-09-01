//! The Prometheus exposition surface, hand-rendered against the platform's
//! own precedent so the process carries no metrics dependency.
//!
//! **One table, two consumers.** `samples` is the single place a metric
//! exists: `prometheus_text` renders from it and `exported_metric_names`
//! reads from it, so deleting a metric removes it from BOTH and any
//! objective naming it fails. An earlier revision kept those as separate
//! hand-maintained lists, and they had already diverged inside one diff —
//! the list omitted the very gauge an objective's denominator needed.
//!
//! Counters are process-lifetime and unlabelled by tenant: this surface is
//! unauthenticated by design, so it must not become a tenancy oracle. That
//! choice constrains what an objective can express, and the objectives are
//! written to what it can, not the reverse.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::composition::AppState;

/// Request accounting. Served and refused are separate counters rather than
/// one labelled counter, so an objective reads two independent numbers and a
/// refusal can never be silently folded into a success.
#[derive(Debug, Default)]
pub struct Metrics {
    submit_served: AtomicU64,  // data_class: INTERNAL_ONLY
    submit_refused: AtomicU64, // data_class: INTERNAL_ONLY
    read_served: AtomicU64,    // data_class: INTERNAL_ONLY
    read_refused: AtomicU64,   // data_class: INTERNAL_ONLY
}

impl Metrics {
    pub fn submit_served(&self) {
        self.submit_served.fetch_add(1, Ordering::Relaxed);
    }

    pub fn submit_refused(&self) {
        self.submit_refused.fetch_add(1, Ordering::Relaxed);
    }

    pub fn read_served(&self) {
        self.read_served.fetch_add(1, Ordering::Relaxed);
    }

    pub fn read_refused(&self) {
        self.read_refused.fetch_add(1, Ordering::Relaxed);
    }
}

/// One exported series.
pub struct Sample {
    pub name: &'static str, // data_class: INTERNAL_ONLY
    pub kind: &'static str, // data_class: INTERNAL_ONLY
    pub help: &'static str, // data_class: INTERNAL_ONLY
    pub value: u64,         // data_class: INTERNAL_ONLY
}

/// Everything this process exports, evaluated against current state. The
/// single source: add a metric here and both the exposition and the
/// objective-validation see it; delete one and both stop seeing it.
pub fn samples(state: &AppState) -> Vec<Sample> {
    let metrics = &state.metrics;
    vec![
        Sample {
            name: "foundry_projection_lag",
            kind: "gauge",
            help: "Entries appended to a tenant's log that its projection has not yet consumed.",
            value: state
                .tenants
                .values()
                .map(|tenant| {
                    tenant
                        .try_lock()
                        .map_or(0, |tenant| tenant.sync_status().lag)
                })
                .sum(),
        },
        Sample {
            name: "foundry_poisoned_entries",
            kind: "gauge",
            help: "Log entries the fold consumed and deterministically refused.",
            value: state.poisoned_count(),
        },
        Sample {
            name: "foundry_served_tenants",
            kind: "gauge",
            help: "Tenants this process serves; the configured roster is the served set.",
            value: state.tenant_count() as u64,
        },
        Sample {
            name: "foundry_action_submit_served_total",
            kind: "counter",
            help: "Action submissions the writer accepted into the log.",
            value: metrics.submit_served.load(Ordering::Relaxed),
        },
        Sample {
            name: "foundry_action_submit_refused_total",
            kind: "counter",
            help: "Action submissions refused before or by the writer.",
            value: metrics.submit_refused.load(Ordering::Relaxed),
        },
        Sample {
            name: "foundry_read_served_total",
            kind: "counter",
            help: "Read requests answered from the projection.",
            value: metrics.read_served.load(Ordering::Relaxed),
        },
        Sample {
            name: "foundry_read_refused_total",
            kind: "counter",
            help: "Read requests refused by credential, policy, or surface.",
            value: metrics.read_refused.load(Ordering::Relaxed),
        },
    ]
}

/// The metric names this process exports, read from the same table the
/// exposition renders. The SLO suite asserts every declared indicator names
/// one of these, so an objective cannot outlive its signal.
pub fn exported_metric_names(state: &AppState) -> BTreeSet<&'static str> {
    samples(state)
        .into_iter()
        .map(|sample| sample.name)
        .collect()
}

/// Render the current values in Prometheus text format.
pub fn prometheus_text(state: &AppState) -> String {
    let mut out = String::new();
    for sample in samples(state) {
        out.push_str(&format!(
            "# HELP {name} {help}\n# TYPE {name} {kind}\n{name} {value}\n",
            name = sample.name,
            help = sample.help,
            kind = sample.kind,
            value = sample.value,
        ));
    }
    out
}
