#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::time::Duration;

use messenger_domain::{Error, ObjectRef};
use messenger_foundry_api::Foundry;

const MAX_RESPONSE_BYTES: usize = 1_048_576;

pub struct HttpFoundry {
    client: reqwest::Client,
    origin: reqwest::Url,
    tenant: String,
}

impl HttpFoundry {
    pub fn new(origin: &str, tenant: &str) -> Result<Self, Error> {
        if tenant.is_empty() {
            return Err(Error::Invalid("Foundry tenant required".into()));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .build()
            .map_err(|_| Error::Denied)?;
        Ok(Self {
            origin: service_origin(origin)?,
            tenant: tenant.into(),
            client,
        })
    }

    fn validate(&self, object: &ObjectRef, credential: &str) -> Result<(), Error> {
        object.validate()?;
        if object.tenant != self.tenant
            || credential.is_empty()
            || credential.len() > 8192
            || credential.chars().any(char::is_control)
        {
            return Err(Error::Denied);
        }
        Ok(())
    }
}

fn service_origin(value: &str) -> Result<reqwest::Url, Error> {
    let url =
        reqwest::Url::parse(value).map_err(|_| Error::Invalid("invalid server URL".into()))?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(Error::Invalid(
            "use an HTTPS server origin (HTTP only on loopback)".into(),
        ));
    }
    Ok(url)
}

fn map_status(status: reqwest::StatusCode, invoke: bool) -> Result<(), Error> {
    if status.is_success() {
        return Ok(());
    }
    let code = status.as_u16();
    if invoke && matches!(code, 400 | 401 | 403) {
        return Err(Error::ActionRejected);
    }
    if !invoke && matches!(code, 401 | 403) {
        return Err(Error::Denied);
    }
    if status.is_server_error() || matches!(code, 408 | 429) {
        let detail = if invoke {
            "Foundry action outcome is unknown"
        } else {
            "Foundry response unavailable"
        };
        return Err(Error::Unavailable(detail.into()));
    }
    Err(Error::Denied)
}

async fn bounded_json(mut response: reqwest::Response) -> Result<serde_json::Value, Error> {
    let oversized = || Error::Unavailable("Foundry response exceeds 1 MiB".into());
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(oversized());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| Error::Unavailable("Foundry response unavailable".into()))?
    {
        if chunk.len() > MAX_RESPONSE_BYTES - bytes.len() {
            return Err(oversized());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| Error::Unavailable("Foundry response unavailable".into()))
}

impl Foundry for HttpFoundry {
    async fn read(&self, credential: &str, object: &ObjectRef) -> Result<serde_json::Value, Error> {
        self.validate(object, credential)?;
        let mut url = self.origin.clone();
        url.path_segments_mut().map_err(|_| Error::Denied)?.extend([
            "v1",
            "objects",
            &object.object,
        ]);
        url.query_pairs_mut()
            .append_pair("revision", &object.revision.to_string());
        let response = self
            .client
            .get(url)
            .bearer_auth(credential)
            .send()
            .await
            .map_err(|_| Error::Unavailable("Foundry response unavailable".into()))?;
        map_status(response.status(), false)?;
        bounded_json(response).await
    }

    async fn invoke(
        &self,
        credential: &str,
        object: &ObjectRef,
        action: &str,
        idempotency_key: &str,
        occurred_at: u64,
        properties: BTreeMap<String, String>,
    ) -> Result<(), Error> {
        self.validate(object, credential)
            .map_err(|_| Error::ActionRejected)?;
        if action.is_empty() || idempotency_key.is_empty() {
            return Err(Error::ActionRejected);
        }
        let mut url = self.origin.clone();
        url.set_path("/v1/actions");
        let response = self
            .client
            .post(url)
            .bearer_auth(credential)
            .json(&serde_json::json!({
                "object_ref": object.object,
                "action_type": action,
                "idempotency_key": idempotency_key,
                "occurred_at_epoch_seconds": occurred_at,
                "properties": properties
            }))
            .send()
            .await
            .map_err(|_| Error::Unavailable("Foundry action outcome is unknown".into()))?;
        map_status(response.status(), true)
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpFoundry, service_origin};

    #[test]
    fn tenant_and_origin_are_required() {
        assert!(HttpFoundry::new("https://foundry.example", "").is_err());
        assert!(HttpFoundry::new("http://example.org", "acme").is_err());
        assert!(HttpFoundry::new("https://foundry.example", "acme").is_ok());
        assert!(HttpFoundry::new("http://127.0.0.1:1", "acme").is_ok());
    }

    #[test]
    fn remote_http_and_non_origins_are_refused() {
        for url in [
            "http://example.org",
            "https://u:p@example.org",
            "https://example.org/a",
            "file:///tmp/a",
        ] {
            assert!(service_origin(url).is_err(), "{url}");
        }
        for url in [
            "https://foundry.example",
            "http://127.0.0.1:8008",
            "http://[::1]:8008",
        ] {
            assert!(service_origin(url).is_ok(), "{url}");
        }
    }
}
