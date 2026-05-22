use crate::domain::{
    AuditEventKind, Capability, ChangeId, IncidentTicket, Priority, RequesterId, TenantId,
    TicketId, TicketStatus,
};
use crate::error::{ServiceError, ServiceResult};

pub trait TicketRepository {
    fn put_ticket(&mut self, ticket: IncidentTicket) -> ServiceResult<IncidentTicket>;
    fn get_ticket(
        &self,
        tenant_id: &TenantId,
        ticket_id: &TicketId,
    ) -> ServiceResult<Option<IncidentTicket>>;
}

pub trait PolicyAuthorizer {
    fn authorize(&self, tenant_id: &TenantId, capability: Capability) -> ServiceResult<()>;
}

pub trait AuditPublisher {
    fn publish_audit(
        &mut self,
        tenant_id: &TenantId,
        event_kind: AuditEventKind,
        subject: &str,
    ) -> ServiceResult<()>;
}

pub trait ItsmPorts: TicketRepository + PolicyAuthorizer + AuditPublisher {}

impl<T> ItsmPorts for T where T: TicketRepository + PolicyAuthorizer + AuditPublisher {}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OpenIncidentCommand {
    pub tenant_id: TenantId,
    pub ticket_id: TicketId,
    pub requester_id: RequesterId,
    pub title: String,
    pub priority: Priority,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecomputeSlaCommand {
    pub tenant_id: TenantId,
    pub ticket_id: TicketId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ApproveChangeCommand {
    pub tenant_id: TenantId,
    pub ticket_id: TicketId,
    pub change_id: ChangeId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UsecaseReceipt {
    pub tenant_id: TenantId,
    pub ticket_id: TicketId,
    pub audit_event: AuditEventKind,
    pub status: TicketStatus,
}

pub struct OpenIncident;

impl OpenIncident {
    pub fn execute(
        ports: &mut impl ItsmPorts,
        command: OpenIncidentCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::IncidentOpen)?;
        let ticket = IncidentTicket::new(
            command.tenant_id.clone(),
            command.ticket_id.clone(),
            command.requester_id,
            command.title,
            command.priority,
            TicketStatus::Draft,
        )
        .open()?;
        ticket.validate()?;
        let ticket = ports.put_ticket(ticket)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::IncidentOpened,
            command.ticket_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: ticket.tenant_id,
            ticket_id: ticket.ticket_id,
            audit_event: AuditEventKind::IncidentOpened,
            status: ticket.status,
        })
    }
}

pub struct RecomputeSla;

impl RecomputeSla {
    pub fn execute(
        ports: &mut impl ItsmPorts,
        command: RecomputeSlaCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::SlaRecompute)?;
        let ticket = ports
            .get_ticket(&command.tenant_id, &command.ticket_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "ticket_repository",
            })?
            .recompute_sla()?;
        let ticket = ports.put_ticket(ticket)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::SlaBreached,
            command.ticket_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: ticket.tenant_id,
            ticket_id: ticket.ticket_id,
            audit_event: AuditEventKind::SlaBreached,
            status: ticket.status,
        })
    }
}

pub struct ApproveChange;

impl ApproveChange {
    pub fn execute(
        ports: &mut impl ItsmPorts,
        command: ApproveChangeCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::ChangeApprove)?;
        let mut ticket = ports
            .get_ticket(&command.tenant_id, &command.ticket_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "ticket_repository",
            })?;
        ticket.status = TicketStatus::ChangePending;
        let ticket = ticket.approve_change()?;
        let ticket = ports.put_ticket(ticket)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::ChangeApproved,
            command.change_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: ticket.tenant_id,
            ticket_id: ticket.ticket_id,
            audit_event: AuditEventKind::ChangeApproved,
            status: ticket.status,
        })
    }
}

pub struct ItsmService<P> {
    ports: P,
}

impl<P> ItsmService<P>
where
    P: ItsmPorts,
{
    pub fn new(ports: P) -> Self {
        Self { ports }
    }

    pub fn open_incident(&mut self, command: OpenIncidentCommand) -> ServiceResult<UsecaseReceipt> {
        OpenIncident::execute(&mut self.ports, command)
    }

    pub fn recompute_sla(&mut self, command: RecomputeSlaCommand) -> ServiceResult<UsecaseReceipt> {
        RecomputeSla::execute(&mut self.ports, command)
    }

    pub fn approve_change(
        &mut self,
        command: ApproveChangeCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ApproveChange::execute(&mut self.ports, command)
    }

    pub fn into_ports(self) -> P {
        self.ports
    }
}
