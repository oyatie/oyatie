use std::time::Duration;

use application_ontology_card::OntologyCardFacts;
use reqwest::{Client, StatusCode, Url, header};

use super::StatusBody;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HumanOntologyRefusal {
    Unauthenticated,
    Forbidden,
    Unavailable,
}

/// Async cookie transport for the Foundry route that authorizes human users.
/// This source never carries an operator token or caches a membership result.
#[derive(Clone)]
pub struct HumanOntologyCardSource {
    base: Url,
    client: Client,
}

impl HumanOntologyCardSource {
    /// Requires a fixed HTTPS origin without credentials, a query or a path.
    pub fn new(base: &str) -> Result<Self, HumanOntologyRefusal> {
        Self::configured(base, false)
    }

    /// Explicit development transport; HTTP is limited to localhost/127.0.0.1.
    pub fn for_loopback(base: &str) -> Result<Self, HumanOntologyRefusal> {
        Self::configured(base, true)
    }

    fn configured(base: &str, loopback: bool) -> Result<Self, HumanOntologyRefusal> {
        let base = Url::parse(base).map_err(|_| HumanOntologyRefusal::Unavailable)?;
        if !base.username().is_empty()
            || base.password().is_some()
            || base.path() != "/"
            || base.query().is_some()
            || base.fragment().is_some()
            || !(base.scheme() == "https"
                || (loopback
                    && base.scheme() == "http"
                    && matches!(base.host_str(), Some("localhost" | "127.0.0.1"))))
        {
            return Err(HumanOntologyRefusal::Unavailable);
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| HumanOntologyRefusal::Unavailable)?;
        Ok(Self { base, client })
    }

    /// Cookie and canonical tenant are supplied by the shell's identity guard.
    /// Foundry revalidates IAM membership and decides Surface::Use itself.
    pub async fn facts(
        &self,
        tenant: &str,
        session_cookie: &str,
    ) -> Result<OntologyCardFacts, HumanOntologyRefusal> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| HumanOntologyRefusal::Unavailable)?
            .extend(["v1", "tenants", tenant, "status"]);
        let mut response = self
            .client
            .get(url)
            .header(header::COOKIE, session_cookie)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| HumanOntologyRefusal::Unavailable)?;
        match response.status() {
            StatusCode::UNAUTHORIZED => return Err(HumanOntologyRefusal::Unauthenticated),
            StatusCode::FORBIDDEN => return Err(HumanOntologyRefusal::Forbidden),
            StatusCode::OK => {}
            _ => return Err(HumanOntologyRefusal::Unavailable),
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| HumanOntologyRefusal::Unavailable)?
        {
            if body.len() + chunk.len() > 16 * 1024 {
                return Err(HumanOntologyRefusal::Unavailable);
            }
            body.extend_from_slice(&chunk);
        }
        let body: StatusBody =
            serde_json::from_slice(&body).map_err(|_| HumanOntologyRefusal::Unavailable)?;
        Ok(body.into_facts())
    }
}
