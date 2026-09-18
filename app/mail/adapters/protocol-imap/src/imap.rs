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
        let (send, mut receive) = tokio::sync::mpsc::channel(2);
        let service = service.clone();
        let worker = tokio::task::spawn_blocking(move || -> io::Result<_> {
            let mut output = response::Output::new(send);
            let logout =
                session.respond(&service, &parts, encrypted, starttls, append, &mut output);
            output.finish()?;
            Ok((session, logout))
        });
        while let Some(bytes) = receive.recv().await {
            write(stream.get_mut(), &bytes).await?;
        }
        let (next, logout) = worker
            .await
            .map_err(|_| io::Error::other("IMAP worker failed"))??;
        session = next;
        if logout {
            return Ok(None);
        }
    }
    Ok(None)
}

#[derive(Default)]
struct Session {
    utf8: bool,
    condstore: bool,
    qresync: bool,
    objectid: bool,
    uidonly: bool,
    credential: String,
    account_id: String,
    selected: Option<selected::Selection>,
}
impl Session {
    fn respond(
        &mut self,
        service: &MailService,
        parts: &[String],
        encrypted: bool,
        starttls: bool,
        append: Option<append::Append>,
        output: &mut response::Output,
    ) -> bool {
        output.utf8 = self.utf8;
        output.condstore = self.condstore;
        output.qresync = self.qresync;
        output.objectid = self.objectid;
        output.uidonly = self.uidonly;
        if let Some(append) = append {
            append.respond(service, self, parts, output);
            return false;
        }
        let tag = &parts[0];
        let verb = parts[1].to_ascii_uppercase();
        if matches!(
            verb.as_str(),
            "CAPABILITY"
                | "LOGOUT"
                | "NOOP"
                | "CHECK"
                | "CLOSE"
                | "EXPUNGE"
                | "UNAUTHENTICATE"
                | "NAMESPACE"
        ) && parts.len() != 2
        {
            output
                .extend_from_slice(format!("{tag} BAD Command takes no arguments\r\n").as_bytes());
            return false;
        }
        let mut status = "OK";
        let mut code = None;
        match verb.as_str() {
            "CAPABILITY" => uidonly::capabilities(self, encrypted, starttls, output),
            "ID" => uidonly::id(output),
            "UNAUTHENTICATE" => {
                if let Err(kind) = uidonly::unauthenticate(self, output) {
                    status = kind;
                }
            }
            "ENABLE" => {
                if parts.len() < 3 || self.credential.is_empty() {
                    status = "BAD";
                } else if service.read(&self.credential, &self.account_id).is_err() {
                    status = "NO";
                } else if let Err(kind) = condstore::enable(self, parts, output) {
                    status = kind;
                }
            }
            "LOGOUT" => {
                output.extend_from_slice(
                    format!("* BYE Closing\r\n{tag} OK LOGOUT completed\r\n").as_bytes(),
                );
                return true;
            }
            "LOGIN" => {
                if !encrypted || !self.credential.is_empty() || parts.len() != 4 {
                    status = "NO";
                } else {
                    match auth::login(service, &parts[2], &parts[3]) {
                        Ok(account) => {
                            self.credential.clone_from(&parts[3]);
                            self.account_id = account;
                        }
                        Err(_) => status = "NO",
                    }
                }
            }
            "NOOP" if self.credential.is_empty() => {}
            _ => match service.read(&self.credential, &self.account_id) {
                Err(_) => status = "NO",
                Ok(account) => {
                    selected::synchronize(&account, parts, &mut self.selected, output);
                    let result = command(
                        service,
                        &self.credential,
                        &account,
                        parts,
                        &mut self.selected,
                        output,
                    );
                    match result {
                        Ok(completion) => code = completion,
                        Err(kind) => status = kind,
                    }
                }
            },
        }

        self.condstore = output.condstore;
        self.objectid = output.objectid;
        let mode = if status == "OK" && matches!(verb.as_str(), "SELECT" | "EXAMINE") {
            if self.selected.as_ref().is_some_and(|s| s.readonly) {
                " [READ-ONLY]"
            } else {
                " [READ-WRITE]"
            }
        } else {
            ""
        };
        let code = code.map(|s| format!(" {s}")).unwrap_or_default();
        output.extend_from_slice(
            format!("{tag} {status}{code}{mode} {verb} completed\r\n").as_bytes(),
        );
        false
    }
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
mod search;
mod select;
mod selected;
mod state;
mod status;
mod store;
mod syntax;
mod transfer;
mod uidonly;
use commands::command;
pub use fetch::render;
pub use folders::matches as list_matches;
use objectid::object_id;
