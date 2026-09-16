use crate::Error;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

fn identity_text(value: &str) -> bool {
    !value.is_empty()
        && value != "00000000-0000-0000-0000-000000000000"
        && value.len() <= 36
        && !value.chars().any(char::is_whitespace)
}

/// Console Company (RLS cell), never the parent Group account or a room grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsoleObjectRef {
    pub company: String,
    pub object_type: String,
    pub instance: String,
    pub revision: u32,
}

impl ConsoleObjectRef {
    pub fn validate(&self) -> Result<(), Error> {
        if !identity_text(&self.company)
            || !identity_text(&self.object_type)
            || !identity_text(&self.instance)
            || self.revision == 0
        {
            return Err(Error::Invalid("invalid Console reference".into()));
        }
        Ok(())
    }
    pub fn message(&self) -> Result<String, Error> {
        self.validate()?;
        Ok(json!({"console_object":self}).to_string())
    }
    pub fn from_message(body: &str) -> Option<Self> {
        if body.len() > crate::MAX_MESSAGE_BYTES {
            return None;
        }
        let value: Value = serde_json::from_str(body).ok()?;
        let object: Self = serde_json::from_value(value.get("console_object")?.clone()).ok()?;
        object.validate().ok()?;
        Some(object)
    }
}

/// Same command drives Console's canonical preflight and execute. Keep this
/// complete value in the encrypted outbox until its receipt is acknowledged.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConsoleCommand {
    pub object: ConsoleObjectRef,
    pub action: String,
    pub command_id: String,
    pub params: Value,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub checklist_all_acknowledged: Option<bool>,
    #[serde(default)]
    pub four_eyes_request_ref: Option<String>,
}

impl ConsoleCommand {
    pub fn validate(&self) -> Result<(), Error> {
        self.object.validate()?;
        if !identity_text(&self.command_id)
            || self.action.is_empty()
            || self.action.len() > 256
            || matches!(self.action.as_str(), "." | "..")
            || !self
                .action
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
            || !self.params.is_object()
            || self.reason.as_ref().is_some_and(|s| s.len() > 4096)
            || serde_json::to_vec(self).map_or(true, |v| v.len() > 65_536)
        {
            return Err(Error::Invalid("invalid Console action".into()));
        }
        Ok(())
    }
    pub fn request(&self) -> Result<Value, Error> {
        self.validate()?;
        Ok(json!({
            "object_type_id":self.object.object_type,
            "instance_id":self.object.instance,
            "expected_revision":self.object.revision,
            "command_id":self.command_id,
            "params":self.params,
            "reason":self.reason,
            "checklist_all_acknowledged":self.checklist_all_acknowledged,
            "four_eyes_request_ref":self.four_eyes_request_ref,
        }))
    }
}
