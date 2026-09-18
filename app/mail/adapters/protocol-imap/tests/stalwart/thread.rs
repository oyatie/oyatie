use super::{TOKEN, client::*};
use crate::imap::append;
use mail_api::MetadataStore;
use mail_kernel::{Account, Command};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use sha2::{Digest, Sha256};
use std::{cell::RefCell, rc::Rc, sync::Arc};
mod upstream {
    include!(env!("STALWART_IMAP_THREAD_SOURCE"));
}
#[tokio::test]
async fn upstream_imap_thread() {
    for (source, expected) in [
        (
            include_bytes!(env!("STALWART_IMAP_THREAD_SOURCE")).as_slice(),
            "b6eae4f76eba52121653c3e0129dbe85932e821b87e3c7e5b2f2b5832e26e9a9",
        ),
        (
            include_bytes!(env!("STALWART_IMAP_APPEND_SOURCE")).as_slice(),
            "bf0424ed6393f3af4d7a064425a0ae0e7d7655ac44f89e3cea5dfc2934afa7cf",
        ),
    ] {
        assert_eq!(format!("{:x}", Sha256::digest(source)), expected);
    }
    let service = service();
    let outcomes = Rc::new(RefCell::new(vec![]));
    let mut primary = ImapConnection::connect(service.clone(), outcomes.clone()).await;
    let mut check = ImapConnection::connect(service, outcomes.clone()).await;
    let server = crate::utils::server::TestServer {
        server: crate::utils::server::Server,
    };
    upstream::test(&mut primary, &mut check, &server).await;
    primary.close().await;
    check.close().await;
    let outcomes = outcomes.borrow();
    let passed = outcomes.iter().filter(|v| v["passed"] == true).count();
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"upstream_revision":"474dd0229cb20cf513036619781ed97bd8073c3f","suite":"tests/src/imap/thread.rs","transport":"duplex IMAP wire","wire_predicates":outcomes.len(),"passed":passed,"skipped":0,"outcomes":*outcomes})).unwrap());
    assert_eq!(
        passed,
        outcomes.len(),
        "unchanged upstream THREAD predicates failed"
    );
}

fn service() -> Arc<MailService> {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(0),
        vec![Command::CreateMailbox {
            name: "Deleted Items".into(),
        }],
    )
    .unwrap();
    Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    })
}

#[tokio::test]
async fn upstream_imap_append() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_IMAP_APPEND_SOURCE")))
        ),
        "bf0424ed6393f3af4d7a064425a0ae0e7d7655ac44f89e3cea5dfc2934afa7cf"
    );
    let service = service();
    let outcomes = Rc::new(RefCell::new(vec![]));
    let mut primary = ImapConnection::connect(service.clone(), outcomes.clone()).await;
    let mut check = ImapConnection::connect(service, outcomes.clone()).await;
    let server = crate::utils::server::TestServer {
        server: crate::utils::server::Server,
    };
    append::test(&mut primary, &mut check, &server).await;
    primary.close().await;
    check.close().await;
    let outcomes = outcomes.borrow();
    let passed = outcomes.iter().filter(|v| v["passed"] == true).count();
    println!(
        "{}",
        serde_json::json!({"suite":"tests/src/imap/append.rs", "passed":passed, "wire_predicates":outcomes.len(), "skipped":0})
    );
    assert_eq!(
        passed,
        outcomes.len(),
        "unchanged upstream APPEND predicates failed"
    );
}
