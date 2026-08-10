pub mod http {
    use crate::domain::{ChangeId, Priority, RequesterId, TenantId, TicketId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        ApproveChangeCommand, ItsmPorts, ItsmService, OpenIncidentCommand, RecomputeSlaCommand,
        UsecaseReceipt,
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub enum HttpMethod {
        Get,
        Post,
        Put,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct RouteDescriptor {
        pub method: HttpMethod,
        pub path: &'static str,
        pub capability: &'static str,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct OpenIncidentHttpRequest {
        pub tenant_id: String,
        pub ticket_id: String,
        pub requester_id: String,
        pub title: String,
        pub priority: Priority,
    }

    pub struct ItsmHttpHandler;

    impl ItsmHttpHandler {
        pub fn routes() -> Vec<RouteDescriptor> {
            vec![
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/incidents",
                    capability: "itsm.incident.open",
                },
                RouteDescriptor {
                    method: HttpMethod::Put,
                    path: "/v1/tickets/{ticket_id}/sla",
                    capability: "itsm.sla.recompute",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/tickets/{ticket_id}/changes/{change_id}/approve",
                    capability: "itsm.change.approve",
                },
            ]
        }

        pub fn open_incident(
            service: &mut ItsmService<impl ItsmPorts>,
            request: OpenIncidentHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.open_incident(OpenIncidentCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                ticket_id: TicketId::parse(request.ticket_id)?,
                requester_id: RequesterId::parse(request.requester_id)?,
                title: request.title,
                priority: request.priority,
            })
        }

        pub fn recompute_sla(
            service: &mut ItsmService<impl ItsmPorts>,
            tenant_id: String,
            ticket_id: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.recompute_sla(RecomputeSlaCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                ticket_id: TicketId::parse(ticket_id)?,
            })
        }

        pub fn approve_change(
            service: &mut ItsmService<impl ItsmPorts>,
            tenant_id: String,
            ticket_id: String,
            change_id: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.approve_change(ApproveChangeCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                ticket_id: TicketId::parse(ticket_id)?,
                change_id: ChangeId::parse(change_id)?,
            })
        }
    }
}

pub mod grpc {
    use crate::domain::{Priority, RequesterId, TenantId, TicketId};
    use crate::error::ServiceResult;
    use crate::usecase::{ItsmPorts, ItsmService, OpenIncidentCommand, UsecaseReceipt};

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct TicketGrpcRequest {
        pub tenant_id: String,
        pub ticket_id: String,
        pub requester_id: String,
        pub title: String,
        pub priority: Priority,
        pub request_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct TicketGrpcResponse {
        pub tenant_id: String,
        pub ticket_id: String,
        pub status: String,
        pub audit_event: String,
    }

    pub struct ItsmGrpcHandler;

    impl ItsmGrpcHandler {
        pub fn open_incident(
            service: &mut ItsmService<impl ItsmPorts>,
            request: TicketGrpcRequest,
        ) -> ServiceResult<TicketGrpcResponse> {
            let receipt = service.open_incident(OpenIncidentCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                ticket_id: TicketId::parse(request.ticket_id)?,
                requester_id: RequesterId::parse(request.requester_id)?,
                title: request.title,
                priority: request.priority,
            })?;
            Ok(Self::response_from_receipt(receipt))
        }

        fn response_from_receipt(receipt: UsecaseReceipt) -> TicketGrpcResponse {
            TicketGrpcResponse {
                tenant_id: receipt.tenant_id.as_str().to_owned(),
                ticket_id: receipt.ticket_id.as_str().to_owned(),
                status: format!("{:?}", receipt.status),
                audit_event: format!("{:?}", receipt.audit_event),
            }
        }
    }
}

pub mod asyncapi {
    use crate::domain::{AuditEventKind, TenantId, TicketId};
    use crate::error::ServiceResult;

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct IncidentOpenedEvent {
        pub tenant_id: TenantId,
        pub ticket_id: TicketId,
        pub audit_event: AuditEventKind,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct SlaBreachedEvent {
        pub tenant_id: TenantId,
        pub ticket_id: TicketId,
        pub elapsed_minutes: u32,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct ChangeApprovedEvent {
        pub tenant_id: TenantId,
        pub ticket_id: TicketId,
        pub change_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PublishedMessage {
        pub topic: String,
        pub payload_json: String,
    }

    pub struct ItsmAsyncApiHandler;

    impl ItsmAsyncApiHandler {
        pub fn incident_opened(
            prefix: &str,
            event: IncidentOpenedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.incident.opened"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn sla_breached(
            prefix: &str,
            event: SlaBreachedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.sla.breached"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn change_approved(
            prefix: &str,
            event: ChangeApprovedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.change.approved"),
                payload_json: serde_json::to_string(&event)?,
            })
        }
    }
}

pub mod memory {
    use std::collections::BTreeMap;

    use crate::domain::{AuditEventKind, Capability, IncidentTicket, TenantId, TicketId};
    use crate::error::{ServiceError, ServiceResult};
    use crate::usecase::{AuditPublisher, PolicyAuthorizer, TicketRepository};

    #[derive(Clone, Debug, Default)]
    pub struct InMemoryItsmPorts {
        tickets: BTreeMap<String, IncidentTicket>,
        audit_events: Vec<String>,
        denied_capabilities: Vec<Capability>,
    }

    impl InMemoryItsmPorts {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn deny(mut self, capability: Capability) -> Self {
            self.denied_capabilities.push(capability);
            self
        }

        pub fn audit_events(&self) -> &[String] {
            &self.audit_events
        }

        fn key(tenant_id: &TenantId, ticket_id: &TicketId) -> String {
            format!("{}::{}", tenant_id.as_str(), ticket_id.as_str())
        }
    }

    impl TicketRepository for InMemoryItsmPorts {
        fn put_ticket(&mut self, ticket: IncidentTicket) -> ServiceResult<IncidentTicket> {
            let key = Self::key(&ticket.tenant_id, &ticket.ticket_id);
            self.tickets.insert(key, ticket.clone());
            Ok(ticket)
        }

        fn get_ticket(
            &self,
            tenant_id: &TenantId,
            ticket_id: &TicketId,
        ) -> ServiceResult<Option<IncidentTicket>> {
            Ok(self.tickets.get(&Self::key(tenant_id, ticket_id)).cloned())
        }
    }

    impl PolicyAuthorizer for InMemoryItsmPorts {
        fn authorize(&self, _tenant_id: &TenantId, capability: Capability) -> ServiceResult<()> {
            if self.denied_capabilities.contains(&capability) {
                Err(ServiceError::policy_denied(
                    capability.action_slug(),
                    "capability denied by in-memory policy",
                ))
            } else {
                Ok(())
            }
        }
    }

    impl AuditPublisher for InMemoryItsmPorts {
        fn publish_audit(
            &mut self,
            tenant_id: &TenantId,
            event_kind: AuditEventKind,
            subject: &str,
        ) -> ServiceResult<()> {
            self.audit_events
                .push(format!("{}::{event_kind:?}::{subject}", tenant_id.as_str()));
            Ok(())
        }
    }
}
