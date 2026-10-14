use application_ontology_read::{
    MAX_RELATIONSHIP_RESPONSE_BYTES, OntologyReadError, OntologyReadSource, RelationshipQuery,
};
use serde_json::{Value, json};

use super::support::{cookie, source, stub};

fn query() -> RelationshipQuery {
    RelationshipQuery::decode(br#"{"idempotency_key":"attempt-1","root":{"entity_type":"ety_record","object_ref":"ent_alpha","revision":1},"link_type":"lty_related","direction":"outbound","observed_at_epoch_seconds":123}"#).unwrap()
}
fn links() -> String {
    json!([{"link_type":"lty_related","from_entity_type":"ety_record","to_entity_type":"ety_profile","cardinality":"one_to_many","allow_cross_tenant":false,"revision":1}]).to_string()
}
fn graph() -> Value {
    json!({"nodes":[{"entity_id":"ent_alpha","entity_type_id":"ety_record","written_revision":1},{"entity_id":"ent_beta","entity_type_id":"ety_profile","written_revision":7}],"edges":[{"from_entity_id":"ent_alpha","to_entity_id":"ent_beta","edge_type_id":"lty_related"}],"observed_at_epoch_seconds":123,"truncated":false})
}
fn response(status: u16, body: impl ToString) -> (u16, String, String) {
    (status, body.to_string(), String::new())
}

#[tokio::test]
async fn exact_one_hop_wire_keeps_fresh_cookie_and_independent_target_pin() {
    let (base, seen) = stub(vec![response(200, links()), response(200, graph())]);
    let read = source(&base, &cookie('A'))
        .search_around("ten_acme", &query())
        .await
        .unwrap();
    assert_eq!(read.nodes[1].written_revision, 7);
    let catalog = seen.recv().unwrap();
    assert!(catalog.starts_with("GET /v1/tenants/ten_acme/link-types HTTP/1.1\r\n"));
    let request = seen.recv().unwrap();
    assert!(request.starts_with("POST /v1/tenants/ten_acme/search-around HTTP/1.1\r\n"));
    for wire in [&catalog, &request] {
        assert!(wire.contains(&format!("cookie: {}\r\n", cookie('A'))));
        assert!(!wire.to_ascii_lowercase().contains("authorization:"));
    }
    assert!(request.contains("content-type: application/json\r\n"));
    assert_eq!(
        serde_json::from_str::<Value>(request.split_once("\r\n\r\n").unwrap().1).unwrap(),
        serde_json::to_value(query()).unwrap()
    );
}

#[tokio::test]
async fn authorization_protocol_and_native_revision_errors_remain_distinct() {
    let cases = [
        (401, "{}", OntologyReadError::Unauthenticated),
        (403, "{}", OntologyReadError::Forbidden),
        (
            400,
            r#"{"gate":"surface","cause":"different intent"}"#,
            OntologyReadError::InvalidQuery,
        ),
        (
            409,
            r#"{"gate":"surface","cause":"that revision was never accepted for this entity type"}"#,
            OntologyReadError::RevisionConflict,
        ),
        (
            409,
            r#"{"gate":"surface","cause":"that idempotency key is already answering another query"}"#,
            OntologyReadError::SourceUnavailable,
        ),
        (404, "{}", OntologyReadError::SourceUnavailable),
        (413, "{}", OntologyReadError::InvalidQuery),
        (415, "{}", OntologyReadError::InvalidQuery),
        (503, "backend details", OntologyReadError::SourceUnavailable),
        (302, "{}", OntologyReadError::SourceUnavailable),
        (200, "{}", OntologyReadError::SourceUnavailable),
    ];
    for (status, body, expected) in cases {
        let (base, seen) = stub(vec![response(200, links()), response(status, body)]);
        assert_eq!(
            source(&base, &cookie('A'))
                .search_around("ten_acme", &query())
                .await,
            Err(expected)
        );
        assert!(seen.recv().unwrap().starts_with("GET "));
        assert!(seen.recv().unwrap().starts_with("POST "));
    }
}

#[tokio::test]
async fn malformed_or_unrelated_graphs_never_become_empty_success() {
    let mut invalid = vec![json!([]), json!({})];
    for (field, value) in [
        ("nodes", json!([])),
        ("edges", json!([])),
        ("extra", json!(true)),
        ("observed_at_epoch_seconds", json!(124)),
    ] {
        let mut wire = graph();
        wire[field] = value;
        invalid.push(wire);
    }
    let mut duplicate = graph();
    let node = duplicate["nodes"][0].clone();
    duplicate["nodes"].as_array_mut().unwrap().push(node);
    invalid.push(duplicate);
    let mut wrong = graph();
    wrong["nodes"][1]["entity_type_id"] = json!("ety_record");
    invalid.push(wrong);
    let mut orphan = graph();
    orphan["edges"][0]["to_entity_id"] = json!("ent_missing");
    invalid.push(orphan);
    for wire in invalid {
        let (base, _) = stub(vec![response(200, links()), response(200, wire)]);
        assert_eq!(
            source(&base, &cookie('A'))
                .search_around("ten_acme", &query())
                .await,
            Err(OntologyReadError::SourceUnavailable)
        );
    }
    let mut truncated = graph();
    truncated["truncated"] = json!(true);
    let (base, _) = stub(vec![response(200, links()), response(200, truncated)]);
    assert!(
        source(&base, &cookie('A'))
            .search_around("ten_acme", &query())
            .await
            .unwrap()
            .truncated
    );
}

#[tokio::test]
async fn selected_relationship_must_exist_and_match_the_roots_directed_role() {
    for catalog in [
        "[]".to_owned(),
        links().replace("ety_record", "ety_another"),
    ] {
        let (base, seen) = stub(vec![response(200, catalog)]);
        assert_eq!(
            source(&base, &cookie('A'))
                .search_around("ten_acme", &query())
                .await,
            Err(OntologyReadError::InvalidQuery)
        );
        assert!(seen.recv().unwrap().starts_with("GET "));
        assert!(seen.try_recv().is_err());
    }
    let (base, seen) = stub(vec![]);
    let mut invalid = query();
    invalid.idempotency_key = " bad".into();
    assert_eq!(
        source(&base, &cookie('A'))
            .search_around("ten_acme", &invalid)
            .await,
        Err(OntologyReadError::InvalidQuery)
    );
    assert!(seen.try_recv().is_err());
}

#[tokio::test]
async fn catalog_authorization_refusals_and_oversized_graphs_stay_fail_closed() {
    for (status, body, expected) in [
        (401, "{}", OntologyReadError::Unauthenticated),
        (403, "{}", OntologyReadError::Forbidden),
        (
            404,
            r#"{"gate":"surface","cause":"the durable projection holds no object at this reference"}"#,
            OntologyReadError::SourceUnavailable,
        ),
        (
            409,
            r#"{"gate":"surface","cause":"that revision was never accepted for this entity type"}"#,
            OntologyReadError::SourceUnavailable,
        ),
    ] {
        let (base, seen) = stub(vec![response(status, body)]);
        assert_eq!(
            source(&base, &cookie('A'))
                .search_around("ten_acme", &query())
                .await,
            Err(expected)
        );
        assert!(seen.recv().unwrap().starts_with("GET "));
        assert!(seen.try_recv().is_err());
    }
    let (base, _) = stub(vec![
        response(200, links()),
        response(200, " ".repeat(MAX_RELATIONSHIP_RESPONSE_BYTES + 1)),
    ]);
    assert_eq!(
        source(&base, &cookie('A'))
            .search_around("ten_acme", &query())
            .await,
        Err(OntologyReadError::SourceUnavailable)
    );
}
