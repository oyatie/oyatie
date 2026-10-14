use super::{
    super::HumanOntologyCardSource,
    support::{cookie, raw_stub, stub},
};
use application_ontology_read::*;
use std::time::Duration;
fn document() -> ModelDocument {
    ModelDocument {
        entities: vec![],
        links: vec![],
    }
}
fn draft(id: &str, version: &str) -> ModelDraftWire {
    ModelDraftWire::decode(serde_json::json!({"id":id,"tenant_id":"ten_acme","owner":"usr_alice","base_generation":"9007199254740993","version":version,"document":document(),"publication":null}).to_string().as_bytes()).unwrap()
}
#[tokio::test]
async fn six_model_routes_keep_native_decimal_documents_cookie_origin_and_exact_acknowledgements() {
    let number = ModelNumber::parse("9007199254740993").unwrap();
    let first = draft("9007199254740993", "1");
    let saved = draft("9007199254740993", "2");
    let workspace = ModelWorkspace {
        generation: number,
        document: document(),
        capabilities: ModelCapabilities {
            author: true,
            publish: false,
        },
    };
    let page = ModelDraftPage {
        drafts: vec![ModelDraftSummary {
            id: number,
            version: first.version,
            base_generation: number,
            published_generation: None,
        }],
        next: None,
    };
    let validation = ModelValidation {
        draft_id: number,
        draft_version: saved.version,
        base_generation: number,
        valid: true,
        diagnostics: vec![],
    };
    let replies = vec![
        (200, workspace.encode().unwrap()),
        (200, page.encode().unwrap()),
        (200, first.encode().unwrap()),
        (201, first.encode().unwrap()),
        (200, saved.encode().unwrap()),
        (200, validation.encode().unwrap()),
    ];
    let (base, seen) = stub(
        replies
            .into_iter()
            .map(|(s, b)| (s, String::from_utf8(b).unwrap(), String::new()))
            .collect(),
    );
    let transport = HumanOntologyCardSource::for_loopback(&base).unwrap();
    let source = transport
        .model_source(&cookie('A'), "http://localhost:3000")
        .unwrap();
    assert_eq!(source.workspace("ten_acme").await.unwrap(), workspace);
    assert_eq!(
        source.owned_drafts("ten_acme", None, 25).await.unwrap(),
        page
    );
    assert_eq!(source.draft("ten_acme", number).await.unwrap(), first);
    assert_eq!(
        source
            .create_draft(
                "ten_acme",
                &CreateModelDraft {
                    expected_generation: number,
                    document: document()
                }
            )
            .await
            .unwrap(),
        first
    );
    assert_eq!(
        source
            .save_draft(
                "ten_acme",
                number,
                &SaveModelDraft {
                    expected_version: first.version,
                    expected_base: number,
                    document: document()
                }
            )
            .await
            .unwrap(),
        saved
    );
    assert_eq!(
        source
            .validate_draft(
                "ten_acme",
                number,
                &PrepareModelDraft {
                    expected_version: saved.version,
                    expected_base: number
                }
            )
            .await
            .unwrap(),
        validation
    );
    for (method, path) in [
        ("GET", "model"),
        ("GET", "model/drafts?limit=25"),
        ("GET", "model/drafts/9007199254740993"),
        ("POST", "model/drafts"),
        ("POST", "model/drafts/9007199254740993/save"),
        ("POST", "model/drafts/9007199254740993/validate"),
    ] {
        let request = seen.recv().unwrap();
        assert!(request.starts_with(&format!(
            "{method} /v1/tenants/ten_acme/{path} HTTP/1.1\r\n"
        )));
        assert!(request.contains(&format!("cookie: {}\r\n", cookie('A'))));
        assert!(!request.contains("authorization:"));
        if method == "POST" {
            assert!(request.contains("origin: http://localhost:3000\r\n"));
            assert!(request.contains("content-type: application/json\r\n"));
        }
    }
    assert!(
        transport
            .model_source(&cookie('A'), "http://evil.invalid")
            .is_err()
    );
    assert!(
        transport
            .model_source("bad-cookie", "http://localhost:3000")
            .is_err()
    );
}
#[tokio::test]
async fn malformed_or_wrong_mutation_ack_is_unknown_and_closed_refusal_keeps_its_native_kind() {
    let number = ModelNumber::new(1).unwrap();
    let create = CreateModelDraft {
        expected_generation: number,
        document: document(),
    };
    for (status, body, expected) in [
        (201, "{}", ModelSourceError::OutcomeUnknown),
        (
            201,
            r#"{"id":9007199254740993}"#,
            ModelSourceError::OutcomeUnknown,
        ),
        (
            403,
            r#"{"error":"forbidden"}"#,
            ModelSourceError::Refused(ModelRefusalCode::Forbidden),
        ),
        (
            409,
            r#"{"error":"draft_conflict"}"#,
            ModelSourceError::Refused(ModelRefusalCode::DraftConflict),
        ),
        (
            503,
            r#"{"error":"model_unavailable"}"#,
            ModelSourceError::OutcomeUnknown,
        ),
        (302, "{}", ModelSourceError::OutcomeUnknown),
    ] {
        let (base, _) = stub(vec![(status, body.into(), String::new())]);
        let source = HumanOntologyCardSource::for_loopback(&base)
            .unwrap()
            .model_source(&cookie('A'), "http://localhost:3000")
            .unwrap();
        assert_eq!(
            source.create_draft("ten_acme", &create).await,
            Err(expected)
        );
    }
}
#[tokio::test]
async fn workspace_http_refuses_declared_and_chunked_overflow_before_decode() {
    let maximum = model_wire_limits().unwrap().workspace;
    for chunked in [false, true] {
        let bytes = if chunked {
            let body = " ".repeat(maximum + 1);
            format!("HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n",body.len()).into_bytes()
        } else {
            format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                maximum + 1
            )
            .into_bytes()
        };
        let base = raw_stub(bytes, Duration::ZERO);
        let source = HumanOntologyCardSource::for_loopback(&base)
            .unwrap()
            .model_source(&cookie('A'), "http://localhost:3000")
            .unwrap();
        assert_eq!(
            source.workspace("ten_acme").await,
            Err(ModelSourceError::Refused(
                ModelRefusalCode::ModelUnavailable
            ))
        );
    }
}
