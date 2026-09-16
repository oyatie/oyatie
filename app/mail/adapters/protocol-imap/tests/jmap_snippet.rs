use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::Store;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
fn setup() -> (Arc<SqliteStore>, Router) {
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
    let app = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "http://localhost".into(),
    );
    (db, app)
}
async fn response(app: &Router, name: &str, args: Value) -> Value {
    let response = app.clone().oneshot(Request::post("/jmap").header("authorization", format!("Bearer {TOKEN}")).header("content-type", "application/json").body(Body::from(json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[[name,args,"x"]]}).to_string())).unwrap()).await.unwrap();
    serde_json::from_slice::<Value>(&response.into_body().collect().await.unwrap().to_bytes())
        .unwrap()["methodResponses"][0]
        .clone()
}
#[tokio::test]
async fn query_and_snippet_share_decoded_text_and_escape_html() {
    let (db, app) = setup();
    let body = format!(
        "{} <script> CAFÉ & xylophone <tag> {}",
        "padding ".repeat(200),
        "tail ".repeat(200)
    );
    db.deliver(
        &["alice@example.org".into()],
        format!(
            "Subject: Financial <Report>\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body}"
        )
        .as_bytes(),
    )
    .unwrap();
    let query = response(
        &app,
        "Email/query",
        json!({"accountId":"a","filter":{"text":"café"}}),
    )
    .await;
    assert_eq!(query[0], "Email/query", "{query}");
    assert_eq!(query[1]["ids"], json!(["e1"]));
    assert_eq!(query[1]["canCalculateChanges"], false);
    let snippet = response(
        &app,
        "SearchSnippet/get",
        json!({"accountId":"a","emailIds":["e1","missing"],"filter":{"text":"café"}}),
    )
    .await;
    assert_eq!(snippet[0], "SearchSnippet/get", "{snippet}");
    let preview = snippet[1]["list"][0]["preview"].as_str().unwrap();
    assert!(preview.contains("<mark>CAFÉ</mark>"), "{preview}");
    assert!(
        preview.contains("&lt;script&gt;") && preview.contains("&amp;"),
        "{preview}"
    );
    assert!(!preview.contains("<script>"));
    assert!(preview.len() < 1500);
    assert!(snippet[1]["list"][0]["subject"].is_null());
    assert_eq!(snippet[1]["notFound"], json!(["missing"]));
    let subject = response(
        &app,
        "SearchSnippet/get",
        json!({"accountId":"a","emailIds":["e1"],"filter":{"subject":"report"}}),
    )
    .await;
    assert_eq!(
        subject[1]["list"][0]["subject"],
        "Financial &lt;<mark>Report</mark>&gt;"
    );
    assert!(subject[1]["notFound"].is_null());
}
#[tokio::test]
async fn no_match_metadata_only_and_negative_terms_have_null_snippets() {
    let (db, app) = setup();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: conference\r\n\r\nconference",
    )
    .unwrap();
    for filter in [
        Value::Null,
        json!({"inMailbox":"inbox"}),
        json!({"text":"xylophone"}),
        json!({"operator":"NOT","conditions":[{"text":"conference"}]}),
    ] {
        let result = response(
            &app,
            "SearchSnippet/get",
            json!({"accountId":"a","emailIds":["e1"],"filter":filter}),
        )
        .await;
        assert_eq!(result[0], "SearchSnippet/get", "{result}");
        assert_eq!(
            result[1]["list"],
            json!([{"emailId":"e1","subject":null,"preview":null}])
        );
    }
}
#[tokio::test]
async fn malformed_and_cross_tenant_calls_fail_before_content_access() {
    let (_, app) = setup();
    for (args, kind) in [
        (
            json!({"accountId":"b","emailIds":["missing"]}),
            "accountNotFound",
        ),
        (json!({"accountId":"a","emailIds":[3]}), "invalidArguments"),
        (
            json!({"accountId":"a","emailIds":vec!["x";257]}),
            "tooManyObjects",
        ),
        (
            json!({"accountId":"a","emailIds":[],"filter":{"text":4}}),
            "invalidArguments",
        ),
        (
            json!({"accountId":"a","emailIds":[],"filter":{"unrecognized":"x"}}),
            "unsupportedFilter",
        ),
    ] {
        let result = response(&app, "SearchSnippet/get", args).await;
        assert_eq!(result[0], "error");
        assert_eq!(result[1]["type"], kind, "{result}");
    }
}

#[tokio::test]
async fn boolean_filters_address_search_and_unicode_highlights() {
    let (db, app) = setup();
    db.deliver(&["alice@example.org".into()], "From: Alice Sender <sender@example.net>\r\nSubject: İSTANBUL\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>conference &amp; café</p>".as_bytes()).unwrap();
    for filter in [
        json!({"text":"sender@example.net"}),
        json!({"operator":"AND","conditions":[{"inMailbox":"inbox"},{"body":"café"}]}),
        json!({"operator":"OR","conditions":[{"subject":"absent"},{"text":"conference"}]}),
    ] {
        let result = response(
            &app,
            "Email/query",
            json!({"accountId":"a","filter":filter}),
        )
        .await;
        assert_eq!(result[1]["ids"], json!(["e1"]), "{result}");
    }
    let snippet = response(
        &app,
        "SearchSnippet/get",
        json!({"accountId":"a","emailIds":["e1","e1"],"filter":{"subject":"i"}}),
    )
    .await;
    assert_eq!(snippet[1]["list"].as_array().unwrap().len(), 1);
    assert_eq!(snippet[1]["list"][0]["subject"], "<mark>İ</mark>STANBUL");
    let html = response(
        &app,
        "SearchSnippet/get",
        json!({"accountId":"a","emailIds":["e1"],"filter":{"body":"café"}}),
    )
    .await;
    let preview = html[1]["list"][0]["preview"].as_str().unwrap();
    assert!(preview.contains("&amp; <mark>café</mark>"), "{preview}");
    assert!(!preview.contains("<p>"));
    let mut deep = json!({"text":"x"});
    for _ in 0..18 {
        deep = json!({"operator":"AND","conditions":[deep]});
    }
    let result = response(
        &app,
        "SearchSnippet/get",
        json!({"accountId":"a","emailIds":[],"filter":deep}),
    )
    .await;
    assert_eq!(result[1]["type"], "unsupportedFilter");
}
