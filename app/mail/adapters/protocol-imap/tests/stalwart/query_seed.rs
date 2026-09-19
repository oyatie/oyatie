// Loads the upstream Email/query corpus into the store with now-relative
// receive times and resolves the mailbox/role/email ids the suite asks for.
use super::corpus::{self, Seed};
use super::corpus_tail;
use crate::{Duration, Utc};
use mail_api::Store;
use mail_kernel::{Command, MailboxProperties};
use mail_sqlite_store::SqliteStore;
use std::collections::BTreeMap;

pub struct Seeded {
    pub mailboxes: BTreeMap<String, String>,
    pub roles: BTreeMap<String, String>,
    pub emails: BTreeMap<String, String>,
}

fn child(name: &str, parent: &str) -> Command {
    let mut properties = MailboxProperties::named(name.into());
    properties.parent_id = Some(parent.into());
    Command::SetMailbox {
        id: None,
        properties,
    }
}

pub fn seed(db: &SqliteStore, account: &str) -> Seeded {
    let now = Utc::now();
    let stamp = |h: i64| (now - Duration::hours(h)).to_rfc2822();
    let state = db.account(account).unwrap();
    let state = db
        .execute(
            account,
            state.revision,
            vec![
                Command::CreateMailbox {
                    name: "Test Folder A".into(),
                },
                Command::CreateMailbox {
                    name: "Test Folder B".into(),
                },
            ],
        )
        .unwrap();
    let id = |state: &mail_kernel::Account, name: &str| {
        state
            .mailboxes
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.id.clone())
            .expect("seeded mailbox")
    };
    let folder_a = id(&state, "Test Folder A");
    let state = db
        .execute(
            account,
            state.revision,
            vec![child("Child 1", &folder_a), child("Child 2", &folder_a)],
        )
        .unwrap();
    let mailboxes = BTreeMap::from([
        ("folderA".to_owned(), folder_a),
        ("folderB".to_owned(), id(&state, "Test Folder B")),
        ("child1".to_owned(), id(&state, "Child 1")),
        ("child2".to_owned(), id(&state, "Child 2")),
    ]);
    let roles = state
        .mailboxes
        .iter()
        .filter_map(|m| m.role.clone().map(|role| (role, m.id.clone())))
        .collect();
    let mut emails = BTreeMap::new();
    for seed in emails_for(&stamp) {
        let targets: Vec<String> = seed
            .mailbox
            .split('+')
            .map(|m| mailboxes.get(m).cloned().unwrap_or_else(|| m.to_owned()))
            .collect();
        let state = db.account(account).unwrap();
        let state = db
            .execute(
                account,
                state.revision,
                vec![Command::Append {
                    mailboxes: targets,
                    raw: seed.raw.into_bytes(),
                    keywords: seed.keywords,
                    received_at: (now - Duration::hours(seed.hours_ago)).timestamp(),
                }],
            )
            .unwrap();
        emails.insert(
            seed.key.to_owned(),
            state.messages.last().unwrap().id.clone(),
        );
    }
    Seeded {
        mailboxes,
        roles,
        emails,
    }
}

fn emails_for(stamp: &dyn Fn(i64) -> String) -> Vec<Seed> {
    let mut seeds = corpus::first(|d| stamp(d * 24));
    seeds.extend(corpus_tail::rest(|d| stamp(d * 24), stamp));
    seeds
}
