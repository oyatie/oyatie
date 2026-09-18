use super::{ImapConnection, ResponseType, Type, client, fixture};
use mail_kernel::Command;

async fn request(client: &mut ImapConnection, text: &str, kind: ResponseType) -> String {
    client.send(text).await;
    let lines = client.assert_read(Type::Tagged, kind).await;
    let status = match kind {
        ResponseType::Ok => "OK",
        ResponseType::No => "NO",
        ResponseType::Bad => "BAD",
        ResponseType::Bye => "BYE",
    };
    assert_eq!(
        lines.last().unwrap().split_ascii_whitespace().nth(1),
        Some(status),
        "{lines:?}"
    );
    lines.join("\n")
}
async fn connect() -> ImapConnection {
    let mut client = ImapConnection::connect(b"h").await;
    client.assert_read(Type::Untagged, ResponseType::Ok).await;
    client
}
async fn login(client: &mut ImapConnection) {
    client.authenticate("alice@example.org", super::TOKEN).await;
}

#[tokio::test]
async fn uidonly_is_authenticated_one_way_and_rejected_commands_do_not_mutate() {
    let service = fixture();
    client::initialize(service.clone());
    let mut client = connect().await;
    assert!(
        !request(&mut client, "CAPABILITY", ResponseType::Ok)
            .await
            .contains("UIDONLY")
    );
    request(&mut client, "ENABLE UIDONLY", ResponseType::Bad).await;
    login(&mut client).await;
    assert!(
        request(&mut client, "CAPABILITY", ResponseType::Ok)
            .await
            .contains("UIDONLY")
    );
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    let before = service.store.account("a").unwrap();
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    for text in [
        "STORE 1 +FLAGS (\\Seen)",
        "MOVE 1 \"Deleted Items\"",
        "COPY 1 \"Deleted Items\"",
        "FETCH 1 BODY[]",
        "SEARCH ALL",
        "SORT (ARRIVAL) UTF-8 ALL",
        "THREAD REFERENCES UTF-8 ALL",
    ] {
        assert!(
            request(&mut client, text, ResponseType::Bad)
                .await
                .contains("[UIDREQUIRED]")
        );
        assert_eq!(service.store.account("a").unwrap(), before);
    }
    client.close().await;
}

#[tokio::test]
async fn uidonly_uses_uids_after_sequence_numbers_diverge_and_for_unsolicited_changes() {
    let service = fixture();
    client::initialize(service.clone());
    let mut client = connect().await;
    login(&mut client).await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY CONDSTORE", ResponseType::Ok).await;
    request(
        &mut client,
        "UID STORE 1 +FLAGS (\\Deleted)",
        ResponseType::Ok,
    )
    .await;
    let response = request(&mut client, "EXPUNGE", ResponseType::Ok).await;
    assert!(
        response.contains("* VANISHED 1") && !response.contains(" EXPUNGE\n"),
        "{response}"
    );
    let response = request(&mut client, "UID FETCH 2 (FLAGS)", ResponseType::Ok).await;
    assert!(
        response.contains("* 2 UIDFETCH (") && !response.contains(" FETCH ("),
        "{response}"
    );
    let account = service.store.account("a").unwrap();
    let id = account.messages[0].id.clone();
    service
        .store
        .execute(
            "a",
            mail_api::Precondition::Observed(account.revision),
            vec![Command::Keywords {
                id,
                keywords: vec!["$answered".into()],
            }],
        )
        .unwrap();
    let response = request(&mut client, "NOOP", ResponseType::Ok).await;
    assert!(
        response.contains("* 2 UIDFETCH (")
            && response.contains("\\Answered")
            && !response.contains(" FETCH ("),
        "{response}"
    );
    request(
        &mut client,
        "UID MOVE 2 \"Deleted Items\"",
        ResponseType::Ok,
    )
    .await;
    request(&mut client, "SELECT \"Deleted Items\"", ResponseType::Ok).await;
    assert!(
        request(&mut client, "UID FETCH 1 (FLAGS)", ResponseType::Ok)
            .await
            .contains("* 1 UIDFETCH (")
    );
    client.close().await;
}

#[tokio::test]
async fn uidonly_rejects_nested_sequence_criteria_but_preserves_uid_saved_and_text_criteria() {
    client::initialize(fixture());
    let mut client = connect().await;
    login(&mut client).await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    for criteria in [
        "1:*",
        "NOT (UID 1:* 1:*)",
        "OR UID 1:* (NOT 1)",
        "(ALL (1,2))",
    ] {
        for verb in [
            "UID SEARCH",
            "UID SORT (ARRIVAL) UTF-8",
            "UID THREAD REFERENCES UTF-8",
        ] {
            assert!(
                request(
                    &mut client,
                    &format!("{verb} {criteria}"),
                    ResponseType::Bad
                )
                .await
                .contains("[UIDREQUIRED]")
            );
        }
    }
    for text in [
        "UID SEARCH UID 1:*",
        "UID SEARCH SUBJECT 1:*",
        "UID SEARCH RETURN (SAVE) UID 2",
        "UID SEARCH $",
        "UID FETCH $ (FLAGS)",
        "UID SEARCH OR UID $ UID 1",
    ] {
        request(&mut client, text, ResponseType::Ok).await;
    }
    client.close().await;
}

#[tokio::test]
async fn unauthenticate_resets_every_extension_and_saved_selection_without_mutating_mail() {
    let service = fixture();
    client::initialize(service.clone());
    let mut client = connect().await;
    login(&mut client).await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(
        &mut client,
        "ENABLE UIDONLY OBJECTID+ QRESYNC UTF8=ACCEPT",
        ResponseType::Ok,
    )
    .await;
    request(
        &mut client,
        "UID SEARCH RETURN (SAVE) ALL",
        ResponseType::Ok,
    )
    .await;
    let before = service.store.account("a").unwrap();
    request(&mut client, "UNAUTHENTICATE extra", ResponseType::Bad).await;
    assert!(
        request(&mut client, "FETCH 1 FLAGS", ResponseType::Bad)
            .await
            .contains("UIDREQUIRED")
    );
    request(&mut client, "UNAUTHENTICATE", ResponseType::Ok).await;
    assert!(
        !request(&mut client, "CAPABILITY", ResponseType::Ok)
            .await
            .contains("UIDONLY")
    );
    request(&mut client, "UID FETCH 1 FLAGS", ResponseType::No).await;
    login(&mut client).await;
    let response = request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    assert!(
        !response.contains("OBJECTID") && !response.contains("HIGHESTMODSEQ"),
        "{response}"
    );
    let response = request(&mut client, "FETCH 1 (UID)", ResponseType::Ok).await;
    assert!(
        response.contains(" FETCH (") && !response.contains("UIDFETCH"),
        "{response}"
    );
    let response = request(&mut client, "UID FETCH $ (UID)", ResponseType::Ok).await;
    assert!(!response.contains("FETCH ("), "{response}");
    request(&mut client, "SEARCH CHARSET UTF-8 ALL", ResponseType::Ok).await;
    request(
        &mut client,
        "UID FETCH 1 (FLAGS) (CHANGEDSINCE 0 VANISHED)",
        ResponseType::Bad,
    )
    .await;
    assert_eq!(service.store.account("a").unwrap(), before);
    client.close().await;
}

#[path = "uidonly_runtime.rs"]
mod runtime;
