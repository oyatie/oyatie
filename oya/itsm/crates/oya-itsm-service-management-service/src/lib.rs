#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! ITSM µservice umbrella crate.
//!
//! Per the Wave 15A remediation (audit `coherence-audit-2026-05-20.md` F-IC-12 + PRD §C
//! plurality fix), the ITSM µservice composes 5 bounded-context crates:
//!
//! 1. [`on_call_schedule`] — PagerDuty / Opsgenie / FireHydrant schedule displacement.
//! 2. [`escalation_policy`] — escalation chains across PersonalMessenger / SMS / voice / push.
//! 3. [`incident_room`] — MLS-encrypted (RFC 9420 per ADR-0246) war-rooms for major incidents.
//! 4. [`status_update`] — statuspage-class incident communications.
//! 5. [`postmortem`] — blameless retros with action-item linkage to change / problem records.
//!
//! Legacy service-management modules (incident, problem, change, service-request,
//! configuration-item) continue to be re-exported through the umbrella while the migration
//! to per-bounded-context crates proceeds, but the canonical ITIL operational primitives now
//! live in [`oya_itsm_on_call_schedule`], [`oya_itsm_escalation_policy`], [`oya_itsm_incident_room`],
//! [`oya_itsm_status_update`], and [`oya_itsm_postmortem`].

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use adapter::{
    asyncapi::{ChangeApprovedEvent, IncidentOpenedEvent, ItsmAsyncApiHandler, SlaBreachedEvent},
    grpc::{ItsmGrpcHandler, TicketGrpcRequest, TicketGrpcResponse},
    http::{HttpMethod, ItsmHttpHandler, RouteDescriptor},
};
pub use config::{RuntimeProfile, ServiceConfig};
pub use domain::{
    ArchitectureLayer, AuditEventKind, Capability, ChangeId, CiId, DataClass, DomainInvariant,
    IncidentTicket, ItsmPolicy, Priority, RequesterId, ServiceImpact, SlaClock, TenantId, TicketId,
    TicketStatus,
};
pub use error::{ServiceError, ServiceResult};
pub use usecase::{
    ApproveChange, ApproveChangeCommand, ItsmPorts, ItsmService, OpenIncident, OpenIncidentCommand,
    RecomputeSla, RecomputeSlaCommand,
};

// Re-export each bounded-context crate as a nested module so downstream callers can address
// `oya_itsm::on_call_schedule::ShiftWindow` etc., proving the plurality at the API surface.
pub use oya_itsm_escalation_policy as escalation_policy;
pub use oya_itsm_incident_room as incident_room;
pub use oya_itsm_on_call_schedule as on_call_schedule;
pub use oya_itsm_postmortem as postmortem;
pub use oya_itsm_status_update as status_update;

pub const MICROSERVICE: &str = "itsm";
pub const BOUNDED_CONTEXT_UMBRELLA: &str = "service-management";
/// Canonical bounded-context plurality declared by the µservice (audit fix for F-IC-12/§C).
pub const BOUNDED_CONTEXTS: &[&str] = &[
    "on-call-schedule",
    "escalation-policy",
    "incident-room",
    "status-update",
    "postmortem",
];
pub const PRIMARY_CAPABILITY: &str = "service-ticket-lifecycle";
pub const PRIMARY_ADR: &str = "ADR-0105";
pub const USECASE_RENAME_ADR: &str = "ADR-0106";
pub const TENANT_RESERVED_NAMESPACE: &str = "oyatie.it-ops";
pub const SUBSTANCE_BAR_ADR: &str = "ADR-0328";
pub const OBSERVABILITY_ADR: &str = "ADR-0263";
pub const CEDAR_ADR: &str = "ADR-0243";
pub const FOUNDRY_ABSORPTION_ADRS: &[&str] = &["ADR-0247", "ADR-0255-amendment"];
pub const OPENAPI_CONTRACT: &str = "microservices/itsm/contracts/openapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "microservices/itsm/contracts/itsm-v1.proto";
pub const ASYNCAPI_CONTRACT: &str = "microservices/itsm/contracts/asyncapi-v1.yaml";

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServiceDescriptor {
    pub microservice: &'static str,
    pub bounded_context_umbrella: &'static str,
    pub bounded_contexts: Vec<&'static str>,
    pub primary_capability: &'static str,
    pub architecture_layers: Vec<&'static str>,
    pub contract_paths: Vec<&'static str>,
}

impl ServiceDescriptor {
    pub fn layer_count(&self) -> usize {
        self.architecture_layers.len()
    }

    pub fn contract_count(&self) -> usize {
        self.contract_paths.len()
    }

    pub fn bounded_context_count(&self) -> usize {
        self.bounded_contexts.len()
    }

    pub fn includes_layer(&self, layer: ArchitectureLayer) -> bool {
        self.architecture_layers.contains(&layer.slug())
    }
}

pub fn descriptor() -> ServiceDescriptor {
    ServiceDescriptor {
        microservice: MICROSERVICE,
        bounded_context_umbrella: BOUNDED_CONTEXT_UMBRELLA,
        bounded_contexts: BOUNDED_CONTEXTS.to_vec(),
        primary_capability: PRIMARY_CAPABILITY,
        architecture_layers: ArchitectureLayer::all()
            .iter()
            .map(ArchitectureLayer::slug)
            .collect(),
        contract_paths: vec![OPENAPI_CONTRACT, GRPC_CONTRACT, ASYNCAPI_CONTRACT],
    }
}

pub fn default_incident_ticket() -> IncidentTicket {
    IncidentTicket::new(
        TenantId::new("tenant-demo"),
        TicketId::new("ticket-demo"),
        RequesterId::new("requester-demo"),
        "VPN access degraded".to_owned(),
        Priority::P2,
        TicketStatus::Draft,
    )
}

pub fn default_http_routes() -> Vec<RouteDescriptor> {
    ItsmHttpHandler::routes()
}

pub fn default_domain_invariants() -> Vec<DomainInvariant> {
    vec![
        DomainInvariant::tenant_scoped("tickets remain tenant scoped"),
        DomainInvariant::policy_checked("change approvals check freeze windows"),
        DomainInvariant::audit_emitted("major incident bridge opens emit audit events"),
        DomainInvariant::data_classified("requester notes stay support confidential"),
        DomainInvariant::region_bound("CMDB relations honor residency pack boundaries"),
        DomainInvariant::sla_monotonic("SLA clock recomputation never hides breached time"),
    ]
}

/// Verify the µservice scaffold meets ADR-0105/ADR-0565 (12 active layers), ADR-0131 (flat layout),
/// 3 contracts, and the 5-bounded-context plurality (audit fix for F-IC-12/§C).
pub fn validate_scaffold() -> ServiceResult<()> {
    let descriptor = descriptor();
    if descriptor.layer_count() != 12 {
        return Err(ServiceError::InvariantViolation {
            invariant: "adr_0105_layer_count",
            details: format!("expected 12 layers, found {}", descriptor.layer_count()),
        });
    }
    if descriptor.contract_count() != 3 {
        return Err(ServiceError::InvariantViolation {
            invariant: "contract_surface_count",
            details: format!(
                "expected 3 contracts, found {}",
                descriptor.contract_count()
            ),
        });
    }
    if descriptor.bounded_context_count() != 5 {
        return Err(ServiceError::InvariantViolation {
            invariant: "bounded_context_plurality",
            details: format!(
                "expected 5 bounded contexts (on-call-schedule, escalation-policy, incident-room, status-update, postmortem); found {}",
                descriptor.bounded_context_count()
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod umbrella_tests {
    use super::*;

    #[test]
    fn scaffold_validates() {
        validate_scaffold().expect("scaffold should validate");
    }

    #[test]
    fn bounded_contexts_match_audit_required_set() {
        assert_eq!(BOUNDED_CONTEXTS.len(), 5);
        assert!(BOUNDED_CONTEXTS.contains(&"on-call-schedule"));
        assert!(BOUNDED_CONTEXTS.contains(&"escalation-policy"));
        assert!(BOUNDED_CONTEXTS.contains(&"incident-room"));
        assert!(BOUNDED_CONTEXTS.contains(&"status-update"));
        assert!(BOUNDED_CONTEXTS.contains(&"postmortem"));
    }

    #[test]
    fn bounded_context_crates_export_their_slug() {
        assert_eq!(on_call_schedule::BOUNDED_CONTEXT, "on-call-schedule");
        assert_eq!(escalation_policy::BOUNDED_CONTEXT, "escalation-policy");
        assert_eq!(incident_room::BOUNDED_CONTEXT, "incident-room");
        assert_eq!(status_update::BOUNDED_CONTEXT, "status-update");
        assert_eq!(postmortem::BOUNDED_CONTEXT, "postmortem");
    }
}
