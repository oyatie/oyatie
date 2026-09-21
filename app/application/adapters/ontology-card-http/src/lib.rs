//! `OntologyCardSource` over HTTP: GET `<base>/statusz` on the Foundry facade.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::time::Duration;

use application_ontology_card::{OntologyCardError, OntologyCardFacts, OntologyCardSource};
use serde::Deserialize;

pub struct HttpOntologyCardSource {
    statusz: reqwest::Url,
    token: String, // data_class: SECRET
}

/// The four card fields of the facade's `StatusBody`; every other field is ignored.
#[derive(Deserialize)]
struct StatusBody {
    policy_version: String,
    served_tenants: u64,
    projection_lag: u64,
    poisoned_entries: u64,
}

impl HttpOntologyCardSource {
    pub const TIMEOUT: Duration = Duration::from_secs(2);

    /// # Errors
    /// [`OntologyCardError::Unavailable`] when `base_url` is not an `http` or
    /// `https` URL or `token` is empty.
    pub fn new(base_url: &str, token: &str) -> Result<Self, OntologyCardError> {
        let statusz = reqwest::Url::parse(&format!("{}/statusz", base_url.trim_end_matches('/')))
            .map_err(|error| unavailable(format!("ontology status url: {error}")))?;
        if !matches!(statusz.scheme(), "http" | "https") {
            return Err(unavailable("ontology status url must be http or https"));
        }
        if token.is_empty() {
            return Err(unavailable("ontology status token is empty"));
        }
        Ok(Self {
            statusz,
            token: token.to_owned(),
        })
    }

    fn fetch(&self) -> Result<OntologyCardFacts, OntologyCardError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Self::TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| unavailable(format!("http client: {error}")))?;
        let response = client
            .get(self.statusz.clone())
            .bearer_auth(&self.token)
            .send()
            .map_err(|error| unavailable(format!("GET /statusz: {}", error.without_url())))?;
        let status = response.status();
        if !status.is_success() {
            return Err(unavailable(format!("GET /statusz answered {status}")));
        }
        let body = response
            .bytes()
            .map_err(|error| unavailable(format!("GET /statusz body: {}", error.without_url())))?;
        let body: StatusBody = serde_json::from_slice(&body)
            .map_err(|error| unavailable(format!("/statusz body: {:?}", error.classify())))?;
        Ok(OntologyCardFacts {
            policy_version: body.policy_version,
            served_tenants: body.served_tenants,
            projection_lag: body.projection_lag,
            poisoned_entries: body.poisoned_entries,
        })
    }
}

fn unavailable(reason: impl Into<String>) -> OntologyCardError {
    OntologyCardError::Unavailable(reason.into())
}

impl OntologyCardSource for HttpOntologyCardSource {
    /// Blocks the caller for the read (up to `TIMEOUT` per stage: connect+send,
    /// then body) on a thread of its own: the callers sit on Tokio workers, where
    /// a blocking client's runtime guard panics in debug builds.
    // ponytail: a thread and a client per read, and the worker waits; pool them
    // and read off-worker if the card is polled.
    fn ontology_card_facts(&self) -> Result<OntologyCardFacts, OntologyCardError> {
        std::thread::scope(|scope| scope.spawn(|| self.fetch()).join())
            .unwrap_or_else(|_| Err(unavailable("GET /statusz: the request thread panicked")))
    }
}

#[cfg(test)]
mod tests;
