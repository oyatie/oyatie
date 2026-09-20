use super::{Mode, SmtpParams, limits::Meter};
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
    smtp_starttls_session_with(stream, service, submission, upgrade, &SmtpParams::default()).await
}

pub async fn smtp_starttls_session_with<S, T, F, U>(
    stream: S,
    service: Arc<MailService>,
    submission: bool,
    upgrade: F,
    params: &SmtpParams,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
    F: FnOnce(S) -> U,
    U: std::future::Future<Output = io::Result<T>>,
{
    let plain = Mode {
        submission,
        starttls: true,
        tls: false,
        greeting: true,
    };
    // One meter for the connection: RFC 3207 discards session state, not
    // the bytes the peer has already sent or the time it has been open.
    let mut meter = Meter::new(params);
    if let Some(stream) = super::session(stream, service.clone(), plain, params, &mut meter).await?
    {
        let stream = tokio::time::timeout(Duration::from_secs(10), upgrade(stream))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "SMTP TLS handshake stalled"))??;
        // RFC 3207 discards EHLO, authentication and envelope state. TLS does
        // not introduce another SMTP greeting; the client must send EHLO again.
        let secured = Mode {
            submission,
            starttls: false,
            tls: true,
            greeting: false,
        };
        super::session(stream, service, secured, params, &mut meter).await?;
    }
    Ok(())
}
