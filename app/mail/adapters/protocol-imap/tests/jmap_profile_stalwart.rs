#![cfg(feature = "upstream-tests")]
use mail_api::MetadataStore;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, fmt::Debug, future::Future, sync::Arc};
#[path = "stalwart/profile_client.rs"]
mod client;
use client::Client;
mod identity {
    include!(env!("STALWART_IDENTITY_SOURCE"));
}
mod search_snippet {
    include!(env!("STALWART_SEARCH_SNIPPET_SOURCE"));
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
fn skip(reason: &str) -> TestOutcome {
    Err(format!("unexpected skip: {reason}"))
}
struct CompCtx<'a> {
    primary: &'a Client,
    identity_ids: Vec<String>,
    outcomes: RefCell<Vec<Value>>,
}
impl CompCtx<'_> {
    fn email(&self, name: &str) -> &str {
        match name {
            "plain-simple" => "e1",
            "html-attachment" => "e2",
            "thread-reply-2" => "e3",
            _ => panic!("unknown fixture {name}"),
        }
    }
    fn account_id(&self) -> &str {
        "a"
    }
    async fn run(&self, name: &str, future: impl Future<Output = TestOutcome>) {
        let result = future.await;
        self.outcomes
            .borrow_mut()
            .push(json!({"case":name,"passed":result.is_ok(),"failure":result.err()}));
    }
}
#[tokio::test]
async fn unchanged_upstream_identity_and_snippet_suites() {
    for (path, expected) in [
        (
            env!("STALWART_IDENTITY_SOURCE"),
            "89573c033c868ac02f2dba9ba6db47d243a5f1d6fb079dbb6f9532bf972d487a",
        ),
        (
            env!("STALWART_SEARCH_SNIPPET_SOURCE"),
            "d024790fcfaa911f0a7487b3de42ca6c8b51e5f76cd8c2dbcdf312b22d05e910",
        ),
    ] {
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
            expected,
            "upstream source must be the pinned unchanged file"
        );
    }
    let db = Arc::new(mail_sqlite_store::contract::converted_store(&[]));
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    for raw in [
        "Subject: Meeting tomorrow morning\r\n\r\nLet's meet tomorrow at 9am in the conference room.",
        "Subject: Q3 Financial Report\r\nContent-Type: text/html\r\n\r\n<h1>Q3 Report</h1><p>Please find the report attached.</p>",
        "Subject: Re: Project Alpha Discussion\r\n\r\nThursday works for me. I'll bring the xylophone presentation materials.",
    ] {
        db.deliver(&["alice@example.org".into()], raw.as_bytes())
            .unwrap();
    }
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let client = Client {
        router: mail_protocol_imap::jmap_router(service, "http://localhost".into()),
    };
    let ctx = CompCtx {
        primary: &client,
        identity_ids: vec!["a".into()],
        outcomes: RefCell::new(vec![]),
    };
    identity::run(&ctx).await;
    search_snippet::run(&ctx).await;
    let outcomes = ctx.outcomes.into_inner();
    println!("{}", serde_json::to_string_pretty(&json!({"revision":"474dd0229cb20cf513036619781ed97bd8073c3f","transport":"in-process HTTP router","cases":outcomes})).unwrap());
    assert_eq!(outcomes.len(), 19);
    assert!(
        outcomes.iter().all(|case| case["passed"] == true),
        "{outcomes:?}"
    );
}
