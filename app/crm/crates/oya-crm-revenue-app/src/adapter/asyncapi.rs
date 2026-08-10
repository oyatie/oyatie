//! CRM AsyncAPI adapter.
//!
//! ## AUTH-005 fail-closed seam (ADR-0603)
//!
//! [`AsyncApiMessage::tenant_id`] is a CALLER/PRODUCER-SUPPLIED field — it is
//! **non-authoritative** (grants nothing, never selects the resource tenant).
//! An inbound message that drives a CRM MUTATION MUST go through
//! [`AsyncApiHandler::handle`], which runs the [`crate::authz`] fail-closed gate
//! FIRST against a principal derived from the broker's verified producer
//! identity (SASL/mTLS principal or a signed envelope), NOT from the message
//! body. Subscribe-only projection channels that mutate nothing are exempt; the
//! command-bearing channels are not.
//!
//! ## Edge obligation (dead-until-edge)
//!
//! The business logic still returns `contract_stub`. The broker consumer that
//! binds a real subscription MUST derive the verified producer credential from
//! the transport (NOT the message body), refuse to consume without a
//! [`crate::authz::CrmAuthzProvider`], and run the gate before dispatch.

use crate::authz::{authorize_crm_command, AuthorizedCrmContext, CallerCredential, CrmAction, CrmAuthzProvider};
use crate::domain::Capability;
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AsyncApiChannel { pub channel: &'static str, pub direction: ChannelDirection, pub message: &'static str }
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelDirection { Publish, Subscribe }
/// AsyncAPI message DTO. NOTE: `tenant_id` is non-authoritative
/// producer-supplied data (see module docs / ADR-0603); it is structurally never
/// read by the gate and never selects the resource tenant.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AsyncApiMessage { pub tenant_id: String, pub message_type: String, pub payload_json: serde_json::Value }

pub struct AsyncApiHandler;
impl AsyncApiHandler {
    pub fn channels() -> Vec<AsyncApiChannel> {
        vec![
            AsyncApiChannel { channel: "crm.command.accepted.v1", direction: ChannelDirection::Publish, message: "CommandAccepted" },
            AsyncApiChannel { channel: "crm.reconciliation.queued.v1", direction: ChannelDirection::Publish, message: "ReconciliationQueued" },
            AsyncApiChannel { channel: "crm.governance.hold-applied.v1", direction: ChannelDirection::Publish, message: "GovernanceHoldApplied" },
            AsyncApiChannel { channel: "crm.customer.changed.v1", direction: ChannelDirection::Subscribe, message: "CustomerChanged" },
            AsyncApiChannel { channel: "crm.finance.approval-changed.v1", direction: ChannelDirection::Subscribe, message: "FinanceApprovalChanged" },
        ]
    }

    /// Handle an inbound CRM-mutating message through the fail-closed authz gate.
    ///
    /// `credential` is the verified producer credential from the broker
    /// transport (SASL/mTLS / signed envelope), NOT the message body.
    /// `capability` is server-side channel metadata. The gate verifies the
    /// producer and authorizes the action against the VERIFIED tenant before any
    /// business logic.
    ///
    /// The resource scope is bound by [`Self::resolve_scope`] from the VERIFIED
    /// tenant, NEVER from `message.tenant_id` — a forged envelope tenant is
    /// structurally ignored.
    ///
    /// # Errors
    /// `Unauthenticated`/`Forbidden` on a failed gate; `ContractStub` once
    /// authorized.
    pub fn handle(provider: &CrmAuthzProvider, credential: &CallerCredential, capability: Capability, message: AsyncApiMessage) -> Result<()> {
        let scope = Self::resolve_scope(provider, credential, capability, &message)?;
        let _ = scope.tenant_id();
        Err(ServiceError::contract_stub("asyncapi"))
    }

    /// Run the fail-closed gate and return the scope the handler MUST act within.
    /// The returned context's tenant is the VERIFIED tenant; `message.tenant_id`
    /// is structurally discarded.
    ///
    /// # Errors
    /// `Unauthenticated`/`Forbidden` on a failed gate.
    pub fn resolve_scope(provider: &CrmAuthzProvider, credential: &CallerCredential, capability: Capability, _message: &AsyncApiMessage) -> Result<AuthorizedCrmContext> {
        authorize_crm_command(provider, credential, CrmAction(capability)).map_err(ServiceError::from)
    }
}

pub fn validate_channels(channels: &[AsyncApiChannel]) -> Result<()> {
    let has_publish = channels.iter().any(|channel| channel.direction == ChannelDirection::Publish);
    let has_subscribe = channels.iter().any(|channel| channel.direction == ChannelDirection::Subscribe);
    if has_publish && has_subscribe { Ok(()) } else { Err(ServiceError::validation("asyncapi_channels", "scaffold must include publish and subscribe channels")) }
}
