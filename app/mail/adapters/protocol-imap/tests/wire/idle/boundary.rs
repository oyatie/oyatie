use super::*;

#[tokio::test]
async fn idle_reports_flags_changed_between_fetch_and_continuation() {
    let (service, db) = super::super::service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: boundary\r\n\r\nbody",
    )
    .unwrap();
    let (mut client, task) = start(service, true, 4096).await;
    let before = command(&mut client, "f FETCH 1 FLAGS").await;
    assert!(!before.contains("\\Seen"), "{before}");
    let account = db.account("a").unwrap();
    db.execute(
        "a",
        account.revision,
        vec![Command::Keywords {
            id: "e1".into(),
            keywords: vec!["$seen".into()],
        }],
    )
    .unwrap();
    client.get_mut().write_all(b"i IDLE\r\n").await.unwrap();
    let transition = until(&mut client, "+").await;
    assert!(
        transition.contains("* 1 FETCH (UID 1 FLAGS (\\Seen))"),
        "{transition}"
    );
    client
        .get_mut()
        .write_all(b"DONE\r\nz LOGOUT\r\n")
        .await
        .unwrap();
    assert!(until(&mut client, "i ").await.contains("i OK"));
    until(&mut client, "z ").await;
    task.await.unwrap().unwrap();
}
