use super::{Outcome, Session, auth, output::Output};
use mail_kernel::Error;
use mail_service::{Budget, MailService};

pub(super) fn parse(bytes: &[u8]) -> Option<Vec<String>> {
    let text = std::str::from_utf8(bytes).ok()?;
    if text.chars().any(char::is_control) {
        return None;
    }
    let (verb, args) = text.split_once(' ').unwrap_or((text, ""));
    if verb.is_empty() || !verb.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }
    let verb = verb.to_ascii_uppercase();
    let mut parts = vec![verb.clone()];
    if verb == "PASS" {
        if !args.is_empty() {
            parts.push(args.into());
        }
    } else {
        parts.extend(args.split(' ').filter(|s| !s.is_empty()).map(String::from));
    }
    Some(parts)
}

impl Session {
    pub(super) fn respond(
        &mut self,
        service: &MailService,
        parts: &[String],
        encrypted: bool,
        starttls: bool,
        budget: &Budget,
        output: &mut Output,
    ) -> Outcome {
        let close = if parts[0] == "QUIT" && parts.len() == 1 {
            Outcome::Close
        } else {
            Outcome::Continue
        };
        if let Some(maildrop) = &self.maildrop
            && let Err(error) = service.authorize(
                &maildrop.credential,
                &maildrop.account,
                mail_api::Action::Read,
            )
        {
            output.text(super::error(error));
            return close;
        }
        match self.command(service, parts, encrypted, starttls, budget, output) {
            // The socket task backs off and re-dispatches within the budget.
            Err(Error::Busy) => Outcome::Busy,
            Err(error) => {
                output.text(super::error(error));
                close
            }
            Ok(()) => close,
        }
    }

    fn command(
        &mut self,
        service: &MailService,
        parts: &[String],
        encrypted: bool,
        starttls: bool,
        budget: &Budget,
        output: &mut Output,
    ) -> Result<(), Error> {
        let verb = parts[0].as_str();
        let args = &parts[1..];
        match verb {
            "CAPA" if args.is_empty() => {
                output.text("+OK Capability list\r\nTOP\r\nUIDL\r\nRESP-CODES\r\nPIPELINING\r\nUTF8\r\nIMPLEMENTATION Oyatie\r\n");
                if encrypted && self.maildrop.is_none() {
                    output.text("USER\r\nSASL PLAIN\r\n");
                }
                if starttls && !encrypted && self.maildrop.is_none() {
                    output.text("STLS\r\n");
                }
                output.text(".\r\n");
            }
            "NOOP" | "UTF8" if args.is_empty() => output.text("+OK\r\n"),
            "QUIT" if args.is_empty() => {
                if let Some(maildrop) = &self.maildrop {
                    maildrop.commit(service, budget)?;
                }
                output.text("+OK Goodbye\r\n");
            }
            "USER" if args.len() == 1 && encrypted && self.maildrop.is_none() => {
                self.username = Some(args[0].clone());
                output.text("+OK Send password\r\n");
            }
            "PASS" if args.len() == 1 && encrypted && self.maildrop.is_none() => {
                let username = self.username.take().ok_or(Error::Invalid)?;
                match auth::login(service, &username, &args[0]) {
                    Ok(maildrop) => {
                        self.maildrop = Some(maildrop);
                        output.text("+OK Authenticated\r\n");
                    }
                    Err(error) => {
                        self.auth_failures += 1;
                        return Err(error);
                    }
                }
            }
            "STAT" | "LIST" | "UIDL" | "RETR" | "TOP" | "DELE" | "RSET" => {
                self.maildrop
                    .as_mut()
                    .ok_or(Error::Forbidden)?
                    .respond(service, verb, args, output)?;
            }
            _ => return Err(Error::Invalid),
        }
        Ok(())
    }
}
