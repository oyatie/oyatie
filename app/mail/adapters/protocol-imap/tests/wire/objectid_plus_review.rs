use super::*;
use mail_api::{AccountInfo, Action, Policy, Principal};
use mail_kernel::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn enabling_extensions_after_selection_updates_fetch_store_and_vanished_behavior() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: selected enable\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "s SELECT INBOX").await;
    command(&mut client, "c ENABLE CONDSTORE").await;
    let updated = command(&mut client, "c1 STORE 1 +FLAGS (\\Seen)").await;
    assert!(
        updated.contains("c1 OK") && updated.contains("MODSEQ ("),
        "{updated}"
    );
    let enabled = command(&mut client, "e ENABLE QRESYNC OBJECTID+").await;
    assert!(enabled.contains("e OK"), "{enabled}");
    let fetched = command(
        &mut client,
        "f UID FETCH 1 (OBJECTID FLAGS) (CHANGEDSINCE 0 VANISHED)",
    )
    .await;
    assert!(
        fetched.contains("f OK") && fetched.contains("EMAILID"),
        "{fetched}"
    );
    let stored = command(&mut client, "w STORE 1 +FLAGS (\\Flagged)").await;
    assert!(
        stored.contains("w OK") && stored.contains("MODSEQ ("),
        "{stored}"
    );
    let account = db.account("a").unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(account.revision),
        vec![mail_kernel::Command::Destroy {
            id: account.messages[0].id.clone(),
        }],
    )
    .unwrap();
    let changed = command(&mut client, "n NOOP").await;
    assert!(
        changed.contains("* VANISHED 1") && !changed.contains("EXPUNGE"),
        "{changed}"
    );
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn malformed_enable_is_atomic_and_does_not_activate_preceding_capabilities() {
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    for invalid in ["OBJECTID+ UNKNOWN", "OBJECTID+ (CONDSTORE)", "OBJECTID+ )"] {
        let refused = command(&mut client, &format!("bad ENABLE {invalid}")).await;
        assert!(
            refused.contains("bad BAD") && !refused.contains("ENABLED"),
            "{refused}"
        );
        let selected = command(&mut client, "s SELECT INBOX").await;
        assert!(
            !selected.contains("OBJECTID") && !selected.contains("HIGHESTMODSEQ"),
            "{selected}"
        );
    }
    let accepted = command(&mut client, "yes ENABLE OBJECTID+").await;
    assert!(accepted.contains("yes OK") && accepted.contains("ENABLED OBJECTID+"));
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn unsuccessful_identifier_selection_preserves_the_previous_readonly_mailbox() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: retained\r\n\r\nprivate body",
    )
    .unwrap();
    let before = db.account("a").unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    let examined = command(&mut client, "s EXAMINE INBOX").await;
    assert!(examined.contains("READ-ONLY"));
    for arguments in [
        "nonexistent (OBJECTID (ACCOUNTID foreign MAILBOXID invalid))",
        "nonexistent (OBJECTID)",
        "nonexistent",
    ] {
        let failed = command(&mut client, &format!("bad SELECT {arguments}")).await;
        assert!(failed.contains("bad NO"), "{failed}");
        let retained = command(&mut client, "f UID FETCH 1 (OBJECTID BODY[])").await;
        assert!(
            retained.contains("f OK") && retained.contains("private body"),
            "{retained}"
        );
        let denied = command(&mut client, "w STORE 1 +FLAGS (\\Seen)").await;
        assert!(denied.contains("w NO"), "{denied}");
        assert_eq!(db.account("a").unwrap(), before);
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn real_foreign_identifiers_only_fall_back_to_the_authorized_local_name() {
    let (service, db) = service();
    let foreign_token = "abcdef0123456789abcdef0123456789";
    db.provision(
        Account::new("b", "foreign-tenant", "bob", "bob@example.org").unwrap(),
        foreign_token,
    )
    .unwrap();
    db.deliver(
        &["bob@example.org".into()],
        b"Subject: foreign\r\n\r\nforeign private body",
    )
    .unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: local\r\n\r\nlocal private body",
    )
    .unwrap();
    let (foreign, server) = tokio::io::duplex(65536);
    let foreign_task = tokio::spawn(mail_protocol_imap::imap_session(
        server,
        service.clone(),
        true,
    ));
    let mut foreign = BufReader::new(foreign);
    command(
        &mut foreign,
        &format!("a LOGIN bob@example.org {foreign_token}"),
    )
    .await;
    let status = command(&mut foreign, "s STATUS INBOX (OBJECTID)").await;
    let mailbox = id(&status, "MAILBOXID ");
    command(&mut foreign, "z LOGOUT").await;
    foreign_task.await.unwrap().unwrap();

    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    for account in ["a", "b"] {
        let failed = command(
            &mut client,
            &format!("bad SELECT nonexistent (OBJECTID (ACCOUNTID {account} MAILBOXID {mailbox}))"),
        )
        .await;
        assert!(
            failed.contains("bad NO") && !failed.contains(&mailbox),
            "{failed}"
        );
        let selected = command(
            &mut client,
            &format!("s SELECT INBOX (OBJECTID (ACCOUNTID {account} MAILBOXID {mailbox}))"),
        )
        .await;
        assert!(
            selected.contains("s OK")
                && selected.contains("ACCOUNTID a ")
                && !selected.contains(&mailbox),
            "{selected}"
        );
        let content = command(&mut client, "f UID FETCH 1 (OBJECTID BODY.PEEK[])").await;
        assert!(
            content.contains("local private body") && !content.contains("foreign private body"),
            "{content}"
        );
        assert!(
            !content.contains("ACCOUNTID"),
            "FETCH compound must omit account id: {content}"
        );
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

struct MutablePolicy(Arc<AtomicUsize>);
impl Policy for MutablePolicy {
    fn authorize(
        &self,
        principal: &Principal,
        action: Action,
        account: &AccountInfo,
    ) -> Result<(), Error> {
        OwnerPolicy.authorize(principal, action, account)?;
        if self.0.load(Ordering::SeqCst) == 2
            || (self.0.load(Ordering::SeqCst) == 1 && action != Action::Read)
        {
            Err(Error::Forbidden)
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn identifier_reads_preserve_readonly_policy_and_recheck_revocation_after_activation() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: policy\r\n\r\nread-only body",
    )
    .unwrap();
    let before = db.account("a").unwrap();
    let policy = Arc::new(AtomicUsize::new(1));
    let mut service = Arc::try_unwrap(service).ok().unwrap();
    service.policy = Arc::new(MutablePolicy(policy.clone()));
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(
        server,
        Arc::new(service),
        true,
    ));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    let selected = command(&mut client, "s SELECT INBOX (OBJECTID)").await;
    assert!(
        selected.contains("READ-ONLY") && selected.contains("ACCOUNTID a "),
        "{selected}"
    );
    let fetched = command(&mut client, "f FETCH 1 (OBJECTID BODY[])").await;
    assert!(
        fetched.contains("read-only body") && fetched.contains("EMAILID"),
        "{fetched}"
    );
    assert_eq!(db.account("a").unwrap(), before);
    let write = command(&mut client, "w CREATE forbidden").await;
    assert!(
        write.contains("w NO") && !write.contains("OBJECTID"),
        "{write}"
    );
    policy.store(2, Ordering::SeqCst);
    for command_text in [
        "SELECT INBOX (OBJECTID)",
        "STATUS INBOX (OBJECTID)",
        "FETCH 1 (OBJECTID)",
        "ENABLE OBJECTID+",
    ] {
        let refused = command(&mut client, &format!("deny {command_text}")).await;
        assert!(
            refused.contains("deny NO")
                && !refused.contains("ACCOUNTID")
                && !refused.contains("EMAILID")
                && !refused.contains("ENABLED"),
            "{refused}"
        );
    }
    assert_eq!(db.account("a").unwrap(), before);
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
