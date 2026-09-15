use super::*;
use mail_kernel::Command;

#[path = "search/literals.rs"]
mod literals;

#[path = "search/literal_review.rs"]
mod literal_review;
#[path = "search/saved.rs"]
mod saved;

type Client = BufReader<tokio::io::DuplexStream>;

pub(super) async fn command(client: &mut Client, value: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{value}\r\n").as_bytes())
        .await
        .unwrap();
    read(client, value.split_once(' ').unwrap().0).await
}

pub(super) async fn read(client: &mut Client, tag: &str) -> String {
    let mut output = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        let done = line.starts_with(&format!("{tag} "));
        output.push_str(&line);
        if done {
            return output;
        }
    }
}

async fn session() -> (
    Client,
    tokio::task::JoinHandle<std::io::Result<()>>,
    Arc<SqliteStore>,
) {
    let (service, db) = service();
    for (raw, keywords, received_at) in [
        (
            "From: alice@example.org\r\nTo: bob@example.org\r\nSubject: Alpha\r\nDate: Mon, 1 Jan 2024 23:00:00 -0500\r\n\r\nred apple",
            vec!["$seen", "$flagged"],
            1706745600,
        ),
        (
            "From: bob@example.org\r\nTo: alice@example.org\r\nSubject: Beta\r\nX-Tag: present\r\nDate: Tue, 2 Jan 2024 01:00:00 +0000\r\n\r\nblue berry",
            vec!["$answered", "project"],
            1706832000,
        ),
        (
            "From: carol@example.org\r\nTo: alice@example.org\r\nSubject: Gamma\r\nDate: Wed, 3 Jan 2024 01:00:00 +0000\r\nContent-Transfer-Encoding: base64\r\n\r\ncmVkIHN0b25l",
            vec!["$draft", "$deleted"],
            1706918400,
        ),
    ] {
        db.execute(
            "a",
            db.account("a").unwrap().revision,
            vec![Command::Append {
                mailboxes: vec!["inbox".into()],
                received_at,
                raw: raw.as_bytes().to_vec(),
                keywords: keywords.into_iter().map(str::to_owned).collect(),
            }],
        )
        .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    assert!(
        command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}"))
            .await
            .contains("a OK")
    );
    assert!(
        command(&mut client, "b SELECT INBOX")
            .await
            .contains("b OK")
    );
    (client, task, db)
}

async fn search(client: &mut Client, criteria: &str, expected: &[u32]) {
    let result = command(client, &format!("s SEARCH {criteria}")).await;
    assert!(result.contains("s OK"), "{criteria}: {result}");
    let line = result
        .lines()
        .find(|line| line.starts_with("* SEARCH"))
        .expect("SEARCH response");
    let ids = line
        .split_ascii_whitespace()
        .skip(2)
        .map(|s| s.parse::<u32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids, expected, "{criteria}: {result}");
}

#[tokio::test]
async fn search_flags_and_keywords_use_message_state_without_mutating_it() {
    let (mut client, task, db) = session().await;
    let before = db.account("a").unwrap();
    for (criteria, expected) in [
        ("ALL", vec![1, 2, 3]),
        ("SEEN", vec![1]),
        ("UNSEEN", vec![2, 3]),
        ("ANSWERED", vec![2]),
        ("UNANSWERED", vec![1, 3]),
        ("FLAGGED", vec![1]),
        ("UNFLAGGED", vec![2, 3]),
        ("DELETED", vec![3]),
        ("UNDELETED", vec![1, 2]),
        ("DRAFT", vec![3]),
        ("UNDRAFT", vec![1, 2]),
        ("KEYWORD project", vec![2]),
        ("UNKEYWORD project", vec![1, 3]),
    ] {
        search(&mut client, criteria, &expected).await;
    }
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn search_dates_distinguish_internal_and_sent_days_and_sizes_are_strict() {
    let (mut client, task, db) = session().await;
    for (criteria, expected) in [
        ("ON 1-Feb-2024", vec![1]),
        ("BEFORE 2-Feb-2024", vec![1]),
        ("SINCE 2-Feb-2024", vec![2, 3]),
        ("SENTON 1-Jan-2024", vec![1]),
        ("SENTBEFORE 2-Jan-2024", vec![1]),
        ("SENTSINCE 2-Jan-2024", vec![2, 3]),
    ] {
        search(&mut client, criteria, &expected).await;
    }
    let size = db.account("a").unwrap().messages[0].size;
    search(&mut client, &format!("UID 1 LARGER {size}"), &[]).await;
    search(&mut client, &format!("UID 1 SMALLER {size}"), &[]).await;
    search(&mut client, &format!("UID 1 LARGER {}", size - 1), &[1]).await;
    search(&mut client, &format!("UID 1 SMALLER {}", size + 1), &[1]).await;
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn search_strings_decode_bodies_and_combine_logical_groups() {
    let (mut client, task, _) = session().await;
    for (criteria, expected) in [
        ("FROM alice", vec![1]),
        ("TO alice", vec![2, 3]),
        ("SUBJECT \"ALPHA\"", vec![1]),
        ("HEADER X-Tag \"\"", vec![2]),
        ("BODY \"red\"", vec![1, 3]),
        ("TEXT \"alpha\"", vec![1]),
        ("CHARSET UTF-8 BODY \"RED\"", vec![1, 3]),
        ("OR SEEN ANSWERED", vec![1, 2]),
        ("NOT DELETED", vec![1, 2]),
        ("(OR FROM alice FROM bob) UNDELETED", vec![1, 2]),
        ("OR (SEEN FLAGGED) (DRAFT DELETED)", vec![1, 3]),
        ("UID 2:*", vec![2, 3]),
        ("2:3", vec![2, 3]),
    ] {
        search(&mut client, criteria, &expected).await;
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn saved_search_keeps_message_identities_across_expunge_and_empty_replacement() {
    let (mut client, task, db) = session().await;
    let initial = command(&mut client, "initial FETCH $ (UID)").await;
    assert!(
        initial.contains("initial OK") && !initial.contains(" FETCH (UID"),
        "{initial}"
    );
    let saved = command(&mut client, "c UID SEARCH RETURN (SAVE) UID 2:3").await;
    assert!(saved.contains("c OK"), "{saved}");
    let found = command(&mut client, "d UID FETCH $ (UID)").await;
    assert!(
        found.contains("UID 2") && found.contains("UID 3"),
        "{found}"
    );
    db.execute(
        "a",
        db.account("a").unwrap().revision,
        vec![Command::Destroy { id: "e1".into() }],
    )
    .unwrap();
    assert!(command(&mut client, "e NOOP").await.contains("* 1 EXPUNGE"));
    let found = command(&mut client, "f FETCH $ (UID)").await;
    assert!(
        found.contains("* 1 FETCH (UID 2") && found.contains("* 2 FETCH (UID 3"),
        "{found}"
    );
    let saved = command(&mut client, "g SEARCH RETURN (SAVE) FROM nobody").await;
    assert!(saved.contains("g OK"), "{saved}");
    let empty = command(&mut client, "h FETCH $ (UID)").await;
    assert!(
        empty.contains("h OK") && !empty.contains("* 1 FETCH"),
        "{empty}"
    );
    assert!(
        command(&mut client, "i SEARCH RETURN (SAVE) ALL")
            .await
            .contains("i OK")
    );
    command(&mut client, "j SELECT INBOX").await;
    let reset = command(&mut client, "k FETCH $ (UID)").await;
    assert!(
        reset.contains("k OK") && !reset.contains(" FETCH (UID"),
        "{reset}"
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn invalid_search_is_atomic_and_does_not_replace_saved_results() {
    let (mut client, task, db) = session().await;
    assert!(
        command(&mut client, "c SEARCH RETURN (SAVE) SEEN")
            .await
            .contains("c OK")
    );
    let before = db.account("a").unwrap();
    for invalid in [
        "",
        "OR ALL",
        "NOT",
        "(ALL",
        "ALL)",
        "BEFORE 31-Feb-2024",
        "LARGER -1",
        "HEADER Subject",
        "UID 1::2",
        "RETURN (SAVE) UNKNOWN",
    ] {
        client
            .get_mut()
            .write_all(format!("bad SEARCH {invalid}\r\nnext UID FETCH $ (UID)\r\n").as_bytes())
            .await
            .unwrap();
        let result = read(&mut client, "next").await;
        assert!(result.contains("bad BAD"), "{invalid}: {result}");
        assert!(
            result.contains("next OK") && result.contains("UID 1"),
            "{result}"
        );
        assert!(!result.contains("UID 2"), "saved result changed: {result}");
        assert_eq!(db.account("a").unwrap(), before);
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[path = "search/zero.rs"]
mod zero;
