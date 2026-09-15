use super::*;

fn id(value: &str, marker: &str) -> String {
    value
        .split_once(marker)
        .unwrap_or_else(|| panic!("{value}"))
        .1
        .split_once(')')
        .unwrap()
        .0
        .to_owned()
}

#[tokio::test]
async fn mailbox_ids_survive_rename_match_status_and_change_after_recreation() {
    let (service, _) = service();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    client.write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nactivate ENABLE OBJECTID+\r\nb CREATE Archive\r\nc STATUS Archive (MAILBOXID MESSAGES UNSEEN UIDNEXT UIDVALIDITY RECENT)\r\nd RENAME Archive Saved\r\ne SELECT Saved\r\nf CLOSE\r\ng DELETE Saved\r\nh CREATE Saved\r\ni STATUS Saved (MAILBOXID)\r\nz LOGOUT\r\n").as_bytes()).await.unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    for tag in ["b", "c", "d", "e", "f", "g", "h", "i"] {
        assert!(result.contains(&format!("{tag} OK")), "{result}");
    }
    let first = id(&result, "b OK [OBJECTID (ACCOUNTID a MAILBOXID ");
    let status = id(&result, "* STATUS \"Archive\" (MAILBOXID (");
    let selected = id(&result, "* OK [OBJECTID (ACCOUNTID a MAILBOXID ");
    let recreated = id(&result, "h OK [OBJECTID (ACCOUNTID a MAILBOXID ");
    assert_eq!(first, status);
    assert_eq!(first, selected);
    assert_ne!(first, recreated);
    assert_eq!(recreated, id(&result, "* STATUS \"Saved\" (MAILBOXID ("));
    assert!(result.contains("MESSAGES 0 UNSEEN 0 UIDNEXT 1"), "{result}");
}

#[tokio::test]
async fn object_id_search_matches_current_mailbox_and_rejects_bad_ids() {
    let (service, db) = service();
    db.deliver(&["alice@example.org".into()], b"Subject: item\r\n\r\nbody")
        .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    client.get_mut().write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\nc FETCH 1 (EMAILID THREADID)\r\n").as_bytes()).await.unwrap();
    let mut result = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        result.push_str(&line);
        if line.starts_with("c ") {
            break;
        }
    }
    let email = id(&result, "EMAILID (");
    let thread = id(&result, "THREADID (");
    client.get_mut().write_all(format!("d UID SEARCH EMAILID {email}\r\ne SEARCH THREADID {thread}\r\nf SEARCH EMAILID {thread}\r\ng SEARCH THREADID \"invalid!\"\r\nz LOGOUT\r\n").as_bytes()).await.unwrap();
    result.clear();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(result.matches("* SEARCH 1\r\n").count(), 2, "{result}");
    assert!(result.contains("* SEARCH\r\nf OK"), "{result}");
    assert!(result.contains("g BAD"), "{result}");
}
