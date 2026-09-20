//! SPF on the wire: which host a domain authorizes, what a sealed table can
//! and cannot answer, and the cost a re-greeting client is allowed to impose.
use super::*;

/// SPF is decided about the address the peer actually connected from, and
/// `strict` answers only the domain's own `-all` refusal. The table is sealed
/// and its resolver points nowhere, so an unseeded name is decided here or
/// not at all — the seal alone does not bound a run.
#[tokio::test]
async fn spf_strict_refuses_only_the_host_the_domain_disowns() {
    use mail_auth::{MessageAuthenticator, common::parse::TxtRecordParser, spf::Spf};
    let dns = std::sync::Arc::new(mail_protocol_imap::MailDns::sealed());
    dns.txt_add(
        "mx1.example.org",
        Spf::parse(b"v=spf1 ip4:10.0.0.1 -all").unwrap(),
    );
    // `exists:` asks whether a name resolves at all. A sealed table that
    // answered its record-set misses would assert that every unseeded name
    // exists, and this `-all` record would pass instead of refusing.
    dns.txt_add(
        "mx2.example.org",
        Spf::parse(b"v=spf1 exists:nothing.example -all").unwrap(),
    );
    let authentication = mail_protocol_imap::Authentication {
        verifier: Some(mail_protocol_imap::Verifier(std::sync::Arc::new(
            // A sealed table declines the record-set lookups, so those still
            // reach the resolver. Point it at a closed loopback port: the
            // lookup fails immediately and no packet leaves the host.
            MessageAuthenticator::new(nowhere(), refuse_fast()).unwrap(),
        ))),
        dns: Some(dns),
        spf_ehlo: mail_protocol_imap::Verify::Strict,
        ..Default::default()
    };
    // .1 is the one address the record authorizes; .2 is covered by `-all`;
    // the unseeded domain has no record at all, which is not a refusal.
    for (peer, domain, expected, refused) in [
        ([10, 0, 0, 1], "mx1.example.org", "250-", false),
        ([10, 0, 0, 2], "mx1.example.org", "550 5.7.23", true),
        ([10, 0, 0, 2], "unseeded.example.org", "250-", false),
        // A sealed table cannot answer `exists:` -- the record-set tables
        // decline, so the lookup reaches a resolver that is pointed nowhere.
        // The verdict is genuinely unknown, and strict says so rather than
        // guessing. P4-2b needs a stub nameserver answering real NXDOMAIN if
        // it binds a suite whose records use `exists:`.
        ([10, 0, 0, 2], "mx2.example.org", "451 4.4.3", true),
    ] {
        let (service, _db) = service();
        let params = mail_protocol_imap::SmtpParams {
            peer: std::net::IpAddr::from(peer),
            authentication: authentication.clone(),
            ..mail_protocol_imap::SmtpParams::default()
        };
        let (mut client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(async move {
            mail_protocol_imap::smtp_session_with(server, service, &params).await
        });
        client
            .write_all(format!("EHLO {domain}\r\nQUIT\r\n").as_bytes())
            .await
            .unwrap();
        let mut result = String::new();
        client.read_to_string(&mut result).await.unwrap();
        task.await.unwrap().unwrap();
        assert!(result.contains(expected), "{peer:?} {domain}: {result}");
        assert_eq!(
            result.contains("250-"),
            !refused,
            "a refused session must not also be greeted: {peer:?} {domain}: {result}"
        );
    }
}

/// A resolver aimed at a closed loopback port: every lookup it is asked to
/// make fails at once, and nothing leaves the host.
fn nowhere() -> mail_auth::hickory_resolver::config::ResolverConfig {
    use mail_auth::hickory_resolver::config::{NameServerConfig, ResolverConfig};
    let mut server = NameServerConfig::udp_and_tcp(std::net::Ipv4Addr::LOCALHOST.into());
    for connection in &mut server.connections {
        connection.port = 1;
    }
    ResolverConfig::from_name_servers(vec![server])
}

fn refuse_fast() -> mail_auth::hickory_resolver::config::ResolverOpts {
    let mut options = mail_auth::hickory_resolver::config::ResolverOpts::default();
    options.attempts = 0;
    options.timeout = std::time::Duration::from_millis(50);
    options
}

/// An EHLO is eight bytes and a verification is a chain of DNS lookups, so a
/// session that re-greets must not become an amplifier pointed at our own
/// resolver: a repeated domain is decided once, and a session that keeps
/// greeting with new ones is closed.
#[tokio::test]
async fn repeated_greetings_do_not_multiply_dns_work() {
    let dns = std::sync::Arc::new(mail_protocol_imap::MailDns::sealed());
    let authentication = mail_protocol_imap::Authentication {
        verifier: Some(mail_protocol_imap::Verifier(std::sync::Arc::new(
            mail_auth::MessageAuthenticator::new(nowhere(), refuse_fast()).unwrap(),
        ))),
        dns: Some(dns),
        spf_ehlo: mail_protocol_imap::Verify::Relaxed,
        ..Default::default()
    };
    let (service, _db) = service();
    let params = mail_protocol_imap::SmtpParams {
        peer: std::net::IpAddr::from([10, 0, 0, 1]),
        authentication,
        ..mail_protocol_imap::SmtpParams::default()
    };
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(async move {
        mail_protocol_imap::smtp_session_with(server, service, &params).await
    });
    // The same domain five times is one decision; five different ones exceed
    // what any honest client needs and end the session.
    let same = "EHLO mx1.example.org\r\n".repeat(5);
    let different: String = (0..5)
        .map(|n| format!("EHLO other{n}.example.org\r\n"))
        .collect();
    client
        .write_all(format!("{same}{different}").as_bytes())
        .await
        .unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    // One banner ends with 8BITMIME, so this counts greetings, not lines.
    assert_eq!(
        result.matches("250 8BITMIME").count(),
        // The repeated domain is decided once and greeted five times, then
        // three new domains exhaust the four evaluations a session gets.
        8,
        "{result}"
    );
    assert!(result.contains("421 4.7.0"), "{result}");
}

/// A refusal is a decision about the peer, not an event that happens once.
/// Repeating the greeting must reach the same answer: remembering only that a
/// domain had been decided, without remembering what was decided, lets a host
/// the domain disowns in by asking twice.
#[tokio::test]
async fn a_refused_greeting_is_still_refused_when_it_is_repeated() {
    use mail_auth::common::parse::TxtRecordParser;
    let dns = std::sync::Arc::new(mail_protocol_imap::MailDns::sealed());
    dns.txt_add(
        "mx1.example.org",
        mail_auth::spf::Spf::parse(b"v=spf1 ip4:10.0.0.1 -all").unwrap(),
    );
    let (service, db) = service();
    let params = mail_protocol_imap::SmtpParams {
        peer: std::net::IpAddr::from([10, 0, 0, 2]),
        authentication: mail_protocol_imap::Authentication {
            verifier: Some(mail_protocol_imap::Verifier(std::sync::Arc::new(
                mail_auth::MessageAuthenticator::new(nowhere(), refuse_fast()).unwrap(),
            ))),
            dns: Some(dns),
            spf_ehlo: mail_protocol_imap::Verify::Strict,
            ..Default::default()
        },
        ..mail_protocol_imap::SmtpParams::default()
    };
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(async move {
        mail_protocol_imap::smtp_session_with(server, service, &params).await
    });
    client
        .write_all(
            b"EHLO mx1.example.org\r\nEHLO mx1.example.org\r\nEHLO MX1.Example.ORG\r\nMAIL FROM:<s@example.net>\r\nQUIT\r\n",
        )
        .await
        .unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(result.matches("550 5.7.23").count(), 3, "{result}");
    assert!(!result.contains("250 8BITMIME"), "never greeted: {result}");
    assert!(!result.contains("Sender accepted"), "{result}");
    assert!(db.account("a").unwrap().messages.is_empty());
}

/// The verification budget belongs to verification. A deployment that has not
/// turned any check on must not acquire a new reason to close sessions.
#[tokio::test]
async fn the_verification_budget_does_not_bind_a_session_that_verifies_nothing() {
    let (service, _db) = service();
    let params = mail_protocol_imap::SmtpParams::default();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(async move {
        mail_protocol_imap::smtp_session_with(server, service, &params).await
    });
    let greetings: String = (0..9)
        .map(|n| format!("EHLO other{n}.example.org\r\n"))
        .collect();
    client
        .write_all(format!("{greetings}QUIT\r\n").as_bytes())
        .await
        .unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(result.matches("250 8BITMIME").count(), 9, "{result}");
    assert!(!result.contains("421 "), "{result}");
}

/// A connection carries as many messages as the client has to send. Bounding
/// verification by counting reverse paths, without noticing that each became a
/// message, cuts off a sender the domain explicitly authorizes.
#[tokio::test]
async fn a_connection_may_deliver_more_messages_than_its_verification_budget() {
    use mail_auth::common::parse::TxtRecordParser;
    let dns = std::sync::Arc::new(mail_protocol_imap::MailDns::sealed());
    dns.txt_add(
        "example.net",
        mail_auth::spf::Spf::parse(b"v=spf1 ip4:10.0.0.1 -all").unwrap(),
    );
    let (service, _db) = service();
    let params = mail_protocol_imap::SmtpParams {
        // The peer the record authorizes, so every verdict is a Pass.
        peer: std::net::IpAddr::from([10, 0, 0, 1]),
        verification_budget: 4,
        authentication: mail_protocol_imap::Authentication {
            verifier: Some(mail_protocol_imap::Verifier(std::sync::Arc::new(
                mail_auth::MessageAuthenticator::new(nowhere(), refuse_fast()).unwrap(),
            ))),
            dns: Some(dns),
            spf_mail_from: mail_protocol_imap::Verify::Strict,
            ..Default::default()
        },
        ..mail_protocol_imap::SmtpParams::default()
    };
    let (mut client, server) = tokio::io::duplex(262144);
    let task = tokio::spawn(async move {
        mail_protocol_imap::smtp_session_with(server, service, &params).await
    });
    // Ten messages over one connection, against a budget of four.
    let transactions: String = (0..10)
        .map(|n| {
            format!(
                "MAIL FROM:<s@example.net>\r\nRCPT TO:<alice@example.org>\r\nDATA\r\nSubject: m{n}\r\n\r\nbody\r\n.\r\n"
            )
        })
        .collect();
    client
        .write_all(format!("EHLO mx1.example.net\r\n{transactions}QUIT\r\n").as_bytes())
        .await
        .unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(result.matches("250 2.0.0 Queued").count(), 10, "{result}");
    assert!(
        !result.contains("421 "),
        "a delivering connection must not exhaust the budget: {result}"
    );
}
