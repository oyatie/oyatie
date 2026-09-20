use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::{MetadataStore, Policy};
use mail_kernel::{Account, Command};
use mail_service::MailService;
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use tower::ServiceExt;
pub const TOKEN: &str = "0123456789abcdef0123456789abcdef";
pub const RAW: &[u8] = b"From: alice@example.org\r\nTo: visible@remote.org\r\nBcc: blind@remote.org\r\n\t, hidden@remote.org\r\nResent-Bcc: hidden-again@remote.org\r\nSubject: draft\r\n\r\nbody\r\n";
pub struct Fixture {
    pub db: Arc<SqliteStore>,
    pub service: Arc<MailService>,
    pub app: Router,
}
impl Fixture {
    pub fn new(path: impl AsRef<Path>, raw: &[u8], policy: Arc<dyn Policy>) -> Self {
        let db = Arc::new(SqliteStore::open(path).unwrap());
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
        db.execute(
            "a",
            mail_api::Precondition::Observed(0),
            vec![Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: raw.to_vec(),
                keywords: vec!["$draft".into(), "$seen".into()],
                received_at: 1,
            }],
        )
        .unwrap();
        let service = Arc::new(MailService {
            outbound: Some(db.clone()),
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy,
        });
        let app =
            mail_protocol_imap::jmap_router(service.clone(), "https://mail.example.org".into());
        Self { db, service, app }
    }
    pub async fn call(&self, method: &str, args: Value) -> Value {
        let body = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],"methodCalls":[[method,args,"r"]]});
        let response = self
            .app
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
    pub async fn create(&self, envelope: Value) -> Value {
        let mut object = json!({"identityId":"a","emailId":"e1"});
        if !envelope.is_null() {
            object["envelope"] = envelope;
        }
        self.call(
            "EmailSubmission/set",
            json!({"accountId":"a","create":{"s":object}}),
        )
        .await
    }
}
pub fn id(response: &Value) -> String {
    response["methodResponses"][0][1]["created"]["s"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("submission creation failed: {response}"))
        .into()
}
pub fn envelope(sender: &str, recipients: &[&str]) -> Value {
    json!({"mailFrom":{"email":sender},"rcptTo":recipients.iter().map(|r| json!({"email":r})).collect::<Vec<_>>()})
}
pub fn db_path() -> std::path::PathBuf {
    // Parallel tests start within one clock tick; a sequence keeps files apart.
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "mail-submission-review-{}-{}.db",
        std::process::id(),
        SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}
