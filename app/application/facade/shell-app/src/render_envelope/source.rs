//! Where the served Ontology card's source comes from: the process
//! environment, or a refusal that names what is missing. Never the fixture.

use std::env;

use application_ontology_card::{OntologyCardError, OntologyCardFacts, OntologyCardSource};
use application_ontology_card_http::HttpOntologyCardSource;

/// Base URL of the Foundry ontology facade, e.g. `http://127.0.0.1:8090`.
pub const STATUS_URL_VAR: &str = "OYATIE_CONSOLE_ONTOLOGY_STATUS_URL";
/// The operator bearer the facade's `/statusz` authorizes.
pub const STATUS_TOKEN_VAR: &str = "OYATIE_CONSOLE_ONTOLOGY_STATUS_TOKEN"; // data_class: SECRET

pub(super) fn configured_source() -> Box<dyn OntologyCardSource> {
    source_from(non_empty(STATUS_URL_VAR), non_empty(STATUS_TOKEN_VAR))
}

/// The HTTP adapter when both values are present; otherwise a source that
/// refuses with the name of the missing value.
pub fn source_from(url: Option<String>, token: Option<String>) -> Box<dyn OntologyCardSource> {
    match (url, token) {
        (Some(url), Some(token)) => match HttpOntologyCardSource::new(&url, &token) {
            Ok(source) => Box::new(source),
            Err(OntologyCardError::Unavailable(reason)) => Box::new(Refusal(reason)),
        },
        (None, _) => Box::new(Refusal("ontology status source not configured".to_owned())),
        (Some(_), None) => Box::new(Refusal("ontology status token not configured".to_owned())),
    }
}

fn non_empty(variable: &str) -> Option<String> {
    env::var(variable).ok().filter(|value| !value.is_empty())
}

struct Refusal(String);

impl OntologyCardSource for Refusal {
    fn ontology_card_facts(&self) -> Result<OntologyCardFacts, OntologyCardError> {
        Err(OntologyCardError::Unavailable(self.0.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use application_ontology_card::{OntologyCardError, OntologyCardSource};

    use super::source_from;
    use crate::render_envelope::{OperatorContext, permitted_envelope_snapshot};

    fn refusal(url: Option<&str>, token: Option<&str>) -> String {
        match source_from(url.map(str::to_owned), token.map(str::to_owned)).ontology_card_facts() {
            Err(OntologyCardError::Unavailable(reason)) => reason,
            Ok(facts) => panic!("answered {facts:?}"),
        }
    }

    /// One `/statusz` answer over a loopback socket; the request is never
    /// inspected here, the adapter's own suite does that.
    fn listener(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request);
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        });
        base
    }

    #[test]
    fn nothing_configured_names_the_source_as_missing() {
        assert_eq!(refusal(None, None), "ontology status source not configured");
        assert_eq!(
            refusal(None, Some("token")),
            "ontology status source not configured"
        );
    }

    #[test]
    fn a_url_without_a_token_names_the_token_as_missing() {
        assert_eq!(
            refusal(Some("http://127.0.0.1:1"), None),
            "ontology status token not configured"
        );
    }

    #[test]
    fn a_url_the_adapter_refuses_carries_the_adapters_reason() {
        assert!(refusal(Some("ftp://host"), Some("token")).contains("http or https"));
    }

    #[test]
    fn a_configured_listener_puts_its_numbers_on_the_card() {
        let base = listener(
            r#"{"policy_version":"listener-policy-77","served_tenants":12,"projection_lag":4,"poisoned_entries":6,"observed_tenants":12}"#,
        );
        let source = source_from(Some(base), Some("operator-token".to_owned()));
        let envelope = permitted_envelope_snapshot(OperatorContext::TenantAdmin, &*source);
        let card = envelope
            .modules
            .iter()
            .find(|card| card.name == "Ontology")
            .expect("tenant admin sees the Ontology card");
        for fragment in ["listener-policy-77", "12 tenants", "lag 4", "poisoned 6"] {
            assert!(card.description.contains(fragment), "{}", card.description);
        }
    }
}
