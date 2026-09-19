use super::*;
use std::time::Duration;

type Client = BufReader<tokio::io::DuplexStream>;

async fn command(client: &mut Client, input: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{input}\r\n").as_bytes())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        let mut result = String::new();
        let tag = input.split_once(' ').unwrap().0;
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0, "{result}");
            let done = line.starts_with(&format!("{tag} "));
            result.push_str(&line);
            if done {
                return result;
            }
        }
    })
    .await
    .expect("session response deadline")
}

#[tokio::test]
async fn namespace_and_unselect_leave_deleted_messages_intact_and_clear_saved_selection() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: retained\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    let capability = command(&mut client, "c CAPABILITY").await;
    assert!(
        capability.contains("NAMESPACE") && capability.contains("UNSELECT"),
        "{capability}"
    );
    assert!(
        !command(&mut client, "n NAMESPACE")
            .await
            .contains("* NAMESPACE")
    );
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    let namespace = command(&mut client, "n NAMESPACE").await;
    assert!(
        namespace.contains("* NAMESPACE ((\"\" \"/\")) NIL NIL\r\n") && namespace.contains("n OK"),
        "{namespace}"
    );
    assert!(command(&mut client, "u UNSELECT").await.contains("u BAD"));
    assert!(command(&mut client, "k CHECK").await.contains("k BAD"));
    command(&mut client, "s SELECT INBOX").await;
    command(&mut client, "f STORE 1 +FLAGS (\\Deleted)").await;
    assert!(
        command(&mut client, "r SEARCH RETURN (SAVE) ALL")
            .await
            .contains("r OK")
    );
    assert!(
        command(&mut client, "v FETCH $ (UID)")
            .await
            .contains("* 1 FETCH")
    );
    let before = db.account("a").unwrap();
    assert!(command(&mut client, "u UNSELECT").await.contains("u OK"));
    assert!(
        command(&mut client, "f FETCH 1 (UID)")
            .await
            .contains("f BAD")
    );
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "s SELECT INBOX").await;
    let fetch = command(&mut client, "f FETCH $ (UID)").await;
    assert!(
        fetch.contains("f OK") && !fetch.contains("* 1 FETCH"),
        "{fetch}"
    );
    assert!(command(&mut client, "k CHECK").await.contains("k OK"));
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
    assert_eq!(db.account("a").unwrap(), before);
}

#[tokio::test]
async fn argument_free_commands_refuse_extras_without_closing_or_mutating_selected_mailbox() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: retained\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "s SELECT INBOX").await;
    command(&mut client, "f STORE 1 +FLAGS (\\Deleted)").await;
    let before = db.account("a").unwrap();
    // UNSELECT is absent: upstream tolerates a stray mailbox name there
    // (tests/src/imap/mailbox.rs sends `UNSELECT "L&APg-bende opgaver"`).
    for verb in [
        "CAPABILITY",
        "NOOP",
        "CHECK",
        "CLOSE",
        "EXPUNGE",
        "NAMESPACE",
        "LOGOUT",
    ] {
        for argument in ["extra", "\"\""] {
            let result = command(&mut client, &format!("b {verb} {argument}")).await;
            assert!(
                result.contains("b BAD") && !result.contains("* BYE"),
                "{result}"
            );
            let selected = command(&mut client, "f FETCH 1 (UID FLAGS)").await;
            assert!(
                selected.contains("f OK") && selected.contains("\\Deleted"),
                "{selected}"
            );
            assert_eq!(db.account("a").unwrap(), before);
        }
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn unselect_readonly_preserves_deleted_and_namespace_rechecks_revocation() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: retained\r\n\r\nbody",
    )
    .unwrap();
    let state = db.account("a").unwrap();
    db.execute(
        "a",
        state.revision,
        vec![mail_kernel::Command::Keywords {
            id: state.messages[0].id.clone(),
            keywords: vec!["$deleted".into()],
        }],
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    assert!(
        command(&mut client, "s EXAMINE INBOX")
            .await
            .contains("[READ-ONLY]")
    );
    let before = db.account("a").unwrap();
    let unselect = command(&mut client, "u UNSELECT").await;
    assert!(
        unselect.contains("u OK") && !unselect.contains("EXPUNGE"),
        "{unselect}"
    );
    assert!(
        command(&mut client, "f FETCH 1 (UID)")
            .await
            .contains("f BAD")
    );
    assert_eq!(db.account("a").unwrap(), before);
    db.revoke("a").unwrap();
    let namespace = command(&mut client, "n NAMESPACE").await;
    assert!(
        namespace.contains("n NO") && !namespace.contains("* NAMESPACE"),
        "{namespace}"
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
