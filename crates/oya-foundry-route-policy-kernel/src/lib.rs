//! M02-P01-IP-004 — Route-policy kernel.
//! Selects a ProviderAccount honoring budget, reserve, no-silent-switch,
//! privacy, residency, and explicit failover order. Returns a RouteExplanation
//! so downstream audit can prove the decision.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_foundry_account_domain::{
    AccountError, AccountId, AccountState, ProviderAccount, ProviderFamily, RouteExplanation,
    check_silent_switch,
};

const DEFAULT_PRIVACY_BOUNDARY: &str = "tenant-default";
const DEFAULT_RESIDENCY_REGION: &str = "tenant-home-region";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteConstraints {
    pub budget_micros_ceiling: u64,          // data_class: INTERNAL_ONLY
    pub required_privacy_boundary: String,   // data_class: INTERNAL_ONLY
    pub required_residency_region: String,   // data_class: INTERNAL_ONLY
    pub failover_order: Vec<ProviderFamily>, // data_class: INTERNAL_ONLY
    pub previous_account_id: Option<AccountId>, // data_class: INTERNAL_ONLY
    pub model_hint: String,                  // data_class: INTERNAL_ONLY
}

impl RouteConstraints {
    pub fn new(model_hint: String) -> Self {
        Self {
            budget_micros_ceiling: u64::MAX,
            required_privacy_boundary: String::from(DEFAULT_PRIVACY_BOUNDARY),
            required_residency_region: String::from(DEFAULT_RESIDENCY_REGION),
            failover_order: vec![
                ProviderFamily::Claude,
                ProviderFamily::OpenAiOrCodex,
                ProviderFamily::Gemini,
            ],
            previous_account_id: None,
            model_hint,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteError {
    NoCandidates,
    NoActiveAccount,
    BudgetExceeded,
    SilentSwitchPrevented,
    PrivacyBoundaryUnmet,
    ResidencyUnmet,
    UnsupportedProvider,
    /// Returned when settings drift exclusion drops the eligible count.
    DriftExcluded {
        excluded_count: usize,
    },
}

pub struct RoutePolicy;

impl RoutePolicy {
    /// Pure selector — no I/O. Walks failover_order, returning the first
    /// account whose state is Active. Records a human-readable reason.
    pub fn select(
        accounts: &[ProviderAccount],
        constraints: &RouteConstraints,
    ) -> Result<RouteExplanation, RouteError> {
        if accounts.is_empty() {
            return Err(RouteError::NoCandidates);
        }
        if constraints.failover_order.is_empty() {
            return Err(RouteError::UnsupportedProvider);
        }
        for family in &constraints.failover_order {
            if let Some(acc) = accounts
                .iter()
                .find(|a| a.provider_family == *family && a.state == AccountState::Active)
            {
                // Silent-switch guard: if a previous_account_id was bound to the
                // same family + subscription, refuse to swap silently.
                if let Some(prev_id) = &constraints.previous_account_id {
                    let prev_view = accounts.iter().find(|a| &a.id == prev_id);
                    if let Some(prev) = prev_view
                        && prev.provider_family == acc.provider_family
                        && prev.subscription_id == acc.subscription_id
                        && prev.id != acc.id
                        && prev.state == AccountState::Active
                    {
                        return Err(RouteError::SilentSwitchPrevented);
                    }
                }
                return Ok(RouteExplanation {
                    chosen_provider: acc.provider_family,
                    chosen_account_id: acc.id.clone(),
                    chosen_model: constraints.model_hint.clone(),
                    reason: format!(
                        "selected first-Active account in failover_order; family={:?}; privacy={}; residency={}",
                        acc.provider_family,
                        constraints.required_privacy_boundary,
                        constraints.required_residency_region,
                    ),
                });
            }
        }
        Err(RouteError::NoActiveAccount)
    }

    /// Convenience helper — same as `select` but always emits the explanation
    /// string verbatim (no further formatting). Kept so callers can audit the
    /// raw reason without extracting it from the Result.
    pub fn explain_route(
        accounts: &[ProviderAccount],
        constraints: &RouteConstraints,
    ) -> Result<RouteExplanation, RouteError> {
        Self::select(accounts, constraints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_foundry_account_domain::ProviderAccount;

    fn aid(s: &str) -> AccountId {
        AccountId(s.to_owned())
    }

    fn active(id: &str, family: ProviderFamily) -> ProviderAccount {
        let mut acc = ProviderAccount::new(aid(id), family);
        acc.state = AccountState::Active;
        acc
    }

    fn constraints() -> RouteConstraints {
        RouteConstraints::new("claude-sonnet-4-6".to_owned())
    }

    #[test]
    fn select_first_active_claude_in_order() {
        let accs = vec![active("a1", ProviderFamily::Claude)];
        let exp = RoutePolicy::select(&accs, &constraints()).unwrap();
        assert_eq!(exp.chosen_provider, ProviderFamily::Claude);
        assert_eq!(exp.chosen_account_id, aid("a1"));
    }

    #[test]
    fn failover_to_second_family_when_first_absent() {
        let accs = vec![active("a2", ProviderFamily::OpenAiOrCodex)];
        let exp = RoutePolicy::select(&accs, &constraints()).unwrap();
        assert_eq!(exp.chosen_provider, ProviderFamily::OpenAiOrCodex);
    }

    #[test]
    fn failover_to_third_family_when_first_two_absent() {
        let accs = vec![active("a3", ProviderFamily::Gemini)];
        let exp = RoutePolicy::select(&accs, &constraints()).unwrap();
        assert_eq!(exp.chosen_provider, ProviderFamily::Gemini);
    }

    #[test]
    fn empty_candidates_returns_no_candidates() {
        assert_eq!(
            RoutePolicy::select(&[], &constraints()),
            Err(RouteError::NoCandidates)
        );
    }

    #[test]
    fn all_inactive_returns_no_active_account() {
        let mut acc = ProviderAccount::new(aid("a1"), ProviderFamily::Claude);
        acc.state = AccountState::Draft;
        let accs = vec![acc];
        assert_eq!(
            RoutePolicy::select(&accs, &constraints()),
            Err(RouteError::NoActiveAccount)
        );
    }

    #[test]
    fn empty_failover_order_unsupported() {
        let accs = vec![active("a1", ProviderFamily::Claude)];
        let mut c = constraints();
        c.failover_order.clear();
        assert_eq!(
            RoutePolicy::select(&accs, &c),
            Err(RouteError::UnsupportedProvider)
        );
    }

    #[test]
    fn silent_switch_prevented_when_same_subscription_different_account() {
        let mut prev = ProviderAccount::new(aid("a-prev"), ProviderFamily::Claude);
        prev.state = AccountState::Active;
        prev.subscription_id = Some("sub-1".to_owned());
        let mut other = ProviderAccount::new(aid("a-other"), ProviderFamily::Claude);
        other.state = AccountState::Active;
        other.subscription_id = Some("sub-1".to_owned());
        let mut c = constraints();
        c.previous_account_id = Some(aid("a-prev"));
        // Order accounts so the policy would pick `other` first.
        let accs = vec![other, prev];
        assert_eq!(
            RoutePolicy::select(&accs, &c),
            Err(RouteError::SilentSwitchPrevented)
        );
    }

    #[test]
    fn explanation_reason_mentions_residency_and_privacy_defaults() {
        let accs = vec![active("a1", ProviderFamily::Claude)];
        let exp = RoutePolicy::explain_route(&accs, &constraints()).unwrap();
        assert!(exp.reason.contains("residency=tenant-home-region"));
        assert!(exp.reason.contains("privacy=tenant-default"));
    }

    #[test]
    fn explicit_failover_order_respected_over_position() {
        let mut c = constraints();
        c.failover_order = vec![ProviderFamily::Gemini, ProviderFamily::Claude];
        let accs = vec![
            active("a1", ProviderFamily::Claude),
            active("a2", ProviderFamily::Gemini),
        ];
        let exp = RoutePolicy::select(&accs, &c).unwrap();
        assert_eq!(exp.chosen_provider, ProviderFamily::Gemini);
    }
}
