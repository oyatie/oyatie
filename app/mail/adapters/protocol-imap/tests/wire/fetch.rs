use super::*;

#[path = "fetch/advanced.rs"]
mod advanced;

const HEADERS: &str = concat!(
    "From: Alice <alice@example.org>\r\n",
    "Subject: section test\r\n\tcontinued\r\n",
    "X-Trace: first\r\nX-Trace: second\r\n",
    "Content-Type: text/plain; charset=utf-8\r\n\r\n",
);
const TEXT: &str = "aéz\r\nsecond line\r\n";

async fn read_response(client: &mut BufReader<tokio::io::DuplexStream>, tag: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let prefix = format!("{tag} ");
    loop {
        let mut line = Vec::new();
        assert!(client.read_until(b'\n', &mut line).await.unwrap() > 0);
        let done = line.starts_with(prefix.as_bytes());
        output.extend(line);
        if done {
            return output;
        }
    }
}

async fn command(client: &mut BufReader<tokio::io::DuplexStream>, value: &str) -> Vec<u8> {
    client
        .get_mut()
        .write_all(format!("{value}\r\n").as_bytes())
        .await
        .unwrap();
    read_response(client, value.split_once(' ').unwrap().0).await
}

fn contains(bytes: &[u8], expected: &str) -> bool {
    bytes
        .windows(expected.len())
        .any(|w| w == expected.as_bytes())
}

fn literal(response: &[u8], label: &str, expected: &[u8]) {
    let separator = if label.ends_with('~') { "" } else { " " };
    let marker = format!("{label}{separator}{{{}}}\r\n", expected.len());
    let start = response
        .windows(marker.len())
        .position(|w| w == marker.as_bytes())
        .unwrap_or_else(|| panic!("missing {marker:?}: {}", String::from_utf8_lossy(response)))
        + marker.len();
    assert_eq!(&response[start..start + expected.len()], expected);
}

async fn select(client: &mut BufReader<tokio::io::DuplexStream>, read_only: bool) {
    let login = command(client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    assert!(contains(&login, "a OK"));
    let result = command(
        client,
        if read_only {
            "b EXAMINE INBOX"
        } else {
            "b SELECT INBOX"
        },
    )
    .await;
    assert!(contains(&result, "b OK"));
}

#[tokio::test]
async fn fetch_headers_preserve_folding_duplicates_and_blank_line() {
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
    let result = command(&mut client, "c UID FETCH 1 (BODY.PEEK[HEADER] BODY.PEEK[TEXT] BODY.PEEK[HEADER.FIELDS (subject x-trace)] BODY.PEEK[HEADER.FIELDS.NOT (from content-type)] BODY.PEEK[HEADER.FIELDS (Missing)])").await;
    assert!(contains(&result, "c OK"));
    literal(&result, "BODY[HEADER]", HEADERS.as_bytes());
    literal(&result, "BODY[TEXT]", TEXT.as_bytes());
    let subset =
        b"Subject: section test\r\n\tcontinued\r\nX-Trace: first\r\nX-Trace: second\r\n\r\n";
    literal(&result, "BODY[HEADER.FIELDS (SUBJECT X-TRACE)]", subset);
    literal(
        &result,
        "BODY[HEADER.FIELDS.NOT (FROM CONTENT-TYPE)]",
        subset,
    );
    literal(&result, "BODY[HEADER.FIELDS (MISSING)]", b"\r\n");
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_partial_offsets_count_octets_and_clip_at_end() {
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
    let result = command(&mut client, "c FETCH 1 (BODY.PEEK[TEXT]<2.1> BODY.PEEK[TEXT]<17.99> BODY.PEEK[TEXT]<999.4> BODY.PEEK[HEADER.FIELDS (subject)]<9.7>)").await;
    assert!(contains(&result, "c OK"));
    literal(&result, "BODY[TEXT]<2>", &[0xa9]);
    literal(&result, "BODY[TEXT]<17>", &TEXT.as_bytes()[17..]);
    literal(&result, "BODY[TEXT]<999>", b"");
    literal(&result, "BODY[HEADER.FIELDS (SUBJECT)]<9>", b"section");
    assert!(!contains(&result, "<2.1>"));
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_numbered_parts_preserve_transfer_encoding_and_mime_headers() {
    let raw = concat!(
        "Subject: multipart\r\nMIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=outer\r\n\r\n",
        "preamble\r\n--outer\r\n",
        "Content-Type: text/plain; charset=iso-8859-1\r\n",
        "Content-Transfer-Encoding: base64\r\n\r\n6Q==\r\n",
        "--outer\r\nContent-Type: multipart/alternative; boundary=inner\r\n\r\n",
        "--inner\r\nContent-Type: text/plain\r\n\r\nnested text\r\n",
        "--inner--\r\n--outer--\r\nepilogue\r\n",
    );
    let (service, db) = service();
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, false).await;
    let result = command(&mut client, "c FETCH 1 (BODY.PEEK[1] BODY.PEEK[1.TEXT] BODY.PEEK[1.MIME] BODY.PEEK[2.1] BODY.PEEK[MIME])").await;
    assert!(contains(&result, "c OK"));
    literal(&result, "BODY[1]", b"6Q==");
    literal(&result, "BODY[1.TEXT]", b"6Q==");
    literal(&result, "BODY[1.MIME]", b"Content-Type: text/plain; charset=iso-8859-1\r\nContent-Transfer-Encoding: base64\r\n\r\n");
    literal(&result, "BODY[2.1]", b"nested text");
    literal(
        &result,
        "BODY[MIME]",
        b"Content-Type: multipart/mixed; boundary=outer\r\n\r\n",
    );
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_sections_set_seen_only_when_writable_and_emit_unsolicited_flags() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        format!("{HEADERS}{TEXT}").as_bytes(),
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    select(&mut client, true).await;
    let result = command(&mut client, "c FETCH 1 (BODY[TEXT])").await;
    literal(&result, "BODY[TEXT]", TEXT.as_bytes());
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "d SELECT INBOX").await;
    let result = command(&mut client, "e FETCH 1 (BODY.PEEK[TEXT])").await;
    literal(&result, "BODY[TEXT]", TEXT.as_bytes());
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    let result = command(&mut client, "f FETCH 1 (BODY[TEXT])").await;
    literal(&result, "BODY[TEXT]", TEXT.as_bytes());
    assert!(
        contains(&result, "FLAGS (\\Seen)"),
        "{}",
        String::from_utf8_lossy(&result)
    );
    assert!(
        db.account("a").unwrap().messages[0]
            .keywords
            .contains(&"$seen".into())
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn malformed_fetch_sections_are_atomic_and_preserve_pipelined_commands() {
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
    let before = db.account("a").unwrap();
    for invalid in [
        "(BODY[TEXT] UNKNOWN)",
        "BODY[TEXT] UID",
        "(BODY[TEXT]",
        "BODY[TEXT]]",
        "BODY[0]",
        "BODY[1..2]",
        "BODY[HEADER.FIELDS ()]",
        "BODY[HEADER.FIELDS (Subject)(From)]",
        "BODY[HEADER.FIELDS (\"Subject)]",
        "BODY[HEADER.FIELDS (Subject)]junk",
        "BODY[TEXT]<0>",
        "BODY[TEXT]<0.0>",
        "BODY[TEXT]<-1.2>",
        "BODY[TEXT]<0.18446744073709551616>",
        "(UID) (CHANGEDSINCE 1 VANISHED)",
    ] {
        client
            .get_mut()
            .write_all(format!("bad FETCH 1 {invalid}\r\nnext FETCH 1 (UID)\r\n").as_bytes())
            .await
            .unwrap();
        let result = read_response(&mut client, "next").await;
        assert!(
            contains(&result, "bad BAD"),
            "{invalid}: {}",
            String::from_utf8_lossy(&result)
        );
        assert!(contains(&result, "next OK"));
        assert!(contains(&result, "UID 1"));
        assert_eq!(db.account("a").unwrap(), before, "{invalid}");
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
