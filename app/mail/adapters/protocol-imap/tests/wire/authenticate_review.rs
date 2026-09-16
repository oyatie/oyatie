use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};

type Client = BufReader<tokio::io::DuplexStream>;
type Task = tokio::task::JoinHandle<std::io::Result<()>>;

async fn until(client: &mut Client, prefix: &str) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut output = String::new();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0, "{output}");
            let done = line.starts_with(prefix);
            output.push_str(&line);
            if done {
                return output;
            }
        }
    })
    .await
    .expect("independent SASL response deadline")
}

async fn command(client: &mut Client, text: &str, tag: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{text}\r\n").as_bytes())
        .await
        .unwrap();
    until(client, tag).await
}

async fn start(service: Arc<MailService>, encrypted: bool) -> (Client, Task) {
    let (client, server) = tokio::io::duplex(32768);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, encrypted));
    let mut client = BufReader::new(client);
    until(&mut client, "* OK").await;
    (client, task)
}

async fn finish(mut client: Client, task: Task) {
    command(&mut client, "z LOGOUT", "z ").await;
    task.await.unwrap().unwrap();
}

fn response() -> String {
    STANDARD.encode(format!("\0alice@example.org\0{TOKEN}"))
}

struct Policy {
    runtime_thread: std::thread::ThreadId,
    allowed: AtomicBool,
    calls: AtomicUsize,
}
impl mail_api::Policy for Policy {
    fn authorize(
        &self,
        _: &mail_api::Principal,
        action: mail_api::Action,
        _: &mail_api::AccountInfo,
    ) -> Result<(), mail_kernel::Error> {
        assert_ne!(
            std::thread::current().id(),
            self.runtime_thread,
            "policy ran on socket runtime"
        );
        self.calls.fetch_add(1, Ordering::SeqCst);
        if action == mail_api::Action::Read && self.allowed.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(mail_kernel::Error::Forbidden)
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn sasl_uses_current_read_policy_off_socket_runtime_after_challenge() {
    let (_, db) = service();
    let policy = Arc::new(Policy {
        runtime_thread: std::thread::current().id(),
        allowed: AtomicBool::new(true),
        calls: AtomicUsize::new(0),
    });
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: policy.clone(),
    });
    let (mut client, task) = start(service, true).await;
    assert_eq!(
        command(&mut client, "a AUTHENTICATE PLAIN", "+").await,
        "+ \r\n"
    );
    assert_eq!(policy.calls.load(Ordering::SeqCst), 0);
    policy.allowed.store(false, Ordering::SeqCst);
    assert!(
        command(&mut client, &response(), "a ")
            .await
            .contains("a NO")
    );
    assert!(policy.calls.load(Ordering::SeqCst) > 0);
    assert!(
        command(&mut client, "s SELECT INBOX", "s ")
            .await
            .contains("s NO")
    );
    policy.allowed.store(true, Ordering::SeqCst);
    assert!(
        command(
            &mut client,
            &format!("b AUTHENTICATE PLAIN {}", response()),
            "b "
        )
        .await
        .contains("b OK")
    );
    assert!(
        command(&mut client, "s SELECT INBOX", "s ")
            .await
            .contains("s OK [READ-ONLY]")
    );
    finish(client, task).await;
}

#[tokio::test]
async fn sasl_revoked_token_between_challenge_and_response_never_authenticates() {
    let (service, db) = service();
    let (mut client, task) = start(service, true).await;
    command(&mut client, "a AUTHENTICATE PLAIN", "+").await;
    db.revoke("a").unwrap();
    let reply = command(
        &mut client,
        &format!("{}\r\ns SELECT INBOX", response()),
        "s ",
    )
    .await;
    assert!(reply.contains("a NO") && reply.contains("s NO"), "{reply}");
    assert!(!reply.contains(TOKEN), "{reply}");
    finish(client, task).await;
}

#[tokio::test]
async fn sasl_truncated_or_oversized_response_closes_without_executing_response_as_commands() {
    for oversized in [false, true] {
        let (service, _) = service();
        let (mut client, task) = start(service, true).await;
        command(&mut client, "a AUTHENTICATE PLAIN", "+").await;
        let text = if oversized {
            format!("{}\r\ninjected LOGOUT\r\n", "A".repeat(16384))
        } else {
            response()
        };
        client.get_mut().write_all(text.as_bytes()).await.unwrap();
        if !oversized {
            client.get_mut().shutdown().await.unwrap();
        }
        let mut reply = String::new();
        tokio::time::timeout(Duration::from_secs(5), client.read_to_string(&mut reply))
            .await
            .unwrap()
            .unwrap();
        assert!(
            !reply.contains("a OK") && !reply.contains("injected OK"),
            "{reply}"
        );
        let _ = task.await.unwrap();
    }
}

#[tokio::test]
async fn sasl_strict_base64_and_plain_field_boundaries_preserve_next_command() {
    let (service, db) = service();
    let before = db.account("a").unwrap();
    let (mut client, task) = start(service, true).await;
    for encoded in [
        STANDARD.encode(b"\0alice@example.org\0\xff"),
        STANDARD.encode(format!("\0alice@example.org\0{TOKEN}\0")),
        STANDARD.encode(format!("bob@example.org\0alice@example.org\0{TOKEN}")),
        format!("{}!", response()),
        "injected LOGOUT".to_owned(),
        "*".to_owned(),
        "=".to_owned(),
    ] {
        command(&mut client, "a AUTHENTICATE PLAIN", "+").await;
        let reply = command(&mut client, &format!("{encoded}\r\nnext NOOP"), "next ").await;
        assert!(reply.contains("a BAD") || reply.contains("a NO"), "{reply}");
        assert!(
            reply.contains("next OK") && !reply.contains("injected OK"),
            "{reply}"
        );
        assert!(!reply.contains(TOKEN), "{reply}");
        assert!(
            command(&mut client, "s SELECT INBOX", "s ")
                .await
                .contains("s NO")
        );
    }
    assert_eq!(db.account("a").unwrap(), before);
    finish(client, task).await;
}

#[tokio::test]
async fn sasl_plaintext_inline_response_and_reauthentication_do_not_change_identity() {
    for encrypted in [false, true] {
        let (service, db) = service();
        db.provision(
            Account::new("b", "other-tenant", "bob", "bob@example.org").unwrap(),
            "fedcba9876543210fedcba9876543210",
        )
        .unwrap();
        let (mut client, task) = start(service, encrypted).await;
        let first = command(
            &mut client,
            &format!("a AUTHENTICATE PLAIN {}", response()),
            "a ",
        )
        .await;
        assert_eq!(first.contains("a OK"), encrypted, "{first}");
        let bob = STANDARD.encode("\0bob@example.org\0fedcba9876543210fedcba9876543210");
        let second = command(
            &mut client,
            &format!("b AUTHENTICATE PLAIN {bob}\r\ns SELECT INBOX"),
            "s ",
        )
        .await;
        assert!(
            !second.contains("b OK") && !second.contains("+ \r\n"),
            "{second}"
        );
        assert_eq!(second.contains("s OK"), encrypted, "{second}");
        if encrypted {
            db.revoke("a").unwrap();
            assert!(command(&mut client, "n NOOP", "n ").await.contains("n NO"));
        }
        finish(client, task).await;
    }
}

#[tokio::test]
async fn sasl_stalled_response_releases_connection_at_the_wire_deadline() {
    let (service, _) = service();
    let (mut client, task) = start(service, true).await;
    command(&mut client, "a AUTHENTICATE PLAIN", "+").await;
    let mut reply = String::new();
    tokio::time::timeout(Duration::from_secs(65), client.read_to_string(&mut reply))
        .await
        .unwrap()
        .unwrap();
    assert!(!reply.contains("a OK"), "{reply}");
    assert_eq!(
        task.await.unwrap().unwrap_err().kind(),
        std::io::ErrorKind::TimedOut
    );
}
