//! Hourly history compaction under the default retention policy, bounded by
//! the enabled consumers' cursors; feed anomalies are logged each pass.
use mail_api::{ChangeFeed, FeedRead, MetadataStore};
use mail_kernel::{Retention, RetentionPolicy};
use mail_sqlite_store::SqliteStore;
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(super) async fn run(store: Arc<SqliteStore>, mut stop: oneshot::Receiver<()>) {
    loop {
        let db = store.clone();
        if let Err(error) = tokio::task::spawn_blocking(move || pass(&db)).await {
            eprintln!("mail-app: compaction unavailable: {error:?}");
        }
        tokio::select! { _ = &mut stop => return, _ = tokio::time::sleep(Duration::from_secs(3600)) => {} }
    }
}

fn pass(store: &SqliteStore) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let enabled = store.consumers();
    let accounts = match store.account_ids() {
        Ok(accounts) => accounts,
        Err(error) => return eprintln!("mail-app: compaction unavailable: {error:?}"),
    };
    for account in &accounts {
        let result = store.cursors(account, enabled).and_then(|cursors| {
            store.compact_history(account, now, RetentionPolicy::default(), &cursors)
        });
        match result {
            Ok(Retention::Blocked {
                floor,
                wanted,
                consumer,
            }) => eprintln!(
                "mail-app: event=retention-blocked account={account} consumer={consumer} floor={floor} wanted={wanted}"
            ),
            Ok(Retention::Advanced { .. }) => {}
            Err(error) => eprintln!("mail-app: compaction failed account={account}: {error:?}"),
        }
        for consumer in enabled {
            if let Ok(FeedRead::BelowFloor { cursor, floor }) = store.changes(*consumer, account, 0)
            {
                eprintln!(
                    "mail-app: event=cursor-below-floor consumer={} account={account} cursor={} floor={floor}",
                    consumer.name(),
                    cursor.0
                );
            }
        }
    }
    for consumer in enabled {
        for (dirty, reason) in store.poisoned(*consumer).unwrap_or_default() {
            eprintln!(
                "mail-app: event=poison consumer={} tenant={} account={} reason={reason:?}",
                consumer.name(),
                dirty.tenant,
                dirty.account
            );
        }
    }
}
