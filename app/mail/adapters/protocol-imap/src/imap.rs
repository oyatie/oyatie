use crate::wire::{line, write};
use mail_service::MailService;
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

/// `encrypted` is supplied by the TLS listener, never by client input.
pub async fn imap_session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    encrypted: bool,
) -> io::Result<()> {
    run(stream, service, encrypted, false, true)
        .await
        .map(|_| ())
}

/// The facade's upgrade must perform a verified server-side TLS handshake.
pub async fn imap_starttls_session<S, T, F, U>(
    stream: S,
    service: Arc<MailService>,
    upgrade: F,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
    F: FnOnce(S) -> U,
    U: std::future::Future<Output = io::Result<T>>,
{
    if let Some(stream) = run(stream, service.clone(), false, true, true).await? {
        let stream = tokio::time::timeout(std::time::Duration::from_secs(10), upgrade(stream))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IMAP TLS handshake stalled"))??;
        run(stream, service, true, false, false).await?;
    }
    Ok(())
}

async fn run<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    encrypted: bool,
    starttls: bool,
    greeting: bool,
) -> io::Result<Option<S>> {
    let mut stream = BufReader::new(stream);
    if greeting {
        write(stream.get_mut(), b"* OK Oyatie mail ready\r\n").await?;
    }
    let mut session = Session::default();
    while let Some(bytes) = line(&mut stream, 8192).await? {
        let Some(mut parts) =
            literal::read(&mut stream, &bytes, service.clone(), &session, encrypted).await?
        else {
            continue;
        };
        let tag = parts[0].clone();
        if uidonly::requires_uid(&session, &parts[1]) {
            write(
                stream.get_mut(),
                format!("{tag} BAD [UIDREQUIRED] Message numbers are disabled\r\n").as_bytes(),
            )
            .await?;
            continue;
        }
        if mailboxes::decode_command(&mut parts, session.utf8).is_none() {
            write(
                stream.get_mut(),
                format!("{tag} BAD Invalid mailbox encoding\r\n").as_bytes(),
            )
            .await?;
            literal::refuse_payload::<()>(parts.last().unwrap().as_bytes())?;
            continue;
        }
        let tag = &parts[0];
        if parts[1].eq_ignore_ascii_case("STARTTLS") {
            if !starttls || encrypted || !session.credential.is_empty() || parts.len() != 2 {
                write(
                    stream.get_mut(),
                    format!("{tag} BAD STARTTLS unavailable\r\n").as_bytes(),
                )
                .await?;
                continue;
            }
            // No plaintext bytes may cross the TLS boundary. Compliant clients
            // wait for tagged OK before sending their TLS ClientHello.
            if !stream.buffer().is_empty() {
                write(stream.get_mut(), b"* BYE Pipelined STARTTLS refused\r\n").await?;
                return Ok(None);
            }
            write(
                stream.get_mut(),
                format!("{tag} OK Begin TLS negotiation\r\n").as_bytes(),
            )
            .await?;
            return Ok(Some(stream.into_inner()));
        }
        if parts[1].eq_ignore_ascii_case("AUTHENTICATE") {
            auth::run(
                &mut stream,
                service.clone(),
                &mut session,
                &parts,
                encrypted,
            )
            .await?;
            continue;
        }
        if parts[1].eq_ignore_ascii_case("IDLE") {
            if idle::run(&mut stream, service.clone(), &mut session, &parts).await? {
                return Ok(None);
            }
            continue;
        }
        let append = if parts[1].eq_ignore_ascii_case("APPEND") {
            match append::read(&mut stream, service.clone(), &session, &parts).await? {
                Some(append) => Some(append),
                None => continue,
            }
        } else {
            None
        };
        // The command budget starts here: after the literal, before the worker.
        let transport = Transport {
            encrypted,
            starttls,
        };
        let (next, logout) =
            retry::dispatch(&mut stream, &service, session, parts, append, transport).await?;
        session = next;
        if logout {
            return Ok(None);
        }
    }
    Ok(None)
}

mod append;
mod auth;
mod commands;
mod condstore;
mod fetch;
mod folders;
mod hierarchy;
mod idle;
mod list;
mod literal;
mod mailboxes;
mod objectid;
mod response;
mod retry;
mod search;
mod select;
mod selected;
mod session;
mod state;
mod status;
mod store;
mod syntax;
mod transfer;
mod uidonly;
pub use fetch::render;
pub use folders::matches as list_matches;
use objectid::object_id;
use session::{Session, Transport};
