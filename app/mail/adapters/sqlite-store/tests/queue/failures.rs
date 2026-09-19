use super::*;

#[test]
fn terminal_local_failures_are_fenced_durable_and_recoverable_without_data_loss() {
    let path = std::env::temp_dir().join(format!("mail-terminal-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.quota_bytes = 1024;
    db.provision(account, &"a".repeat(32)).unwrap();
    let target = DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    };
    let raw = b"Subject: retained\r\n\r\noriginal";
    db.enqueue("", &[target], raw).unwrap();
    db.deliver(&["alice@example.org".into()], &vec![b'x'; 1024])
        .unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute(
        "UPDATE queued_messages SET received_at=unixepoch()-432001",
        [],
    )
    .unwrap();
    let lease = db.claim(1).unwrap().pop().unwrap();
    let message = db.queued_message(&lease).unwrap();
    let outcome = db.deliver_once("a", &lease.delivery_id(), &message.raw, message.received_at);
    assert_eq!(outcome, Err(Error::OverQuota));
    sql.execute_batch("CREATE TRIGGER reject_terminal BEFORE INSERT ON failed_delivery_jobs BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert_eq!(db.finish(&lease, outcome), Err(Error::Conflict));
    assert_eq!(db.queued_message(&lease).unwrap().raw, raw);
    sql.execute_batch("DROP TRIGGER reject_terminal").unwrap();
    db.finish(&lease, outcome).unwrap();
    assert!(db.claim(10).unwrap().is_empty());
    assert_eq!(db.finish(&lease, outcome), Err(Error::Conflict));
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    let failures = db.failed_deliveries("a", 10).unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].message, lease.message);
    assert_eq!(failures[0].reason, "OverQuota");
    assert!(db.failed_deliveries("other", 10).unwrap().is_empty());
    assert_eq!(
        db.retry_failed_delivery("other", &lease.message),
        Err(Error::NotFound)
    );
    assert_eq!(
        db.retry_failed_delivery("a", &lease.message),
        Err(Error::OverQuota)
    );
    let account = db.account("a").unwrap();
    db.execute(
        "a",
        Precondition::Require(account.revision),
        vec![mail_kernel::Command::Destroy {
            id: account.messages[0].id.clone(),
        }],
    )
    .unwrap();
    db.retry_failed_delivery("a", &lease.message).unwrap();
    assert!(db.failed_deliveries("a", 10).unwrap().is_empty());
    let retry = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(retry.message, lease.message);
    assert_ne!(retry.token, lease.token);
    let message = db.queued_message(&retry).unwrap();
    assert_eq!(message.raw, raw);
    db.deliver_once("a", &retry.delivery_id(), &message.raw, message.received_at)
        .unwrap();
    db.finish(&retry, Ok(())).unwrap();
    assert_eq!(db.account("a").unwrap().messages.len(), 1);
    drop(db);
    drop(sql);
    std::fs::remove_file(path).unwrap();
}
