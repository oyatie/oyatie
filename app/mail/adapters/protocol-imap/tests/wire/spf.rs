//! SPF on the wire: which host a domain authorizes, what a sealed table can
//! and cannot answer, and the cost a re-greeting client is allowed to impose.
use super::*;

/// SPF is decided about the address the peer actually connected from, and
/// `strict` answers only the domain's own `-all` refusal. The table is sealed,
/// so a name nobody seeded is an authoritative "no record" and no lookup can
/// leave the process.
#[tokio::test]
async fn spf_strict_refuses_only_the_host_the_domain_disowns() {
    use mail_auth::{
        MessageAuthenticator,
        common::parse::TxtRecordParser,
        hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts},
        spf::Spf,
    };
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
#[cfg(test)]
fn nowhere() -> mail_auth::hickory_resolver::config::ResolverConfig {
    use mail_auth::hickory_resolver::config::{NameServerConfig, ResolverConfig};
    let mut server = NameServerConfig::udp_and_tcp(std::net::Ipv4Addr::LOCALHOST.into());
    for connection in &mut server.connections {
        connection.port = 1;
    }
    ResolverConfig::from_name_servers(vec![server])
}

#[cfg(test)]
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
