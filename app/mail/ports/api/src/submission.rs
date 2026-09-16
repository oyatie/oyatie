use crate::QueuedMessage;
use mail_kernel::Error;
use std::{future::Future, pin::Pin};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboundLease {
    pub message: String,
    pub account: String,
    pub recipient: String,
    pub token: String,
    pub attempt: u32,
}

/// SMTP reply codes only, never untrusted remote diagnostics or credentials.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Delivered,
    Temporary(u16),
    Permanent(u16),
}

/// Submission atomically accepts all local and remote recipients. Implementations
/// bind the sender to the submitting account and charge its outbound capacity.
pub trait SubmissionQueue: Send + Sync {
    fn enqueue_submission(
        &self,
        account: &str,
        sender: &str,
        recipients: &[String],
        raw: &[u8],
    ) -> Result<String, Error>;
    fn claim_outbound(&self, limit: usize) -> Result<Vec<OutboundLease>, Error>;
    fn outbound_message(&self, lease: &OutboundLease) -> Result<QueuedMessage, Error>;
    /// Extend a still-owned lease. Expired or superseded tokens cannot renew.
    fn renew_outbound(&self, lease: &OutboundLease) -> Result<(), Error>;
    /// Fenced completion: retry temporary failures; atomically enqueue a local
    /// delivery-status notice for permanent failures or five-day queue expiry.
    fn finish_outbound(&self, lease: &OutboundLease, outcome: DeliveryOutcome)
    -> Result<(), Error>;
}

/// External SMTP delivery can be duplicated after ambiguous acknowledgements.
/// Lease fencing prevents stale workers from settling another worker's job; it
/// cannot make a remote SMTP server participate in the queue transaction.
pub trait MailTransport: Send + Sync {
    fn send<'a>(
        &'a self,
        recipient: &'a str,
        message: &'a QueuedMessage,
    ) -> Pin<Box<dyn Future<Output = DeliveryOutcome> + Send + 'a>>;
}
