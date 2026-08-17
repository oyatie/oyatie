#![forbid(unsafe_code)]
//! `oya-itsm-postmortem`: bounded context for blameless retros. Counterparts: FireHydrant Retro,
//! Jeli, PagerDuty Postmortem. Each postmortem ties to a closed incident and is exported as
//! audit-chain-pinned evidence (ADR-0263).

use serde::{Deserialize, Serialize};

pub const BOUNDED_CONTEXT: &str = "postmortem";
pub const COUNTERPARTS: &[&str] = &["FireHydrant Retro", "Jeli", "PagerDuty Postmortem"];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PostmortemKind {
    Blameless,
    Compliance,
    SecurityIncident,
    CustomerFacing,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub epoch_seconds: i64,
    pub principal_id: String,
    pub event_kind: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionItem {
    pub action_id: String,
    pub owner_principal_id: String,
    pub due_epoch_seconds: i64,
    pub linked_change_id: Option<String>,
    pub linked_problem_id: Option<String>,
    pub status: ActionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActionStatus {
    Open,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Postmortem {
    pub tenant_id: String,
    pub postmortem_id: String,
    pub incident_id: String,
    pub kind: PostmortemKind,
    pub timeline: Vec<TimelineEntry>,
    pub action_items: Vec<ActionItem>,
    pub published_epoch_seconds: Option<i64>,
}

pub fn invariants() -> Vec<&'static str> {
    vec![
        "postmortem_tenant_required",
        "postmortem_incident_must_be_closed",
        "postmortem_action_items_must_have_owners",
        "postmortem_blameless_no_named_blame_assignment",
        "postmortem_timeline_monotonic",
        "postmortem_publication_emits_audit_event",
        "postmortem_action_links_to_change_or_problem_record",
    ]
}

pub fn validate_postmortem(pm: &Postmortem) -> Result<(), &'static str> {
    if pm.tenant_id.is_empty() {
        return Err("postmortem_tenant_required");
    }
    for action in &pm.action_items {
        if action.owner_principal_id.is_empty() {
            return Err("postmortem_action_items_must_have_owners");
        }
    }
    let mut prev: Option<i64> = None;
    for entry in &pm.timeline {
        if let Some(p) = prev {
            if entry.epoch_seconds < p {
                return Err("postmortem_timeline_monotonic");
            }
        }
        prev = Some(entry.epoch_seconds);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_without_owner_rejected() {
        let bad = Postmortem {
            tenant_id: "t1".into(),
            postmortem_id: "pm1".into(),
            incident_id: "inc1".into(),
            kind: PostmortemKind::Blameless,
            timeline: vec![],
            action_items: vec![ActionItem {
                action_id: "a1".into(),
                owner_principal_id: String::new(),
                due_epoch_seconds: 100,
                linked_change_id: None,
                linked_problem_id: None,
                status: ActionStatus::Open,
            }],
            published_epoch_seconds: None,
        };
        assert_eq!(
            validate_postmortem(&bad),
            Err("postmortem_action_items_must_have_owners")
        );
    }

    #[test]
    fn timeline_monotonic_enforced() {
        let bad = Postmortem {
            tenant_id: "t1".into(),
            postmortem_id: "pm1".into(),
            incident_id: "inc1".into(),
            kind: PostmortemKind::Blameless,
            timeline: vec![
                TimelineEntry {
                    epoch_seconds: 200,
                    principal_id: "p1".into(),
                    event_kind: "page_acked".into(),
                    description: "x".into(),
                },
                TimelineEntry {
                    epoch_seconds: 100,
                    principal_id: "p2".into(),
                    event_kind: "page_received".into(),
                    description: "y".into(),
                },
            ],
            action_items: vec![],
            published_epoch_seconds: None,
        };
        assert_eq!(
            validate_postmortem(&bad),
            Err("postmortem_timeline_monotonic")
        );
    }
}
