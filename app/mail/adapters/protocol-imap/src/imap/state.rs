use super::selected::messages_in;
use mail_kernel::Account;

pub(super) struct Selection {
    pub mailbox: String,
    pub uid_validity: u32,
    pub readonly: bool,
    pub condstore: bool,
    pub qresync: bool,
    pub modseq: u64,
    pub ids: Vec<(String, u32)>,
    pub saved: std::collections::BTreeSet<(String, u32)>,
}

impl Selection {
    pub fn largest_uid(&self, account: &Account) -> u32 {
        account
            .messages
            .iter()
            .filter_map(|m| m.uid_in(&self.mailbox))
            .max()
            .unwrap_or_default()
    }
}

pub(super) fn synchronize(
    account: &Account,
    parts: &[String],
    selected: &mut Option<Selection>,
    output: &mut super::response::Output,
) {
    let Some(selection) = selected else {
        return;
    };
    if !account
        .mailboxes
        .iter()
        .any(|m| m.id == selection.mailbox && m.uid_validity == selection.uid_validity)
    {
        *selected = None;
        output.extend_from_slice(b"* OK [CLOSED] Selected mailbox no longer available\r\n");
        return;
    }
    let previous_count = selection.ids.len();
    let mut vanished = Vec::new();
    let current = messages_in(account, &selection.mailbox);
    let ids: std::collections::BTreeSet<_> = current
        .iter()
        .map(|m| {
            (
                m.id.as_str(),
                m.uid_in(&selection.mailbox).unwrap_or_default(),
            )
        })
        .collect();
    // RFC 3501 forbids EXPUNGE while responding to sequence FETCH/STORE/SEARCH.
    // Retain vanished entries as tombstones until a command permits notification.
    if !matches!(
        parts[1].to_ascii_uppercase().as_str(),
        "FETCH" | "STORE" | "SEARCH" | "SORT" | "THREAD" | "COPY" | "MOVE"
    ) {
        let mut sequence = 1;
        selection.ids.retain(|(id, uid)| {
            if ids.contains(&(id.as_str(), *uid)) {
                sequence += 1;
                true
            } else {
                if selection.qresync || output.uidonly {
                    vanished.push(*uid);
                } else {
                    output.extend_from_slice(format!("* {sequence} EXPUNGE\r\n").as_bytes());
                }
                false
            }
        });
    }
    if !vanished.is_empty() {
        output.extend_from_slice(
            format!("* VANISHED {}\r\n", super::condstore::ranges(vanished)).as_bytes(),
        );
    }
    let known: std::collections::BTreeSet<_> = selection.ids.iter().cloned().collect();
    let before = selection.ids.len();
    selection.ids.extend(
        current
            .iter()
            .map(|m| {
                (
                    m.id.clone(),
                    m.uid_in(&selection.mailbox).unwrap_or_default(),
                )
            })
            .filter(|entry| !known.contains(entry)),
    );
    // EXISTS follows both arrivals and expunges, so the client's message count
    // is restated after every change (RFC 3501 §7.3.1, matching Stalwart).
    if before != selection.ids.len() || previous_count != selection.ids.len() {
        output.extend_from_slice(format!("* {} EXISTS\r\n", selection.ids.len()).as_bytes());
    }
    if selection.condstore && !parts[1].eq_ignore_ascii_case("IDLE") {
        let by_id: std::collections::BTreeMap<_, _> =
            current.iter().map(|m| (m.id.as_str(), *m)).collect();
        for (i, (id, uid)) in selection.ids.iter().enumerate() {
            if let Some(m) = by_id
                .get(id.as_str())
                .filter(|m| m.uid_in(&selection.mailbox) == Some(*uid))
                && m.modseq > selection.modseq
                && known.contains(&(id.clone(), *uid))
            {
                output.fetch_start(i + 1, *uid);
                output.extend_from_slice(
                    format!(
                        " FLAGS ({}) MODSEQ ({}))\r\n",
                        super::syntax::flags(m),
                        m.modseq
                    )
                    .as_bytes(),
                );
            }
        }
    }
    selection.modseq = account.mail_modseq;
}
