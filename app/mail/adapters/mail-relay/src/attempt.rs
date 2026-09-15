use mail_api::DeliveryOutcome;

/// The peer may have committed DATA even when its final reply was lost. Keep
/// that uncertainty until route selection ends; another MX in this attempt
/// would cause an avoidable immediate duplicate. The durable queue still owns
/// SMTP's retry policy when no successful acknowledgment reached us.
pub(super) struct Failure {
    pub outcome: DeliveryOutcome,
    pub retry_route: bool,
}

impl Failure {
    pub fn ambiguous(outcome: DeliveryOutcome) -> Self {
        Self {
            outcome,
            retry_route: false,
        }
    }
}

impl From<DeliveryOutcome> for Failure {
    fn from(outcome: DeliveryOutcome) -> Self {
        Self {
            outcome,
            retry_route: true,
        }
    }
}
