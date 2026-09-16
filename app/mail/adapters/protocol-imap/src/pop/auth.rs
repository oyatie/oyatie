use super::{Maildrop, Message, Session};
use crate::{
    sasl::plain,
    wire::{line, write},
};
use mail_kernel::Error;
use mail_service::MailService;
use sha2::{Digest, Sha256};
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

pub(super) fn login(service: &MailService, username: &str, token: &str) -> Result<Maildrop, Error> {
    let principal = service.principal(token)?;
    let info = service.authorize(token, &principal.account, mail_api::Action::Read)?;
    if !info.address.eq_ignore_ascii_case(username) {
        return Err(Error::Forbidden);
    }
    let account = service.read(token, &info.id)?;
    let inbox = account
        .mailboxes
        .iter()
        .find(|m| m.id == account.inbox())
        .ok_or(Error::NotFound)?;
    let mut messages = Vec::new();
    for message in &account.messages {
        if let Some(uid) = message.uid_in(account.inbox()) {
            if messages.len() == 100_000 {
                return Err(Error::OverQuota);
            }
            let mut hash = Sha256::new();
            hash.update((account.id.len() as u64).to_be_bytes());
            hash.update(account.id.as_bytes());
            hash.update(inbox.uid_validity.to_be_bytes());
            hash.update(uid.to_be_bytes());
            messages.push(Message {
                id: message.id.clone(),
                uid,
                uidl: format!("{:x}", hash.finalize()),
                size: message.size,
                deleted: false,
            });
        }
    }
    messages.sort_unstable_by_key(|m| m.uid);
    messages
        .iter()
        .try_fold(0usize, |n, m| n.checked_add(m.size))
        .ok_or(Error::OverQuota)?;
    Ok(Maildrop {
        credential: token.into(),
        account: info.id,
        messages,
    })
}

pub(super) async fn run<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: Arc<MailService>,
    session: &mut Session,
    parts: &[String],
    encrypted: bool,
) -> io::Result<()> {
    session.username = None;
    if !encrypted || session.maildrop.is_some() {
        return write(stream.get_mut(), b"-ERR Authentication unavailable\r\n").await;
    }
    if parts.len() == 1 {
        return write(
            stream.get_mut(),
            b"+OK Supported mechanisms\r\nPLAIN\r\n.\r\n",
        )
        .await;
    }
    if !(2..=3).contains(&parts.len()) || !parts[1].eq_ignore_ascii_case("PLAIN") {
        return write(
            stream.get_mut(),
            b"-ERR Unsupported authentication mechanism\r\n",
        )
        .await;
    }
    let response = if let Some(response) = parts.get(2) {
        response.as_bytes().to_vec()
    } else {
        write(stream.get_mut(), b"+ \r\n").await?;
        line(stream, 8192).await?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "authentication response missing",
            )
        })?
    };
    if response == b"*" {
        return write(stream.get_mut(), b"-ERR Authentication cancelled\r\n").await;
    }
    let result = if let Some((authorization, username, token)) = plain(&response) {
        tokio::task::spawn_blocking(move || {
            if !authorization.is_empty() && !authorization.eq_ignore_ascii_case(&username) {
                return Err(Error::Forbidden);
            }
            login(&service, &username, &token)
        })
        .await
        .map_err(|_| io::Error::other("POP3 authentication worker failed"))?
    } else {
        Err(Error::Invalid)
    };
    match result {
        Ok(maildrop) => {
            session.maildrop = Some(maildrop);
            write(stream.get_mut(), b"+OK Authenticated\r\n").await
        }
        Err(error) => {
            session.auth_failures += 1;
            write(stream.get_mut(), super::error(error).as_bytes()).await
        }
    }
}
