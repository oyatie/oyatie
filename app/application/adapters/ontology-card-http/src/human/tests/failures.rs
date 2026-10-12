use super::super::HumanOntologyCardSource;
use super::support::{cookie, raw_stub, source, stub};
use application_ontology_read::{OntologyReadError, OntologyReadSource};

#[test]
fn origins_and_canonical_native_cookie_are_validated_without_sending_credentials() {
    for origin in [
        "",
        "ftp://host",
        "http://example.com",
        "https://user:pass@host",
        "https://host/path",
        "https://host?q=x",
        "https://host#x",
    ] {
        assert!(HumanOntologyCardSource::new(origin).is_err(), "{origin}");
        assert!(
            HumanOntologyCardSource::for_loopback(origin).is_err(),
            "{origin}"
        );
    }
    assert!(HumanOntologyCardSource::new("https://foundry.example").is_ok());
    assert!(HumanOntologyCardSource::new("http://127.0.0.1:1").is_err());
    let transport = HumanOntologyCardSource::for_loopback("http://127.0.0.1:1").unwrap();
    for bad in [
        String::new(),
        "operator-token".into(),
        "__Host-oya_session=bad".into(),
        format!("{}; other=value", cookie('A')),
        format!("{}; {}", cookie('A'), cookie('B')),
        format!("other=value; {}", cookie('A')),
        format!("{}\r\nAuthorization: Bearer operator", cookie('A')),
        format!("__Host-oya_session=v1.{}B", "A".repeat(42)),
    ] {
        assert!(matches!(
            transport.read_source(&bad),
            Err(OntologyReadError::Unauthenticated)
        ));
    }
}

#[tokio::test]
async fn native_error_status_and_cause_keep_all_seven_typed_refusals_distinct() {
    for (status, body, expected) in [
        (401, "{}", OntologyReadError::Unauthenticated),
        (403, "{}", OntologyReadError::Forbidden),
        (
            400,
            r#"{"gate":"surface","cause":"a page limit must be a whole number from 1 to 1000"}"#,
            OntologyReadError::InvalidQuery,
        ),
        (
            404,
            r#"{"gate":"surface","cause":"the durable projection holds no object at this reference"}"#,
            OntologyReadError::AbsentObject,
        ),
        (
            409,
            r#"{"gate":"surface","cause":"that revision was never accepted for this entity type"}"#,
            OntologyReadError::RevisionConflict,
        ),
        (
            409,
            r#"{"gate":"surface","cause":"the stored values of that property are of another kind"}"#,
            OntologyReadError::FilterConflict,
        ),
        (
            503,
            r#"{"gate":"store","cause":"the projection store could not be read"}"#,
            OntologyReadError::SourceUnavailable,
        ),
        (
            503,
            r#"{"gate":"log","cause":"the action log could not be read; this view is unavailable"}"#,
            OntologyReadError::SourceUnavailable,
        ),
        (404, "", OntologyReadError::SourceUnavailable),
        (
            409,
            r#"{"gate":"surface","cause":"unknown conflict"}"#,
            OntologyReadError::SourceUnavailable,
        ),
        (
            400,
            r#"{"gate":"other","cause":"bad"}"#,
            OntologyReadError::SourceUnavailable,
        ),
        (
            500,
            "internal-host-and-secret",
            OntologyReadError::SourceUnavailable,
        ),
    ] {
        let (base, _) = stub(vec![(status, body.into(), String::new())]);
        assert_eq!(
            source(&base, &cookie('A'))
                .object("ten_acme", "ent_alpha", 1)
                .await,
            Err(expected)
        );
    }
}

#[tokio::test]
async fn redirects_are_refused_and_never_receive_the_session_cookie() {
    let target = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    target.set_nonblocking(true).unwrap();
    let location = format!(
        "location: http://{}/stolen\r\n",
        target.local_addr().unwrap()
    );
    let (base, _) = stub(vec![(302, "".into(), location)]);
    assert_eq!(
        source(&base, &cookie('A')).types("ten_acme").await,
        Err(OntologyReadError::SourceUnavailable)
    );
    assert!(matches!(target.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
}

#[tokio::test]
async fn malformed_unknown_and_oversized_bodies_are_unavailable_without_echoing_content() {
    for body in [
        "<html>secret</html>".to_owned(),
        r#"[{"entity_type":"ety_record","revision":"secret","properties":[]}]"#.into(),
        r#"[{"entity_type":"ety_record","revision":1,"properties":[],"extra":"secret"}]"#.into(),
        " ".repeat(1024 * 1024 + 1),
    ] {
        let (base, _) = stub(vec![(200, body, String::new())]);
        assert_eq!(
            source(&base, &cookie('A')).types("ten_acme").await,
            Err(OntologyReadError::SourceUnavailable)
        );
    }
}

#[tokio::test]
async fn truncated_chunked_oversized_and_slow_transports_refuse_with_the_fixed_timeout() {
    use std::time::{Duration, Instant};
    let too_large = " ".repeat(1024 * 1024 + 1);
    let chunked = format!(
        "HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n{:x}\r\n{too_large}\r\n0\r\n\r\n",
        too_large.len()
    );
    for reply in [
        b"HTTP/1.1 200 OK\r\ncontent-length: 200\r\nconnection: close\r\n\r\n[".to_vec(),
        chunked.into_bytes(),
    ] {
        let base = raw_stub(reply, Duration::ZERO);
        assert_eq!(
            source(&base, &cookie('A')).audit("ten_acme").await,
            Err(OntologyReadError::SourceUnavailable)
        );
    }
    let base = raw_stub(
        b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n[]".to_vec(),
        Duration::from_secs(6),
    );
    let start = Instant::now();
    assert_eq!(
        source(&base, &cookie('A')).types("ten_acme").await,
        Err(OntologyReadError::SourceUnavailable)
    );
    assert!(start.elapsed() < Duration::from_secs(4));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    assert_eq!(
        source(&base, &cookie('A')).types("ten_acme").await,
        Err(OntologyReadError::SourceUnavailable)
    );
}
