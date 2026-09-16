use super::*;

#[tokio::test]
async fn mime_charsets_decode_subject_and_body_without_changing_stored_octets() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let app = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "https://localhost".into(),
    );
    for (charset, encoded, expected) in [
        ("Shift_JIS", "k/qWe4zq", "日本語"),
        ("MS932", "k/qWe4zq", "日本語"),
        ("Big5", "tmyl8w==", "郵件"),
        ("EUC-KR", "uN7Azw==", "메일"),
        ("GB18030", "08q8/g==", "邮件"),
        ("ISO-2022-JP", "GyRCRnxLXDhsGyhC", "日本語"),
        ("ISO-8859-1", "gA==", "€"),
    ] {
        let raw = format!(
            "Subject: =?{charset}?B?{encoded}?=\r\nContent-Type: text/plain; charset={charset}\r\nContent-Transfer-Encoding: base64\r\n\r\n{encoded}"
        );
        db.deliver(&["alice@example.org".into()], raw.as_bytes())
            .unwrap();
        let before = db.account("a").unwrap();
        let id = &before.messages.last().unwrap().id;
        let result = call(
            &app,
            "Email/get",
            json!({"accountId":"a","ids":[id],
            "properties":["subject", "bodyValues", "preview"],"fetchTextBodyValues":true}),
        )
        .await;
        let message = &result["list"][0];
        assert_eq!(message["subject"], expected, "{charset}: {message}");
        assert_eq!(
            message["bodyValues"]["0"]["value"], expected,
            "{charset}: {message}"
        );
        assert_eq!(message["preview"], expected, "{charset}: {message}");
        assert_eq!(db.blob("a", id).unwrap(), raw.as_bytes());
        assert_eq!(db.account("a").unwrap(), before);
    }
}

#[test]
fn truncated_received_fold_is_safe_in_message_and_header_parsers() {
    for ending in [" ", "\t", " \t ", " \r", " \r\nbody"] {
        let raw = format!("Received: x\r\n{ending}");
        let parser = mail_parser::MessageParser::default();
        let parsed = parser.parse(raw.as_bytes());
        let headers = parser.parse_headers(raw.as_bytes());
        assert_eq!(parsed.is_some(), headers.is_some());
    }
}
