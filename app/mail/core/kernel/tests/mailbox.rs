use mail_kernel::{Account, BlobRef, Command, Error, MailboxProperties};

/// A reference the kernel charges by size; the hash is irrelevant to it.
fn blob_of(raw: impl AsRef<[u8]>) -> BlobRef {
    BlobRef {
        hash: "h".into(),
        version_id: "1".into(),
        size: raw.as_ref().len(),
    }
}

#[test]
fn hierarchy_refuses_cycles_and_preserves_messages_on_rejected_deletion() {
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account
        .apply(Command::CreateMailbox {
            name: "Parent".into(),
        })
        .unwrap();
    let mut child = MailboxProperties::named("Child".into());
    child.parent_id = Some("m1".into());
    account
        .apply(Command::SetMailbox {
            id: None,
            properties: child.clone(),
        })
        .unwrap();
    assert_eq!(account.mailbox_path("m2").as_deref(), Some("Parent/Child"));
    assert_eq!(
        account.apply(Command::SetMailbox {
            id: None,
            properties: child
        }),
        Err(Error::Conflict)
    );
    let mut parent = account.mailboxes[1].properties();
    parent.parent_id = Some("m2".into());
    let before = account.clone();
    assert_eq!(
        account.apply(Command::SetMailbox {
            id: Some("m1".into()),
            properties: parent
        }),
        Err(Error::Invalid)
    );
    assert_eq!(account, before);
    account
        .apply(Command::Append {
            mailboxes: vec!["m1".into()],
            received_at: 0,
            blob: blob_of(vec![255]),
            keywords: vec![],
        })
        .unwrap();
    account
        .apply(Command::Append {
            mailboxes: vec!["inbox".into(), "m2".into()],
            received_at: 0,
            blob: blob_of(vec![0, 1]),
            keywords: vec![],
        })
        .unwrap();
    let before = account.clone();
    assert_eq!(
        account.apply(Command::RemoveMailbox {
            id: "m1".into(),
            remove_emails: true
        }),
        Err(Error::Conflict)
    );
    assert_eq!(account, before);
    account
        .apply(Command::RemoveMailbox {
            id: "m2".into(),
            remove_emails: true,
        })
        .unwrap();
    assert_eq!(account.messages[1].uid_in("inbox"), Some(1));
    assert_eq!(account.messages[1].size, 2);
    account
        .apply(Command::RemoveMailbox {
            id: "m1".into(),
            remove_emails: true,
        })
        .unwrap();
    assert_eq!(account.messages.len(), 1);
    assert_eq!(account.mailbox_states()[0].total_emails, 1);
}

#[test]
fn mailbox_memberships_preserve_existing_uids_and_expunge_only_the_selected_mailbox() {
    let mut account = Account::new("a", "t", "owner", "a@example.org").unwrap();
    account
        .apply(Command::CreateMailbox {
            name: "Archive".into(),
        })
        .unwrap();
    let archive = account.mailboxes[1].id.clone();
    account
        .apply(Command::Append {
            mailboxes: vec!["inbox".into()],
            received_at: 0,
            blob: blob_of(b"Subject: shared\r\n\r\nbody".to_vec()),
            keywords: vec![],
        })
        .unwrap();
    let id = account.messages[0].id.clone();
    account
        .apply(Command::SetMailboxes {
            id: id.clone(),
            mailboxes: vec!["inbox".into(), archive.clone()],
        })
        .unwrap();
    assert_eq!(account.messages[0].uid_in("inbox"), Some(1));
    assert_eq!(account.messages[0].uid_in(&archive), Some(1));
    let before = account.clone();
    assert_eq!(
        account.apply(Command::SetMailboxes {
            id: id.clone(),
            mailboxes: vec![archive.clone(), "missing".into()]
        }),
        Err(Error::NotFound)
    );
    assert_eq!(account, before);
    account
        .apply(Command::Keywords {
            id: id.clone(),
            keywords: vec!["$deleted".into()],
        })
        .unwrap();
    account
        .apply(Command::Expunge {
            mailbox: "inbox".into(),
        })
        .unwrap();
    assert_eq!(account.messages.len(), 1);
    assert_eq!(account.messages[0].uid_in("inbox"), None);
    assert_eq!(account.messages[0].uid_in(&archive), Some(1));
    account
        .apply(Command::SetMailboxes {
            id,
            mailboxes: vec!["inbox".into(), archive.clone()],
        })
        .unwrap();
    assert_eq!(account.messages[0].uid_in("inbox"), Some(2));
    assert_eq!(account.messages[0].uid_in(&archive), Some(1));
}

#[test]
fn mailbox_lifecycle_preserves_uids_state_and_message_size() {
    let mut account = Account::new("a", "tenant-a", "alice", "alice@example.org").unwrap();
    let raw = b"Subject: hello\r\n\r\n\xff\0body\r\n".to_vec();
    let inbox = account.inbox().to_owned();
    account
        .apply(Command::Append {
            mailboxes: vec![inbox.clone()],
            received_at: 0,
            blob: blob_of(raw.clone()),
            keywords: vec![],
        })
        .unwrap();
    assert_eq!(account.revision, 1);
    assert_eq!(account.messages[0].uid_in("inbox").unwrap(), 1);
    assert_eq!(account.messages[0].size, raw.len());
    let first = account.messages[0].id.clone();
    account
        .apply(Command::Keywords {
            id: first,
            keywords: vec!["$seen".into(), "$deleted".into()],
        })
        .unwrap();
    account
        .apply(Command::Expunge {
            mailbox: inbox.clone(),
        })
        .unwrap();
    account
        .apply(Command::Append {
            mailboxes: vec![inbox],
            received_at: 0,
            blob: blob_of(&raw),
            keywords: vec![],
        })
        .unwrap();
    assert_eq!(account.messages.len(), 1);
    assert_eq!(account.messages[0].uid_in("inbox").unwrap(), 2);
    assert_eq!(account.revision, 4);
}

#[test]
fn failed_command_does_not_change_state_and_inbox_cannot_be_destroyed() {
    let mut account = Account::new("a", "tenant-a", "alice", "alice@example.org").unwrap();
    let original = account.clone();
    assert_eq!(
        account.apply(Command::DeleteMailbox {
            id: account.inbox().into()
        }),
        Err(Error::Forbidden)
    );
    assert_eq!(account, original);
    assert_eq!(
        account.apply(Command::Append {
            mailboxes: vec!["missing".into()],
            received_at: 0,
            blob: blob_of(vec![1]),
            keywords: vec![]
        }),
        Err(Error::NotFound)
    );
    assert_eq!(account, original);
    assert!(Account::new("", "tenant-a", "alice", "alice@example.org").is_err());
    assert!(Account::new("a", "tenant-a", "alice", "alice@example.org\r\nRCPT TO:x").is_err());
}

#[test]
fn mailbox_names_and_quota_are_checked_before_mutation() {
    let mut account = Account::new("a", "tenant-a", "alice", "alice@example.org").unwrap();
    account.quota_bytes = 3;
    let original = account.clone();
    assert_eq!(
        account.apply(Command::CreateMailbox {
            name: "bad\r\nname".into()
        }),
        Err(Error::Invalid)
    );
    assert_eq!(
        account.apply(Command::Append {
            mailboxes: vec![account.inbox().into()],
            received_at: 0,
            blob: blob_of(vec![0; 4]),
            keywords: vec![]
        }),
        Err(Error::OverQuota)
    );
    assert_eq!(account, original);
}

#[test]
fn uid_validity_exhaustion_is_an_error_without_mutation() {
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.revision = u64::MAX - 1;
    let original = account.clone();
    assert_eq!(
        account.apply(Command::CreateMailbox {
            name: "exhausted".into()
        }),
        Err(Error::OverQuota)
    );
    assert_eq!(account, original);
}
#[test]
fn repeated_existing_memberships_are_invalid_and_leave_the_account_unchanged() {
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account
        .apply(Command::Append {
            mailboxes: vec!["inbox".into()],
            received_at: 0,
            blob: blob_of(vec![]),
            keywords: vec![],
        })
        .unwrap();
    let before = account.clone();
    assert_eq!(
        account.apply(Command::SetMailboxes {
            id: "e1".into(),
            mailboxes: vec!["inbox".into(), "inbox".into()]
        }),
        Err(Error::Invalid)
    );
    assert_eq!(account, before);
}
