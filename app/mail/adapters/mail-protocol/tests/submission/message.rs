use super::*;

#[test]
fn submission_strips_blind_headers_adds_missing_metadata_and_preserves_body() {
    let (service, db) = service(Arc::new(OwnerPolicy));
    let raw = b"From: alice@example.org\r\nBcc: hidden@remote.org,\r\n second@remote.org\r\nResent-Bcc: third@remote.org\r\nSubject: private\r\n\r\nBcc: this is body\r\n";
    service
        .submit(
            TOKEN,
            "alice@example.org",
            "alice@example.org",
            &["alice@example.org".into()],
            raw,
        )
        .unwrap();
    assert_eq!(service.deliver_pending(1).unwrap(), 1);
    let stored = db.blob("a", "e1").unwrap();
    let parsed = mail_parser::MessageParser::default()
        .parse(&stored)
        .unwrap();
    assert!(parsed.date().is_some());
    assert!(parsed.message_id().is_some());
    assert!(parsed.bcc().is_none());
    let text = String::from_utf8(stored.clone()).unwrap();
    assert!(
        !text.contains("hidden@remote.org")
            && !text.contains("second@remote.org")
            && !text.contains("third@remote.org")
    );
    assert!(text.ends_with("\r\n\r\nBcc: this is body\r\n"));
    // Already valid headers are byte-preserved through a later submission.
    service
        .submit(
            TOKEN,
            "alice@example.org",
            "alice@example.org",
            &["alice@example.org".into()],
            &stored,
        )
        .unwrap();
    assert_eq!(service.deliver_pending(1).unwrap(), 1);
    assert_eq!(db.blob("a", "e2").unwrap(), stored);
    for extra in [
        "Date: invalid\r\n",
        "Message-ID: <>\r\n",
        "Message-ID: <a@example.org> <b@example.org>\r\n",
        "Bcc : hidden@remote.org\r\n",
        "Subject: bad\nBcc: hidden@remote.org\r\n",
        "Date: Tue, 15 Sep 2026 00:00:00 +0000\r\nDate: Tue, 15 Sep 2026 00:00:00 +0000\r\n",
    ] {
        let raw = format!("From: alice@example.org\r\n{extra}\r\nbody\r\n");
        assert_eq!(
            service.submit(
                TOKEN,
                "alice@example.org",
                "alice@example.org",
                &["alice@example.org".into()],
                raw.as_bytes()
            ),
            Err(Error::Invalid)
        );
    }
    assert_eq!(service.deliver_pending(1).unwrap(), 0);
}

#[tokio::test]
async fn enabling_relay_never_allows_unauthenticated_inbound_relay() {
    let (_, db) = service(Arc::new(OwnerPolicy));
    let service = Arc::new(MailService {
        outbound: Some(db.clone()),
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol::smtp_session(server, service.clone()));
    let mut client = BufReader::new(client);
    reply(&mut client, "220").await;
    command(&mut client, "EHLO client", "250").await;
    command(&mut client, "MAIL FROM:<alice@example.org>", "250").await;
    command(&mut client, "RCPT TO:<bob@remote.org>", "550").await;
    command(&mut client, "QUIT", "221").await;
    task.await.unwrap().unwrap();
    assert!(
        mail_api::SubmissionQueue::claim_outbound(db.as_ref(), 1)
            .unwrap()
            .is_empty()
    );
    service
        .submit(
            TOKEN,
            "alice@example.org",
            "alice@example.org",
            &["bob@remote.org".into()],
            b"From: alice@example.org\r\n\r\nbody\r\n",
        )
        .unwrap();
    let lease = mail_api::SubmissionQueue::claim_outbound(db.as_ref(), 1)
        .unwrap()
        .pop()
        .unwrap();
    let raw = mail_api::SubmissionQueue::outbound_message(db.as_ref(), &lease)
        .unwrap()
        .raw;
    assert!(
        mail_parser::MessageParser::default()
            .parse(&raw)
            .unwrap()
            .message_id()
            .is_some()
    );
}

#[test]
fn submission_completes_final_line_and_refuses_untransportable_content() {
    let (service, db) = service(Arc::new(OwnerPolicy));
    let recipients = ["alice@example.org".into()];
    service
        .submit(
            TOKEN,
            "alice@example.org",
            "alice@example.org",
            &recipients,
            b"From: alice@example.org\r\n\r\nbody",
        )
        .unwrap();
    service.deliver_pending(1).unwrap();
    assert!(db.blob("a", "e1").unwrap().ends_with(b"body\r\n"));
    for body in [
        b"zero\0byte\r\n".to_vec(),
        b"bare\nnewline\r\n".to_vec(),
        b"bare\rreturn\r\n".to_vec(),
        vec![b'x'; 999],
    ] {
        let mut raw = b"From: alice@example.org\r\n\r\n".to_vec();
        raw.extend(body);
        assert_eq!(
            service.submit(
                TOKEN,
                "alice@example.org",
                "alice@example.org",
                &recipients,
                &raw
            ),
            Err(Error::Invalid)
        );
    }
    assert_eq!(service.deliver_pending(1).unwrap(), 0);
}
