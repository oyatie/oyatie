//! `Busy` handling for one JMAP method call. A method that meets `Busy`
//! before committing anything returns `BUSY`; its blocking worker is already
//! released when the request task sees it, sleeps in async context, checks
//! that the HTTP request is still wanted and its fixed budget remains, then
//! takes a worker again and re-runs the same call.
use super::Jmap;
use axum::http::StatusCode;
use mail_api::{Execution, Precondition};
use mail_kernel::{Account, Command, Error};
use mail_service::{Budget, MailService, backoff};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// In-process signal that a method must be re-run; never sent to a client.
pub(super) const BUSY: &str = "serverBusy";

/// Method error for one object of a batch: `Busy` before the first commit
/// re-runs the whole method, after it the object alone fails.
pub(super) fn object(kind: &'static str, committed: bool) -> Result<&'static str, &'static str> {
    match (kind == BUSY, committed) {
        (false, _) => Ok(kind),
        (true, true) => Ok("serverFail"),
        (true, false) => Err(BUSY),
    }
}

/// One mutation: `conditional` (the client's `ifInState`) is the only case
/// that may surface `Conflict`; otherwise the store re-applies the commands
/// on the state another writer produced meanwhile.
pub(super) fn commit(
    service: &MailService,
    token: &str,
    account: &Account,
    conditional: bool,
    commands: Vec<Command>,
    budget: &Budget,
) -> Result<(Execution, Account), Error> {
    let precondition = if conditional {
        Precondition::Require(account.revision)
    } else {
        Precondition::Observed(account.revision)
    };
    service.execute_read(token, &account.id, precondition, commands, budget)
}

/// Set when the HTTP request future is dropped, so a call that is waiting to
/// re-run stops instead of spending its budget for nobody.
pub(super) struct Guard(pub Arc<AtomicBool>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Run `step` on a blocking worker until it stops reporting `BUSY`. `None`
/// means the budget ran out (or the request was abandoned) while busy.
pub(super) async fn run<R: Send + 'static, T: Send + 'static>(
    state: &Jmap,
    cancelled: &AtomicBool,
    mut run: R,
    step: impl Fn(&Jmap, &mut R, &Budget) -> Result<T, &'static str> + Clone + Send + 'static,
) -> Result<(R, Option<Result<T, &'static str>>), StatusCode> {
    let budget = Budget::fixed();
    let mut attempt = 0;
    loop {
        let step = step.clone();
        let (next, result) = state
            .blocking(move |state| {
                let result = step(&state, &mut run, &budget);
                Ok((run, result))
            })
            .await?;
        run = next;
        if !matches!(result, Err(kind) if kind == BUSY) {
            return Ok((run, Some(result)));
        }
        tokio::time::sleep(backoff(attempt).min(budget.remaining())).await;
        attempt += 1;
        if budget.expired() || cancelled.load(Ordering::Relaxed) {
            return Ok((run, None));
        }
    }
}
