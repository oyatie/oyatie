mod failures;
mod labels;
mod links;
mod query;
mod schemas;
mod support;

use super::HumanOntologyCardSource;
use application_ontology_read::{
    AuditDisposition, ObjectListQuery, OntologyReadSource, UpcastState,
};
use support::{OBJECT, cookie, source, stub};

#[tokio::test]
async fn all_five_native_routes_preserve_wire_fields_and_separate_the_requested_pin() {
    let bodies = [
        r#"[{"entity_type":"ety_record","revision":2,"properties":["name"]}]"#.to_owned(),
        format!(r#"{{"objects":[{OBJECT}],"next":"ent_alpha"}}"#),
        OBJECT.to_owned(),
        r#"[{"ordinal":3,"object_sequence":2,"principal_id":"usr_alice","decision_id":"dec_1","action_type":"act_update","audit_event_type":"updated","schema_revision":1}]"#.to_owned(),
        r#"[{"ordinal":4,"object_ref":"ent_alpha","action_type":"act_update","disposition":"poisoned","poison_reason":"invalid","principal_id":null,"decision_id":null}]"#.to_owned(),
    ];
    let (base, seen) = stub(
        bodies
            .into_iter()
            .map(|body| (200, body, String::new()))
            .collect(),
    );
    let source = source(&base, &cookie('A'));
    let types = source.types("ten_acme").await.unwrap();
    assert_eq!(
        (types[0].revision, types[0].properties[0].as_str()),
        (2, "name")
    );
    let mut query = ObjectListQuery::first("ety_record", 2);
    query.limit = 1;
    let page = source.objects("ten_acme", &query).await.unwrap();
    assert_eq!(page.requested_revision, 2);
    assert_eq!(page.next.as_deref(), Some("ent_alpha"));
    let detail = source.object("ten_acme", "ent_alpha", 2).await.unwrap();
    assert_eq!(detail.requested_revision, 2);
    assert_eq!(detail.object, page.objects[0]);
    assert_eq!(detail.object.written_revision, 1);
    assert_eq!(detail.object.upcast_state, UpcastState::UpcastPending);
    assert_eq!(detail.object.properties["name"].data_class, "PII");
    assert_eq!(detail.object.properties["name"].value_type, "string");
    assert_eq!(detail.object.properties["name"].value, "Ada");
    let history = source.history("ten_acme", "ent_alpha").await.unwrap();
    assert_eq!((history[0].ordinal, history[0].object_sequence), (3, 2));
    assert_eq!(history[0].schema_revision, 1);
    assert_eq!(history[0].principal_id, "usr_alice");
    assert_eq!(history[0].decision_id, "dec_1");
    let audit = source.audit("ten_acme").await.unwrap();
    assert_eq!(audit[0].disposition, AuditDisposition::Poisoned);
    assert_eq!(audit[0].poison_reason.as_deref(), Some("invalid"));
    assert_eq!(audit[0].principal_id, None);
    assert_eq!(audit[0].decision_id, None);
    for path in [
        "/types",
        "/objects?type=ety_record&revision=2&limit=1",
        "/objects/ent_alpha?revision=2",
        "/objects/ent_alpha/history",
        "/audit",
    ] {
        let request = seen.recv().unwrap();
        assert!(
            request.starts_with(&format!("GET /v1/tenants/ten_acme{path} HTTP/1.1\r\n")),
            "{request}"
        );
        assert!(request.contains(&format!("cookie: {}\r\n", cookie('A'))));
        assert!(!request.to_ascii_lowercase().contains("authorization:"));
    }
}

#[tokio::test]
async fn fresh_request_cookies_replace_each_other_and_no_success_caches_authorization() {
    let (base, seen) = stub(vec![
        (200, "[]".into(), format!("set-cookie: {}\r\n", cookie('C'))),
        (200, "[]".into(), String::new()),
        (401, "{}".into(), String::new()),
    ]);
    let transport = HumanOntologyCardSource::for_loopback(&base).unwrap();
    let first = transport.read_source(&cookie('A')).unwrap();
    assert!(first.types("ten_acme").await.is_ok());
    let second = transport.read_source(&cookie('B')).unwrap();
    assert!(second.audit("ten_acme").await.is_ok());
    assert_eq!(
        first.history("ten_acme", "ent_alpha").await,
        Err(application_ontology_read::OntologyReadError::Unauthenticated)
    );
    for expected in [cookie('A'), cookie('B'), cookie('A')] {
        let request = seen.recv().unwrap();
        assert!(request.contains(&format!("cookie: {expected}\r\n")));
        assert!(!request.contains(&cookie('C')));
        assert!(!request.contains("authorization:"));
    }
}

#[tokio::test]
async fn shared_transport_preserves_the_existing_human_status_card() {
    let full = r#"{"policy_version":"cedar-test","served_tenants":1,"projection_lag":2,"poisoned_entries":3,"extra":"ignored"}"#;
    let (base, seen) = stub(vec![
        (200, full.into(), String::new()),
        (403, "{}".into(), String::new()),
    ]);
    let transport = HumanOntologyCardSource::for_loopback(&base).unwrap();
    let facts = transport.facts("ten_acme", &cookie('A')).await.unwrap();
    assert_eq!(facts.policy_version, "cedar-test");
    assert_eq!(
        (
            facts.served_tenants,
            facts.projection_lag,
            facts.poisoned_entries
        ),
        (1, 2, 3)
    );
    assert_eq!(
        transport.facts("ten_acme", &cookie('B')).await,
        Err(super::HumanOntologyRefusal::Forbidden)
    );
    for expected in [cookie('A'), cookie('B')] {
        let request = seen.recv().unwrap();
        assert!(request.starts_with("GET /v1/tenants/ten_acme/status HTTP/1.1\r\n"));
        assert!(request.contains(&format!("cookie: {expected}\r\n")));
    }
}
