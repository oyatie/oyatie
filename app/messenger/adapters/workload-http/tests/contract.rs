#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use messenger_domain::{Error, InstallationSpec, IntegrationCapability, IntegrationKind};
use messenger_workload_api::Workload;
use messenger_workload_http::HttpWorkload;
use serde_json::{Value, json};
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
        service: "bridge.slack".into(),
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
    HttpWorkload::with_timeout(base, "caller-token", Duration::from_secs(2)).unwrap()
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
        .authorize("tenant-a", &spec(), IntegrationCapability::SendMessages)
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
    assert_eq!(body["tenantId"], "tenant-a");
    assert_eq!(body["workloadId"], "wl_bridge");
    assert_eq!(body["owningCapability"], "bridge.slack");
    assert_eq!(body["action"], "messenger.integration.messages.send");
    assert_eq!(body["resource"]["resourceType"], "messenger.room");
    assert_eq!(body["resource"]["resourceId"], "!room:messenger.test");
}

#[tokio::test]
async fn authorize_maps_deny_and_outage() {
    let (base, _, handle) = serve(403, r#"{"effect":"DENY","reason":{"kind":"defaultDeny"}}"#);
    let denied = client(&base)
        .authorize("tenant-a", &spec(), IntegrationCapability::SendMessages)
        .await;
    handle.join().unwrap();
    assert_eq!(denied, Err(Error::Denied));

    let (base, _, handle) = serve(
        503,
        r#"{"error":{"code":"DEPENDENCY_UNAVAILABLE","message":"x"}}"#,
    );
    let outage = client(&base)
        .authorize("tenant-a", &spec(), IntegrationCapability::SendMessages)
        .await;
    handle.join().unwrap();
    assert!(matches!(outage, Err(Error::Unavailable(_))));

    let (base, _, handle) = serve(200, r#"{"effect":"DENY","reason":{"kind":"defaultDeny"}}"#);
    let body_deny = client(&base)
        .authorize("tenant-a", &spec(), IntegrationCapability::SendMessages)
        .await;
    handle.join().unwrap();
    assert_eq!(body_deny, Err(Error::Denied));
}

#[tokio::test]
async fn identify_projects_principal_and_refuses_bad_token() {
    let (base, captured, handle) = serve(
        200,
        r#"{"tenantId":"tenant-a","workloadId":"wl_bridge","owningCapability":"bridge.slack","trustDomain":"spiffe://tenant-a","state":"active","scopes":[]}"#,
    );
    let identity = client(&base).identify("jwt").await.unwrap();
    handle.join().unwrap();
    let raw = captured.lock().unwrap().clone();
    assert!(raw.contains("POST /tokens/validate"));
    assert_eq!(request_json(&raw), json!({"token":"jwt"}));
    assert_eq!(identity.tenant, "tenant-a");
    assert_eq!(identity.workload, "wl_bridge");
    assert_eq!(identity.service, "bridge.slack");
    assert_eq!(identity.state, "active");

    let (base, _, handle) = serve(422, r#"{"error":{"code":"TOKEN_INVALID","message":"x"}}"#);
    let rejected = client(&base).identify("jwt").await;
    handle.join().unwrap();
    assert_eq!(rejected, Err(Error::Denied));
}

#[tokio::test]
async fn unreachable_iam_is_unavailable() {
    let err = client("http://127.0.0.1:1")
        .authorize("tenant-a", &spec(), IntegrationCapability::SendMessages)
        .await;
    assert!(matches!(err, Err(Error::Unavailable(_))));
}
