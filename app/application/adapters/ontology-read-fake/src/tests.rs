use application_ontology_read::{
    AuditDisposition, FilterValue, ObjectListQuery, OntologyReadError, OntologyReadSource,
    PropertyFilter, UpcastState,
};

use super::FixtureOntologyReadSource;

const TENANT: &str = FixtureOntologyReadSource::TENANT;
const ALPHA: &str = "ent_fixture_alpha";
const BETA: &str = "ent_fixture_beta";

fn query(revision: u32) -> ObjectListQuery {
    ObjectListQuery::first(FixtureOntologyReadSource::ENTITY_TYPE, revision)
}

#[tokio::test]
async fn undeclared_sample_links_stay_empty_and_tenant_and_source_refusals_are_preserved() {
    let source = FixtureOntologyReadSource::sample();
    assert_eq!(source.link_types(TENANT).await, Ok(vec![]));
    assert_eq!(
        source.link_types("ten_foreign").await,
        Err(OntologyReadError::Forbidden)
    );
    for error in [
        OntologyReadError::Unauthenticated,
        OntologyReadError::Forbidden,
        OntologyReadError::SourceUnavailable,
    ] {
        assert_eq!(
            FixtureOntologyReadSource::refusing(error)
                .link_types(TENANT)
                .await,
            Err(error)
        );
    }
}

#[tokio::test]
async fn current_schema_metadata_is_explicit_sample_and_preserves_refusals() {
    let source = FixtureOntologyReadSource::sample();
    let schema = source.schemas(TENANT).await.unwrap().remove(0);
    assert_eq!(schema.validate(), Ok(()));
    assert_eq!(schema.revision, 2);
    assert_eq!(schema.title_property.as_deref(), Some("name"));
    assert_eq!(schema.primary_key_property, None);
    assert_eq!(
        schema
            .properties
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>(),
        source.types(TENANT).await.unwrap()[0].properties
    );
    assert_eq!(schema.properties[0].data_class, "PII_IDENTIFYING");
    assert_eq!(schema.properties[3].value_type, None);
    assert_eq!(
        source.schemas("ten_foreign").await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        FixtureOntologyReadSource::refusing(OntologyReadError::SourceUnavailable)
            .schemas(TENANT)
            .await,
        Err(OntologyReadError::SourceUnavailable)
    );
}

#[tokio::test]
async fn schema_list_and_detail_use_one_pinned_vocabulary_without_fabricating_an_upcast() {
    let source = FixtureOntologyReadSource::sample();
    let request = query(2);
    let (types, page) = tokio::join!(source.types(TENANT), source.objects(TENANT, &request));
    assert_eq!(types.unwrap()[0].revision, 2);
    let page = page.unwrap();
    assert_eq!(page.requested_revision, 2);
    assert_eq!(page.objects.len(), 2);
    assert_eq!(page.next, None);
    for object in page.objects {
        let detail = source.object(TENANT, &object.object_ref, 2).await.unwrap();
        assert_eq!(detail.requested_revision, 2);
        assert_eq!(detail.object, object);
    }
    let old_write = source.object(TENANT, ALPHA, 2).await.unwrap();
    assert_eq!(old_write.object.written_revision, 1);
    assert_eq!(old_write.object.upcast_state, UpcastState::UpcastPending);
    assert!(!old_write.object.properties.contains_key("note"));
    let old_pin = source.object(TENANT, BETA, 1).await.unwrap();
    assert_eq!(old_pin.object.written_revision, 2);
    assert_eq!(old_pin.object.upcast_state, UpcastState::Current);
    assert!(!old_pin.object.properties.contains_key("note"));
    let name = &old_pin.object.properties["name"];
    assert_eq!(name.data_class, "PII_IDENTIFYING");
    assert_eq!(name.value_type, "string");
    assert_eq!(name.value, "Fixture Bob");
}

#[tokio::test]
async fn continuation_is_exclusive_and_exists_only_for_more_matching_rows() {
    let source = FixtureOntologyReadSource::sample();
    let mut request = query(1);
    request.limit = 1;
    let first = source.objects(TENANT, &request).await.unwrap();
    assert_eq!(first.objects[0].object_ref, ALPHA);
    assert_eq!(first.next.as_deref(), Some(ALPHA));
    request.cursor = first.next;
    let second = source.objects(TENANT, &request).await.unwrap();
    assert_eq!(second.objects[0].object_ref, BETA);
    assert_eq!(second.next, None);
    request.cursor = Some(BETA.to_owned());
    let end = source.objects(TENANT, &request).await.unwrap();
    assert!(end.objects.is_empty());
    assert_eq!(end.next, None);
    request.cursor = None;
    request.filter = Some(PropertyFilter::Equals {
        property: "active".to_owned(),
        value: FilterValue::Boolean(true),
    });
    let matching = source.objects(TENANT, &request).await.unwrap();
    assert_eq!(matching.objects.len(), 1);
    assert_eq!(matching.objects[0].object_ref, ALPHA);
    assert_eq!(matching.next, None);
}

#[tokio::test]
async fn native_equality_range_and_absence_have_different_answers() {
    let source = FixtureOntologyReadSource::sample();
    let mut request = query(1);
    request.filter = Some(PropertyFilter::Equals {
        property: "score".to_owned(),
        value: FilterValue::Text("7".to_owned()),
    });
    assert!(
        source
            .objects(TENANT, &request)
            .await
            .unwrap()
            .objects
            .is_empty()
    );
    request.filter = Some(PropertyFilter::Range {
        property: "score".to_owned(),
        from: FilterValue::Integer(7),
        to: FilterValue::Integer(11),
    });
    assert_eq!(
        source
            .objects(TENANT, &request)
            .await
            .unwrap()
            .objects
            .len(),
        2
    );
    request.filter = Some(PropertyFilter::Equals {
        property: "note".to_owned(),
        value: FilterValue::Text("Sample fixture only".to_owned()),
    });
    assert_eq!(
        source.objects(TENANT, &request).await,
        Err(OntologyReadError::InvalidQuery)
    );
    request.revision = 2;
    let filtered = source.objects(TENANT, &request).await.unwrap();
    assert_eq!(filtered.objects.len(), 1);
    assert_eq!(filtered.objects[0].object_ref, BETA);
    request.filter = Some(PropertyFilter::Range {
        property: "name".to_owned(),
        from: FilterValue::Integer(1),
        to: FilterValue::Integer(2),
    });
    request.cursor = Some("ent_zzzz".to_owned());
    request.limit = 1;
    assert_eq!(
        source.objects(TENANT, &request).await,
        Err(OntologyReadError::FilterConflict)
    );
}

#[tokio::test]
async fn absence_unretained_pins_and_invalid_queries_do_not_become_empty_success() {
    let source = FixtureOntologyReadSource::sample();
    assert_eq!(
        source.object(TENANT, "ent_missing", 1).await,
        Err(OntologyReadError::AbsentObject)
    );
    assert_eq!(
        source.object(TENANT, ALPHA, 99).await,
        Err(OntologyReadError::RevisionConflict)
    );
    assert_eq!(
        source.objects(TENANT, &query(99)).await,
        Err(OntologyReadError::RevisionConflict)
    );
    let mut request = query(1);
    request.entity_type = "ety_missing".to_owned();
    assert_eq!(
        source.objects(TENANT, &request).await,
        Err(OntologyReadError::InvalidQuery)
    );
    for limit in [0, 1001] {
        request = query(1);
        request.limit = limit;
        assert_eq!(
            source.objects(TENANT, &request).await,
            Err(OntologyReadError::InvalidQuery)
        );
    }
}

#[tokio::test]
async fn history_and_audit_preserve_log_evidence_and_poisoned_entries() {
    let source = FixtureOntologyReadSource::sample();
    let history = source.history(TENANT, ALPHA).await.unwrap();
    assert_eq!(
        history.iter().map(|row| row.ordinal).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(history[1].object_sequence, 2);
    assert_eq!(history[1].principal_id, "prn_fixture_user");
    assert_eq!(history[1].decision_id, "fixture_decision_2");
    assert_eq!(history[1].schema_revision, 1);
    assert!(
        source
            .history(TENANT, "ent_missing")
            .await
            .unwrap()
            .is_empty()
    );
    let audit = source.audit(TENANT).await.unwrap();
    assert_eq!(audit.len(), 4);
    assert_eq!(audit[3].disposition, AuditDisposition::Poisoned);
    assert_eq!(
        audit[3].poison_reason.as_deref(),
        Some("fixture rejected entry")
    );
    assert_eq!(audit[3].principal_id, None);
    assert_eq!(audit[3].decision_id, None);
    assert_eq!(source.audit(TENANT).await.unwrap(), audit);
}

#[tokio::test]
async fn explicit_refusals_reach_every_read_without_sample_fallback() {
    for error in [
        OntologyReadError::Unauthenticated,
        OntologyReadError::Forbidden,
        OntologyReadError::InvalidQuery,
        OntologyReadError::AbsentObject,
        OntologyReadError::RevisionConflict,
        OntologyReadError::FilterConflict,
        OntologyReadError::SourceUnavailable,
    ] {
        let source = FixtureOntologyReadSource::refusing(error);
        assert_eq!(source.schemas(TENANT).await, Err(error));
        assert_eq!(source.types(TENANT).await, Err(error));
        assert_eq!(source.objects(TENANT, &query(1)).await, Err(error));
        assert_eq!(source.object(TENANT, ALPHA, 1).await, Err(error));
        assert_eq!(source.history(TENANT, ALPHA).await, Err(error));
        assert_eq!(source.audit(TENANT).await, Err(error));
    }
}

#[tokio::test]
async fn fixture_scope_refusal_precedes_queries_and_object_existence() {
    let source = FixtureOntologyReadSource::sample();
    let mut invalid = query(99);
    invalid.limit = 0;
    assert_eq!(
        source.types("ten_other").await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        source.objects("ten_other", &invalid).await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        source.object("ten_other", ALPHA, 1).await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        source.object("ten_other", "ent_missing", 99).await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        source.history("ten_other", ALPHA).await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        source.audit("ten_other").await,
        Err(OntologyReadError::Forbidden)
    );
    assert_eq!(
        FixtureOntologyReadSource::refusing(OntologyReadError::Unauthenticated)
            .objects(TENANT, &invalid)
            .await,
        Err(OntologyReadError::Unauthenticated)
    );
}
