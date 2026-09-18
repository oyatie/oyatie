use super::*;
use mail_kernel::Command;

#[path = "sort/collation.rs"]
mod collation;
#[path = "sort/review.rs"]
mod review;

type Client = BufReader<tokio::io::DuplexStream>;

async fn command(client: &mut Client, value: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{value}\r\n").as_bytes())
        .await
        .unwrap();
    read(client, value.split_once(' ').unwrap().0).await
}

async fn read(client: &mut Client, tag: &str) -> String {
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
    for (headers, received_at, size, seen) in [
        (
            "From: Alpha <bob@a.example>\r\nCc: zed@example.org\r\nTo: charlie@example.org\r\nSubject: Zulu\r\nDate: Mon, 1 Jan 2024 23:00:00 -0200\r\n",
            300,
            1000,
            false,
        ),
        (
            "From: Zebra <bob@z.example>\r\nCc: alpha@example.org\r\nTo: beta@example.org\r\nSubject: [team] Re: Alpha (fwd)\r\nDate: Tue, 2 Jan 2024 00:00:00 +0000\r\n",
            100,
            800,
            true,
        ),
        ("Subject: Re: beta\r\n", 100, 600, false),
        (
            "From: amy@example.org\r\nCc: beta@example.org\r\nTo: alpha@example.org\r\nSubject: [fwd: Fwd: Alpha]\r\nDate: Tue, 2 Jan 2024 00:00:00 +0000\r\n",
            200,
            1200,
            false,
        ),
    ] {
        let mut raw = format!("{headers}\r\nbody ").into_bytes();
        raw.resize(size, b'x');
        db.execute(
            "a",
            mail_api::Precondition::Observed(db.account("a").unwrap().revision),
            vec![Command::Append {
                mailboxes: vec!["inbox".into()],
                received_at,
                raw,
                keywords: if seen { vec!["$seen".into()] } else { vec![] },
            }],
        )
        .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
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

async fn sorted(client: &mut Client, request: &str, expected: &[u32]) {
    let result = command(client, &format!("s {request}")).await;
    assert!(result.contains("s OK"), "{request}: {result}");
    let line = result
        .lines()
        .find(|line| line.starts_with("* SORT"))
        .expect("SORT response");
    let ids = line
        .split_ascii_whitespace()
        .skip(2)
        .map(|id| id.parse::<u32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids, expected, "{request}: {result}");
}

#[tokio::test]
async fn sort_all_standard_keys_use_oracle_index_order_and_preserve_flags() {
    let (mut client, task, db) = session().await;
    let before = db.account("a").unwrap();
    for (key, expected) in [
        ("ARRIVAL", vec![2, 3, 4, 1]),
        ("CC", vec![2, 4, 1, 3]),
        ("DATE", vec![2, 4, 1, 3]),
        ("FROM", vec![4, 1, 2, 3]),
        ("SIZE", vec![3, 2, 1, 4]),
        ("SUBJECT", vec![2, 4, 3, 1]),
        ("TO", vec![4, 2, 1, 3]),
    ] {
        sorted(&mut client, &format!("SORT ({key}) UTF-8 ALL"), &expected).await;
    }
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn sort_reverse_applies_to_one_key_and_never_reverses_implicit_ties() {
    let (mut client, task, _) = session().await;
    for (keys, expected) in [
        ("REVERSE SUBJECT", vec![1, 3, 2, 4]),
        ("REVERSE FROM", vec![1, 2, 4, 3]),
        ("SUBJECT REVERSE ARRIVAL", vec![4, 2, 3, 1]),
        ("REVERSE DATE", vec![1, 2, 4, 3]),
    ] {
        sorted(
            &mut client,
            &format!("SORT ({keys}) US-ASCII ALL"),
            &expected,
        )
        .await;
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn sort_uid_output_keeps_sequence_search_criteria_and_empty_results() {
    let (mut client, task, db) = session().await;
    db.execute(
        "a",
        mail_api::Precondition::Observed(db.account("a").unwrap().revision),
        vec![Command::Destroy { id: "e1".into() }],
    )
    .unwrap();
    command(&mut client, "n NOOP").await;
    sorted(&mut client, "UID SORT (SUBJECT) UTF-8 2:3", &[4, 3]).await;
    sorted(&mut client, "SORT (SUBJECT) UTF-8 UID 3:4", &[3, 2]).await;
    sorted(&mut client, "UID SORT (SUBJECT) UTF-8 UNSEEN", &[4, 3]).await;
    sorted(&mut client, "SORT (ARRIVAL) US-ASCII SUBJECT missing", &[]).await;
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn sequence_sort_defers_concurrent_expunge_and_preserves_requested_positions() {
    let (mut client, task, db) = session().await;
    db.execute(
        "a",
        mail_api::Precondition::Observed(db.account("a").unwrap().revision),
        vec![Command::Destroy { id: "e1".into() }],
    )
    .unwrap();
    let result = command(&mut client, "s SORT (SUBJECT) UTF-8 2:4").await;
    assert!(
        result.contains("s OK") && result.contains("* SORT 2 4 3\r\n"),
        "{result}"
    );
    assert!(
        !result.contains("EXPUNGE"),
        "non-UID SORT cannot emit EXPUNGE: {result}"
    );
    sorted(&mut client, "UID SORT (SUBJECT) UTF-8 ALL", &[2, 4, 3]).await;
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

fn result_ids(response: &str, name: &str) -> Vec<u32> {
    let line = response
        .lines()
        .find(|line| line.starts_with("* ESEARCH"))
        .unwrap_or_else(|| panic!("{response}"));
    let words = line.split_ascii_whitespace().collect::<Vec<_>>();
    let value = words[words.iter().position(|word| *word == name).unwrap() + 1];
    value
        .split(',')
        .flat_map(|part| {
            if let Some((a, b)) = part.split_once(':') {
                let a = a.parse::<u32>().unwrap();
                let b = b.parse::<u32>().unwrap();
                (a.min(b)..=a.max(b)).collect::<Vec<_>>()
            } else {
                vec![part.parse::<u32>().unwrap()]
            }
        })
        .collect()
}

#[tokio::test]
async fn esort_reports_oracle_numeric_extrema_and_save_reuses_matching_identities() {
    let (mut client, task, _) = session().await;
    let result = command(
        &mut client,
        "c UID SORT RETURN (ALL COUNT MIN MAX SAVE) (SUBJECT) UTF-8 ALL",
    )
    .await;
    assert!(
        result.contains("c OK") && result.contains("* ESEARCH (TAG \"c\") UID"),
        "{result}"
    );
    assert_eq!(result_ids(&result, "ALL"), [1, 4]);
    assert_eq!(result_ids(&result, "MIN"), [1]);
    assert_eq!(result_ids(&result, "MAX"), [4]);
    assert!(result.contains("COUNT 4"), "{result}");
    let saved = command(&mut client, "d UID SEARCH $").await;
    assert!(saved.contains("* SEARCH 1 4\r\n"), "{saved}");
    let result = command(
        &mut client,
        "e UID SORT RETURN (MIN SAVE) (SUBJECT) UTF-8 ALL",
    )
    .await;
    assert_eq!(result_ids(&result, "MIN"), [1]);
    let saved = command(&mut client, "f UID SEARCH $").await;
    assert!(saved.contains("* SEARCH 1\r\n"), "{saved}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn sort_rejects_malformed_bounded_programs_without_replacing_saved_results() {
    let (mut client, task, db) = session().await;
    command(&mut client, "c SEARCH RETURN (SAVE) SEEN").await;
    let before = db.account("a").unwrap();
    let mut invalid = [
        "() UTF-8 ALL",
        "(REVERSE) UTF-8 ALL",
        "(UNKNOWN) UTF-8 ALL",
        "(SUBJECT UTF-8 ALL",
        "SUBJECT UTF-8 ALL",
        "(SUBJECT) UTF-8",
        "(SUBJECT) UTF-8 OR ALL",
        "RETURN (SAVE UNKNOWN) (SUBJECT) UTF-8 ALL",
    ]
    .map(str::to_owned)
    .to_vec();
    invalid.push(format!("({}) UTF-8 ALL", "DATE ".repeat(1100)));
    invalid.push(format!("(SUBJECT) UTF-8 {}ALL", "NOT ".repeat(500)));
    for args in invalid {
        client
            .get_mut()
            .write_all(format!("bad SORT {args}\r\nnext UID SEARCH $\r\n").as_bytes())
            .await
            .unwrap();
        let result =
            tokio::time::timeout(std::time::Duration::from_secs(5), read(&mut client, "next"))
                .await
                .unwrap();
        assert!(
            result.contains("bad BAD") && result.contains("next OK"),
            "{args}: {result}"
        );
        assert!(result.contains("* SEARCH 2\r\n"), "{result}");
        assert_eq!(db.account("a").unwrap(), before);
    }
    let charset = command(&mut client, "bad SORT (SUBJECT) x-unknown ALL").await;
    assert!(charset.contains("bad NO [BADCHARSET"), "{charset}");
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
