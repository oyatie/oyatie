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
fn router(db: &Arc<SqliteStore>, policy: Arc<dyn mail_api::Policy>) -> Router {
    mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy,
        }),
        "http://localhost".into(),
    )
}
async fn response(app: &Router, name: &str, args: Value) -> Value {
    let response = app.clone().oneshot(Request::post("/jmap").header("authorization", format!("Bearer {TOKEN}")).header("content-type", "application/json").body(Body::from(json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:submission"],"methodCalls":[[name,args,"x"]]}).to_string())).unwrap()).await.unwrap();
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
async fn identity_is_persistent_and_changes_are_isolated_from_email_state() {
    let db = setup();
    let app = router(&db, Arc::new(OwnerPolicy));
    let get = response(&app, "Identity/get", json!({"accountId":"a"})).await;
    assert_eq!(get[0], "Identity/get", "{get}");
    assert_eq!(get[1]["list"].as_array().unwrap().len(), 1);
    let id = get[1]["list"][0]["id"].as_str().unwrap();
    assert_eq!(get[1]["list"][0]["email"], "alice@example.org");
    assert_eq!(get[1]["list"][0]["mayDelete"], false);
    let patch = json!({"name":"Alice","textSignature":"Regards","htmlSignature":"<p>Regards</p>","replyTo":[{"name":"Reply","email":"reply@example.org"}],"bcc":[{"name":null,"email":"archive@example.org"}]});
    let set = response(
        &app,
        "Identity/set",
        json!({"accountId":"a","ifInState":get[1]["state"],"update":{id:patch}}),
    )
    .await;
    assert_eq!(set[0], "Identity/set", "{set}");
    assert!(
        set[1]["updated"].as_object().unwrap().contains_key(id),
        "{set}"
    );
    let reloaded = router(&db, Arc::new(OwnerPolicy));
    let current = response(
        &reloaded,
        "Identity/get",
        json!({"accountId":"a","ids":[id]}),
    )
    .await;
    for (property, value) in patch.as_object().unwrap() {
        assert_eq!(&current[1]["list"][0][property], value);
    }
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: change\r\n\r\nbody",
    )
    .unwrap();
    let changes = response(
        &reloaded,
        "Identity/changes",
        json!({"accountId":"a","sinceState":get[1]["state"]}),
    )
    .await;
    assert_eq!(changes[1]["updated"], json!([id]));
    assert_eq!(changes[1]["newState"], current[1]["state"]);
    let unchanged = response(
        &reloaded,
        "Identity/changes",
        json!({"accountId":"a","sinceState":current[1]["state"]}),
    )
    .await;
    assert_eq!(unchanged[1]["updated"], json!([]));
    let stale = response(
        &app,
        "Identity/set",
        json!({"accountId":"a","ifInState":get[1]["state"],"update":{id:{"name":"Lost"}}}),
    )
    .await;
    assert_eq!(stale[1]["type"], "stateMismatch");
}
struct ReadOnly;
impl mail_api::Policy for ReadOnly {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        action: mail_api::Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), mail_kernel::Error> {
        mail_api::Policy::authorize(&OwnerPolicy, principal, action, account)?;
        if action == mail_api::Action::Read {
            Ok(())
        } else {
            Err(mail_kernel::Error::Forbidden)
        }
    }
}
#[tokio::test]
async fn identity_patch_validation_and_policy_are_atomic() {
    let db = setup();
    let app = router(&db, Arc::new(OwnerPolicy));
    let get = response(&app, "Identity/get", json!({"accountId":"a"})).await;
    assert_eq!(get[0], "Identity/get", "{get}");
    let id = get[1]["list"][0]["id"].as_str().unwrap();
    for patch in [
        json!({"name":"Changed","email":"victim@example.org"}),
        json!({"replyTo":[{"email":"x\r\nBcc: victim@example.org"}]}),
        json!({"bcc":[]}),
        json!({"name":"x\r\ny"}),
        json!({"textSignature":"x".repeat(65537)}),
    ] {
        let before = db.account("a").unwrap();
        let result = response(
            &app,
            "Identity/set",
            json!({"accountId":"a","update":{id:patch}}),
        )
        .await;
        assert_eq!(
            result[1]["notUpdated"][id]["type"], "invalidProperties",
            "{result}"
        );
        assert_eq!(db.account("a").unwrap(), before);
    }
    let result = response(
        &router(&db, Arc::new(ReadOnly)),
        "Identity/set",
        json!({"accountId":"a","update":{id:{"name":"No"}}}),
    )
    .await;
    assert_eq!(result[1]["notUpdated"][id]["type"], "forbidden");
    let foreign = response(&app, "Identity/get", json!({"accountId":"b"})).await;
    assert_eq!(foreign[1]["type"], "accountNotFound");
    let missing = response(&app, "Identity/set", json!({"accountId":"a","update":{"missing":{"name":"No"}},"destroy":[id],"create":{"new":{"email":"other@example.org"}}})).await;
    assert_eq!(missing[1]["notUpdated"]["missing"]["type"], "notFound");
    assert_eq!(missing[1]["notDestroyed"][id]["type"], "forbidden");
    assert_eq!(missing[1]["notCreated"]["new"]["type"], "forbidden");
}

#[tokio::test]
async fn legacy_snapshot_loads_and_identity_survives_database_reopen() {
    let path = std::env::temp_dir().join(format!(
        "mail-identity-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let before;
    {
        let db = Arc::new(SqliteStore::open(&path).unwrap());
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: legacy\r\n\r\nretained",
        )
        .unwrap();
        before = db.account("a").unwrap();
    }
    {
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute("UPDATE accounts SET state=json_remove(state,'$.identity','$.identity_revision') WHERE id='a'", []).unwrap();
    }
    let state;
    {
        let db = Arc::new(SqliteStore::open(&path).unwrap());
        assert_eq!(
            db.account("a").unwrap(),
            before,
            "legacy snapshot defaults must preserve existing mail state"
        );
        let app = router(&db, Arc::new(OwnerPolicy));
        let result = response(&app, "Identity/set", json!({"accountId":"a","update":{"a":{"name":"Durable","replyTo":[{"email":"reply@example.org"}]}}})).await;
        assert!(
            result[1]["updated"].as_object().unwrap().contains_key("a"),
            "{result}"
        );
        state = result[1]["newState"].clone();
    }
    {
        let db = Arc::new(SqliteStore::open(&path).unwrap());
        let app = router(&db, Arc::new(OwnerPolicy));
        let result = response(&app, "Identity/get", json!({"accountId":"a"})).await;
        assert_eq!(result[1]["list"][0]["name"], "Durable");
        assert_eq!(result[1]["state"], state);
        assert_eq!(db.account("a").unwrap().messages, before.messages);
        assert_eq!(db.account("a").unwrap().mail_modseq, before.mail_modseq);
        assert_eq!(
            db.blob("a", "e1").unwrap(),
            b"Subject: legacy\r\n\r\nretained"
        );
    }
    std::fs::remove_file(path).unwrap();
}
