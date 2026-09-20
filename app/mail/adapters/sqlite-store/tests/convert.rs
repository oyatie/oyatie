//! Conversion of a `41ce37e61` database built at test time from the embedded
//! legacy DDL: every row round-trips, the audit row is written, and the
//! binary refuses what it must refuse.
use mail_api::{DeliveryQueue, Events, Identity, MetadataStore, SubmissionStore};
use mail_kernel::{Error, UndoStatus};
use mail_sqlite_store::contract::legacy::{LegacyAccountSpec, LegacyFixture, Shape, blob};
use mail_sqlite_store::{SqliteStore, contract::legacy::submission_id, convert::Converter};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[path = "convert/rate.rs"]
mod rate;
#[path = "convert/refusals.rs"]
mod refusals;
#[path = "convert/replay.rs"]
mod replay;

/// A unique scratch directory removed on drop.
pub struct Temp(PathBuf);
impl Temp {
    pub fn new(name: &str) -> Self {
        let now = std::time::SystemTime::now();
        let nanos = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("mail-convert-{name}-{pid}-{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    pub fn path(&self, file: &str) -> PathBuf {
        self.0.join(file)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Four accounts covering both `state` shapes, a trusted thread index, a
/// backfilled one, and an empty account.
pub fn specs() -> Vec<LegacyAccountSpec> {
    let mut beta = LegacyAccountSpec::generated("beta", Shape::Newer, 6);
    beta.thread_indexed = true;
    let mut gamma = LegacyAccountSpec::generated("gamma", Shape::Newer, 5);
    gamma.messages[2].links.push(("m1".into(), 1));
    gamma.mailboxes[1].uid_next = 2;
    vec![
        LegacyAccountSpec::generated("alpha", Shape::Oldest, 7),
        beta,
        gamma,
        LegacyAccountSpec::new("delta", Shape::Newer),
    ]
}

pub const OPERATOR: &str = "ops@example.org";

/// Build the legacy database, back it up under the lock and convert it.
pub fn convert_all(
    temp: &Temp,
    specs: &[LegacyAccountSpec],
) -> (
    LegacyFixture,
    mail_sqlite_store::convert::Conversion,
    PathBuf,
) {
    let database = temp.path("mail.sqlite");
    let backup = temp.path("backup.sqlite");
    let fixture = LegacyFixture::create(&database, specs).unwrap();
    let converter = Converter::open(&database).unwrap();
    converter.backup_into(&backup).unwrap();
    let conversion = converter.convert(&backup, OPERATOR).unwrap();
    (fixture, conversion, backup)
}

pub fn sha256_of(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

fn count(db: &rusqlite::Connection, sql: &str) -> u64 {
    db.query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn every_legacy_row_round_trips_and_the_audit_row_is_complete() {
    let temp = Temp::new("roundtrip");
    let specs = specs();
    let (fixture, conversion, backup) = convert_all(&temp, &specs);
    let database = temp.path("mail.sqlite");
    assert_eq!(conversion.accounts, 4);
    assert_eq!(fixture.messages, 18);
    assert!(
        !Path::new(&format!("{}-wal", database.display())).exists(),
        "no -wal beside the converted file"
    );
    let store = SqliteStore::open(&database).unwrap();
    for spec in &specs {
        let account = store.account(&spec.id).unwrap();
        let r = spec.revision + 1;
        assert_eq!(
            (account.revision, account.history_floor, account.mail_modseq),
            (r, r, r)
        );
        assert_eq!(
            (&account.tenant, &account.owner, &account.address),
            (&spec.tenant, &spec.owner, &spec.address)
        );
        assert_eq!(account.quota_bytes, spec.quota_bytes);
        assert_eq!(account.mailboxes.len(), spec.mailboxes.len());
        for (mailbox, expected) in account.mailboxes.iter().zip(&spec.mailboxes) {
            assert_eq!(
                (
                    &mailbox.id,
                    &mailbox.name,
                    &mailbox.role,
                    mailbox.uid_next,
                    mailbox.uid_validity
                ),
                (
                    &expected.id,
                    &expected.name,
                    &expected.role,
                    expected.uid_next,
                    expected.uid_validity
                )
            );
            let members: Vec<_> = spec
                .messages
                .iter()
                .filter(|m| m.links.iter().any(|(id, _)| *id == mailbox.id))
                .collect();
            assert_eq!(mailbox.total_emails, members.len(), "{}", mailbox.id);
            assert_eq!(
                mailbox.unread_emails,
                members
                    .iter()
                    .filter(|m| !m.keywords.iter().any(|k| k == "$seen"))
                    .count()
            );
            assert_eq!(
                mailbox.size_bytes,
                members.iter().map(|m| m.raw.len()).sum::<usize>()
            );
            let selection = store.mailbox_uids(&spec.id, &mailbox.id).unwrap();
            let mut expected_uids: Vec<(u32, String)> = members
                .iter()
                .map(|m| {
                    (
                        m.links.iter().find(|(id, _)| *id == mailbox.id).unwrap().1,
                        m.id.clone(),
                    )
                })
                .collect();
            expected_uids.sort();
            assert_eq!(selection.uids, expected_uids);
            assert_eq!(selection.highest_modseq, mailbox.highest_modseq);
        }
        assert_eq!(
            account.used_bytes,
            spec.messages.iter().map(|m| m.raw.len()).sum::<usize>()
        );
        assert_eq!(account.messages.len(), spec.messages.len());
        for expected in &spec.messages {
            let message = account
                .messages
                .iter()
                .find(|m| m.id == expected.id)
                .unwrap();
            for (mailbox, uid) in &expected.links {
                assert_eq!(
                    message.uid_in(mailbox),
                    Some(*uid),
                    "{}/{}",
                    spec.id,
                    expected.id
                );
            }
            assert_eq!(message.mailboxes.len(), expected.links.len());
            assert_eq!(message.keywords, expected.keywords);
            assert_eq!(message.size, expected.raw.len());
            assert_eq!(message.received_at, expected.received_at);
            // The oldest shape carried no MODSEQ; the newer one is clamped.
            let legacy_modseq = match spec.shape {
                Shape::Oldest => None,
                Shape::Newer => expected.modseq,
            };
            assert_eq!(
                message.modseq,
                legacy_modseq.unwrap_or(1).min(account.revision)
            );
            assert!(message.modseq <= account.mail_modseq);
            assert_eq!(store.blob(&spec.id, &expected.id).unwrap(), expected.raw);
            let n: usize = expected.id[1..].parse().unwrap();
            let group = (n - 1) / 5 * 5 + 1;
            assert_eq!(
                message.thread_id(),
                format!("e{group}"),
                "{}/{}",
                spec.id,
                expected.id
            );
        }
        assert_eq!(
            store
                .authenticate(spec.token.as_ref().unwrap())
                .unwrap()
                .account,
            spec.id
        );
        let (blob_id, content) = blob(&spec.id);
        assert_eq!(store.blob(&spec.id, &blob_id).unwrap(), content);
        assert_eq!(
            store.deliver_once(&spec.id, &format!("receipt-{}", spec.id), b"other bytes", 1),
            Err(Error::Conflict),
            "delivery receipt survived"
        );
        let submissions = store.submissions(&spec.id, None).unwrap();
        assert_eq!(submissions.revision, 1);
        assert_eq!(submissions.records.len(), 1);
        assert_eq!(submissions.records[0].id, submission_id(&spec.id));
        assert_eq!(submissions.records[0].undo_status, UndoStatus::Pending);
        assert_eq!(store.resolve(&spec.address).unwrap(), spec.id);
    }
    assert_eq!(store.pending("fresh", 100).unwrap().len(), specs.len());
    assert_eq!(
        store.pending("foundry", 100).unwrap().len(),
        specs.len() - 1
    );
    let lease = store.claim(10).unwrap();
    assert_eq!(lease.len(), 1);
    assert_eq!(lease[0].account, "alpha");
    assert_eq!(
        store.queued_message(&lease[0]).unwrap().raw,
        b"Subject: queued\r\n\r\nbody\r\n"
    );
    let failures = store.failed_deliveries("alpha", 10).unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].reason, "OverQuota");
    drop(store);

    let db = rusqlite::Connection::open(&database).unwrap();
    let (version, state, path, digest, at, operator): (u64, String, String, String, String, String) = db
        .query_row(
            "SELECT version,state,backup_path,backup_sha256,converted_at_utc,operator FROM schema_version",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .unwrap();
    assert_eq!((version, state.as_str()), (2, "complete"));
    assert_eq!(path, backup.to_string_lossy());
    assert_eq!(digest, sha256_of(&backup));
    assert_eq!(digest, conversion.backup_sha256);
    assert_eq!(at.len(), 20);
    assert!(at.ends_with('Z') && at.as_bytes()[10] == b'T', "{at}");
    assert_eq!(operator, OPERATOR);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM sqlite_master WHERE type='trigger'"
        ),
        0
    );
    for legacy in [
        "legacy_accounts",
        "legacy_thread_indexed",
        "thread_indexed",
        "account_access",
        "message_changes",
        "mailbox_commits",
        "mailbox_changes",
        "message_index_state",
        "message_metadata",
    ] {
        assert_eq!(
            count(
                &db,
                &format!("SELECT count(*) FROM sqlite_master WHERE name='{legacy}'")
            ),
            0,
            "{legacy} dropped"
        );
    }
    assert_eq!(count(&db, "SELECT count(*) FROM vacation_sent"), 4);
    assert_eq!(count(&db, "SELECT count(*) FROM history_commits"), 4);
    assert_eq!(count(&db, "SELECT count(*) FROM history"), 0);
    assert_eq!(count(&db, "SELECT count(*) FROM message_bodies"), 18);
    assert_eq!(count(&db, "SELECT count(*) FROM thread_members"), 18);
    assert_eq!(count(&db, "SELECT count(*) FROM outbound_jobs"), 4);
    // A resumed run has nothing to do; a converted file is not legacy. (The
    // inspecting connection above must close first: it holds a shared lock.)
    drop(db);
    assert!(matches!(
        Converter::open(&database)
            .unwrap()
            .convert(&backup, OPERATOR),
        Err(mail_sqlite_store::convert::ConvertError::NotLegacy(_))
    ));
}
