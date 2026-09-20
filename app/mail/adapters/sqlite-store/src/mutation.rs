//! One atomic batch: load the bounded working set, let the kernel compute the
//! effects, persist exactly those rows. Conflict re-apply lives here; only a
//! `Require` precondition surfaces `Conflict`.
use super::{records, storage, threads};
use mail_api::{Consumer, Execution, Precondition};
use mail_kernel::{Account, Command, Effects, Error, Scope};
use rusqlite::{Connection, params};

pub(super) struct Batch {
    pub(super) account: Account,
    before: Account,
    outside: Vec<String>,
    /// Members this commit may still re-thread by merging (`MERGE_LIMIT`).
    merge_budget: usize,
    /// Consumers whose dirty key this commit writes.
    enabled: Vec<Consumer>,
}

impl Batch {
    /// Load the account record, its mailboxes and the records the commands
    /// need. Under `Observed`, a stale caller's commands are re-applied to
    /// the current state.
    pub(super) fn open(
        db: &Connection,
        id: &str,
        precondition: Precondition,
        commands: &[Command],
        enabled: &[Consumer],
    ) -> Result<(Self, bool), Error> {
        let mut account = records::header(db, id)?;
        let reapply = match precondition {
            Precondition::Require(revision) if revision != account.revision => {
                return Err(Error::Conflict);
            }
            Precondition::Require(_) => false,
            Precondition::Observed(revision) => revision != account.revision,
        };
        let scopes: Vec<Scope<'_>> = commands.iter().map(Command::scope).collect();
        records::working_set(db, &mut account, &scopes)?;
        Ok((
            Self {
                before: account.clone(),
                account,
                outside: vec![],
                merge_budget: threads::MERGE_LIMIT,
                enabled: enabled.to_vec(),
            },
            reapply,
        ))
    }

    /// Apply one command, persisting its body and thread references as the
    /// kernel mutates the working set.
    pub(super) fn apply(&mut self, db: &Connection, command: Command) -> Result<(), Error> {
        let (body, next) = match &command {
            Command::Append { blob, .. } => {
                let next = self
                    .account
                    .revision
                    .checked_add(1)
                    .ok_or(Error::OverQuota)?;
                (Some(blob.clone()), next)
            }
            _ => (None, 0),
        };
        let transfer = match &command {
            Command::Transfer { id, .. } => Some(id.clone()),
            _ => None,
        };
        let held: Vec<String> = self.account.messages.iter().map(|m| m.id.clone()).collect();
        self.account.apply(command)?;
        if let Some(blob) = body {
            let id = format!("e{next}");
            let raw =
                super::blob::reference(db, &self.account.id, &blob, &format!("message:{id}"))?;
            let refs = threads::references(&raw)?;
            self.outside.extend(threads::link(
                db,
                &mut self.account,
                &id,
                refs,
                &mut self.merge_budget,
            )?);
        }
        if let Some(source) = transfer {
            super::transfer::copy(
                db,
                &self.account.id,
                &source,
                &format!("e{}", self.account.revision),
            )?;
        }
        // A record whose last link went away in this command must not thread
        // a later Append of the same batch: forget its keys now, not at commit.
        for id in held
            .iter()
            .filter(|id| !self.account.messages.iter().any(|m| m.id == **id))
        {
            threads::forget(db, &self.account.id, id)?;
        }
        Ok(())
    }

    /// A re-applied `Destroy` whose record vanished is a no-op (POP re-apply);
    /// every other command on a vanished record surfaces `NotFound`.
    pub(super) fn skips(&self, command: &Command) -> bool {
        match (command, command.scope()) {
            (Command::Destroy { .. }, Scope::Message(id)) => {
                !self.account.messages.iter().any(|m| m.id == id)
            }
            _ => false,
        }
    }

    /// Stamp, persist the effects and the event row; returns the execution
    /// summary. Nothing is written when no command ran.
    pub(super) fn commit(mut self, db: &Connection) -> Result<(Execution, Account), Error> {
        if self.account.revision == self.before.revision {
            return Ok((
                Execution {
                    revision: self.account.revision,
                    ..Execution::default()
                },
                self.account,
            ));
        }
        let mut effects = self.account.commit(&self.before)?;
        threads::stamp(
            db,
            &mut self.account,
            &self.outside,
            effects.revision,
            &mut effects.history,
        )?;
        persist(db, &self.before, &self.account, &effects, &self.enabled)?;
        Ok((
            Execution {
                revision: effects.revision,
                ids: effects.ids,
                allocations: effects.allocations,
            },
            self.account,
        ))
    }
}

fn persist(
    db: &Connection,
    before: &Account,
    after: &Account,
    effects: &Effects,
    enabled: &[Consumer],
) -> Result<(), Error> {
    records::upsert_header(db, after)?;
    for mailbox in &after.mailboxes {
        if before.mailboxes.iter().any(|m| m == mailbox) {
            continue;
        }
        records::upsert_mailbox(db, &after.id, mailbox)?;
    }
    for old in &before.mailboxes {
        if !after.mailboxes.iter().any(|m| m.id == old.id) {
            db.execute(
                "DELETE FROM mailboxes WHERE account=?1 AND id=?2",
                params![after.id, old.id],
            )
            .map_err(storage)?;
        }
    }
    for message in &after.messages {
        if before.messages.iter().any(|m| m == message) {
            continue;
        }
        records::upsert_message(db, &after.id, message)?;
    }
    for id in &effects.deleted {
        delete_message(db, &after.id, id)?;
    }
    super::history::record(db, &after.id, effects.revision, &effects.history)?;
    super::feed::mark(db, enabled, &after.tenant, &after.id)?;
    Ok(())
}

/// Execute a batch on an open transaction: shared by `execute` and delivery.
pub(super) fn run(
    db: &Connection,
    id: &str,
    precondition: Precondition,
    commands: Vec<Command>,
    enabled: &[Consumer],
) -> Result<(Execution, Account), Error> {
    let (mut batch, reapply) = Batch::open(db, id, precondition, &commands, enabled)?;
    for command in commands {
        if reapply && batch.skips(&command) {
            continue;
        }
        batch.apply(db, command)?;
    }
    batch.commit(db)
}

pub(super) fn delete_message(db: &Connection, account: &str, id: &str) -> Result<(), Error> {
    for table in ["message_mailboxes", "thread_members", "thread_references"] {
        db.execute(
            &format!("DELETE FROM {table} WHERE account=?1 AND message=?2"),
            params![account, id],
        )
        .map_err(storage)?;
    }
    db.execute(
        "DELETE FROM messages WHERE account=?1 AND id=?2",
        params![account, id],
    )
    .map_err(storage)?;
    db.execute(
        "DELETE FROM blob_links WHERE account=?1 AND owner=?2",
        params![account, format!("message:{id}")],
    )
    .map_err(storage)?;
    Ok(())
}
