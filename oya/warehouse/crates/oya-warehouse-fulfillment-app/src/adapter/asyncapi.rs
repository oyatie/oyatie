use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AsyncApiChannel {
    pub channel: &'static str,
    pub direction: ChannelDirection,
    pub message: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelDirection {
    Publish,
    Subscribe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AsyncApiMessage {
    pub tenant_id: String,
    pub message_type: String,
    pub payload_json: serde_json::Value,
}

pub struct AsyncApiHandler;

impl AsyncApiHandler {
    pub fn channels() -> Vec<AsyncApiChannel> {
        vec![
            AsyncApiChannel {
                channel: "warehouse.command.accepted.v1",
                direction: ChannelDirection::Publish,
                message: "CommandAccepted",
            },
            AsyncApiChannel {
                channel: "warehouse.reconciliation.queued.v1",
                direction: ChannelDirection::Publish,
                message: "ReconciliationQueued",
            },
            AsyncApiChannel {
                channel: "warehouse.governance.hold-applied.v1",
                direction: ChannelDirection::Publish,
                message: "GovernanceHoldApplied",
            },
            AsyncApiChannel {
                channel: "warehouse.inventory.changed.v1",
                direction: ChannelDirection::Subscribe,
                message: "InventoryChanged",
            },
            AsyncApiChannel {
                channel: "warehouse.transport.changed.v1",
                direction: ChannelDirection::Subscribe,
                message: "TransportChanged",
            },
        ]
    }

    pub fn handle(_message: AsyncApiMessage) -> Result<()> {
        Err(ServiceError::contract_stub("asyncapi"))
    }
}

pub fn validate_channels(channels: &[AsyncApiChannel]) -> Result<()> {
    let has_publish = channels
        .iter()
        .any(|channel| channel.direction == ChannelDirection::Publish);
    let has_subscribe = channels
        .iter()
        .any(|channel| channel.direction == ChannelDirection::Subscribe);
    if has_publish && has_subscribe {
        Ok(())
    } else {
        Err(ServiceError::validation(
            "asyncapi_channels",
            "scaffold must include publish and subscribe channels",
        ))
    }
}
