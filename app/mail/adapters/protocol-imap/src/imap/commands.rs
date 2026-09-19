use super::hierarchy;
use mail_kernel::{Account, Command};
use mail_service::MailService;

pub(super) fn command(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
    selected: &mut Option<super::selected::Selection>,
    output: &mut super::response::Output,
) -> Result<Option<String>, &'static str> {
    let verb = parts[1].to_ascii_uppercase();
    match verb.as_str() {
        "NOOP" => Ok(None),
        "CHECK" if selected.is_some() => Ok(None),
        "NAMESPACE" => {
            output.extend_from_slice(b"* NAMESPACE ((\"\" \"/\")) NIL NIL\r\n");
            Ok(None)
        }
        // RFC 3691 takes no arguments; a stray mailbox name is tolerated.
        "UNSELECT" if parts.len() <= 3 => {
            selected.take().ok_or("BAD")?;
            Ok(None)
        }
        "STATUS" => super::status::execute(account, parts, output),
        "LIST" | "LSUB" if parts.len() == 3 => super::list::execute(account, parts, output),
        "SELECT" | "EXAMINE" => {
            super::select::execute(service, token, account, parts, selected, output)
        }
        "CREATE" if parts.len() == 4 => hierarchy::create(service, token, account, parts, output),
        "RENAME" if parts.len() == 4 => hierarchy::rename(service, token, account, parts, output),
        "DELETE" if parts.len() == 3 => hierarchy::delete(service, token, account, parts),
        "SUBSCRIBE" | "UNSUBSCRIBE" if parts.len() == 3 => {
            hierarchy::subscribe(service, token, account, parts)
        }
        "CLOSE" | "EXPUNGE" => {
            let selection = selected.as_ref().ok_or("BAD")?;
            let mailbox = &selection.mailbox;
            if selection.readonly {
                if verb == "CLOSE" {
                    *selected = None;
                    return Ok(None);
                }
                return Err("NO");
            }
            let updated = service
                .execute(
                    token,
                    &account.id,
                    account.revision,
                    vec![Command::Expunge {
                        mailbox: mailbox.clone(),
                    }],
                )
                .map_err(|_| "NO")?;
            if verb == "EXPUNGE" {
                super::selected::synchronize(&updated, parts, selected, output);
            } else {
                *selected = None;
            }
            Ok(None)
        }
        "UID" | "FETCH" | "STORE" | "SEARCH" | "SORT" | "THREAD" | "COPY" | "MOVE" => {
            super::selected::selected_command(service, token, account, parts, selected, output)
        }
        _ => Err("BAD"),
    }
}
