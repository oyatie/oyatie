use super::*;
use mail_api::BlobStore;
use mail_api::Precondition;
use mail_kernel::{Command, Error};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

#[path = "idle/boundary.rs"]
mod boundary;
#[path = "idle/support.rs"]
mod support;
use support::{idle, observed, wait_for};

type Client = BufReader<tokio::io::DuplexStream>;

async fn until(client: &mut Client, prefix: &str) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut result = String::new();
        loop {
            let mut line = String::new();
            assert!(
                client.read_line(&mut line).await.unwrap() > 0,
                "EOF before {prefix}: {result}"
            );
            let done = line.starts_with(prefix);
            result.push_str(&line);
            if done {
                return result;
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("response deadline waiting for {prefix}"))
}

async fn command(client: &mut Client, value: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{value}\r\n").as_bytes())
        .await
        .unwrap();
    until(
        client,
        &format!("{} ", value.split_ascii_whitespace().next().unwrap()),
    )
    .await
}

async fn start(
    service: Arc<MailService>,
    selected: bool,
    capacity: usize,
) -> (Client, tokio::task::JoinHandle<std::io::Result<()>>) {
    let (client, server) = tokio::io::duplex(capacity);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    assert!(
        command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}"))
            .await
            .contains("a OK")
    );
    if selected {
        assert!(
            command(&mut client, "b SELECT INBOX")
                .await
                .contains("b OK")
        );
    }
    (client, task)
}

#[tokio::test]
async fn idle_accepts_authenticated_and_selected_states_and_done_preserves_pipeline() {
    for selected in [false, true] {
        let (service, _) = super::service();
        let (mut client, task) = start(service, selected, 4096).await;
        let capability = command(&mut client, "cap CAPABILITY").await;
        assert!(
            capability
                .split_ascii_whitespace()
                .any(|word| word == "IDLE"),
            "{capability}"
        );
        assert!(
            command(&mut client, "bad IDLE unexpected")
                .await
                .contains("bad BAD")
        );
        idle(&mut client).await;
        client
            .get_mut()
            .write_all(b"DONE\r\nnext NOOP\r\nz LOGOUT\r\n")
            .await
            .unwrap();
        let completed = until(&mut client, "next ").await;
        assert!(
            completed.contains("i OK") && completed.contains("next OK"),
            "{completed}"
        );
        assert!(completed.find("i OK").unwrap() < completed.find("next OK").unwrap());
        until(&mut client, "z ").await;
        task.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn idle_announces_new_mail_flags_and_renumbered_expunge_without_client_polling() {
    let (service, db) = super::service();
    for _ in 0..3 {
        db.deliver(&["alice@example.org".into()], b"Subject: idle\r\n\r\nbody")
            .unwrap();
    }
    let (mut client, task) = start(service.clone(), true, 4096).await;
    idle(&mut client).await;
    // A commit the hint misses is seen within the polling floor (≤ 1.5 s).
    let started = std::time::Instant::now();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: arrival\r\n\r\nbody",
    )
    .unwrap();
    until(&mut client, "* 4 EXISTS").await;
    assert!(started.elapsed() <= Duration::from_millis(1500));
    // A commit through the service signals: seen well inside the floor.
    let started = std::time::Instant::now();
    let account = db.account("a").unwrap();
    service
        .execute(
            TOKEN,
            "a",
            Precondition::Observed(account.revision),
            vec![Command::Keywords {
                id: "e2".into(),
                keywords: vec!["$seen".into(), "$flagged".into()],
            }],
            &mail_service::Budget::fixed(),
        )
        .unwrap();
    let changed = until(&mut client, "* 2 FETCH").await;
    assert!(started.elapsed() < Duration::from_millis(500));
    assert!(
        changed.contains("FLAGS (") && changed.contains("\\Seen") && changed.contains("\\Flagged"),
        "{changed}"
    );
    let account = db.account("a").unwrap();
    db.execute(
        "a",
        Precondition::Observed(account.revision),
        vec![
            Command::Destroy { id: "e1".into() },
            Command::Destroy { id: "e3".into() },
        ],
    )
    .unwrap();
    until(&mut client, "* 1 EXPUNGE").await;
    until(&mut client, "* 2 EXPUNGE").await;
    client
        .get_mut()
        .write_all(b"DONE\r\nz LOGOUT\r\n")
        .await
        .unwrap();
    assert!(until(&mut client, "i ").await.contains("i OK"));
    until(&mut client, "z ").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn idle_closes_after_credential_revocation_and_releases_deleted_selection() {
    let (service, db) = super::service();
    let (mut client, task) = start(service, true, 4096).await;
    idle(&mut client).await;
    db.revoke("a").unwrap();
    until(&mut client, "* BYE").await;
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let (service, db) = super::service();
    let account = db
        .execute(
            "a",
            Precondition::Observed(0),
            vec![Command::CreateMailbox {
                name: "Archive".into(),
            }],
        )
        .map(|_| db.account("a").unwrap())
        .unwrap();
    let folder = account
        .mailboxes
        .iter()
        .find(|m| m.name == "Archive")
        .unwrap()
        .id
        .clone();
    let (mut client, task) = start(service, false, 4096).await;
    assert!(
        command(&mut client, "b SELECT Archive")
            .await
            .contains("b OK")
    );
    idle(&mut client).await;
    db.execute(
        "a",
        Precondition::Observed(account.revision),
        vec![Command::DeleteMailbox { id: folder }],
    )
    .unwrap();
    until(&mut client, "* OK [CLOSED]").await;
    client.get_mut().write_all(b"DONE\r\n").await.unwrap();
    assert!(until(&mut client, "i ").await.contains("i OK"));
    assert!(
        command(&mut client, "c FETCH 1 UID")
            .await
            .contains("c BAD")
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn idle_slow_storage_does_not_block_other_socket_tasks_and_disconnect_cancels() {
    let (service, store) = observed();
    let (mut client, task) = start(service, true, 4096).await;
    idle(&mut client).await;
    store.slow.store(true, Ordering::SeqCst);
    wait_for(|| store.entered.load(Ordering::SeqCst)).await;
    // Single-thread Tokio runtime: the observed store also asserts its thread identity.
    let (healthy, _) = super::service();
    let (mut other, other_task) = start(healthy, false, 4096).await;
    assert!(command(&mut other, "next NOOP").await.contains("next OK"));
    command(&mut other, "z LOGOUT").await;
    other_task.await.unwrap().unwrap();
    drop(client);
    let _ = tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn idle_backpressure_stops_more_store_snapshots_and_peer_drop_releases_session() {
    let (service, store) = observed();
    let commands = (0..200)
        .map(|_| {
            store
                .inner
                .append(
                    "a",
                    vec!["inbox".into()],
                    b"Subject: idle\r\n\r\nbody",
                    vec![],
                    1,
                )
                .unwrap()
        })
        .collect();
    let account = store
        .inner
        .execute("a", Precondition::Observed(0), commands)
        .map(|_| store.inner.account("a").unwrap())
        .unwrap();
    let (mut client, task) = start(service, true, 128).await;
    idle(&mut client).await;
    let before = store.reads.load(Ordering::SeqCst);
    store
        .inner
        .execute(
            "a",
            Precondition::Observed(account.revision),
            account
                .messages
                .iter()
                .map(|m| Command::Keywords {
                    id: m.id.clone(),
                    keywords: vec!["$seen".into()],
                })
                .collect(),
        )
        .unwrap();
    mail_service::notify::signal("a");
    wait_for(|| store.reads.load(Ordering::SeqCst) > before).await;
    // Hints keep arriving while the client stalls; none may start a refresh.
    for _ in 0..3 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        mail_service::notify::signal("a");
    }
    assert!(
        store.reads.load(Ordering::SeqCst) <= before + 2,
        "a stalled reader must prevent accumulating snapshots"
    );
    drop(client);
    let _ = tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap();
}
