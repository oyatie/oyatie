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
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
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
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
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

async fn script(script: &str) -> String {
    let (service, _) = service();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    client
        .write_all(format!("a LOGIN alice@example.org {TOKEN}\r\n{script}z LOGOUT\r\n").as_bytes())
        .await
        .unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    result
}

#[tokio::test]
async fn id_answers_with_the_server_name_and_accepts_a_client_id() {
    let result = script("b ID NIL\r\nc ID (\"name\" \"client\" \"version\" \"1\")\r\n").await;
    assert!(
        result.contains("* ID (\"name\" \"Oyatie mail\")"),
        "{result}"
    );
    assert!(
        result.contains("b OK") && result.contains("c OK"),
        "{result}"
    );
}

#[tokio::test]
async fn list_return_options_report_subscribed_children_special_use_and_status() {
    let result = script(
        "b CREATE Parent/Child\r\nc SUBSCRIBE Parent\r\nd CREATE Drafts (USE (\\Drafts))\r\ne LIST \"\" \"*\" RETURN (SUBSCRIBED CHILDREN SPECIAL-USE STATUS (MESSAGES))\r\nf LIST (SPECIAL-USE) \"\" \"*\"\r\n",
    )
    .await;
    for tag in ["b", "c", "d", "e", "f"] {
        assert!(result.contains(&format!("{tag} OK")), "{result}");
    }
    assert!(
        result.contains("\\HasChildren") && result.contains("\\Subscribed"),
        "{result}"
    );
    assert!(
        result.contains("\\HasNoChildren) \"/\" \"Parent/Child\""),
        "{result}"
    );
    assert!(result.contains("\\Drafts) \"/\" \"Drafts\""), "{result}");
    assert!(
        result.contains("* STATUS \"Parent/Child\" (MESSAGES 0)"),
        "{result}"
    );
    let after_f = result.split("e OK").nth(1).unwrap();
    assert!(
        after_f.contains("\"Drafts\"") && !after_f.contains("\"Parent\""),
        "{result}"
    );
}

#[tokio::test]
async fn create_special_use_assigns_a_role_once_and_refuses_a_second_holder() {
    let result = script(
        "b CREATE Archive (USE (\\Archive))\r\nc CREATE Other (USE (\\Archive))\r\nd CREATE Bad (USE (\\Nope))\r\ne LIST \"\" Archive\r\n",
    )
    .await;
    assert!(result.contains("b OK"), "{result}");
    assert!(
        result.contains("c NO [USEATTR]") && result.contains("d NO [USEATTR]"),
        "{result}"
    );
    assert!(result.contains("\\Archive) \"/\" \"Archive\""), "{result}");
}
