//! The session entry points: one per listener shape. Each owns the
//! connection's meter, except the STARTTLS pair in `tls.rs`, which spans
//! both halves with one.
use super::{Mode, SmtpParams, limits::Meter, session};
use crate::wire::write;
use mail_service::MailService;
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};

pub async fn smtp_session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
) -> io::Result<()> {
    smtp_session_with(stream, service, &SmtpParams::default()).await
}

pub async fn smtp_session_with<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    params: &SmtpParams,
) -> io::Result<()> {
    let mode = Mode {
        submission: false,
        starttls: false,
        tls: false,
        greeting: true,
    };
    session(stream, service, mode, params, &mut Meter::new(params))
        .await
        .map(|_| ())
}

/// An inbound session on an already-encrypted connection: no STARTTLS is
/// offered and one sent is refused as already in TLS mode.
pub async fn smtp_tls_session_with<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    params: &SmtpParams,
) -> io::Result<()> {
    let mode = Mode {
        submission: false,
        starttls: false,
        tls: true,
        greeting: true,
    };
    session(stream, service, mode, params, &mut Meter::new(params))
        .await
        .map(|_| ())
}

/// The caller must establish TLS before setting `protected`. The facade supplies
/// only a completed TLS stream; plaintext invocations fail before a greeting.
pub async fn submission_session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    protected: bool,
) -> io::Result<()> {
    submission_session_with(stream, service, protected, &SmtpParams::default()).await
}

pub async fn submission_session_with<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    service: Arc<MailService>,
    protected: bool,
    params: &SmtpParams,
) -> io::Result<()> {
    if !protected {
        return write(&mut stream, b"554 5.7.0 TLS required\r\n").await;
    }
    let mode = Mode {
        submission: true,
        starttls: false,
        tls: true,
        greeting: true,
    };
    session(stream, service, mode, params, &mut Meter::new(params))
        .await
        .map(|_| ())
}
