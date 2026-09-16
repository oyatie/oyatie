#![cfg(feature = "upstream-tests")]
// Test-only upstream source remains outside the production workspace.
#[path = "pop/oracle.rs"]
mod oracle;
mod upstream {
    include!(env!("STALWART_POP_SOURCE"));
}
mod utils {
    pub mod imap {
        pub use crate::oracle::AssertResult;
    }
    pub mod pop3 {
        pub use crate::oracle::{Pop3Connection, ResponseType};
    }
    pub mod smtp {
        pub use crate::oracle::SmtpConnection;
    }
    pub mod server {
        pub struct TestServer;
        impl TestServer {
            pub fn account(&self, name: &str) -> &Self {
                assert_eq!(name, self.name());
                self
            }
            pub fn name(&self) -> &str {
                "popper@example.com"
            }
            pub fn secret(&self) -> &str {
                crate::oracle::TOKEN
            }
        }
    }
}

#[tokio::test]
async fn unchanged_stalwart_pop3_assertions() {
    use mail_kernel::Account;
    use mail_service::{MailService, OwnerPolicy};
    use mail_sqlite_store::SqliteStore;
    use sha2::{Digest, Sha256};
    use std::sync::{Arc, Mutex};
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_POP_SOURCE")))
        ),
        "2353449185edc636a4c810826430d243105f7796c82869c8888b8ae5857fb937"
    );
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "popper", "popper@example.com").unwrap(),
        oracle::TOKEN,
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let context = Arc::new(oracle::Context {
        service,
        tasks: Mutex::new(vec![]),
        outcomes: Mutex::new(vec![]),
    });
    oracle::CONTEXT
        .scope(context.clone(), async {
            upstream::test(&utils::server::TestServer).await;
        })
        .await;
    let tasks = std::mem::take(&mut *context.tasks.lock().unwrap());
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    let outcomes = context.outcomes.lock().unwrap();
    println!(
        "{}",
        serde_json::json!({"suite":"tests/src/imap/pop.rs", "sha256":"2353449185edc636a4c810826430d243105f7796c82869c8888b8ae5857fb937", "transport":"duplex SMTP and POP3 wire", "predicates":outcomes.len(), "passed":outcomes.iter().filter(|v| v["passed"] == true).count(), "skipped":0, "outcomes":*outcomes})
    );
}
