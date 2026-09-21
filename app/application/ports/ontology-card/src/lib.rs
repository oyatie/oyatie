//! What the console's Ontology module card needs from the wedge product.
//!
//! Every field is an aggregate `app/foundry/facade/ontology-app` serves on
//! `/statusz`; nothing tenant-scoped is carried.

use std::fmt;

/// The numbers the card renders.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyCardFacts {
    /// `/statusz` `policy_version`: the Cedar policy set the facade loaded.
    pub policy_version: String, // data_class: INTERNAL_ONLY
    /// `/statusz` `served_tenants`, `/metrics` `foundry_served_tenants`.
    pub served_tenants: u64, // data_class: INTERNAL_ONLY
    /// `/statusz` `projection_lag`, `/metrics` `foundry_projection_lag`.
    pub projection_lag: u64, // data_class: INTERNAL_ONLY
    /// `/statusz` `poisoned_entries`, `/metrics` `foundry_poisoned_entries`.
    pub poisoned_entries: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OntologyCardError {
    /// The source could not answer; the card says so rather than showing a guess.
    Unavailable(String),
}

impl fmt::Display for OntologyCardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(reason) => write!(f, "ontology status unavailable: {reason}"),
        }
    }
}

/// Where the card's facts come from.
pub trait OntologyCardSource {
    /// # Errors
    /// [`OntologyCardError::Unavailable`] when no facts can be read.
    fn ontology_card_facts(&self) -> Result<OntologyCardFacts, OntologyCardError>;
}

#[cfg(test)]
mod tests {
    use super::{OntologyCardError, OntologyCardFacts, OntologyCardSource};

    struct Held(Result<OntologyCardFacts, OntologyCardError>);

    impl OntologyCardSource for Held {
        fn ontology_card_facts(&self) -> Result<OntologyCardFacts, OntologyCardError> {
            self.0.clone()
        }
    }

    #[test]
    fn a_source_round_trips_the_facts_it_holds() {
        let facts = OntologyCardFacts {
            policy_version: "policy-2026-09-21".to_owned(),
            served_tenants: 3,
            projection_lag: 7,
            poisoned_entries: 1,
        };
        let held = Held(Ok(facts.clone()));
        assert_eq!(held.ontology_card_facts(), Ok(facts));
    }

    #[test]
    fn a_source_round_trips_its_refusal() {
        let held = Held(Err(OntologyCardError::Unavailable(
            "listener down".to_owned(),
        )));
        let error = held.ontology_card_facts().unwrap_err();
        assert_eq!(
            error,
            OntologyCardError::Unavailable("listener down".to_owned())
        );
        assert_eq!(
            error.to_string(),
            "ontology status unavailable: listener down"
        );
    }
}
