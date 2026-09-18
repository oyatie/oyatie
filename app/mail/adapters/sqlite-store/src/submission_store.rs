use super::{SqliteStore, storage, submission_history as history};
use mail_api::{
    SubmissionAcceptance, SubmissionChanges, SubmissionFailure, SubmissionPage,
    SubmissionSelection, SubmissionStore,
};
use mail_kernel::{
    Error, SubmissionDelivered, SubmissionDeliveryStatus, SubmissionQuery, UndoStatus,
};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use std::collections::BTreeSet;

impl SubmissionStore for SqliteStore {
    fn submissions(
        &self,
        account: &str,
        ids: Option<&[String]>,
    ) -> Result<SubmissionSelection, Error> {
        if ids.is_some_and(|ids| ids.len() > 256) {
            return Err(Error::OverQuota);
        }
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db.transaction().map_err(storage)?;
        let revision = history::revision(&tx, account)?;
        let records = if let Some(ids) = ids {
            let mut records = Vec::new();
            for id in ids.iter().collect::<BTreeSet<_>>() {
                match history::current(&tx, account, id) {
                    Ok(r) => records.push(r),
                    Err(Error::NotFound) => {}
                    Err(e) => return Err(e),
                }
            }
            records
        } else {
            let mut stmt = tx.prepare("SELECT state FROM submission_versions WHERE account=?1 AND until_revision IS NULL AND state IS NOT NULL ORDER BY id LIMIT 257").map_err(storage)?;
            let states = stmt
                .query_map([account], |r| r.get::<_, String>(0))
                .map_err(storage)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(storage)?;
            if states.len() > 256 {
                return Err(Error::OverQuota);
            }
            states
                .iter()
                .map(|s| history::decode(s))
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(SubmissionSelection { revision, records })
    }

    fn accept_submission(
        &self,
        account: &str,
        revision: u64,
        acceptance: SubmissionAcceptance,
    ) -> Result<SubmissionSelection, Error> {
        let mut record = acceptance.record;
        if record.identity_id != account
            || !record.id.is_empty()
            || record.send_at < 0
            || record.send_at > i64::MAX - 432000
            || record.undo_status != UndoStatus::Pending
        {
            return Err(Error::Invalid);
        }
        for recipient in &mut record.envelope.rcpt_to {
            if let Some((local, domain)) = recipient.email.rsplit_once('@') {
                recipient.email = format!("{local}@{}", domain.to_ascii_lowercase());
            }
        }
        let encoded = serde_json::to_vec(&record).map_err(|_| Error::Invalid)?;
        if encoded.len() > 65536 {
            return Err(Error::OverQuota);
        }
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        if history::revision(&tx, account)? != revision {
            return Err(Error::Conflict);
        }
        let email_revision: u64 = tx
            .query_row(
                "SELECT revision FROM accounts WHERE id=?1",
                [account],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if email_revision != acceptance.email_revision {
            return Err(Error::Conflict);
        }
        let thread: Option<String> = tx
            .query_row(
                "SELECT thread FROM messages WHERE account=?1 AND id=?2",
                params![account, record.email_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        record.thread_id = thread.ok_or(Error::NotFound)?;
        let current: usize = tx.query_row("SELECT count(*) FROM submission_versions WHERE account=?1 AND until_revision IS NULL AND state IS NOT NULL", [account], |r| r.get(0)).map_err(storage)?;
        if current >= 10000 {
            return Err(Error::OverQuota);
        }
        let recipients: Vec<_> = record
            .envelope
            .rcpt_to
            .iter()
            .map(|r| r.email.clone())
            .collect();
        record.id = super::submission::enqueue_tx(
            &tx,
            account,
            &record.envelope.mail_from.email,
            &recipients,
            &acceptance.raw,
            acceptance.allow_remote,
        )?;
        record.delivery_status = recipients
            .into_iter()
            .map(|r| {
                (
                    r,
                    SubmissionDeliveryStatus {
                        smtp_reply: "250 2.1.5 Queued".into(),
                        delivered: SubmissionDelivered::Queued,
                    },
                )
            })
            .collect();
        tx.execute("INSERT INTO submission_schedule(message,account,send_at) VALUES(?1,?2,max(?3,unixepoch()))", params![record.id,account,record.send_at]).map_err(storage)?;
        for table in ["delivery_jobs", "outbound_jobs"] {
            tx.execute(
                &format!("UPDATE {table} SET next_attempt=max(?2,unixepoch()) WHERE message=?1"),
                params![record.id, record.send_at],
            )
            .map_err(storage)?;
        }
        let revision = history::write(&tx, account, &record, false)?;
        tx.commit().map_err(storage)?;
        Ok(SubmissionSelection {
            revision,
            records: vec![record],
        })
    }

    fn cancel_submission(
        &self,
        account: &str,
        revision: u64,
        id: &str,
    ) -> Result<SubmissionSelection, SubmissionFailure> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        if history::revision(&tx, account)? != revision {
            return Err(Error::Conflict.into());
        }
        let mut record = history::current(&tx, account, id)?;
        let claimed: Option<bool> = tx
            .query_row(
                "SELECT claimed FROM submission_schedule WHERE message=?1 AND account=?2",
                params![id, account],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        if record.undo_status != UndoStatus::Pending || claimed != Some(false) {
            return Err(SubmissionFailure::CannotUnsend);
        }
        for table in ["delivery_jobs", "outbound_jobs"] {
            let attempted: bool = tx
                .query_row(
                    &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE message=?1 AND attempt>0)"),
                    [id],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            if attempted {
                return Err(SubmissionFailure::CannotUnsend);
            }
        }
        for (table, column) in [
            ("delivery_jobs", "message"),
            ("outbound_jobs", "message"),
            ("queued_messages", "id"),
            ("submitted_messages", "id"),
            ("submission_schedule", "message"),
        ] {
            tx.execute(&format!("DELETE FROM {table} WHERE {column}=?1"), [id])
                .map_err(storage)?;
        }
        record.undo_status = UndoStatus::Canceled;
        history::reset(&mut record);
        let revision = history::write(&tx, account, &record, false)?;
        tx.commit().map_err(storage)?;
        Ok(SubmissionSelection {
            revision,
            records: vec![record],
        })
    }

    fn destroy_submission(&self, account: &str, revision: u64, id: &str) -> Result<u64, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        if history::revision(&tx, account)? != revision {
            return Err(Error::Conflict);
        }
        let record = history::current(&tx, account, id)?;
        let revision = history::write(&tx, account, &record, true)?;
        tx.commit().map_err(storage)?;
        Ok(revision)
    }

    fn query_submissions(
        &self,
        account: &str,
        revision: Option<u64>,
        query: &SubmissionQuery,
    ) -> Result<SubmissionPage, SubmissionFailure> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db.transaction().map_err(storage)?;
        super::submission_query::query(&tx, account, revision, query)
    }

    fn submission_changes(
        &self,
        account: &str,
        since: u64,
        limit: usize,
    ) -> Result<SubmissionChanges, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db.transaction().map_err(storage)?;
        history::changes(&tx, account, since, limit)
    }
}
