use mail_service::MailService;
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(super) async fn run(service: Arc<MailService>, mut stop: oneshot::Receiver<()>) {
    loop {
        if !matches!(stop.try_recv(), Err(oneshot::error::TryRecvError::Empty)) {
            return;
        }
        let service = service.clone();
        let result = tokio::task::spawn_blocking(move || service.deliver_pending(1)).await;
        let delay = match result {
            Ok(Ok(n)) if n > 0 => continue,
            Ok(Ok(_)) => Duration::from_millis(250),
            error => {
                eprintln!("mail-app: delivery worker unavailable: {error:?}");
                Duration::from_secs(1)
            }
        };
        tokio::select! { _ = &mut stop => return, _ = tokio::time::sleep(delay) => {} }
    }
}
