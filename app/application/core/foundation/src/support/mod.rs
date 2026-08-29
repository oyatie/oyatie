//! Shared foundation helpers.

use crate::*;

mod attributes;
mod data_use;

pub(crate) use attributes::*;
pub(crate) use data_use::*;

pub(crate) fn emit_invocation_trace(
    span: &dyn CapabilityInvocationTraceSpan,
    result: &'static str,
    error_type: Option<&'static str>,
) {
    span.emit_result(InvocationTraceResult { result, error_type });
}

pub(crate) fn autonomy_tier_label(tier: AutonomyTier) -> &'static str {
    match tier {
        AutonomyTier::T1ViewOnly => "T1",
        AutonomyTier::T2Advisory => "T2",
        AutonomyTier::T3ExecuteWithApproval => "T3",
        AutonomyTier::T4AutoExecute => "T4",
    }
}

pub(crate) fn epoch_seconds_to_epoch_days(seconds: u64) -> u64 {
    seconds / 86_400
}

pub(crate) fn apply_autonomy_break_glass(
    autonomy_decision: &mut AutonomyDecision,
    break_glass: &AutonomyBreakGlass,
) {
    autonomy_decision.denial_threshold = break_glass.permitted_tier.value;
    autonomy_decision.effective_ceiling = autonomy_decision.required_tier;
    autonomy_decision.verdict = AutonomyVerdict::Allow;
    autonomy_decision.blocking_cap_source = None;
    autonomy_decision.blocking_cap_reason = None;
    autonomy_decision.lowering_cap_source = AutonomyCapSource::CapabilityRequired;
    autonomy_decision.lowering_cap_reason = AutonomyCapReason::CapabilityRequiredTier;
}

pub(crate) fn autonomy_decision_label(autonomy_decision: &AutonomyDecision) -> &'static str {
    if autonomy_decision.allowed() {
        "ALLOW"
    } else {
        "DENY"
    }
}

pub(crate) fn map_policy_error(error: PolicyError) -> FoundationError {
    match error {
        PolicyError::VersionAlreadyExists => FoundationError::PolicyVersionAlreadyExists,
        PolicyError::InvalidPolicyId
        | PolicyError::InvalidSemver
        | PolicyError::EmptyRules
        | PolicyError::EmptyRuleField
        | PolicyError::SupersedesSelf
        | PolicyError::SupersedesMissing
        | PolicyError::SupersedesScopeMismatch
        | PolicyError::SupersedesNotOlder => FoundationError::InvalidInput,
    }
}
