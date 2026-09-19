use super::super::{mailboxes::quote, response::Output, selected::messages_in};
use mail_kernel::Account;
use std::collections::BTreeMap;

/// Mailbox-level snapshot compared between IDLE refreshes. Mailbox creation
/// and deletion surface as unsolicited LIST lines; changed message counters
/// surface as unsolicited STATUS lines, before any selected-mailbox update.
#[derive(Default)]
pub(super) struct Watch {
    paths: BTreeMap<String, String>,
    counters: BTreeMap<String, (usize, usize, u32)>,
}

impl Watch {
    pub(super) fn refresh(&mut self, account: &Account, initial: bool, output: &mut Output) {
        let mut paths = BTreeMap::new();
        let mut counters = BTreeMap::new();
        let mut order = Vec::new();
        for mailbox in &account.mailboxes {
            let Some(path) = account.mailbox_path(&mailbox.id) else {
                continue;
            };
            let messages = messages_in(account, &mailbox.id);
            let unseen = messages
                .iter()
                .filter(|m| !m.keywords.iter().any(|k| k == "$seen"))
                .count();
            paths.insert(mailbox.id.clone(), path);
            counters.insert(
                mailbox.id.clone(),
                (messages.len(), unseen, mailbox.uid_next),
            );
            order.push(mailbox.id.as_str());
        }
        if !initial {
            for (id, path) in &self.paths {
                if paths.get(id) != Some(path) {
                    let name = quote(path, output.utf8);
                    output.extend_from_slice(
                        format!("* LIST (\\NonExistent) \"/\" {name}\r\n").as_bytes(),
                    );
                }
            }
            for id in &order {
                let path = &paths[*id];
                if self.paths.get(*id) != Some(path) {
                    let name = quote(path, output.utf8);
                    output.extend_from_slice(format!("* LIST () \"/\" {name}\r\n").as_bytes());
                }
            }
            for id in &order {
                let current = counters[*id];
                if self
                    .counters
                    .get(*id)
                    .is_some_and(|previous| *previous != current)
                {
                    let name = quote(&paths[*id], output.utf8);
                    let (messages, unseen, uid_next) = current;
                    output.extend_from_slice(
                        format!(
                            "* STATUS {name} (MESSAGES {messages} UNSEEN {unseen} UIDNEXT {uid_next})\r\n"
                        )
                        .as_bytes(),
                    );
                }
            }
        }
        self.paths = paths;
        self.counters = counters;
    }
}
