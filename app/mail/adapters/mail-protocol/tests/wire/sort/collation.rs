use super::*;

#[tokio::test]
async fn base_subject_follows_rfc_artifacts_without_localized_threading_heuristics() {
    for (subjects, forward, reverse) in [
        (["AW: Zebra", "Bravo"], [1, 2], [2, 1]),
        (["x (fw)", "x"], [2, 1], [1, 2]),
        (["[first] [zebra]", "[middle]"], [2, 1], [1, 2]),
        (["A  B", "A B"], [1, 2], [1, 2]),
        (["[fwd: [fwd: Re: Alpha]]", "Alpha"], [1, 2], [1, 2]),
        (["Re [2]: Alpha", "Alpha"], [1, 2], [1, 2]),
    ] {
        let (service, db) = service();
        for subject in subjects {
            db.deliver(
                &["alice@example.org".into()],
                format!("Subject: {subject}\r\n\r\nbody").as_bytes(),
            )
            .unwrap();
        }
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
        let mut client = BufReader::new(client);
        command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
        command(&mut client, "b SELECT INBOX").await;
        sorted(&mut client, "SORT (SUBJECT) UTF-8 ALL", &forward).await;
        sorted(&mut client, "SORT (REVERSE SUBJECT) UTF-8 ALL", &reverse).await;
        command(&mut client, "z LOGOUT").await;
        task.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn subject_sort_uses_unicode_titlecase_then_compatibility_decomposition() {
    let (service, db) = service();
    for subject in [
        "Ǆ",
        "ǅ",
        "ǆ",
        "D[",
        "é",
        "e\u{301}",
        "Ｅ",
        "e",
        "ﬁ",
        "FI",
        "FZ",
        "=?utf-8?Q?=C3=89?=",
    ] {
        db.deliver(
            &["alice@example.org".into()],
            format!("Subject: {subject}\r\n\r\nbody").as_bytes(),
        )
        .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    // RFC5051's own DZ example becomes D + small-z + caron, after D[.
    // UnicodeData has no simple titlecase for the fi ligature: NFKD gives fi.
    sorted(
        &mut client,
        "SORT (SUBJECT) UTF-8 ALL",
        &[4, 1, 2, 3, 7, 8, 5, 6, 12, 10, 11, 9],
    )
    .await;
    sorted(
        &mut client,
        "SORT (REVERSE SUBJECT) UTF-8 ALL",
        &[9, 11, 10, 5, 6, 12, 7, 8, 1, 2, 3, 4],
    )
    .await;
    sorted(
        &mut client,
        "SORT (SUBJECT) UTF-8 SUBJECT \"Ǆ\"",
        &[1, 2, 3],
    )
    .await;
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
