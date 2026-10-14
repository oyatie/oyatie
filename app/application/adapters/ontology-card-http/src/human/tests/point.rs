use application_ontology_read::{ObjectPointQuery, OntologyReadError, OntologyReadSource};

use super::support::{OBJECT, cookie, source, stub};

#[tokio::test]
async fn captured_point_type_ref_and_pin_are_form_encoded_once_with_current_cookie() {
    for entity_type in [
        "ety_record",
        "ety_+%&=#/?",
        "ety_가",
        "ety_a b",
        "ety_\u{feff}",
        &format!("ety_{}", "가".repeat(84)),
    ] {
        let (base, seen) = stub(vec![(200, OBJECT.into(), String::new())]);
        let query = ObjectPointQuery {
            entity_type: entity_type.into(),
            object_ref: "ent_+%가\u{feff}".into(),
            revision: u32::MAX,
        };
        source(&base, &cookie('A'))
            .object("ten_acme", &query)
            .await
            .unwrap();
        let mut expected = reqwest::Url::parse(&format!("{base}/v1/tenants/ten_acme")).unwrap();
        expected
            .path_segments_mut()
            .unwrap()
            .extend(["objects", query.object_ref.as_str()]);
        expected
            .query_pairs_mut()
            .append_pair("type", entity_type)
            .append_pair("revision", "4294967295");
        let request = seen.recv().unwrap();
        assert!(
            request.starts_with(&format!(
                "GET {}?{} HTTP/1.1\r\n",
                expected.path(),
                expected.query().unwrap()
            )),
            "{request}"
        );
        assert!(request.contains(&format!("cookie: {}\r\n", cookie('A'))));
        assert!(!request.to_ascii_lowercase().contains("authorization:"));
    }
}

#[tokio::test]
async fn invalid_point_context_is_refused_before_transport() {
    let source = source("http://127.0.0.1:1", &cookie('A'));
    for (entity_type, object_ref) in [
        ("", "ent_a"),
        ("ety_", "ent_a"),
        ("ety_a\n", "ent_a"),
        ("ety_a", "ent_"),
        ("ety_a", "ent_a\u{85}"),
    ] {
        let query = ObjectPointQuery {
            entity_type: entity_type.into(),
            object_ref: object_ref.into(),
            revision: 1,
        };
        assert_eq!(
            source.object("ten_acme", &query).await,
            Err(OntologyReadError::InvalidQuery)
        );
    }
}
