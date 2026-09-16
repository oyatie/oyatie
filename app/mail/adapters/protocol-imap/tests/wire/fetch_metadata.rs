use super::*;
#[derive(Debug, PartialEq)]
enum Value {
    Atom(String),
    Text(String),
    List(Vec<Value>),
}
impl Value {
    fn list(&self) -> &[Self] {
        let Self::List(value) = self else {
            panic!("expected list: {self:?}")
        };
        value
    }
    fn text(&self) -> &str {
        let (Self::Text(value) | Self::Atom(value)) = self else {
            panic!("expected string: {self:?}")
        };
        value
    }
}
fn parse(input: &mut &[u8]) -> Value {
    while input.first() == Some(&b' ') {
        *input = &input[1..];
    }
    let head = input[0];
    *input = &input[1..];
    if head == b'(' {
        let mut values = Vec::new();
        while input.first() != Some(&b')') {
            values.push(parse(input));
            while input.first() == Some(&b' ') {
                *input = &input[1..];
            }
        }
        *input = &input[1..];
        return Value::List(values);
    }
    let mut value = Vec::new();
    if head == b'"' {
        while input[0] != b'"' {
            if input[0] == b'\\' {
                *input = &input[1..];
            }
            assert!(
                !input[0].is_ascii_control(),
                "control byte in IMAP quoted string"
            );
            value.push(input[0]);
            *input = &input[1..];
        }
        *input = &input[1..];
    } else {
        value.push(head);
        while input.first().is_some_and(|b| !b" )".contains(b)) {
            value.push(input[0]);
            *input = &input[1..];
        }
    }
    let value = String::from_utf8(value).unwrap();
    if head == b'"' {
        Value::Text(value)
    } else {
        Value::Atom(value)
    }
}

async fn fetch(raw: &str, items: &str) -> (Value, String) {
    let (service, db) = service();
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    let before = db.account("a").unwrap();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    client.write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\nc FETCH 1 {items}\r\nz LOGOUT\r\n").as_bytes()).await.unwrap();
    let mut response = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.read_to_string(&mut response),
    )
    .await
    .unwrap()
    .unwrap();
    task.await.unwrap().unwrap();
    assert!(response.contains("c OK"), "{response}");
    assert_eq!(db.account("a").unwrap(), before);
    let line = response
        .lines()
        .find_map(|l| l.strip_prefix("* 1 FETCH "))
        .unwrap();
    let mut bytes = line.as_bytes();
    let parsed = parse(&mut bytes);
    assert!(bytes.is_empty(), "trailing FETCH syntax: {line}");
    (parsed, response)
}

fn item<'a>(response: &'a Value, name: &str) -> &'a Value {
    &response
        .list()
        .as_chunks::<2>()
        .0
        .iter()
        .find(|p| p[0].text().eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("missing {name}: {response:?}"))[1]
}
fn structure(actual: &Value, expected: &str) {
    // MIME media types, transfer encodings and parameter attribute names are case insensitive.
    assert_eq!(
        format!("{actual:?}").to_ascii_lowercase(),
        format!("{:?}", parse(&mut expected.as_bytes())).to_ascii_lowercase()
    );
}
fn decoded(value: &Value) -> String {
    assert!(
        value.text().is_ascii(),
        "IMAP4rev1 metadata must use RFC2047"
    );
    let raw = format!("Subject: {}\r\n\r\n", value.text());
    mail_parser::MessageParser::default()
        .parse(raw.as_bytes())
        .unwrap()
        .subject()
        .unwrap()
        .to_owned()
}
#[tokio::test]
async fn envelope_keeps_groups_and_defaults_empty_or_absent_sender_and_reply_to() {
    let raw = concat!(
        "Date: Tue,\r\n\t15 Sep 2026 10:11:12 -0400\r\nSubject: handoff\r\n",
        "From: Alice <Alice@example.org>\r\nSender:\r\n",
        "To: Team: Bob <bob@example.net>, Carol <carol@example.net>;, Empty:;\r\n",
        "Cc: Dan <dan@example.net>\r\nBcc: Eve <eve@example.net>\r\n",
        "In-Reply-To: <prior@example.org>\r\n\t<older@example.org>\r\nMessage-ID: <current@example.org>\r\n\r\nbody\r\n",
    );
    let (result, _) = fetch(raw, "ENVELOPE").await;
    let env = item(&result, "ENVELOPE").list();
    assert_eq!(env.len(), 10);
    assert_eq!(env[0].text(), "Tue, 15 Sep 2026 10:11:12 -0400");
    assert_eq!(env[1].text(), "handoff");
    assert_eq!(env[2], env[3]);
    assert_eq!(env[2], env[4]);
    assert_eq!(env[2].list()[0].list()[2].text(), "Alice");
    assert_eq!(env[5], parse(&mut b"((NIL NIL \"Team\" NIL)(\"Bob\" NIL \"bob\" \"example.net\")(\"Carol\" NIL \"carol\" \"example.net\")(NIL NIL NIL NIL)(NIL NIL \"Empty\" NIL)(NIL NIL NIL NIL))".as_slice()));
    assert_eq!(env[6].list()[0].list()[2].text(), "dan");
    assert_eq!(env[7].list()[0].list()[2].text(), "eve");
    assert_eq!(env[8].text(), "<prior@example.org> <older@example.org>");
    assert_eq!(env[9].text(), "<current@example.org>");
    let (explicit, _) = fetch(
        &raw.replace(
            "Sender:\r\n",
            "Sender: sender@example.org\r\nReply-To: reply@example.org\r\n",
        ),
        "ENVELOPE",
    )
    .await;
    let explicit = item(&explicit, "ENVELOPE").list();
    assert_eq!(explicit[3].list()[0].list()[2].text(), "sender");
    assert_eq!(explicit[4].list()[0].list()[2].text(), "reply");
}
#[tokio::test]
async fn envelope_encodes_legacy_unicode_and_escapes_quoted_names_and_subjects() {
    let raw = concat!(
        "From: =?utf-8?Q?Zo=C3=AB?= <zoe@example.org>\r\n",
        "To: \"Quoted \\\"Name\\\" \\\\ Path\" <quoted@example.org>\r\n",
        "Subject: =?utf-8?Q?R=C3=A9sum=C3=A9_=22=E2=98=95=22_=5C_Path?=\r\n\r\nbody\r\n",
    );
    let (result, _) = fetch(raw, "ENVELOPE").await;
    let env = item(&result, "ENVELOPE").list();
    assert_eq!(decoded(&env[1]), "Résumé \"☕\" \\ Path");
    assert_eq!(decoded(&env[2].list()[0].list()[0]), "Zoë");
    assert_eq!(env[5].list()[0].list()[0].text(), "Quoted \"Name\" \\ Path");
    let (result, response) = fetch(
        "From: alice@example.org\r\nSubject: =?utf-8?Q?hello=0D=0A*_BYE_forged?=\r\n\r\nbody",
        "ENVELOPE",
    )
    .await;
    assert_eq!(item(&result, "ENVELOPE").list().len(), 10);
    assert!(
        !response.lines().any(|line| line == "* BYE forged"),
        "{response}"
    );
}

#[tokio::test]
async fn bodystructure_keeps_encoded_octets_lines_and_ordered_extensions() {
    let raw = concat!(
        "Content-Type: text/plain; charset=utf-8\r\n",
        "Content-Transfer-Encoding: quoted-printable\r\nContent-ID: <part@example.org>\r\n",
        "Content-Description: notes\r\nContent-Disposition: attachment; filename=notes.txt\r\n",
        "Content-Language: en, fr\r\nContent-Location: /notes.txt\r\n\r\n",
        "caf=C3=A9\r\nsecond\r\n",
    );
    let (result, _) = fetch(raw, "(BODY BODYSTRUCTURE)").await;
    structure(
        item(&result, "BODY"),
        "(\"text\" \"plain\" (\"charset\" \"utf-8\") \"<part@example.org>\" \"notes\" \"quoted-printable\" 19 2)",
    );
    structure(
        item(&result, "BODYSTRUCTURE"),
        "(\"text\" \"plain\" (\"charset\" \"utf-8\") \"<part@example.org>\" \"notes\" \"quoted-printable\" 19 2 NIL (\"attachment\" (\"filename\" \"notes.txt\")) (\"en\" \"fr\") \"/notes.txt\")",
    );
    let (unicode, _) = fetch(
        &raw.replace("filename=notes.txt", "filename*=utf-8''caf%C3%A9.txt"),
        "BODYSTRUCTURE",
    )
    .await;
    let disposition = item(&unicode, "BODYSTRUCTURE").list()[9].list();
    assert_eq!(decoded(&disposition[1].list()[1]), "café.txt");
}

#[tokio::test]
async fn multipart_body_omits_extensions_but_structure_keeps_nested_message_metadata() {
    let nested =
        "From: nested@example.org\r\nSubject: inner\r\nContent-Type: text/plain\r\n\r\none\r\ntwo";
    let raw = format!(
        "Content-Type: multipart/digest; boundary=outer\r\nContent-Disposition: inline\r\nContent-Language: en\r\nContent-Location: /bundle\r\n\r\npreamble\r\n--outer\r\nContent-Type: text/plain\r\nContent-Transfer-Encoding: base64\r\n\r\nYQ==\r\nYg==\r\n--outer\r\n\r\n{nested}\r\n--outer--\r\nepilogue\r\n"
    );
    let (result, _) = fetch(&raw, "(BODY BODYSTRUCTURE)").await;
    let body = item(&result, "BODY").list();
    assert_eq!(body.len(), 3);
    assert!(body[2].text().eq_ignore_ascii_case("digest"));
    structure(
        &body[0],
        "(\"text\" \"plain\" (\"charset\" \"us-ascii\") NIL NIL \"base64\" 10 1)",
    );
    let message = body[1].list();
    assert_eq!(message.len(), 10);
    assert!(message[0].text().eq_ignore_ascii_case("message"));
    assert!(message[1].text().eq_ignore_ascii_case("rfc822"));
    assert_eq!(message[6].text(), nested.len().to_string());
    assert_eq!(message[7].list()[1].text(), "inner");
    structure(
        &message[8],
        "(\"text\" \"plain\" (\"charset\" \"us-ascii\") NIL NIL \"7bit\" 8 1)",
    );
    assert_eq!(
        message[9].text(),
        nested.bytes().filter(|b| *b == b'\n').count().to_string()
    );
    let extended = item(&result, "BODYSTRUCTURE").list();
    assert_eq!(extended.len(), 7);
    structure(&extended[3], "(\"boundary\" \"outer\")");
    structure(&extended[4], "(\"inline\" NIL)");
    assert!(
        matches!(&extended[5], Value::Text(s) if s.eq_ignore_ascii_case("en"))
            || extended[5] == parse(&mut b"(\"en\")".as_slice())
    );
    assert_eq!(extended[6].text(), "/bundle");
    assert_eq!(extended[0].list().len(), 12);
    assert_eq!(extended[1].list().len(), 14);
    assert_eq!(extended[1].list()[8].list().len(), 12);
}

#[tokio::test]
async fn fetch_macros_expand_exact_metadata_without_marking_seen() {
    for raw in [
        "",
        "From: alice@example.org\r\nSubject: macros\r\n\r\nbody\r\n",
    ] {
        for (name, expected) in [
            ("FAST", "UID FLAGS INTERNALDATE RFC822.SIZE"),
            ("ALL", "UID FLAGS INTERNALDATE RFC822.SIZE ENVELOPE"),
            ("FULL", "UID FLAGS INTERNALDATE RFC822.SIZE ENVELOPE BODY"),
        ] {
            let (result, _) = fetch(raw, name).await;
            let mut names = result
                .list()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| p[0].text().to_ascii_uppercase())
                .collect::<Vec<_>>();
            names.sort();
            let mut expected = expected.split_ascii_whitespace().collect::<Vec<_>>();
            expected.sort();
            assert_eq!(names, expected, "{name}");
            assert_eq!(item(&result, "RFC822.SIZE").text(), raw.len().to_string());
        }
    }
}

#[tokio::test]
async fn fetch_macros_refuse_mixed_item_lists_without_mutation_or_lost_pipeline() {
    let (service, db) = service();
    let raw = b"Subject: macros\r\n\r\nbody";
    db.deliver(&["alice@example.org".into()], raw).unwrap();
    let before = db.account("a").unwrap();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    client.write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\nc FETCH 1 (FULL)\r\nd FETCH 1 (FAST UID)\r\ne FETCH 1 (BODY[] ALL)\r\nf FETCH 1 ALL FLAGS\r\ng FETCH 1 UID\r\nz LOGOUT\r\n").as_bytes()).await.unwrap();
    let mut response = String::new();
    client.read_to_string(&mut response).await.unwrap();
    task.await.unwrap().unwrap();
    for tag in ["c", "d", "e", "f"] {
        assert!(response.contains(&format!("{tag} BAD")), "{response}");
    }
    assert!(response.contains("g OK"), "{response}");
    assert_eq!(response.matches("* 1 FETCH ").count(), 1, "{response}");
    assert_eq!(db.account("a").unwrap(), before);
}
