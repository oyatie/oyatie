mod envelope;
mod items;
mod section;
mod structure;
use super::{response::Output, state::Selection, syntax::flags};
use items::Item;
use mail_kernel::{Account, Command, Message};
use mail_service::MailService;

pub(super) fn execute(
    service: &MailService,
    token: &str,
    account: &Account,
    selected: &mut Selection,
    chosen: Vec<(usize, &Message)>,
    parts: &[String],
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    let uid = parts[1].eq_ignore_ascii_case("UID");
    let offset = if uid { 3 } else { 2 };
    let input = parts.get(offset + 1).ok_or("BAD")?;
    let (args, since, vanished) = super::condstore::fetch_args(input)?;
    if vanished && (!uid || !selected.qresync) {
        return Err("BAD");
    }
    let mut items = items::parse(args)?;
    if items.iter().any(|i| matches!(i, Item::ObjectId)) {
        super::objectid::activate(output);
    }
    if since.is_some() || items.iter().any(|i| matches!(i, Item::Modseq)) {
        selected.condstore = true;
        output.condstore = true;
    }
    if since.is_some() && !items.iter().any(|i| matches!(i, Item::Modseq)) {
        items.push(Item::Modseq);
    }
    let chosen: Vec<_> = chosen
        .into_iter()
        .filter(|(_, m)| since.is_none_or(|s| m.modseq > s))
        .collect();
    let earlier = if vanished {
        super::condstore::vanished(
            service,
            token,
            account,
            &selected.mailbox,
            since.ok_or("BAD")?,
        )?
    } else {
        vec![]
    };
    let set_seen = !selected.readonly
        && items
            .iter()
            .any(|item| matches!(item, Item::Content(c) if !c.peek));
    let changes: Vec<_> = chosen
        .iter()
        .filter(|(_, m)| set_seen && !m.keywords.iter().any(|k| k == "$seen"))
        .map(|(_, m)| {
            let mut keywords = m.keywords.clone();
            keywords.push("$seen".into());
            Command::Keywords {
                id: m.id.clone(),
                keywords,
            }
        })
        .collect();
    let updated = if changes.is_empty() {
        None
    } else {
        Some(
            service
                .execute(token, &account.id, account.revision, changes)
                .map_err(|_| "NO")?,
        )
    };
    let updated = updated.as_ref().unwrap_or(account);
    if !earlier.is_empty() {
        output.extend_from_slice(
            format!(
                "* VANISHED (EARLIER) {}\r\n",
                super::condstore::ranges(earlier)
            )
            .as_bytes(),
        );
    }
    for (sequence, message) in chosen {
        if output.is_closed() {
            return Err("NO");
        }
        let raw = if items.iter().any(|i| {
            matches!(
                i,
                Item::Content(_) | Item::Envelope | Item::Structure { .. } | Item::Preview { .. }
            )
        }) {
            Some(
                service
                    .download(token, &account.id, &message.id)
                    .map_err(|_| "NO")?,
            )
        } else {
            None
        };
        let needs_parsed = items.iter().any(|i| matches!(i, Item::Envelope | Item::Structure { .. } | Item::Preview { .. }) || matches!(i, Item::Content(c) if c.binary || !matches!(c.section, items::Section::Whole)));
        let parsed = raw.as_ref().filter(|_| needs_parsed).and_then(|r| {
            mail_parser::MessageParser::default().parse(r).or_else(|| {
                r.is_empty().then(|| mail_parser::Message {
                    parts: vec![mail_parser::MessagePart::default()],
                    ..Default::default()
                })
            })
        });
        if items.iter().any(|i| matches!(i, Item::Structure { .. })) {
            structure::validate(parsed.as_ref().ok_or("NO")?)?;
        }
        let current = updated
            .messages
            .iter()
            .find(|m| m.id == message.id)
            .ok_or("NO")?;
        output.fetch_start(sequence + 1, message.uid_in(&selected.mailbox).ok_or("NO")?);
        if current.keywords != message.keywords && !items.iter().any(|i| matches!(i, Item::Flags)) {
            output.extend_from_slice(format!(" FLAGS ({})", flags(current)).as_bytes());
        }
        for item in &items {
            if output.is_closed() {
                return Err("NO");
            }
            let result = render(
                item,
                current,
                &account.id,
                raw.as_deref(),
                parsed.as_ref(),
                output,
            );
            if let Err(error) = result {
                output.extend_from_slice(b")\r\n");
                return Err(error);
            }
        }
        output.extend_from_slice(b")\r\n");
    }
    selected.modseq = updated.mail_modseq;
    Ok(None)
}

fn render(
    item: &Item,
    message: &Message,
    account: &str,
    raw: Option<&[u8]>,
    parsed: Option<&mail_parser::Message<'_>>,
    output: &mut Output,
) -> Result<(), &'static str> {
    match item {
        Item::Uid => {}
        Item::Modseq => {
            output.extend_from_slice(format!(" MODSEQ ({})", message.modseq).as_bytes())
        }
        Item::Preview { .. } => {
            let preview = parsed.and_then(|p| p.body_preview(256)).unwrap_or_default();
            output.extend_from_slice(format!(" PREVIEW {{{}}}\r\n", preview.len()).as_bytes());
            output.extend_from_slice(preview.as_bytes());
        }
        Item::EmailId | Item::ThreadId | Item::ObjectId => {
            let email = super::object_id("M", account, message.email_identity());
            let thread = super::object_id("T", account, message.thread_identity());
            match item {
                Item::EmailId => output.extend_from_slice(format!(" EMAILID ({email})").as_bytes()),
                Item::ThreadId => {
                    output.extend_from_slice(format!(" THREADID ({thread})").as_bytes())
                }
                _ => output.extend_from_slice(
                    format!(" OBJECTID (EMAILID {email} THREADID {thread})").as_bytes(),
                ),
            }
        }
        Item::Flags => output.extend_from_slice(format!(" FLAGS ({})", flags(message)).as_bytes()),
        Item::Size => output.extend_from_slice(format!(" RFC822.SIZE {}", message.size).as_bytes()),
        Item::Envelope => {
            let parsed = parsed.ok_or("NO")?;
            output.extend_from_slice(b" ENVELOPE ");
            envelope::write(parsed, output);
        }
        Item::Structure { extended } => {
            let parsed = parsed.ok_or("NO")?;
            output.extend_from_slice(if *extended {
                b" BODYSTRUCTURE "
            } else {
                b" BODY "
            });
            structure::write(parsed, 0, *extended, output);
        }
        Item::Date => {
            let date = mail_parser::DateTime::from_timestamp(message.received_at);
            let month = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ][(date.month - 1) as usize];
            output.extend_from_slice(
                format!(
                    " INTERNALDATE \"{:02}-{month}-{:04} {:02}:{:02}:{:02} +0000\"",
                    date.day, date.year, date.hour, date.minute, date.second
                )
                .as_bytes(),
            );
        }
        Item::Content(request) => {
            let value = section::read(request, parsed, raw.ok_or("NO")?)?;
            match value {
                None => output.extend_from_slice(format!(" {} NIL", request.label).as_bytes()),
                Some(value) if request.size => output
                    .extend_from_slice(format!(" {} {}", request.label, value.len()).as_bytes()),
                Some(value) => {
                    let binary = if request.binary { "~" } else { "" };
                    output.extend_from_slice(
                        format!(" {} {binary}{{{}}}\r\n", request.label, value.len()).as_bytes(),
                    );
                    output.extend_from_slice(&value);
                }
            }
        }
    }
    Ok(())
}
