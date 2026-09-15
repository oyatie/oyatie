use super::*;

fn fixture() -> (Router, Arc<SqliteStore>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let app = mail_protocol::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "https://localhost".into(),
    );
    (app, db)
}

#[tokio::test]
async fn result_references_cannot_expand_a_small_request_without_bound() {
    let (app, _) = fixture();
    let mut calls = vec![json!(["Core/echo", {"value":"x".repeat(128*1024)}, "0"])];
    for i in 1..8 {
        let reference = json!({"resultOf":(i-1).to_string(), "name":"Core/echo", "path":""});
        calls.push(json!(["Core/echo", {"#left":reference, "#right":reference}, i.to_string()]));
    }
    let response = app
        .oneshot(
            Request::post("/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":calls}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(
        bytes.len() < 4 * 1024 * 1024,
        "resolved arguments bypassed request budget: {}",
        bytes.len()
    );
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        value["methodResponses"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v[0] == "error" && v[1]["type"] == "limit")
    );
}

#[tokio::test]
async fn nested_raw_mime_is_bounded_before_recursive_rendering() {
    let (app, db) = fixture();
    let mut raw = String::new();
    for depth in 0..40 {
        raw.push_str(&format!(
            "Content-Type: multipart/mixed; boundary=b{depth}\r\n\r\n--b{depth}\r\n"
        ));
    }
    raw.push_str("Content-Type: text/plain\r\n\r\nbody\r\n");
    for depth in (0..40).rev() {
        raw.push_str(&format!("--b{depth}--\r\n"));
    }
    let blob = db.put_blob("a", raw.as_bytes()).unwrap();
    let parsed = call(
        &app,
        "Email/parse",
        json!({"accountId":"a", "blobIds":[blob]}),
    )
    .await;
    assert_eq!(parsed["notParsable"], json!([blob]));
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    let result = response(&app, "Email/get", json!({"accountId":"a", "ids":["e1"]})).await;
    assert_eq!(result["methodResponses"][0][1]["type"], "tooLarge");
}

#[tokio::test]
async fn body_fetches_have_a_cumulative_response_budget() {
    let (app, db) = fixture();
    let raw = format!("Subject: bounded\r\n\r\n{}", "x".repeat(4 * 1024 * 1024));
    for _ in 0..9 {
        db.deliver(&["alice@example.org".into()], raw.as_bytes())
            .unwrap();
    }
    let ids: Vec<_> = db
        .account("a")
        .unwrap()
        .messages
        .into_iter()
        .map(|m| m.id)
        .collect();
    let result = response(
        &app,
        "Email/get",
        json!({"accountId":"a", "ids":ids, "fetchAllBodyValues":true, "properties":["bodyValues"]}),
    )
    .await;
    assert_eq!(result["methodResponses"][0][1]["type"], "limit");
    // Parsing is subject to the same limit, even when all IDs are individually valid.
    let result = response(&app, "Email/parse", json!({"accountId":"a", "blobIds":ids, "fetchAllBodyValues":true, "properties":["bodyValues"]})).await;
    assert_eq!(result["methodResponses"][0][1]["type"], "limit");
    let calls: Vec<_> = (0..10).map(|i| json!(["Email/get", {"accountId":"a", "ids":["e1"], "fetchAllBodyValues":true, "properties":["bodyValues"]}, i.to_string()])).collect();
    let response = app.oneshot(Request::post("/jmap").header("authorization", format!("Bearer {TOKEN}")).header("content-type", "application/json")
        .body(Body::from(json!({"using":["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"], "methodCalls":calls}).to_string())).unwrap()).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(bytes.len() < 32 * 1024 * 1024);
    let response: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        response["methodResponses"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v[0] == "error" && v[1]["type"] == "limit")
    );
}

#[tokio::test]
async fn repeated_header_projections_share_materialization_budget() {
    let (app, db) = fixture();
    let raw = format!("X-Long-Header: {}\r\n\r\nbody", "x".repeat(128 * 1024));
    db.deliver(&["alice@example.org".into()], raw.as_bytes())
        .unwrap();
    let properties: Vec<_> = (0..512)
        .map(|mask| {
            let mut bit = 0;
            let name: String = "x-long-header"
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        let upper = mask & (1 << bit) != 0;
                        bit += 1;
                        if upper { c.to_ascii_uppercase() } else { c }
                    } else {
                        c
                    }
                })
                .collect();
            format!("header:{name}:asRaw")
        })
        .collect();
    let result = response(
        &app,
        "Email/get",
        json!({"accountId":"a", "ids":["e1"], "properties":properties}),
    )
    .await;
    assert_eq!(result["methodResponses"][0][1]["type"], "limit");
}
