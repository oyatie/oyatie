use crate::wire::{line, write};
use mail_service::MailService;
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

mod auth;
mod command;
mod message;
mod output;

/// `encrypted` comes from the TLS listener, never client input.
pub async fn pop_session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    encrypted: bool,
) -> io::Result<()> {
    run(stream, service, encrypted, false, true)
        .await
        .map(|_| ())
}

/// The facade's upgrade must complete a server-side TLS handshake.
pub async fn pop_starttls_session<S, T, F, U>(
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
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "POP3 TLS handshake stalled"))??;
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
        write(stream.get_mut(), b"+OK Oyatie POP3 ready\r\n").await?;
    }
    let mut session = Session::default();
    while let Some(bytes) = line(&mut stream, 512).await? {
        let Some(parts) = command::parse(&bytes) else {
            write(stream.get_mut(), b"-ERR Invalid command\r\n").await?;
            continue;
        };
        if parts[0] == "STLS" {
            if parts.len() != 1 || !starttls || encrypted || session.maildrop.is_some() {
                write(stream.get_mut(), b"-ERR STLS unavailable\r\n").await?;
                continue;
            }
            if !stream.buffer().is_empty() {
                write(stream.get_mut(), b"-ERR Pipelined STLS refused\r\n").await?;
                return Ok(None);
            }
            write(stream.get_mut(), b"+OK Begin TLS negotiation\r\n").await?;
            return Ok(Some(stream.into_inner()));
        }
        if parts[0] == "AUTH" {
            auth::run(
                &mut stream,
                service.clone(),
                &mut session,
                &parts,
                encrypted,
            )
            .await?;
        } else {
            let service = service.clone();
            let (send, mut receive) = tokio::sync::mpsc::channel(2);
            let worker = tokio::task::spawn_blocking(move || {
                let mut output = output::Output::new(send);
                let close = session.respond(&service, &parts, encrypted, starttls, &mut output);
                output.finish()?;
                Ok::<_, io::Error>((session, close))
            });
            while let Some(bytes) = receive.recv().await {
                write(stream.get_mut(), &bytes).await?;
            }
            let (next, close) = worker
                .await
                .map_err(|_| io::Error::other("POP3 worker failed"))??;
            session = next;
            if close {
                return Ok(None);
            }
        }
        if session.auth_failures >= 5 {
            write(
                stream.get_mut(),
                b"-ERR Too many authentication failures\r\n",
            )
            .await?;
            return Ok(None);
        }
    }
    Ok(None)
}

#[derive(Default)]
struct Session {
    username: Option<String>,
    maildrop: Option<Maildrop>,
    auth_failures: usize,
}

struct Maildrop {
    credential: String,
    account: String,
    messages: Vec<Message>,
}
struct Message {
    id: String,
    uid: u32,
    uidl: String,
    size: usize,
    deleted: bool,
}

fn error(error: mail_kernel::Error) -> &'static str {
    use mail_kernel::Error;
    match error {
        Error::Unavailable => "-ERR [SYS/TEMP] Service unavailable\r\n",
        Error::Forbidden => "-ERR [AUTH] Access denied\r\n",
        Error::Conflict => "-ERR [SYS/TEMP] Maildrop changed\r\n",
        Error::OverQuota => "-ERR [SYS/PERM] Maildrop limit exceeded\r\n",
        Error::NotFound => "-ERR No such message\r\n",
        Error::Invalid => "-ERR Invalid command\r\n",
    }
}
