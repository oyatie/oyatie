use super::*;

#[tokio::test]
async fn sort_invalid_numeric_timezone_uses_utc_instead_of_wrapping_its_digits() {
    for (time, zone) in [
        ("00:00:30", "+2500"),
        ("00:00:30", "-2500"),
        ("00:00:30", "+0061"),
        ("00:00:30", "-0061"),
        ("00:00:30", "+2500 (outer (+0000) comment)"),
        ("00:00:30", "-2500 (outer \\) +0000)"),
        ("00:00:30", "+2500\r\n\t(continued +0000)"),
        ("01:00:30", "+0100 (outer (-2500))"),
    ] {
        let (service, db) = service();
        for date in [
            format!("1 Jan 2024 {time} {zone}"),
            "1 Jan 2024 00:00:00 +0000".into(),
            "1 Jan 2024 00:01:00 +0000".into(),
        ] {
            db.deliver(
                &["alice@example.org".into()],
                format!("Date: {date}\r\nSubject: invalid timezone\r\n\r\nbody").as_bytes(),
            )
            .unwrap();
        }
        let (client, server) = tokio::io::duplex(4096);
        let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
        let mut client = BufReader::new(client);
        command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
        command(&mut client, "b SELECT INBOX").await;
        sorted(&mut client, "SORT (DATE) UTF-8 ALL", &[2, 1, 3]).await;
        sorted(&mut client, "SORT (REVERSE DATE) UTF-8 ALL", &[3, 1, 2]).await;
        command(&mut client, "z LOGOUT").await;
        task.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn extended_search_and_sort_refuse_quoted_return_option_atoms() {
    let (mut client, task, _) = session().await;
    for request in [
        "s SORT RETURN (\"MIN\") (SUBJECT) UTF-8 ALL",
        "s SEARCH RETURN (\"MIN\") ALL",
    ] {
        let result = command(&mut client, request).await;
        assert!(result.contains("s BAD"), "{request}: {result}");
        assert!(!result.contains("* ESEARCH"), "{result}");
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn oracle_missing_sort_fields_follow_indexed_values_in_both_directions() {
    let (service, db) = service();
    for raw in [
        "Subject: Bravo\r\nDate: 2 Jan 2024 00:00:00 +0000\r\nFrom: bob@example.org\r\nTo: bob@example.org\r\nCc: bob@example.org\r\n\r\nbody",
        "X-Only: absent fields\r\n\r\nbody",
        "Subject: Alpha\r\nDate: 1 Jan 2024 00:00:00 +0000\r\nFrom: alice@example.org\r\nTo: alice@example.org\r\nCc: alice@example.org\r\n\r\nbody",
    ] {
        db.deliver(&["alice@example.org".into()], raw.as_bytes())
            .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    for field in ["SUBJECT", "DATE", "FROM", "TO", "CC"] {
        sorted(
            &mut client,
            &format!("SORT ({field}) UTF-8 ALL"),
            &[3, 1, 2],
        )
        .await;
        sorted(
            &mut client,
            &format!("SORT (REVERSE {field}) UTF-8 ALL"),
            &[1, 3, 2],
        )
        .await;
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn oracle_missing_and_indexed_empty_subject_tie_in_descending_order() {
    let (service, db) = service();
    for (header, received_at) in [
        ("X-Only: missing subject", 10),
        ("Subject: Re:", 20),
        ("Subject: Alpha", 30),
    ] {
        db.execute(
            "a",
            db.account("a").unwrap().revision,
            vec![Command::Append {
                mailboxes: vec!["inbox".into()],
                received_at,
                raw: format!("{header}\r\n\r\nbody").into_bytes(),
                keywords: vec![],
            }],
        )
        .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    sorted(&mut client, "SORT (SUBJECT) UTF-8 ALL", &[2, 3, 1]).await;
    sorted(&mut client, "SORT (REVERSE SUBJECT) UTF-8 ALL", &[3, 1, 2]).await;
    sorted(
        &mut client,
        "SORT (REVERSE SUBJECT ARRIVAL) UTF-8 ALL",
        &[3, 1, 2],
    )
    .await;
    sorted(
        &mut client,
        "SORT (REVERSE SUBJECT REVERSE ARRIVAL) UTF-8 ALL",
        &[3, 2, 1],
    )
    .await;
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
