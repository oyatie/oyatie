use crate::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

fn identity_text(value: &str) -> bool {
    !value.is_empty()
        && value != "00000000-0000-0000-0000-000000000000"
        && value.len() <= 36
        && !value.chars().any(char::is_whitespace)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationKind {
    Bot,
    Bridge,
    Service,
    Tool,
    Widget,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationCapability {
    ReadMessages,
    SendMessages,
    InvokeTools,
    ReadObjects,
    WriteObjects,
    RenderWidget,
}

impl IntegrationCapability {
    pub fn action(self) -> &'static str {
        match self {
            Self::ReadMessages => "messenger.integration.messages.read",
            Self::SendMessages => "messenger.integration.messages.send",
            Self::InvokeTools => "messenger.integration.tools.invoke",
            Self::ReadObjects => "messenger.integration.objects.read",
            Self::WriteObjects => "messenger.integration.objects.write",
            Self::RenderWidget => "messenger.integration.widgets.render",
        }
    }
}

/// A named operator service, never a callback URL from room content. Chat
/// access requires its own visible Matrix encryption participant.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstallationSpec {
    pub id: String,
    pub room: String,
    pub service: String,
    pub workload: String,
    pub matrix_user: Option<String>,
    pub kind: IntegrationKind,
    pub capabilities: BTreeSet<IntegrationCapability>,
}

fn bounded(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
fn matrix_id(value: &str, prefix: char) -> bool {
    bounded(value, 512)
        && value.starts_with(prefix)
        && value[1..]
            .split_once(':')
            .is_some_and(|(name, server)| !name.is_empty() && !server.is_empty())
        && !value.chars().any(char::is_whitespace)
}

impl InstallationSpec {
    pub fn validate(&self) -> Result<(), Error> {
        if !identity_text(&self.id)
            || !matrix_id(&self.room, '!')
            || !bounded(&self.workload, 256)
            || !bounded(&self.service, 128)
            || !self
                .service
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || self.capabilities.is_empty()
            || self
                .matrix_user
                .as_ref()
                .is_some_and(|id| !matrix_id(id, '@'))
            || (self
                .capabilities
                .contains(&IntegrationCapability::ReadMessages)
                || self
                    .capabilities
                    .contains(&IntegrationCapability::SendMessages))
                && self.matrix_user.is_none()
        {
            return Err(Error::Invalid("Invalid scoped service installation".into()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Installation {
    pub spec: InstallationSpec,
    pub generation: u64,
    pub enabled: bool,
}

/// The unchanged request is the idempotency unit. A retry may not choose a new
/// installation generation or erase prior bridge hops.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Delivery {
    pub installation: String,
    pub generation: u64,
    pub event: String,
    pub capability: IntegrationCapability,
    pub body: Value,
    #[serde(default)]
    pub via: Vec<String>,
}
impl Delivery {
    pub fn validate(&self) -> Result<(), Error> {
        if !identity_text(&self.installation)
            || self.generation == 0
            || self.generation > i64::MAX as u64
            || !bounded(&self.event, 512)
            || !self.body.is_object()
            || self.via.len() >= 8
            || self.via.iter().any(|id| !identity_text(id))
            || self.via.iter().collect::<BTreeSet<_>>().len() != self.via.len()
            || serde_json::to_vec(self).map_or(true, |bytes| bytes.len() > 65_536)
        {
            return Err(Error::Invalid("Invalid service delivery".into()));
        }
        Ok(())
    }
    pub fn authorize(
        &self,
        spec: &InstallationSpec,
        generation: u64,
        enabled: bool,
    ) -> Result<(), Error> {
        self.validate()?;
        spec.validate()?;
        if !enabled
            || self.installation != spec.id
            || self.generation != generation
            || !spec.capabilities.contains(&self.capability)
            || self.via.contains(&spec.id)
        {
            return Err(Error::Denied);
        }
        Ok(())
    }
}
