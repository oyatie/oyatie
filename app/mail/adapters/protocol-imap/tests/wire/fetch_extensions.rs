use super::*;
type Client = BufReader<tokio::io::DuplexStream>;

async fn command(client: &mut Client, command: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{command}\r\n").as_bytes())
        .await
        .unwrap();
    let tag = format!("{} ", command.split_ascii_whitespace().next().unwrap());
    let mut output = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        let done = line.starts_with(&tag);
        output.push_str(&line);
        if done {
            return output;
        }
    }
}

async fn login(client: &mut Client) {
    let response = command(client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    assert!(response.contains("a OK"), "{response}");
}

fn start(service: Arc<MailService>) -> (Client, tokio::task::JoinHandle<std::io::Result<()>>) {
    let (client, server) = tokio::io::duplex(65536);
    (
        BufReader::new(client),
        tokio::spawn(mail_protocol_imap::imap_session(server, service, true)),
    )
}

fn nstring(response: &str, name: &str) -> Option<String> {
    let start = response
        .find(&format!("{name} "))
        .unwrap_or_else(|| panic!("{response}"));
    let input = &response[start + name.len() + 1..];
    if input.starts_with("NIL") {
        return None;
    }
    if let Some(input) = input.strip_prefix('"') {
        let mut value = String::new();
        let mut chars = input.chars();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => return Some(value),
                '\\' => value.push(chars.next().unwrap()),
                _ => value.push(ch),
            }
        }
        panic!("unterminated quoted value: {response}");
    }
    let (length, rest) = input
        .strip_prefix('{')
        .unwrap()
        .split_once("}\r\n")
        .unwrap();
    Some(rest[..length.parse::<usize>().unwrap()].to_owned())
}

fn objectid(response: &str, name: &str) -> String {
    let prefix = format!("{name} (");
    let value = response
        .split_once(&prefix)
        .unwrap_or_else(|| panic!("{response}"))
        .1
        .split_once(')')
        .unwrap()
        .0;
    assert!(!value.is_empty() && value.len() <= 255, "{response}");
    assert!(
        value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b)),
        "{response}"
    );
    value.to_owned()
}

#[tokio::test]
async fn enable_utf8_is_authenticated_additive_and_scoped_to_one_session() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"From: alice@example.org\r\nSubject: =?utf-8?Q?R=C3=A9sum=C3=A9?=\r\n\r\nbody",
    )
    .unwrap();
    let before = db.account("a").unwrap();
    let (mut client, task) = start(service.clone());
    let rejected = command(&mut client, "pre ENABLE UTF8=ACCEPT").await;
    assert!(
        !rejected.contains("pre OK") && !rejected.contains("* ENABLED"),
        "{rejected}"
    );
    login(&mut client).await;
    let before_cap = command(&mut client, "cap CAPABILITY").await;
    let empty_enable = command(&mut client, "bad ENABLE").await;
    assert!(empty_enable.contains("bad BAD"), "{empty_enable}");
    let unknown = command(&mut client, "bad ENABLE X-UNKNOWN UTF8=ACCEPT").await;
    assert!(
        unknown.contains("bad BAD") && !unknown.contains("ENABLED"),
        "{unknown}"
    );
    let enabled = command(
        &mut client,
        "enable ENABLE STARTTLS LOGINDISABLED UTF8=ACCEPT",
    )
    .await;
    assert!(enabled.contains("enable OK"), "{enabled}");
    let names = enabled
        .lines()
        .find(|l| l.starts_with("* ENABLED"))
        .unwrap();
    assert_eq!(names, "* ENABLED UTF8=ACCEPT");
    let again = command(&mut client, "again ENABLE UTF8=ACCEPT").await;
    assert!(again.contains("again OK"), "{again}");
    let after_cap = command(&mut client, "cap CAPABILITY").await;
    assert_eq!(before_cap, after_cap);
    let caps = before_cap
        .lines()
        .find(|l| l.starts_with("* CAPABILITY"))
        .unwrap();
    assert!(caps.split_ascii_whitespace().any(|c| c == "ENABLE"));
    assert!(caps.split_ascii_whitespace().any(|c| c == "UTF8=ACCEPT"));
    command(&mut client, "b SELECT INBOX").await;
    let envelope = command(&mut client, "c FETCH 1 ENVELOPE").await;
    assert!(envelope.contains("\"Résumé\""), "{envelope}");
    let (mut legacy, legacy_task) = start(service);
    login(&mut legacy).await;
    command(&mut legacy, "b SELECT INBOX").await;
    let envelope = command(&mut legacy, "c FETCH 1 ENVELOPE").await;
    assert!(
        envelope.is_ascii() && envelope.contains("=?utf-8?"),
        "{envelope}"
    );
    command(&mut legacy, "z LOGOUT").await;
    legacy_task.await.unwrap().unwrap();
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
    assert_eq!(db.account("a").unwrap(), before);
}

#[tokio::test]
async fn rejected_plaintext_enable_cannot_cross_the_starttls_session_boundary() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: =?utf-8?Q?caf=C3=A9?=\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_starttls_session(
        server,
        service,
        |stream| async { Ok(stream) },
    ));
    let mut client = BufReader::new(client);
    let before = command(&mut client, "pre ENABLE UTF8=ACCEPT").await;
    assert!(!before.contains("pre OK") && !before.contains("* ENABLED"));
    let upgraded = command(&mut client, "tls STARTTLS").await;
    assert!(upgraded.contains("tls OK"), "{upgraded}");
    login(&mut client).await;
    command(&mut client, "b SELECT INBOX").await;
    let envelope = command(&mut client, "c FETCH 1 ENVELOPE").await;
    assert!(
        envelope.is_ascii() && envelope.contains("=?utf-8?"),
        "{envelope}"
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn preview_is_utf8_plain_text_bounded_and_supports_lazy_without_marking_seen() {
    let (service, db) = service();
    let raw = format!(
        "Content-Type: text/html; charset=utf-8\r\n\r\n<p>Café &amp; tea</p><p>{}</p>",
        "☕".repeat(400)
    );
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    db.deliver(&["alice@example.org".into()], b"").unwrap();
    let before = db.account("a").unwrap();
    let (mut client, task) = start(service);
    login(&mut client).await;
    command(&mut client, "b SELECT INBOX").await;
    let response = command(&mut client, "c FETCH 1 PREVIEW").await;
    assert!(response.contains("c OK"), "{response}");
    let preview = nstring(&response, "PREVIEW").expect("non-LAZY PREVIEW cannot be NIL");
    assert!(preview.contains("Café & tea"), "{preview}");
    assert!(!preview.contains("<p>") && !preview.contains("&amp;"));
    assert!(preview.chars().count() <= 256);
    assert!(
        response.contains("PREVIEW {"),
        "legacy UTF-8 preview must be a literal: {response}"
    );
    let lazy = command(&mut client, "d FETCH 1 (UID PREVIEW (LAZY))").await;
    assert!(lazy.contains("d OK"), "{lazy}");
    if let Some(value) = nstring(&lazy, "PREVIEW") {
        assert_eq!(value, preview);
    }
    let empty = command(&mut client, "empty FETCH 2 PREVIEW").await;
    assert_eq!(nstring(&empty, "PREVIEW"), Some(String::new()));
    let bad = command(&mut client, "e FETCH 1 (BODY[] PREVIEW (UNKNOWN))").await;
    assert!(bad.contains("e BAD"), "{bad}");
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn object_ids_survive_copy_move_keyword_changes_and_late_thread_bridges() {
    let (service, db) = service();
    for id in ["first", "second"] {
        db.deliver(
            &["alice@example.org".into()],
            format!("Message-ID: <{id}@example.org>\r\nSubject: handoff\r\n\r\nbody").as_bytes(),
        )
        .unwrap();
    }
    let (mut client, task) = start(service.clone());
    login(&mut client).await;
    command(&mut client, "b SELECT INBOX").await;
    let first = command(&mut client, "c FETCH 1 (EMAILID THREADID)").await;
    let second = command(&mut client, "d FETCH 2 (EMAILID THREADID)").await;
    let first_ids = (objectid(&first, "EMAILID"), objectid(&first, "THREADID"));
    let second_ids = (objectid(&second, "EMAILID"), objectid(&second, "THREADID"));
    assert_ne!(first_ids.0, first_ids.1);
    assert_ne!(first_ids.1, second_ids.1);
    assert_ne!(first_ids, second_ids);
    db.deliver(&["alice@example.org".into()], b"Message-ID: <bridge@example.org>\r\nReferences: <first@example.org> <second@example.org>\r\nSubject: Re: handoff\r\n\r\nbody").unwrap();
    let merged = db.account("a").unwrap();
    assert_eq!(merged.messages.len(), 3);
    assert!(
        merged
            .messages
            .iter()
            .all(|m| m.thread_id() == merged.messages[0].thread_id()),
        "bridge must actually merge the JMAP threads"
    );
    for (sequence, expected) in [(1, &first_ids), (2, &second_ids)] {
        let response = command(
            &mut client,
            &format!("check FETCH {sequence} (EMAILID THREADID)"),
        )
        .await;
        assert_eq!(
            (
                objectid(&response, "EMAILID"),
                objectid(&response, "THREADID")
            ),
            *expected
        );
    }
    command(&mut client, "e CREATE Archive").await;
    for request in [
        "f UID COPY 1 Archive",
        "g UID STORE 1 +FLAGS (\\Flagged)",
        "h UID MOVE 1 Archive",
    ] {
        let response = command(&mut client, request).await;
        assert!(
            response.contains(&format!("{} OK", &request[..1])),
            "{response}"
        );
    }
    command(&mut client, "i SELECT Archive").await;
    for sequence in [1, 2] {
        let response = command(
            &mut client,
            &format!("j FETCH {sequence} (EMAILID THREADID)"),
        )
        .await;
        assert_eq!(
            (
                objectid(&response, "EMAILID"),
                objectid(&response, "THREADID")
            ),
            first_ids
        );
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
    let (mut client, task) = start(service);
    login(&mut client).await;
    command(&mut client, "b SELECT Archive").await;
    let response = command(&mut client, "c FETCH 1 (EMAILID THREADID)").await;
    assert_eq!(
        (
            objectid(&response, "EMAILID"),
            objectid(&response, "THREADID")
        ),
        first_ids
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
