use super::*;

type Client = BufReader<tokio::io::DuplexStream>;

async fn command(client: &mut Client, request: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{request}\r\n").as_bytes())
        .await
        .unwrap();
    let tag = format!("{} ", request.split_ascii_whitespace().next().unwrap());
    let mut result = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        let done = line.starts_with(&tag);
        result.push_str(&line);
        if done {
            return result;
        }
    }
}

async fn start(
    service: Arc<MailService>,
) -> (Client, tokio::task::JoinHandle<std::io::Result<()>>) {
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    let result = command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    assert!(result.contains("a OK"), "{result}");
    (client, task)
}

#[tokio::test]
async fn legacy_mailbox_modified_utf7_roundtrips_unicode_and_literal_ampersands() {
    let (service, db) = service();
    let (mut client, task) = start(service).await;
    for (encoded, unicode) in [("caf&AOk-", "café"), ("R&-D", "R&D"), ("&2D3eAA-", "😀")] {
        let created = command(&mut client, &format!("create CREATE \"{encoded}\"")).await;
        assert!(created.contains("create OK"), "{created}");
        assert!(
            db.account("a")
                .unwrap()
                .mailboxes
                .iter()
                .any(|m| m.name == unicode)
        );
        let selected = command(&mut client, &format!("select SELECT \"{encoded}\"")).await;
        assert!(selected.contains("select OK"), "{selected}");
        let listed = command(&mut client, &format!("list LIST \"\" \"{encoded}\"")).await;
        assert!(listed.contains(&format!("\"{encoded}\"")), "{listed}");
        assert!(listed.is_ascii(), "{listed}");
    }
    let before = db.account("a").unwrap();
    for invalid in ["café", "R&D", "&AA-", "&2D0-", "&3gA-", "&AOk=-", "&**-"] {
        let refused = command(&mut client, &format!("bad CREATE \"{invalid}\"")).await;
        assert!(refused.contains("bad BAD"), "{invalid}: {refused}");
    }
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn enabled_utf8_mailboxes_roundtrip_and_search_rejects_a_charset_parameter() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: =?utf-8?Q?caf=C3=A9?=\r\n\r\nbody",
    )
    .unwrap();
    let (mut legacy, legacy_task) = start(service.clone()).await;
    command(&mut legacy, "b SELECT INBOX").await;
    let legacy_search = command(&mut legacy, "c SEARCH CHARSET UTF-8 SUBJECT \"café\"").await;
    assert!(
        legacy_search.contains("* SEARCH 1\r\n") && legacy_search.contains("c OK"),
        "{legacy_search}"
    );
    let (mut client, task) = start(service).await;
    let enabled = command(&mut client, "enable ENABLE UTF8=ACCEPT").await;
    assert!(enabled.contains("enable OK"), "{enabled}");
    for name in ["café", "😀", "R&D"] {
        let created = command(&mut client, &format!("create CREATE \"{name}\"")).await;
        assert!(created.contains("create OK"), "{created}");
        let listed = command(&mut client, &format!("list LIST \"\" \"{name}\"")).await;
        assert!(listed.contains(&format!("\"{name}\"")), "{listed}");
    }
    let legacy_list = command(&mut legacy, "list LIST \"\" \"*\"").await;
    assert!(
        legacy_list.contains("\"caf&AOk-\"")
            && legacy_list.contains("\"R&-D\"")
            && legacy_list.contains("\"&2D3eAA-\""),
        "{legacy_list}"
    );
    assert!(legacy_list.is_ascii(), "{legacy_list}");
    command(&mut client, "b SELECT INBOX").await;
    let query = command(&mut client, "c SEARCH SUBJECT \"café\"").await;
    assert!(
        query.contains("* SEARCH 1\r\n") && query.contains("c OK"),
        "{query}"
    );
    for charset in ["UTF-8", "US-ASCII"] {
        let refused = command(&mut client, &format!("bad SEARCH CHARSET {charset} ALL")).await;
        assert!(refused.contains("bad BAD"), "{refused}");
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
    command(&mut legacy, "z LOGOUT").await;
    legacy_task.await.unwrap().unwrap();
}

async fn append(client: &mut Client, prefix: &str, raw: &[u8], close: &[u8]) -> (bool, String) {
    client
        .get_mut()
        .write_all(format!("{prefix}{{{}}}\r\n", raw.len()).as_bytes())
        .await
        .unwrap();
    let mut first = String::new();
    assert!(client.read_line(&mut first).await.unwrap() > 0);
    if !first.starts_with('+') {
        return (false, first);
    }
    client.get_mut().write_all(raw).await.unwrap();
    client.get_mut().write_all(close).await.unwrap();
    let mut response = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        let done = line.starts_with("append ");
        response.push_str(&line);
        if done {
            return (true, response);
        }
    }
}

#[tokio::test]
async fn utf8_append_requires_negotiation_preserves_flags_date_and_rejects_bad_closing() {
    let (service, db) = service();
    let (mut client, task) = start(service).await;
    let raw = "From: alice@example.org\r\nSubject: Résumé\r\n\r\nCafé\r\n".as_bytes();
    let (continued, refused) =
        append(&mut client, "append APPEND INBOX UTF8 (~", raw, b")\r\n").await;
    assert!(
        !continued,
        "unnegotiated UTF8 must refuse before continuation"
    );
    assert!(
        refused.contains("append NO") || refused.contains("append BAD"),
        "{refused}"
    );
    assert!(db.account("a").unwrap().messages.is_empty());
    let enabled = command(&mut client, "enable ENABLE UTF8=ACCEPT").await;
    assert!(enabled.contains("enable OK"), "{enabled}");
    let (continued, response) = append(
        &mut client,
        "append APPEND INBOX (\\Seen) \"01-Jan-2020 00:00:00 +0000\" UTF8 (~",
        raw,
        b")\r\n",
    )
    .await;
    assert!(
        continued && response.contains("append OK [APPENDUID"),
        "{response}"
    );
    let before = db.account("a").unwrap();
    assert_eq!(before.messages.len(), 1);
    let message = &before.messages[0];
    assert_eq!(message.received_at, 1_577_836_800);
    assert_eq!(message.keywords, ["$seen"]);
    assert_eq!(db.blob("a", &message.id).unwrap(), raw);
    let (_, malformed) = append(&mut client, "append APPEND INBOX UTF8 (~", raw, b"]\r\n").await;
    assert!(malformed.contains("append BAD"), "{malformed}");
    assert_eq!(db.account("a").unwrap(), before);
    let next = command(&mut client, "next NOOP").await;
    assert!(next.contains("next OK"), "{next}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn legacy_append_refuses_utf8_headers_but_preserves_high_octets_in_the_body() {
    let (service, db) = service();
    let (mut client, task) = start(service).await;
    let raw = "Subject: café\r\n\r\nbody".as_bytes();
    let (_, refused) = append(&mut client, "append APPEND INBOX ", raw, b"\r\n").await;
    assert!(refused.contains("append NO"), "{refused}");
    assert!(db.account("a").unwrap().messages.is_empty());
    let binary_body = b"Subject: opaque body\r\n\r\n\xff\xfe\r\n";
    let (_, accepted) = append(&mut client, "append APPEND INBOX ", binary_body, b"\r\n").await;
    assert!(accepted.contains("append OK"), "{accepted}");
    assert_eq!(db.blob("a", "e1").unwrap(), binary_body);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
