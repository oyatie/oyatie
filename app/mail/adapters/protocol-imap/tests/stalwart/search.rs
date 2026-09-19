use super::{REVISION, TOKEN, search_client::*};
use mail_api::BlobStore;
use mail_api::MetadataStore;
use mail_kernel::{Account, Command};
use mail_service::{MailService, OwnerPolicy};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{cell::RefCell, rc::Rc, sync::Arc};

mod upstream {
    include!(env!("STALWART_IMAP_SEARCH_SOURCE"));
}

#[tokio::test]
async fn upstream_imap_search() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_IMAP_SEARCH_SOURCE")))
        ),
        "3577c34f4485dba9db25c30546d71b8b710d612bb4a7130e33c21edbd7a60807"
    );
    let db = Arc::new(mail_sqlite_store::contract::converted_store(&[]));
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    // The upstream APPEND suite selects the first20 sorted directory entries,
    // alternating .imap expectations and .txt payloads: precisely000..009.txt.
    let fixtures: [&[u8]; 10] = [
        include_bytes!(env!("STALWART_IMAP_000")),
        include_bytes!(env!("STALWART_IMAP_001")),
        include_bytes!(env!("STALWART_IMAP_002")),
        include_bytes!(env!("STALWART_IMAP_003")),
        include_bytes!(env!("STALWART_IMAP_004")),
        include_bytes!(env!("STALWART_IMAP_005")),
        include_bytes!(env!("STALWART_IMAP_006")),
        include_bytes!(env!("STALWART_IMAP_007")),
        include_bytes!(env!("STALWART_IMAP_008")),
        include_bytes!(env!("STALWART_IMAP_009")),
    ];
    let hashes = [
        "510234d4bcd8cddb0658f2fa8c9f4c49d6cf1a15b78fd43a8edfdae5de441051",
        "db2c87a208b0d14715ad40c3c106a304cf4ecfa167e36227a44906d6300f1949",
        "45abbd25c923a7c7681d9e99592d997664c881689a90da147c6dc530ad801fbf",
        "43ba39d10ba27ee2baaf160e536e658e33482905a5a33c688e13af66ea7b06b5",
        "b12010ce07b6baf30017d8824577aaf986ebf24f57e1e35de35d01626fc1743a",
        "05735dbb3a573173a49543567a53247765afc50dfa2b7d579cdff7caa1a843ef",
        "7d3eab26dbf73d7afb0f4f030fc28447dacbfb6a0c358565acdd54b3c8509c42",
        "22e210024525f2081a7489de151af453e1195f677ff5ead62790c13f7dbeb02e",
        "7cde1bdcad4b14a9f4a01a6717820951612295155f462a7d566e18a14d11c958",
        "c27d0444f36f0f7902484da246d87d55c36358aa9c6f5abc341acc64a983bf39",
    ];
    for (index, raw) in fixtures.into_iter().enumerate() {
        assert_eq!(format!("{:x}", Sha256::digest(raw)), hashes[index]);
        let account = db.account("a").unwrap();
        db.execute(
            "a",
            mail_api::Precondition::Observed(account.revision),
            vec![
                db.append(
                    "a",
                    vec!["inbox".into()],
                    raw,
                    vec![format!("Flag_{index:03}")],
                    1_789_430_400,
                )
                .unwrap(),
            ],
        )
        .unwrap();
    }
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let outcomes = Rc::new(RefCell::new(Vec::new()));
    let mut primary = ImapConnection::connect(service.clone(), outcomes.clone()).await;
    let mut check = ImapConnection::connect(service, outcomes.clone()).await;
    upstream::test(
        &mut primary,
        &mut check,
        &TestServer {
            server: SearchStore,
        },
    )
    .await;
    primary.close().await;
    check.close().await;
    let outcomes = outcomes.borrow();
    let passed = outcomes
        .iter()
        .filter(|value| value["passed"] == true)
        .count();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "upstream_revision": REVISION, "suite": "tests/src/imap/search.rs",
            "transport": "duplex IMAP wire", "passed": passed,
            "assertions": outcomes.len(), "skipped": 0, "outcomes": *outcomes,
        }))
        .unwrap()
    );
    assert_eq!(
        outcomes.len(),
        37,
        "all upstream assertions and session checks must execute"
    );
    assert_eq!(
        passed,
        outcomes.len(),
        "unchanged upstream IMAP search assertions failed"
    );
}
