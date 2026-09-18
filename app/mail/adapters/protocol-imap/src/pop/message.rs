use super::{Maildrop, Message, output::Output};
use mail_api::Precondition;
use mail_kernel::{Command, Error};
use mail_service::{Budget, MailService};

impl Maildrop {
    fn totals(&self) -> (usize, usize) {
        self.messages
            .iter()
            .filter(|m| !m.deleted)
            .fold((0, 0), |(n, size), m| (n + 1, size + m.size))
    }
    fn selected(&self, value: &str) -> Result<(usize, &Message), Error> {
        let index = number(value)?.checked_sub(1).ok_or(Error::NotFound)?;
        let message = self
            .messages
            .get(index)
            .filter(|m| !m.deleted)
            .ok_or(Error::NotFound)?;
        Ok((index + 1, message))
    }
    pub(super) fn respond(
        &mut self,
        service: &MailService,
        verb: &str,
        args: &[String],
        output: &mut Output,
    ) -> Result<(), Error> {
        match verb {
            "STAT" if args.is_empty() => {
                let (count, size) = self.totals();
                output.text(&format!("+OK {count} {size}\r\n"));
            }
            "LIST" | "UIDL" if args.len() <= 1 => {
                if let Some(value) = args.first() {
                    let (index, message) = self.selected(value)?;
                    let value = if verb == "LIST" {
                        message.size.to_string()
                    } else {
                        message.uidl.clone()
                    };
                    output.text(&format!("+OK {index} {value}\r\n"));
                } else {
                    output.text(&format!("+OK {} messages\r\n", self.totals().0));
                    for (i, message) in self.messages.iter().enumerate().filter(|(_, m)| !m.deleted)
                    {
                        if output.is_closed() {
                            break;
                        }
                        let value = if verb == "LIST" {
                            message.size.to_string()
                        } else {
                            message.uidl.clone()
                        };
                        output.text(&format!("{} {value}\r\n", i + 1));
                    }
                    output.text(".\r\n");
                }
            }
            "RETR" | "TOP" if args.len() == if verb == "TOP" { 2 } else { 1 } => {
                let (_, message) = self.selected(&args[0])?;
                let lines = if verb == "TOP" { number(&args[1])? } else { 0 };
                let current = service.messages(
                    &self.credential,
                    &self.account,
                    std::slice::from_ref(&message.id),
                )?;
                if !current
                    .messages
                    .iter()
                    .any(|m| m.id == message.id && m.uid_in("inbox") == Some(message.uid))
                {
                    return Err(Error::NotFound);
                }
                let raw = service.download(&self.credential, &self.account, &message.id)?;
                if raw.len() != message.size || raw.len() > mail_kernel::MAX_MESSAGE_BYTES {
                    return Err(Error::Unavailable);
                }
                output.text(&format!("+OK {} octets\r\n", message.size));
                multiline(&raw, lines, output);
            }
            "DELE" if args.len() == 1 => {
                let (index, _) = self.selected(&args[0])?;
                service.authorize(&self.credential, &self.account, mail_api::Action::Write)?;
                self.messages[index - 1].deleted = true;
                output.text("+OK Message marked for deletion\r\n");
            }
            "RSET" if args.is_empty() => {
                for message in &mut self.messages {
                    message.deleted = false;
                }
                output.text("+OK Deletions reset\r\n");
            }
            _ => return Err(Error::Invalid),
        }
        Ok(())
    }

    /// One observed batch: the store re-applies it on top of a concurrent
    /// commit and skips messages that are already gone, so no re-read loop
    /// is needed and a deletion is never partially committed.
    pub(super) fn commit(&self, service: &MailService, budget: &Budget) -> Result<(), Error> {
        if !self.messages.iter().any(|m| m.deleted) {
            return Ok(());
        }
        service.authorize(&self.credential, &self.account, mail_api::Action::Write)?;
        let account = service.read(&self.credential, &self.account)?;
        let current: std::collections::BTreeMap<_, _> = account
            .messages
            .iter()
            .map(|m| (m.id.as_str(), m))
            .collect();
        let mut commands = Vec::new();
        for pending in self.messages.iter().filter(|m| m.deleted) {
            if let Some(current) = current
                .get(pending.id.as_str())
                .filter(|m| m.uid_in(account.inbox()) == Some(pending.uid))
            {
                let mailboxes: Vec<_> = current
                    .mailboxes
                    .keys()
                    .filter(|m| m.as_str() != account.inbox())
                    .cloned()
                    .collect();
                commands.push(if mailboxes.is_empty() {
                    Command::Destroy {
                        id: current.id.clone(),
                    }
                } else {
                    Command::SetMailboxes {
                        id: current.id.clone(),
                        mailboxes,
                    }
                });
            }
        }
        if commands.is_empty() {
            return Ok(());
        }
        service
            .execute(
                &self.credential,
                &self.account,
                Precondition::Observed(account.revision),
                commands,
                budget,
            )
            .map(|_| ())
    }
}

fn number(value: &str) -> Result<usize, Error> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::Invalid);
    }
    value
        .parse::<u32>()
        .map(|n| n as usize)
        .map_err(|_| Error::Invalid)
}

fn multiline(raw: &[u8], lines: usize, output: &mut Output) {
    // Compatibility with Stalwart TOP: positive counts include headers; zero
    // sends the entire message, as does RETR. Keep content as bytes throughout.
    for (index, line) in raw.split_inclusive(|b| *b == b'\n').enumerate() {
        if output.is_closed() || (lines != 0 && index == lines) {
            break;
        }
        if line.first() == Some(&b'.') {
            output.bytes(b".");
        }
        let line = match line.strip_suffix(b"\n") {
            Some(line) => line.strip_suffix(b"\r").unwrap_or(line),
            None => line,
        };
        output.bytes(line);
        output.bytes(b"\r\n");
    }
    output.bytes(b".\r\n");
}
