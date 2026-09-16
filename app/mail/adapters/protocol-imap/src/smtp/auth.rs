use crate::sasl::plain;
use crate::wire::{line, write};
use mail_kernel::Error;
use mail_service::MailService;
use std::{io, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

#[derive(Default)]
pub(super) struct Submission {
    pub credentials: Option<(String, String)>,
    pub failures: u8,
}

impl Submission {
    pub async fn authenticate<S: AsyncRead + AsyncWrite + Unpin>(
        &mut self,
        stream: &mut BufReader<S>,
        service: Arc<MailService>,
        arg: &str,
    ) -> io::Result<&'static str> {
        let mut fields = arg.split_ascii_whitespace();
        if !fields
            .next()
            .is_some_and(|s| s.eq_ignore_ascii_case("PLAIN"))
        {
            return Ok("504 5.5.4 Unsupported authentication mechanism\r\n");
        }
        let response = fields.next();
        if fields.next().is_some() {
            return Ok("501 5.5.2 Invalid AUTH arguments\r\n");
        }
        let response = if let Some(response) = response {
            response.as_bytes().to_vec()
        } else {
            write(stream.get_mut(), b"334 \r\n").await?;
            line(stream, 12288).await?.unwrap_or_default()
        };
        self.failures += 1;
        if response == b"*" {
            return Ok("501 5.7.0 Authentication cancelled\r\n");
        }
        let Some((authorization, username, token)) = plain(&response) else {
            return Ok("501 5.5.2 Invalid PLAIN response\r\n");
        };
        let result = tokio::task::spawn_blocking(move || {
            if !authorization.is_empty() && !authorization.eq_ignore_ascii_case(&username) {
                return Err(Error::Forbidden);
            }
            service.submission_account(&token, &username)?;
            Ok((username, token))
        })
        .await
        .unwrap_or(Err(Error::Unavailable));
        match result {
            Ok(credentials) => {
                self.credentials = Some(credentials);
                self.failures = 0;
                Ok("235 2.7.0 Authentication successful\r\n")
            }
            Err(Error::Unavailable) => Ok("454 4.7.0 Authentication unavailable\r\n"),
            Err(_) => Ok("535 5.7.8 Authentication failed\r\n"),
        }
    }

    pub async fn sender(&self, service: Arc<MailService>, sender: &str) -> &'static str {
        let Some((username, token)) = self.credentials.clone() else {
            return "530 5.7.0 Authentication required\r\n";
        };
        let sender = sender.to_owned();
        match tokio::task::spawn_blocking(move || {
            let account = service.submission_account(&token, &username)?;
            if account.address.eq_ignore_ascii_case(&sender) {
                Ok(())
            } else {
                Err(Error::Forbidden)
            }
        })
        .await
        .unwrap_or(Err(Error::Unavailable))
        {
            Ok(()) => "250 2.1.0 Sender accepted\r\n",
            Err(Error::Unavailable) => "451 4.3.0 Authorization unavailable\r\n",
            Err(_) => "550 5.7.1 Sender not authorized\r\n",
        }
    }
}
