#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::Error;
use messenger_policy_api::{Action, Decision, Policy, Principal};
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct OyatiePolicy {
    client: reqwest::Client,
    endpoint: reqwest::Url,
    version: String,
}

impl OyatiePolicy {
    pub fn new(
        origin: &str,
        version: &str,
        workload_identity_pem: Option<&[u8]>,
    ) -> Result<Self, Error> {
        let mut endpoint = service_origin(origin)?;
        if version.trim().is_empty() {
            return Err(Error::Invalid("required policy version missing".into()));
        }
        if endpoint.scheme() == "https" && workload_identity_pem.is_none() {
            return Err(Error::Invalid(
                "hosted Policy requires workload mTLS identity".into(),
            ));
        }
        endpoint.set_path("/v1/authorize");
        let mut client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none());
        if let Some(pem) = workload_identity_pem {
            client = client.identity(
                reqwest::Identity::from_pem(pem)
                    .map_err(|_| Error::Invalid("invalid workload identity".into()))?,
            );
        }
        Ok(Self {
            client: client
                .build()
                .map_err(|_| Error::Unavailable("Policy client unavailable".into()))?,
            endpoint,
            version: version.into(),
        })
    }
}

#[derive(Deserialize)]
struct Response {
    request_id: String,
    decision_id: String,
    decision: String,
    policy_version: String,
    determining_policy_ids: Vec<String>,
    obligations: Vec<serde_json::Value>,
}

impl Policy for OyatiePolicy {
    async fn authorize(
        &self,
        principal: &Principal,
        action: Action,
        resource: &str,
    ) -> Result<Decision, Error> {
        if principal.tenant.is_empty() || principal.subject.is_empty() || resource.is_empty() {
            return Err(Error::Denied);
        }
        let id = request_id();
        let body = authorize_body(principal, action, resource, &id, &self.version)?;
        let unavailable = || Error::Unavailable("Policy service unavailable".into());
        let mut response = self
            .client
            .post(self.endpoint.clone())
            .json(&body)
            .send()
            .await
            .map_err(|_| unavailable())?;
        if response.status().is_server_error()
            || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            return Err(unavailable());
        }
        if !response.status().is_success() {
            return Err(Error::Denied);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| unavailable())? {
            if bytes.len() + chunk.len() > 65_536 {
                return Err(unavailable());
            }
            bytes.extend_from_slice(&chunk);
        }
        let response: Response = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
        if response.decision != "allow"
            || response.request_id != id
            || response.policy_version != self.version
            || response.decision_id.is_empty()
            || response.determining_policy_ids.is_empty()
            || !response.obligations.is_empty()
        {
            return Err(Error::Denied);
        }
        Ok(Decision {
            id: response.decision_id,
            version: response.policy_version,
        })
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

fn request_id() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let seq = NEXT.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("req-{nanos}-{seq}")
}

fn authorize_body(
    principal: &Principal,
    action: Action,
    resource: &str,
    request_id: &str,
    version: &str,
) -> Result<serde_json::Value, Error> {
    let principal_ref = serde_json::json!({
        "entity_type": "OyatieMessenger::Principal",
        "entity_id": serde_json::to_string(&(principal.tenant.as_str(), principal.subject.as_str()))
            .map_err(|_| Error::Denied)?,
    });
    let resource_ref = serde_json::json!({
        "entity_type": "OyatieMessenger::Room",
        "entity_id": serde_json::to_string(&(principal.tenant.as_str(), resource))
            .map_err(|_| Error::Denied)?,
    });
    Ok(serde_json::json!({"request": {
        "request_id": request_id,
        "tenant_id": principal.tenant,
        "principal": principal_ref,
        "action": action.slug(),
        "resource": resource_ref,
        "context": {
            "caller_tenant": principal.tenant,
            "caller_id": principal.subject,
        },
        "min_policy_version": version,
    }, "entities": [
        {"uid": principal_ref, "attributes": {"tenant_id": principal.tenant}, "parents": []},
        {"uid": resource_ref, "attributes": {"tenant_id": principal.tenant}, "parents": []},
    ]}))
}
