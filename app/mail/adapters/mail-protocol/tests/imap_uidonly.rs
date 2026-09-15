// Only the test harness includes the unchanged pinned upstream oracle.
extern crate self as imap_proto;
pub use client::{AssertResult, ImapConnection, ResponseType, Type};
#[path = "stalwart/uidonly_client.rs"]
mod client;
#[path = "stalwart/uidonly_hardening.rs"]
mod hardening;
mod upstream {
    include!(env!("STALWART_IMAP_UIDONLY_SOURCE"));
}
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
mod utils {
    pub mod server {
        pub struct TestServer;
        impl TestServer {
            pub fn account(&self, name: &str) -> &Self {
                assert_eq!(name, "jdoe@example.com");
                self
            }
            pub fn name(&self) -> &str {
                "alice@example.org"
            }
            pub fn secret(&self) -> &str {
                crate::TOKEN
            }
        }
    }
}
fn fixture() -> std::sync::Arc<mail_service::MailService> {
    use mail_api::Store;
    use mail_kernel::{Account, Command};
    use mail_service::{MailService, OwnerPolicy};
    use mail_sqlite::SqliteStore;
    use std::sync::Arc;
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
            name: "Deleted Items".into(),
        }],
    )
    .unwrap();
    for body in [
        b"Subject: first\r\n\r\none".as_slice(),
        b"Subject: second\r\n\r\ntwo",
    ] {
        db.deliver(&["alice@example.org".into()], body).unwrap();
    }
    Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    })
}
#[tokio::test]
async fn upstream_imap_uidonly() {
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_IMAP_UIDONLY_SOURCE")))
        ),
        "862d887dfeff1c416a6a8c3b2f4b4ff8b703765e90a288e3de42d29203b491b9"
    );
    let outcomes = client::initialize(fixture());
    upstream::test(&utils::server::TestServer).await;
    let outcomes = outcomes.borrow();
    let passed = outcomes.iter().filter(|o| o["passed"] == true).count();
    println!(
        "{}",
        serde_json::json!({"suite":"tests/src/imap/uidonly.rs", "upstream_revision":"474dd0229cb20cf513036619781ed97bd8073c3f", "passed":passed, "wire_predicates":outcomes.len(), "skipped":0, "outcomes":*outcomes})
    );
    assert_eq!(
        passed,
        outcomes.len(),
        "unchanged UIDONLY oracle predicates failed"
    );
}
#[path = "stalwart/uidonly_review.rs"]
mod review;
