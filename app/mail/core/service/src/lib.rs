#![forbid(unsafe_code)]
use mail_api::{AccountInfo, Action, Identity, Policy, Principal, Store};
use mail_kernel::{Account, Command, Error};
use std::sync::Arc;
mod delivery;
mod email_submission;
mod submission;
mod submission_schedule;
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
    pub fn mailbox_changes(
        &self,
        token: &str,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<mail_api::MailboxChange>, Error> {
        self.authorize(token, account, Action::Read)?;
        self.store.mailbox_changes(account, since, until)
    }
    pub fn changes(
        &self,
        token: &str,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<mail_api::MessageChange>, Error> {
        self.authorize(token, account, Action::Read)?;
        self.store.message_changes(account, since, until)
    }
    pub fn changes_after(
        &self,
        token: &str,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<mail_api::MessageChange>, Error> {
        self.authorize(token, account, Action::Read)?;
        self.store.message_changes_after(account, since, until)
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

    pub fn execute(
        &self,
        token: &str,
        id: &str,
        revision: u64,
        commands: Vec<Command>,
    ) -> Result<Account, Error> {
        self.authorize(token, id, Action::Write)?;
        self.store.execute(id, revision, commands)
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
