#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use messenger_domain::{Error, InstallationSpec, IntegrationCapability, IntegrationKind};
use messenger_workload_api::Workload;
use messenger_workload_http::HttpWorkload;
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn spec() -> InstallationSpec {
    InstallationSpec {
        id: "11111111-1111-1111-1111-111111111111".into(),
        room: "!room:messenger.test".into(),
        service: "cap.bridge.slack".into(),
        workload: "wl_bridge".into(),
        matrix_user: Some("@bot:messenger.test".into()),
        kind: IntegrationKind::Bridge,
        capabilities: BTreeSet::from([IntegrationCapability::SendMessages]),
    }
}

fn read_http(stream: &mut std::net::TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                if let Some(headers) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = &buf[..headers];
                    let extra = buf.len() - headers - 4;
                    let length = std::str::from_utf8(head)
                        .unwrap_or("")
                        .lines()
                        .find_map(|line| {
                            line.split_once(':').and_then(|(name, value)| {
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().ok())
                                    .flatten()
                            })
                        })
                        .unwrap_or(0);
                    if extra >= length {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
        if buf.len() > 65_536 {
            break;
        }
    }
    String::from_utf8_lossy(&buf).into_owned()
}

fn serve(status: u16, body: &str) -> (String, Arc<Mutex<String>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(String::new()));
    let seen = Arc::clone(&captured);
    let body = body.to_owned();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        *seen.lock().unwrap() = read_http(&mut stream);
        let header = format!(
            "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(body.as_bytes());
    });
    (format!("http://{addr}"), captured, handle)
}

fn client(base: &str) -> HttpWorkload {
    HttpWorkload::new(base, "caller-token").unwrap()
}

async fn authorize_against(status: u16, body: &str) -> Result<(), Error> {
    let (base, _, handle) = serve(status, body);
    let result = client(&base)
        .authorize("ten_acme", &spec(), IntegrationCapability::SendMessages)
        .await;
    handle.join().unwrap();
    result
}

fn request_json(raw: &str) -> Value {
    let body = raw.split("\r\n\r\n").nth(1).unwrap_or("");
    serde_json::from_str(body).unwrap_or(Value::Null)
}

#[tokio::test]
async fn authorize_allow_posts_iam_camel_case() {
    let (base, captured, handle) =
        serve(200, r#"{"effect":"ALLOW","reason":{"kind":"defaultDeny"}}"#);
    client(&base)
        .authorize("ten_acme", &spec(), IntegrationCapability::SendMessages)
        .await
        .unwrap();
    handle.join().unwrap();
    let raw = captured.lock().unwrap().clone();
    assert!(raw.contains("POST /authorize"));
    assert!(
        raw.to_ascii_lowercase()
            .contains("authorization: bearer caller-token")
    );
    let body = request_json(&raw);
    assert_eq!(body["tenantId"], "ten_acme");
    assert_eq!(body["workloadId"], "wl_bridge");
    assert_eq!(body["owningCapability"], "cap.bridge.slack");
    assert_eq!(body["action"], "messenger.integration.messages.send");
    assert_eq!(body["resource"]["resourceType"], "messenger.room");
    assert_eq!(body["resource"]["resourceId"], "!room:messenger.test");
}

#[tokio::test]
async fn authorize_maps_deny_and_outage() {
    let deny = r#"{"effect":"DENY","reason":{"kind":"defaultDeny"}}"#;
    assert_eq!(authorize_against(403, deny).await, Err(Error::Denied));
    assert_eq!(authorize_against(401, deny).await, Err(Error::Denied));
    assert_eq!(authorize_against(422, deny).await, Err(Error::Denied));
    assert!(matches!(
        authorize_against(429, "{}").await,
        Err(Error::Unavailable(_))
    ));
    assert!(matches!(
        authorize_against(408, "{}").await,
        Err(Error::Unavailable(_))
    ));
    assert!(matches!(
        authorize_against(302, "{}").await,
        Err(Error::Unavailable(_))
    ));
    assert!(matches!(
        authorize_against(503, "{}").await,
        Err(Error::Unavailable(_))
    ));
    assert!(matches!(
        authorize_against(200, r#"{"effect":"OTHER"}"#).await,
        Err(Error::Unavailable(_))
    ));
}

#[tokio::test]
async fn unreachable_iam_is_unavailable() {
    let err = client("http://127.0.0.1:1")
        .authorize("ten_acme", &spec(), IntegrationCapability::SendMessages)
        .await;
    assert!(matches!(err, Err(Error::Unavailable(_))));
}
