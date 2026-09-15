use std::collections::BTreeMap;

use messenger_domain::{ENTERPRISE_ROOM_TYPE, Error};
use messenger_policy_api::Action;
use serde::Deserialize;
use serde_json::Value;

use crate::config::Tenant;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub event_id: String,
    pub tenant: String,
    pub audit_bot: String,
    pub event: Value,
    pub create: Value,
}

pub(crate) fn parc(
    tenants: &BTreeMap<String, Tenant>,
    request: &Admission,
) -> Result<(String, String, Action, String), Error> {
    let tenant = tenants.get(&request.tenant).ok_or(Error::Denied)?;
    let sender = request.event["sender"].as_str().ok_or(Error::Denied)?;
    if request.event["room_id"]
        .as_str()
        .is_none_or(|room| !room.starts_with('!'))
        || request.event_id.is_empty()
        || request.audit_bot != tenant.audit_bot
        || request.create["type"] != ENTERPRISE_ROOM_TYPE
        || request.create["m.federate"] != false
        || request.create["dev.oyatie.tenant"] != request.tenant
        || request.create["dev.oyatie.audit_bot"] != request.audit_bot
    {
        return Err(Error::Denied);
    }
    let resource = request.create["dev.oyatie.policy_resource"]
        .as_str()
        .map(str::trim)
        .filter(|resource| resource.len() <= 256)
        .ok_or(Error::Denied)?;
    let subject = tenant.subjects.get(sender).ok_or(Error::Denied)?;
    membership(tenant, &request.event)?;
    Ok((
        request.tenant.clone(),
        subject.clone(),
        action(&request.event)?,
        resource.to_owned(),
    ))
}

fn membership(tenant: &Tenant, event: &Value) -> Result<(), Error> {
    if event["type"] != "m.room.member" {
        return Ok(());
    }
    let target = event["state_key"].as_str().ok_or(Error::Denied)?;
    match event["content"]["membership"].as_str() {
        Some("leave" | "ban") => Ok(()),
        Some("invite" | "join" | "knock") if tenant.subjects.contains_key(target) => Ok(()),
        _ => Err(Error::Denied),
    }
}

fn action(event: &Value) -> Result<Action, Error> {
    match event["type"].as_str() {
        Some("m.room.create") => Ok(Action::CreateRoom),
        Some("m.room.encrypted") => Ok(Action::Send),
        Some("m.room.member") => match event["content"]["membership"].as_str() {
            Some("invite") => Ok(Action::Invite),
            Some("leave" | "ban" | "join" | "knock") => Ok(Action::ManageRoom),
            _ => Err(Error::Denied),
        },
        Some(
            "m.room.power_levels"
            | "m.room.encryption"
            | "m.room.join_rules"
            | "m.room.history_visibility"
            | "m.room.guest_access",
        ) => Ok(Action::ManageRoom),
        _ => Err(Error::Denied),
    }
}
