use super::{Session, folders, response::Output, state};
use crate::wire::{line, write};
use mail_api::Action;
use mail_kernel::Command;
use mail_service::MailService;
use std::{io, sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, BufReader};
mod drain;
mod metadata;
use metadata::Literal;

// Bound retained bodies AND metadata, including zero-byte literal batches.
const BUFFER_LIMIT: usize = 50 * 1024 * 1024;
// ponytail: snapshot adapters scan account metadata per command; cap batch work
// until an indexed bulk append operation replaces those repeated scans.
const MESSAGE_LIMIT: usize = 1000;

pub(super) struct Append {
    mailbox: String,
    revision: u64,
    validity: u32,
    uid: u32,
    remaining: usize,
    buffered: usize,
    commands: Vec<Command>,
}

impl Append {
    fn prepare(
        service: &MailService,
        token: &str,
        account: &str,
        mailbox: &str,
    ) -> Result<Self, &'static str> {
        service
            .authorize(token, account, Action::Write)
            .map_err(|_| "NO")?;
        let account = service.read(token, account).map_err(|_| "NO")?;
        let mailbox = folders::find(&account, mailbox).ok_or("NO [TRYCREATE]")?;
        let remaining = account
            .messages
            .iter()
            .try_fold(account.quota_bytes, |remaining, message| {
                remaining.checked_sub(message.size)
            })
            .ok_or("NO [OVERQUOTA]")?;
        Ok(Self {
            mailbox: mailbox.id.clone(),
            revision: account.revision,
            validity: mailbox.uid_validity,
            uid: mailbox.uid_next,
            remaining,
            buffered: 0,
            commands: vec![],
        })
    }

    fn reserve(&mut self, literal: &Literal) -> Result<(), &'static str> {
        if self.commands.len() >= MESSAGE_LIMIT {
            return Err("NO [MESSAGELIMIT 1000]");
        }
        self.remaining = self
            .remaining
            .checked_sub(literal.size)
            .ok_or("NO [OVERQUOTA]")?;
        let metadata = 2 * std::mem::size_of::<Command>()
            + self.mailbox.len()
            + std::mem::size_of::<String>()
            + literal.keywords.capacity() * std::mem::size_of::<String>()
            + literal.keywords.iter().map(String::capacity).sum::<usize>();
        self.buffered = self
            .buffered
            .checked_add(literal.size)
            .and_then(|n| n.checked_add(metadata))
            .filter(|n| *n <= BUFFER_LIMIT)
            .ok_or("NO [TOOBIG]")?;
        self.uid
            .checked_add(self.commands.len() as u32)
            .and_then(|uid| uid.checked_add(1))
            .ok_or("NO")?;
        Ok(())
    }

    pub(super) fn respond(
        self,
        service: &MailService,
        session: &mut Session,
        parts: &[String],
        output: &mut Output,
    ) {
        let last = self.uid + self.commands.len() as u32 - 1;
        let result = service.execute(
            &session.credential,
            &session.account_id,
            self.revision,
            self.commands,
        );
        let tag = &parts[0];
        match result {
            Ok(account) => {
                state::synchronize(&account, parts, &mut session.selected, output);
                let uids = if self.uid == last {
                    self.uid.to_string()
                } else {
                    format!("{}:{last}", self.uid)
                };
                output.extend_from_slice(
                    format!(
                        "{tag} OK [APPENDUID {} {uids}] APPEND completed\r\n",
                        self.validity
                    )
                    .as_bytes(),
                );
            }
            Err(_) => output.extend_from_slice(format!("{tag} NO APPEND failed\r\n").as_bytes()),
        }
    }
}

pub(super) async fn read<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: Arc<MailService>,
    session: &Session,
    parts: &[String],
) -> io::Result<Option<Append>> {
    tokio::time::timeout(
        Duration::from_secs(300),
        collect(stream, service, session, parts),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IMAP literal deadline"))?
}

async fn refuse<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    tag: &str,
    status: &str,
    suffix: &[u8],
) -> io::Result<Option<Append>> {
    write(
        stream.get_mut(),
        format!("{tag} {status} APPEND refused\r\n").as_bytes(),
    )
    .await?;
    super::literal::refuse_payload(suffix)
}

async fn collect<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: Arc<MailService>,
    session: &Session,
    parts: &[String],
) -> io::Result<Option<Append>> {
    let tag = &parts[0];
    let mut literal = match parts
        .get(3..)
        .ok_or("BAD")
        .and_then(|p| Literal::parse(p, session.utf8))
    {
        Ok(literal) => literal,
        Err(status) => return refuse(stream, tag, status, parts.last().unwrap().as_bytes()).await,
    };
    let token = session.credential.clone();
    let account = session.account_id.clone();
    let mailbox = parts[2].clone();
    let worker = service.clone();
    let mut append = match tokio::task::spawn_blocking(move || {
        Append::prepare(&worker, &token, &account, &mailbox)
    })
    .await
    .map_err(|_| io::Error::other("IMAP worker failed"))?
    {
        Ok(append) => append,
        Err(status) => {
            return drain::refuse(stream, tag, status, literal, session.utf8).await;
        }
    };
    let mut marker = parts.last().unwrap().as_bytes().to_vec();
    loop {
        if let Err(status) = append.reserve(&literal) {
            return if matches!(status, "NO [TOOBIG]" | "NO [MESSAGELIMIT 1000]") {
                refuse(stream, tag, status, &marker).await
            } else {
                drain::refuse(stream, tag, status, literal, session.utf8).await
            };
        }
        if !literal.nonsync {
            write(stream.get_mut(), b"+ Ready for literal\r\n").await?;
        }
        let mut raw = vec![0; literal.size];
        stream.read_exact(&mut raw).await?;
        let suffix = line(stream, 8192)
            .await?
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing APPEND suffix"))?;
        let suffix = if literal.utf8 {
            if let Some(suffix) = suffix.strip_prefix(b")") {
                suffix
            } else {
                return refuse(stream, tag, "BAD", &suffix).await;
            }
        } else {
            &suffix
        };
        if !session.utf8 && !metadata::ascii_headers(&raw) {
            return refuse(stream, tag, "NO [UTF8NOTSUPPORTED]", suffix).await;
        }
        append.commands.push(Command::Append {
            mailboxes: vec![append.mailbox.clone()],
            received_at: literal.received_at,
            raw,
            keywords: literal.keywords,
        });
        if suffix.is_empty() {
            return Ok(Some(append));
        }
        if !suffix.starts_with(b" ") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid APPEND terminator",
            ));
        }
        literal = match Literal::next(suffix, session.utf8) {
            Ok(literal) => literal,
            Err(status) => return refuse(stream, tag, status, suffix).await,
        };
        let token = session.credential.clone();
        let account = session.account_id.clone();
        let worker = service.clone();
        let allowed =
            tokio::task::spawn_blocking(move || worker.authorize(&token, &account, Action::Write))
                .await
                .map_err(|_| io::Error::other("IMAP worker failed"))?;
        if allowed.is_err() {
            return drain::refuse(stream, tag, "NO", literal, session.utf8).await;
        }
        marker = suffix.to_vec();
    }
}

#[cfg(test)]
mod tests;
