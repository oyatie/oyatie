use super::*;

#[tokio::test]
async fn text_search_handles_repeated_headers_with_bounded_work() {
    let (service, db) = service();
    let mut raw = "X-Trace: repeated header value that cannot match\r\n".repeat(3000);
    raw.push_str("X-Trace: final needle\r\n\r\nbody");
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    for (query, expected) in [
        ("TEXT absent", vec![]),
        ("TEXT needle", vec![1]),
        ("HEADER x-trace needle", vec![1]),
    ] {
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            search(&mut client, query, &expected),
        )
        .await
        .expect("one small message must not trigger quadratic repeated header parsing");
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn saved_esearch_extrema_follow_oracle_even_with_count_or_all() {
    let (mut client, task, _) = session().await;
    for (options, expected) in [
        ("MIN", vec![1]),
        ("MAX", vec![3]),
        ("MIN MAX", vec![1, 3]),
        ("MIN COUNT", vec![1]),
        ("MAX ALL", vec![3]),
        ("MIN MAX ALL COUNT", vec![1, 3]),
    ] {
        let result = command(
            &mut client,
            &format!("c UID SEARCH RETURN (SAVE {options}) ALL"),
        )
        .await;
        assert!(
            result.contains("c OK") && result.contains("* ESEARCH (TAG \"c\") UID"),
            "{result}"
        );
        if options.contains("MIN") {
            assert!(result.contains("MIN 1"), "{result}");
        }
        if options.contains("MAX") {
            assert!(result.contains("MAX 3"), "{result}");
        }
        if options.contains("COUNT") {
            assert!(result.contains("COUNT 3"), "{result}");
        }
        let fetched = command(&mut client, "d UID FETCH $ (UID)").await;
        for uid in 1..=3 {
            assert_eq!(
                fetched.contains(&format!("FETCH (UID {uid})")),
                expected.contains(&uid),
                "{options}: {fetched}"
            );
        }
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn saved_results_do_not_resurrect_messages_reentering_under_a_new_uid() {
    let (mut client, task, db) = session().await;
    assert!(
        command(&mut client, "c UID SEARCH RETURN (SAVE) UID 2:3")
            .await
            .contains("c OK")
    );
    let created = db
        .execute(
            "a",
            db.account("a").unwrap().revision,
            vec![Command::CreateMailbox {
                name: "Archive".into(),
            }],
        )
        .unwrap();
    let archive = created
        .mailboxes
        .iter()
        .find(|m| m.name == "Archive")
        .unwrap()
        .id
        .clone();
    let moved = db
        .execute(
            "a",
            created.revision,
            vec![Command::SetMailboxes {
                id: "e2".into(),
                mailboxes: vec![archive],
            }],
        )
        .unwrap();
    db.execute(
        "a",
        moved.revision,
        vec![Command::SetMailboxes {
            id: "e2".into(),
            mailboxes: vec!["inbox".into()],
        }],
    )
    .unwrap();
    command(&mut client, "d NOOP").await;
    let fetched = command(&mut client, "e UID FETCH $ (UID)").await;
    assert!(
        fetched.contains("UID 3") && !fetched.contains("UID 4"),
        "{fetched}"
    );
    let search = command(&mut client, "f UID SEARCH $").await;
    assert!(search.contains("* SEARCH 3\r\n"), "{search}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn search_rejects_excessive_grammar_without_losing_saved_state_or_session() {
    let (mut client, task, db) = session().await;
    assert!(
        command(&mut client, "c SEARCH RETURN (SAVE) SEEN")
            .await
            .contains("c OK")
    );
    let before = db.account("a").unwrap();
    for criteria in [
        format!("{}ALL", "NOT ".repeat(500)),
        "ALL ".repeat(1100),
        "()".into(),
        "\"ALL\"".into(),
    ] {
        client
            .get_mut()
            .write_all(format!("bad SEARCH {criteria}\r\nnext UID FETCH $ (UID)\r\n").as_bytes())
            .await
            .unwrap();
        let response =
            tokio::time::timeout(std::time::Duration::from_secs(5), read(&mut client, "next"))
                .await
                .unwrap();
        assert!(
            response.contains("bad BAD") && response.contains("next OK"),
            "{response}"
        );
        assert!(
            response.contains("UID 1") && !response.contains("UID 2"),
            "{response}"
        );
        assert_eq!(db.account("a").unwrap(), before);
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn oracle_not_groups_keep_nested_operator_boundaries() {
    let (mut client, task, db) = session().await;
    let before = db.account("a").unwrap();
    for (query, expected) in [
        ("NOT (FROM alice ANSWERED)", vec![3]),
        ("NOT NOT (FROM alice ANSWERED)", vec![1, 2]),
        ("NOT ((FROM alice) (ANSWERED))", vec![3]),
        ("NOT (OR FROM alice FROM bob)", vec![3]),
        ("NOT (FROM alice (OR ANSWERED DELETED))", vec![]),
        ("OR (FROM alice ANSWERED) DELETED", vec![3]),
    ] {
        search(&mut client, query, &expected).await;
    }
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn oracle_extrema_preserve_singleton_duplicates_and_clear_empty_saved_results() {
    let (mut client, task, _) = session().await;
    for operation in [
        "UID SEARCH RETURN (ALL COUNT MIN MAX SAVE)",
        "UID SORT RETURN (ALL COUNT MIN MAX SAVE) (REVERSE SUBJECT) UTF-8",
    ] {
        let single = command(&mut client, &format!("s {operation} UID 2")).await;
        assert!(
            single.contains("COUNT 1 MIN 2 MAX 2 ALL 2,2") && single.contains("s OK"),
            "{single}"
        );
        search(&mut client, "$", &[2]).await;
        let empty = command(&mut client, &format!("s {operation} UID 0")).await;
        assert!(
            empty.contains("COUNT 0") && empty.contains("s OK"),
            "{empty}"
        );
        assert!(
            !empty.contains(" MIN ") && !empty.contains(" MAX ") && !empty.contains(" ALL "),
            "{empty}"
        );
        search(&mut client, "$", &[]).await;
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
