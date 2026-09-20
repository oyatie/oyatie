//! `Busy` handling for one IMAP command. The blocking worker never sleeps:
//! it reports `Busy` through the RFC 5530 `[INUSE]` status, hands the
//! session (and an unsent APPEND) back, and the socket task backs off and
//! re-dispatches the same command until its fixed budget runs out.
use super::{Session, append::Append, response::Output};
use crate::wire::write;
use mail_api::{Execution, Precondition};
use mail_kernel::{Account, Command, Error};
use mail_service::{Budget, MailService, backoff};
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

/// The tagged answer once the budget is exhausted, and the in-process signal
/// that a command must be re-dispatched while budget remains.
pub(super) const BUSY: &str = "NO [INUSE]";

/// Status for a failed mutation or history read.
pub(super) fn status(error: Error) -> &'static str {
    if error == Error::Busy { BUSY } else { "NO" }
}

/// Unconditional mutation observed at `account.revision`: the store
/// re-applies it on the current state when another writer got there first.
pub(super) fn commit(
    service: &MailService,
    token: &str,
    account: &Account,
    commands: Vec<Command>,
    budget: &Budget,
) -> Result<(Execution, Account), &'static str> {
    service
        .execute_read(
            token,
            &account.id,
            Precondition::Observed(account.revision),
            commands,
            budget,
        )
        .map_err(status)
}

/// Everything a selected-state command needs to reach the store.
pub(super) struct Call<'a> {
    pub service: &'a MailService,
    pub token: &'a str,
    pub account: &'a Account,
    pub budget: &'a Budget,
}

impl Call<'_> {
    pub fn commit(&self, commands: Vec<Command>) -> Result<(Execution, Account), &'static str> {
        commit(
            self.service,
            self.token,
            self.account,
            commands,
            self.budget,
        )
    }
}

pub(super) enum Outcome {
    Done {
        logout: bool,
    },
    /// Re-dispatch; an APPEND keeps its literals so nothing is re-read.
    Busy(Option<Append>),
}

/// Run one command to completion: spawn the blocking worker, stream its
/// output, and on `Busy` sleep in async context before spawning it again.
pub(super) async fn dispatch<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: &Arc<MailService>,
    mut session: Session,
    mut parts: Vec<String>,
    mut append: Option<Append>,
    transport: super::Transport,
) -> io::Result<(Session, bool)> {
    let budget = Budget::fixed();
    let mut attempt = 0;
    loop {
        let (send, mut receive) = tokio::sync::mpsc::channel(2);
        let service = service.clone();
        let worker = tokio::task::spawn_blocking(move || -> io::Result<_> {
            let mut output = Output::new(send);
            let outcome =
                session.respond(&service, &parts, transport, append, &budget, &mut output);
            output.finish()?;
            Ok((session, parts, outcome))
        });
        while let Some(bytes) = receive.recv().await {
            write(stream.get_mut(), &bytes).await?;
        }
        let (next, same, outcome) = worker
            .await
            .map_err(|_| io::Error::other("IMAP worker failed"))??;
        session = next;
        parts = same;
        match outcome {
            Outcome::Done { logout } => return Ok((session, logout)),
            Outcome::Busy(pending) => append = pending,
        }
        tokio::time::sleep(backoff(attempt).min(budget.remaining())).await;
        attempt += 1;
        if budget.expired() {
            let verb = parts[1].to_ascii_uppercase();
            write(
                stream.get_mut(),
                format!("{} {BUSY} {verb} completed\r\n", parts[0]).as_bytes(),
            )
            .await?;
            return Ok((session, false));
        }
    }
}
