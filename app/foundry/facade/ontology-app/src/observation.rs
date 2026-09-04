//! What the process can currently say about its own tenants.
//!
//! Owned here rather than by the exposition, because two surfaces consume it
//! for different reasons: `/readyz` decides whether to serve, and `/metrics`
//! reports. Putting it in `metrics` made a serving decision depend on the
//! Prometheus surface and closed a module cycle; the observation is neither
//! module's private business.

use crate::composition::AppState;

/// One pass over the served tenants: both totals and the number of tenants
/// that could not be read, taken TOGETHER.
///
/// Together is the whole point. An earlier revision computed each gauge in
/// its own pass, so a tenant unreadable during one pass and readable during
/// the next produced `lag 0` beside `unknown 0` — a pair that reads as
/// healthy and describes no state the process was ever in. Three passes also
/// meant three `MAX(ordinal)` queries per tenant per unauthenticated scrape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation {
    /// Tenants whose status was actually read on this pass. A total that
    /// nobody contributed to is not a small total; it is no measurement.
    pub observed: u64, // data_class: INTERNAL_ONLY
    pub lag: u64,      // data_class: INTERNAL_ONLY
    pub poisoned: u64, // data_class: INTERNAL_ONLY
    /// Tenants whose mutex was held when the pass ran — a request in flight,
    /// which is a healthy process being used. Counted separately from
    /// `unreadable` because the two mean opposite things: this one says the
    /// tenant was busy, not that anything is wrong with it.
    pub contended: u64, // data_class: INTERNAL_ONLY
    /// Tenants whose log head could not be read at all. This is the store
    /// failing, and a zero lag in its place is a claim of health made at the
    /// moment the process is least able to make it.
    pub unreadable: u64, // data_class: INTERNAL_ONLY
}

impl Observation {
    /// Caught up AND completely observed — the readiness predicate itself,
    /// evaluated from the same KIND of observation the exposition renders:
    /// one pass per answer, so no single answer can contradict itself. The
    /// probe and a scrape are separate requests taking separate passes, and
    /// they may of course differ — that is time passing, not disagreement.
    ///
    /// A tenant nobody could read is not evidence of freshness, so it is not
    /// readiness either. Poison does NOT enter it: a poisoned entry advances
    /// the fold and touches nothing else, so counting it as un-ready would
    /// red the instrument exactly when the system is making progress.
    pub fn is_caught_up(&self) -> bool {
        self.lag == 0 && self.contended == 0 && self.unreadable == 0
    }

    /// Fresh: caught up as far as anything could be read, ignoring tenants
    /// that were merely BUSY.
    ///
    /// Readiness and freshness diverge here deliberately, and the reason is
    /// the cost of being wrong in each direction. A contended lock means a
    /// request is in flight — a healthy process under load — so failing
    /// closed on it costs `/readyz` one cheap 503 that the next probe
    /// retries, but costs an error budget a share of every scrape taken
    /// while the service is busy. At 99.9% over thirty days that is
    /// forty-three minutes spent on concurrency rather than staleness, and
    /// an objective that reds because the service is being used teaches
    /// operators to ignore it.
    ///
    /// Failing OPEN on contention is safe here in a way it would not be for
    /// a transient signal, because lag persists: a projection behind its log
    /// stays behind until it catches up, so it is observed on the scrapes
    /// that are not contended. An unreadable head is different and still
    /// fails closed — a store nobody can read is not a fresh one.
    ///
    /// That argument requires uncontended scrapes to EXIST, so a tenant may
    /// be skipped for being busy but the whole roster may not. Reads hold
    /// the tenant mutex across a full replay, so a hung store holds it
    /// indefinitely and every pass sees contention; "every tenant we could
    /// read is caught up" is then true of the empty set, and a wedged
    /// process would score fresh forever while `/readyz` refused every
    /// probe. `observed > 0` is what stops a vacuous universal from being
    /// read as health — the same reason `unreadable` fails closed, one level
    /// down.
    pub fn is_fresh(&self) -> bool {
        self.observed > 0 && self.lag == 0 && self.unreadable == 0
    }
}

/// Read every served tenant once.
pub fn observe(state: &AppState) -> Observation {
    let mut seen = Observation {
        observed: 0,
        lag: 0,
        poisoned: 0,
        contended: 0,
        unreadable: 0,
    };
    for (tenant_id, tenant) in &state.tenants {
        let Ok(tenant) = tenant.try_lock() else {
            seen.contended += 1;
            continue;
        };
        match tenant.sync_status(tenant_id) {
            Ok(status) => {
                seen.observed += 1;
                seen.lag += status.lag;
                seen.poisoned += status.poisoned_count;
            }
            Err(_) => seen.unreadable += 1,
        }
    }
    seen
}
