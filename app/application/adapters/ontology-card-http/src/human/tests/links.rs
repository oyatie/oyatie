use application_ontology_read::{OntologyReadError, OntologyReadSource};
use serde_json::json;

use super::support::{cookie, source, stub};

fn wire() -> serde_json::Value {
    json!([{"link_type":"lty_related","from_entity_type":"ety_record",
        "to_entity_type":"ety_record","cardinality":"one_to_many",
        "allow_cross_tenant":true,"revision":0}])
}

#[tokio::test]
async fn link_catalog_preserves_native_metadata_and_uses_only_the_fresh_human_cookie() {
    let expected = wire();
    let (base, seen) = stub(vec![(200, expected.to_string(), String::new())]);
    let links = source(&base, &cookie('A'))
        .link_types("ten_acme")
        .await
        .unwrap();
    assert_eq!(serde_json::to_value(links).unwrap(), expected);
    let request = seen.recv().unwrap();
    assert!(request.starts_with("GET /v1/tenants/ten_acme/link-types HTTP/1.1\r\n"));
    assert!(request.contains(&format!("cookie: {}\r\n", cookie('A'))));
    assert!(!request.to_ascii_lowercase().contains("authorization:"));
}

#[tokio::test]
async fn malformed_or_ambiguous_link_declarations_refuse_the_entire_catalog() {
    let mut malformed = Vec::new();
    for field in [
        "link_type",
        "from_entity_type",
        "to_entity_type",
        "cardinality",
        "allow_cross_tenant",
        "revision",
    ] {
        let mut missing = wire();
        missing[0].as_object_mut().unwrap().remove(field);
        malformed.push(missing);
    }
    for (field, value) in [
        ("link_type", json!("lty_")),
        ("from_entity_type", json!("ent_wrong")),
        ("to_entity_type", json!("ety_")),
        ("cardinality", json!("OneToMany")),
        ("cardinality", json!("many_to_one")),
        ("cardinality", json!({"one_to_many":null})),
        ("allow_cross_tenant", json!("true")),
        ("revision", json!(-1)),
        ("extra", json!(true)),
    ] {
        let mut invalid = wire();
        invalid[0][field] = value;
        malformed.push(invalid);
    }
    let mut duplicate = wire();
    let mut row = duplicate[0].clone();
    row["revision"] = json!(7);
    duplicate.as_array_mut().unwrap().push(row);
    malformed.push(duplicate);
    for invalid in malformed {
        let (base, _) = stub(vec![(200, invalid.to_string(), String::new())]);
        assert_eq!(
            source(&base, &cookie('A')).link_types("ten_acme").await,
            Err(OntologyReadError::SourceUnavailable)
        );
    }
}

#[tokio::test]
async fn empty_catalog_is_valid_and_each_link_read_preserves_current_authorization_refusals() {
    let (base, seen) = stub(vec![
        (200, "[]".into(), String::new()),
        (401, "{}".into(), String::new()),
        (403, "{}".into(), String::new()),
        (503, "backend detail".into(), String::new()),
    ]);
    let source = source(&base, &cookie('A'));
    assert_eq!(source.link_types("ten_acme").await, Ok(vec![]));
    for error in [
        OntologyReadError::Unauthenticated,
        OntologyReadError::Forbidden,
        OntologyReadError::SourceUnavailable,
    ] {
        assert_eq!(source.link_types("ten_acme").await, Err(error));
    }
    for _ in 0..4 {
        assert!(
            seen.recv()
                .unwrap()
                .contains(&format!("cookie: {}\r\n", cookie('A')))
        );
    }
}
