use mail_service::MailService;
use std::{io, sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite};

/// The facade's upgrade must complete a server-side TLS handshake. Submission
/// requires encryption and authentication; inbound SMTP permits local delivery.
pub async fn smtp_starttls_session<S, T, F, U>(
    stream: S,
    service: Arc<MailService>,
    submission: bool,
    upgrade: F,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
    F: FnOnce(S) -> U,
    U: std::future::Future<Output = io::Result<T>>,
{
    if let Some(stream) = super::session(stream, service.clone(), submission, true, true).await? {
        let stream = tokio::time::timeout(Duration::from_secs(10), upgrade(stream))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "SMTP TLS handshake stalled"))??;
        // RFC 3207 discards EHLO, authentication and envelope state. TLS does
        // not introduce another SMTP greeting; the client must send EHLO again.
        super::session(stream, service, submission, false, false).await?;
    }
    Ok(())
}
