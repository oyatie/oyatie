#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod body;
mod origin;

use std::collections::BTreeMap;
use std::time::Duration;

use body::bounded_json;
use messenger_domain::{Error, ObjectRef};
use messenger_foundry_api::Foundry;
use origin::service_origin;

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
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| Error::Denied)?;
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
    ) -> Result<serde_json::Value, Error> {
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
        if matches!(response.status().as_u16(), 400 | 401 | 403) {
            return Err(Error::ActionRejected);
        }
        bounded_json(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::HttpFoundry;

    #[test]
    fn tenant_and_origin_are_required() {
        assert!(HttpFoundry::new("https://foundry.example", "").is_err());
        assert!(HttpFoundry::new("http://example.org", "acme").is_err());
        assert!(HttpFoundry::new("https://foundry.example", "acme").is_ok());
        assert!(HttpFoundry::new("http://127.0.0.1:1", "acme").is_ok());
    }
}
