#![forbid(unsafe_code)]
//! `oya-itsm-incident-room`: bounded context for major-incident war-rooms. Each incident-room is
//! an MLS-encrypted group (RFC 9420 per ADR-0246) with named roles (commander, scribe, comms,
//! liaison, observer). Counterparts: PagerDuty Incident Workflows, FireHydrant Runbooks,
//! ServiceNow Major Incident Management.

use serde::{Deserialize, Serialize};

pub const BOUNDED_CONTEXT: &str = "incident-room";
pub const COUNTERPARTS: &[&str] = &[
    "PagerDuty Incident Workflows",
    "FireHydrant Runbooks",
    "ServiceNow Major Incident Management",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IncidentRole {
    Commander,
    Scribe,
    Communications,
    Liaison,
    Observer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RoomSeverity {
    Sev1,
    Sev2,
    Sev3,
    Sev4,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoomMember {
    pub principal_id: String,
    pub role: IncidentRole,
    pub joined_epoch_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IncidentRoom {
    pub tenant_id: String,
    pub room_id: String,
    pub incident_id: String,
    pub severity: RoomSeverity,
    pub mls_group_id: String,
    pub members: Vec<RoomMember>,
    pub opened_epoch_seconds: i64,
    pub closed_epoch_seconds: Option<i64>,
}

pub fn invariants() -> Vec<&'static str> {
    vec![
        "incident_room_tenant_required",
        "incident_room_mls_group_required_per_adr_0246",
        "incident_room_commander_role_unique",
        "incident_room_sev1_requires_commander_within_5_minutes",
        "incident_room_member_join_emits_audit_event",
        "incident_room_close_emits_postmortem_handoff",
        "incident_room_message_history_pinned_to_audit_chain",
    ]
}

pub fn validate_room(room: &IncidentRoom) -> Result<(), &'static str> {
    if room.tenant_id.is_empty() {
        return Err("incident_room_tenant_required");
    }
    if room.mls_group_id.is_empty() {
        return Err("incident_room_mls_group_required_per_adr_0246");
    }
    let commander_count = room
        .members
        .iter()
        .filter(|m| m.role == IncidentRole::Commander)
        .count();
    if commander_count > 1 {
        return Err("incident_room_commander_role_unique");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mls_group_required() {
        let bad = IncidentRoom {
            tenant_id: "t1".into(),
            room_id: "room-1".into(),
            incident_id: "inc-1".into(),
            severity: RoomSeverity::Sev1,
            mls_group_id: String::new(),
            members: vec![],
            opened_epoch_seconds: 1,
            closed_epoch_seconds: None,
        };
        assert_eq!(
            validate_room(&bad),
            Err("incident_room_mls_group_required_per_adr_0246")
        );
    }

    #[test]
    fn commander_unique() {
        let bad = IncidentRoom {
            tenant_id: "t1".into(),
            room_id: "room-1".into(),
            incident_id: "inc-1".into(),
            severity: RoomSeverity::Sev1,
            mls_group_id: "mls-group-1".into(),
            members: vec![
                RoomMember {
                    principal_id: "p1".into(),
                    role: IncidentRole::Commander,
                    joined_epoch_seconds: 1,
                },
                RoomMember {
                    principal_id: "p2".into(),
                    role: IncidentRole::Commander,
                    joined_epoch_seconds: 2,
                },
            ],
            opened_epoch_seconds: 1,
            closed_epoch_seconds: None,
        };
        assert_eq!(validate_room(&bad), Err("incident_room_commander_role_unique"));
    }
}
