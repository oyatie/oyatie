//! MODSEQ is the account revision of the committing batch (RFC 7162 §3.1.2).
//! Every changed message and every mailbox it was linked to or unlinked from
//! takes that revision; mailbox-only commits leave the watermarks alone.
use mail_kernel::{Account, Command, Error, HistoryEntry};

fn account() -> Account {
    Account::new("a", "t", "alice", "alice@example.org").unwrap()
}
fn append(mailboxes: &[&str]) -> Command {
    Command::Append {
        mailboxes: mailboxes.iter().map(|id| (*id).into()).collect(),
        received_at: 1,
        raw: b"Subject: retained\r\n\r\nbody".to_vec(),
        keywords: vec![],
    }
}
fn folder(account: &mut Account) -> String {
    account
        .execute(vec![Command::CreateMailbox {
            name: "Work".into(),
        }])
        .unwrap();
    account.mailboxes.last().unwrap().id.clone()
}
fn highest(account: &Account, mailbox: &str) -> u64 {
    account
        .mailboxes
        .iter()
        .find(|m| m.id == mailbox)
        .unwrap()
        .highest_modseq
}

#[test]
fn modseq_is_the_commit_revision_and_mailbox_only_commits_keep_watermarks() {
    let mut account = account();
    let folder = folder(&mut account);
    assert_eq!(account.revision, 1);
    assert_eq!(account.mail_modseq, 0);
    assert_eq!(highest(&account, "inbox"), 0);
    // A new mailbox inherits the account watermark (empty SELECT reports it).
    assert_eq!(highest(&account, &folder), 0);
    let effects = account.execute(vec![append(&["inbox"])]).unwrap();
    assert_eq!(effects.revision, 2);
    assert_eq!(account.mail_modseq, 2);
    assert_eq!(account.messages[0].modseq, 2);
    assert_eq!(account.messages[0].created_revision, 2);
    assert_eq!(highest(&account, "inbox"), 2);
    assert_eq!(highest(&account, &folder), 0);
    assert_eq!(effects.ids, vec!["e2".to_string()]);
    assert_eq!(effects.allocations, vec![("inbox".to_string(), 1)]);
    let id = account.messages[0].id.clone();
    account
        .execute(vec![Command::RenameMailbox {
            id: folder.clone(),
            name: "Renamed".into(),
        }])
        .unwrap();
    assert_eq!(account.revision, 3);
    assert_eq!(account.mail_modseq, 2);
    assert_eq!(highest(&account, "inbox"), 2);
    let effects = account
        .execute(vec![Command::Keywords {
            id: id.clone(),
            keywords: vec!["$seen".into()],
        }])
        .unwrap();
    assert_eq!(
        effects.history,
        vec![HistoryEntry::Flags { id: id.clone() }]
    );
    assert_eq!(account.messages[0].modseq, 4);
    assert_eq!(account.mail_modseq, 4);
    // Identical keywords and memberships are not a change.
    let effects = account
        .execute(vec![
            Command::Keywords {
                id: id.clone(),
                keywords: vec!["$seen".into(), "$seen".into()],
            },
            Command::SetMailboxes {
                id: id.clone(),
                mailboxes: vec!["inbox".into()],
            },
        ])
        .unwrap();
    assert!(effects.history.is_empty());
    assert!(!effects.mail_changed);
    assert_eq!(account.revision, 6);
    assert_eq!(account.mail_modseq, 4);
    let effects = account
        .execute(vec![Command::SetMailboxes {
            id: id.clone(),
            mailboxes: vec!["inbox".into(), folder.clone()],
        }])
        .unwrap();
    assert_eq!(
        effects.history,
        vec![HistoryEntry::Added {
            id: id.clone(),
            mailbox: folder.clone(),
            uid: 1
        }]
    );
    assert_eq!(account.messages[0].modseq, 7);
    assert_eq!(highest(&account, &folder), 7);
    assert_eq!(highest(&account, "inbox"), 7);
    assert_eq!(account.messages[0].created_revision, 2);
}

#[test]
fn last_unlink_deletes_the_record_and_emits_only_removed_rows() {
    let mut account = account();
    let folder = folder(&mut account);
    account.execute(vec![append(&["inbox", &folder])]).unwrap();
    account.execute(vec![append(&["inbox"])]).unwrap();
    let first = account.messages[0].id.clone();
    let second = account.messages[1].id.clone();
    let second_modseq = account.messages[1].modseq;
    let effects = account
        .execute(vec![Command::RemoveMailbox {
            id: folder.clone(),
            remove_emails: true,
        }])
        .unwrap();
    assert_eq!(
        effects.history,
        vec![
            HistoryEntry::Removed {
                id: first.clone(),
                mailbox: folder.clone(),
                uid: 1,
                thread: first.clone()
            },
            HistoryEntry::Mailbox { id: folder.clone() }
        ]
    );
    assert!(effects.deleted.is_empty());
    assert_eq!(account.messages[0].modseq, account.revision);
    assert_eq!(account.messages[1].modseq, second_modseq);
    let watermark = account.mail_modseq;
    let effects = account
        .execute(vec![Command::Expunge {
            mailbox: "inbox".into(),
        }])
        .unwrap();
    assert!(effects.history.is_empty());
    assert_eq!(account.mail_modseq, watermark);
    account
        .execute(vec![Command::Keywords {
            id: first.clone(),
            keywords: vec!["$deleted".into()],
        }])
        .unwrap();
    let effects = account
        .execute(vec![Command::Expunge {
            mailbox: "inbox".into(),
        }])
        .unwrap();
    assert_eq!(effects.deleted, vec![first.clone()]);
    assert_eq!(
        effects.history,
        vec![HistoryEntry::Removed {
            id: first.clone(),
            mailbox: "inbox".into(),
            uid: 1,
            thread: first
        }]
    );
    assert_eq!(account.messages.len(), 1);
    assert_eq!(account.messages[0].modseq, second_modseq);
    assert_eq!(highest(&account, "inbox"), account.revision);
    assert_eq!(account.mailboxes[0].total_emails, 1);
    let effects = account
        .execute(vec![Command::Destroy { id: second.clone() }])
        .unwrap();
    assert_eq!(effects.deleted, vec![second]);
    assert!(account.messages.is_empty());
    assert_eq!(account.used_bytes, 0);
    assert_eq!(account.mailboxes[0].total_emails, 0);
    assert_eq!(account.mailboxes[0].size_bytes, 0);
}

#[test]
fn copy_and_move_allocate_uids_and_stamp_both_records() {
    let mut account = account();
    let folder = folder(&mut account);
    account.execute(vec![append(&["inbox", &folder])]).unwrap();
    let id = account.messages[0].id.clone();
    let source_modseq = account.messages[0].modseq;
    let effects = account
        .execute(vec![Command::Transfer {
            id: id.clone(),
            mailbox: folder.clone(),
            remove_from: None,
        }])
        .unwrap();
    assert_eq!(effects.allocations, vec![(folder.clone(), 2)]);
    assert_eq!(effects.ids, vec![account.messages[1].id.clone()]);
    assert_eq!(account.messages[0].modseq, source_modseq);
    assert_eq!(account.messages[1].modseq, account.revision);
    assert_eq!(account.messages[1].created_revision, account.revision);
    let copy_modseq = account.messages[1].modseq;
    let effects = account
        .execute(vec![Command::Transfer {
            id,
            mailbox: folder.clone(),
            remove_from: Some("inbox".into()),
        }])
        .unwrap();
    assert_eq!(effects.allocations, vec![(folder.clone(), 3)]);
    assert_eq!(account.messages[0].modseq, account.revision);
    assert_eq!(account.messages[1].modseq, copy_modseq);
    assert_eq!(account.messages[2].modseq, account.revision);
    assert_eq!(highest(&account, "inbox"), account.revision);
    assert_eq!(account.mailboxes[0].total_emails, 0);
    assert_eq!(account.mailboxes[1].total_emails, 3);
    assert_eq!(account.used_bytes, 3 * account.messages[0].size);
}

#[test]
fn overflow_and_invalid_mutations_preserve_account_and_uid_allocators() {
    let mut account = account();
    account.execute(vec![append(&["inbox"])]).unwrap();
    let before = account.clone();
    assert_eq!(
        account.execute(vec![append(&["missing"])]),
        Err(Error::NotFound)
    );
    assert_eq!(account, before);
    assert_eq!(
        account.execute(vec![append(&["inbox"]), append(&["missing"])]),
        Err(Error::NotFound)
    );
    assert_eq!(account, before);
    assert_eq!(account.mailboxes[0].uid_next, 2);
    account.revision = u64::MAX;
    let before = account.clone();
    assert_eq!(
        account.execute(vec![append(&["inbox"])]),
        Err(Error::OverQuota)
    );
    assert_eq!(account, before);
}
