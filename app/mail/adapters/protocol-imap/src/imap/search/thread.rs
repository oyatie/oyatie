use super::{Output, Selection};
use mail_kernel::Message;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn write(
    messages: &[(usize, &Message)],
    selected: &Selection,
    ids: &[u32],
    uid: bool,
    output: &mut Output,
) {
    let ids: BTreeSet<_> = ids.iter().copied().collect();
    let mut threads: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    for (sequence, message) in messages {
        let id = if uid {
            message.uid_in(&selected.mailbox).unwrap_or_default()
        } else {
            *sequence as u32 + 1
        };
        if ids.contains(&id) {
            threads.entry(message.thread_id()).or_default().push(id);
        }
    }
    // The oracle groups both advertised algorithms by the persisted shared
    // conversation identity, sorting members and then groups numerically.
    let mut threads: Vec<_> = threads.into_values().collect();
    for thread in &mut threads {
        thread.sort_unstable();
    }
    threads.sort_unstable();
    output.extend_from_slice(b"* THREAD ");
    for thread in threads {
        output.extend_from_slice(b"(");
        for (i, id) in thread.iter().enumerate() {
            output.extend_from_slice(format!("{}{id}", if i == 0 { "" } else { " " }).as_bytes());
        }
        output.extend_from_slice(b")");
    }
    output.extend_from_slice(b"\r\n");
}
