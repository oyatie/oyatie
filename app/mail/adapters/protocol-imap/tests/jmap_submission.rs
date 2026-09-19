use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::{MetadataStore, SubmissionQueue};
use mail_kernel::{Account, Command};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn fixture() -> (Arc<SqliteStore>, axum::Router) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "tenant", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "other", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    db.execute("a", mail_api::Precondition::Observed(0), vec![Command::Append {
        mailboxes: vec!["inbox".into()],
        raw: b"From: alice@example.org\r\nTo: recipient@remote.org\r\nBcc: blind@remote.org\r\nSubject: draft\r\n\r\nbody\r\n".to_vec(),
        keywords: vec!["$draft".into()], received_at: 1,
    }]).unwrap();
    let service = Arc::new(MailService {
        outbound: Some(db.clone()),
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    (
        db,
        mail_protocol_imap::jmap_router(service, "https://mail.example.org".into()),
    )
}

async fn call(app: &axum::Router, method: &str, args: Value) -> Value {
    let body = json!({"using":["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:submission"],"methodCalls":[[method,args,"x"]]});
    let response = app
        .clone()
        .oneshot(
            Request::post("/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

#[tokio::test]
async fn submission_atomically_queues_blind_recipients_and_retains_history_after_delivery() {
    let (db, app) = fixture();
    let response = call(
        &app,
        "EmailSubmission/set",
        json!({"accountId":"a","create":{"send":{"identityId":"a","emailId":"e1"}}}),
    )
    .await;
    let created = &response["methodResponses"][0][1]["created"]["send"];
    let id = created["id"].as_str().expect("submission is created");
    assert!(created["sendAt"].is_string());
    let state = response["methodResponses"][0][1]["newState"].clone();
    let leases = db.claim_outbound(100).unwrap();
    assert_eq!(leases.len(), 2);
    for lease in leases {
        let message = db.outbound_message(&lease).unwrap();
        assert!(!String::from_utf8(message.raw).unwrap().contains("Bcc:"));
        db.finish_outbound(&lease, mail_api::DeliveryOutcome::Delivered)
            .unwrap();
    }
    assert!(
        String::from_utf8(db.blob("a", "e1").unwrap())
            .unwrap()
            .contains("Bcc:")
    );
    let response = call(
        &app,
        "EmailSubmission/get",
        json!({"accountId":"a","ids":[id]}),
    )
    .await;
    let record = &response["methodResponses"][0][1]["list"][0];
    assert_eq!(record["emailId"], "e1");
    assert_eq!(record["undoStatus"], "final");
    let response = call(
        &app,
        "EmailSubmission/changes",
        json!({"accountId":"a","sinceState":state}),
    )
    .await;
    assert_eq!(response["methodResponses"][0][1]["updated"], json!([id]));
    let response = call(
        &app,
        "EmailSubmission/get",
        json!({"accountId":"b","ids":[id]}),
    )
    .await;
    assert_eq!(response["methodResponses"][0][1]["type"], "accountNotFound");
}

#[tokio::test]
async fn scheduled_submission_can_be_cancelled_without_deleting_the_original_email() {
    let (db, app) = fixture();
    let response = call(&app, "EmailSubmission/set", json!({"accountId":"a","create":{"send":{"identityId":"a","emailId":"e1","envelope":{"mailFrom":{"email":"alice@example.org","parameters":{"HOLDUNTIL":"2079-11-20T05:00:00Z"}},"rcptTo":[{"email":"recipient@remote.org"}]}}}})).await;
    let id = response["methodResponses"][0][1]["created"]["send"]["id"]
        .as_str()
        .expect("scheduled submission");
    assert!(db.claim_outbound(100).unwrap().is_empty());
    let response = call(
        &app,
        "EmailSubmission/set",
        json!({"accountId":"a","update":{id:{"undoStatus":"canceled"}}}),
    )
    .await;
    assert_eq!(
        response["methodResponses"][0][1]["updated"][id],
        Value::Null
    );
    assert!(
        response["methodResponses"][0][1]["updated"]
            .as_object()
            .unwrap()
            .contains_key(id)
    );
    assert!(db.claim_outbound(100).unwrap().is_empty());
    assert!(db.blob("a", "e1").is_ok());
}
