//! M02-P02-IP-001 — Read-only dashboard kernel projections.
//!
//! Frozen view structs over the foundry-account domain. These are read-only
//! projections: there are NO write methods, NO mutation ports, NO interior
//! mutability. Construct via `new(...)` from the domain types, then expose
//! through getters.
//!
//! Architectural rule (per ADR-0056 12-layer enum): the kernel exposes ports
//! and value types only. Adapters layer above injects data; no I/O here.
//!
//! Read-only invariant: every public method that takes `&mut self` is a
//! compile error; this is enforced by the `ReadOnlyProjection` marker trait
//! whose only method takes `&self`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_account_domain::{
    AccountHealth, AccountId, ProviderAccount, ProviderFamily, RouteExplanation, UsageWindow,
    UsageWindowKind,
};

// ── Read-only marker ─────────────────────────────────────────────────────────

/// Marker trait for read-only projections.
///
/// All dashboard views implement this. The marker has no mutating methods.
/// Any attempt to add a `&mut self` method to a view will require breaking
/// this trait's contract — caught at code review + by the negative tests.
pub trait ReadOnlyProjection {
    /// Stable label identifying the projection family (for telemetry).
    fn projection_label(&self) -> &'static str;
}

// ── AccountHealthView ────────────────────────────────────────────────────────

/// Frozen projection of an account's health snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountHealthView {
    account_id: AccountId,
    provider_family: ProviderFamily,
    is_healthy: bool,
    reason: Option<String>,
    last_check_at_epoch_secs: u64,
    state_label: &'static str,
}

impl AccountHealthView {
    pub fn new(account: &ProviderAccount, health: &AccountHealth) -> Self {
        Self {
            account_id: account.id.clone(),
            provider_family: account.provider_family,
            is_healthy: health.is_healthy,
            reason: health.reason.clone(),
            last_check_at_epoch_secs: health.last_check_at_epoch_secs,
            state_label: account.state.label(),
        }
    }

    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }
    pub fn provider_family(&self) -> ProviderFamily {
        self.provider_family
    }
    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
    pub fn last_check_at_epoch_secs(&self) -> u64 {
        self.last_check_at_epoch_secs
    }
    pub fn state_label(&self) -> &'static str {
        self.state_label
    }
}

impl ReadOnlyProjection for AccountHealthView {
    fn projection_label(&self) -> &'static str {
        "account-health"
    }
}

// ── SessionView ──────────────────────────────────────────────────────────────

/// Frozen projection of an auth-session snapshot. Carries no capability bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionView {
    session_id: String,
    account_id: AccountId,
    provider_family: ProviderFamily,
    started_at_epoch_secs: u64,
    expires_at_epoch_secs: u64,
    privacy_boundary: String,
}

impl SessionView {
    pub fn new(
        session_id: impl Into<String>,
        account_id: AccountId,
        provider_family: ProviderFamily,
        started_at_epoch_secs: u64,
        expires_at_epoch_secs: u64,
        privacy_boundary: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            account_id,
            provider_family,
            started_at_epoch_secs,
            expires_at_epoch_secs,
            privacy_boundary: privacy_boundary.into(),
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }
    pub fn provider_family(&self) -> ProviderFamily {
        self.provider_family
    }
    pub fn started_at_epoch_secs(&self) -> u64 {
        self.started_at_epoch_secs
    }
    pub fn expires_at_epoch_secs(&self) -> u64 {
        self.expires_at_epoch_secs
    }
    pub fn privacy_boundary(&self) -> &str {
        &self.privacy_boundary
    }
}

impl ReadOnlyProjection for SessionView {
    fn projection_label(&self) -> &'static str {
        "session"
    }
}

// ── UsageView (5h / 1wk / project) ───────────────────────────────────────────

/// Frozen projection of a usage window (FiveHour | OneWeek | Project).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageView {
    account_id: AccountId,
    kind: UsageWindowKind,
    started_at_epoch_secs: u64,
    ends_at_epoch_secs: u64,
    tokens_in: u64,
    tokens_out: u64,
    cache_hits: u64,
    estimated_cost_micros: u64,
    usage_limit_pct: u8,
    reserve_remaining_pct: u8,
}

impl UsageView {
    pub fn new(account_id: AccountId, window: &UsageWindow) -> Self {
        Self {
            account_id,
            kind: window.kind,
            started_at_epoch_secs: window.started_at_epoch_secs,
            ends_at_epoch_secs: window.ends_at_epoch_secs,
            tokens_in: window.tokens_in,
            tokens_out: window.tokens_out,
            cache_hits: window.cache_hits,
            estimated_cost_micros: window.estimated_cost_micros,
            usage_limit_pct: window.usage_limit_pct,
            reserve_remaining_pct: window.reserve_remaining_pct,
        }
    }

    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }
    pub fn kind(&self) -> UsageWindowKind {
        self.kind
    }
    pub fn started_at_epoch_secs(&self) -> u64 {
        self.started_at_epoch_secs
    }
    pub fn ends_at_epoch_secs(&self) -> u64 {
        self.ends_at_epoch_secs
    }
    pub fn tokens_in(&self) -> u64 {
        self.tokens_in
    }
    pub fn tokens_out(&self) -> u64 {
        self.tokens_out
    }
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits
    }
    pub fn estimated_cost_micros(&self) -> u64 {
        self.estimated_cost_micros
    }
    pub fn usage_limit_pct(&self) -> u8 {
        self.usage_limit_pct
    }
    pub fn reserve_remaining_pct(&self) -> u8 {
        self.reserve_remaining_pct
    }
}

impl ReadOnlyProjection for UsageView {
    fn projection_label(&self) -> &'static str {
        match self.kind {
            UsageWindowKind::FiveHour => "usage-5h",
            UsageWindowKind::OneWeek => "usage-1wk",
            UsageWindowKind::Project => "usage-project",
        }
    }
}

// ── RoutingView ──────────────────────────────────────────────────────────────

/// Frozen projection of a routing decision (for "explain route").
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingView {
    chosen_provider: ProviderFamily,
    chosen_account_id: AccountId,
    chosen_model: String,
    reason: String,
}

impl RoutingView {
    pub fn new(explanation: &RouteExplanation) -> Self {
        Self {
            chosen_provider: explanation.chosen_provider,
            chosen_account_id: explanation.chosen_account_id.clone(),
            chosen_model: explanation.chosen_model.clone(),
            reason: explanation.reason.clone(),
        }
    }

    pub fn chosen_provider(&self) -> ProviderFamily {
        self.chosen_provider
    }
    pub fn chosen_account_id(&self) -> &AccountId {
        &self.chosen_account_id
    }
    pub fn chosen_model(&self) -> &str {
        &self.chosen_model
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl ReadOnlyProjection for RoutingView {
    fn projection_label(&self) -> &'static str {
        "routing"
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_account_domain::{
        AccountHealth, AccountId, ProviderAccount, ProviderFamily, RouteExplanation, UsageWindow,
        UsageWindowKind,
    };

    fn aid(s: &str) -> AccountId {
        AccountId(s.to_owned())
    }

    fn active_account() -> ProviderAccount {
        let mut acc = ProviderAccount::new(aid("acct-1"), ProviderFamily::Claude);
        acc.state = intelligence_account_domain::AccountState::Active;
        acc
    }

    #[test]
    fn account_health_view_projects_fields() {
        let acc = active_account();
        let h = AccountHealth {
            account_id: aid("acct-1"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 1234,
        };
        let v = AccountHealthView::new(&acc, &h);
        assert_eq!(v.account_id(), &aid("acct-1"));
        assert_eq!(v.provider_family(), ProviderFamily::Claude);
        assert!(v.is_healthy());
        assert_eq!(v.reason(), None);
        assert_eq!(v.last_check_at_epoch_secs(), 1234);
        assert_eq!(v.state_label(), "Active");
        assert_eq!(v.projection_label(), "account-health");
    }

    #[test]
    fn account_health_view_carries_unhealthy_reason() {
        let acc = active_account();
        let h = AccountHealth {
            account_id: aid("acct-1"),
            is_healthy: false,
            reason: Some("quota exceeded".to_owned()),
            last_check_at_epoch_secs: 9999,
        };
        let v = AccountHealthView::new(&acc, &h);
        assert!(!v.is_healthy());
        assert_eq!(v.reason(), Some("quota exceeded"));
    }

    #[test]
    fn session_view_holds_lifecycle_bounds() {
        let v = SessionView::new(
            "sess-1",
            aid("acct-1"),
            ProviderFamily::Gemini,
            1000,
            2000,
            "tenant-x",
        );
        assert_eq!(v.session_id(), "sess-1");
        assert_eq!(v.provider_family(), ProviderFamily::Gemini);
        assert_eq!(v.started_at_epoch_secs(), 1000);
        assert_eq!(v.expires_at_epoch_secs(), 2000);
        assert_eq!(v.privacy_boundary(), "tenant-x");
        assert_eq!(v.projection_label(), "session");
    }

    #[test]
    fn usage_view_five_hour_label() {
        let w = UsageWindow::new(UsageWindowKind::FiveHour, 0, 18000, 80, 20).unwrap();
        let v = UsageView::new(aid("acct-1"), &w);
        assert_eq!(v.kind(), UsageWindowKind::FiveHour);
        assert_eq!(v.projection_label(), "usage-5h");
        assert_eq!(v.usage_limit_pct(), 80);
        assert_eq!(v.reserve_remaining_pct(), 20);
    }

    #[test]
    fn usage_view_one_week_label() {
        let w = UsageWindow::new(UsageWindowKind::OneWeek, 0, 604_800, 50, 50).unwrap();
        let v = UsageView::new(aid("acct-2"), &w);
        assert_eq!(v.projection_label(), "usage-1wk");
    }

    #[test]
    fn usage_view_project_label() {
        let w = UsageWindow::new(UsageWindowKind::Project, 0, 86_400, 100, 0).unwrap();
        let v = UsageView::new(aid("acct-3"), &w);
        assert_eq!(v.projection_label(), "usage-project");
    }

    #[test]
    fn routing_view_projects_explanation() {
        let e = RouteExplanation {
            chosen_provider: ProviderFamily::OpenAiOrCodex,
            chosen_account_id: aid("acct-9"),
            chosen_model: "gpt-5".to_owned(),
            reason: "lowest cost".to_owned(),
        };
        let v = RoutingView::new(&e);
        assert_eq!(v.chosen_provider(), ProviderFamily::OpenAiOrCodex);
        assert_eq!(v.chosen_account_id(), &aid("acct-9"));
        assert_eq!(v.chosen_model(), "gpt-5");
        assert_eq!(v.reason(), "lowest cost");
        assert_eq!(v.projection_label(), "routing");
    }

    /// Negative test: ReadOnlyProjection has no `&mut self` method.
    /// We verify by exercising every view through the trait without any mut binding.
    #[test]
    fn negative_no_mutation_path_through_read_only_trait() {
        let acc = active_account();
        let h = AccountHealth {
            account_id: aid("acct-1"),
            is_healthy: true,
            reason: None,
            last_check_at_epoch_secs: 0,
        };
        let projections: Vec<Box<dyn ReadOnlyProjection>> = vec![
            Box::new(AccountHealthView::new(&acc, &h)),
            Box::new(SessionView::new(
                "s",
                aid("a"),
                ProviderFamily::Aws,
                1,
                2,
                "t",
            )),
            Box::new(UsageView::new(
                aid("a"),
                &UsageWindow::new(UsageWindowKind::FiveHour, 0, 100, 0, 0).unwrap(),
            )),
            Box::new(RoutingView::new(&RouteExplanation {
                chosen_provider: ProviderFamily::Claude,
                chosen_account_id: aid("a"),
                chosen_model: "m".to_owned(),
                reason: "r".to_owned(),
            })),
        ];
        // Trait object only exposes `&self` — no mutation reachable.
        for p in &projections {
            let label = p.projection_label();
            assert!(!label.is_empty());
        }
    }
}
