use super::*;
use mail_auth::{
    AuthenticatedMessage, DkimResult, MessageAuthenticator, Parameters,
    common::{parse::TxtRecordParser, verify::DomainKey},
    dkim::DkimError,
};

/// 2048 bits, because aws-lc-rs refuses anything smaller and a shorter key
/// would fail only at signing time. Fixed rather than generated: a keygen
/// dependency costs about thirty crates to produce a value that never
/// varies. Test-only; it signs nothing that leaves here.
const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIEuwIBADANBgkqhkiG9w0BAQEFAASCBKUwggShAgEAAoIBAQDWJonSt4EOCvf7
1QbnWZ2Wa3umpCda/Ydha+uuvQ3wY4hJwBKbtctGwqTR+nM7h9VoKFTHftbQVsTm
irQ3MbsyYRiL1L9qGYBVTBGbgnXBaW4A8LyeKfUppDOj4pdaOeuMXOB9D2W2gNjO
7mEK8K1yMeMJ+j9ACgp2CqAI098Op3IOVGOrwXwujKWhIKrqb8EI82oOICbNAIEl
ePlP7JOwDHOMjxP9ICJ1m2uG0Wx/dzy23KccHkPJ6jwsyJfMWdXMFVpG5CYP8k3/
zTzINvYx5b5n5CdF3s1Ariyqy9dwZJp0xinV0+2wnuymZCqTUVSqRAxIpiGTVvpv
mNz2RR+BAgMBAAECggEADwhgh2azTAGQG+D93ZKoYc1EvlNqodQHQ1r4jekeh9/s
ysSNadnOnbZ/LHBI04Z7ABdIsEJioPheVRKqiO9YRTaUqwgxsah9nj87Qy/axUt5
2d4MV4v5dkVdDKWU21QSiWVhqtXAXZnY3lnUfRidDFWdKu+irgOmbVfcmRIKZ/RP
STNS+ho2tXkdon9xPGppmHkXijgli0ceFz+YZVtyeFTnwl1CYQirGzHccJnd+aKX
AO+dNXcBf58J1KNnGPzR+EtbAT3XX8Gaes/GktZWneV6OGU5joW1RC9uX9qlGS6H
gV1Tu6AKm4KzrWODXIzuXX5bhwIX+rfzIHKxLMj34wKBgQDr++pwyXOl8bWwZjRU
tfB4Yg0uGs92gRgJC92bKzpNti8a7EwqU5GZXH5Uf54b/yYjKGCMV5lctxgfY4yF
HGsbQSaFQZCRrfc1UML4TFtXwL5Qs/Tyo6/UIL66ICGKTuaKSwrXYj4sEPURS9ZZ
+JvmwSZ00I2DwfUsAXoxgYmILwKBgQDoUIlhAwNrsVg3xtAx0gXrQZzrcOXquqmm
5+eRLGl/cWZCKGYKK9JcQUOeDjj8cpiuTBenviK+sfd0JAIh215wziKmWytQLtfj
GewO3Mxf2tk9Pf7YRmiF4zbky4wr63Pz9NJTigz+OCwdLZTP9fraQf91SDS37SdG
duwGYg43TwKBgQCqoUVb9h3cAFDKUqxGECPnN6amDpax7hgN+nlrCC+pHzEiO3e7
Jx/hDyL9QCV3wt61jy14bKKkinMzxwxE905ur4YF1mmNHIfiEhpX5QYrBl/WBLj5
dOfe7ypZdIAr/G7v0eDt6chgnoZE0lwURBGeIU0ILAAXI/h9sDfFcg+a2wJ/LHc9
1FO/U92eBQ8IyoBooZ7taiMx3rvbvRamPCNEDiCmcgNJhKjemsnjJ12RisBkePgX
jwPVoqptss0xm0lhyjWqbC0HHVHaAJ31kOKyO6an7hDvtnXDi9zxpNlQ+xcWGpvB
pjvGQOJ/jxYqZaOvYBdBzjED5jB+U/5vLsmV7wKBgDI77NR1opuU6SpnejJK5fNs
jLzns+/wBP1r3B0q8dPL3eSuMoQhm3JWocFjXFD2YsEnMY9A0Z/M7ZY8h5ee0cK0
GULFnT1mkgy2BI04XYufq3OXwnslTSCno7HyOy5f3vr42sQ36aF+gCi7wMD3CrZb
uOv74OKArzXsO86qqdfO
-----END PRIVATE KEY-----";

/// The public half of `TEST_KEY`, base64 SPKI, exactly as a receiver reads it
/// out of the `p=` tag. Pasted beside the key rather than derived at runtime
/// for the same reason the key is fixed: deriving it costs a dependency to
/// produce a value that never varies.
const TEST_PUBLIC_KEY: &str = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1iaJ0reBDgr3+9UG51mdlmt7pqQnWv2HYWvrrr0N8GOIScASm7XLRsKk0fpzO4fVaChUx37W0FbE5oq0NzG7MmEYi9S/ahmAVUwRm4J1wWluAPC8nin1KaQzo+KXWjnrjFzgfQ9ltoDYzu5hCvCtcjHjCfo/QAoKdgqgCNPfDqdyDlRjq8F8LoyloSCq6m/BCPNqDiAmzQCBJXj5T+yTsAxzjI8T/SAidZtrhtFsf3c8ttynHB5Dyeo8LMiXzFnVzBVaRuQmD/JN/808yDb2MeW+Z+QnRd7NQK4sqsvXcGSadMYp1dPtsJ7spmQqk1FUqkQMSKYhk1b6b5jc9kUfgQIDAQAB";

const NAMED: [&str; 3] = ["MAIL_DKIM_KEY", "MAIL_DKIM_DOMAIN", "MAIL_DKIM_SELECTOR"];

fn key_file(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("mail-dkim-{name}.pem"));
    std::fs::write(&path, TEST_KEY).unwrap();
    path
}

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
    // 1000-byte limit the outbound path enforces.
    for line in text.split_inclusive("\r\n") {
        assert!(line.ends_with("\r\n"), "unterminated line: {line:?}");
        assert!(line.len() <= 1000, "line too long: {}", line.len());
    }
}

#[test]
fn a_half_configured_signer_is_refused_rather_than_sending_unsigned() {
    // Believing mail is signed when it is not is worse than knowing it is
    // not, so naming one setting demands all three -- and the refusal says
    // which one is missing.
    let refusal = complete(NAMED, [Some("/key.pem".to_owned()), None, None])
        .expect_err("a key without a domain is not a signer");
    assert!(refusal.contains("MAIL_DKIM_DOMAIN"), "{refusal}");
    let refusal = complete(
        NAMED,
        [
            Some("/key.pem".to_owned()),
            Some("example.org".to_owned()),
            None,
        ],
    )
    .expect_err("a key and domain without a selector is not a signer");
    assert!(refusal.contains("MAIL_DKIM_SELECTOR"), "{refusal}");
}

#[test]
fn nothing_configured_signs_nothing() {
    assert_eq!(complete(NAMED, [None, None, None]), Ok(None));
}

#[test]
fn an_unreadable_key_names_the_file_and_not_its_contents() {
    let refusal = Signer::from_key_file(
        "/nonexistent/dkim.pem",
        "example.org".to_owned(),
        "default".to_owned(),
    )
    .expect_err("a missing key is not a signer")
    .to_string();
    assert!(refusal.contains("/nonexistent/dkim.pem"), "{refusal}");
    assert!(!refusal.contains("BEGIN"), "{refusal}");
}

/// The verdict of a receiver, not the shape of a header. A signature with
/// correct syntax over the wrong body hash passes every assertion above and is
/// rejected on arrival, so the key that signed the message is published here
/// and `mail-auth`'s own verifier is asked what it makes of the result.
///
/// The table is sealed and the resolver behind it points at a closed loopback
/// port: every TXT lookup is answered from the table, and nothing else can
/// leave the host.
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

    // `mail-auth` looks the key up at `<selector>._domainkey.<domain>`.
    let dns = mail_protocol_imap::MailDns::sealed();
    dns.txt_add(
        "default._domainkey.example.org",
        DomainKey::parse(format!("v=DKIM1; k=rsa; p={TEST_PUBLIC_KEY}").as_bytes())
            .expect("the published record parses"),
    );
    let authenticator = nowhere();
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

/// A resolver that cannot answer: a sealed table answers TXT itself, and a
/// lookup it declines fails here rather than on the wire.
fn nowhere() -> MessageAuthenticator {
    use mail_auth::hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
    let mut server = NameServerConfig::udp_and_tcp(std::net::Ipv4Addr::LOCALHOST.into());
    for connection in &mut server.connections {
        connection.port = 1;
    }
    let mut options = ResolverOpts::default();
    options.attempts = 0;
    options.timeout = std::time::Duration::from_millis(50);
    MessageAuthenticator::new(ResolverConfig::from_name_servers(vec![server]), options)
        .expect("a resolver pointed nowhere")
}
