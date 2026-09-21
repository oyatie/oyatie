//! What a receiver makes of these bytes, rather than what they look like. A
//! signature with correct syntax over the wrong hash passes every shape
//! assertion and is rejected on arrival, so the key that signed the message is
//! published to a sealed table and `mail-auth`'s own verifier is asked.
//!
//! The resolver behind the table points at a closed loopback port: every TXT
//! lookup is answered from the table, and nothing else can leave the host.
use super::*;
use mail_auth::{
    AuthenticatedMessage, DkimResult, MessageAuthenticator, Parameters,
    common::{parse::TxtRecordParser, verify::DomainKey},
    dkim::DkimError,
};

#[test]
fn the_signature_names_this_domain_and_leaves_the_message_unaltered() {
    let file = key_file("shape");
    let signer = Signer::from_key_file(
        &file.display().to_string(),
        "example.org".to_owned(),
        "default".to_owned(),
    )
    .expect("a 2048-bit key is a signer");
    let raw = b"From: alice@example.org\r\nTo: bob@example.net\r\nSubject: signed\r\n\r\nbody\r\n";
    let signed = signer.signed(raw).expect("a 2048-bit key signs");
    let text = String::from_utf8(signed.clone()).unwrap();
    assert!(text.starts_with("DKIM-Signature: "), "{text}");
    for field in [
        "v=1",
        "a=rsa-sha256",
        "d=example.org",
        "s=default",
        "c=simple/relaxed",
    ] {
        assert!(text.contains(field), "missing {field}: {text}");
    }
    // The header is prepended, so the bytes a receiver verifies are the
    // bytes that were signed.
    assert!(signed.ends_with(raw), "{text}");
    // Folding included, every line ends CRLF and stays inside the
    // 1000-byte limit the outbound path enforces -- with the longest
    // selector and domain the configuration permits, too, since those two
    // are the only part of that first line nothing folds.
    let file = key_file("longest");
    let longest = Signer::from_key_file(
        &file.display().to_string(),
        longest_sub_domain(),
        longest_sub_domain(),
    )
    .expect("a 2048-bit key is a signer");
    for text in [
        text,
        String::from_utf8(longest.signed(raw).unwrap()).unwrap(),
    ] {
        for line in text.split_inclusive("\r\n") {
            assert!(line.ends_with("\r\n"), "unterminated line: {line:?}");
            assert!(line.len() <= 1000, "line too long: {}", line.len());
        }
    }
}

#[tokio::test]
async fn a_signed_message_verifies_and_one_altered_byte_of_body_stops_it() {
    let file = key_file("verify");
    let signer = Signer::from_key_file(
        &file.display().to_string(),
        "example.org".to_owned(),
        "default".to_owned(),
    )
    .expect("a 2048-bit key is a signer");
    let signed = signer
        .signed(
            b"From: alice@example.org\r\nTo: bob@example.net\r\nSubject: signed\r\n\r\nbody\r\n",
        )
        .expect("a 2048-bit key signs");
    let (authenticator, dns) = receiver();
    assert_eq!(
        verdict(&authenticator, &dns, &signed).await,
        DkimResult::Pass
    );

    // One letter of the body -- not whitespace, which relaxed canonicalization
    // is meant to absorb. Without this half the test cannot tell a real
    // verification from one that always says yes.
    let mut tampered = signed.clone();
    let at = tampered.len() - 6;
    assert_eq!(tampered[at], b'b', "the byte to alter is the body's first");
    tampered[at] = b'B';
    let refused = verdict(&authenticator, &dns, &tampered).await;
    assert!(
        matches!(
            refused,
            DkimResult::Neutral(mail_auth::Error::Dkim(DkimError::FailedBodyHashMatch))
        ),
        "an altered body must fail the body hash: {refused:?}"
    );
}

/// RFC 6376 section 8.15: a relay may *add* a header the sender never wrote,
/// and a verifier that reads each name once from the bottom of the message
/// still finds the original underneath it -- the signature passes while the
/// recipient is shown the addition. Naming each header twice signs one absent
/// instance as empty, so a second `Cc` is a header the signature did not
/// cover, and verification stops.
///
/// A `Cc` added where there was none is already refused without the second
/// listing, because `mail-auth` writes every configured name into `h=` whether
/// or not the message carries it. The second instance is the case that the
/// repetition, and only the repetition, catches -- so that is what is asserted
/// here, with the absent-header case kept beside it as the weaker half.
#[tokio::test]
async fn a_header_a_relay_adds_after_signing_no_longer_verifies() {
    let file = key_file("oversign");
    let signer = Signer::from_key_file(
        &file.display().to_string(),
        "example.org".to_owned(),
        "default".to_owned(),
    )
    .expect("a 2048-bit key is a signer");
    let carries_a_cc =
        b"From: alice@example.org\r\nTo: bob@example.net\r\nCc: dave@example.net\r\n\r\nbody\r\n";
    let no_cc = b"From: alice@example.org\r\nTo: bob@example.net\r\n\r\nbody\r\n";
    let (authenticator, dns) = receiver();
    for raw in [carries_a_cc.as_slice(), no_cc.as_slice()] {
        let signed = signer.signed(raw).expect("a 2048-bit key signs");
        assert_eq!(
            verdict(&authenticator, &dns, &signed).await,
            DkimResult::Pass
        );
        // Where a relay puts one: above everything the sender wrote, so a
        // verifier reading from the bottom would still find the original.
        let at = signed
            .windows(6)
            .position(|window| window == b"From: ")
            .expect("the signed message carries a From header");
        let mut relayed = signed.clone();
        relayed.splice(at..at, *b"Cc: eve@example.net\r\n");
        let refused = verdict(&authenticator, &dns, &relayed).await;
        assert_ne!(
            refused,
            DkimResult::Pass,
            "a Cc the sender never wrote must not verify: {refused:?}"
        );
    }
}

/// The one verdict `mail-auth` reaches about these bytes. A message carrying
/// no verdict at all would satisfy `all(Pass)` vacuously, so the count is
/// asserted rather than the iterator.
async fn verdict(
    authenticator: &MessageAuthenticator,
    dns: &mail_protocol_imap::MailDns,
    raw: &[u8],
) -> DkimResult {
    let message = AuthenticatedMessage::parse(raw).expect("the signed bytes parse");
    let output = authenticator
        .verify_dkim(Parameters::new(&message).with_txt_cache(dns))
        .await;
    assert_eq!(output.len(), 1, "one signature, one verdict");
    output[0].result().clone()
}

/// A receiver that has the published key and no way to ask for anything else:
/// `mail-auth` looks the key up at `<selector>._domainkey.<domain>`, the
/// sealed table answers that, and a lookup it declines fails here rather than
/// on the wire.
fn receiver() -> (MessageAuthenticator, mail_protocol_imap::MailDns) {
    use mail_auth::hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
    let dns = mail_protocol_imap::MailDns::sealed();
    dns.txt_add(
        "default._domainkey.example.org",
        DomainKey::parse(format!("v=DKIM1; k=rsa; p={TEST_PUBLIC_KEY}").as_bytes())
            .expect("the published record parses"),
    );
    let mut server = NameServerConfig::udp_and_tcp(std::net::Ipv4Addr::LOCALHOST.into());
    for connection in &mut server.connections {
        connection.port = 1;
    }
    let mut options = ResolverOpts::default();
    options.attempts = 0;
    options.timeout = std::time::Duration::from_millis(50);
    let authenticator =
        MessageAuthenticator::new(ResolverConfig::from_name_servers(vec![server]), options)
            .expect("a resolver pointed nowhere");
    (authenticator, dns)
}
