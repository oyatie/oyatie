//! This process's node lease: taken before any listener binds (`take`) and
//! renewed at a third of its
//! TTL, carrying the binary's schema version. A no-op on the SQLite tier;
//! the hosted tier's schema advance waits for every live lease to agree.
use mail_api::{NODE_LEASE_SECS, NodeLease};
use mail_sqlite_store::SqliteStore;
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(super) fn node() -> String {
    format!("{}:{}", super::convert::operator(), std::process::id())
}

pub(super) fn take(store: &SqliteStore) -> Result<(), mail_kernel::Error> {
    store.heartbeat(&node(), mail_sqlite_store::SCHEMA_VERSION, NODE_LEASE_SECS)
}

pub(super) async fn run(store: Arc<SqliteStore>, mut stop: oneshot::Receiver<()>) {
    let node = node();
    loop {
        let (store, node) = (store.clone(), node.clone());
        let result = tokio::task::spawn_blocking(move || {
            store.heartbeat(&node, mail_sqlite_store::SCHEMA_VERSION, NODE_LEASE_SECS)
        })
        .await;
        if !matches!(result, Ok(Ok(()))) {
            eprintln!("mail-app: node lease renewal failed: {result:?}");
        }
        let period = Duration::from_secs((NODE_LEASE_SECS / 3) as u64);
        tokio::select! { _ = &mut stop => return, _ = tokio::time::sleep(period) => {} }
    }
}
