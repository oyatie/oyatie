#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{Error, InstallationSpec, IntegrationCapability};
use messenger_workload_api::{Workload, WorkloadIdentity};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const ROOM_RESOURCE_TYPE: &str = "messenger.room";
const UNREACHABLE: &str = "workload identity unreachable";
const UNUSABLE: &str = "workload identity response was unusable";
const BAD_REQUEST: &str = "invalid workload identity request";
const BAD_URL: &str = "invalid workload identity url";

#[derive(Clone)]
pub struct HttpWorkload {
    client: reqwest::Client,
    base: String,
    bearer: String,
}

impl HttpWorkload {
    pub fn new(base_url: impl Into<String>, bearer: impl Into<String>) -> Result<Self, Error> {
        Self::with_timeout(base_url, bearer, Duration::from_secs(5))
    }

    pub fn with_timeout(
        base_url: impl Into<String>,
        bearer: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, Error> {
        let base = base_url.into();
        let bearer = bearer.into();
        let url = reqwest::Url::parse(&base).map_err(|_| Error::Invalid(BAD_URL.into()))?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(Error::Invalid(BAD_URL.into()));
        }
        if bearer.is_empty() {
            return Err(Error::Invalid(BAD_REQUEST.into()));
        }
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|_| Error::Unavailable(UNREACHABLE.into()))?;
        Ok(Self {
            client,
            base: base.trim_end_matches('/').to_owned(),
            bearer,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizeBody<'a> {
    tenant_id: &'a str,
    workload_id: &'a str,
    owning_capability: &'a str,
    action: &'a str,
    resource: ResourceBody<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceBody<'a> {
    resource_type: &'a str,
    resource_id: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizeReply {
    effect: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IdentifyBody<'a> {
    token: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdentifyReply {
    tenant_id: String,
    workload_id: String,
    owning_capability: String,
    state: String,
}

fn require_text(value: &str) -> Result<(), Error> {
    if value.is_empty() {
        Err(Error::Invalid(BAD_REQUEST.into()))
    } else {
        Ok(())
    }
}

fn transport(_: reqwest::Error) -> Error {
    Error::Unavailable(UNREACHABLE.into())
}

fn map_status(status: StatusCode) -> Result<(), Error> {
    match status {
        StatusCode::OK => Ok(()),
        StatusCode::BAD_REQUEST => Err(Error::Invalid(BAD_REQUEST.into())),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::UNPROCESSABLE_ENTITY => {
            Err(Error::Denied)
        }
        StatusCode::SERVICE_UNAVAILABLE | StatusCode::GATEWAY_TIMEOUT => {
            Err(Error::Unavailable(UNREACHABLE.into()))
        }
        other if other.is_server_error() => Err(Error::Unavailable(UNREACHABLE.into())),
        _ => Err(Error::Denied),
    }
}

impl Workload for HttpWorkload {
    async fn authorize(
        &self,
        tenant: &str,
        spec: &InstallationSpec,
        capability: IntegrationCapability,
    ) -> Result<(), Error> {
        require_text(tenant)?;
        require_text(&spec.workload)?;
        require_text(&spec.service)?;
        require_text(&spec.room)?;
        let body = AuthorizeBody {
            tenant_id: tenant,
            workload_id: &spec.workload,
            owning_capability: &spec.service,
            action: capability.action(),
            resource: ResourceBody {
                resource_type: ROOM_RESOURCE_TYPE,
                resource_id: &spec.room,
            },
        };
        let response = self
            .client
            .post(self.url("/authorize"))
            .bearer_auth(&self.bearer)
            .json(&body)
            .send()
            .await
            .map_err(transport)?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(transport)?;
        map_status(status)?;
        let reply: AuthorizeReply =
            serde_json::from_slice(&bytes).map_err(|_| Error::Unavailable(UNUSABLE.into()))?;
        match reply.effect.as_str() {
            "ALLOW" => Ok(()),
            "DENY" => Err(Error::Denied),
            _ => Err(Error::Unavailable(UNUSABLE.into())),
        }
    }

    async fn identify(&self, token: &str) -> Result<WorkloadIdentity, Error> {
        require_text(token)?;
        let response = self
            .client
            .post(self.url("/tokens/validate"))
            .bearer_auth(&self.bearer)
            .json(&IdentifyBody { token })
            .send()
            .await
            .map_err(transport)?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(transport)?;
        map_status(status)?;
        let reply: IdentifyReply =
            serde_json::from_slice(&bytes).map_err(|_| Error::Unavailable(UNUSABLE.into()))?;
        require_text(&reply.tenant_id)
            .and(require_text(&reply.workload_id))
            .and(require_text(&reply.owning_capability))
            .and(require_text(&reply.state))
            .map_err(|_| Error::Unavailable(UNUSABLE.into()))?;
        Ok(WorkloadIdentity {
            tenant: reply.tenant_id,
            workload: reply.workload_id,
            service: reply.owning_capability,
            state: reply.state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_refuses_empty_bearer_and_non_http() {
        assert!(HttpWorkload::new("http://127.0.0.1:9", "").is_err());
        assert!(HttpWorkload::new("file:///tmp", "token").is_err());
        assert!(HttpWorkload::new("not-a-url", "token").is_err());
        assert!(HttpWorkload::new("http://127.0.0.1:9", "token").is_ok());
    }
}
