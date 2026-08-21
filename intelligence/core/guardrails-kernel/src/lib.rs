//! Pure guardrail decision kernel for Intelligence requests.
//!
//! The kernel is intentionally fail-closed: missing classifier output, high-risk
//! findings, and always-blocked safety categories produce deny decisions with
//! deterministic evidence references.
//!
//! Shadow mode (`decide_guardrail_shadow`) records what WOULD be denied without
//! enforcing the decision, enabling safe observation of new classifier rollouts.
//! The `FpBudget` type provides deterministic false-positive-budget accounting
//! feeding the guardrails-shadow-mode OpenSLO.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

// ---------------------------------------------------------------------------
// Existing enforced-mode types (unchanged)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GuardrailCategory {
    Benign,
    ChildSafety,
    CredentialLeakage,
    HateHarassment,
    PromptInjection,
    RegulatedFinancial,
    RegulatedMedical,
    SelfHarm,
    SexualContent,
    Violence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailFinding {
    pub category: GuardrailCategory, // data_class: INTERNAL_ONLY
    pub risk_level: RiskLevel,       // data_class: INTERNAL_ONLY
    pub reason: String,              // data_class: INTERNAL_ONLY
    pub evidence_ref: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailRequest {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub content_ref: String,             // data_class: INTERNAL_ONLY
    pub findings: Vec<GuardrailFinding>, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailAllow {
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailDeny {
    pub refusal_reasons: Vec<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuardrailDecision {
    Allow(GuardrailAllow),
    Deny(GuardrailDeny),
}

/// Enforced guardrail evaluation. Fail-closed: returns `Deny` when findings are
/// absent, high-risk, or in an always-blocked category.
pub fn decide_guardrail(request: &GuardrailRequest) -> GuardrailDecision {
    let mut evidence_refs = vec![request.request_evidence_ref.clone()];
    let mut refusal_reasons = Vec::new();

    if request.findings.is_empty() {
        refusal_reasons.push("guardrail classification missing; request denied closed".to_owned());
    }

    for finding in &request.findings {
        evidence_refs.push(finding.evidence_ref.clone());
        if finding_requires_refusal(finding) {
            refusal_reasons.push(refusal_reason_for(finding));
        }
    }

    let evidence_refs = sorted_unique(evidence_refs);
    let refusal_reasons = sorted_unique(refusal_reasons);

    if refusal_reasons.is_empty() {
        GuardrailDecision::Allow(GuardrailAllow { evidence_refs })
    } else {
        GuardrailDecision::Deny(GuardrailDeny {
            refusal_reasons,
            evidence_refs,
        })
    }
}

// ---------------------------------------------------------------------------
// Shadow-mode types
// ---------------------------------------------------------------------------

/// Records what `decide_guardrail` WOULD have decided without enforcing it.
///
/// `would_deny` is `true` iff the enforced path would return `GuardrailDecision::Deny`.
/// This type carries no enforcement — it is purely observational.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShadowDecision {
    pub would_deny: bool,
    pub would_deny_reasons: Vec<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

/// Shadow-mode evaluation: computes what the enforced guardrail path would decide
/// without enforcing the decision. Safe for observation-only pipelines.
///
/// The logic mirrors `decide_guardrail` exactly — same fail-closed empty-findings
/// behaviour, same category/risk-level rules — but returns `ShadowDecision`.
pub fn decide_guardrail_shadow(request: &GuardrailRequest) -> ShadowDecision {
    let mut evidence_refs = vec![request.request_evidence_ref.clone()];
    let mut would_deny_reasons = Vec::new();

    if request.findings.is_empty() {
        would_deny_reasons
            .push("guardrail classification missing; request denied closed".to_owned());
    }

    for finding in &request.findings {
        evidence_refs.push(finding.evidence_ref.clone());
        if finding_requires_refusal(finding) {
            would_deny_reasons.push(refusal_reason_for(finding));
        }
    }

    let evidence_refs = sorted_unique(evidence_refs);
    let would_deny_reasons = sorted_unique(would_deny_reasons);
    let would_deny = !would_deny_reasons.is_empty();

    ShadowDecision {
        would_deny,
        would_deny_reasons,
        evidence_refs,
    }
}

// ---------------------------------------------------------------------------
// False-positive budget accounting
// ---------------------------------------------------------------------------

/// Error returned by `FpBudget::new` or `FpBudget::merge` when parameters are invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FpBudgetError {
    /// `budget_pct` was not in the range `(0.0, 1.0]`.
    InvalidBudgetPct,
    /// `total_evals` was zero, making rate computation undefined.
    ZeroTotalEvals,
    /// `merge` called on two budgets with different `budget_pct` values.
    BudgetPctMismatch,
}

/// Deterministic false-positive-budget accounting for guardrail shadow-mode observations.
///
/// Tracks observed FP count against a configured percentage budget. Feeds the
/// `guardrails-shadow-mode-fp-budget` OpenSLO indicator.
///
/// `budget_pct` is a fraction in `(0.0, 1.0]` (e.g. `0.05` = 5 % budget).
#[derive(Clone, Debug, PartialEq)]
pub struct FpBudget {
    pub observed_fp: u32,
    pub total_evals: u32,
    pub budget_pct: f64, // data_class: INTERNAL_ONLY
}

impl FpBudget {
    /// Construct a new `FpBudget`.
    ///
    /// Returns `Err(FpBudgetError::ZeroTotalEvals)` when `total_evals == 0`.
    /// Returns `Err(FpBudgetError::InvalidBudgetPct)` when `budget_pct` is not
    /// in `(0.0, 1.0]`.
    pub fn new(observed_fp: u32, total_evals: u32, budget_pct: f64) -> Result<Self, FpBudgetError> {
        if total_evals == 0 {
            return Err(FpBudgetError::ZeroTotalEvals);
        }
        if budget_pct <= 0.0 || budget_pct > 1.0 {
            return Err(FpBudgetError::InvalidBudgetPct);
        }
        Ok(Self {
            observed_fp,
            total_evals,
            budget_pct,
        })
    }

    /// Ratio of observed false positives to total evaluations.
    pub fn observed_fp_rate(&self) -> f64 {
        self.observed_fp as f64 / self.total_evals as f64
    }

    /// Returns `true` when the observed FP rate meets or exceeds the configured budget.
    pub fn budget_exhausted(&self) -> bool {
        self.observed_fp_rate() >= self.budget_pct
    }

    /// Returns the remaining headroom as `budget_pct - observed_fp_rate`, clamped to `>= 0.0`.
    pub fn remaining_headroom(&self) -> f64 {
        f64::max(0.0, self.budget_pct - self.observed_fp_rate())
    }

    /// Severity-weighted false-positive aggregate over a slice of `(RiskLevel, count)` pairs.
    ///
    /// Returns the sum of `sw.weight_for(level) * count as f64` for each pair.
    /// The result is independent of `self.observed_fp` and `self.total_evals`.
    pub fn weighted_fp(&self, sw: &SeverityWeight, findings: &[(RiskLevel, u32)]) -> f64 {
        findings
            .iter()
            .map(|(level, count)| sw.weight_for(*level) * (*count as f64))
            .sum()
    }

    /// Merge two non-overlapping observation windows into a single `FpBudget`.
    ///
    /// Returns `Err(FpBudgetError::BudgetPctMismatch)` when the two budgets have different
    /// `budget_pct` values. Returns `Err(FpBudgetError::ZeroTotalEvals)` when the sum of
    /// `total_evals` is zero. Uses saturating addition to avoid overflow.
    pub fn merge(&self, other: &FpBudget) -> Result<FpBudget, FpBudgetError> {
        if (self.budget_pct - other.budget_pct).abs() > f64::EPSILON {
            return Err(FpBudgetError::BudgetPctMismatch);
        }
        let merged_total = self.total_evals.saturating_add(other.total_evals);
        let merged_fp = self.observed_fp.saturating_add(other.observed_fp);
        FpBudget::new(merged_fp, merged_total, self.budget_pct)
    }
}

// ---------------------------------------------------------------------------
// Severity-weight mapping
// ---------------------------------------------------------------------------

/// Per-level weights used by `FpBudget::weighted_fp`.
///
/// All weights must be non-negative; the caller is responsible for constructing
/// sensible values. Typical usage: Low=1.0, Medium=2.0, High=3.0.
#[derive(Clone, Debug, PartialEq)]
pub struct SeverityWeight {
    pub low: f64,
    pub medium: f64,
    pub high: f64,
}

impl SeverityWeight {
    /// Returns the weight for `level`.
    pub fn weight_for(&self, level: RiskLevel) -> f64 {
        match level {
            RiskLevel::Low => self.low,
            RiskLevel::Medium => self.medium,
            RiskLevel::High => self.high,
        }
    }
}

// ---------------------------------------------------------------------------
// Private helpers (unchanged)
// ---------------------------------------------------------------------------

fn finding_requires_refusal(finding: &GuardrailFinding) -> bool {
    finding.risk_level == RiskLevel::High || is_always_blocked_category(finding.category)
}

fn is_always_blocked_category(category: GuardrailCategory) -> bool {
    matches!(
        category,
        GuardrailCategory::ChildSafety
            | GuardrailCategory::CredentialLeakage
            | GuardrailCategory::PromptInjection
    )
}

fn refusal_reason_for(finding: &GuardrailFinding) -> String {
    let reason = finding.reason.trim();
    if reason.is_empty() {
        format!(
            "guardrail finding {:?}/{:?} requires refusal",
            finding.category, finding.risk_level
        )
    } else {
        reason.to_owned()
    }
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- helpers ---

    fn request_with_finding(finding: GuardrailFinding) -> GuardrailRequest {
        GuardrailRequest {
            tenant_id: "ten_a".to_owned(),
            content_ref: "content:2".to_owned(),
            findings: vec![finding],
            request_evidence_ref: "req:2".to_owned(),
        }
    }

    fn benign_finding() -> GuardrailFinding {
        GuardrailFinding {
            category: GuardrailCategory::Benign,
            risk_level: RiskLevel::Low,
            reason: "benign".to_owned(),
            evidence_ref: "classifier:1".to_owned(),
        }
    }

    // --- existing enforced-mode tests (unchanged) ---

    #[test]
    fn denies_high_risk_and_preserves_refusal_reason() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::RegulatedMedical,
            risk_level: RiskLevel::High,
            reason: "medical diagnosis request requires licensed escalation".to_owned(),
            evidence_ref: "classifier:2".to_owned(),
        });

        assert_eq!(
            decide_guardrail(&request),
            GuardrailDecision::Deny(GuardrailDeny {
                refusal_reasons: vec![
                    "medical diagnosis request requires licensed escalation".to_owned()
                ],
                evidence_refs: vec!["classifier:2".to_owned(), "req:2".to_owned()],
            })
        );
    }

    #[test]
    fn blank_high_risk_reason_still_denies_with_fallback_reason() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::PromptInjection,
            risk_level: RiskLevel::High,
            reason: " ".to_owned(),
            evidence_ref: "classifier:3".to_owned(),
        });

        assert_eq!(
            decide_guardrail(&request),
            GuardrailDecision::Deny(GuardrailDeny {
                refusal_reasons: vec![
                    "guardrail finding PromptInjection/High requires refusal".to_owned()
                ],
                evidence_refs: vec!["classifier:3".to_owned(), "req:2".to_owned()],
            })
        );
    }

    // --- shadow-mode tests ---

    #[test]
    fn shadow_allows_benign_finding() {
        let request = request_with_finding(benign_finding());
        let shadow = decide_guardrail_shadow(&request);
        assert!(!shadow.would_deny);
        assert!(shadow.would_deny_reasons.is_empty());
        assert_eq!(
            shadow.evidence_refs,
            vec!["classifier:1".to_owned(), "req:2".to_owned()]
        );
    }

    #[test]
    fn shadow_denies_high_risk_finding() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::RegulatedMedical,
            risk_level: RiskLevel::High,
            reason: "medical diagnosis request requires licensed escalation".to_owned(),
            evidence_ref: "classifier:2".to_owned(),
        });
        let shadow = decide_guardrail_shadow(&request);
        assert!(shadow.would_deny);
        assert_eq!(
            shadow.would_deny_reasons,
            vec!["medical diagnosis request requires licensed escalation".to_owned()]
        );
    }

    #[test]
    fn shadow_denies_always_blocked_child_safety() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::ChildSafety,
            risk_level: RiskLevel::Low,
            reason: "".to_owned(),
            evidence_ref: "classifier:cs".to_owned(),
        });
        let shadow = decide_guardrail_shadow(&request);
        assert!(shadow.would_deny);
        assert_eq!(
            shadow.would_deny_reasons,
            vec!["guardrail finding ChildSafety/Low requires refusal".to_owned()]
        );
    }

    #[test]
    fn shadow_denies_always_blocked_credential_leakage() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::CredentialLeakage,
            risk_level: RiskLevel::Low,
            reason: "".to_owned(),
            evidence_ref: "classifier:cl".to_owned(),
        });
        let shadow = decide_guardrail_shadow(&request);
        assert!(shadow.would_deny);
    }

    #[test]
    fn shadow_denies_always_blocked_prompt_injection() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::PromptInjection,
            risk_level: RiskLevel::Low,
            reason: "".to_owned(),
            evidence_ref: "classifier:pi".to_owned(),
        });
        let shadow = decide_guardrail_shadow(&request);
        assert!(shadow.would_deny);
    }

    #[test]
    fn shadow_denies_empty_findings() {
        let request = GuardrailRequest {
            tenant_id: "ten_a".to_owned(),
            content_ref: "content:empty".to_owned(),
            findings: vec![],
            request_evidence_ref: "req:empty".to_owned(),
        };
        let shadow = decide_guardrail_shadow(&request);
        assert!(shadow.would_deny);
        assert_eq!(
            shadow.would_deny_reasons,
            vec!["guardrail classification missing; request denied closed".to_owned()]
        );
    }

    #[test]
    fn shadow_decision_matches_enforced_decision_allow() {
        let request = request_with_finding(benign_finding());
        let enforced = decide_guardrail(&request);
        let shadow = decide_guardrail_shadow(&request);
        let enforced_is_deny = matches!(enforced, GuardrailDecision::Deny(_));
        assert_eq!(shadow.would_deny, enforced_is_deny);
    }

    #[test]
    fn shadow_decision_matches_enforced_decision_deny() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::Violence,
            risk_level: RiskLevel::High,
            reason: "high-risk violence".to_owned(),
            evidence_ref: "classifier:v".to_owned(),
        });
        let enforced = decide_guardrail(&request);
        let shadow = decide_guardrail_shadow(&request);
        let enforced_is_deny = matches!(enforced, GuardrailDecision::Deny(_));
        assert_eq!(shadow.would_deny, enforced_is_deny);
    }

    // --- FpBudget tests ---

    #[test]
    fn fp_budget_invalid_pct_zero() {
        assert_eq!(
            FpBudget::new(0, 100, 0.0),
            Err(FpBudgetError::InvalidBudgetPct)
        );
    }

    #[test]
    fn fp_budget_invalid_pct_negative() {
        assert_eq!(
            FpBudget::new(0, 100, -0.1),
            Err(FpBudgetError::InvalidBudgetPct)
        );
    }

    #[test]
    fn fp_budget_invalid_pct_over_one() {
        assert_eq!(
            FpBudget::new(0, 100, 1.1),
            Err(FpBudgetError::InvalidBudgetPct)
        );
    }

    #[test]
    fn fp_budget_zero_total_evals() {
        assert_eq!(
            FpBudget::new(0, 0, 0.05),
            Err(FpBudgetError::ZeroTotalEvals)
        );
    }

    #[test]
    fn fp_budget_valid_at_max_pct() {
        assert!(FpBudget::new(100, 100, 1.0).is_ok());
    }

    #[test]
    fn fp_budget_not_exhausted() {
        let budget = FpBudget::new(1, 100, 0.05).unwrap();
        assert!(!budget.budget_exhausted());
    }

    #[test]
    fn fp_budget_exhausted_at_boundary() {
        let budget = FpBudget::new(5, 100, 0.05).unwrap();
        assert!(budget.budget_exhausted());
    }

    #[test]
    fn fp_budget_exhausted_over_budget() {
        let budget = FpBudget::new(6, 100, 0.05).unwrap();
        assert!(budget.budget_exhausted());
    }

    #[test]
    fn fp_budget_observed_fp_rate_precision() {
        let budget = FpBudget::new(1, 100, 0.05).unwrap();
        assert!((budget.observed_fp_rate() - 0.01).abs() < f64::EPSILON);
    }

    // --- remaining_headroom tests ---

    #[test]
    fn remaining_headroom_at_budget_is_zero() {
        // 5/100 == 0.05 budget_pct exactly
        let budget = FpBudget::new(5, 100, 0.05).unwrap();
        assert!((budget.remaining_headroom() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remaining_headroom_below_budget_is_positive() {
        // 1/100 == 0.01, budget 0.05 -> headroom 0.04
        let budget = FpBudget::new(1, 100, 0.05).unwrap();
        assert!(budget.remaining_headroom() > 0.0);
        assert!((budget.remaining_headroom() - 0.04).abs() < 1e-12);
    }

    #[test]
    fn remaining_headroom_over_budget_clamped_to_zero() {
        // 10/100 == 0.10 > 0.05 budget -> headroom clamped to 0.0
        let budget = FpBudget::new(10, 100, 0.05).unwrap();
        assert!((budget.remaining_headroom() - 0.0).abs() < f64::EPSILON);
    }

    // --- weighted_fp tests ---

    #[test]
    fn weighted_fp_all_three_levels_unit_counts() {
        let budget = FpBudget::new(1, 100, 0.05).unwrap();
        let sw = SeverityWeight {
            low: 1.0,
            medium: 2.0,
            high: 3.0,
        };
        let findings = [
            (RiskLevel::Low, 1u32),
            (RiskLevel::Medium, 1),
            (RiskLevel::High, 1),
        ];
        let result = budget.weighted_fp(&sw, &findings);
        // 1*1 + 2*1 + 3*1 = 6.0
        assert!((result - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_fp_zero_counts_returns_zero() {
        let budget = FpBudget::new(1, 100, 0.05).unwrap();
        let sw = SeverityWeight {
            low: 1.0,
            medium: 2.0,
            high: 3.0,
        };
        let findings = [
            (RiskLevel::Low, 0u32),
            (RiskLevel::Medium, 0),
            (RiskLevel::High, 0),
        ];
        assert!((budget.weighted_fp(&sw, &findings)).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_fp_empty_slice_returns_zero() {
        let budget = FpBudget::new(1, 100, 0.05).unwrap();
        let sw = SeverityWeight {
            low: 1.0,
            medium: 2.0,
            high: 3.0,
        };
        assert!((budget.weighted_fp(&sw, &[])).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_fp_high_only() {
        let budget = FpBudget::new(1, 100, 0.05).unwrap();
        let sw = SeverityWeight {
            low: 1.0,
            medium: 2.0,
            high: 5.0,
        };
        let findings = [(RiskLevel::High, 3u32)];
        // 5.0 * 3 = 15.0
        assert!((budget.weighted_fp(&sw, &findings) - 15.0).abs() < f64::EPSILON);
    }

    // --- merge tests ---

    #[test]
    fn merge_happy_path_sums_observations() {
        let a = FpBudget::new(3, 50, 0.10).unwrap();
        let b = FpBudget::new(2, 50, 0.10).unwrap();
        let merged = a.merge(&b).unwrap();
        assert_eq!(merged.observed_fp, 5);
        assert_eq!(merged.total_evals, 100);
        assert!((merged.budget_pct - 0.10).abs() < f64::EPSILON);
    }

    #[test]
    fn merge_preserves_budget_pct() {
        let a = FpBudget::new(1, 100, 0.05).unwrap();
        let b = FpBudget::new(0, 100, 0.05).unwrap();
        let merged = a.merge(&b).unwrap();
        assert!((merged.budget_pct - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn merge_err_budget_pct_mismatch() {
        let a = FpBudget::new(1, 100, 0.05).unwrap();
        let b = FpBudget::new(1, 100, 0.10).unwrap();
        assert_eq!(a.merge(&b), Err(FpBudgetError::BudgetPctMismatch));
    }

    #[test]
    fn merge_err_zero_total_evals() {
        // Cannot construct FpBudget::new(0, 0, ...) because new() rejects zero totals.
        // We use the merge path: two budgets with total_evals=0 cannot be constructed.
        // Instead test the ZeroTotalEvals variant is still reachable via new() directly
        // to confirm the path exists; merge of two real budgets always has nonzero total.
        assert_eq!(
            FpBudget::new(0, 0, 0.05),
            Err(FpBudgetError::ZeroTotalEvals)
        );
    }

    #[test]
    fn merge_zero_observed_fps() {
        let a = FpBudget::new(0, 100, 0.05).unwrap();
        let b = FpBudget::new(0, 100, 0.05).unwrap();
        let merged = a.merge(&b).unwrap();
        assert_eq!(merged.observed_fp, 0);
        assert_eq!(merged.total_evals, 200);
    }

    // --- SeverityWeight tests ---

    #[test]
    fn severity_weight_for_each_level() {
        let sw = SeverityWeight {
            low: 1.0,
            medium: 2.0,
            high: 4.0,
        };
        assert!((sw.weight_for(RiskLevel::Low) - 1.0).abs() < f64::EPSILON);
        assert!((sw.weight_for(RiskLevel::Medium) - 2.0).abs() < f64::EPSILON);
        assert!((sw.weight_for(RiskLevel::High) - 4.0).abs() < f64::EPSILON);
    }
}
