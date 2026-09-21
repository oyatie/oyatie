use super::MailService;
use mail_api::DeliveryTarget;
use mail_kernel::{Error, valid_address};
use std::collections::BTreeMap;

impl MailService {
    /// Bind a submission identity to its mailbox address and a distinct policy
    /// action. Read/write access alone does not permit sending as this account.
    pub fn submission_account(
        &self,
        token: &str,
        username: &str,
    ) -> Result<mail_api::AccountInfo, Error> {
        let principal = self.identity.authenticate(token)?;
        let account = self.store.account_info(&principal.account)?;
        if !account.address.eq_ignore_ascii_case(username) {
            return Err(Error::Forbidden);
        }
        self.policy
            .authorize(&principal, mail_api::Action::Submit, &account)?;
        Ok(account)
    }

    /// Revalidate credentials and sender at acceptance, including after DATA.
    pub fn submit(
        &self,
        token: &str,
        username: &str,
        sender: &str,
        recipients: &[String],
        raw: &[u8],
    ) -> Result<String, Error> {
        let account = self.submission_account(token, username)?;
        if !account.address.eq_ignore_ascii_case(sender) {
            return Err(Error::Forbidden);
        }
        let raw = super::submission::normalize(raw, &account.address)?;
        if let Some(queue) = &self.outbound {
            queue.enqueue_submission(&account.id, sender, recipients, &raw)
        } else {
            self.receive(sender, recipients, &raw)
        }
    }

    pub fn receive(
        &self,
        sender: &str,
        recipients: &[String],
        raw: &[u8],
    ) -> Result<String, Error> {
        if recipients.is_empty() || recipients.len() > 100 {
            return Err(Error::Invalid);
        }
        let mut targets = BTreeMap::new();
        for address in recipients {
            if !valid_address(address) {
                return Err(Error::Invalid);
            }
            let account = self.store.resolve(address)?;
            targets.entry(account).or_insert_with(|| address.clone());
        }
        let targets = targets
            .into_iter()
            .map(|(account, address)| DeliveryTarget { account, address })
            .collect::<Vec<_>>();
        self.queue.enqueue(sender, &targets, raw)
    }

    /// Each mailbox commit includes a receipt. A worker crash before finishing
    /// its queue lease can therefore be recovered on any other worker.
    pub fn deliver_pending(&self, limit: usize) -> Result<usize, Error> {
        let leases = self.queue.claim(limit)?;
        let mut delivered = 0;
        for lease in leases {
            let result = self.queue.queued_message(&lease).and_then(|message| {
                self.store.deliver_once(
                    &lease.account,
                    &lease.delivery_id(),
                    &message.raw,
                    message.received_at,
                )
            });
            let ok = result.is_ok();
            if ok {
                super::notify::signal(&lease.account);
            }
            match self.queue.finish(&lease, result) {
                Ok(()) if ok => delivered += 1,
                Ok(()) | Err(Error::Conflict) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(delivered)
    }
}
