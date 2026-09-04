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
    pub lag: u64,      // data_class: INTERNAL_ONLY
    pub poisoned: u64, // data_class: INTERNAL_ONLY
    /// Served tenants whose status could not be read, because the tenant was
    /// locked or its log head was unreadable. Both mean NOT KNOWN, and a zero
    /// in their place is a claim of health made at the moment the process is
    /// least able to make it.
    pub unknown: u64, // data_class: INTERNAL_ONLY
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
        self.lag == 0 && self.unknown == 0
    }
}

/// Read every served tenant once.
pub fn observe(state: &AppState) -> Observation {
    let mut seen = Observation {
        lag: 0,
        poisoned: 0,
        unknown: 0,
    };
    for (tenant_id, tenant) in &state.tenants {
        match tenant
            .try_lock()
            .ok()
            .and_then(|tenant| tenant.sync_status(tenant_id).ok())
        {
            Some(status) => {
                seen.lag += status.lag;
                seen.poisoned += status.poisoned_count;
            }
            None => seen.unknown += 1,
        }
    }
    seen
}
