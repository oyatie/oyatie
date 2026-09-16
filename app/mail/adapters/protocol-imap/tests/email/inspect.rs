use super::*;
use axum::http::StatusCode;

#[tokio::test]
async fn nested_message_parts_remain_downloadable_and_keep_content_language() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "other", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    let raw = b"Subject: outer\r\nContent-Type: message/rfc822\r\n\r\nSubject: inner\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Language: en, fr\r\n\r\nBonjour";
    let blob = db.put_blob("a", raw).unwrap();
    let app = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "https://localhost".into(),
    );
    let parsed = call(
        &app,
        "Email/parse",
        json!({"accountId":"a","blobIds":[blob],"properties":["bodyStructure"]}),
    )
    .await;
    let inner_blob = parsed["parsed"][&blob]["bodyStructure"]["blobId"]
        .as_str()
        .unwrap();
    let inner = call(&app, "Email/parse", json!({"accountId":"a","blobIds":[inner_blob],"properties":["bodyStructure"],"bodyProperties":["blobId","language"]})).await;
    let body = &inner["parsed"][inner_blob]["bodyStructure"];
    let part = body["blobId"].as_str().unwrap();
    let download = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/download/a/{part}/text"))
                .header("Authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(download.status(), StatusCode::OK);
    assert_eq!(
        download.into_body().collect().await.unwrap().to_bytes(),
        b"Bonjour".as_slice()
    );
    assert_eq!(body["language"], json!(["en", "fr"]));
    let foreign = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/download/b/{part}/text"))
                .header("Authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(foreign.status(), StatusCode::NOT_FOUND);
    for id in [
        format!("{blob}.bogus"),
        format!("{blob}.999"),
        format!("{blob}{}", ".0".repeat(33)),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/download/a/{id}/text"))
                    .header("Authorization", format!("Bearer {TOKEN}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn header_forms_preserve_repeated_values_and_parse_respects_account_access() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "other", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    let raw = "From: Team: Alice <alice@example.org>, Bob <bob@example.org>;\r\nSubject: =?UTF-8?B?Y2Fmw6k=?=\r\nX-Note: first\r\nX-Note: second\r\nDate: Mon, 14 Sep 2026 08:00:00 -0400\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\néclair";
    let blob = db.put_blob("a", raw.as_bytes()).unwrap();
    let foreign_blob = db.put_blob("b", raw.as_bytes()).unwrap();
    let app = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "https://localhost".into(),
    );
    let parsed = call(&app, "Email/parse", json!({"accountId":"a","blobIds":[blob],"properties":["subject","header:X-Note:asText","header:X-Note:asText:all","header:From:asGroupedAddresses","header:Missing:all","sentAt","bodyValues"],"fetchTextBodyValues":true,"maxBodyValueBytes":1})).await;
    let email = &parsed["parsed"][&blob];
    assert_eq!(email["subject"], "café");
    assert_eq!(email["header:X-Note:asText"], "second");
    assert_eq!(
        email["header:X-Note:asText:all"],
        json!(["first", "second"])
    );
    assert_eq!(email["header:Missing:all"], json!([]));
    assert_eq!(email["header:From:asGroupedAddresses"][0]["name"], "Team");
    assert_eq!(
        email["header:From:asGroupedAddresses"][0]["addresses"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(email["sentAt"], "2026-09-14T12:00:00Z");
    assert_eq!(email["bodyValues"]["0"]["value"], "");
    assert_eq!(email["bodyValues"]["0"]["isTruncated"], true);
    let foreign = response(
        &app,
        "Email/parse",
        json!({"accountId":"b","blobIds":[foreign_blob]}),
    )
    .await;
    assert_eq!(foreign["methodResponses"][0][1]["type"], "accountNotFound");
    for args in [
        json!({"properties":["header:Subject:invalid"]}),
        json!({"bodyProperties":["bogus"]}),
        json!({"fetchTextBodyValues":1}),
        json!({"maxBodyValueBytes":-1}),
    ] {
        let mut args = args;
        args["accountId"] = json!("a");
        args["blobIds"] = json!([blob]);
        let value = response(&app, "Email/parse", args).await;
        assert_eq!(value["methodResponses"][0][1]["type"], "invalidArguments");
    }
}

#[tokio::test]
async fn thread_changes_preserve_atomic_merges_and_page_at_real_commits() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let app = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "https://localhost".into(),
    );
    for (id, refs) in [("one", ""), ("two", ""), ("bridge", "<one> <two>")] {
        db.deliver(
            &["alice@example.org".into()],
            format!("Subject: Topic\r\nMessage-ID: <{id}>\r\nReferences: {refs}\r\n\r\nbody")
                .as_bytes(),
        )
        .unwrap();
    }
    let changed = call(
        &app,
        "Thread/changes",
        json!({"accountId":"a","sinceState":"2"}),
    )
    .await;
    assert_eq!(changed["updated"], json!(["e1"]));
    assert_eq!(changed["destroyed"], json!(["e2"]));
    let too_small = response(
        &app,
        "Thread/changes",
        json!({"accountId":"a","sinceState":"2","maxChanges":1}),
    )
    .await;
    assert_eq!(
        too_small["methodResponses"][0][1]["type"],
        "cannotCalculateChanges"
    );
    db.execute(
        "a",
        3,
        vec![Command::Keywords {
            id: "e1".into(),
            keywords: vec!["$seen".into()],
        }],
    )
    .unwrap();
    let flags = call(
        &app,
        "Thread/changes",
        json!({"accountId":"a","sinceState":"3"}),
    )
    .await;
    assert_eq!(flags["updated"], json!([]));
    db.execute("a", 4, vec![Command::Destroy { id: "e3".into() }])
        .unwrap();
    db.execute(
        "a",
        5,
        vec![
            Command::Destroy { id: "e1".into() },
            Command::Destroy { id: "e2".into() },
        ],
    )
    .unwrap();
    let removed = call(
        &app,
        "Thread/changes",
        json!({"accountId":"a","sinceState":"3"}),
    )
    .await;
    assert_eq!(removed["destroyed"], json!(["e1"]));
    let current = call(&app, "Thread/get", json!({"accountId":"a","ids":["e1"]})).await;
    assert_eq!(current["notFound"], json!(["e1"]));
}
