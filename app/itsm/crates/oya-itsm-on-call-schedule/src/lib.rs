#![forbid(unsafe_code)]
//! `oya-itsm-on-call-schedule`: bounded context for on-call rotation, shift coverage, override,
//! and schedule rendering. Counterparts: PagerDuty Schedules, Opsgenie On-Call, FireHydrant
//! Schedules. Tenant-scoped per ADR-0244; Cedar-gated per ADR-0243; audit-emitted per ADR-0263.

use serde::{Deserialize, Serialize};

pub const BOUNDED_CONTEXT: &str = "on-call-schedule";
pub const COUNTERPARTS: &[&str] = &["PagerDuty", "Opsgenie", "FireHydrant"];

/// Rotation kind expressed as a closed enum so Cedar can match on a finite cardinality.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RotationKind {
    Weekly,
    Daily,
    FollowTheSun,
    Custom,
}

/// A shift coverage window. Times are intentionally stored as integer seconds since epoch so
/// HLC ordering (ADR-0252) is preserved without depending on a chrono crate at the kernel layer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShiftWindow {
    pub tenant_id: String,
    pub schedule_id: String,
    pub responder_id: String,
    pub start_epoch_seconds: i64,
    pub end_epoch_seconds: i64,
    pub rotation_kind: RotationKind,
}

/// An override entry. Override authority is recorded against the principal that filed it so the
/// audit chain (ADR-0263) can prove who reassigned the shift and why.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScheduleOverride {
    pub tenant_id: String,
    pub schedule_id: String,
    pub original_responder_id: String,
    pub override_responder_id: String,
    pub principal_id: String,
    pub reason: String,
    pub start_epoch_seconds: i64,
    pub end_epoch_seconds: i64,
}

/// Domain invariants for the on-call-schedule context.
pub fn invariants() -> Vec<&'static str> {
    vec![
        "shift_window_tenant_required",
        "shift_window_responder_member_of_tenant",
        "override_principal_has_schedule_admin_grant",
        "override_window_within_shift_window",
        "rotation_change_emits_audit_event",
        "schedule_render_is_tenant_scoped_only",
        "follow_the_sun_window_respects_residency_pack",
    ]
}

/// Validate a `ShiftWindow` against the closed invariant set.
pub fn validate_shift(window: &ShiftWindow) -> Result<(), &'static str> {
    if window.tenant_id.is_empty() {
        return Err("shift_window_tenant_required");
    }
    if window.responder_id.is_empty() {
        return Err("shift_window_responder_member_of_tenant");
    }
    if window.end_epoch_seconds <= window.start_epoch_seconds {
        return Err("shift_window_end_after_start");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invariants_are_named_and_unique() {
        let names = invariants();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "invariant names must be unique");
    }

    #[test]
    fn shift_window_requires_tenant() {
        let bad = ShiftWindow {
            tenant_id: String::new(),
            schedule_id: "sched-1".into(),
            responder_id: "responder-1".into(),
            start_epoch_seconds: 0,
            end_epoch_seconds: 3600,
            rotation_kind: RotationKind::Weekly,
        };
        assert_eq!(validate_shift(&bad), Err("shift_window_tenant_required"));
    }

    #[test]
    fn shift_window_requires_end_after_start() {
        let bad = ShiftWindow {
            tenant_id: "t1".into(),
            schedule_id: "sched-1".into(),
            responder_id: "responder-1".into(),
            start_epoch_seconds: 100,
            end_epoch_seconds: 100,
            rotation_kind: RotationKind::Daily,
        };
        assert_eq!(validate_shift(&bad), Err("shift_window_end_after_start"));
    }
}
