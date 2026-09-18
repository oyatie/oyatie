use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::{Action, MetadataStore, Policy};
use mail_kernel::{Account, Error};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tower::ServiceExt;
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
fn setup(policy: Arc<dyn Policy>) -> (Arc<SqliteStore>, Router) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    for (id, tenant, owner, address, token) in [
        ("a", "t", "alice", "alice@example.org", TOKEN),
        (
            "b",
            "other",
            "bob",
            "bob@example.org",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
    ] {
        db.provision(Account::new(id, tenant, owner, address).unwrap(), token)
            .unwrap();
    }
    let app = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy,
        }),
        "http://localhost".into(),
    );
    (db, app)
}
async fn calls(app: &Router, calls: Value) -> Value {
    let request = Request::post("/jmap")
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json")
        .body(Body::from(json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],"methodCalls":calls}).to_string())).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status().is_success(), "{}", response.status());
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}
async fn call(app: &Router, name: &str, args: Value) -> Value {
    calls(app, json!([[name, args, "r"]])).await["methodResponses"][0].clone()
}
#[tokio::test]
async fn identity_update_scheduled_for_destruction_is_skipped() {
    let (db, app) = setup(Arc::new(OwnerPolicy));
    let before = db.account("a").unwrap();
    let response = call(
        &app,
        "Identity/set",
        json!({"accountId":"a","update":{"a":{"name":"Must not persist"}},"destroy":["a"]}),
    )
    .await;
    assert_eq!(
        response[1]["notUpdated"]["a"]["type"], "willDestroy",
        "{response}"
    );
    assert_eq!(response[1]["notDestroyed"]["a"]["type"], "forbidden");
    assert!(response[1]["updated"].as_object().unwrap().is_empty());
    assert_eq!(db.account("a").unwrap(), before);
}
#[tokio::test]
async fn identity_nested_fields_and_malformed_arguments_never_partially_mutate() {
    let (db, app) = setup(Arc::new(OwnerPolicy));
    let before = db.account("a").unwrap();
    for patch in [
        json!({"name":"changed","mayDelete":true}),
        json!({"name":"changed","replyTo":[{"email":"r@example.org","bcc":"secret@example.org"}]}),
        json!({"name":"changed","bcc":[{"name":"x\r\nSubject: forged","email":"r@example.org"}]}),
        json!({"name":"changed","replyTo/0/email":"r@example.org"}),
        json!({"name":"changed","htmlSignature":"bad\u{0000}html"}),
    ] {
        let response = call(
            &app,
            "Identity/set",
            json!({"accountId":"a","update":{"a":patch}}),
        )
        .await;
        assert_eq!(
            response[1]["notUpdated"]["a"]["type"], "invalidProperties",
            "{response}"
        );
        assert_eq!(db.account("a").unwrap(), before);
    }
    for destroy in [json!(5), json!([null]), json!(["a", false])] {
        let response = call(
            &app,
            "Identity/set",
            json!({"accountId":"a","update":{"a":{"name":"changed"}},"destroy":destroy}),
        )
        .await;
        assert_eq!(response[1]["type"], "invalidArguments");
        assert_eq!(db.account("a").unwrap(), before);
    }
}
struct RevokeAfterRead(AtomicUsize);
impl Policy for RevokeAfterRead {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        action: Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), Error> {
        OwnerPolicy.authorize(principal, action, account)?;
        if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
            Ok(())
        } else {
            Err(Error::Forbidden)
        }
    }
}
#[tokio::test]
async fn policy_is_rechecked_before_identity_write_and_snippet_blob_access() {
    for snippet in [false, true] {
        let (db, app) = setup(Arc::new(RevokeAfterRead(AtomicUsize::new(0))));
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: secret\r\n\r\nsecret",
        )
        .unwrap();
        let before = db.account("a").unwrap();
        let response = if snippet {
            call(
                &app,
                "SearchSnippet/get",
                json!({"accountId":"a","emailIds":["e1"],"filter":{"text":"secret"}}),
            )
            .await
        } else {
            call(
                &app,
                "Identity/set",
                json!({"accountId":"a","update":{"a":{"name":"changed"}}}),
            )
            .await
        };
        if snippet {
            assert_eq!(response[1]["type"], "forbidden", "{response}");
            assert!(!response.to_string().contains("secret"));
        } else {
            assert_eq!(
                response[1]["notUpdated"]["a"]["type"], "forbidden",
                "{response}"
            );
        }
        assert_eq!(db.account("a").unwrap(), before);
    }
}
#[tokio::test]
async fn cross_tenant_methods_and_object_overflow_cannot_modify_identity() {
    let (db, app) = setup(Arc::new(OwnerPolicy));
    let before = db.account("b").unwrap();
    for name in [
        "Identity/get",
        "Identity/changes",
        "Identity/set",
        "SearchSnippet/get",
        "Email/query",
    ] {
        let response = call(&app, name, json!({"accountId":"b","sinceState":"0","update":{"b":{"name":"stolen"}},"emailIds":["e1"],"filter":{"text":"secret"}})).await;
        assert_eq!(response[1]["type"], "accountNotFound", "{response}");
    }
    assert_eq!(db.account("b").unwrap(), before);
    let before = db.account("a").unwrap();
    let response = call(
        &app,
        "Identity/set",
        json!({"accountId":"a","update":{"a":{"name":"changed"}},"destroy":vec!["missing";256]}),
    )
    .await;
    assert_eq!(response[1]["type"], "tooManyObjects");
    assert_eq!(db.account("a").unwrap(), before);
}
#[tokio::test]
async fn identity_receipt_survives_maximum_object_response_and_reference() {
    let (db, app) = setup(Arc::new(OwnerPolicy));
    let create: serde_json::Map<_, _> = (0..255)
        .map(|i| (format!("{i}{}", "x".repeat(3500)), json!({})))
        .collect();
    let response = calls(&app, json!([
        ["Core/echo", {"create":create}, "e"],
        ["Identity/set", {"accountId":"a","#create":{"resultOf":"e","name":"Core/echo","path":"/create"},"update":{"a":{"name":"committed"}}}, "s"],
        ["Identity/changes", {"accountId":"a","sinceState":"0"}, "c"]
    ])).await;
    let set = &response["methodResponses"][1];
    assert_eq!(set[0], "Identity/set", "{set}");
    assert!(set[1]["updated"].as_object().unwrap().contains_key("a"));
    assert_eq!(set[1]["notCreated"].as_object().unwrap().len(), 255);
    assert_eq!(db.account("a").unwrap().identity.name, "committed");
    assert_eq!(
        response["methodResponses"][2][1]["newState"],
        set[1]["newState"]
    );
}
#[tokio::test]
async fn unicode_expansions_overlaps_and_escaped_markup_keep_valid_boundaries() {
    let (db, app) = setup(Arc::new(OwnerPolicy));
    db.deliver(
        &["alice@example.org".into()],
        "Subject: İ & <İ> ΟΣ 😀\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n<İ> 'x' & 😀"
            .as_bytes(),
    )
    .unwrap();
    for term in ["i", "\u{0307}", "i\u{0307}", "ος", "😀"] {
        let response = call(
            &app,
            "SearchSnippet/get",
            json!({"accountId":"a","emailIds":["e1"],"filter":{"text":term}}),
        )
        .await;
        assert_eq!(response[0], "SearchSnippet/get", "{response}");
        let subject = response[1]["list"][0]["subject"].as_str().unwrap();
        assert!(!subject.contains("<İ>"), "{subject}");
        assert_eq!(
            subject.matches("<mark>").count(),
            subject.matches("</mark>").count()
        );
        assert!(subject.contains("<mark>"), "{subject}");
    }
    let response = call(&app, "SearchSnippet/get", json!({"accountId":"a","emailIds":["e1"],"filter":{"operator":"OR","conditions":[{"subject":"i"},{"subject":"i\u{0307}"}]}})).await;
    assert_eq!(
        response[1]["list"][0]["subject"],
        "<mark>İ</mark> &amp; &lt;<mark>İ</mark>&gt; ΟΣ 😀"
    );
}
#[tokio::test]
async fn boolean_query_uses_decoded_text_without_negative_snippet_highlights() {
    let (db, app) = setup(Arc::new(OwnerPolicy));
    db.deliver(&["alice@example.org".into()], b"Subject: =?UTF-8?B?Y2Fmw6k=?=\r\nContent-Transfer-Encoding: base64\r\n\r\nY29uZmVyZW5jZQ==").unwrap();
    let filter = json!({"operator":"AND","conditions":[{"subject":"café"},{"operator":"NOT","conditions":[{"body":"absent"},{"subject":"wrong"}]}]});
    let query = call(
        &app,
        "Email/query",
        json!({"accountId":"a","filter":filter}),
    )
    .await;
    assert_eq!(query[1]["ids"], json!(["e1"]), "{query}");
    let snippet = call(
        &app,
        "SearchSnippet/get",
        json!({"accountId":"a","emailIds":["e1"],"filter":filter}),
    )
    .await;
    assert_eq!(snippet[1]["list"][0]["subject"], "<mark>café</mark>");
    assert!(snippet[1]["list"][0]["preview"].is_null());
}
