#![cfg(feature = "upstream-tests")]
// Upstream assertions are included unchanged from the pinned test-only archive.
extern crate self as imap_proto;
pub use client::{AssertResult, ImapConnection, ResponseType, Type};
#[path = "stalwart/condstore_client.rs"]
mod client;
mod upstream {
    include!(env!("STALWART_IMAP_OBJECTID_SOURCE"));
}
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
mod utils {
    pub mod server {
        pub struct TestServer {
            pub service: std::sync::Arc<mail_service::MailService>,
            pub outcomes: crate::client::Outcomes,
        }
        impl TestServer {
            pub fn account(&self, name: &str) -> &Self {
                assert_eq!(name, "jdoe@example.com");
                self
            }
            pub fn id_string(&self) -> &str {
                "a"
            }
            pub async fn imap_client(&self) -> crate::ImapConnection {
                crate::ImapConnection::connect(self.service.clone(), self.outcomes.clone()).await
            }
        }
    }
}
#[tokio::test]
async fn upstream_imap_objectid() {
    use mail_api::MetadataStore;
    use mail_kernel::Account;
    use mail_service::{MailService, OwnerPolicy};
    use sha2::{Digest, Sha256};
    use std::{cell::RefCell, rc::Rc, sync::Arc};
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_IMAP_OBJECTID_SOURCE")))
        ),
        "719c6e9d41550220b16e718b43327cb66bdec95183abb9c1e4ca235fd53ec8d1"
    );
    let db = Arc::new(mail_sqlite_store::contract::converted_store(&[]));
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: object identity\r\n\r\nbody",
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let outcomes = Rc::new(RefCell::new(vec![]));
    let server = utils::server::TestServer {
        service,
        outcomes: outcomes.clone(),
    };
    upstream::test(&server).await;
    let outcomes = outcomes.borrow();
    let passed = outcomes.iter().filter(|v| v["passed"] == true).count();
    println!(
        "{}",
        serde_json::json!({"upstream_revision":"474dd0229cb20cf513036619781ed97bd8073c3f", "suite":"tests/src/imap/objectid.rs", "transport":"duplex IMAP wire", "wire_predicates":outcomes.len(), "passed":passed, "skipped":0, "outcomes":*outcomes})
    );
    assert_eq!(
        passed,
        outcomes.len(),
        "unchanged upstream OBJECTID+ predicates failed"
    );
}
