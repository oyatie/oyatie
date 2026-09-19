use super::*;
use mail_api::BlobStore;

#[tokio::test]
async fn fetch_header_field_names_accept_quoted_astrings() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        format!("{HEADERS}{TEXT}").as_bytes(),
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, false).await;
    client
        .get_mut()
        .write_all(b"c FETCH 1 (BODY.PEEK[HEADER.FIELDS (\"Subject\")])\r\nnext FETCH 1 (UID)\r\n")
        .await
        .unwrap();
    let result = read_response(&mut client, "next").await;
    assert!(
        contains(&result, "c OK"),
        "{}",
        String::from_utf8_lossy(&result)
    );
    literal(
        &result,
        "BODY[HEADER.FIELDS (SUBJECT)]",
        b"Subject: section test\r\n\tcontinued\r\n\r\n",
    );
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_quoted_header_name_brackets_are_data_not_section_delimiters() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"X]Trace: bracket\r\nX[Trace: open\r\nX\"Trace: quote\r\nX\\Trace: slash\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, false).await;
    for (name, header) in [
        (r#""X]Trace""#, "X]Trace: bracket\r\n\r\n"),
        (r#""X[Trace""#, "X[Trace: open\r\n\r\n"),
        (r#""X\"Trace""#, "X\"Trace: quote\r\n\r\n"),
        (r#""X\\Trace""#, "X\\Trace: slash\r\n\r\n"),
    ] {
        let result = command(
            &mut client,
            &format!("c FETCH 1 (BODY.PEEK[HEADER.FIELDS ({name})])"),
        )
        .await;
        assert!(
            contains(&result, "c OK"),
            "{name}: {}",
            String::from_utf8_lossy(&result)
        );
        assert!(
            contains(&result, header),
            "{name}: {}",
            String::from_utf8_lossy(&result)
        );
    }
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_binary_decodes_transfer_encoding_without_transcoding_charset() {
    let raw = concat!(
        "Content-Type: multipart/mixed; boundary=outer\r\n\r\n",
        "--outer\r\nContent-Type: text/plain; charset=iso-8859-1\r\n",
        "Content-Transfer-Encoding: base64\r\n\r\n6Q==\r\n",
        "--outer\r\nContent-Type: text/plain; charset=iso-8859-1\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\n\r\n=E9=00\r\n",
        "--outer\r\nContent-Type: text/plain; charset=utf-16le\r\n",
        "Content-Transfer-Encoding: base64\r\n\r\nQQDpAA==\r\n--outer--\r\n",
    );
    let (service, db) = service();
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, false).await;
    let result = command(&mut client, "c FETCH 1 (BINARY.PEEK[1] BINARY.SIZE[1] BINARY.PEEK[2] BINARY.SIZE[2] BINARY.PEEK[2]<1.1>)").await;
    assert!(
        contains(&result, "c OK"),
        "{}",
        String::from_utf8_lossy(&result)
    );
    literal(&result, "BINARY[1] ~", &[0xe9]);
    literal(&result, "BINARY[2] ~", &[0xe9, 0]);
    literal(&result, "BINARY[2]<1> ~", &[0]);
    assert!(contains(&result, "BINARY.SIZE[1] 1"));
    assert!(contains(&result, "BINARY.SIZE[2] 2"));
    let result = command(&mut client, "d FETCH 1 (BINARY.PEEK[3] BINARY.SIZE[3])").await;
    literal(&result, "BINARY[3] ~", &[0x41, 0, 0xe9, 0]);
    assert!(contains(&result, "BINARY.SIZE[3] 4"));
    let result = command(&mut client, "e FETCH 1 (BINARY.PEEK[] BINARY.SIZE[])").await;
    literal(&result, "BINARY[] ~", raw.as_bytes());
    assert!(contains(&result, &format!("BINARY.SIZE[] {}", raw.len())));
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_unknown_transfer_encoding_returns_typed_error_without_marking_peek_seen() {
    let (service, db) = service();
    let raw = b"Content-Type: text/plain\r\nContent-Transfer-Encoding: x-unknown\r\n\r\nopaque\r\n";
    db.deliver(&["alice@example.org".into()], raw).unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, false).await;
    let before = db.account("a").unwrap();
    let result = command(&mut client, "c FETCH 1 (BINARY.PEEK[1])").await;
    assert!(
        contains(&result, "NO [UNKNOWN-CTE]"),
        "{}",
        String::from_utf8_lossy(&result)
    );
    assert_eq!(db.account("a").unwrap(), before);
    let result = command(&mut client, "d FETCH 1 (BODY.PEEK[])").await;
    literal(&result, "BODY[]", raw);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_whole_message_does_not_require_successful_mime_parsing() {
    let (service, db) = service();
    for raw in [b"".as_slice(), b"\r\n", b"just body bytes\xff"] {
        let revision = db.account("a").unwrap().revision;
        db.execute(
            "a",
            mail_api::Precondition::Observed(revision),
            vec![
                db.append(
                    "a",
                    vec!["inbox".into()],
                    &raw.to_vec(),
                    vec![],
                    1_000_000_000,
                )
                .unwrap(),
            ],
        )
        .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, false).await;
    for (uid, raw) in [
        (1, b"".as_slice()),
        (2, b"\r\n"),
        (3, b"just body bytes\xff"),
    ] {
        let result = command(&mut client, &format!("c UID FETCH {uid} (BODY.PEEK[])")).await;
        literal(&result, "BODY[]", raw);
        assert!(contains(&result, "c OK"));
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_embedded_message_sections_resolve_nested_singlepart_message() {
    let nested_header =
        "From: nested@example.org\r\nSubject: embedded\r\nContent-Type: text/plain\r\n\r\n";
    let raw = format!(
        "Content-Type: multipart/mixed; boundary=outer\r\n\r\n--outer\r\nContent-Type: message/rfc822\r\n\r\n{nested_header}nested body\r\n--outer--\r\n"
    );
    let (service, db) = service();
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, false).await;
    let result = command(
        &mut client,
        "c FETCH 1 (BODY.PEEK[1.HEADER] BODY.PEEK[1.TEXT] BODY.PEEK[1.1] BODY.PEEK[1.MIME])",
    )
    .await;
    assert!(
        contains(&result, "c OK"),
        "{}",
        String::from_utf8_lossy(&result)
    );
    literal(&result, "BODY[1.HEADER]", nested_header.as_bytes());
    literal(&result, "BODY[1.TEXT]", b"nested body");
    literal(&result, "BODY[1.1]", b"nested body");
    literal(
        &result,
        "BODY[1.MIME]",
        b"Content-Type: message/rfc822\r\n\r\n",
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
