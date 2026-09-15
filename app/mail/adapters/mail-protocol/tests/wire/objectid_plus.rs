use super::*;

#[tokio::test]
async fn objectid_selection_requires_both_account_and_mailbox_identity() {
    let (service, db) = service();
    db.provision(
        Account::new("foreign", "other-tenant", "bob", "bob@example.org").unwrap(),
        "abcdef0123456789abcdef0123456789",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    let enabled = command(&mut client, "s SELECT INBOX (OBJECTID)").await;
    let mailbox = id(&enabled, "MAILBOXID ");
    let before = db.account("a").unwrap();
    let foreign = db.account("foreign").unwrap();
    for ids in [
        format!("ACCOUNTID foreign MAILBOXID {mailbox}"),
        format!("MAILBOXID {mailbox}"),
        "ACCOUNTID a MAILBOXID invalid".into(),
        "ACCOUNTID foreign MAILBOXID invalid".into(),
    ] {
        let response = command(
            &mut client,
            &format!("bad SELECT nonexistent (OBJECTID ({ids}))"),
        )
        .await;
        assert!(response.contains("bad NO"), "{response}");
        assert!(!response.contains("ACCOUNTID"), "{response}");
        let response = command(
            &mut client,
            &format!("fallback SELECT INBOX (OBJECTID ({ids}))"),
        )
        .await;
        assert!(
            response.contains("fallback OK")
                && response.contains(&format!("ACCOUNTID a MAILBOXID {mailbox}")),
            "{response}"
        );
    }
    let response = command(
        &mut client,
        &format!("ok SELECT nonexistent (OBJECTID (ACCOUNTID a MAILBOXID {mailbox}))"),
    )
    .await;
    assert!(
        response.contains("ok OK") && response.contains(&format!("MAILBOXID {mailbox}")),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap(), before);
    assert_eq!(db.account("foreign").unwrap(), foreign);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn malformed_objectid_requests_preserve_selection_and_do_not_activate() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: retained selection\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "s SELECT INBOX").await;
    let before = db.account("a").unwrap();
    let mut invalid = vec![
        "SELECT INBOX (OBJECTID (MAILBOXID))".to_owned(),
        "SELECT INBOX (OBJECTID (MAILBOXID (nested)))".into(),
        "SELECT INBOX (OBJECTID (MAILBOXID x) UNKNOWN)".into(),
        "SELECT INBOX (OBJECTID) trailing".into(),
        "SELECT INBOX (OBJECTID (MAILBOXID x)".into(),
        "SELECT INBOX (OBJECTID) (CONDSTORE)".into(),
        "SELECT INBOX (OBJECTID QRESYNC (1 0))".into(),
        "STATUS INBOX (OBJECTID UNKNOWN)".into(),
        "FETCH 1 (OBJECTID UNKNOWN)".into(),
    ];
    invalid.push(format!("SELECT INBOX (OBJECTID ({}))", "x x ".repeat(600)));
    for invalid in invalid {
        let response = command(&mut client, &format!("bad {invalid}")).await;
        assert!(
            response.contains("bad BAD")
                && !response.contains("ENABLED")
                && !response.contains("[OBJECTID")
                && !response.contains("* STATUS"),
            "{invalid}: {response}"
        );
        let response = command(&mut client, "f FETCH 1 (UID)").await;
        assert!(
            response.contains("f OK") && response.contains("UID 1"),
            "{response}"
        );
        let response = command(&mut client, "s SELECT INBOX").await;
        assert!(
            !response.contains("OBJECTID") && !response.contains("MAILBOXID"),
            "{response}"
        );
        assert_eq!(db.account("a").unwrap(), before);
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn objectid_activation_coexists_with_condstore_and_qresync_once() {
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "e ENABLE QRESYNC").await;
    let response = command(
        &mut client,
        "s SELECT INBOX (OBJECTID QRESYNC (1 0) CONDSTORE)",
    )
    .await;
    assert!(
        response.contains("s OK")
            && response.contains("HIGHESTMODSEQ")
            && response.contains("[OBJECTID (ACCOUNTID a MAILBOXID "),
        "{response}"
    );
    assert_eq!(
        response.matches("* ENABLED OBJECTID+").count(),
        1,
        "{response}"
    );
    for value in [
        "s SELECT INBOX (OBJECTID)",
        "s STATUS INBOX (OBJECTID)",
        "s UID FETCH 1 (OBJECTID)",
    ] {
        let response = command(&mut client, value).await;
        assert!(
            response.contains("s OK") && !response.contains("ENABLED"),
            "{response}"
        );
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn revoked_identity_cannot_activate_or_read_compound_identifiers() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "s SELECT INBOX").await;
    db.revoke("a").unwrap();
    let before = db.account("a").unwrap();
    for value in [
        "ENABLE OBJECTID+",
        "SELECT INBOX (OBJECTID)",
        "STATUS INBOX (OBJECTID)",
        "FETCH 1 (OBJECTID)",
        "CREATE private",
        "RENAME INBOX private",
    ] {
        let response = command(&mut client, &format!("no {value}")).await;
        assert!(
            response.contains("no NO")
                && !response.contains("ENABLED")
                && !response.contains("ACCOUNTID")
                && !response.contains("MAILBOXID"),
            "{response}"
        );
        assert_eq!(db.account("a").unwrap(), before);
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn valid_objectid_status_activates_even_when_mailbox_name_is_missing() {
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    for value in [
        "c CREATE Archive",
        "r RENAME Archive Saved",
        "s SELECT Saved",
    ] {
        let response = command(&mut client, value).await;
        assert!(
            !response.contains("OBJECTID") && !response.contains("MAILBOXID"),
            "{response}"
        );
    }
    let response = command(&mut client, "missing STATUS nonexistent (OBJECTID)").await;
    assert!(
        response.contains("missing NO") && response.contains("* ENABLED OBJECTID+"),
        "{response}"
    );
    assert!(!response.contains("MAILBOXID"), "{response}");
    let response = command(&mut client, "s SELECT Saved").await;
    assert!(
        response.contains("[OBJECTID (") && !response.contains("ENABLED"),
        "{response}"
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn objectid_keys_ignore_case_values_preserve_case_and_last_key_wins() {
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    let response = command(&mut client, "s SELECT INBOX (OBJECTID)").await;
    let mailbox = id(&response, "MAILBOXID ");
    let response = command(&mut client, &format!("s SELECT nonexistent (OBJECTID (ACCOUNTID foreign accountid a MAILBOXID bad mailboxid \"{mailbox}\" unknown \"ignored value\"))")).await;
    assert!(
        response.contains("s OK") && response.contains(&format!("MAILBOXID {mailbox}")),
        "{response}"
    );
    let response = command(
        &mut client,
        &format!(
            "s SELECT nonexistent (OBJECTID (ACCOUNTID a MAILBOXID {}))",
            mailbox.to_uppercase()
        ),
    )
    .await;
    assert!(response.contains("s NO"), "{response}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
