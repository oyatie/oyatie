#![forbid(unsafe_code)]
use mail_api::{
    AccountInfo, Action, Execution, HistoryPage, Identity, MailboxSelection, Policy, Precondition,
    Principal, Store,
};
use mail_kernel::{Account, Command, Error};
use std::sync::Arc;
mod admission;
mod delivery;
mod email_submission;
mod submission;
mod submission_schedule;
pub use admission::{Budget, MUTATION_DEADLINE, PER_ACCOUNT_MUTATIONS, backoff};
pub use email_submission::{SubmitEmail, SubmitEmailError};
pub use submission_schedule::MAX_DELAYED_SEND;

pub struct MailService {
    pub outbound: Option<Arc<dyn mail_api::SubmissionQueue>>,
    pub queue: Arc<dyn mail_api::DeliveryQueue>,
    pub store: Arc<dyn Store>,
    pub identity: Arc<dyn Identity>,
    pub policy: Arc<dyn Policy>,
}

impl MailService {
    pub fn messages(
        &self,
        token: &str,
        account: &str,
        ids: &[String],
    ) -> Result<mail_api::MessageSelection, Error> {
        self.authorize(token, account, Action::Read)?;
        self.store.messages(account, ids)
    }
    pub fn mailbox_uids(
        &self,
        token: &str,
        account: &str,
        mailbox: &str,
    ) -> Result<MailboxSelection, Error> {
        self.authorize(token, account, Action::Read)?;
        self.store.mailbox_uids(account, mailbox)
    }
    /// History rows after `since`; `page.below_floor()` tells the caller to
    /// fall back per protocol.
    pub fn history(
        &self,
        token: &str,
        account: &str,
        since: u64,
        limit: usize,
    ) -> Result<HistoryPage, Error> {
        self.authorize(token, account, Action::Read)?;
        self.store.history(account, since, limit)
    }
    pub fn upload(&self, token: &str, id: &str, raw: &[u8]) -> Result<String, Error> {
        self.authorize(token, id, Action::Write)?;
        self.store.put_blob(id, raw)
    }

    pub fn download(&self, token: &str, account: &str, blob: &str) -> Result<Vec<u8>, Error> {
        self.authorize(token, account, Action::Read)?;
        self.store.blob(account, blob)
    }

    pub fn principal(&self, token: &str) -> Result<Principal, Error> {
        self.identity.authenticate(token)
    }

    pub fn read(&self, token: &str, id: &str) -> Result<Account, Error> {
        self.authorize(token, id, Action::Read)?;
        self.store.account(id)
    }

    pub fn authorize(&self, token: &str, id: &str, action: Action) -> Result<AccountInfo, Error> {
        let principal = self.identity.authenticate(token)?;
        let account = self.store.account_info(id)?;
        self.policy.authorize(&principal, action, &account)?;
        Ok(account)
    }

    /// One mutation batch under the caller's budget: waits for a per-account
    /// slot (the wait counts toward the deadline), then commits once. `Busy`
    /// tells the caller to back off and re-attempt without holding a worker.
    pub fn execute(
        &self,
        token: &str,
        id: &str,
        precondition: Precondition,
        commands: Vec<Command>,
        budget: &Budget,
    ) -> Result<Execution, Error> {
        self.authorize(token, id, Action::Write)?;
        let _slot = admission::Admission::node().acquire(id, budget)?;
        if budget.expired() {
            return Err(Error::Busy);
        }
        self.store.execute(id, precondition, commands)
    }

    /// Execute, then read the projection the protocol layer synchronizes from.
    pub fn execute_read(
        &self,
        token: &str,
        id: &str,
        precondition: Precondition,
        commands: Vec<Command>,
        budget: &Budget,
    ) -> Result<(Execution, Account), Error> {
        let execution = self.execute(token, id, precondition, commands, budget)?;
        Ok((execution, self.store.account(id)?))
    }
}

/// Standalone policy grants only the authenticated account's owner. Hosted
/// composition replaces this adapter with its PDP-backed Policy implementation.
pub struct OwnerPolicy;
impl Policy for OwnerPolicy {
    fn authorize(
        &self,
        principal: &Principal,
        _: Action,
        account: &AccountInfo,
    ) -> Result<(), Error> {
        if principal.tenant == account.tenant
            && principal.subject == account.owner
            && principal.account == account.id
        {
            Ok(())
        } else {
            Err(Error::Forbidden)
        }
    }
}
