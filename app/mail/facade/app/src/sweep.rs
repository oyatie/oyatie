use mail_api::BlobStore;
use mail_service::MailService;
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

/// The sole expiry decider for bodies: every minute, delete unreferenced
/// bodies in bounded batches until a batch comes back empty.
pub(super) async fn run(service: Arc<MailService>, mut stop: oneshot::Receiver<()>) {
    loop {
        let service = service.clone();
        let result = tokio::task::spawn_blocking(move || {
            service.store.orphan_sweep(mail_api::Clock.now_secs(), 1000)
        })
        .await;
        let delay = match result {
            Ok(Ok(n)) if n > 0 => {
                if !matches!(stop.try_recv(), Err(oneshot::error::TryRecvError::Empty)) {
                    return;
                }
                continue;
            }
            Ok(Ok(_)) => Duration::from_secs(60),
            error => {
                eprintln!("mail-app: blob sweep unavailable: {error:?}");
                Duration::from_secs(60)
            }
        };
        tokio::select! { _ = &mut stop => return, _ = tokio::time::sleep(delay) => {} }
    }
}
