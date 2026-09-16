use super::mailboxes::quote;
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
        "UNSELECT" => {
            selected.take().ok_or("BAD")?;
            Ok(None)
        }
        "STATUS" => super::status::execute(account, parts, output),
        "LIST" | "LSUB" if parts.len() == 4 => {
            if parts[3].is_empty() {
                output
                    .extend_from_slice(format!("* {verb} (\\Noselect) \"/\" \"\"\r\n").as_bytes());
                return Ok(None);
            }
            let pattern = format!("{}{}", parts[2], parts[3]);
            for mailbox in &account.mailboxes {
                let path = account.mailbox_path(&mailbox.id).ok_or("NO")?;
                if (verb != "LSUB" || mailbox.is_subscribed)
                    && super::folders::matches(&pattern, &path)
                {
                    let children = if account
                        .mailboxes
                        .iter()
                        .any(|m| m.parent_id.as_deref() == Some(&mailbox.id))
                    {
                        "\\HasChildren"
                    } else {
                        "\\HasNoChildren"
                    };
                    output.extend_from_slice(
                        format!(
                            "* {verb} ({children}) \"/\" {}\r\n",
                            quote(&path, output.utf8)
                        )
                        .as_bytes(),
                    );
                }
            }
            Ok(None)
        }
        "SELECT" | "EXAMINE" => {
            super::select::execute(service, token, account, parts, selected, output)
        }
        "CREATE" if parts.len() == 3 => service
            .execute(
                token,
                &account.id,
                account.revision,
                vec![Command::SetMailbox {
                    id: None,
                    properties: super::folders::properties(account, &parts[2], None)?,
                }],
            )
            .and_then(|updated| {
                let folder = super::folders::find(&updated, parts[2].trim_end_matches('/'))
                    .ok_or(mail_kernel::Error::NotFound)?;
                Ok(output
                    .objectid
                    .then(|| format!("[OBJECTID {}]", super::objectid::compound(&updated, folder))))
            })
            .map_err(|_| "NO"),
        "DELETE" | "RENAME" if parts.len() == if verb == "DELETE" { 3 } else { 4 } => {
            let mailbox = super::folders::find(account, &parts[2]).ok_or("NO")?;
            let command = if verb == "DELETE" {
                Command::RemoveMailbox {
                    id: mailbox.id.clone(),
                    remove_emails: true,
                }
            } else {
                Command::SetMailbox {
                    id: Some(mailbox.id.clone()),
                    properties: super::folders::properties(account, &parts[3], Some(mailbox))?,
                }
            };
            service
                .execute(token, &account.id, account.revision, vec![command])
                .map(|_| {
                    (output.objectid && verb == "RENAME").then(|| {
                        format!("[OBJECTID {}]", super::objectid::compound(account, mailbox))
                    })
                })
                .map_err(|_| "NO")
        }
        "SUBSCRIBE" | "UNSUBSCRIBE" if parts.len() == 3 => {
            let mailbox = super::folders::find(account, &parts[2]).ok_or("NO")?;
            let mut properties = mailbox.properties();
            properties.is_subscribed = verb == "SUBSCRIBE";
            service
                .execute(
                    token,
                    &account.id,
                    account.revision,
                    vec![Command::SetMailbox {
                        id: Some(mailbox.id.clone()),
                        properties,
                    }],
                )
                .map(|_| None)
                .map_err(|_| "NO")
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
