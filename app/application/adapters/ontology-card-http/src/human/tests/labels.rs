use application_ontology_read::{OntologyReadError, OntologyReadSource};
use serde_json::json;

use super::support::{cookie, source, stub};

#[tokio::test]
async fn native_labels_preserve_text_and_use_only_the_fresh_session_cookie() {
    let wire = json!([{"entity_type":"ety_기록","revision":0,"display_name":"기록 <script>"}]);
    let (base, seen) = stub(vec![(200, wire.to_string(), String::new())]);
    let labels = source(&base, &cookie('A'))
        .type_labels("ten_acme")
        .await
        .unwrap();
    assert_eq!(serde_json::to_value(labels).unwrap(), wire);
    let request = seen.recv().unwrap();
    assert!(request.starts_with("GET /v1/tenants/ten_acme/type-labels HTTP/1.1\r\n"));
    assert!(request.contains(&format!("cookie: {}\r\n", cookie('A'))));
    assert!(!request.to_ascii_lowercase().contains("authorization:"));
}

#[tokio::test]
async fn malformed_or_ambiguous_labels_refuse_without_a_name_fallback() {
    for wire in [
        r#"[["ety_record",1,"Record"]]"#,
        r#"[{"entity_type":"ety_record","revision":1}]"#,
        r#"[{"entity_type":"ety_record","revision":1,"display_name":"Record","extra":true}]"#,
        r#"[{"entity_type":"ety_record","revision":1,"display_name":"Record","display_name":"Other"}]"#,
        r#"[{"entity_type":"ety_record","revision":1,"display_name":"Record"},{"entity_type":"ety_record","revision":1,"display_name":"Record"}]"#,
        r#"[{"entity_type":"ety_","revision":1,"display_name":"Record"}]"#,
        r#"[{"entity_type":"ety_record","revision":1,"display_name":"\u2003"}]"#,
    ] {
        let (base, _) = stub(vec![(200, wire.into(), String::new())]);
        assert_eq!(
            source(&base, &cookie('A')).type_labels("ten_acme").await,
            Err(OntologyReadError::SourceUnavailable),
            "{wire}"
        );
    }
    for (status, expected) in [
        (401, OntologyReadError::Unauthenticated),
        (403, OntologyReadError::Forbidden),
        (503, OntologyReadError::SourceUnavailable),
    ] {
        let (base, _) = stub(vec![(status, "{}".into(), String::new())]);
        assert_eq!(
            source(&base, &cookie('A')).type_labels("ten_acme").await,
            Err(expected)
        );
    }
}
