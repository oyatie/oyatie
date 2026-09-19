//! Executable capability matrix: the live IMAP CAPABILITY greeting and the
//! JMAP Session object must equal exactly the matrix rows, and every row must
//! name a test function that exists. Runs under cargo (the merge verdict).
#[path = "capability_matrix/rows.rs"]
mod rows;
use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use mail_api::MetadataStore;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use rows::Row;
use serde_json::Value;
use std::{collections::BTreeSet, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn service() -> Arc<MailService> {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    })
}

fn tokens(rows: &[&[Row]]) -> BTreeSet<String> {
    rows.iter()
        .flat_map(|rows| rows.iter().map(|row| row.token.to_owned()))
        .collect()
}

fn proven(rows: &[Row]) {
    for row in rows {
        assert!(!row.proof.is_empty(), "{} has no proof", row.token);
        for proof in row.proof {
            assert!(
                proof.source.contains(proof.needle),
                "{}: proof `{}` is not defined in tests/{}",
                row.token,
                proof.needle,
                proof.file
            );
        }
    }
}

async fn command(client: &mut BufReader<DuplexStream>, line: &str) -> Vec<String> {
    let tag = line.split(' ').next().unwrap().to_owned();
    client
        .get_mut()
        .write_all(format!("{line}\r\n").as_bytes())
        .await
        .unwrap();
    let mut lines = Vec::new();
    loop {
        let mut text = String::new();
        assert!(client.read_line(&mut text).await.unwrap() > 0, "{lines:?}");
        let done = text.starts_with(&format!("{tag} "));
        lines.push(text.trim_end().to_owned());
        if done {
            return lines;
        }
    }
}

async fn capabilities(client: &mut BufReader<DuplexStream>) -> BTreeSet<String> {
    let lines = command(client, "c CAPABILITY").await;
    assert!(lines.last().unwrap().starts_with("c OK"), "{lines:?}");
    let line = lines
        .iter()
        .find_map(|line| line.strip_prefix("* CAPABILITY "))
        .expect("untagged CAPABILITY");
    let list: Vec<&str> = line.split(' ').collect();
    let set: BTreeSet<String> = list.iter().map(|token| (*token).to_owned()).collect();
    assert_eq!(list.len(), set.len(), "duplicate tokens in {line}");
    set
}

async fn greeting(client: &mut BufReader<DuplexStream>) {
    let mut text = String::new();
    client.read_line(&mut text).await.unwrap();
    assert!(text.starts_with("* OK "), "{text}");
}

fn spawn(encrypted: bool) -> (BufReader<DuplexStream>, tokio::task::JoinHandle<()>) {
    let (client, server) = tokio::io::duplex(65536);
    let service = service();
    let task = tokio::spawn(async move {
        mail_protocol_imap::imap_session(server, service, encrypted)
            .await
            .unwrap();
    });
    (BufReader::new(client), task)
}

#[tokio::test]
async fn imap_capability_equals_the_matrix_on_every_transport_and_state() {
    for rows in [
        rows::IMAP_ALWAYS,
        rows::IMAP_AUTHENTICATED,
        rows::IMAP_ENCRYPTED,
        rows::IMAP_STARTTLS,
        rows::IMAP_PLAINTEXT,
    ] {
        proven(rows);
    }
    // Encrypted transport: pre-auth, then authenticated.
    let (mut client, task) = spawn(true);
    greeting(&mut client).await;
    assert_eq!(
        capabilities(&mut client).await,
        tokens(&[rows::IMAP_ALWAYS, rows::IMAP_ENCRYPTED])
    );
    let login = command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    assert!(login.last().unwrap().starts_with("a OK"), "{login:?}");
    assert_eq!(
        capabilities(&mut client).await,
        tokens(&[
            rows::IMAP_ALWAYS,
            rows::IMAP_ENCRYPTED,
            rows::IMAP_AUTHENTICATED
        ])
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap();
    // Plaintext transport without an upgrade path.
    let (mut client, task) = spawn(false);
    greeting(&mut client).await;
    assert_eq!(
        capabilities(&mut client).await,
        tokens(&[rows::IMAP_ALWAYS, rows::IMAP_PLAINTEXT])
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap();
    // Plaintext transport before STARTTLS.
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_starttls_session(
        server,
        service(),
        |stream| async { Ok(stream) },
    ));
    let mut client = BufReader::new(client);
    greeting(&mut client).await;
    assert_eq!(
        capabilities(&mut client).await,
        tokens(&[rows::IMAP_ALWAYS, rows::IMAP_STARTTLS])
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

fn keys(value: &Value) -> BTreeSet<String> {
    value.as_object().expect("object").keys().cloned().collect()
}

#[tokio::test]
async fn jmap_session_equals_the_matrix() {
    proven(rows::JMAP_CAPABILITIES);
    proven(rows::JMAP_URLS);
    proven(rows::JMAP_QUERY_SORTS);
    let router = mail_protocol_imap::jmap_router(service(), "https://mail.example".into());
    let response = router
        .oneshot(
            Request::get("/.well-known/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let session: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(
        keys(&session["capabilities"]),
        tokens(&[rows::JMAP_CAPABILITIES])
    );
    let account_capabilities: BTreeSet<String> = rows::JMAP_ACCOUNT_CAPABILITIES
        .iter()
        .map(|c| (*c).to_owned())
        .collect();
    assert_eq!(
        keys(&session["accounts"]["a"]["accountCapabilities"]),
        account_capabilities
    );
    assert_eq!(keys(&session["primaryAccounts"]), account_capabilities);
    let urls: BTreeSet<String> = keys(&session)
        .into_iter()
        .filter(|key| key.ends_with("Url"))
        .collect();
    assert_eq!(urls, tokens(&[rows::JMAP_URLS]));
    for url in &urls {
        assert!(
            session[url]
                .as_str()
                .is_some_and(|u| u.starts_with("https://mail.example/")),
            "{url}"
        );
    }
    let sorts: BTreeSet<String> = session["accounts"]["a"]["accountCapabilities"]
        ["urn:ietf:params:jmap:mail"]["emailQuerySortOptions"]
        .as_array()
        .expect("emailQuerySortOptions")
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(sorts, tokens(&[rows::JMAP_QUERY_SORTS]));
    // RFC 8620 §2: the Session state changes only when the object changes.
    assert_eq!(session["state"], "1");
}
