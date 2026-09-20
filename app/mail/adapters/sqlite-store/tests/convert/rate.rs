//! Conversion throughput on a fixture large enough to be meaningful: ten
//! accounts, a thousand messages, every body parsed once for threading.
use super::{OPERATOR, Temp};
use mail_api::MetadataStore;
use mail_sqlite_store::SqliteStore;
use mail_sqlite_store::contract::legacy::{LegacyAccountSpec, LegacyFixture, Shape};
use mail_sqlite_store::convert::Converter;

#[test]
fn convert_rate_over_ten_accounts_and_a_thousand_messages() {
    let temp = Temp::new("rate");
    let database = temp.path("mail.sqlite");
    let backup = temp.path("backup.sqlite");
    // Five accounts in each shape; none thread-indexed, so every body is
    // parsed and threaded during conversion.
    let specs: Vec<LegacyAccountSpec> = (0..10)
        .map(|n| {
            let shape = if n % 2 == 0 {
                Shape::Oldest
            } else {
                Shape::Newer
            };
            LegacyAccountSpec::generated(&format!("acct{n}"), shape, 100)
        })
        .collect();
    let fixture = LegacyFixture::create(&database, &specs).unwrap();
    assert_eq!(fixture.messages, 1000);
    let converter = Converter::open(&database).unwrap();
    converter.backup_into(&backup).unwrap();
    let started = std::time::Instant::now();
    let conversion = converter.convert(&backup, OPERATOR).unwrap();
    let elapsed = started.elapsed();
    let accounts = conversion.accounts;
    let messages = fixture.messages;
    println!(
        "convert rate: {accounts} accounts, {messages} messages in {elapsed:?} = {:.1} accounts/s",
        accounts as f64 / elapsed.as_secs_f64()
    );
    assert_eq!(accounts, 10);
    assert!(conversion.elapsed <= elapsed);
    let store = SqliteStore::open(&database).unwrap();
    for spec in &specs {
        let account = store.account(&spec.id).unwrap();
        assert_eq!(account.messages.len(), 100);
        // Legacy revision 101 (100 appends + provisioning) plus the conversion commit.
        assert_eq!(account.revision, 102);
        assert_eq!(account.history_floor, 102);
        assert_eq!(account.mailboxes[0].total_emails, 100);
        assert_eq!(account.mailboxes[0].unread_emails, 50);
        // Threads of five, backfilled from the bodies.
        let threads: std::collections::BTreeSet<_> =
            account.messages.iter().map(|m| m.thread_id()).collect();
        assert_eq!(threads.len(), 20, "{}", spec.id);
        assert_eq!(
            account
                .messages
                .iter()
                .find(|m| m.id == "e99")
                .unwrap()
                .thread_id(),
            "e96"
        );
    }
    assert_eq!(
        store.mailbox_uids("acct3", "inbox").unwrap().uids.len(),
        100
    );
}
