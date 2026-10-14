use application_ontology_read::{ObjectPointQuery, OntologyReadError, OntologyReadSource};

use super::FixtureOntologyReadSource;

pub(super) fn point(object_ref: &str, revision: u32) -> ObjectPointQuery {
    ObjectPointQuery {
        entity_type: FixtureOntologyReadSource::ENTITY_TYPE.into(),
        object_ref: object_ref.into(),
        revision,
    }
}

#[tokio::test]
async fn point_type_is_required_and_mismatch_is_indistinguishable_from_absence() {
    let source = FixtureOntologyReadSource::sample();
    let mut query = ObjectPointQuery {
        entity_type: FixtureOntologyReadSource::ENTITY_TYPE.into(),
        object_ref: "ent_fixture_alpha".into(),
        revision: 1,
    };
    assert!(
        source
            .object(FixtureOntologyReadSource::TENANT, &query)
            .await
            .is_ok()
    );
    query.entity_type = "ety_other".into();
    query.revision = 99;
    assert_eq!(
        source
            .object(FixtureOntologyReadSource::TENANT, &query)
            .await,
        Err(OntologyReadError::AbsentObject)
    );
    query.entity_type.clear();
    assert_eq!(
        source
            .object(FixtureOntologyReadSource::TENANT, &query)
            .await,
        Err(OntologyReadError::InvalidQuery)
    );
    assert_eq!(
        source.object("ten_other", &query).await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        FixtureOntologyReadSource::refusing(OntologyReadError::Unauthenticated)
            .object(FixtureOntologyReadSource::TENANT, &query)
            .await,
        Err(OntologyReadError::Unauthenticated)
    );
}
