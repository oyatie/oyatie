#![cfg(feature = "upstream-tests")]
use mail_api::MetadataStore;
use mail_kernel::{Account, Command, MailboxProperties};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fmt::Debug,
    future::{Future, poll_fn},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    task::Poll,
};
#[path = "stalwart/profile_client.rs"]
#[allow(dead_code)] // Shared client also supports the identity and snippet suites.
mod client;
#[path = "stalwart/submission_client.rs"]
mod submission_client;
use client::Client;
mod utils {
    pub mod jmap {
        pub use crate::submission_client::JmapUtils;
    }
}
mod submission {
    include!(env!("STALWART_SUBMISSION_SOURCE"));
}
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
type TestOutcome = Result<(), String>;
fn check(condition: bool, message: impl std::fmt::Display) -> TestOutcome {
    if condition {
        Ok(())
    } else {
        Err(message.to_string())
    }
}
fn check_eq<T: PartialEq + Debug>(
    actual: T,
    expected: T,
    message: impl std::fmt::Display,
) -> TestOutcome {
    check(
        actual == expected,
        format!("{message}: expected {expected:?}, got {actual:?}"),
    )
}
fn skip(reason: &str) -> TestOutcome {
    Err(format!("unexpected skip: {reason}"))
}
struct CompCtx<'a> {
    primary: &'a Client,
    identity_ids: Vec<String>,
    identity_email: &'a str,
    secondary_email: &'a str,
    roles: BTreeMap<String, String>,
    outcomes: RefCell<Vec<Value>>,
}
impl CompCtx<'_> {
    fn account_id(&self) -> &str {
        "a"
    }
    fn role_opt(&self, role: &str) -> Option<&str> {
        self.roles.get(role).map(String::as_str)
    }
    fn role(&self, role: &str) -> &str {
        self.role_opt(role).expect("required mailbox role")
    }
    async fn run(&self, name: &str, future: impl Future<Output = TestOutcome>) {
        let mut future = std::pin::pin!(future);
        // Missing baseline response fields can panic upstream helper accessors.
        // Record those as failures and still execute every unchanged oracle case.
        let result =
            poll_fn(
                |cx| match catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(cx))) {
                    Ok(result) => result,
                    Err(error) => Poll::Ready(Err(format!(
                        "oracle panic: {}",
                        error
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .or_else(|| error.downcast_ref::<&str>().copied())
                            .unwrap_or("non-string panic")
                    ))),
                },
            )
            .await;
        self.outcomes
            .borrow_mut()
            .push(json!({"case":name,"passed":result.is_ok(),"failure":result.err()}));
    }
}

#[tokio::test]
async fn unchanged_upstream_submission_suite() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(env!("STALWART_SUBMISSION_SOURCE")).unwrap())
        ),
        "28b5250032ed10cbdc03c887b9297a5b1cfa8ea2b52184037151b6e60546a570",
        "upstream submission source must remain pinned and unchanged"
    );
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    for (id, owner, address, token) in [
        ("a", "alice", "alice@example.org", TOKEN),
        (
            "b",
            "bob",
            "bob@example.org",
            "abcdef0123456789abcdef0123456789",
        ),
    ] {
        db.provision(Account::new(id, "t", owner, address).unwrap(), token)
            .unwrap();
    }
    for role in ["drafts", "sent"] {
        let revision = db.account("a").unwrap().revision;
        db.execute(
            "a",
            mail_api::Precondition::Observed(revision),
            vec![Command::SetMailbox {
                id: None,
                properties: MailboxProperties {
                    name: role.into(),
                    role: Some(role.into()),
                    ..MailboxProperties::named(role.into())
                },
            }],
        )
        .unwrap();
    }
    let roles = db
        .account("a")
        .unwrap()
        .mailboxes
        .into_iter()
        .filter_map(|mailbox| mailbox.role.map(|role| (role, mailbox.id)))
        .collect();
    let service = Arc::new(MailService {
        outbound: Some(db.clone()),
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let client = Client {
        router: mail_protocol_imap::jmap_router(service, "https://mail.example.org".into()),
    };
    let ctx = CompCtx {
        primary: &client,
        identity_ids: vec!["a".into()],
        identity_email: "alice@example.org",
        secondary_email: "bob@example.org",
        roles,
        outcomes: RefCell::new(vec![]),
    };
    submission::run(&ctx).await;
    let outcomes = ctx.outcomes.into_inner();
    println!("{}", serde_json::to_string_pretty(&json!({"revision":"474dd0229cb20cf513036619781ed97bd8073c3f","source":"tests/src/jmap/compliance/submission.rs","transport":"in-process HTTP router with durable SQLite submission queue","cases":outcomes})).unwrap());
    assert_eq!(
        outcomes.len(),
        14,
        "all unchanged submission cases must execute"
    );
    assert!(
        outcomes.iter().all(|case| case["passed"] == true),
        "{outcomes:?}"
    );
}
