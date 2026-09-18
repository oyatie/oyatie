use super::{Account, Error, valid_keywords};

impl Account {
    pub(super) fn set_keywords(
        &mut self,
        id: &str,
        mut keywords: Vec<String>,
    ) -> Result<(), Error> {
        if !valid_keywords(&keywords) {
            return Err(Error::Invalid);
        }
        let index = self.index_of(id)?;
        keywords.sort();
        keywords.dedup();
        let was_seen = self.messages[index].seen();
        let now_seen = keywords.iter().any(|k| k == "$seen");
        if was_seen != now_seen {
            let links: Vec<_> = self.messages[index].mailboxes.keys().cloned().collect();
            for folder in self.mailboxes.iter_mut().filter(|m| links.contains(&m.id)) {
                if now_seen {
                    folder.unread_emails -= 1;
                } else {
                    folder.unread_emails += 1;
                }
            }
        }
        self.messages[index].keywords = keywords;
        Ok(())
    }

    pub(super) fn destroy(&mut self, id: &str) -> Result<(), Error> {
        let index = self.index_of(id)?;
        let links: Vec<_> = self.messages[index].mailboxes.keys().cloned().collect();
        for mailbox in links {
            self.detach(index, &mailbox);
        }
        Ok(())
    }

    /// Unlink every loaded `$deleted` message from the mailbox. The working
    /// set must hold at least those members (`Scope::Flagged`).
    pub(super) fn expunge(&mut self, mailbox: &str) -> Result<(), Error> {
        if !self.mailboxes.iter().any(|m| m.id == mailbox) {
            return Err(Error::NotFound);
        }
        let mut index = 0;
        while index < self.messages.len() {
            let message = &self.messages[index];
            if message.mailboxes.contains_key(mailbox)
                && message.keywords.iter().any(|k| k == "$deleted")
            {
                let before = self.messages.len();
                self.detach(index, mailbox);
                if self.messages.len() == before {
                    index += 1;
                }
            } else {
                index += 1;
            }
        }
        Ok(())
    }
}
