use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::time::Duration;

type Client = BufReader<tokio::io::DuplexStream>;

async fn response(client: &mut Client, prefix: &str) -> String {
    tokio::time::timeout(Duration::from_secs(3), async {
        let mut result = String::new();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0, "{result}");
            let done = line.starts_with(prefix);
            result.push_str(&line);
            if done {
                return result;
            }
        }
    })
    .await
    .expect("authentication response deadline")
}

async fn command(client: &mut Client, input: &str, prefix: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{input}\r\n").as_bytes())
        .await
        .unwrap();
    response(client, prefix).await
}

async fn start(
    encrypted: bool,
) -> (
    Client,
    tokio::task::JoinHandle<std::io::Result<()>>,
    Arc<SqliteStore>,
) {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(16384);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, encrypted));
    let mut client = BufReader::new(client);
    response(&mut client, "* OK").await;
    (client, task, db)
}

fn plain(authz: &str, user: &str, token: &str) -> String {
    STANDARD.encode(format!("{authz}\0{user}\0{token}"))
}

#[tokio::test]
async fn sasl_plain_supports_initial_response_and_continuation_with_same_read_policy() {
    for initial in [false, true] {
        let (mut client, task, db) = start(true).await;
        let capability = command(&mut client, "c CAPABILITY", "c ").await;
        assert!(
            capability.contains("AUTH=PLAIN") && capability.contains("SASL-IR"),
            "{capability}"
        );
        let encoded = plain("", "ALICE@example.org", TOKEN);
        let result = if initial {
            command(
                &mut client,
                &format!("a AUTHENTICATE PLAIN {encoded}"),
                "a ",
            )
            .await
        } else {
            assert_eq!(
                command(&mut client, "a AUTHENTICATE PLAIN", "+").await,
                "+ \r\n"
            );
            command(&mut client, &encoded, "a ").await
        };
        assert!(result.contains("a OK"), "{result}");
        assert!(
            command(&mut client, "s SELECT INBOX", "s ")
                .await
                .contains("s OK")
        );
        db.revoke("a").unwrap();
        assert!(command(&mut client, "r NOOP", "r ").await.contains("r NO"));
        command(&mut client, "z LOGOUT", "z ").await;
        task.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn sasl_never_grants_authorization_identity_impersonation_or_wrong_account_access() {
    let (mut client, task, db) = start(true).await;
    let before = db.account("a").unwrap();
    for encoded in [
        plain("bob@example.org", "alice@example.org", TOKEN),
        plain("", "bob@example.org", TOKEN),
        plain("", "alice@example.org", "wrong"),
        STANDARD.encode(format!("\0alice@example.org\0{TOKEN}\0extra")),
        STANDARD.encode("\0\0token"),
        "====".into(),
        "=".into(),
    ] {
        let result = command(
            &mut client,
            &format!("a AUTHENTICATE PLAIN {encoded}"),
            "a ",
        )
        .await;
        assert!(
            result.contains("a NO") || result.contains("a BAD"),
            "{result}"
        );
        assert!(
            !result.contains(TOKEN) && !result.contains(&encoded),
            "{result}"
        );
        assert!(
            command(&mut client, "s SELECT INBOX", "s ")
                .await
                .contains("s NO")
        );
    }
    let encoded = plain("ALICE@example.org", "alice@example.org", TOKEN);
    assert!(
        command(
            &mut client,
            &format!("a AUTHENTICATE PLAIN {encoded}"),
            "a "
        )
        .await
        .contains("a OK")
    );
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT", "z ").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn sasl_cancel_and_invalid_response_preserve_the_following_command() {
    let (mut client, task, _) = start(true).await;
    for answer in ["*", "injected LOGOUT", "!notbase64!"] {
        assert_eq!(
            command(&mut client, "a AUTHENTICATE PLAIN", "+").await,
            "+ \r\n"
        );
        let result = command(&mut client, &format!("{answer}\r\nnext NOOP"), "next ").await;
        assert!(
            result.contains("a BAD") && result.contains("next OK"),
            "{result}"
        );
        assert!(!result.contains("injected OK"), "{result}");
    }
    assert!(
        command(
            &mut client,
            &format!("a LOGIN alice@example.org {TOKEN}"),
            "a "
        )
        .await
        .contains("a OK")
    );
    assert!(
        command(&mut client, "b AUTHENTICATE PLAIN", "b ")
            .await
            .contains("b BAD")
    );
    command(&mut client, "z LOGOUT", "z ").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn sasl_requires_tls_and_refuses_unsupported_mechanisms_without_continuation() {
    for encrypted in [false, true] {
        let (mut client, task, _) = start(encrypted).await;
        let capability = command(&mut client, "c CAPABILITY", "c ").await;
        assert_eq!(capability.contains("AUTH=PLAIN"), encrypted);
        for input in if encrypted {
            vec![
                "a AUTHENTICATE UNKNOWN",
                "a AUTHENTICATE",
                "a AUTHENTICATE PLAIN a b",
                "a AUTHENTICATE \"PLAIN\"",
            ]
        } else {
            vec!["a AUTHENTICATE PLAIN"]
        } {
            let result = command(&mut client, &format!("{input}\r\nnext NOOP"), "next ").await;
            assert!(
                result.contains("a BAD") || result.contains("a NO"),
                "{result}"
            );
            assert!(!result.contains("+ \r\n"), "{result}");
        }
        command(&mut client, "z LOGOUT", "z ").await;
        task.await.unwrap().unwrap();
    }
}
