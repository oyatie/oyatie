use super::*;

#[test]
fn invalid_outbound_configuration_fails_closed_without_credentials_in_diagnostics() {
    let fixture = Fixture::new();
    let cases = [
        (vec![("MAIL_MX_REQUIRE_TLS", "true")], "MAIL_MX_DNS_SERVERS"),
        (
            vec![("MAIL_MX_DNS_SERVERS", "127.0.0.1:53")],
            "MAIL_MX_HELO",
        ),
        (vec![("MAIL_MX_DNS_SERVERS", "")], "invalid socket address"),
        (
            vec![("MAIL_MX_DNS_SERVERS", "127.0.0.1:53,")],
            "invalid socket address",
        ),
        (
            vec![
                ("MAIL_MX_DNS_SERVERS", "127.0.0.1:53"),
                ("MAIL_MX_HELO", "out.example"),
                ("MAIL_MX_REQUIRE_TLS", "TRUE"),
            ],
            "must be true or false",
        ),
        (
            vec![
                ("MAIL_MX_DNS_SERVERS", "0.0.0.0:53"),
                ("MAIL_MX_HELO", "out.example"),
            ],
            "Invalid",
        ),
        (
            vec![
                ("MAIL_MX_DNS_SERVERS", "127.0.0.1:53"),
                ("MAIL_MX_HELO", "bad\r\nMAIL"),
            ],
            "Invalid",
        ),
        (
            vec![
                ("MAIL_MX_DNS_SERVERS", "127.0.0.1:53"),
                ("MAIL_MX_HELO", "out.example"),
                ("MAIL_MX_PORT", "0"),
            ],
            "Invalid",
        ),
        (
            vec![
                ("MAIL_MX_DNS_SERVERS", "127.0.0.1:53"),
                ("MAIL_RELAY_PASSWORD", "private-review-password"),
            ],
            "mutually exclusive",
        ),
        (
            vec![("MAIL_RELAY_PASSWORD", "private-review-password")],
            "MAIL_RELAY_HOST",
        ),
        (
            vec![
                ("MAIL_RELAY_HOST", "relay.example"),
                ("MAIL_RELAY_PASSWORD", "private-review-password"),
            ],
            "must both be supplied",
        ),
        // A check with nothing to verify against must be refused at startup,
        // not silently pass every message.
        (vec![("MAIL_SPF_EHLO", "strict")], "needs a resolver"),
        (
            vec![("MAIL_SPF_MAIL_FROM", "Strict")],
            "must be disable, relaxed or strict",
        ),
        // Signing half-configured would send unsigned mail while the operator
        // believed otherwise; the refusal names the setting that is missing.
        (
            vec![("MAIL_DKIM_DOMAIN", "example.org")],
            "MAIL_DKIM_KEY is required",
        ),
        (
            vec![
                ("MAIL_DKIM_KEY", "/nonexistent/private-review-key.pem"),
                ("MAIL_DKIM_DOMAIN", "example.org"),
            ],
            "MAIL_DKIM_SELECTOR is required",
        ),
        (
            vec![
                ("MAIL_DKIM_KEY", "/nonexistent/private-review-key.pem"),
                ("MAIL_DKIM_DOMAIN", "example.org"),
                ("MAIL_DKIM_SELECTOR", "default"),
            ],
            "not a readable PEM private key",
        ),
    ];
    for (settings, expected) in cases {
        let mut command = fixture.command();
        if settings
            .iter()
            .any(|(name, _)| name.starts_with("MAIL_MX_"))
        {
            command.env("MAIL_MX_CA", &fixture.cert);
        }
        command.envs(settings);
        let mut server = Server(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(10);
        while server.0.try_wait().unwrap().is_none() {
            assert!(
                Instant::now() < deadline,
                "invalid configuration started server"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!server.0.wait().unwrap().success());
        let mut diagnostic = String::new();
        server
            .0
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut diagnostic)
            .unwrap();
        assert!(
            diagnostic.contains(expected),
            "expected {expected}: {diagnostic}"
        );
        assert!(!diagnostic.contains("private-review-password"));
        assert!(!diagnostic.contains("mail-app: SMTP"));
    }
}

/// A refused `provision` must name the cause. The store answers both an
/// existing account and a token already in use with `Conflict`; an operator
/// reusing one token across accounts reads the bare word as "the account
/// already exists" and edits the wrong thing.
#[test]
fn provision_refusal_names_both_causes_of_a_conflict() {
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("mail.db");
    let provision = |account: &str, address: &str, token: &str| {
        let binary = option_env!("MAIL_APP_BINARY")
            .or(option_env!("CARGO_BIN_EXE_mail-app"))
            .expect("the build must provide the mail-app executable");
        std::process::Command::new(binary)
            .arg("provision")
            .arg(&db)
            .args(["acme", account, account, address])
            .env("MAIL_TOKEN", token)
            .output()
            .unwrap()
    };
    let alice = "alice-token-0123456789abcdef0123456789";
    let first = provision("alice", "alice@example.org", alice);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    // Same token, different account: the UNIQUE token constraint.
    let reused = provision("bob", "bob@example.org", alice);
    assert!(!reused.status.success());
    let text = String::from_utf8_lossy(&reused.stderr).into_owned();
    assert!(text.contains("MAIL_TOKEN is already in use"), "{text}");
    assert!(text.contains("already provisioned"), "{text}");

    // Same account again, fresh token: the id/address check.
    let repeat = provision(
        "alice",
        "alice@example.org",
        "another-token-0123456789abcdef01234",
    );
    assert!(!repeat.status.success());
    let text = String::from_utf8_lossy(&repeat.stderr).into_owned();
    assert!(text.contains("already provisioned"), "{text}");
}
