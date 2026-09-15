use super::search::{command, read};
use super::*;

#[tokio::test]
async fn condstore_tracks_messages_and_conditionally_updates_without_lost_writes() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    let initial = command(&mut client, "s SELECT INBOX (CONDSTORE)").await;
    assert!(initial.contains("s OK"), "{initial}");
    let high = number(&initial, "HIGHESTMODSEQ ");
    command(&mut client, "u UNSELECT").await;
    command(&mut client, "c CREATE Folder").await;
    let status = command(&mut client, "t STATUS Folder (HIGHESTMODSEQ)").await;
    assert_eq!(number(&status, "HIGHESTMODSEQ "), high);
    let enable = command(&mut client, "e ENABLE CONDSTORE QRESYNC").await;
    assert!(enable.contains("ENABLED CONDSTORE QRESYNC"), "{enable}");
    command(&mut client, "s SELECT INBOX").await;
    db.deliver(&["alice@example.org".into()], b"Subject: one\r\n\r\none")
        .unwrap();
    let fetched = command(&mut client, "f UID FETCH 1 (FLAGS MODSEQ)").await;
    let first = number(&fetched, "MODSEQ (");
    assert!(first > high, "{fetched}");
    let changed = command(
        &mut client,
        &format!("k UID STORE 1 (UNCHANGEDSINCE {first}) +FLAGS.SILENT (\\Seen)"),
    )
    .await;
    assert!(
        changed.contains("k OK") && changed.contains("MODSEQ ("),
        "{changed}"
    );
    let next = number(&changed, "MODSEQ (");
    assert!(next > first);
    let before = db.account("a").unwrap();
    let stale = command(
        &mut client,
        &format!("k UID STORE 1 (UNCHANGEDSINCE {first}) +FLAGS.SILENT (\\Deleted)"),
    )
    .await;
    assert!(stale.contains("k OK [MODIFIED 1]"), "{stale}");
    assert_eq!(db.account("a").unwrap().messages, before.messages);
    let none = command(
        &mut client,
        &format!("f UID FETCH 1:* (FLAGS) (CHANGEDSINCE {next})"),
    )
    .await;
    assert!(none.contains("f OK") && !none.contains("FETCH ("), "{none}");
    let search = command(&mut client, &format!("q SEARCH MODSEQ {first}")).await;
    assert!(
        search.contains(&format!("SEARCH 1 (MODSEQ {next})")),
        "{search}"
    );
    for bad in [
        "UID FETCH 1 (FLAGS) (CHANGEDSINCE -1)",
        "UID FETCH 1 (FLAGS) (CHANGEDSINCE 18446744073709551616)",
        "UID STORE 1 (UNCHANGEDSINCE +1) +FLAGS (\\Deleted)",
        "SELECT INBOX (QRESYNC (1 2 1:* extra))",
    ] {
        let result = command(&mut client, &format!("bad {bad}")).await;
        assert!(result.contains("bad BAD"), "{result}");
        assert_eq!(db.account("a").unwrap().messages, before.messages);
    }
    client.get_mut().write_all(b"z LOGOUT\r\n").await.unwrap();
    read(&mut client, "z").await;
    task.await.unwrap().unwrap();
}

fn number(response: &str, prefix: &str) -> u64 {
    response
        .split_once(prefix)
        .unwrap_or_else(|| panic!("{response}"))
        .1
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[path = "condstore_review.rs"]
mod condstore_review;

#[tokio::test]
async fn condstore_optional_search_entries_and_qresync_sequence_hints_follow_oracle() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "e ENABLE QRESYNC").await;
    db.deliver(&["alice@example.org".into()], b"Subject: one\r\n\r\none")
        .unwrap();
    let current = db.account("a").unwrap().mail_modseq;
    for options in [
        "(QRESYNC (1 0 (1 1)))",
        "(CONDSTORE QRESYNC (1 0 1:* (1 1)))",
        "(QRESYNC (1 0 1:*) CONDSTORE)",
    ] {
        let result = command(&mut client, &format!("s SELECT INBOX {options}")).await;
        assert!(
            result.contains("s OK") && result.contains("FETCH (UID 1"),
            "{result}"
        );
    }
    for criteria in [
        "MODSEQ 0",
        "MODSEQ \"/flags/\\\\Seen\" all 0",
        "MODSEQ \"/flags/project\" priv 0",
        "MODSEQ \"/flags/$seen\" shared 0",
    ] {
        let result = command(&mut client, &format!("s SEARCH {criteria}")).await;
        assert!(
            result.contains(&format!("SEARCH 1 (MODSEQ {current})")),
            "{result}"
        );
    }
    let empty = command(
        &mut client,
        &format!("s SEARCH RETURN (ALL) MODSEQ {current}"),
    )
    .await;
    assert!(
        empty.contains(&format!(" MODSEQ {current}")) && !empty.contains(" ALL "),
        "{empty}"
    );
    let reverse = command(
        &mut client,
        "f UID FETCH 1 (FLAGS) (VANISHED CHANGEDSINCE 0)",
    )
    .await;
    assert!(
        reverse.contains("f OK") && reverse.contains("FETCH (UID 1"),
        "{reverse}"
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
