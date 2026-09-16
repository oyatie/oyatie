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
