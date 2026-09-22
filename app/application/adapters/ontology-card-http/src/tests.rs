//! Against a one-shot `std::net` listener that speaks just enough HTTP/1.1.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use application_ontology_card::{OntologyCardError, OntologyCardSource};

use super::HttpOntologyCardSource;

const TOKEN: &str = "operator-token-for-tests";
const FULL: &str = r#"{"policy_version":"cedar-2026-09-21","served_tenants":3,"projection_lag":7,"poisoned_entries":1}"#;

fn stub(status: &str, body: &str, delay: Duration) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (seen, requests) = channel();
    let (status, body) = (status.to_owned(), body.to_owned());
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
        }
        let _ = seen.send(String::from_utf8_lossy(&request).into_owned());
        std::thread::sleep(delay);
        let _ = write!(
            stream,
            "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
    });
    (base, requests)
}

fn stub_that_closes_mid_body() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut chunk = [0_u8; 1024];
        let _ = stream.read(&mut chunk);
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 200\r\n\r\n{{\"policy"
        );
    });
    base
}

fn read(base: &str) -> Result<application_ontology_card::OntologyCardFacts, OntologyCardError> {
    HttpOntologyCardSource::new(base, TOKEN)
        .unwrap()
        .ontology_card_facts()
}

fn reason(
    result: Result<application_ontology_card::OntologyCardFacts, OntologyCardError>,
) -> String {
    match result {
        Ok(facts) => panic!("answered {facts:?}"),
        Err(OntologyCardError::Unavailable(reason)) => reason,
    }
}

#[test]
fn a_full_body_answers_the_four_fields_and_the_request_carried_the_bearer() {
    let (base, requests) = stub("200 OK", FULL, Duration::ZERO);
    let facts = read(&base).unwrap();
    assert_eq!(facts.policy_version, "cedar-2026-09-21");
    assert_eq!(facts.served_tenants, 3);
    assert_eq!(facts.projection_lag, 7);
    assert_eq!(facts.poisoned_entries, 1);
    let request = requests.recv().unwrap();
    assert!(
        request.starts_with("GET /statusz HTTP/1.1\r\n"),
        "{request}"
    );
    assert!(
        request
            .to_ascii_lowercase()
            .contains(&format!("authorization: bearer {TOKEN}\r\n")),
        "{request}"
    );
}

#[test]
fn fields_the_card_does_not_render_are_ignored() {
    let extra = FULL.replacen('{', r#"{"observed_tenants":9,"tenant":null,"#, 1);
    let (base, _requests) = stub("200 OK", &extra, Duration::ZERO);
    assert_eq!(read(&base).unwrap().served_tenants, 3);
}

#[test]
fn a_missing_field_is_unavailable_not_a_default() {
    let short = FULL.replace(r#","poisoned_entries":1"#, "");
    let (base, _requests) = stub("200 OK", &short, Duration::ZERO);
    assert_eq!(reason(read(&base)), "/statusz body: Data");
}

#[test]
fn a_server_error_is_unavailable() {
    let (base, _requests) = stub("500 Internal Server Error", "{}", Duration::ZERO);
    assert!(reason(read(&base)).contains("500"));
}

#[test]
fn a_refused_credential_is_unavailable() {
    let (base, _requests) = stub(
        "401 Unauthorized",
        r#"{"refusal":"credential"}"#,
        Duration::ZERO,
    );
    assert!(reason(read(&base)).contains("401"));
}

#[test]
fn a_closed_port_is_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let reason = reason(read(&base));
    assert!(
        !reason.contains(&base[7..]),
        "the card text must not carry the listener address: {reason}"
    );
}

#[test]
fn a_body_that_is_not_json_is_unavailable() {
    let (base, _requests) = stub("200 OK", "<html>maintenance</html>", Duration::ZERO);
    let _ = reason(read(&base));
}

#[test]
fn a_connection_closed_mid_body_names_no_address() {
    let base = stub_that_closes_mid_body();
    let reason = reason(read(&base));
    assert!(
        reason.starts_with("GET /statusz body:"),
        "the body stage must be what refused: {reason}"
    );
    assert!(
        !reason.contains(&base[7..]),
        "the card text must not carry the listener address: {reason}"
    );
}

#[test]
fn a_body_with_a_wrong_field_type_echoes_no_body_content() {
    let body = r#"{"policy_version":"p","served_tenants":"http://foundry.internal:8090","projection_lag":0,"poisoned_entries":0}"#;
    let (base, _requests) = stub("200 OK", body, Duration::ZERO);
    let reason = reason(read(&base));
    assert!(
        !reason.contains("foundry.internal"),
        "the card text must not carry body content: {reason}"
    );
}

#[test]
fn a_slow_listener_is_unavailable_within_the_timeout() {
    let (base, _requests) = stub("200 OK", FULL, HttpOntologyCardSource::TIMEOUT * 3);
    let started = Instant::now();
    let _ = reason(read(&base));
    assert!(
        started.elapsed() < HttpOntologyCardSource::TIMEOUT * 2,
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_bad_base_url_or_an_empty_token_is_refused_at_construction() {
    assert!(HttpOntologyCardSource::new("", TOKEN).is_err());
    assert!(HttpOntologyCardSource::new("ftp://host", TOKEN).is_err());
    assert!(HttpOntologyCardSource::new("http://127.0.0.1:1", "").is_err());
    assert!(HttpOntologyCardSource::new("http://127.0.0.1:1/", TOKEN).is_ok());
}

/// The composition root runs on Tokio worker threads; a blocking read there
/// must neither panic nor need the caller to hop to a blocking pool.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_read_from_inside_a_tokio_worker_answers() {
    let (base, _requests) = stub("200 OK", FULL, Duration::ZERO);
    assert_eq!(read(&base).unwrap().projection_lag, 7);
}
