use super::{
    REVISION,
    client::{ImapConnection, Outcomes, ResponseType, Type},
    fixture::{self, JDOE, SmtpConnection, TestServer},
    suite,
};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, rc::Rc};

fn digest(source: &[u8], expected: &str, suite: &str) {
    assert_eq!(
        format!("{:x}", Sha256::digest(source)),
        expected,
        "{suite} changed upstream; review before updating the baseline"
    );
}

fn server() -> TestServer {
    let (db, service) = fixture::open();
    SmtpConnection::install(&db);
    TestServer {
        db,
        service,
        outcomes: Rc::new(RefCell::new(vec![])),
    }
}

/// Upstream imports the ten `resources/imap/*.txt` messages through its own
/// APPEND suite before every dependent suite; the same wire path seeds here.
async fn seeded(server: &TestServer) -> (ImapConnection, ImapConnection) {
    let mut imap = server.connect(JDOE, "_x ").await;
    let mut check = server.connect(JDOE, "_y ").await;
    suite::append::test(&mut imap, &mut check, server).await;
    (imap, check)
}

fn finish(name: &str, outcomes: &Outcomes) {
    let outcomes = outcomes.borrow();
    let passed = outcomes.iter().filter(|o| o["passed"] == true).count();
    println!(
        "{}",
        serde_json::json!({"suite":format!("tests/src/imap/{name}.rs"),"upstream_revision":REVISION,"transport":"duplex IMAP wire","wire_predicates":outcomes.len(),"passed":passed,"skipped":0,"failures":outcomes.iter().filter(|o| o["passed"] == false).collect::<Vec<_>>()})
    );
    assert_eq!(
        passed,
        outcomes.len(),
        "unchanged upstream {name} predicates failed"
    );
}

#[tokio::test]
async fn upstream_imap_basic() {
    digest(
        include_bytes!(env!("STALWART_IMAP_BASIC_SOURCE")),
        "99bbec038face3e9670e78ffddbdb2c902c6fb49b7c68caaacf4467ad9aaf9b7",
        "basic",
    );
    let server = server();
    let mut imap = server.unauthenticated("_x ").await;
    let mut check = server.unauthenticated("_y ").await;
    suite::basic::test(&mut imap, &mut check).await;
    imap.close().await;
    check.close().await;
    finish("basic", &server.outcomes);
}

#[tokio::test]
async fn upstream_imap_store() {
    digest(
        include_bytes!(env!("STALWART_IMAP_STORE_SOURCE")),
        "2def6c03bd890eca23862af4923ba01fbbbb621f4bcd1996816a6a0c2cd2d368",
        "store",
    );
    let server = server();
    let (mut imap, mut check) = seeded(&server).await;
    suite::store::test(&mut imap, &mut check, &server).await;
    imap.close().await;
    check.close().await;
    finish("store", &server.outcomes);
}

#[tokio::test]
async fn upstream_imap_fetch() {
    digest(
        include_bytes!(env!("STALWART_IMAP_FETCH_SOURCE")),
        "111da88835328d11398e9c6eedd8f7f514a15b17950d602cc030d56ad217175a",
        "fetch",
    );
    let server = server();
    let (mut imap, mut check) = seeded(&server).await;
    // Upstream's mailbox suite enabled IMAP4rev2 on `imap` before FETCH runs:
    // the suite expects decoded UTF-8 ENVELOPE strings on `imap` only.
    imap.send("ENABLE IMAP4rev2").await;
    imap.assert_read(Type::Tagged, ResponseType::Ok).await;
    suite::fetch::test(&mut imap, &mut check).await;
    imap.close().await;
    check.close().await;
    finish("fetch", &server.outcomes);
}

#[tokio::test]
async fn upstream_imap_copy_move() {
    digest(
        include_bytes!(env!("STALWART_IMAP_COPY_MOVE_SOURCE")),
        "5109d26d61e6010b8554b5c09cfc8aa49f6cd0d606856206b5a2b33aedf2c64f",
        "copy_move",
    );
    let server = server();
    let (mut imap, mut check) = seeded(&server).await;
    suite::copy_move::test(&mut imap, &mut check).await;
    imap.close().await;
    check.close().await;
    finish("copy_move", &server.outcomes);
}

#[tokio::test]
async fn upstream_imap_idle() {
    digest(
        include_bytes!(env!("STALWART_IMAP_IDLE_SOURCE")),
        "3b93a2663cbc6b8f6b68b109998abcd7290da88ccb59dc65a384f4bdfd15510b",
        "idle",
    );
    let server = server();
    let (mut imap, mut check) = seeded(&server).await;
    suite::idle::test(&mut imap, &mut check, false).await;
    imap.close().await;
    check.close().await;
    finish("idle", &server.outcomes);
}

#[tokio::test]
async fn upstream_imap_mailbox() {
    digest(
        include_bytes!(env!("STALWART_IMAP_MAILBOX_SOURCE")),
        "8522c61c210c6dce3ce2455bbadeed040184d5cbac1a697e775426c2f9947507",
        "mailbox",
    );
    let server = server();
    let mut imap = server.connect(JDOE, "_x ").await;
    let mut check = server.connect(JDOE, "_y ").await;
    suite::mailbox::test(&mut imap, &mut check, &server).await;
    imap.close().await;
    check.close().await;
    finish("mailbox", &server.outcomes);
}
