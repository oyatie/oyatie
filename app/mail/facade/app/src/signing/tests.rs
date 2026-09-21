use super::*;

mod verify;

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

/// The longest value `sub_domain` permits, built from the longest labels it
/// permits. One validator covers both settings, so this is the selector's
/// maximum as well as the domain's -- the worst case the signature's first
/// line can carry, and so the case the allowance has to cover.
pub(super) fn longest_sub_domain() -> String {
    let domain =
        ["a", "b", "c"].map(|label| label.repeat(63)).join(".") + &format!(".{}", "d".repeat(61));
    assert_eq!(domain.len(), 253);
    sub_domain("MAIL_DKIM_DOMAIN", &domain).expect("the longest permitted domain is permitted");
    domain
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

/// A selector or domain goes verbatim into the signature's first line, which
/// nothing folds. One long or malformed value would push every outbound
/// message past the 1000-byte line limit -- not one message, all of them --
/// so these are refused at startup rather than on the wire.
#[test]
fn a_selector_or_domain_that_would_overrun_the_signature_line_is_refused() {
    for bad in [
        String::new(),
        "a".repeat(254),
        format!("{}.example", "a".repeat(64)),
        "exam ple.org".to_owned(),
        "example..org".to_owned(),
        "selector\r\nX-Injected: yes".to_owned(),
        "sel:ector".to_owned(),
    ] {
        let refusal = sub_domain("MAIL_DKIM_SELECTOR", &bad)
            .expect_err("a selector that cannot fold is not a selector");
        assert!(refusal.contains("MAIL_DKIM_SELECTOR"), "{refusal}");
    }
    for good in [
        "default",
        "s2026-09",
        "mail.example.org",
        &longest_sub_domain(),
    ] {
        sub_domain("MAIL_DKIM_DOMAIN", good).expect("a sub-domain is a sub-domain");
    }
}

/// `MAX_SUBMISSION_BYTES` is what submission accepts; `MAX_MESSAGE_BYTES` is
/// what outbound wire validation allows. Signing happens between them, so a
/// message accepted at the ceiling must still fit once signed -- otherwise the
/// transports answer `554`, the queue writes a DSN and deletes the job, and a
/// maximum-size message that used to deliver becomes a permanent bounce.
///
/// Signed here with the longest selector and domain the configuration permits,
/// because that is the largest signature an operator can provoke.
#[test]
fn a_message_at_the_submission_ceiling_still_fits_once_signed() {
    let file = key_file("ceiling");
    let signer = Signer::from_key_file(
        &file.display().to_string(),
        longest_sub_domain(),
        longest_sub_domain(),
    )
    .expect("a 2048-bit key is a signer");
    let raw = at_the_ceiling();
    assert_eq!(raw.len(), mail_kernel::MAX_SUBMISSION_BYTES);
    let signed = signer.signed(&raw).expect("a 2048-bit key signs");
    assert!(
        signed.len() <= mail_kernel::MAX_MESSAGE_BYTES,
        "signing put the message {} bytes over the deliverable ceiling",
        signed.len() - mail_kernel::MAX_MESSAGE_BYTES
    );
}

/// Every signed header present with a realistic value, padded to exactly the
/// size submission accepts.
fn at_the_ceiling() -> Vec<u8> {
    let mut raw = Vec::with_capacity(mail_kernel::MAX_SUBMISSION_BYTES);
    for header in HEADERS {
        raw.extend_from_slice(header.as_bytes());
        raw.extend_from_slice(b"\r\n");
    }
    raw.extend_from_slice(b"\r\n");
    let line = b"padding to the ceiling, sixty-four bytes of body per line...\r\n";
    while raw.len() + line.len() <= mail_kernel::MAX_SUBMISSION_BYTES {
        raw.extend_from_slice(line);
    }
    raw.resize(mail_kernel::MAX_SUBMISSION_BYTES - 2, b'x');
    raw.extend_from_slice(b"\r\n");
    raw
}

pub(super) const HEADERS: &[&str] = &[
    "From: \"Alice Example-Longname\" <alice@example.org>",
    "To: \"Bob\" <bob@example.net>, \"Carol\" <carol@example.net>",
    "Cc: \"Dave\" <dave@example.net>",
    "Subject: a realistic subject line about the quarterly reporting deadline",
    "Date: Mon, 21 Sep 2026 12:34:56 +0000",
    "Message-ID: <0123456789abcdef0123456789abcdef@example.org>",
    "MIME-Version: 1.0",
    "Content-Type: multipart/mixed; boundary=\"----------0123456789abcdef0123\"",
    "Content-Transfer-Encoding: 8bit",
];

/// PEM that parses as a container and holds nothing usable, and a key that is
/// valid PEM but not RSA. Both reach a refusal with contents to leak, which
/// the file this replaced -- a path that does not exist -- never could.
const MALFORMED: &str = "-----BEGIN PRIVATE KEY-----
SENTINEL-NOT-A-KEY
-----END PRIVATE KEY-----";

const ED25519: &str = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEILRgaJ7Idv/+uz6RhULamOB3mxH4Rqzs2uWNgtMaOgz6
-----END PRIVATE KEY-----";

#[test]
fn a_key_that_cannot_sign_names_the_file_and_not_its_contents() {
    for (name, contents) in [("malformed", MALFORMED), ("ed25519", ED25519)] {
        let path = std::env::temp_dir().join(format!("mail-dkim-{name}.pem"));
        std::fs::write(&path, contents).unwrap();
        let path = path.display().to_string();
        let refusal = Signer::from_key_file(&path, "example.org".to_owned(), "default".to_owned())
            .expect_err("a key that cannot sign is not a signer")
            .to_string();
        assert!(refusal.contains(&path), "{refusal}");
        assert!(!refusal.contains("SENTINEL"), "{refusal}");
        assert!(!refusal.contains("BEGIN"), "{refusal}");
        for line in contents.lines().filter(|l| !l.starts_with("---")) {
            assert!(!refusal.contains(line), "{refusal}");
        }
    }
}
