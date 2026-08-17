#![forbid(unsafe_code)]
//! `oya-itsm-escalation-policy`: bounded context for escalation chains, notification rules,
//! and unacknowledged-page promotion. Counterparts: PagerDuty Escalation Policies, Opsgenie
//! Escalations, FireHydrant Notification Policies. Tenant-scoped per ADR-0244; Cedar-gated
//! per ADR-0243; audit-emitted per ADR-0263.

use serde::{Deserialize, Serialize};

pub const BOUNDED_CONTEXT: &str = "escalation-policy";
pub const COUNTERPARTS: &[&str] = &["PagerDuty", "Opsgenie", "FireHydrant"];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NotifyChannel {
    PersonalMessenger,
    Sms,
    Voice,
    Email,
    PushMobile,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EscalationStep {
    pub step_index: u8,
    pub responder_or_schedule_id: String,
    pub wait_seconds_before_escalate: u32,
    pub channels: Vec<NotifyChannel>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub tenant_id: String,
    pub policy_id: String,
    pub steps: Vec<EscalationStep>,
    pub loop_after_steps: bool,
    pub stop_after_minutes: u32,
}

pub fn invariants() -> Vec<&'static str> {
    vec![
        "escalation_policy_tenant_required",
        "escalation_step_ordered_monotonically",
        "escalation_step_responder_member_of_tenant",
        "escalation_personal_messenger_uses_mls_rfc_9420",
        "escalation_stop_after_minutes_is_finite",
        "escalation_change_emits_audit_event",
    ]
}

pub fn validate_policy(policy: &EscalationPolicy) -> Result<(), &'static str> {
    if policy.tenant_id.is_empty() {
        return Err("escalation_policy_tenant_required");
    }
    if policy.steps.is_empty() {
        return Err("escalation_policy_has_at_least_one_step");
    }
    let mut prev: Option<u8> = None;
    for step in policy.steps.iter() {
        if let Some(prev_idx) = prev
            && step.step_index <= prev_idx
        {
            return Err("escalation_step_ordered_monotonically");
        }
        prev = Some(step.step_index);
    }
    if policy.stop_after_minutes == 0 {
        return Err("escalation_stop_after_minutes_is_finite");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(idx: u8) -> EscalationStep {
        EscalationStep {
            step_index: idx,
            responder_or_schedule_id: format!("r-{idx}"),
            wait_seconds_before_escalate: 300,
            channels: vec![NotifyChannel::PersonalMessenger, NotifyChannel::PushMobile],
        }
    }

    #[test]
    fn empty_steps_rejected() {
        let bad = EscalationPolicy {
            tenant_id: "t1".into(),
            policy_id: "p1".into(),
            steps: vec![],
            loop_after_steps: false,
            stop_after_minutes: 30,
        };
        assert_eq!(
            validate_policy(&bad),
            Err("escalation_policy_has_at_least_one_step")
        );
    }

    #[test]
    fn monotonic_ordering_enforced() {
        let bad = EscalationPolicy {
            tenant_id: "t1".into(),
            policy_id: "p1".into(),
            steps: vec![step(2), step(1)],
            loop_after_steps: false,
            stop_after_minutes: 30,
        };
        assert_eq!(
            validate_policy(&bad),
            Err("escalation_step_ordered_monotonically")
        );
    }

    #[test]
    fn valid_policy_accepted() {
        let good = EscalationPolicy {
            tenant_id: "t1".into(),
            policy_id: "p1".into(),
            steps: vec![step(1), step(2), step(3)],
            loop_after_steps: true,
            stop_after_minutes: 30,
        };
        assert_eq!(validate_policy(&good), Ok(()));
    }
}
