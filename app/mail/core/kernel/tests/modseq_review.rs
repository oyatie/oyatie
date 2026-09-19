//! Pairwise review: after any two commands, every message's MODSEQ is the
//! revision of the batch that last changed it, counters equal a recount of the
//! working set, and a failed command leaves the aggregate untouched.
use mail_kernel::{Account, BlobRef, Command, MailboxProperties, Scope};
use std::collections::BTreeMap;

/// A reference the kernel charges by size; the hash is irrelevant to it.
fn blob_of(raw: impl AsRef<[u8]>) -> BlobRef {
    BlobRef {
        hash: "h".into(),
        version_id: "1".into(),
        size: raw.as_ref().len(),
    }
}

fn append() -> Command {
    Command::Append {
        mailboxes: vec!["inbox".into()],
        blob: blob_of(b"Subject: stable\r\n\r\nbody".to_vec()),
        received_at: 1,
        keywords: vec![],
    }
}

fn recount(account: &Account) {
    let mut used = 0;
    let mut totals: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    for message in &account.messages {
        used += message.size;
        for mailbox in message.mailboxes.keys() {
            let entry = totals.entry(mailbox).or_default();
            entry.0 += 1;
            entry.1 += usize::from(!message.seen());
            entry.2 += message.size;
        }
    }
    assert_eq!(account.used_bytes, used);
    for mailbox in &account.mailboxes {
        let (total, unread, size) = totals.get(mailbox.id.as_str()).copied().unwrap_or_default();
        assert_eq!(mailbox.total_emails, total, "{}", mailbox.id);
        assert_eq!(mailbox.unread_emails, unread, "{}", mailbox.id);
        assert_eq!(mailbox.size_bytes, size, "{}", mailbox.id);
    }
}

#[test]
fn message_state_changes_and_failures_preserve_modseq_invariants() {
    let mut initial = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    initial.execute(vec![append()]).unwrap();
    let id = initial.messages[0].id.clone();
    let commands = || {
        vec![
            Command::Keywords {
                id: id.clone(),
                keywords: vec![],
            },
            Command::Keywords {
                id: id.clone(),
                keywords: vec!["$seen".into()],
            },
            Command::Keywords {
                id: id.clone(),
                keywords: vec!["$seen".into(), "$seen".into()],
            },
            Command::Keywords {
                id: "missing".into(),
                keywords: vec!["$seen".into()],
            },
            Command::SetMailboxes {
                id: id.clone(),
                mailboxes: vec!["inbox".into()],
            },
            Command::SetMailboxes {
                id: id.clone(),
                mailboxes: vec!["missing".into()],
            },
            Command::Destroy { id: id.clone() },
            Command::Destroy {
                id: "missing".into(),
            },
            Command::Expunge {
                mailbox: "inbox".into(),
            },
            Command::Expunge {
                mailbox: "missing".into(),
            },
            Command::Transfer {
                id: id.clone(),
                mailbox: "inbox".into(),
                remove_from: None,
            },
            Command::CreateMailbox {
                name: "Work".into(),
            },
            Command::SetMailbox {
                id: None,
                properties: MailboxProperties::named("Work".into()),
            },
            append(),
        ]
    };
    for first in 0..14 {
        for second in 0..14 {
            let mut account = initial.clone();
            for command in [first, second] {
                let before = account.clone();
                let Ok(effects) = account.execute(vec![commands().remove(command)]) else {
                    assert_eq!(account, before);
                    continue;
                };
                recount(&account);
                assert_eq!(effects.revision, account.revision);
                let old = before
                    .messages
                    .iter()
                    .map(|m| (m.id.clone(), m.state()))
                    .collect::<BTreeMap<_, _>>();
                let mut changed = old
                    .keys()
                    .any(|id| !account.messages.iter().any(|m| &m.id == id));
                for message in &account.messages {
                    let prior = old.get(&message.id);
                    let mut state = message.state();
                    if let Some(prior) = prior {
                        state.modseq = prior.modseq;
                    }
                    let modified = prior.is_none_or(|prior| *prior != state);
                    changed |= modified;
                    assert_eq!(
                        message.modseq,
                        if modified {
                            account.revision
                        } else {
                            prior.unwrap().modseq
                        }
                    );
                    assert!(message.modseq <= account.mail_modseq);
                }
                assert_eq!(changed, effects.mail_changed);
                assert_eq!(
                    account.mail_modseq,
                    if changed {
                        account.revision
                    } else {
                        before.mail_modseq
                    }
                );
                for mailbox in &account.mailboxes {
                    let touched = effects.history.iter().any(|e| match e {
                        mail_kernel::HistoryEntry::Added { mailbox: m, .. }
                        | mail_kernel::HistoryEntry::Removed { mailbox: m, .. } => *m == mailbox.id,
                        mail_kernel::HistoryEntry::Flags { id }
                        | mail_kernel::HistoryEntry::Thread { id } => account
                            .messages
                            .iter()
                            .any(|m| m.id == *id && m.mailboxes.contains_key(&mailbox.id)),
                        mail_kernel::HistoryEntry::Mailbox { .. } => false,
                    });
                    if touched {
                        assert_eq!(mailbox.highest_modseq, account.revision);
                    }
                }
            }
        }
    }
}

#[test]
fn scopes_name_exactly_the_records_a_command_reads() {
    assert_eq!(append().scope(), Scope::None);
    assert_eq!(
        Command::Destroy { id: "e1".into() }.scope(),
        Scope::Message("e1")
    );
    assert_eq!(
        Command::Expunge {
            mailbox: "inbox".into()
        }
        .scope(),
        Scope::Flagged("inbox", "$deleted")
    );
    assert_eq!(
        Command::RemoveMailbox {
            id: "m1".into(),
            remove_emails: true
        }
        .scope(),
        Scope::Mailbox("m1")
    );
    assert_eq!(
        Command::RemoveMailbox {
            id: "m1".into(),
            remove_emails: false
        }
        .scope(),
        Scope::None
    );
}

#[test]
fn commit_refuses_a_foreign_or_rewound_baseline() {
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.execute(vec![append()]).unwrap();
    let mut other = Account::new("b", "t", "alice", "b@example.org").unwrap();
    assert_eq!(other.commit(&account), Err(mail_kernel::Error::Conflict));
    let mut rewound = account.clone();
    rewound.revision = 0;
    assert_eq!(rewound.commit(&account), Err(mail_kernel::Error::Conflict));
}
