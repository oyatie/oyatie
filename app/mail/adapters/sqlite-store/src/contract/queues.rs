//! Forwarding of the two queue ports, so a wrapped store can stand in for
//! the real one wherever the suite's fixture is asked for a whole store.
macro_rules! delivery_queue {
    ($ty:ident) => {
        impl<T: mail_api::DeliveryQueue> mail_api::DeliveryQueue for $ty<T> {
            fn enqueue(
                &self,
                sender: &str,
                recipients: &[mail_api::DeliveryTarget],
                raw: &[u8],
            ) -> Result<String, Error> {
                self.inner.enqueue(sender, recipients, raw)
            }
            fn claim(&self, limit: usize) -> Result<Vec<mail_api::DeliveryLease>, Error> {
                self.inner.claim(limit)
            }
            fn queued_message(
                &self,
                lease: &mail_api::DeliveryLease,
            ) -> Result<mail_api::QueuedMessage, Error> {
                self.inner.queued_message(lease)
            }
            fn failed_deliveries(
                &self,
                account: &str,
                limit: usize,
            ) -> Result<Vec<mail_api::DeliveryFailure>, Error> {
                self.inner.failed_deliveries(account, limit)
            }
            fn retry_failed_delivery(&self, account: &str, message: &str) -> Result<(), Error> {
                self.inner.retry_failed_delivery(account, message)
            }
            fn finish(
                &self,
                lease: &mail_api::DeliveryLease,
                outcome: Result<(), Error>,
            ) -> Result<(), Error> {
                self.inner.finish(lease, outcome)
            }
        }
    };
}

macro_rules! submission_queue {
    ($ty:ident) => {
        impl<T: mail_api::SubmissionQueue> mail_api::SubmissionQueue for $ty<T> {
            fn enqueue_submission(
                &self,
                account: &str,
                sender: &str,
                recipients: &[String],
                raw: &[u8],
            ) -> Result<String, Error> {
                self.inner
                    .enqueue_submission(account, sender, recipients, raw)
            }
            fn claim_outbound(&self, limit: usize) -> Result<Vec<mail_api::OutboundLease>, Error> {
                self.inner.claim_outbound(limit)
            }
            fn outbound_message(
                &self,
                lease: &mail_api::OutboundLease,
            ) -> Result<mail_api::QueuedMessage, Error> {
                self.inner.outbound_message(lease)
            }
            fn renew_outbound(&self, lease: &mail_api::OutboundLease) -> Result<(), Error> {
                self.inner.renew_outbound(lease)
            }
            fn finish_outbound(
                &self,
                lease: &mail_api::OutboundLease,
                outcome: mail_api::DeliveryOutcome,
            ) -> Result<(), Error> {
                self.inner.finish_outbound(lease, outcome)
            }
        }
    };
}
