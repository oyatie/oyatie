use super::*;

async fn command(client: &mut BufReader<tokio::io::DuplexStream>, value: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{value}\r\n").as_bytes())
        .await
        .unwrap();
    read(client, value.split_once(' ').unwrap().0).await
}

async fn read(client: &mut BufReader<tokio::io::DuplexStream>, tag: &str) -> String {
    let mut output = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        let done = line.starts_with(&format!("{tag} "));
        output.push_str(&line);
        if done {
            return output;
        }
    }
}

fn id(response: &str, marker: &str) -> String {
    response
        .split_once(marker)
        .unwrap_or_else(|| panic!("{response}"))
        .1
        .split_once(')')
        .unwrap()
        .0
        .into()
}

#[tokio::test]
async fn status_preserves_quoted_mailbox_astrings_with_escaped_quotes() {
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "activate ENABLE OBJECTID+").await;
    let created = command(&mut client, r#"b CREATE "Work \"Queue\"""#).await;
    assert!(created.contains("b OK"), "{created}");
    let status = command(
        &mut client,
        r#"c STATUS "Work \"Queue\"" (MAILBOXID MESSAGES)"#,
    )
    .await;
    assert!(status.contains("c OK"), "{status}");
    assert_eq!(id(&created, "MAILBOXID "), id(&status, "MAILBOXID ("));
    assert!(status.contains(r#"* STATUS "Work \"Queue\"""#), "{status}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn create_with_trailing_hierarchy_delimiter_returns_committed_mailbox_id() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "activate ENABLE OBJECTID+").await;
    let created = command(&mut client, "b CREATE Archive/").await;
    assert!(
        created.contains("b OK [OBJECTID (ACCOUNTID a MAILBOXID "),
        "{created}"
    );
    let identity = id(&created, "MAILBOXID ");
    let status = command(&mut client, "c STATUS Archive (MAILBOXID)").await;
    assert_eq!(id(&status, "MAILBOXID ("), identity);
    let before = db.account("a").unwrap();
    let duplicate = command(&mut client, "d CREATE Archive/").await;
    assert!(duplicate.contains("d NO"), "{duplicate}");
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn malformed_status_does_not_emit_partial_results_or_mutate_selected_state() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: status\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    let before = db.account("a").unwrap();
    for invalid in [
        "MAILBOXID",
        "()",
        "(MAILBOXID UNKNOWN)",
        "((MAILBOXID))",
        "(MAILBOXID)(MESSAGES)",
        "(MAILBOXID) extra",
        "( \"MAILBOXID\" )",
    ] {
        client
            .get_mut()
            .write_all(format!("bad STATUS INBOX {invalid}\r\nnext FETCH 1 (UID)\r\n").as_bytes())
            .await
            .unwrap();
        let result = read(&mut client, "next").await;
        assert!(result.contains("bad BAD"), "{invalid}: {result}");
        assert!(
            !result.contains("* STATUS"),
            "partial invalid STATUS: {result}"
        );
        assert!(result.contains("next OK") && result.contains("UID 1"));
        assert_eq!(db.account("a").unwrap(), before);
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn mailbox_identity_is_stable_across_parent_rename_and_unicode_wire_names() {
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b CREATE Parent").await;
    command(&mut client, "activate ENABLE OBJECTID+").await;
    let created = command(&mut client, "c CREATE Parent/&ZeVnLIqe-").await;
    let identity = id(&created, "MAILBOXID ");
    assert!(
        command(&mut client, "d RENAME Parent Renamed")
            .await
            .contains("d OK")
    );
    let selected = command(&mut client, "e EXAMINE Renamed/&ZeVnLIqe-").await;
    assert_eq!(id(&selected, "MAILBOXID "), identity);
    command(&mut client, "close CLOSE").await;
    assert!(
        command(&mut client, "f ENABLE UTF8=ACCEPT")
            .await
            .contains("f OK")
    );
    let status = command(&mut client, "g STATUS Renamed/日本語 (MAILBOXID MESSAGES)").await;
    assert_eq!(id(&status, "MAILBOXID ("), identity);
    assert!(status.contains("* STATUS \"Renamed/日本語\""), "{status}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn object_id_search_is_case_sensitive_scoped_and_composes_with_other_criteria() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: original\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    let capabilities = command(&mut client, "cap CAPABILITY").await;
    for capability in ["ESEARCH", "SEARCHRES", "OBJECTID+"] {
        assert!(
            capabilities
                .split_ascii_whitespace()
                .any(|item| item == capability),
            "{capabilities}"
        );
    }
    assert!(
        !capabilities
            .split_ascii_whitespace()
            .any(|item| item == "OBJECTID")
    );
    command(&mut client, "b SELECT INBOX").await;
    let fetched = command(&mut client, "c FETCH 1 (EMAILID THREADID)").await;
    let email = id(&fetched, "EMAILID (");
    let thread = id(&fetched, "THREADID (");
    let before = db.account("a").unwrap();
    let result = command(
        &mut client,
        &format!("d UID SEARCH EMAILID {email} THREADID {thread} UNSEEN"),
    )
    .await;
    assert!(result.contains("* SEARCH 1\r\n"), "{result}");
    let result = command(
        &mut client,
        &format!("e SEARCH EMAILID {}", email.to_ascii_uppercase()),
    )
    .await;
    assert!(result.contains("* SEARCH\r\ne OK"), "{result}");
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "f CREATE Archive").await;
    command(&mut client, "g SELECT Archive").await;
    let result = command(
        &mut client,
        &format!("h UID SEARCH OR EMAILID {email} THREADID {thread}"),
    )
    .await;
    assert!(result.contains("* SEARCH\r\nh OK"), "{result}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[path = "objectid_plus.rs"]
mod objectid_plus;

#[path = "objectid_plus_review.rs"]
mod objectid_plus_review;
