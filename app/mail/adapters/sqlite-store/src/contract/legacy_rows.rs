//! Rows beside the account `state`: the legacy journal (dropped by
//! conversion), the thread and message indexes, and every table the
//! relational schema keeps unchanged and must therefore carry over intact.
use super::legacy::LegacyAccountSpec;
use mail_kernel::{EnvelopeAddress, SubmissionEnvelope, SubmissionRecord, UndoStatus};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Submission and queue message id of the account's one pending submission.
pub fn submission_id(account: &str) -> String {
    format!("s-{account}")
}

/// Content of the account's one temporary blob and its id.
pub fn blob(account: &str) -> (String, Vec<u8>) {
    let content = format!("blob for {account}").into_bytes();
    (format!("b{:x}", Sha256::digest(&content)), content)
}

/// The legacy per-revision journal: `history_commits(previous, revision)`
/// chains, one `message_changes` row per message at the head revision, and
/// one mailbox commit. Conversion drops all four tables.
pub(super) fn journal(tx: &Connection, spec: &LegacyAccountSpec) -> rusqlite::Result<()> {
    for revision in 1..=spec.revision {
        tx.execute(
            "INSERT INTO history_commits(account,previous,revision) VALUES(?1,?2,?3)",
            params![spec.id, revision - 1, revision],
        )?;
    }
    for message in &spec.messages {
        tx.execute(
            "INSERT INTO message_changes(account,revision,id,before_state,after_state) VALUES(?1,?2,?3,NULL,?4)",
            params![
                spec.id,
                spec.revision,
                message.id,
                serde_json::json!({"id":message.id,"keywords":message.keywords}).to_string()
            ],
        )?;
    }
    tx.execute(
        "INSERT INTO mailbox_commits(account,previous,revision) VALUES(?1,0,1)",
        [&spec.id],
    )?;
    tx.execute(
        "INSERT INTO mailbox_changes(account,revision,id,before_state,after_state) VALUES(?1,1,'inbox',NULL,'{}')",
        [&spec.id],
    )?;
    Ok(())
}

/// `thread_indexed` marker, `thread_members` / `thread_references` computed
/// the way the legacy indexer did, and the lazy message index.
pub(super) fn thread_index(tx: &Connection, spec: &LegacyAccountSpec) -> rusqlite::Result<()> {
    tx.execute("INSERT INTO thread_indexed(account) VALUES(?1)", [&spec.id])?;
    tx.execute(
        "INSERT INTO message_index_state(account,revision) VALUES(?1,?2)",
        params![spec.id, spec.revision],
    )?;
    for message in &spec.messages {
        let refs =
            crate::threads::references(&message.raw).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let thread = message.thread.as_deref().unwrap_or(&message.id);
        tx.execute(
            "INSERT INTO thread_members(account,message,subject,thread) VALUES(?1,?2,?3,?4)",
            params![spec.id, message.id, refs.subject, thread],
        )?;
        for key in &refs.keys {
            tx.execute(
                "INSERT INTO thread_references(account,message,reference) VALUES(?1,?2,?3)",
                params![spec.id, message.id, key],
            )?;
        }
        tx.execute(
            "INSERT INTO message_metadata(account,id,state) VALUES(?1,?2,?3)",
            params![
                spec.id,
                message.id,
                serde_json::json!({"id":message.id,"thread":thread}).to_string()
            ],
        )?;
    }
    Ok(())
}

/// Tables the relational schema keeps: one event, one temporary blob, one
/// delivery receipt, one vacation reply marker and one pending submission
/// (head, version, schedule, submitted body, outbound job) per account.
pub(super) fn ancillary(tx: &Connection, spec: &LegacyAccountSpec) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO events(tenant,account,revision,delivered,observed_at_ms) VALUES(?1,?2,?3,0,1000)",
        params![spec.tenant, spec.id, spec.revision],
    )?;
    let (blob_id, content) = blob(&spec.id);
    tx.execute(
        "INSERT INTO blobs(account,id,content,expires_at) VALUES(?1,?2,?3,unixepoch()+86400)",
        params![spec.id, blob_id, content],
    )?;
    tx.execute(
        "INSERT INTO delivery_receipts(account,id,digest) VALUES(?1,?2,?3)",
        params![
            spec.id,
            format!("receipt-{}", spec.id),
            Sha256::digest(b"receipt").as_slice()
        ],
    )?;
    tx.execute(
        "INSERT INTO vacation_sent(account,sender) VALUES(?1,'friend@remote.org')",
        [&spec.id],
    )?;
    let id = submission_id(&spec.id);
    let email = spec
        .messages
        .first()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "e1".into());
    let address = |email: &str| EnvelopeAddress {
        email: email.into(),
        parameters: BTreeMap::new(),
    };
    let record = SubmissionRecord {
        id: id.clone(),
        identity_id: spec.id.clone(),
        email_id: email.clone(),
        thread_id: email.clone(),
        envelope: SubmissionEnvelope {
            mail_from: address(&spec.address),
            rcpt_to: vec![address("remote@example.net")],
        },
        send_at: 2_000_000_000,
        undo_status: UndoStatus::Pending,
        delivery_status: BTreeMap::new(),
    };
    let state = serde_json::to_string(&record).map_err(|_| rusqlite::Error::InvalidQuery)?;
    tx.execute(
        "INSERT INTO submission_heads(account,revision,floor) VALUES(?1,1,0)",
        [&spec.id],
    )?;
    tx.execute(
        "INSERT INTO submission_versions(account,id,revision,until_revision,state,identity_id,email_id,thread_id,send_at,undo)
         VALUES(?1,?2,1,NULL,?3,?1,?4,?4,2000000000,'pending')",
        params![spec.id, id, state, email],
    )?;
    tx.execute(
        "INSERT INTO submission_schedule(message,account,send_at,claimed) VALUES(?1,?2,2000000000,0)",
        params![id, spec.id],
    )?;
    let raw = format!("From: {}\r\nSubject: queued\r\n\r\nbody\r\n", spec.address);
    tx.execute(
        "INSERT INTO submitted_messages(id,account,sender,content,size,received_at) VALUES(?1,?2,?3,?4,?5,1700000000)",
        params![id, spec.id, spec.address, raw.as_bytes(), raw.len()],
    )?;
    tx.execute(
        "INSERT INTO outbound_jobs(message,account,recipient,next_attempt) VALUES(?1,?2,'remote@example.net',2000000000)",
        params![id, spec.id],
    )?;
    Ok(())
}

/// One queued inbound delivery, one retained failure and one event cursor,
/// all bound to `first`.
pub(super) fn queues(tx: &Connection, first: &LegacyAccountSpec) -> rusqlite::Result<()> {
    let raw: &[u8] = b"Subject: queued\r\n\r\nbody\r\n";
    tx.execute(
        "INSERT INTO queued_messages(id,sender,content,size,received_at) VALUES('q1','sender@example.net',?1,?2,unixepoch())",
        params![raw, raw.len()],
    )?;
    tx.execute(
        "INSERT INTO delivery_jobs(message,account,address,next_attempt) VALUES('q1',?1,?2,0)",
        params![first.id, first.address],
    )?;
    tx.execute(
        "INSERT INTO failed_delivery_messages(id,sender,content,size,received_at) VALUES('f1','',?1,?2,1700000000)",
        params![raw, raw.len()],
    )?;
    tx.execute(
        "INSERT INTO failed_delivery_jobs(message,account,address,failed_at,reason) VALUES('f1',?1,?2,1700000001,'OverQuota')",
        params![first.id, first.address],
    )?;
    tx.execute(
        "INSERT INTO event_cursors(consumer,sequence) VALUES('foundry',1)",
        [],
    )?;
    Ok(())
}
