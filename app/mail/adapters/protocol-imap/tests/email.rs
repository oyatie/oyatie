use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::Store;
use mail_kernel::{Account, Command};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";
#[path = "email/content.rs"]
mod content;
#[path = "email/decoding.rs"]
mod decoding;
#[path = "email/inspect.rs"]
mod inspect;
#[path = "email/limits.rs"]
mod limits;
#[path = "email/mailbox.rs"]
mod mailbox;
async fn call(app: &Router, name: &str, args: Value) -> Value {
    let value = response(app, name, args).await;
    assert_eq!(value["methodResponses"][0][0], name, "{value}");
    value["methodResponses"][0][1].clone()
}

async fn response(app: &Router, name: &str, args: Value) -> Value {
    let response = app.clone().oneshot(Request::post("/jmap")
        .header("authorization",format!("Bearer {TOKEN}")).header("content-type","application/json")
        .body(Body::from(json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[[name,args,"c"]]}).to_string())).unwrap()).await.unwrap();
    assert!(response.status().is_success());
    let value: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    value
}

#[tokio::test]
async fn changes_page_at_commits_and_query_delta_reconstructs_the_list() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
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
    let append = |date| Command::Append {
        mailboxes: vec!["inbox".into()],
        received_at: date,
        raw: b"Subject: sync\r\n\r\nbody".to_vec(),
        keywords: vec![],
    };
    db.execute("a", 0, vec![append(20)]).unwrap();
    db.execute("a", 1, vec![append(10)]).unwrap();
    let old = call(&app, "Email/query", json!({"accountId":"a"})).await;
    assert_eq!(old["ids"], json!(["e1", "e2"]));
    db.execute("a", 2, vec![append(30)]).unwrap();
    db.execute("a", 3, vec![Command::Destroy { id: "e2".into() }])
        .unwrap();
    let first = call(
        &app,
        "Email/changes",
        json!({"accountId":"a","sinceState":"2","maxChanges":1}),
    )
    .await;
    assert_eq!(first["created"], json!(["e3"]));
    assert_eq!(first["newState"], "3");
    assert_eq!(first["hasMoreChanges"], true);
    let next = call(
        &app,
        "Email/changes",
        json!({"accountId":"a","sinceState":first["newState"],"maxChanges":1}),
    )
    .await;
    assert_eq!(next["destroyed"], json!(["e2"]));
    assert_eq!(next["hasMoreChanges"], false);
    let delta = call(
        &app,
        "Email/queryChanges",
        json!({"accountId":"a","sinceQueryState":old["queryState"]}),
    )
    .await;
    let mut ids = old["ids"].as_array().unwrap().clone();
    ids.retain(|id| !delta["removed"].as_array().unwrap().contains(id));
    for item in delta["added"].as_array().unwrap() {
        ids.insert(item["index"].as_u64().unwrap() as usize, item["id"].clone());
    }
    let current = call(&app, "Email/query", json!({"accountId":"a"})).await;
    assert_eq!(json!(ids), current["ids"]);
    let mismatch = response(&app,"Email/queryChanges",json!({"accountId":"a","filter":{"inMailbox":"missing"},"sinceQueryState":old["queryState"]})).await;
    assert_eq!(
        mismatch["methodResponses"][0][1]["type"],
        "cannotCalculateChanges"
    );
    db.execute("a", 4, vec![append(40), append(50)]).unwrap();
    let atomic = response(
        &app,
        "Email/changes",
        json!({"accountId":"a","sinceState":"4","maxChanges":1}),
    )
    .await;
    assert_eq!(
        atomic["methodResponses"][0][1]["type"],
        "cannotCalculateChanges"
    );
    db.execute("a", 6, vec![append(60)]).unwrap();
    db.execute("a", 7, vec![Command::Destroy { id: "e7".into() }])
        .unwrap();
    let cancelled = call(
        &app,
        "Email/changes",
        json!({"accountId":"a","sinceState":"6"}),
    )
    .await;
    assert_eq!(cancelled["created"], json!([]));
    assert_eq!(cancelled["destroyed"], json!([]));
}

struct CopyPolicy {
    source_write: bool,
    target_write: bool,
}
impl mail_api::Policy for CopyPolicy {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        action: mail_api::Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), mail_kernel::Error> {
        if principal.tenant == account.tenant
            && (action == mail_api::Action::Read
                || (account.id == "a" && self.source_write)
                || (account.id == "b" && self.target_write))
        {
            Ok(())
        } else {
            Err(mail_kernel::Error::Forbidden)
        }
    }
}

#[tokio::test]
async fn copy_and_implicit_destroy_apply_each_accounts_write_policy() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "t", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], b"Subject: copy\r\n\r\nbody")
        .unwrap();
    let router = |source_write, target_write| {
        mail_protocol_imap::jmap_router(
            Arc::new(MailService {
                outbound: None,
                queue: db.clone(),
                store: db.clone(),
                identity: db.clone(),
                policy: Arc::new(CopyPolicy {
                    source_write,
                    target_write,
                }),
            }),
            "http://localhost".into(),
        )
    };
    let args = json!({"accountId":"b","fromAccountId":"a","create":{"x":{"id":"e1","mailboxIds":{"inbox":true}}},"onSuccessDestroyOriginal":true});
    let denied = response(&router(true, false), "Email/copy", args.clone()).await;
    assert_eq!(
        denied["methodResponses"][0][1]["notCreated"]["x"]["type"],
        "forbidden"
    );
    assert_eq!(db.account("a").unwrap().messages.len(), 1);
    assert!(db.account("b").unwrap().messages.is_empty());
    let copied = response(&router(false, true), "Email/copy", args.clone()).await;
    assert!(copied["methodResponses"][0][1]["created"]["x"]["id"].is_string());
    assert_eq!(copied["methodResponses"][1][0], "Email/set");
    assert_eq!(
        copied["methodResponses"][1][1]["notDestroyed"]["e1"]["type"],
        "forbidden"
    );
    assert_eq!(db.account("a").unwrap().messages.len(), 1);
    let copied = response(&router(true, true), "Email/copy", args).await;
    assert_eq!(copied["methodResponses"][1][1]["destroyed"], json!(["e1"]));
    assert!(db.account("a").unwrap().messages.is_empty());
    assert_eq!(db.account("b").unwrap().messages.len(), 2);
    assert_eq!(db.blob("b", "e1").unwrap(), b"Subject: copy\r\n\r\nbody");
    let app = router(true, true);
    let copied_blob = call(
        &app,
        "Blob/copy",
        json!({"fromAccountId":"b","accountId":"a","blobIds":["e1.0"]}),
    )
    .await;
    let blob = copied_blob["copied"]["e1.0"]
        .as_str()
        .expect("MIME part must be copyable");
    assert_eq!(db.blob("a", blob).unwrap(), b"body");
    let draft = call(&app,"Email/set",json!({"accountId":"b","create":{"draft":{
        "mailboxIds":{"inbox":true},"subject":"reused part","bodyStructure":{"type":"text/plain","blobId":"e1.0"}
    }}})).await;
    assert!(draft["created"]["draft"]["id"].is_string(), "{draft}");
}

#[tokio::test]
async fn import_and_patch_preserve_raw_content_and_mailbox_uids() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.execute(
        "a",
        0,
        vec![Command::CreateMailbox {
            name: "Archive".into(),
        }],
    )
    .unwrap();
    let raw =
        b"From: sender@example.net\r\nTo: alice@example.org\r\nSubject: imported\r\n\r\n\xff\r\n";
    let blob = db.put_blob("a", raw).unwrap();
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
    let result = call(&app,"Email/import",json!({"accountId":"a","ifInState":"1","emails":{
        "good":{"blobId":blob,"mailboxIds":{"inbox":true,"m1":true},"keywords":{"$seen":true},"receivedAt":"2025-06-15T10:30:00Z"},
        "bad":{"blobId":"missing","mailboxIds":{"inbox":true}}
    }})).await;
    assert_eq!(result["notCreated"]["bad"]["type"], "invalidProperties");
    let id = result["created"]["good"]["id"]
        .as_str()
        .expect("created email");
    assert_eq!(result["created"]["good"]["size"], raw.len());
    let get = call(&app, "Email/get", json!({"accountId":"a","ids":[id]})).await;
    assert_eq!(get["list"][0]["receivedAt"], "2025-06-15T10:30:00Z");
    assert_eq!(
        get["list"][0]["mailboxIds"],
        json!({"inbox":true,"m1":true})
    );
    assert_eq!(db.blob("a", id).unwrap(), raw);
    let before = db.account("a").unwrap();
    let failed = call(
        &app,
        "Email/set",
        json!({"accountId":"a","update":{id:{"keywords/$flagged":true,"mailboxIds/missing":true}}}),
    )
    .await;
    assert_eq!(failed["notUpdated"][id]["type"], "invalidProperties");
    assert_eq!(db.account("a").unwrap(), before);
    let result = call(
        &app,
        "Email/set",
        json!({"accountId":"a","update":{id:{"keywords/$seen":null,"mailboxIds/inbox":null}}}),
    )
    .await;
    assert!(result["updated"].as_object().unwrap().contains_key(id));
    let account = db.account("a").unwrap();
    assert_eq!(account.messages[0].uid_in("m1"), Some(1));
    assert_eq!(account.messages[0].uid_in("inbox"), None);
    assert!(account.messages[0].keywords.is_empty());
    assert_eq!(db.blob("a", &account.messages[0].id).unwrap(), raw);
}
