use super::*;

#[tokio::test]
async fn oracle_zero_sequence_bounds_match_existing_messages_without_allocating_uid_zero() {
    let (mut client, task, db) = session().await;
    let before = db.account("a").unwrap();
    for (criteria, expected) in [
        ("UID 0", vec![]),
        ("UID 0:2", vec![1, 2]),
        ("UID 2:0", vec![1, 2]),
        ("UID 0:*", vec![1, 2, 3]),
        ("0:2", vec![1, 2]),
        ("0", vec![]),
        ("0:0", vec![]),
        ("00:0002", vec![1, 2]),
    ] {
        search(&mut client, criteria, &expected).await;
    }
    for set in ["+1", "1:+2", "-1", "4294967296", "0::2"] {
        let result = command(&mut client, &format!("bad UID SEARCH UID {set}")).await;
        assert!(result.contains("bad BAD"), "{result}");
    }
    for operation in [
        "FETCH 0 (UID)",
        "UID FETCH 0 (UID)",
        "STORE 0 +FLAGS (\\Deleted)",
        "UID STORE 0 +FLAGS (\\Deleted)",
        "UID EXPUNGE 0",
    ] {
        let response = command(&mut client, &format!("zero {operation}")).await;
        assert!(response.contains("zero OK"), "{operation}: {response}");
        assert!(
            !response.contains("* 1 FETCH") && !response.contains("* 1 EXPUNGE"),
            "{response}"
        );
    }
    let after = db.account("a").unwrap();
    assert_eq!(after.messages, before.messages);
    assert_eq!(after.mailboxes, before.mailboxes);
    db.execute(
        "a",
        after.revision,
        vec![Command::Destroy { id: "e1".into() }],
    )
    .unwrap();
    search(&mut client, "0:1", &[]).await;
    search(&mut client, "0:2", &[2]).await;
    let fetched = command(&mut client, "f FETCH 0:1 (UID)").await;
    assert!(
        fetched.contains("f OK") && !fetched.contains("* 1 FETCH"),
        "{fetched}"
    );
    let noop = command(&mut client, "n NOOP").await;
    assert!(noop.contains("* 1 EXPUNGE"), "{noop}");
    search(&mut client, "0:1", &[1]).await;
    search(&mut client, "UID 0:1", &[]).await;
    let after = db.account("a").unwrap();
    assert!(after.mailboxes.iter().all(|m| m.uid_next > 0));
    assert!(
        after
            .messages
            .iter()
            .all(|m| m.mailboxes.values().all(|uid| *uid > 0))
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
