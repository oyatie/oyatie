use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::MetadataStore;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn router(db: &Arc<SqliteStore>) -> Router {
    mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "http://localhost".into(),
    )
}

async fn call(app: &Router, name: &str, args: Value) -> Value {
    let using = [
        "urn:ietf:params:jmap:core",
        "urn:ietf:params:jmap:mail",
        "urn:ietf:params:jmap:vacationresponse",
    ];
    let response = app
        .clone()
        .oneshot(
            Request::post("/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"using":using,"methodCalls":[[name,args,"x"]]}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    serde_json::from_slice::<Value>(&response.into_body().collect().await.unwrap().to_bytes())
        .unwrap()["methodResponses"][0]
        .clone()
}

fn setup() -> Arc<SqliteStore> {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "other", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    db
}

#[tokio::test]
async fn vacation_is_a_persistent_singleton_and_refuses_create_or_destroy() {
    let db = setup();
    let app = router(&db);
    let get = call(&app, "VacationResponse/get", json!({"accountId":"a"})).await;
    assert_eq!(get[0], "VacationResponse/get", "{get}");
    assert_eq!(get[1]["list"][0]["id"], "singleton");
    assert_eq!(get[1]["list"][0]["isEnabled"], false);
    let missing = call(
        &app,
        "VacationResponse/get",
        json!({"accountId":"a","ids":["not-singleton"]}),
    )
    .await;
    assert_eq!(missing[1]["list"].as_array().unwrap().len(), 0);
    assert_eq!(missing[1]["notFound"], json!(["not-singleton"]));
    let set = call(
        &app,
        "VacationResponse/set",
        json!({"accountId":"a","ifInState":get[1]["state"],"update":{"singleton":{
            "isEnabled":true,
            "subject":"Out of Office - Test",
            "textBody":"I am currently out of the office for testing.",
            "fromDate":"2026-03-01T00:00:00Z",
            "toDate":"2026-03-15T00:00:00Z",
            "htmlBody":"<p>I am out of office.</p>",
        }}}),
    )
    .await;
    assert_eq!(set[0], "VacationResponse/set", "{set}");
    assert!(
        set[1]["updated"]
            .as_object()
            .unwrap()
            .contains_key("singleton")
    );
    let reloaded = router(&db);
    let current = call(
        &reloaded,
        "VacationResponse/get",
        json!({"accountId":"a","ids":["singleton"]}),
    )
    .await;
    assert_eq!(current[1]["list"][0]["isEnabled"], true);
    assert_eq!(current[1]["list"][0]["subject"], "Out of Office - Test");
    assert_eq!(current[1]["list"][0]["fromDate"], "2026-03-01T00:00:00Z");
    assert!(
        current[1]["list"][0]["htmlBody"]
            .as_str()
            .unwrap()
            .contains("out of office")
    );
    let created = call(
        &app,
        "VacationResponse/set",
        json!({"accountId":"a","create":{"i0":{"isEnabled":false}}}),
    )
    .await;
    assert_eq!(created[1]["notCreated"]["i0"]["type"], "singleton");
    let destroyed = call(
        &app,
        "VacationResponse/set",
        json!({"accountId":"a","destroy":["singleton"]}),
    )
    .await;
    assert_eq!(
        destroyed[1]["notDestroyed"]["singleton"]["type"],
        "singleton"
    );
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: unrelated\r\n\r\nbody",
    )
    .unwrap();
    let after = call(&reloaded, "VacationResponse/get", json!({"accountId":"a"})).await;
    assert_eq!(after[1]["state"], current[1]["state"]);
}

#[tokio::test]
async fn vacation_updates_are_isolated_by_account_and_reject_invalid_values() {
    let db = setup();
    let app = router(&db);
    call(
        &app,
        "VacationResponse/set",
        json!({"accountId":"a","update":{"singleton":{"isEnabled":true,"subject":"Alice"}}}),
    )
    .await;
    let other = call(&app, "VacationResponse/get", json!({"accountId":"b"})).await;
    assert_eq!(other[0], "error");
    let bob = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "http://localhost".into(),
    );
    let request = Request::post("/jmap")
        .header("authorization", format!("Bearer {}", "b".repeat(32)))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:vacationresponse"],"methodCalls":[["VacationResponse/get",{"accountId":"b"},"x"]]})
                .to_string(),
        ))
        .unwrap();
    let response = bob.clone().oneshot(request).await.unwrap();
    let value: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(
        value["methodResponses"][0][1]["list"][0]["isEnabled"],
        false
    );
    let invalid = call(
        &app,
        "VacationResponse/set",
        json!({"accountId":"a","update":{"singleton":{"fromDate":"tomorrow"}}}),
    )
    .await;
    assert_eq!(
        invalid[1]["notUpdated"]["singleton"]["type"],
        "invalidProperties"
    );
    let stale = call(
        &app,
        "VacationResponse/set",
        json!({"accountId":"a","ifInState":"0","update":{"singleton":{"isEnabled":false}}}),
    )
    .await;
    assert_eq!(stale[0], "error");
    assert_eq!(stale[1]["type"], "stateMismatch");
}
