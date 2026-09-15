use mail_api::Events;
use mail_kernel::{Account, Error};
use mail_sqlite::SqliteStore;

#[test]
fn independent_consumers_resume_without_skipping_events_or_rewinding_each_other() {
    let path = std::env::temp_dir().join(format!("mail-consumers-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], b"first").unwrap();
    db.deliver(&["alice@example.org".into()], b"second")
        .unwrap();
    let events = db.pending("foundry", 10).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(
        db.acknowledge("foundry", events[1].sequence),
        Err(Error::Conflict)
    );
    db.acknowledge("foundry", events[0].sequence).unwrap();
    assert_eq!(db.pending("foundry", 10).unwrap(), [events[1].clone()]);
    assert_eq!(db.pending("console", 10).unwrap(), events);
    let other = SqliteStore::open(&path).unwrap();
    other.acknowledge("foundry", events[1].sequence).unwrap();
    db.acknowledge("foundry", events[0].sequence).unwrap();
    assert!(db.pending("foundry", 10).unwrap().is_empty());
    assert_eq!(db.acknowledge("console", u64::MAX), Err(Error::Conflict));
    assert_eq!(db.pending("console", 10).unwrap(), events);
    assert_eq!(db.pending("", 1), Err(Error::Invalid));
    drop((db, other));
    let db = SqliteStore::open(&path).unwrap();
    assert!(db.pending("foundry", 10).unwrap().is_empty());
    assert_eq!(db.pending("console", 10).unwrap(), events);
    drop(db);
    std::fs::remove_file(path).unwrap();
}
