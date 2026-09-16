use super::{
    response::Output,
    state::{Selection, synchronize},
};
use mail_kernel::{Account, Command, Message};
use mail_service::MailService;

pub(super) fn execute(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
    selection: &mut Option<Selection>,
    chosen: Vec<(usize, &Message)>,
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    let selected = selection.as_ref().ok_or("NO")?;
    let mailbox = &selected.mailbox;
    let uid = parts[1].eq_ignore_ascii_case("UID");
    let offset = if uid { 3 } else { 2 };
    let operation = parts[if uid { 2 } else { 1 }].to_ascii_uppercase();
    if parts.len() != offset + 2 || (operation == "MOVE" && selected.readonly) {
        return Err("NO");
    }
    let target = super::folders::find(account, &parts[offset + 1]).ok_or("NO [TRYCREATE]")?;
    if chosen.is_empty() {
        return Ok(None);
    }
    let source = chosen
        .iter()
        .map(|(_, m)| m.uid_in(mailbox).unwrap_or_default().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let commands = chosen
        .iter()
        .map(|(_, m)| Command::Transfer {
            id: m.id.clone(),
            mailbox: target.id.clone(),
            remove_from: (operation == "MOVE").then(|| mailbox.clone()),
        })
        .collect();
    let updated = service
        .execute(token, &account.id, account.revision, commands)
        .map_err(|_| "NO")?;
    let destination = (0..chosen.len())
        .map(|i| (u64::from(target.uid_next) + i as u64).to_string())
        .collect::<Vec<_>>()
        .join(",");
    let code = format!("[COPYUID {} {source} {destination}]", target.uid_validity);
    if operation == "MOVE" {
        output.extend_from_slice(format!("* OK {code} MOVE committed\r\n").as_bytes());
    }
    // Sequence identities were resolved before any concurrent EXPUNGE. Now
    // the command is committed, selected state can be synchronized safely.
    let mut safe = parts.to_vec();
    safe[1] = "NOOP".into();
    synchronize(&updated, &safe, selection, output);
    Ok((operation == "COPY").then_some(code))
}
