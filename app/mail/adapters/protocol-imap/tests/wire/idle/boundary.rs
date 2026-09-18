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
        mail_api::Precondition::Observed(account.revision),
        vec![Command::Keywords {
            id: "e1".into(),
            keywords: vec!["$seen".into()],
        }],
    )
    .unwrap();
    client.get_mut().write_all(b"i IDLE\r\n").await.unwrap();
    let transition = until(&mut client, "+").await;
    assert!(
        transition.contains("* 1 FETCH (FLAGS (\\Seen) UID 1)"),
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

#[tokio::test]
async fn idle_preserves_a_fragmented_done_continuation_across_mailbox_updates() {
    let (service, store) = observed();
    let (mut client, task) = start(service, true, 4096).await;
    idle(&mut client).await;
    client.get_mut().write_all(b"DO").await.unwrap();
    store
        .inner
        .deliver(
            &["alice@example.org".into()],
            b"Subject: fragmented\r\n\r\nbody",
        )
        .unwrap();
    until(&mut client, "* 1 EXISTS").await;
    client
        .get_mut()
        .write_all(b"NE\r\nnext NOOP\r\nz LOGOUT\r\n")
        .await
        .unwrap();
    let result = until(&mut client, "next ").await;
    assert!(
        result.contains("i OK") && result.contains("next OK"),
        "{result}"
    );
    until(&mut client, "z ").await;
    task.await.unwrap().unwrap();
}
