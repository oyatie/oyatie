use super::*;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

type Client = BufReader<tokio::io::DuplexStream>;
type Task = tokio::task::JoinHandle<std::io::Result<()>>;

struct Writable(AtomicBool);
impl mail_api::Policy for Writable {
    fn authorize(
        &self,
        _: &mail_api::Principal,
        action: mail_api::Action,
        _: &mail_api::AccountInfo,
    ) -> Result<(), mail_kernel::Error> {
        if action == mail_api::Action::Read || self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(mail_kernel::Error::Forbidden)
        }
    }
}

async fn until(client: &mut Client, prefix: &str) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut response = String::new();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0, "{response}");
            let done = line.starts_with(prefix);
            response.push_str(&line);
            if done {
                return response;
            }
        }
    })
    .await
    .expect("literal resource review deadline")
}

async fn start(quota: usize) -> (Client, Task, Arc<SqliteStore>, Arc<Writable>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.quota_bytes = quota;
    db.provision(account, TOKEN).unwrap();
    let policy = Arc::new(Writable(AtomicBool::new(true)));
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: policy.clone(),
    });
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    until(&mut client, "* OK").await;
    client
        .get_mut()
        .write_all(format!("a LOGIN alice@example.org {TOKEN}\r\n").as_bytes())
        .await
        .unwrap();
    assert!(until(&mut client, "a ").await.contains("a OK"));
    (client, task, db, policy)
}

async fn finish(mut client: Client, task: Task) {
    client.get_mut().write_all(b"z LOGOUT\r\n").await.unwrap();
    until(&mut client, "z ").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn append_handoff_preserves_large_binary_body_after_quote_bearing_mailbox_literal() {
    for nonsync in [false, true] {
        let (mut client, task, db, _) = start(65536).await;
        let mailbox = "Folder \"quoted\" {curly}";
        db.execute(
            "a",
            0,
            vec![mail_kernel::Command::CreateMailbox {
                name: mailbox.into(),
            }],
        )
        .unwrap();
        let mut raw = b"Subject: binary\r\n\r\n".to_vec();
        raw.extend(std::iter::repeat_n(0xff, 10000));
        raw.extend_from_slice(b"\r\ninjected LOGOUT\r\n");
        let plus = if nonsync { "+" } else { "" };
        client
            .get_mut()
            .write_all(
                format!(
                    "b APPEND {{{}+}}\r\n{mailbox} {{{}{plus}}}\r\n",
                    mailbox.len(),
                    raw.len()
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        if !nonsync {
            assert!(until(&mut client, "+").await.starts_with('+'));
        }
        client.get_mut().write_all(&raw).await.unwrap();
        client
            .get_mut()
            .write_all(b"\r\nnext NOOP\r\n")
            .await
            .unwrap();
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains("b OK [APPENDUID") && response.contains("next OK"),
            "{response}"
        );
        assert!(!response.contains("injected OK"), "{response}");
        let account = db.account("a").unwrap();
        assert_eq!(db.blob("a", &account.messages[0].id).unwrap(), raw);
        let target = account
            .mailboxes
            .iter()
            .find(|m| m.name == mailbox)
            .unwrap();
        assert_eq!(
            account.messages[0].mailboxes.keys().collect::<Vec<_>>(),
            [&target.id]
        );
        finish(client, task).await;
    }
}

#[tokio::test]
async fn append_rechecks_policy_and_quota_between_mailbox_and_body_literals() {
    for quota_refusal in [false, true] {
        let (mut client, task, db, policy) = start(if quota_refusal { 4 } else { 65536 }).await;
        client
            .get_mut()
            .write_all(b"b APPEND {5}\r\n")
            .await
            .unwrap();
        assert!(until(&mut client, "+").await.starts_with('+'));
        if !quota_refusal {
            policy.0.store(false, Ordering::SeqCst);
        }
        client
            .get_mut()
            .write_all(b"INBOX {32}\r\nnext NOOP\r\n")
            .await
            .unwrap();
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains("b NO") && response.contains("next OK"),
            "{response}"
        );
        if quota_refusal {
            assert!(response.contains("[OVERQUOTA]"), "{response}");
        }
        assert!(!response.contains("+ Ready"), "{response}");
        assert!(db.account("a").unwrap().messages.is_empty());
        finish(client, task).await;
    }
}

#[tokio::test]
async fn denied_copy_and_move_refuse_before_destination_literal_continuation() {
    for verb in ["COPY", "MOVE", "UID COPY", "UID MOVE"] {
        let (mut client, task, db, policy) = start(65536).await;
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: retained\r\n\r\nbody",
        )
        .unwrap();
        client
            .get_mut()
            .write_all(b"s SELECT INBOX\r\n")
            .await
            .unwrap();
        until(&mut client, "s ").await;
        policy.0.store(false, Ordering::SeqCst);
        client
            .get_mut()
            .write_all(format!("b {verb} 1 {{5}}\r\n").as_bytes())
            .await
            .unwrap();
        let mut response = String::new();
        tokio::time::timeout(Duration::from_secs(5), client.read_line(&mut response))
            .await
            .unwrap()
            .unwrap();
        assert!(
            response.starts_with("b NO") || response.starts_with("b BAD"),
            "{verb}: {response}"
        );
        assert_eq!(db.account("a").unwrap().messages.len(), 1);
        finish(client, task).await;
    }
}
