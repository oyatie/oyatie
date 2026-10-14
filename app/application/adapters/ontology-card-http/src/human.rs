use std::time::Duration;

use application_ontology_card::OntologyCardFacts;
use application_ontology_read::OntologyReadError;
use reqwest::{Client, StatusCode, Url, header};

mod model;
mod query;
pub use model::HumanOntologyModelSource;
mod reads;
mod transport;
pub use reads::HumanOntologyReadSource;

#[cfg(test)]
mod tests;

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
        let mut cookie = header::HeaderValue::from_str(session_cookie)
            .map_err(|_| HumanOntologyRefusal::Unavailable)?;
        cookie.set_sensitive(true);
        let response = self.request(url, &cookie).await.map_err(card_refusal)?;
        match response.status() {
            StatusCode::UNAUTHORIZED => return Err(HumanOntologyRefusal::Unauthenticated),
            StatusCode::FORBIDDEN => return Err(HumanOntologyRefusal::Forbidden),
            StatusCode::OK => {}
            _ => return Err(HumanOntologyRefusal::Unavailable),
        }
        let body: StatusBody = transport::decode(response, 16 * 1024)
            .await
            .map_err(card_refusal)?;
        Ok(body.into_facts())
    }
}

fn card_refusal(error: OntologyReadError) -> HumanOntologyRefusal {
    match error {
        OntologyReadError::Unauthenticated => HumanOntologyRefusal::Unauthenticated,
        OntologyReadError::Forbidden => HumanOntologyRefusal::Forbidden,
        _ => HumanOntologyRefusal::Unavailable,
    }
}
