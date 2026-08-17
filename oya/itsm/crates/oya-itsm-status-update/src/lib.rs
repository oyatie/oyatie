#![forbid(unsafe_code)]
//! `oya-itsm-status-update`: bounded context for statuspage-class incident communications.
//! Counterparts: Atlassian Statuspage, FireHydrant Statuspage, Opsgenie Status. Every status
//! update is tenant-scoped (ADR-0244), Cedar-gated (ADR-0243), and audit-emitted (ADR-0263).

use serde::{Deserialize, Serialize};

pub const BOUNDED_CONTEXT: &str = "status-update";
pub const COUNTERPARTS: &[&str] = &[
    "Atlassian Statuspage",
    "FireHydrant Statuspage",
    "Opsgenie Status",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StatusStage {
    Investigating,
    Identified,
    Monitoring,
    Resolved,
    Maintenance,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AudienceScope {
    InternalOperators,
    InternalAllEmployees,
    TenantCustomersScoped,
    PublicReader,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub tenant_id: String,
    pub status_page_id: String,
    pub update_id: String,
    pub incident_id: String,
    pub stage: StatusStage,
    pub audience_scope: AudienceScope,
    pub body_markdown: String,
    pub posted_epoch_seconds: i64,
    pub posted_by_principal: String,
}

pub fn invariants() -> Vec<&'static str> {
    vec![
        "status_update_tenant_required",
        "status_update_stage_monotonic_per_incident",
        "status_update_public_audience_requires_pack_disclosure_review",
        "status_update_body_redacts_pii_per_data_class",
        "status_update_emits_audit_event_with_principal",
    ]
}

pub fn validate_update(update: &StatusUpdate) -> Result<(), &'static str> {
    if update.tenant_id.is_empty() {
        return Err("status_update_tenant_required");
    }
    if update.body_markdown.is_empty() {
        return Err("status_update_body_must_be_nonempty");
    }
    if update.posted_by_principal.is_empty() {
        return Err("status_update_emits_audit_event_with_principal");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_required() {
        let bad = StatusUpdate {
            tenant_id: String::new(),
            status_page_id: "sp1".into(),
            update_id: "u1".into(),
            incident_id: "inc-1".into(),
            stage: StatusStage::Investigating,
            audience_scope: AudienceScope::TenantCustomersScoped,
            body_markdown: "We are investigating".into(),
            posted_epoch_seconds: 1,
            posted_by_principal: "p1".into(),
        };
        assert_eq!(validate_update(&bad), Err("status_update_tenant_required"));
    }

    #[test]
    fn principal_required() {
        let bad = StatusUpdate {
            tenant_id: "t1".into(),
            status_page_id: "sp1".into(),
            update_id: "u1".into(),
            incident_id: "inc-1".into(),
            stage: StatusStage::Identified,
            audience_scope: AudienceScope::PublicReader,
            body_markdown: "Root cause identified".into(),
            posted_epoch_seconds: 2,
            posted_by_principal: String::new(),
        };
        assert_eq!(
            validate_update(&bad),
            Err("status_update_emits_audit_event_with_principal")
        );
    }
}
