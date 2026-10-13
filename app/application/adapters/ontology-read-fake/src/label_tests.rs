use application_ontology_read::{OntologyReadError, OntologyReadSource};

use super::FixtureOntologyReadSource;

const TENANT: &str = FixtureOntologyReadSource::TENANT;

#[tokio::test]
async fn native_labels_are_explicit_fixture_text_and_preserve_tenant_and_source_refusals() {
    let source = FixtureOntologyReadSource::sample();
    let label = source.type_labels(TENANT).await.unwrap().remove(0);
    assert_eq!(label.display_name, "Fixture record");
    assert_eq!(
        label.entity_type,
        source.schemas(TENANT).await.unwrap()[0].entity_type
    );
    assert_eq!(label.revision, 2);
    assert_eq!(label.validate(), Ok(()));
    assert_eq!(
        source.type_labels("ten_foreign").await,
        Err(OntologyReadError::Forbidden)
    );
    for error in [
        OntologyReadError::Unauthenticated,
        OntologyReadError::Forbidden,
        OntologyReadError::SourceUnavailable,
    ] {
        assert_eq!(
            FixtureOntologyReadSource::refusing(error)
                .type_labels(TENANT)
                .await,
            Err(error)
        );
    }
}
