use mail_kernel::{Account, Command, Error};

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
        .apply(Command::CreateMailbox {
            name: "Work".into(),
        })
        .unwrap();
    account.mailboxes.last().unwrap().id.clone()
}

#[test]
fn mailbox_changes_preserve_highest_modseq_and_message_changes_track_revision() {
    let mut account = account();
    assert_eq!(account.mail_modseq, 0);
    let folder = folder(&mut account);
    assert_eq!(account.mail_modseq, 0);
    assert_eq!(account.revision, 1);
    account.apply(append(&["inbox"])).unwrap();
    assert_eq!(account.mail_modseq, account.revision + 1);
    let id = account.messages[0].id.clone();
    assert_eq!(account.messages[0].modseq, account.mail_modseq);
    let previous = account.mail_modseq;
    account
        .apply(Command::RenameMailbox {
            id: folder.clone(),
            name: "Renamed".into(),
        })
        .unwrap();
    assert_eq!(account.mail_modseq, previous);
    account
        .apply(Command::Keywords {
            id: id.clone(),
            keywords: vec!["$seen".into()],
        })
        .unwrap();
    assert!(account.mail_modseq > previous);
    assert_eq!(account.messages[0].modseq, account.revision + 1);
    let previous = account.mail_modseq;
    account
        .apply(Command::Keywords {
            id: id.clone(),
            keywords: vec!["$seen".into(), "$seen".into()],
        })
        .unwrap();
    account
        .apply(Command::SetMailboxes {
            id: id.clone(),
            mailboxes: vec!["inbox".into()],
        })
        .unwrap();
    assert_eq!(account.mail_modseq, previous);
    account
        .apply(Command::SetMailboxes {
            id,
            mailboxes: vec!["inbox".into(), folder],
        })
        .unwrap();
    assert_eq!(account.messages[0].modseq, account.revision + 1);
    assert!(account.mail_modseq > previous);
    let state = account.messages[0].state();
    assert_eq!(state.modseq, account.messages[0].modseq);
    assert_eq!(state.uids, account.messages[0].mailboxes);
}

#[test]
fn deletion_expunge_and_mailbox_removal_advance_modseq_only_for_changed_messages() {
    let mut account = account();
    let folder = folder(&mut account);
    account.apply(append(&["inbox", &folder])).unwrap();
    account.apply(append(&["inbox"])).unwrap();
    let first = account.messages[0].id.clone();
    let second = account.messages[1].id.clone();
    let second_modseq = account.messages[1].modseq;
    account
        .apply(Command::RemoveMailbox {
            id: folder,
            remove_emails: true,
        })
        .unwrap();
    assert_eq!(account.messages[0].modseq, account.mail_modseq);
    assert_eq!(account.messages[1].modseq, second_modseq);
    let previous = account.mail_modseq;
    account
        .apply(Command::Expunge {
            mailbox: "inbox".into(),
        })
        .unwrap();
    assert_eq!(account.mail_modseq, previous);
    account
        .apply(Command::Keywords {
            id: first,
            keywords: vec!["$deleted".into()],
        })
        .unwrap();
    let previous = account.mail_modseq;
    account
        .apply(Command::Expunge {
            mailbox: "inbox".into(),
        })
        .unwrap();
    assert!(account.mail_modseq > previous);
    assert_eq!(account.messages.len(), 1);
    assert_eq!(account.messages[0].modseq, second_modseq);
    let previous = account.mail_modseq;
    account.apply(Command::Destroy { id: second }).unwrap();
    assert!(account.mail_modseq > previous);
    assert!(account.messages.is_empty());
}

#[test]
fn copy_and_move_stamp_new_messages_and_membership_changes() {
    let mut account = account();
    let folder = folder(&mut account);
    account.apply(append(&["inbox", &folder])).unwrap();
    let id = account.messages[0].id.clone();
    let source_modseq = account.messages[0].modseq;
    account
        .apply(Command::Transfer {
            id: id.clone(),
            mailbox: folder.clone(),
            remove_from: None,
        })
        .unwrap();
    assert_eq!(account.messages[0].modseq, source_modseq);
    assert_eq!(account.messages[1].modseq, account.mail_modseq);
    let copy_modseq = account.messages[1].modseq;
    account
        .apply(Command::Transfer {
            id,
            mailbox: folder,
            remove_from: Some("inbox".into()),
        })
        .unwrap();
    assert_eq!(account.messages[0].modseq, account.mail_modseq);
    assert_eq!(account.messages[1].modseq, copy_modseq);
    assert_eq!(account.messages[2].modseq, account.mail_modseq);
}

#[test]
fn overflow_and_invalid_mutations_preserve_account_and_uid_allocators() {
    let mut account = account();
    account.apply(append(&["inbox"])).unwrap();
    account.mail_modseq = u64::MAX;
    let before = account.clone();
    for command in [
        append(&["inbox"]),
        Command::Keywords {
            id: before.messages[0].id.clone(),
            keywords: vec!["$seen".into()],
        },
        Command::Destroy {
            id: before.messages[0].id.clone(),
        },
    ] {
        assert_eq!(account.apply(command), Err(Error::OverQuota));
        assert_eq!(account, before);
    }
    account.mail_modseq = 2;
    let before = account.clone();
    assert_eq!(account.apply(append(&["missing"])), Err(Error::NotFound));
    assert_eq!(account, before);
    account.revision = u64::MAX - 1;
    let before = account.clone();
    assert_eq!(account.apply(append(&["inbox"])), Err(Error::OverQuota));
    assert_eq!(account, before);
}

#[test]
fn legacy_serialized_accounts_and_journal_states_load_with_initial_modseq() {
    let mut empty = serde_json::to_value(account()).unwrap();
    empty.as_object_mut().unwrap().remove("mail_modseq");
    assert_eq!(
        serde_json::from_value::<Account>(empty)
            .unwrap()
            .mail_modseq,
        0
    );
    let mut original = account();
    original.apply(append(&["inbox"])).unwrap();
    let mut legacy = serde_json::to_value(&original).unwrap();
    legacy.as_object_mut().unwrap().remove("mail_modseq");
    for message in legacy["messages"].as_array_mut().unwrap() {
        message.as_object_mut().unwrap().remove("modseq");
    }
    let mut restored: Account = serde_json::from_value(legacy).unwrap();
    assert_eq!(restored.mail_modseq, 1);
    assert_eq!(restored.messages[0].modseq, 1);
    restored
        .apply(Command::Keywords {
            id: restored.messages[0].id.clone(),
            keywords: vec!["$seen".into()],
        })
        .unwrap();
    assert_eq!(restored.mail_modseq, restored.revision + 1);
    assert_eq!(restored.messages[0].modseq, restored.mail_modseq);
    let encoded = serde_json::to_vec(&restored).unwrap();
    assert_eq!(
        serde_json::from_slice::<Account>(&encoded).unwrap(),
        restored
    );
    let mut state = serde_json::to_value(restored.messages[0].state()).unwrap();
    state.as_object_mut().unwrap().remove("modseq");
    state.as_object_mut().unwrap().remove("uids");
    let old: mail_kernel::MessageState = serde_json::from_value(state).unwrap();
    assert_eq!(old.modseq, 1);
    assert!(old.uids.is_empty());
}
