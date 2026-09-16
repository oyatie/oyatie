use super::Session;
use crate::{
    sasl::plain,
    wire::{line, write},
};
use mail_kernel::Error;
use mail_service::MailService;
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

// LOGIN and SASL bind the same account identity and current Read policy. Neither
// authentication path loads mailbox contents just to establish a session.
pub(super) fn login(service: &MailService, username: &str, token: &str) -> Result<String, Error> {
    let principal = service.principal(token)?;
    let account = service.authorize(token, &principal.account, mail_api::Action::Read)?;
    if !account.address.eq_ignore_ascii_case(username) {
        return Err(Error::Forbidden);
    }
    Ok(account.id)
}

pub(super) async fn run<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: Arc<MailService>,
    session: &mut Session,
    parts: &[String],
    encrypted: bool,
) -> io::Result<()> {
    let tag = &parts[0];
    let refusal = if !encrypted {
        Some("NO [PRIVACYREQUIRED] TLS required")
    } else if !session.credential.is_empty() {
        Some("BAD Already authenticated")
    } else if !(3..=4).contains(&parts.len()) {
        Some("BAD Invalid authentication arguments")
    } else if !parts[2].eq_ignore_ascii_case("PLAIN") {
        Some("NO Unsupported authentication mechanism")
    } else {
        None
    };
    if let Some(refusal) = refusal {
        return write(stream.get_mut(), format!("{tag} {refusal}\r\n").as_bytes()).await;
    }
    let response = if let Some(response) = parts.get(3) {
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
    let status = if response == b"*" {
        "BAD Authentication cancelled"
    } else if let Some((authorization, username, token)) = plain(&response) {
        let result = tokio::task::spawn_blocking(move || {
            if !authorization.is_empty() && !authorization.eq_ignore_ascii_case(&username) {
                return Err(Error::Forbidden);
            }
            let account = login(&service, &username, &token)?;
            Ok((account, token))
        })
        .await
        .map_err(|_| io::Error::other("authentication worker failed"))?;
        match result {
            Ok((account, token)) => {
                session.account_id = account;
                session.credential = token;
                "OK AUTHENTICATE completed"
            }
            Err(Error::Unavailable) => "NO [UNAVAILABLE] Authentication unavailable",
            Err(_) => "NO [AUTHENTICATIONFAILED] Authentication failed",
        }
    } else {
        "BAD Invalid PLAIN response"
    };
    write(stream.get_mut(), format!("{tag} {status}\r\n").as_bytes()).await
}
