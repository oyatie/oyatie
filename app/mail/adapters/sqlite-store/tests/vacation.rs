use mail_api::{DeliveryQueue, Store, SubmissionQueue};
use mail_kernel::{Account, Command, VacationSettings};
use mail_sqlite_store::SqliteStore;

fn store() -> SqliteStore {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.provision(
        Account::new("b", "t", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    db.execute(
        "a",
        0,
        vec![Command::SetVacation {
            settings: VacationSettings {
                is_enabled: true,
                subject: Some("Off the Florida Keys there's a place called Kokomo".into()),
                text_body: Some("That's where you wanna go to get away from it all".into()),
                ..VacationSettings::default()
            },
        }],
    )
    .unwrap();
    db
}

#[test]
fn enabled_vacation_queues_one_remote_reply_and_skips_repeats_and_lists() {
    let db = store();
    db.deliver(
        &["alice@example.org".into()],
        b"From: bill@remote.org\r\nTo: alice@example.org\r\nSubject: TPS Report\r\n\r\nNeed those TPS reports.",
    )
    .unwrap();
    let leases = db.claim_outbound(10).unwrap();
    assert_eq!(leases.len(), 1);
    let message = db.outbound_message(&leases[0]).unwrap();
    assert_eq!(leases[0].recipient, "bill@remote.org");
    assert_eq!(message.sender, "alice@example.org");
    let raw = String::from_utf8(message.raw).unwrap();
    assert!(raw.contains("Kokomo"));
    assert!(raw.contains("Auto-Submitted: auto-replied"));
    db.deliver(
        &["alice@example.org".into()],
        b"From: bill@remote.org\r\nTo: alice@example.org\r\nSubject: reminder\r\n\r\nStill waiting.",
    )
    .unwrap();
    assert!(db.claim_outbound(10).unwrap().is_empty());
    db.deliver(
        &["alice@example.org".into()],
        b"From: list@remote.org\r\nList-Id: <announce.example.org>\r\n\r\nbulk",
    )
    .unwrap();
    assert!(db.claim_outbound(10).unwrap().is_empty());
}

#[test]
fn vacation_delivers_locally_once_and_ignores_mailer_daemon() {
    let db = store();
    db.deliver(
        &["alice@example.org".into()],
        b"From: bob@example.org\r\nTo: alice@example.org\r\nSubject: hi\r\n\r\nhello",
    )
    .unwrap();
    let leases = db.claim(10).unwrap();
    assert_eq!(leases.len(), 1);
    let queued = db.queued_message(&leases[0]).unwrap();
    db.deliver_once(
        &leases[0].account,
        &leases[0].delivery_id(),
        &queued.raw,
        queued.received_at,
    )
    .unwrap();
    db.finish(&leases[0], Ok(())).unwrap();
    let bob = db.account("b").unwrap();
    assert!(
        bob.messages.iter().any(|m| db
            .blob("b", &m.id)
            .unwrap()
            .windows(6)
            .any(|w| w == b"Kokomo")),
        "{bob:?}"
    );
    let count = bob.messages.len();
    db.deliver(
        &["alice@example.org".into()],
        b"From: bob@example.org\r\nTo: alice@example.org\r\nSubject: again\r\n\r\nhello",
    )
    .unwrap();
    assert_eq!(db.account("b").unwrap().messages.len(), count);
    db.deliver(
        &["alice@example.org".into()],
        b"From: MAILER-DAEMON@remote.org\r\nSubject: bounce\r\n\r\nfailed",
    )
    .unwrap();
    assert!(db.claim_outbound(10).unwrap().is_empty());
}
