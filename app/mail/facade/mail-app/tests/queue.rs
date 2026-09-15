use mail_api::DeliveryQueue;
use mail_kernel::{Account, Error};
use mail_sqlite::SqliteStore;
use std::process::Command;

#[test]
fn operator_can_inspect_and_retry_retained_mail_without_changing_identity() {
    let binary = option_env!("MAIL_APP_BINARY")
        .or(option_env!("CARGO_BIN_EXE_mail-app"))
        .expect("mail executable");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mail.sqlite");
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.enqueue(
        "",
        &[mail_api::DeliveryTarget {
            account: "a".into(),
            address: "alice@example.org".into(),
        }],
        b"Subject: preserve\r\n\r\nbody",
    )
    .unwrap();
    let original = db.claim(1).unwrap().pop().unwrap();
    db.finish(&original, Err(Error::Invalid)).unwrap();
    let listing = Command::new(binary)
        .arg("failed")
        .arg(&path)
        .arg("a")
        .output()
        .unwrap();
    assert!(listing.status.success());
    assert!(
        String::from_utf8(listing.stdout)
            .unwrap()
            .contains(&original.message)
    );
    assert!(
        !Command::new(binary)
            .arg("retry")
            .arg(&path)
            .arg("other")
            .arg(&original.message)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        Command::new(binary)
            .arg("retry")
            .arg(&path)
            .arg("a")
            .arg(&original.message)
            .output()
            .unwrap()
            .status
            .success()
    );
    let retry = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(retry.message, original.message);
    assert!(db.failed_deliveries("a", 1).unwrap().is_empty());
}
