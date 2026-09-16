use mail_api::{DeliveryOutcome, MailTransport, SubmissionQueue};
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub(super) async fn run(
    queue: Arc<dyn SubmissionQueue>,
    transport: Arc<dyn MailTransport>,
    mut stop: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        if *stop.borrow() {
            return;
        }
        let claim = queue.clone();
        let result = tokio::task::spawn_blocking(move || {
            let lease = claim.claim_outbound(1)?.pop();
            lease
                .map(|lease| {
                    claim
                        .outbound_message(&lease)
                        .map(|message| (lease, message))
                })
                .transpose()
        })
        .await;
        let delay = match result {
            Ok(Ok(Some((lease, message)))) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|t| t.as_secs())
                    .unwrap_or(0);
                let outcome = if i64::try_from(now)
                    .unwrap_or(i64::MAX)
                    .saturating_sub(message.retry_at)
                    >= 432000
                {
                    DeliveryOutcome::Temporary(451)
                } else {
                    match send_owned(
                        queue.clone(),
                        transport.as_ref(),
                        &lease,
                        &message,
                        Duration::from_secs(30),
                    )
                    .await
                    {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            eprintln!("mail-app: outbound ownership lost: {error:?}");
                            continue;
                        }
                    }
                };
                let complete = queue.clone();
                if let Err(error) =
                    tokio::task::spawn_blocking(move || complete.finish_outbound(&lease, outcome))
                        .await
                        .unwrap_or(Err(mail_kernel::Error::Unavailable))
                {
                    eprintln!("mail-app: outbound completion failed: {error:?}");
                }
                continue;
            }
            Ok(Ok(None)) => Duration::from_millis(250),
            _ => {
                eprintln!("mail-app: outbound queue unavailable");
                Duration::from_secs(1)
            }
        };
        tokio::select! { result = stop.changed() => { if result.is_err() { return; } }, _ = tokio::time::sleep(delay) => {} }
    }
}

async fn send_owned(
    queue: Arc<dyn SubmissionQueue>,
    transport: &dyn MailTransport,
    lease: &mail_api::OutboundLease,
    message: &mail_api::QueuedMessage,
    period: Duration,
) -> Result<DeliveryOutcome, mail_kernel::Error> {
    renew_lease(queue.clone(), lease.clone()).await?;
    let send = transport.send(&lease.recipient, message);
    tokio::pin!(send);
    let mut renew = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
    renew.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            outcome = &mut send => return Ok(outcome),
            _ = renew.tick() => {
                renew_lease(queue.clone(), lease.clone()).await?;
            }
        }
    }
}

async fn renew_lease(
    queue: Arc<dyn SubmissionQueue>,
    lease: mail_api::OutboundLease,
) -> Result<(), mail_kernel::Error> {
    // A stuck database cannot leave SMTP running past lease expiry.
    tokio::time::timeout(
        Duration::from_secs(30),
        tokio::task::spawn_blocking(move || queue.renew_outbound(&lease)),
    )
    .await
    .map_err(|_| mail_kernel::Error::Unavailable)?
    .map_err(|_| mail_kernel::Error::Unavailable)?
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Immediate(std::sync::atomic::AtomicUsize);
    impl MailTransport for Immediate {
        fn send<'a>(
            &'a self,
            _: &'a str,
            _: &'a mail_api::QueuedMessage,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = DeliveryOutcome> + Send + 'a>>
        {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(std::future::ready(DeliveryOutcome::Delivered))
        }
    }
    struct Slow;
    impl MailTransport for Slow {
        fn send<'a>(
            &'a self,
            _: &'a str,
            _: &'a mail_api::QueuedMessage,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = DeliveryOutcome> + Send + 'a>>
        {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                DeliveryOutcome::Delivered
            })
        }
    }
    #[tokio::test]
    async fn smtp_worker_keeps_ownership_and_cancels_when_fencing_fails() {
        let db = Arc::new(mail_sqlite_store::SqliteStore::open(":memory:").unwrap());
        db.provision(
            mail_kernel::Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            &"a".repeat(32),
        )
        .unwrap();
        db.enqueue_submission(
            "a",
            "alice@example.org",
            &["one@remote.org".into()],
            b"From: alice@example.org\r\n\r\nbody\r\n",
        )
        .unwrap();
        let lease = db.claim_outbound(1).unwrap().pop().unwrap();
        let message = db.outbound_message(&lease).unwrap();
        assert_eq!(
            send_owned(
                db.clone(),
                &Slow,
                &lease,
                &message,
                Duration::from_millis(1)
            )
            .await,
            Ok(DeliveryOutcome::Delivered)
        );
        assert!(db.outbound_message(&lease).is_ok());
        let mut stale = lease.clone();
        stale.token = "superseded".into();
        let transport = Immediate(std::sync::atomic::AtomicUsize::new(0));
        let result = send_owned(
            db.clone(),
            &transport,
            &stale,
            &message,
            Duration::from_millis(1),
        )
        .await;
        assert_eq!(transport.0.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(result, Err(mail_kernel::Error::Conflict));
        assert_eq!(
            send_owned(
                db.clone(),
                &Slow,
                &stale,
                &message,
                Duration::from_millis(1)
            )
            .await,
            Err(mail_kernel::Error::Conflict)
        );
        assert!(db.outbound_message(&lease).is_ok());
    }
}
