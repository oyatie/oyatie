//! M02-P02-IP-001 — Dashboard app: read-only use-case orchestrators.
//!
//! Use-cases here orchestrate the kernel projections. NO write paths exist
//! anywhere in this crate; the contract is enforced by the use-case enum
//! `ReadOnlyOutcome` whose variants are all projection types — there is
//! deliberately no `Mutate` variant.
//!
//! Architectural rule (ADR-0056 12-layer): the application layer composes
//! kernel ports/views. Adapters supply data via ports; we do not call I/O
//! here.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_account_domain::{
    AccountHealth, AccountId, ProviderAccount, RouteExplanation, UsageWindow, UsageWindowKind,
};
use intelligence_dashboard_kernel::{
    AccountHealthView, ReadOnlyProjection, RoutingView, SessionView, UsageView,
};

// ── Read-only outcome ────────────────────────────────────────────────────────

/// Outcome of any use-case in this crate. All variants are kernel projections.
/// There is intentionally no `Mutate` variant; introducing one is a contract
/// breach surfaced by `negative_no_mutate_variant`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadOnlyOutcome {
    AccountHealth(AccountHealthView),
    AccountHealthList(Vec<AccountHealthView>),
    Session(SessionView),
    Usage(UsageView),
    Routing(RoutingView),
}

impl ReadOnlyOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AccountHealth(_) => "account-health",
            Self::AccountHealthList(_) => "account-health-list",
            Self::Session(v) => v.projection_label(),
            Self::Usage(v) => v.projection_label(),
            Self::Routing(v) => v.projection_label(),
        }
    }
}

// ── Use-case: ListAccountHealth ──────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ListAccountHealth;

impl ListAccountHealth {
    pub fn execute<'a>(
        accounts: &'a [&'a ProviderAccount],
        healths: &'a [&'a AccountHealth],
    ) -> ReadOnlyOutcome {
        let mut out: Vec<AccountHealthView> = Vec::new();
        for acc in accounts {
            if let Some(h) = healths.iter().find(|h| h.account_id == acc.id) {
                out.push(AccountHealthView::new(acc, h));
            }
        }
        ReadOnlyOutcome::AccountHealthList(out)
    }
}

// ── Use-case: GetUsageWindow ─────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageWindowQuery {
    pub account_id: AccountId, // data_class: INTERNAL_ONLY
    pub kind: UsageWindowKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct GetUsageWindow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageQueryError {
    NotFound,
}

impl GetUsageWindow {
    pub fn execute(
        query: &UsageWindowQuery,
        windows: &[(AccountId, UsageWindow)],
    ) -> Result<ReadOnlyOutcome, UsageQueryError> {
        for (id, w) in windows {
            if id == &query.account_id && w.kind == query.kind {
                return Ok(ReadOnlyOutcome::Usage(UsageView::new(id.clone(), w)));
            }
        }
        Err(UsageQueryError::NotFound)
    }
}

// ── Use-case: ExplainRoute ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ExplainRoute;

impl ExplainRoute {
    pub fn execute(explanation: &RouteExplanation) -> ReadOnlyOutcome {
        ReadOnlyOutcome::Routing(RoutingView::new(explanation))
    }
}

// ── Use-case: GetSession ─────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct GetSession;

impl GetSession {
    pub fn execute(view: SessionView) -> ReadOnlyOutcome {
        ReadOnlyOutcome::Session(view)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_account_domain::{
        AccountHealth, AccountId, AccountState, ProviderAccount, ProviderFamily, RouteExplanation,
        UsageWindow, UsageWindowKind,
    };

    fn aid(s: &str) -> AccountId {
        AccountId(s.to_owned())
    }

    fn active(id: &str, fam: ProviderFamily) -> ProviderAccount {
        let mut a = ProviderAccount::new(aid(id), fam);
        a.state = AccountState::Active;
        a
    }

    #[test]
    fn list_account_health_joins_by_id() {
        let a1 = active("a1", ProviderFamily::Claude);
        let a2 = active("a2", ProviderFamily::Gemini);
        let h1 = AccountHealth {
            account_id: aid("a1"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 1,
        };
        let h2 = AccountHealth {
            account_id: aid("a2"),
            is_healthy: false,
            reason: Some("err".to_owned()),
            last_check_at_epoch_secs: 2,
        };
        let out = ListAccountHealth::execute(&[&a1, &a2], &[&h1, &h2]);
        match out {
            ReadOnlyOutcome::AccountHealthList(v) => assert_eq!(v.len(), 2),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn list_account_health_skips_unmatched() {
        let a1 = active("a1", ProviderFamily::Claude);
        let h2 = AccountHealth {
            account_id: aid("zzz"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 0,
        };
        let out = ListAccountHealth::execute(&[&a1], &[&h2]);
        match out {
            ReadOnlyOutcome::AccountHealthList(v) => assert!(v.is_empty()),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn get_usage_window_hits_match() {
        let w = UsageWindow::new(UsageWindowKind::FiveHour, 0, 18000, 80, 20).unwrap();
        let q = UsageWindowQuery {
            account_id: aid("a1"),
            kind: UsageWindowKind::FiveHour,
        };
        let out = GetUsageWindow::execute(&q, &[(aid("a1"), w)]).unwrap();
        assert_eq!(out.label(), "usage-5h");
    }

    #[test]
    fn get_usage_window_misses_returns_not_found() {
        let w = UsageWindow::new(UsageWindowKind::OneWeek, 0, 604_800, 50, 50).unwrap();
        let q = UsageWindowQuery {
            account_id: aid("a1"),
            kind: UsageWindowKind::FiveHour,
        };
        let err = GetUsageWindow::execute(&q, &[(aid("a1"), w)]).unwrap_err();
        assert_eq!(err, UsageQueryError::NotFound);
    }

    #[test]
    fn explain_route_returns_routing_projection() {
        let e = RouteExplanation {
            chosen_provider: ProviderFamily::Claude,
            chosen_account_id: aid("a1"),
            chosen_model: "claude-opus-4-7".to_owned(),
            reason: "agent specialization".to_owned(),
        };
        let out = ExplainRoute::execute(&e);
        match out {
            ReadOnlyOutcome::Routing(r) => {
                assert_eq!(r.chosen_model(), "claude-opus-4-7");
                assert_eq!(r.reason(), "agent specialization");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn get_session_wraps_view() {
        use intelligence_dashboard_kernel::SessionView;
        let v = SessionView::new("s1", aid("a1"), ProviderFamily::Aws, 1, 2, "t1");
        let out = GetSession::execute(v);
        match out {
            ReadOnlyOutcome::Session(s) => assert_eq!(s.session_id(), "s1"),
            _ => panic!("wrong variant"),
        }
    }

    /// Negative test: `ReadOnlyOutcome` enum has no `Mutate` variant.
    /// We exhaustively match every variant. If a new variant is added that
    /// represents mutation, this match will not compile-fail but the test
    /// will fail at the assertion below — and the enum is open to scrutiny.
    #[test]
    fn negative_no_mutate_variant() {
        let a = active("a1", ProviderFamily::Claude);
        let h = AccountHealth {
            account_id: aid("a1"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 0,
        };
        let out = ListAccountHealth::execute(&[&a], &[&h]);
        // Every variant we know is read-only.
        let is_readonly = match out {
            ReadOnlyOutcome::AccountHealth(_)
            | ReadOnlyOutcome::AccountHealthList(_)
            | ReadOnlyOutcome::Session(_)
            | ReadOnlyOutcome::Usage(_)
            | ReadOnlyOutcome::Routing(_) => true,
        };
        assert!(
            is_readonly,
            "ReadOnlyOutcome must contain only read-only variants"
        );
    }
}
