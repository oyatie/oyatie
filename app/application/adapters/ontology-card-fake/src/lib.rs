//! Fixture-backed `OntologyCardSource`: deterministic, in-process, no I/O.

use application_ontology_card::{OntologyCardError, OntologyCardFacts, OntologyCardSource};

/// Answers one fixed set of facts; `POLICY_VERSION` is a value no facade emits.
#[derive(Clone, Copy, Debug)]
pub struct FixtureOntologyCardSource;

impl FixtureOntologyCardSource {
    pub const POLICY_VERSION: &'static str = "fixture-policy-0000";
}

impl OntologyCardSource for FixtureOntologyCardSource {
    fn ontology_card_facts(&self) -> Result<OntologyCardFacts, OntologyCardError> {
        Ok(OntologyCardFacts {
            policy_version: Self::POLICY_VERSION.to_owned(),
            served_tenants: 3,
            projection_lag: 0,
            poisoned_entries: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use application_ontology_card::OntologyCardSource;

    use super::FixtureOntologyCardSource;

    #[test]
    fn the_fixture_answers_the_same_facts_on_every_call() {
        let source = FixtureOntologyCardSource;
        let first = source.ontology_card_facts().expect("fixture answers");
        let second = source.ontology_card_facts().expect("fixture answers");
        assert_eq!(first, second);
        assert_eq!(
            first.policy_version,
            FixtureOntologyCardSource::POLICY_VERSION
        );
        assert_eq!(first.served_tenants, 3);
        assert_eq!(first.projection_lag, 0);
        assert_eq!(first.poisoned_entries, 0);
    }
}
