use mail_kernel::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryTarget {
    pub account: String,
    pub address: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryLease {
    pub message: String,
    pub account: String,
    pub token: String,
    pub attempt: u32,
}
impl DeliveryLease {
    pub fn delivery_id(&self) -> String {
        format!("{}:{}", self.message, self.account)
    }
}

pub struct QueuedMessage {
    pub sender: String,
    pub raw: Vec<u8>,
    pub received_at: i64,
    /// Retry lifetime starts at scheduled release, independently of receipt identity.
    pub retry_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryFailure {
    pub message: String,
    pub account: String,
    pub failed_at: i64,
    pub reason: String,
}

/// Acceptance commits envelope, content and all recipient jobs before success.
/// Workers lease recipients independently. Mailbox commit and queue completion
/// may use different databases: `Store::deliver_once` makes replay safe.
pub trait DeliveryQueue: Send + Sync {
    fn enqueue(
        &self,
        sender: &str,
        recipients: &[DeliveryTarget],
        raw: &[u8],
    ) -> Result<String, Error>;
    fn claim(&self, limit: usize) -> Result<Vec<DeliveryLease>, Error>;
    fn queued_message(&self, lease: &DeliveryLease) -> Result<QueuedMessage, Error>;
    /// Local administration: retained terminal failures remain charged to quota.
    fn failed_deliveries(&self, account: &str, limit: usize)
    -> Result<Vec<DeliveryFailure>, Error>;
    /// Retry preserves the original delivery key, content and receipt semantics.
    fn retry_failed_delivery(&self, account: &str, message: &str) -> Result<(), Error>;
    /// Only the active lease can settle a job. Temporary failure retries for five
    /// days; permanent/expired failures retain their content for operator recovery.
    fn finish(&self, lease: &DeliveryLease, outcome: Result<(), Error>) -> Result<(), Error>;
}
