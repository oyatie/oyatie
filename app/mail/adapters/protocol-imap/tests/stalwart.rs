#![cfg(feature = "upstream-tests")]
use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::MetadataStore;
use mail_kernel::Account;
use mail_service::MailService;
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, fmt::Debug, future::Future, sync::Arc};
use tower::ServiceExt;
#[path = "stalwart/client.rs"]
mod client;
use client::Client;
#[path = "stalwart/digests.rs"]
mod digests;
#[path = "stalwart/fixtures.rs"]
mod fixtures;
#[path = "stalwart/mutation_client.rs"]
mod mutation_client;
mod thread {
    include!(env!("STALWART_THREAD_SOURCE"));
}
mod inspect {
    include!(env!("STALWART_INSPECT_SOURCE"));
}
mod jmap {
    pub mod compliance {
        pub use crate::{CompCtx, TestOutcome, check, check_contains, check_eq, check_ne, skip};
    }
}
mod mutate {
    include!(env!("STALWART_MUTATE_SOURCE"));
}
mod mailbox {
    include!(env!("STALWART_MAILBOX_SOURCE"));
}
pub fn check_ne<T: PartialEq + Debug>(
    actual: T,
    expected: T,
    message: impl std::fmt::Display,
) -> TestOutcome {
    check(actual != expected, message)
}
mod utils {
    pub mod jmap {
        pub use crate::client::{JmapResponse, JmapUtils};
        pub use crate::mutation_client::ChangeType;
    }
}
mod binary {
    include!(env!("STALWART_BINARY_SOURCE"));
}
pub fn skip(reason: &str) -> TestOutcome {
    Err(format!("unexpected skip: {reason}"))
}
struct SharedPolicy;
impl mail_api::Policy for SharedPolicy {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        _: mail_api::Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), mail_kernel::Error> {
        if principal.subject == "alice"
            && principal.tenant == "t"
            && account.tenant == "t"
            && ["a", "b"].contains(&account.id.as_str())
        {
            Ok(())
        } else {
            Err(mail_kernel::Error::Forbidden)
        }
    }
}

// Upstream assertions compile only into this test executable. They are
// neither copied into nor linked by the mail server's production artifacts.
mod core {
    include!(env!("STALWART_CORE_SOURCE"));
}

const REVISION: &str = "474dd0229cb20cf513036619781ed97bd8073c3f";
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
pub type TestOutcome = Result<(), String>;
pub fn check(condition: bool, message: impl std::fmt::Display) -> TestOutcome {
    if condition {
        Ok(())
    } else {
        Err(message.to_string())
    }
}
pub fn check_eq<T: PartialEq + Debug>(
    actual: T,
    expected: T,
    message: impl std::fmt::Display,
) -> TestOutcome {
    check(
        actual == expected,
        format!("{message}: expected {expected:?}, got {actual:?}"),
    )
}
pub fn check_contains(value: &str, needle: &str, message: impl std::fmt::Display) -> TestOutcome {
    check(
        value.contains(needle),
        format!("{message}: missing {needle:?}"),
    )
}

pub struct CompCtx<'a> {
    primary: &'a Client,
    session: Value,
    cross_account_id: Option<String>,
    pdf: String,
    emails: std::collections::BTreeMap<String, String>,
    outcomes: RefCell<Vec<Value>>,
}
impl CompCtx<'_> {
    fn email(&self, name: &str) -> &str {
        self.emails.get(name).expect("known fixture")
    }
    fn mailbox(&self, name: &str) -> &str {
        match name {
            "folderA" => "m3",
            "folderB" => "m4",
            "child1" => "m5",
            "child2" => "m6",
            _ => panic!("unknown fixture {name}"),
        }
    }
    fn blob(&self, name: &str) -> &str {
        assert_eq!(name, "pdf");
        &self.pdf
    }
    fn download_url(&self, account: &str, blob: &str, typ: &str, name: &str) -> String {
        format!(
            "/download/{account}/{blob}/{name}?type={}",
            typ.replace('/', "%2F")
        )
    }
    async fn upload(&self, client: &Client, typ: &str, data: Vec<u8>) -> Value {
        let response = client
            .http_raw(
                Request::post("/upload/a")
                    .header("authorization", format!("Bearer {TOKEN}"))
                    .header("content-type", typ)
                    .body(Body::from(data))
                    .unwrap(),
            )
            .await;
        serde_json::from_slice(&response.body).expect("upload JSON")
    }
    fn account_id(&self) -> &str {
        "a"
    }
    fn role(&self, role: &str) -> &str {
        assert_eq!(role, "inbox");
        "inbox"
    }
    async fn run(&self, name: &str, future: impl Future<Output = TestOutcome>) {
        let mut future = std::pin::pin!(future);
        let outcome = std::future::poll_fn(|cx| {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                future.as_mut().poll(cx)
            })) {
                Ok(result) => result,
                Err(_) => std::task::Poll::Ready(Err("upstream case panicked".into())),
            }
        })
        .await;
        self.outcomes
            .borrow_mut()
            .push(json!({"case":name,"passed":outcome.is_ok(),"failure":outcome.err()}));
    }
}

#[tokio::test]
async fn upstream_jmap_compliance() {
    digests::verify();
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("b", "t", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], fixtures::plain().as_bytes())
        .unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(1),
        vec![
            mail_kernel::Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: b"Subject: custom keywords\r\n\r\nbody\r\n".to_vec(),
                keywords: vec!["$seen".into(), "$forwarded".into(), "custom_label".into()],
                received_at: 0,
            },
            mail_kernel::Command::CreateMailbox {
                name: "Test Folder A".into(),
            },
            mail_kernel::Command::CreateMailbox {
                name: "Folder B".into(),
            },
        ],
    )
    .unwrap();
    let pdf = db
        .put_blob("a", b"%PDF-1.4\n1 0 obj <</Type /Catalog>> endobj\n%%EOF")
        .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(SharedPolicy),
    });
    let router = mail_protocol_imap::jmap_router(service, "http://localhost:8080".into());
    let response = router
        .clone()
        .oneshot(
            Request::get("/.well-known/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let session =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let client = Client { router };
    for name in ["Child 1", "Child 2"] {
        let created = client
            .jmap_create(
                "Mailbox",
                [json!({"name":name,"parentId":"m3"})],
                Vec::<(String, Value)>::new(),
            )
            .await;
        assert!(created.created(0)["id"].is_string());
    }
    let emails = fixtures::seed(&db);
    let ctx = CompCtx {
        primary: &client,
        session,
        cross_account_id: Some("b".into()),
        pdf,
        emails,
        outcomes: RefCell::new(vec![]),
    };
    core::run(&ctx).await;
    binary::run(&ctx).await;
    inspect::run(&ctx).await;
    thread::run(&ctx).await;
    mutate::run(&ctx).await;
    mailbox::run(&ctx).await;
    let outcomes = ctx.outcomes.into_inner();
    let passed = outcomes.iter().filter(|o| o["passed"] == true).count();
    println!("{}",serde_json::to_string_pretty(&json!({"upstream_revision":REVISION,"suites":["tests/src/jmap/compliance/core.rs","tests/src/jmap/compliance/binary.rs","tests/src/jmap/compliance/email/mutate.rs","tests/src/jmap/compliance/mailbox.rs","tests/src/jmap/compliance/email/inspect.rs","tests/src/jmap/compliance/thread.rs"],"transport":"in-process HTTP router","passed":passed,"executed":outcomes.len(),"skipped":0,"full_parity":false,"cases":outcomes})).unwrap());
    assert_eq!(
        outcomes.len(),
        198,
        "the complete pinned Core, Binary, Email mutation, inspection and Mailbox suites must execute"
    );
    assert_eq!(passed, outcomes.len(), "upstream compliance failures");
}
