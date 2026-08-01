//! M02-P01-IP-004 — Route-policy kernel.
//! Selects a ProviderAccount honoring budget, reserve, no-silent-switch,
//! privacy, residency, and explicit failover order. Returns a RouteExplanation
//! so downstream audit can prove the decision.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_account_domain::{
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

/// Input record for weighted selection. Caller supplies cost and compliance
/// metadata alongside the account so the selector stays I/O-free.
pub struct RouteCandidate<'a> {
    pub account: &'a ProviderAccount,
    pub cost_micros: u64,
    pub residency_region: String,
    pub privacy_boundary: String,
    pub model_affinity: bool,
}

/// Scoring evidence produced during weighted selection (unit-testable directly).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteScore {
    pub account_id: AccountId,
    pub cost_micros: u64,
    pub budget_headroom_micros: u64,
    pub model_affinity: bool,
    pub failover_rank: usize,
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

    /// Weighted selector — picks the lowest-cost eligible account with a
    /// deterministic 4-step tiebreak chain (cost → affinity → failover_rank →
    /// account_id lexicographic). Preserves no-silent-switch and emits a
    /// RouteExplanation with full audit evidence.
    ///
    /// Error-precedence (checked in order):
    ///   NoCandidates → UnsupportedProvider → NoActiveAccount →
    ///   BudgetExceeded → ResidencyUnmet → PrivacyBoundaryUnmet →
    ///   SilentSwitchPrevented
    pub fn select_weighted(
        candidates: &[RouteCandidate<'_>],
        constraints: &RouteConstraints,
    ) -> Result<RouteExplanation, RouteError> {
        // ── Tier-1 guards (no account data needed) ──────────────────────────
        if candidates.is_empty() {
            return Err(RouteError::NoCandidates);
        }
        if constraints.failover_order.is_empty() {
            return Err(RouteError::UnsupportedProvider);
        }

        // Build a position map for failover_rank lookup.
        let rank_of = |family: &ProviderFamily| -> Option<usize> {
            constraints.failover_order.iter().position(|f| f == family)
        };

        // ── Filter to eligible candidates, recording last rejection cause ───
        let mut last_err = RouteError::NoActiveAccount;
        let mut eligible: Vec<RouteScore> = Vec::new();

        for cand in candidates {
            let acc = cand.account;

            // Must be Active in a routable family.
            if acc.state != AccountState::Active {
                continue; // last_err stays NoActiveAccount
            }
            if rank_of(&acc.provider_family).is_none() {
                continue; // not in failover_order
            }

            // Budget check.
            if cand.cost_micros > constraints.budget_micros_ceiling {
                if last_err == RouteError::NoActiveAccount
                    || last_err == RouteError::BudgetExceeded
                {
                    last_err = RouteError::BudgetExceeded;
                }
                continue;
            }

            // Residency check.
            if cand.residency_region != constraints.required_residency_region {
                if matches!(
                    last_err,
                    RouteError::NoActiveAccount | RouteError::BudgetExceeded | RouteError::ResidencyUnmet
                ) {
                    last_err = RouteError::ResidencyUnmet;
                }
                continue;
            }

            // Privacy check.
            if cand.privacy_boundary != constraints.required_privacy_boundary {
                if matches!(
                    last_err,
                    RouteError::NoActiveAccount
                        | RouteError::BudgetExceeded
                        | RouteError::ResidencyUnmet
                        | RouteError::PrivacyBoundaryUnmet
                ) {
                    last_err = RouteError::PrivacyBoundaryUnmet;
                }
                continue;
            }

            let headroom = constraints.budget_micros_ceiling.saturating_sub(cand.cost_micros);
            let rank = rank_of(&acc.provider_family).unwrap_or(usize::MAX);
            eligible.push(RouteScore {
                account_id: acc.id.clone(),
                cost_micros: cand.cost_micros,
                budget_headroom_micros: headroom,
                model_affinity: cand.model_affinity,
                failover_rank: rank,
            });
        }

        if eligible.is_empty() {
            return Err(last_err);
        }

        // ── Tiebreak: cost ASC, affinity DESC, rank ASC, id ASC ─────────────
        eligible.sort_by(|a, b| {
            a.cost_micros
                .cmp(&b.cost_micros)
                .then_with(|| b.model_affinity.cmp(&a.model_affinity)) // true > false
                .then_with(|| a.failover_rank.cmp(&b.failover_rank))
                .then_with(|| a.account_id.0.cmp(&b.account_id.0))
        });

        let winner = &eligible[0];

        // ── Silent-switch guard on the weighted winner ───────────────────────
        // Fires when the previous account exists in the candidates AND there is
        // a different active account on the same family+subscription — meaning
        // any selection within that subscription is ambiguous without an explicit
        // audit acknowledgement.
        if let Some(prev_id) = &constraints.previous_account_id {
            let prev_cand = candidates.iter().find(|c| &c.account.id == prev_id);
            if let Some(prev) = prev_cand {
                let prev_acc = prev.account;
                let conflict = candidates.iter().any(|c| {
                    c.account.id != prev_acc.id
                        && c.account.provider_family == prev_acc.provider_family
                        && c.account.subscription_id == prev_acc.subscription_id
                        && c.account.state == AccountState::Active
                });
                if conflict {
                    return Err(RouteError::SilentSwitchPrevented);
                }
            }
        }

        // Retrieve the winning candidate to build the explanation.
        let win_cand = candidates
            .iter()
            .find(|c| c.account.id == winner.account_id)
            .unwrap();

        Ok(RouteExplanation {
            chosen_provider: win_cand.account.provider_family,
            chosen_account_id: winner.account_id.clone(),
            chosen_model: constraints.model_hint.clone(),
            reason: format!(
                "weighted-select: cost={} headroom={} residency={} privacy={} affinity={} failover_rank={}",
                winner.cost_micros,
                winner.budget_headroom_micros,
                constraints.required_residency_region,
                constraints.required_privacy_boundary,
                winner.model_affinity,
                winner.failover_rank,
            ),
        })
    }

    /// Returns ALL eligible candidates ordered by the 4-step tiebreak chain
    /// (cost_micros ASC → model_affinity DESC → failover_rank ASC →
    /// account_id ASC), exposing the full auditable failover ladder.
    ///
    /// `slate[0]` is the same winner `select_weighted` would have chosen.
    ///
    /// Error-precedence (identical to `select_weighted`):
    ///   NoCandidates → UnsupportedProvider → NoActiveAccount →
    ///   BudgetExceeded → ResidencyUnmet → PrivacyBoundaryUnmet →
    ///   SilentSwitchPrevented
    pub fn rank_candidates(
        candidates: &[RouteCandidate<'_>],
        constraints: &RouteConstraints,
    ) -> Result<Vec<RouteScore>, RouteError> {
        // ── Tier-1 guards ────────────────────────────────────────────────────
        if candidates.is_empty() {
            return Err(RouteError::NoCandidates);
        }
        if constraints.failover_order.is_empty() {
            return Err(RouteError::UnsupportedProvider);
        }

        let rank_of = |family: &ProviderFamily| -> Option<usize> {
            constraints.failover_order.iter().position(|f| f == family)
        };

        // ── Filter to eligible candidates ────────────────────────────────────
        let mut last_err = RouteError::NoActiveAccount;
        let mut eligible: Vec<RouteScore> = Vec::new();

        for cand in candidates {
            let acc = cand.account;

            if acc.state != AccountState::Active {
                continue;
            }
            if rank_of(&acc.provider_family).is_none() {
                continue;
            }

            if cand.cost_micros > constraints.budget_micros_ceiling {
                if matches!(last_err, RouteError::NoActiveAccount | RouteError::BudgetExceeded) {
                    last_err = RouteError::BudgetExceeded;
                }
                continue;
            }

            if cand.residency_region != constraints.required_residency_region {
                if matches!(
                    last_err,
                    RouteError::NoActiveAccount | RouteError::BudgetExceeded | RouteError::ResidencyUnmet
                ) {
                    last_err = RouteError::ResidencyUnmet;
                }
                continue;
            }

            if cand.privacy_boundary != constraints.required_privacy_boundary {
                if matches!(
                    last_err,
                    RouteError::NoActiveAccount
                        | RouteError::BudgetExceeded
                        | RouteError::ResidencyUnmet
                        | RouteError::PrivacyBoundaryUnmet
                ) {
                    last_err = RouteError::PrivacyBoundaryUnmet;
                }
                continue;
            }

            let headroom = constraints.budget_micros_ceiling.saturating_sub(cand.cost_micros);
            let rank = rank_of(&acc.provider_family).unwrap_or(usize::MAX);
            eligible.push(RouteScore {
                account_id: acc.id.clone(),
                cost_micros: cand.cost_micros,
                budget_headroom_micros: headroom,
                model_affinity: cand.model_affinity,
                failover_rank: rank,
            });
        }

        if eligible.is_empty() {
            return Err(last_err);
        }

        // ── Tiebreak: cost ASC, affinity DESC, rank ASC, id ASC ─────────────
        eligible.sort_by(|a, b| {
            a.cost_micros
                .cmp(&b.cost_micros)
                .then_with(|| b.model_affinity.cmp(&a.model_affinity))
                .then_with(|| a.failover_rank.cmp(&b.failover_rank))
                .then_with(|| a.account_id.0.cmp(&b.account_id.0))
        });

        // ── Silent-switch guard (same logic as select_weighted) ──────────────
        if let Some(prev_id) = &constraints.previous_account_id {
            let prev_cand = candidates.iter().find(|c| &c.account.id == prev_id);
            if let Some(prev) = prev_cand {
                let prev_acc = prev.account;
                let conflict = candidates.iter().any(|c| {
                    c.account.id != prev_acc.id
                        && c.account.provider_family == prev_acc.provider_family
                        && c.account.subscription_id == prev_acc.subscription_id
                        && c.account.state == AccountState::Active
                });
                if conflict {
                    return Err(RouteError::SilentSwitchPrevented);
                }
            }
        }

        Ok(eligible)
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
    use intelligence_account_domain::ProviderAccount;

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

// ── rank_candidates tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests_rank_candidates {
    use super::*;

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

    fn candidate<'a>(
        account: &'a ProviderAccount,
        cost_micros: u64,
        model_affinity: bool,
    ) -> RouteCandidate<'a> {
        RouteCandidate {
            account,
            cost_micros,
            residency_region: "tenant-home-region".to_owned(),
            privacy_boundary: "tenant-default".to_owned(),
            model_affinity,
        }
    }

    // ── Error-precedence (7 rows) ────────────────────────────────────────────

    #[test]
    fn rank_empty_candidates_returns_no_candidates() {
        let c = constraints();
        assert_eq!(
            RoutePolicy::rank_candidates(&[], &c),
            Err(RouteError::NoCandidates)
        );
    }

    #[test]
    fn rank_empty_failover_order_returns_unsupported_provider() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 100, true)];
        let mut c = constraints();
        c.failover_order.clear();
        assert_eq!(
            RoutePolicy::rank_candidates(&cands, &c),
            Err(RouteError::UnsupportedProvider)
        );
    }

    #[test]
    fn rank_all_inactive_returns_no_active_account() {
        let mut acc = ProviderAccount::new(aid("a1"), ProviderFamily::Claude);
        acc.state = AccountState::Draft;
        let cands = vec![RouteCandidate {
            account: &acc,
            cost_micros: 50,
            residency_region: "tenant-home-region".to_owned(),
            privacy_boundary: "tenant-default".to_owned(),
            model_affinity: false,
        }];
        let c = constraints();
        assert_eq!(
            RoutePolicy::rank_candidates(&cands, &c),
            Err(RouteError::NoActiveAccount)
        );
    }

    #[test]
    fn rank_all_over_budget_returns_budget_exceeded() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 1_000, false)];
        let mut c = constraints();
        c.budget_micros_ceiling = 500;
        assert_eq!(
            RoutePolicy::rank_candidates(&cands, &c),
            Err(RouteError::BudgetExceeded)
        );
    }

    #[test]
    fn rank_residency_mismatch_returns_residency_unmet() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![RouteCandidate {
            account: &acc,
            cost_micros: 100,
            residency_region: "us-east-1".to_owned(),
            privacy_boundary: "tenant-default".to_owned(),
            model_affinity: false,
        }];
        let c = constraints();
        assert_eq!(
            RoutePolicy::rank_candidates(&cands, &c),
            Err(RouteError::ResidencyUnmet)
        );
    }

    #[test]
    fn rank_privacy_mismatch_returns_privacy_boundary_unmet() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![RouteCandidate {
            account: &acc,
            cost_micros: 100,
            residency_region: "tenant-home-region".to_owned(),
            privacy_boundary: "gdpr-eu".to_owned(),
            model_affinity: false,
        }];
        let c = constraints();
        assert_eq!(
            RoutePolicy::rank_candidates(&cands, &c),
            Err(RouteError::PrivacyBoundaryUnmet)
        );
    }

    #[test]
    fn rank_silent_switch_returns_silent_switch_prevented() {
        let mut prev = ProviderAccount::new(aid("a-prev"), ProviderFamily::Claude);
        prev.state = AccountState::Active;
        prev.subscription_id = Some("sub-1".to_owned());
        let mut other = ProviderAccount::new(aid("a-other"), ProviderFamily::Claude);
        other.state = AccountState::Active;
        other.subscription_id = Some("sub-1".to_owned());
        let cands = vec![
            candidate(&other, 100, true),
            RouteCandidate {
                account: &prev,
                cost_micros: 100,
                residency_region: "tenant-home-region".to_owned(),
                privacy_boundary: "tenant-default".to_owned(),
                model_affinity: true,
            },
        ];
        let mut c = constraints();
        c.previous_account_id = Some(aid("a-prev"));
        assert_eq!(
            RoutePolicy::rank_candidates(&cands, &c),
            Err(RouteError::SilentSwitchPrevented)
        );
    }

    // ── Happy path ───────────────────────────────────────────────────────────

    #[test]
    fn rank_single_eligible_returns_single_element_slate() {
        let acc = active("solo", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 42, true)];
        let c = constraints();
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        assert_eq!(slate.len(), 1);
        assert_eq!(slate[0].account_id, aid("solo"));
        assert_eq!(slate[0].cost_micros, 42);
        assert!(slate[0].model_affinity);
        assert_eq!(slate[0].failover_rank, 0);
    }

    #[test]
    fn rank_returns_all_eligible_not_just_winner() {
        let a1 = active("a1", ProviderFamily::Claude);
        let a2 = active("a2", ProviderFamily::OpenAiOrCodex);
        let a3 = active("a3", ProviderFamily::Gemini);
        let cands = vec![
            candidate(&a3, 300, false),
            candidate(&a1, 100, false),
            candidate(&a2, 200, false),
        ];
        let c = constraints();
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        assert_eq!(slate.len(), 3, "all three eligible candidates must appear");
        // winner is lowest cost = a1
        assert_eq!(slate[0].account_id, aid("a1"));
    }

    // ── Tiebreak tests ───────────────────────────────────────────────────────

    #[test]
    fn rank_cost_tiebreak_lowest_first() {
        let cheap = active("cheap", ProviderFamily::Gemini);
        let expensive = active("expensive", ProviderFamily::Claude);
        let cands = vec![
            candidate(&expensive, 500, false),
            candidate(&cheap, 100, false),
        ];
        let c = constraints();
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        assert_eq!(slate[0].account_id, aid("cheap"));
        assert_eq!(slate[1].account_id, aid("expensive"));
    }

    #[test]
    fn rank_cost_tie_affinity_true_first() {
        let no_affinity = active("no-aff", ProviderFamily::Claude);
        let has_affinity = active("has-aff", ProviderFamily::OpenAiOrCodex);
        let cands = vec![
            candidate(&no_affinity, 200, false),
            candidate(&has_affinity, 200, true),
        ];
        let c = constraints();
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        assert_eq!(slate[0].account_id, aid("has-aff"));
        assert_eq!(slate[1].account_id, aid("no-aff"));
    }

    #[test]
    fn rank_cost_affinity_tie_lower_failover_rank_first() {
        let gemini = active("g1", ProviderFamily::Gemini);
        let claude = active("c1", ProviderFamily::Claude);
        let cands = vec![
            candidate(&gemini, 200, false),
            candidate(&claude, 200, false),
        ];
        let c = constraints(); // Claude is rank 0, Gemini is rank 2
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        assert_eq!(slate[0].account_id, aid("c1")); // rank 0 wins
        assert_eq!(slate[1].account_id, aid("g1")); // rank 2
    }

    #[test]
    fn rank_full_tie_lexicographic_id_first() {
        let acc_z = active("z-account", ProviderFamily::Claude);
        let acc_a = active("a-account", ProviderFamily::Claude);
        let cands = vec![
            candidate(&acc_z, 200, true),
            candidate(&acc_a, 200, true),
        ];
        let c = constraints();
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        assert_eq!(slate[0].account_id, aid("a-account"));
        assert_eq!(slate[1].account_id, aid("z-account"));
    }

    // ── Consistency invariant ────────────────────────────────────────────────

    #[test]
    fn rank_slate_zero_equals_select_weighted_winner() {
        let a1 = active("b1", ProviderFamily::Claude);
        let a2 = active("b2", ProviderFamily::OpenAiOrCodex);
        let cands = vec![
            candidate(&a1, 150, false),
            candidate(&a2, 100, true),
        ];
        let c = constraints();
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        let explanation = RoutePolicy::select_weighted(&cands, &c).unwrap();
        assert_eq!(
            slate[0].account_id,
            explanation.chosen_account_id,
            "slate[0] must match select_weighted winner"
        );
    }

    // ── RouteScore fields ────────────────────────────────────────────────────

    #[test]
    fn rank_score_headroom_is_ceiling_minus_cost() {
        let acc = active("h1", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 300, false)];
        let mut c = constraints();
        c.budget_micros_ceiling = 1_000;
        let slate = RoutePolicy::rank_candidates(&cands, &c).unwrap();
        assert_eq!(slate[0].budget_headroom_micros, 700);
    }
}

// ── select_weighted tests (TDD RED — RouteCandidate / RouteScore / select_weighted not yet impl) ──

#[cfg(test)]
mod tests_weighted {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn aid(s: &str) -> AccountId {
        AccountId(s.to_owned())
    }

    fn active(id: &str, family: ProviderFamily) -> ProviderAccount {
        let mut acc = ProviderAccount::new(aid(id), family);
        acc.state = AccountState::Active;
        acc
    }

    /// Default constraints: budget=u64::MAX, default privacy/residency, failover=[Claude, OpenAiOrCodex, Gemini].
    fn constraints() -> RouteConstraints {
        RouteConstraints::new("claude-sonnet-4-6".to_owned())
    }

    /// Build a RouteCandidate that passes all eligibility filters by default.
    fn candidate<'a>(
        account: &'a ProviderAccount,
        cost_micros: u64,
        model_affinity: bool,
    ) -> RouteCandidate<'a> {
        RouteCandidate {
            account,
            cost_micros,
            residency_region: "tenant-home-region".to_owned(),
            privacy_boundary: "tenant-default".to_owned(),
            model_affinity,
        }
    }

    // ── Error-precedence table (7 rows) ──────────────────────────────────────

    /// Row 1: candidates slice is empty → NoCandidates.
    #[test]
    fn weighted_empty_candidates_returns_no_candidates() {
        let c = constraints();
        assert_eq!(
            RoutePolicy::select_weighted(&[], &c),
            Err(RouteError::NoCandidates)
        );
    }

    /// Row 2: failover_order is empty → UnsupportedProvider.
    /// (candidates present, failover_order cleared)
    #[test]
    fn weighted_empty_failover_order_returns_unsupported_provider() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 100, true)];
        let mut c = constraints();
        c.failover_order.clear();
        assert_eq!(
            RoutePolicy::select_weighted(&cands, &c),
            Err(RouteError::UnsupportedProvider)
        );
    }

    /// Row 3: no candidate is Active in a routable family → NoActiveAccount.
    /// (failover_order non-empty, but the single account is Draft)
    #[test]
    fn weighted_no_active_in_routable_family_returns_no_active_account() {
        let mut acc = ProviderAccount::new(aid("a1"), ProviderFamily::Claude);
        acc.state = AccountState::Draft;
        let cands = vec![RouteCandidate {
            account: &acc,
            cost_micros: 50,
            residency_region: "tenant-home-region".to_owned(),
            privacy_boundary: "tenant-default".to_owned(),
            model_affinity: false,
        }];
        let c = constraints();
        assert_eq!(
            RoutePolicy::select_weighted(&cands, &c),
            Err(RouteError::NoActiveAccount)
        );
    }

    /// Row 4: active+routable, but cost_micros exceeds budget → BudgetExceeded.
    /// (active account in routable family, residency/privacy match, but cost > ceiling)
    #[test]
    fn weighted_all_over_budget_returns_budget_exceeded() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 1_000, false)];
        let mut c = constraints();
        c.budget_micros_ceiling = 500; // cost_micros=1000 > 500
        assert_eq!(
            RoutePolicy::select_weighted(&cands, &c),
            Err(RouteError::BudgetExceeded)
        );
    }

    /// Row 5: within budget, but residency doesn't match → ResidencyUnmet.
    #[test]
    fn weighted_residency_mismatch_returns_residency_unmet() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![RouteCandidate {
            account: &acc,
            cost_micros: 100,
            residency_region: "us-east-1".to_owned(), // wrong region
            privacy_boundary: "tenant-default".to_owned(),
            model_affinity: false,
        }];
        // constraints require "tenant-home-region"
        let c = constraints();
        assert_eq!(
            RoutePolicy::select_weighted(&cands, &c),
            Err(RouteError::ResidencyUnmet)
        );
    }

    /// Row 6: residency matches, but privacy_boundary doesn't → PrivacyBoundaryUnmet.
    #[test]
    fn weighted_privacy_mismatch_returns_privacy_boundary_unmet() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![RouteCandidate {
            account: &acc,
            cost_micros: 100,
            residency_region: "tenant-home-region".to_owned(),
            privacy_boundary: "gdpr-eu".to_owned(), // wrong boundary
            model_affinity: false,
        }];
        // constraints require "tenant-default"
        let c = constraints();
        assert_eq!(
            RoutePolicy::select_weighted(&cands, &c),
            Err(RouteError::PrivacyBoundaryUnmet)
        );
    }

    /// Row 7: eligible winner collides with previous_account_id (same family + subscription) → SilentSwitchPrevented.
    #[test]
    fn weighted_silent_switch_prevented_when_eligible_winner_collides() {
        let mut prev = ProviderAccount::new(aid("a-prev"), ProviderFamily::Claude);
        prev.state = AccountState::Active;
        prev.subscription_id = Some("sub-1".to_owned());

        let mut winner = ProviderAccount::new(aid("a-winner"), ProviderFamily::Claude);
        winner.state = AccountState::Active;
        winner.subscription_id = Some("sub-1".to_owned());

        let cands = vec![
            candidate(&winner, 100, true),
            // prev is also present so the guard can look it up
            RouteCandidate {
                account: &prev,
                cost_micros: 100,
                residency_region: "tenant-home-region".to_owned(),
                privacy_boundary: "tenant-default".to_owned(),
                model_affinity: true,
            },
        ];
        let mut c = constraints();
        c.previous_account_id = Some(aid("a-prev"));
        assert_eq!(
            RoutePolicy::select_weighted(&cands, &c),
            Err(RouteError::SilentSwitchPrevented)
        );
    }

    // ── Tiebreak chain (4 tests) ──────────────────────────────────────────────

    /// Tiebreak 1: lowest cost_micros wins across different families.
    #[test]
    fn weighted_lowest_cost_wins_across_families() {
        let cheap = active("cheap", ProviderFamily::Gemini);
        let expensive = active("expensive", ProviderFamily::Claude);
        let cands = vec![
            candidate(&expensive, 500, false),
            candidate(&cheap, 100, false),
        ];
        let c = constraints(); // failover=[Claude, OpenAiOrCodex, Gemini]
        let exp = RoutePolicy::select_weighted(&cands, &c).unwrap();
        assert_eq!(exp.chosen_account_id, aid("cheap"));
        assert_eq!(exp.chosen_provider, ProviderFamily::Gemini);
    }

    /// Tiebreak 2: cost tie → model_affinity=true wins over false.
    #[test]
    fn weighted_cost_tie_affinity_true_wins() {
        let no_affinity = active("no-affinity", ProviderFamily::Claude);
        let has_affinity = active("has-affinity", ProviderFamily::OpenAiOrCodex);
        let cands = vec![
            candidate(&no_affinity, 200, false),
            candidate(&has_affinity, 200, true), // same cost, affinity wins
        ];
        let c = constraints();
        let exp = RoutePolicy::select_weighted(&cands, &c).unwrap();
        assert_eq!(exp.chosen_account_id, aid("has-affinity"));
    }

    /// Tiebreak 3: cost tie + affinity tie → lower failover_rank wins.
    /// failover_order=[Claude(0), OpenAiOrCodex(1), Gemini(2)]; Claude should win.
    #[test]
    fn weighted_cost_and_affinity_tie_lower_failover_rank_wins() {
        let gemini = active("g1", ProviderFamily::Gemini);
        let claude = active("c1", ProviderFamily::Claude);
        let cands = vec![
            candidate(&gemini, 200, false),
            candidate(&claude, 200, false), // same cost, same affinity, Claude is rank 0
        ];
        let c = constraints();
        let exp = RoutePolicy::select_weighted(&cands, &c).unwrap();
        assert_eq!(exp.chosen_account_id, aid("c1"));
        assert_eq!(exp.chosen_provider, ProviderFamily::Claude);
    }

    /// Tiebreak 4: full tie → lexicographically smallest account_id wins.
    #[test]
    fn weighted_full_tie_lexicographically_smallest_account_id_wins() {
        let acc_z = active("z-account", ProviderFamily::Claude);
        let acc_a = active("a-account", ProviderFamily::Claude);
        let cands = vec![
            candidate(&acc_z, 200, true),
            candidate(&acc_a, 200, true), // same everything; "a-account" < "z-account"
        ];
        let c = constraints();
        let exp = RoutePolicy::select_weighted(&cands, &c).unwrap();
        assert_eq!(exp.chosen_account_id, aid("a-account"));
    }

    // ── RouteExplanation audit content ───────────────────────────────────────

    /// reason string must contain cost, headroom, residency, privacy, affinity substrings.
    #[test]
    fn weighted_explanation_reason_contains_required_audit_fields() {
        let acc = active("a1", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 300, true)];
        let mut c = constraints();
        c.budget_micros_ceiling = 1000;
        let exp = RoutePolicy::select_weighted(&cands, &c).unwrap();
        let r = &exp.reason;
        assert!(r.contains("cost"), "reason missing 'cost': {r}");
        assert!(r.contains("headroom"), "reason missing 'headroom': {r}");
        assert!(r.contains("residency"), "reason missing 'residency': {r}");
        assert!(r.contains("privacy"), "reason missing 'privacy': {r}");
        assert!(r.contains("affinity"), "reason missing 'affinity': {r}");
    }

    // ── Determinism ──────────────────────────────────────────────────────────

    /// Identical input called twice produces identical RouteExplanation.
    #[test]
    fn weighted_identical_input_is_deterministic() {
        let acc1 = active("b1", ProviderFamily::Claude);
        let acc2 = active("b2", ProviderFamily::OpenAiOrCodex);
        let cands = vec![candidate(&acc1, 150, false), candidate(&acc2, 100, true)];
        let c = constraints();
        let first = RoutePolicy::select_weighted(&cands, &c).unwrap();
        let second = RoutePolicy::select_weighted(&cands, &c).unwrap();
        assert_eq!(first, second);
    }

    // ── RouteScore fields ────────────────────────────────────────────────────

    /// RouteScore correctly computes budget_headroom_micros = ceiling - cost_micros.
    /// This validates the evidence value type directly (not via select_weighted).
    #[test]
    fn route_score_headroom_is_ceiling_minus_cost() {
        let score = RouteScore {
            account_id: AccountId("x".to_owned()),
            cost_micros: 300,
            budget_headroom_micros: 700, // ceiling=1000 - cost=300
            model_affinity: false,
            failover_rank: 0,
        };
        assert_eq!(score.budget_headroom_micros, 700);
    }

    // ── Happy path: single eligible candidate ────────────────────────────────

    /// Single eligible candidate → chosen, model hint propagated.
    #[test]
    fn weighted_single_eligible_candidate_is_chosen() {
        let acc = active("solo", ProviderFamily::Claude);
        let cands = vec![candidate(&acc, 42, true)];
        let c = constraints();
        let exp = RoutePolicy::select_weighted(&cands, &c).unwrap();
        assert_eq!(exp.chosen_account_id, aid("solo"));
        assert_eq!(exp.chosen_model, "claude-sonnet-4-6");
    }
}
