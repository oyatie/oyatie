//! The Prometheus exposition surface, hand-rendered against the platform's
//! own precedent so the process carries no metrics dependency.
//!
//! **One table, three consumers.** `samples` is the single place a metric
//! exists: `prometheus_text` renders EVERY sample from it, while
//! `objective_eligible_metrics` and `objective_ineligible_metrics` partition
//! it, so deleting a metric removes it from all three and any objective
//! naming it fails. An earlier revision kept those as separate
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
    /// Whether an objective may be written over this series. EXPORTED IS NOT
    /// ELIGIBLE: a metric can be worth showing an operator and still be
    /// useless as an indicator, and "is exported" is too weak a predicate to
    /// keep declared coverage honest — it admits an objective over a series
    /// that cannot move. A sample that is not eligible carries the reason on
    /// its `ineligible_because` line.
    pub objective_eligible: bool, // data_class: INTERNAL_ONLY
    /// Why this series may not back an objective, or empty when it may.
    /// Read into the failure message, so the refusal explains itself.
    pub ineligible_because: &'static str, // data_class: INTERNAL_ONLY
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
            help: "Entries in a tenant's BOOT-TIME log mirror that its projection has \
                   not yet consumed. Not a freshness signal: the mirror is fixed at \
                   compose and never appended to, so this cannot observe an append made \
                   after boot and is zero for the life of the process.",
            value: state
                .tenants
                .values()
                .map(|tenant| {
                    tenant
                        .try_lock()
                        .map_or(0, |tenant| tenant.sync_status().lag)
                })
                .sum(),
            objective_eligible: false,
            ineligible_because: "the lag is read from the boot-time entries mirror, which is \
                                  never appended to after compose, so this series is zero for \
                                  the life of the process and an objective over it could not \
                                  breach",
        },
        Sample {
            name: "foundry_poisoned_entries",
            kind: "gauge",
            help: "Log entries the fold consumed and deterministically refused.",
            value: state.poisoned_count(),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_served_tenants",
            kind: "gauge",
            help: "Tenants this process serves; the configured roster is the served set.",
            value: state.tenant_count() as u64,
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_action_submit_served_total",
            kind: "counter",
            help: "Action submissions the writer accepted into the log.",
            value: metrics.submit_served.load(Ordering::Relaxed),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_action_submit_refused_total",
            kind: "counter",
            help: "Action submissions refused before or by the writer.",
            value: metrics.submit_refused.load(Ordering::Relaxed),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_read_served_total",
            kind: "counter",
            help: "Read requests answered from the projection.",
            value: metrics.read_served.load(Ordering::Relaxed),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_read_refused_total",
            kind: "counter",
            help: "Read requests refused by credential, policy, surface, or an \
                   unreadable log.",
            value: metrics.read_refused.load(Ordering::Relaxed),
            objective_eligible: true,
            ineligible_because: "",
        },
    ]
}

/// The metric names an objective MAY be written over. This is a strict
/// subset of what the exposition renders: a series can be worth showing an
/// operator and still be unusable as an indicator, and the earlier name for
/// this function promised the full export set while returning the filtered
/// one. The SLO suite asserts every declared indicator names one of these,
/// so an objective cannot outlive its signal.
pub fn objective_eligible_metrics(state: &AppState) -> BTreeSet<&'static str> {
    samples(state)
        .into_iter()
        .filter(|sample| sample.objective_eligible)
        .map(|sample| sample.name)
        .collect()
}

/// Series this process exports that an objective may NOT be written over,
/// each with its reason. The refusal is only useful if it says why, and the
/// reason has to travel with the table rather than living in a comment
/// beside the objectives — a rule nothing reconciles is not a rule.
pub fn objective_ineligible_metrics(state: &AppState) -> Vec<(&'static str, &'static str)> {
    samples(state)
        .into_iter()
        .filter(|sample| !sample.objective_eligible)
        .map(|sample| (sample.name, sample.ineligible_because))
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
