use super::support::{cookie, source, stub};
use application_ontology_read::{
    FilterValue, ObjectListQuery, OntologyReadError, OntologyReadSource, PropertyFilter,
};

#[tokio::test]
async fn query_values_reach_the_native_raw_parser_without_form_encoding() {
    let (base, seen) = stub(vec![
        (
            200,
            r#"{"objects":[],"next":null}"#.into(),
            String::new()
        );
        4
    ]);
    let source = source(&base, &cookie('A'));
    let mut query = ObjectListQuery::first("ety_record", 2);
    query.cursor = Some("ent_alpha".into());
    for (filter, suffix) in [
        (
            PropertyFilter::Equals {
                property: "name".into(),
                value: FilterValue::Text("Ada+%20=:?".into()),
            },
            "&property=name&equals=text:Ada+%20=:?",
        ),
        (
            PropertyFilter::Equals {
                property: "name".into(),
                value: FilterValue::Text(String::new()),
            },
            "&property=name&equals=text:",
        ),
        (
            PropertyFilter::Range {
                property: "count".into(),
                from: FilterValue::Integer(-4),
                to: FilterValue::Integer(7),
            },
            "&property=count&from=int:-4&to=int:7",
        ),
        (
            PropertyFilter::Equals {
                property: "active".into(),
                value: FilterValue::Boolean(false),
            },
            "&property=active&equals=bool:false",
        ),
    ] {
        query.filter = Some(filter);
        source.objects("ten_acme", &query).await.unwrap();
        assert!(seen.recv().unwrap().starts_with(&format!("GET /v1/tenants/ten_acme/objects?type=ety_record&revision=2&limit=100&cursor=ent_alpha{suffix} HTTP/1.1\r\n")));
    }
}

#[tokio::test]
async fn unrepresentable_query_values_and_invalid_shapes_refuse_before_network_io() {
    let source = source("http://127.0.0.1:1", &cookie('A'));
    for raw in [
        "Ada Lovelace",
        "Ada&revision=9",
        "#fragment",
        "\n",
        "é",
        "\t",
        r#""quoted""#,
        "<tag>",
        "'single'",
    ] {
        let mut query = ObjectListQuery::first("ety_record", 1);
        query.filter = Some(PropertyFilter::Equals {
            property: "name".into(),
            value: FilterValue::Text(raw.into()),
        });
        assert_eq!(
            source.objects("ten_acme", &query).await,
            Err(OntologyReadError::InvalidQuery),
            "{raw:?}"
        );
    }
    let mut query = ObjectListQuery::first("ety_record&revision=9", 1);
    assert_eq!(
        source.objects("ten_acme", &query).await,
        Err(OntologyReadError::InvalidQuery)
    );
    query.entity_type = "ety_record".into();
    query.limit = 0;
    assert_eq!(
        source.objects("ten_acme", &query).await,
        Err(OntologyReadError::InvalidQuery)
    );
    query.limit = 1001;
    assert_eq!(
        source.objects("ten_acme", &query).await,
        Err(OntologyReadError::InvalidQuery)
    );
    query.limit = 1;
    query.cursor = Some("wrong".into());
    assert_eq!(
        source.objects("ten_acme", &query).await,
        Err(OntologyReadError::InvalidQuery)
    );
    assert_eq!(
        source.types("wrong").await,
        Err(OntologyReadError::InvalidQuery)
    );
    assert_eq!(
        source.object("ten_acme", "..", 1).await,
        Err(OntologyReadError::InvalidQuery)
    );
}

#[tokio::test]
async fn opaque_path_components_cannot_replace_the_origin_or_escape_the_tenant_route() {
    let (base, seen) = stub(vec![(200, "[]".into(), String::new())]);
    source(&base, &cookie('A'))
        .history("ten_acme/../ten_other", "ent_a/../audit")
        .await
        .unwrap();
    assert!(seen.recv().unwrap().starts_with(
        "GET /v1/tenants/ten_acme%2F..%2Ften_other/objects/ent_a%2F..%2Faudit/history HTTP/1.1\r\n"
    ));
}
