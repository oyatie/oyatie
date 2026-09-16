use super::*;

#[tokio::test]
async fn mailbox_creation_references_and_reordered_query_changes_replay_correctly() {
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
        "http://localhost".into(),
    );
    let created = call(&app,"Mailbox/set",json!({"accountId":"a","create":{
        "child":{"name":"Child","parentId":"#parent"},"parent":{"name":"Parent"},"other":{"name":"Alpha"}
    }})).await;
    let child = created["created"]["child"]["id"]
        .as_str()
        .expect("forward parent creation reference");
    let parent = created["created"]["parent"]["id"].as_str().unwrap();
    let other = created["created"]["other"]["id"].as_str().unwrap();
    let before = call(
        &app,
        "Mailbox/query",
        json!({"accountId":"a","sort":[{"property":"name"}]}),
    )
    .await;
    let changed = call(
        &app,
        "Mailbox/set",
        json!({"accountId":"a","update":{other:{"name":"Zulu"}}}),
    )
    .await;
    assert!(changed["updated"].as_object().unwrap().contains_key(other));
    let changes = call(&app,"Mailbox/queryChanges",json!({"accountId":"a","sort":[{"property":"name"}],"sinceQueryState":before["queryState"]})).await;
    let mut ids = before["ids"].as_array().unwrap().clone();
    ids.retain(|id| !changes["removed"].as_array().unwrap().contains(id));
    for added in changes["added"].as_array().unwrap() {
        ids.insert(
            added["index"].as_u64().unwrap() as usize,
            added["id"].clone(),
        );
    }
    let after = call(
        &app,
        "Mailbox/query",
        json!({"accountId":"a","sort":[{"property":"name"}]}),
    )
    .await;
    assert_eq!(json!(ids), after["ids"]);
    let snapshot = db.account("a").unwrap();
    let refused = call(
        &app,
        "Mailbox/set",
        json!({"accountId":"a","update":{parent:{"parentId":child}}}),
    )
    .await;
    assert_eq!(refused["notUpdated"][parent]["type"], "invalidProperties");
    assert_eq!(db.account("a").unwrap(), snapshot);
    let deleted = call(
        &app,
        "Mailbox/set",
        json!({"accountId":"a","destroy":[parent,child]}),
    )
    .await;
    assert_eq!(deleted["destroyed"].as_array().unwrap().len(), 2);
    let readonly = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(CopyPolicy {
                source_write: false,
                target_write: false,
            }),
        }),
        "http://localhost".into(),
    );
    let get = call(
        &readonly,
        "Mailbox/get",
        json!({"accountId":"a","ids":["inbox"]}),
    )
    .await;
    assert_eq!(get["list"][0]["myRights"]["mayAddItems"], false);
    assert_eq!(get["list"][0]["myRights"]["mayCreateChild"], false);
}

#[tokio::test]
async fn mailbox_thread_counts_distinguish_messages_and_unread_threads() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    for raw in [
        b"Message-ID: <first@example.org>\r\n\r\nfirst".as_slice(),
        b"Message-ID: <reply@example.org>\r\nIn-Reply-To: <first@example.org>\r\n\r\nreply",
    ] {
        db.deliver(&["alice@example.org".into()], raw).unwrap();
    }
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
    for (seen, unread_emails, unread_threads) in
        [(None, 2, 1), (Some("e1"), 1, 1), (Some("e2"), 0, 0)]
    {
        if let Some(id) = seen {
            let revision = db.account("a").unwrap().revision;
            db.execute(
                "a",
                revision,
                vec![Command::Keywords {
                    id: id.into(),
                    keywords: vec!["$seen".into()],
                }],
            )
            .unwrap();
        }
        let mailboxes = call(
            &app,
            "Mailbox/get",
            json!({"accountId":"a", "ids":["inbox"]}),
        )
        .await;
        let inbox = &mailboxes["list"][0];
        assert_eq!(inbox["totalEmails"], 2);
        assert_eq!(inbox["totalThreads"], 1);
        assert_eq!(inbox["unreadEmails"], unread_emails);
        assert_eq!(inbox["unreadThreads"], unread_threads);
    }
}
