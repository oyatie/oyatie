//! Builds a `41ce37e61` database at test time from the embedded DDL, with
//! accounts in both JSON `state` shapes that binary wrote, plus rows in every
//! table conversion must carry over. No database file is ever checked in.
use super::legacy_ddl::*;
use super::legacy_rows;
pub use super::legacy_rows::{blob, submission_id};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Which `state` encoding the account row uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Per-message `mailbox` + `uid` and the embedded `raw` byte array; the
    /// binary that wrote it had no thread index or `message_bodies` rows.
    Oldest,
    /// `mailboxes` map + `size`; bodies in `message_bodies`; `modseq` and
    /// `thread` per message; `mail_modseq` on the account.
    Newer,
}

#[derive(Clone, Debug)]
pub struct LegacyMessageSpec {
    pub id: String,
    pub links: Vec<(String, u32)>,
    pub raw: Vec<u8>,
    pub keywords: Vec<String>,
    pub received_at: i64,
    /// `Newer` only; `None` writes no `modseq` key.
    pub modseq: Option<u64>,
    /// `Newer` only: effective thread id when it differs from `id`.
    pub thread: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LegacyMailboxSpec {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub uid_next: u32,
    pub uid_validity: u32,
}

#[derive(Clone, Debug)]
pub struct LegacyAccountSpec {
    pub id: String,
    pub tenant: String,
    pub owner: String,
    pub address: String,
    /// Stored as `sha256(token)` like `provision` did.
    pub token: Option<String>,
    pub revision: u64,
    pub mail_modseq: Option<u64>,
    pub quota_bytes: usize,
    pub shape: Shape,
    /// Writes the `thread_indexed` marker plus `thread_members` /
    /// `thread_references` rows; conversion then trusts them instead of
    /// re-parsing bodies. Ignored for `Oldest`.
    pub thread_indexed: bool,
    pub mailboxes: Vec<LegacyMailboxSpec>,
    pub messages: Vec<LegacyMessageSpec>,
}

impl LegacyAccountSpec {
    /// Account `id` at `id@example.org` with INBOX and a Junk-role mailbox.
    pub fn new(id: &str, shape: Shape) -> Self {
        let mailbox = |mid: &str, name: &str, role: &str| LegacyMailboxSpec {
            id: mid.into(),
            name: name.into(),
            role: Some(role.into()),
            uid_next: 1,
            uid_validity: 7,
        };
        Self {
            id: id.into(),
            tenant: "t".into(),
            owner: format!("owner-{id}"),
            address: format!("{id}@example.org"),
            token: Some(format!("{id:0>32}")),
            revision: 1,
            mail_modseq: None,
            quota_bytes: 64 * 1024 * 1024,
            shape,
            thread_indexed: false,
            mailboxes: vec![
                mailbox("inbox", "INBOX", "inbox"),
                mailbox("m1", "Junk", "junk"),
            ],
            messages: vec![],
        }
    }

    /// `count` INBOX messages `e1..=e{count}` whose `Message-ID` /
    /// `References` headers thread them in groups of five; every other one
    /// is `$seen`. `revision` becomes `count + 1`.
    pub fn generated(id: &str, shape: Shape, count: usize) -> Self {
        let mut spec = Self::new(id, shape);
        for n in 1..=count {
            let group = (n - 1) / 5 * 5 + 1;
            let references = if n == group {
                String::new()
            } else {
                format!("References: <{group}@{id}.example>\r\n")
            };
            let raw = format!(
                "Message-ID: <{n}@{id}.example>\r\n{references}Subject: {}topic {group}\r\n\r\nbody {n}\r\n",
                if n == group { "" } else { "Re: " }
            );
            spec.messages.push(LegacyMessageSpec {
                id: format!("e{n}"),
                links: vec![("inbox".into(), n as u32)],
                raw: raw.into_bytes(),
                keywords: if n % 2 == 0 {
                    vec!["$seen".into()]
                } else {
                    vec![]
                },
                received_at: 1_700_000_000 + n as i64,
                modseq: Some(n as u64),
                thread: (n != group).then(|| format!("e{group}")),
            });
        }
        spec.mailboxes[0].uid_next = count as u32 + 1;
        spec.revision = count as u64 + 1;
        if shape == Shape::Newer {
            spec.mail_modseq = Some(count as u64);
        }
        spec
    }

    /// The JSON `state` column exactly as the shape's binary serialized it.
    pub fn state(&self) -> Value {
        let mailboxes: Vec<Value> = self
            .mailboxes
            .iter()
            .map(|m| {
                json!({"id":m.id,"name":m.name,"role":m.role,"parent_id":null,"sort_order":0,
                    "is_subscribed":true,"uid_next":m.uid_next,"uid_validity":m.uid_validity})
            })
            .collect();
        let messages: Vec<Value> = self.messages.iter().map(|m| self.message(m)).collect();
        let mut state = json!({"id":self.id,"tenant":self.tenant,"owner":self.owner,
            "address":self.address,"revision":self.revision,"quota_bytes":self.quota_bytes,
            "mailboxes":mailboxes,"messages":messages});
        if self.shape == Shape::Newer {
            state["mail_modseq"] = json!(self.mail_modseq.unwrap_or(0));
            state["identity"] = json!(mail_kernel::IdentitySettings::default());
            state["identity_revision"] = json!(0);
            state["vacation"] = json!(mail_kernel::VacationSettings::default());
            state["vacation_revision"] = json!(0);
        }
        state
    }

    fn message(&self, m: &LegacyMessageSpec) -> Value {
        match self.shape {
            Shape::Oldest => {
                assert_eq!(m.links.len(), 1, "the oldest shape holds one link");
                json!({"id":m.id,"mailbox":m.links[0].0,"uid":m.links[0].1,"raw":m.raw,
                    "keywords":m.keywords,"received_at":m.received_at})
            }
            Shape::Newer => {
                let links: serde_json::Map<String, Value> = m
                    .links
                    .iter()
                    .map(|(mailbox, uid)| (mailbox.clone(), json!(uid)))
                    .collect();
                let mut value = json!({"id":m.id,"mailboxes":links,"size":m.raw.len(),
                    "keywords":m.keywords,"received_at":m.received_at});
                if let Some(modseq) = m.modseq {
                    value["modseq"] = json!(modseq);
                }
                if let Some(thread) = &m.thread {
                    value["thread"] = json!(thread);
                }
                value
            }
        }
    }
}

/// Replay `41ce37e61`'s `initialize` sequence on any connection: first batch
/// (opens the transaction), `access()`, submission schema with its `floor`
/// check, the conditional `events.observed_at_ms` column, event cursors,
/// queue tables, and the last batch that commits.
pub fn legacy_initialize(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(LEGACY_DDL)?;
    db.execute_batch(LEGACY_ACCESS)?;
    db.execute_batch(LEGACY_SUBMISSION)?;
    if !has_column(db, "submission_heads", "floor")? {
        db.execute_batch(
            "ALTER TABLE submission_heads ADD COLUMN floor INTEGER NOT NULL DEFAULT 0",
        )?;
    }
    if !has_column(db, "events", "observed_at_ms")? {
        db.execute_batch(LEGACY_EVENTS_COLUMN)?;
    }
    db.execute_batch(LEGACY_EVENT_CURSORS)?;
    db.execute_batch(LEGACY_QUEUE)?;
    db.execute_batch(LEGACY_THREADS)
}

fn has_column(db: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut columns = db.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = columns
        .query_map([], |r| r.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(names.iter().any(|name| name == column))
}

/// A legacy database written to `path`, with the row counts the conversion
/// tests compare against.
#[derive(Clone, Debug)]
pub struct LegacyFixture {
    pub path: PathBuf,
    pub accounts: usize,
    pub messages: usize,
}

impl LegacyFixture {
    /// Write a WAL-mode legacy database holding `accounts`, each with its
    /// journal, event, blob, receipt, vacation, thread index and submission
    /// rows, plus one queued delivery, one failed delivery and one event
    /// cursor for the first account.
    pub fn create(path: &Path, accounts: &[LegacyAccountSpec]) -> rusqlite::Result<Self> {
        let db = Connection::open(path)?;
        legacy_initialize(&db)?;
        let tx = db.unchecked_transaction()?;
        for spec in accounts {
            let token = spec
                .token
                .as_ref()
                .map(|t| Sha256::digest(t.as_bytes()).to_vec());
            tx.execute(
                "INSERT INTO accounts(id,address,token,state) VALUES(?1,?2,?3,?4)",
                params![spec.id, spec.address, token, spec.state().to_string()],
            )?;
            if spec.shape == Shape::Newer {
                for m in &spec.messages {
                    tx.execute(
                        "INSERT INTO message_bodies(account,id,content) VALUES(?1,?2,?3)",
                        params![spec.id, m.id, m.raw],
                    )?;
                }
                if spec.thread_indexed {
                    legacy_rows::thread_index(&tx, spec)?;
                }
            }
            legacy_rows::journal(&tx, spec)?;
            legacy_rows::ancillary(&tx, spec)?;
        }
        if let Some(first) = accounts.first() {
            legacy_rows::queues(&tx, first)?;
        }
        tx.commit()?;
        Ok(Self {
            path: path.to_path_buf(),
            accounts: accounts.len(),
            messages: accounts.iter().map(|a| a.messages.len()).sum(),
        })
    }
}
