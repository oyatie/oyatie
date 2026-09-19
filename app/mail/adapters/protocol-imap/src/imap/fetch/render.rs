//! Rendering entry points over a raw message, used by the conformance harness
//! to compare BODY, BODYSTRUCTURE and section output against golden files.
//! They run the same parser and writers as the FETCH command.
use super::{items, section, structure};
use crate::imap::response::Output;

fn parsed(raw: &[u8]) -> Result<mail_parser::Message<'_>, &'static str> {
    mail_parser::MessageParser::default().parse(raw).ok_or("NO")
}

/// `BODY` (`extended == false`) or `BODYSTRUCTURE` for the whole message,
/// with `utf8` selecting UTF8=ACCEPT string quoting.
pub fn structure(raw: &[u8], extended: bool, utf8: bool) -> Result<Vec<u8>, &'static str> {
    let message = parsed(raw)?;
    structure::validate(&message)?;
    let (send, mut receive) = tokio::sync::mpsc::channel(1024);
    let mut output = Output::new(send);
    output.utf8 = utf8;
    structure::write(&message, 0, extended, &mut output);
    output.finish().map_err(|_| "NO")?;
    drop(output);
    let mut bytes = Vec::new();
    while let Ok(chunk) = receive.try_recv() {
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

/// Content of one `BODY[...]`, `BINARY[...]` or `BINARY.SIZE[...]` item as
/// written on the FETCH command line, e.g. `BODY[1.2.HEADER]<10.25>`.
/// `Ok(None)` is the NIL response for a section that does not exist.
pub fn section(raw: &[u8], item: &str) -> Result<Option<Vec<u8>>, &'static str> {
    let mut parsed_items = items::parse(item)?;
    let Some(items::Item::Content(request)) = parsed_items.pop() else {
        return Err("BAD");
    };
    if !parsed_items.is_empty() {
        return Err("BAD");
    }
    let message = parsed(raw).ok();
    Ok(section::read(&request, message.as_ref(), raw)?.map(|value| value.into_owned()))
}
