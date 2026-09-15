use std::collections::BTreeMap;
use std::net::SocketAddr;

use messenger_domain::Error;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    pub policy_origin: String,
    pub policy_version: String,
    pub workload_identity_file: Option<String>,
    pub tenants: BTreeMap<String, Tenant>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Error> {
        let bytes = std::fs::read(path).map_err(|_| Error::Invalid("config unreadable".into()))?;
        serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("invalid config".into()))
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tenant {
    pub audit_bot: String,
    pub subjects: BTreeMap<String, String>,
}
