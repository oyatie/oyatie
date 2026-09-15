use mail_api::Store;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite::SqliteStore;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, fmt::Debug, future::Future, sync::Arc};
#[path = "stalwart/profile_client.rs"]
mod client;
#[path = "stalwart/submission_client.rs"]
mod submission_client;
use client::Client;
mod utils {
    pub mod jmap {
        pub use crate::submission_client::JmapUtils;
    }
}
mod vacation {
    include!(env!("STALWART_VACATION_SOURCE"));
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
fn check_contains(value: &str, needle: &str, message: impl std::fmt::Display) -> TestOutcome {
    check(
        value.contains(needle),
        format!("{message}: missing {needle:?}"),
    )
}
struct CompCtx<'a> {
    primary: &'a Client,
    outcomes: RefCell<Vec<Value>>,
}
impl CompCtx<'_> {
    async fn run(&self, name: &str, future: impl Future<Output = TestOutcome>) {
        let result = future.await;
        self.outcomes.borrow_mut().push(json!({
            "case": name,
            "passed": result.is_ok(),
            "failure": result.err()
        }));
    }
}

#[tokio::test]
async fn unchanged_upstream_vacation_suite() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(env!("STALWART_VACATION_SOURCE")).unwrap())
        ),
        "49d2e4d49be39a9bb69aa4f7b6ba39e9609319e55a462301e16b33dae06671f8",
        "upstream vacation source must remain pinned and unchanged"
    );
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let client = Client {
        router: mail_protocol::jmap_router(service, "http://localhost".into()),
    };
    let ctx = CompCtx {
        primary: &client,
        outcomes: RefCell::new(vec![]),
    };
    vacation::run(&ctx).await;
    let outcomes = ctx.outcomes.into_inner();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "revision": "474dd0229cb20cf513036619781ed97bd8073c3f",
            "transport": "in-process HTTP router",
            "cases": outcomes
        }))
        .unwrap()
    );
    assert_eq!(outcomes.len(), 10);
    assert!(
        outcomes.iter().all(|case| case["passed"] == true),
        "{outcomes:?}"
    );
}
