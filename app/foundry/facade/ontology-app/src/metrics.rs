//! Counters are process-lifetime and unlabelled by tenant: this surface is
//! unauthenticated by design, so it must not become a tenancy oracle. That
//! choice constrains what an objective can express, and the objectives are
//! written to what it can, not the reverse.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::composition::AppState;
use crate::observation::observe;

/// Request accounting. Served and refused are separate counters rather than
/// one labelled counter, so an objective reads two independent numbers and a
/// refusal can never be silently folded into a success.
#[derive(Debug, Default)]
pub struct Metrics {
    submit_served: AtomicU64,  // data_class: INTERNAL_ONLY
    submit_refused: AtomicU64, // data_class: INTERNAL_ONLY
    read_served: AtomicU64,    // data_class: INTERNAL_ONLY
    read_refused: AtomicU64,   // data_class: INTERNAL_ONLY
    /// Accepted invocations on the single-Action surface, timed from the
    /// handler's entry to its answer; the migration run surface, where one
    /// request is many writes, is not timed against this budget.
    invocation_answered: AtomicU64, // data_class: INTERNAL_ONLY
    invocation_answered_within_budget: AtomicU64, // data_class: INTERNAL_ONLY
    /// Refusals the WRITER issued on the single-Action surface, and how many
    /// of them the denial trail holds. A refusal before the writer (no
    /// credential, a malformed body, an unserved tenant, a policy denial, an
    /// unrepresentable edit) is not a denial, and neither is an append the
    /// log refused after the writer's gates passed.
    denial_issued: AtomicU64, // data_class: INTERNAL_ONLY
    denial_recorded: AtomicU64, // data_class: INTERNAL_ONLY
}

/// The latency objective's budget; the series name carries the same number.
pub const INVOCATION_LATENCY_BUDGET: Duration = Duration::from_millis(250);
const INVOCATION_WITHIN_BUDGET_SERIES: &str =
    "foundry_action_invocation_answered_within_250ms_total";

impl Metrics {
    pub fn submit_served(&self) {
        self.submit_served.fetch_add(1, Ordering::Relaxed);
    }

    pub fn invocation_answered(&self, elapsed: Duration) {
        self.invocation_answered.fetch_add(1, Ordering::Relaxed);
        if elapsed <= INVOCATION_LATENCY_BUDGET {
            self.invocation_answered_within_budget
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn denial_issued(&self, recorded_on_trail: bool) {
        self.denial_issued.fetch_add(1, Ordering::Relaxed);
        if recorded_on_trail {
            self.denial_recorded.fetch_add(1, Ordering::Relaxed);
        }
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
    /// Empty exactly when `objective_eligible`; read into the refusal.
    pub ineligible_because: &'static str, // data_class: INTERNAL_ONLY
}

pub fn samples(state: &AppState) -> Vec<Sample> {
    let metrics = &state.metrics;
    // ONE observation for every gauge below, so the three cannot disagree.
    let seen = observe(state);
    vec![
        Sample {
            name: "foundry_projection_lag",
            kind: "gauge",
            help: "Entries durably appended to a tenant's log that its projection has \
                   not yet consumed, summed over served tenants.",
            value: seen.lag,
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_projection_contended",
            kind: "gauge",
            help: "Served tenants whose mutex was held when the scrape ran — a request \
                   in flight, not a fault. Subtract from foundry_projection_lag_unknown \
                   for the tenants whose log head could not be read at all. SUSTAINED \
                   NON-ZERO is the wedge to alert on — reads hold the tenant mutex \
                   across a full replay, so a hung store never releases it. Equality \
                   with the served-tenant count is the subcase the freshness \
                   objective already reports, because nothing was observed; the \
                   partial case, where some tenants stay contended forever and the \
                   rest are fresh, is the one this gauge names directly — it is \
                   inferable from a fresh scrape with a non-zero unknown, but only \
                   this gauge says which half.",
            value: seen.contended,
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_projection_fresh",
            kind: "gauge",
            help: "1 when every tenant the process could read has consumed its whole \
                   log, 0 when any is behind or any log head was unreadable. A tenant \
                   that was merely BUSY does not zero this: a lock held by a request \
                   in flight is a service being used, and lag persists, so a tenant \
                   genuinely behind is seen on the scrapes that are not contended. \
                   /readyz answers the stricter question and fails closed on \
                   contention too, because one retried 503 is cheap where a spent \
                   error budget is not.",
            value: u64::from(seen.is_fresh()),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_projection_lag_unknown",
            kind: "gauge",
            help: "Served tenants whose lag could not be sampled, because the tenant \
                   was locked or its log head was unreadable. A tenant counted here \
                   contributes nothing to foundry_projection_lag, so a zero lag is \
                   only evidence of freshness while this is also zero. \
                   foundry_projection_contended splits out the busy half, so the \
                   remainder is the unreadable one.",
            value: seen.contended + seen.unreadable,
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_poisoned_entries",
            kind: "gauge",
            help: "Log entries the fold consumed and deterministically refused, summed \
                   over served tenants. Taken in the same pass as the lag, so \
                   foundry_projection_lag_unknown qualifies this total too: an \
                   unsampled tenant contributes nothing to either.",
            value: seen.poisoned,
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
            name: "foundry_action_invocation_answered_total",
            kind: "counter",
            help: "Accepted single-Action invocations, timed from handler entry to \
                   answer. Migration runs are not timed here.",
            value: metrics.invocation_answered.load(Ordering::Relaxed),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: INVOCATION_WITHIN_BUDGET_SERIES,
            kind: "counter",
            help: "The subset of foundry_action_invocation_answered_total answered \
                   within the 250 ms budget.",
            value: metrics
                .invocation_answered_within_budget
                .load(Ordering::Relaxed),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_denial_issued_total",
            kind: "counter",
            help: "Refusals the writer issued on the single-Action surface. Every \
                   other refusal there, before the writer or by the log after its \
                   gates, is counted only in foundry_action_submit_refused_total.",
            value: metrics.denial_issued.load(Ordering::Relaxed),
            objective_eligible: true,
            ineligible_because: "",
        },
        Sample {
            name: "foundry_denial_recorded_total",
            kind: "counter",
            help: "The subset of foundry_denial_issued_total the denial trail holds. \
                   The gap is refusals that stood whose record the trail does not hold.",
            value: metrics.denial_recorded.load(Ordering::Relaxed),
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

pub fn objective_eligible_metrics(state: &AppState) -> BTreeSet<&'static str> {
    samples(state)
        .into_iter()
        .filter(|sample| sample.objective_eligible)
        .map(|sample| sample.name)
        .collect()
}

pub fn objective_ineligible_metrics(state: &AppState) -> Vec<(&'static str, &'static str)> {
    samples(state)
        .into_iter()
        .filter(|sample| !sample.objective_eligible)
        .map(|sample| (sample.name, sample.ineligible_because))
        .collect()
}

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
