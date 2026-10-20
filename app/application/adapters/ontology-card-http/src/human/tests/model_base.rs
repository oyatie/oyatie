use super::{
    super::HumanOntologyCardSource,
    support::{cookie, raw_stub, stub},
};
use application_ontology_read::*;
use std::time::Duration;

fn expected() -> PrepareModelDraft {
    PrepareModelDraft {
        expected_version: ModelNumber::parse("9007199254740993").unwrap(),
        expected_base: ModelNumber::parse("9223372036854775807").unwrap(),
    }
}
fn base() -> serde_json::Value {
    serde_json::json!({"draft_id":"9223372036854775807","draft_version":"9007199254740993","base_generation":"9223372036854775807","document":{"entities":[],"links":[]}})
}
#[tokio::test]
async fn draft_base_get_preserves_exact_tuple_cookie_and_closed_read_refusals() {
    let id = ModelNumber::parse("9223372036854775807").unwrap();
    let mut replies = vec![(200, base().to_string(), String::new())];
    for field in ["draft_id", "draft_version", "base_generation"] {
        let mut wrong = base();
        wrong[field] = "1".into();
        replies.push((200, wrong.to_string(), String::new()));
    }
    for wrong in [
        serde_json::json!({}),
        {
            let mut b = base();
            b["extra"] = true.into();
            b
        },
        {
            let mut b = base();
            b.as_object_mut().unwrap().remove("document");
            b
        },
        {
            let mut b = base();
            b["draft_version"] = 1.into();
            b
        },
    ] {
        replies.push((200, wrong.to_string(), String::new()));
    }
    replies.extend([
        (302, "{}".into(), String::new()),
        (503, r#"{"error":"forbidden"}"#.into(), String::new()),
        (
            503,
            r#"{"error":"model_unavailable"}"#.into(),
            String::new(),
        ),
    ]);
    let count = replies.len();
    let (url, seen) = stub(replies);
    let source = HumanOntologyCardSource::for_loopback(&url)
        .unwrap()
        .model_source(&cookie('A'), "http://localhost:3000")
        .unwrap();
    assert_eq!(
        serde_json::to_value(
            source
                .draft_base("ten_acme", id, &expected())
                .await
                .unwrap()
        )
        .unwrap(),
        base()
    );
    for _ in 1..count {
        assert_eq!(
            source.draft_base("ten_acme", id, &expected()).await,
            Err(ModelSourceError::Refused(
                ModelRefusalCode::ModelUnavailable
            ))
        );
    }
    for _ in 0..count {
        let request = seen.recv().unwrap();
        assert!(request.starts_with("GET /v1/tenants/ten_acme/model/drafts/9223372036854775807/base?expected_version=9007199254740993&expected_base=9223372036854775807 HTTP/1.1\r\n"));
        assert!(request.contains(&format!("cookie: {}\r\n", cookie('A'))));
        assert!(
            !request.contains("authorization:")
                && !request.contains("origin:")
                && !request.contains("content-type:")
        );
    }
}
#[tokio::test]
async fn draft_base_read_refuses_declared_and_chunked_overflow() {
    let maximum = model_wire_limits().unwrap().draft_base;
    for chunked in [false, true] {
        let bytes = if chunked {
            let body = " ".repeat(maximum + 1);
            format!("HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n", body.len()).into_bytes()
        } else {
            format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                maximum + 1
            )
            .into_bytes()
        };
        let url = raw_stub(bytes, Duration::ZERO);
        let source = HumanOntologyCardSource::for_loopback(&url)
            .unwrap()
            .model_source(&cookie('A'), "http://localhost:3000")
            .unwrap();
        assert_eq!(
            source
                .draft_base("ten_acme", ModelNumber::new(1).unwrap(), &expected())
                .await,
            Err(ModelSourceError::Refused(
                ModelRefusalCode::ModelUnavailable
            ))
        );
    }
}
