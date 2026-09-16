use mail_kernel::{Account, Command, MailboxProperties};
use std::collections::BTreeMap;

fn append() -> Command {
    Command::Append {
        mailboxes: vec!["inbox".into()],
        raw: b"Subject: stable\r\n\r\nbody".to_vec(),
        received_at: 1,
        keywords: vec![],
    }
}
#[test]
fn message_state_changes_and_failures_preserve_modseq_invariants() {
    let mut initial = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    initial.apply(append()).unwrap();
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
                if account.apply(commands().remove(command)).is_err() {
                    assert_eq!(account, before);
                    continue;
                }
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
                            account.revision + 1
                        } else {
                            prior.unwrap().modseq
                        }
                    );
                    assert!(message.modseq <= account.mail_modseq);
                }
                assert_eq!(
                    account.mail_modseq,
                    if changed {
                        account.revision + 1
                    } else {
                        before.mail_modseq
                    }
                );
            }
        }
    }
}

#[test]
fn legacy_missing_watermark_is_distinguished_from_malformed_present_watermark() {
    let original = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    let mut json = serde_json::to_value(&original).unwrap();
    json.as_object_mut().unwrap().remove("mail_modseq");
    assert_eq!(
        serde_json::from_value::<Account>(json.clone())
            .unwrap()
            .mail_modseq,
        0
    );
    for invalid in [
        serde_json::Value::Null,
        serde_json::json!("0"),
        serde_json::json!(-1),
    ] {
        json["mail_modseq"] = invalid;
        assert!(
            serde_json::from_value::<Account>(json.clone()).is_err(),
            "explicit malformed watermark accepted: {json}"
        );
    }
}

#[test]
fn final_commit_stamp_overflow_is_atomic_even_after_valid_message_edits() {
    let mut before = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    before.apply(append()).unwrap();
    let mut candidate = before.clone();
    candidate.messages[0].keywords.push("$seen".into());
    candidate.revision = u64::MAX;
    let unchanged = candidate.clone();
    assert_eq!(
        candidate.complete_batch(&before),
        Err(mail_kernel::Error::OverQuota)
    );
    assert_eq!(candidate, unchanged);
}
