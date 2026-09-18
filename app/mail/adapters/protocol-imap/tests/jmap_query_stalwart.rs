#![cfg(feature = "upstream-tests")]
// The unchanged pinned Email/query compliance suite compiles only here. The
// `chrono` alias resolves the suite's clock import to the shim below.
extern crate self as chrono;
use axum::{Router, body::Body, http::Request};
use mail_api::MetadataStore;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, collections::BTreeMap, fmt::Debug, future::Future, sync::Arc};
#[path = "stalwart/query_builders.rs"]
mod builders;
#[path = "stalwart/client.rs"]
mod client;
#[path = "stalwart/query_clock.rs"]
mod clock;
#[path = "stalwart/query_corpus.rs"]
mod corpus;
#[path = "stalwart/query_corpus_tail.rs"]
mod corpus_tail;
#[path = "stalwart/query_seed.rs"]
mod seed;
use client::Client;
pub use clock::{DateTime, Duration, Utc};
mod jmap {
    pub mod compliance {
        pub use crate::{CompCtx, TestOutcome, check, check_eq};
    }
}
mod utils {
    pub mod jmap {
        pub use crate::client::JmapResponse;
    }
}
mod query {
    include!(env!("STALWART_EMAIL_QUERY_SOURCE"));
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
pub struct CompCtx<'a> {
    primary: &'a Client,
    account_id: String,
    pub email_ids: BTreeMap<String, String>,
    mailbox_ids: BTreeMap<String, String>,
    role_mailboxes: BTreeMap<String, String>,
    outcomes: RefCell<Vec<Value>>,
}
impl CompCtx<'_> {
    fn account_id(&self) -> &str {
        &self.account_id
    }
    fn email(&self, key: &str) -> &str {
        self.email_ids
            .get(key)
            .unwrap_or_else(|| panic!("seed email {key} not found"))
    }
    fn mailbox(&self, key: &str) -> &str {
        self.mailbox_ids
            .get(key)
            .unwrap_or_else(|| panic!("seed mailbox {key} not found"))
    }
    fn role(&self, key: &str) -> &str {
        self.role_mailboxes
            .get(key)
            .unwrap_or_else(|| panic!("role mailbox {key} not found"))
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
async fn upstream_jmap_email_query() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_EMAIL_QUERY_SOURCE")))
        ),
        "e7ca6011ba102f4ed4891de1cb1a0e135a3ab277d1a8edba58444d6f1cb15827",
        "upstream Email/query suite changed; review before updating the baseline"
    );
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "testuser@example.com").unwrap(),
        TOKEN,
    )
    .unwrap();
    let seeded = seed::seed(&db, "a");
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let router: Router = mail_protocol_imap::jmap_router(service, "http://localhost:8080".into());
    let client = Client { router };
    let session = client
        .http_raw(
            Request::get("/.well-known/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(session.status, 200);
    let ctx = CompCtx {
        primary: &client,
        account_id: "a".into(),
        email_ids: seeded.emails,
        mailbox_ids: seeded.mailboxes,
        role_mailboxes: seeded.roles,
        outcomes: RefCell::new(vec![]),
    };
    query::run(&ctx).await;
    let outcomes = ctx.outcomes.into_inner();
    let passed = outcomes.iter().filter(|o| o["passed"] == true).count();
    println!("{}",serde_json::to_string_pretty(&json!({"upstream_revision":REVISION,"suite":"tests/src/jmap/compliance/email/query.rs","transport":"in-process HTTP router","passed":passed,"executed":outcomes.len(),"skipped":0,"cases":outcomes})).unwrap());
    assert_eq!(
        outcomes.len(),
        57,
        "the complete pinned Email/query suite must execute"
    );
    assert_eq!(passed, outcomes.len(), "upstream Email/query failures");
}
