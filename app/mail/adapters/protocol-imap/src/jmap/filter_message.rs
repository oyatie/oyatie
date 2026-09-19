use mail_kernel::MessageState;
use mail_parser::{Address, HeaderForm, HeaderValue, Message, MessageParser};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn thread_id(state: &MessageState) -> &str {
    state.thread.as_deref().unwrap_or(&state.id)
}

/// Keywords carried by any (`some`) and by every (`all`) message of a thread.
#[derive(Default)]
pub(super) struct ThreadKeywords {
    some: BTreeMap<String, BTreeSet<String>>,
    all: BTreeMap<String, BTreeSet<String>>,
}
impl ThreadKeywords {
    pub fn build<'a>(states: impl IntoIterator<Item = &'a MessageState>) -> Self {
        let mut threads = Self::default();
        for state in states {
            let keywords: BTreeSet<_> = state.keywords.iter().cloned().collect();
            let thread = thread_id(state).to_owned();
            threads
                .some
                .entry(thread.clone())
                .or_default()
                .extend(keywords.iter().cloned());
            threads
                .all
                .entry(thread)
                .and_modify(|all| all.retain(|k| keywords.contains(k)))
                .or_insert(keywords);
        }
        threads
    }
    pub fn some(&self, thread: &str, keyword: &str) -> bool {
        self.some.get(thread).is_some_and(|k| k.contains(keyword))
    }
    pub fn all(&self, thread: &str, keyword: &str) -> bool {
        self.all.get(thread).is_some_and(|k| k.contains(keyword))
    }
}

/// One message as seen by filter and sort evaluation. `size` and `content`
/// are absent when only stored metadata is available (Email/queryChanges).
#[derive(Clone, Copy)]
pub(super) struct Candidate<'a> {
    pub state: &'a MessageState,
    pub size: Option<usize>,
    pub content: Option<&'a Content>,
    pub threads: Option<&'a ThreadKeywords>,
}
impl Candidate<'_> {
    pub fn thread_id(&self) -> &str {
        thread_id(self.state)
    }
}

/// Search text and sort keys derived from one parsed message. Search fields
/// are lower-cased once; sort keys keep the exact strings Email/get renders.
pub(super) struct Content {
    subject: String,
    body: String,
    from: String,
    to: String,
    cc: String,
    bcc: String,
    /// Lower-cased (name, decoded value) for every top-level header.
    pub headers: Vec<(String, String)>,
    pub has_attachment: bool,
    pub sort_subject: String,
    pub sort_from: String,
    pub sort_to: String,
    pub sent_at: Option<i64>,
}
impl Content {
    pub fn parse(raw: &[u8]) -> Result<Self, &'static str> {
        let parsed = parse(raw)?;
        let (subject, body) = subject_body(&parsed);
        let [from, to, cc, bcc] =
            [parsed.from(), parsed.to(), parsed.cc(), parsed.bcc()].map(address_text);
        let names: BTreeMap<String, &str> = parsed
            .headers()
            .iter()
            .map(|h| (h.name().to_lowercase(), h.name()))
            .collect();
        let mut headers = Vec::new();
        for (lower, name) in names {
            for value in parsed.header_as(name, HeaderForm::Text) {
                let text = match value {
                    HeaderValue::Text(text) => text.to_lowercase(),
                    _ => String::new(),
                };
                headers.push((lower.clone(), text));
            }
        }
        Ok(Self {
            sort_subject: subject.clone(),
            subject: subject.to_lowercase(),
            body: body.to_lowercase(),
            sort_from: display(parsed.from()),
            sort_to: display(parsed.to()),
            from: from.to_lowercase(),
            to: to.to_lowercase(),
            cc: cc.to_lowercase(),
            bcc: bcc.to_lowercase(),
            headers,
            has_attachment: parsed.attachment_count() > 0,
            sent_at: parsed.date().map(mail_parser::DateTime::to_timestamp),
        })
    }
    /// Case-insensitive substring match of a lower-cased `term` within the
    /// RFC 8621 `text`, `subject`, `body`, `from`, `to`, `cc` or `bcc` field.
    pub fn matches(&self, property: &str, term: &str) -> bool {
        match property {
            "subject" => self.subject.contains(term),
            "body" => self.body.contains(term),
            "from" => self.from.contains(term),
            "to" => self.to.contains(term),
            "cc" => self.cc.contains(term),
            "bcc" => self.bcc.contains(term),
            _ => [
                &self.subject,
                &self.body,
                &self.from,
                &self.to,
                &self.cc,
                &self.bcc,
            ]
            .iter()
            .any(|field| field.contains(term)),
        }
    }
}

fn parse(raw: &[u8]) -> Result<Message<'_>, &'static str> {
    let parsed = MessageParser::default().parse(raw).ok_or("serverFail")?;
    if parsed.parts.len() > 1000 {
        return Err("tooLarge");
    }
    Ok(parsed)
}

fn subject_body(parsed: &Message<'_>) -> (String, String) {
    let subject = parsed.subject().unwrap_or_default().to_owned();
    let mut body = String::new();
    for index in 0..parsed.text_body.len().max(parsed.html_body.len()) {
        if let Some(part) = parsed.body_text(index) {
            body.push_str(&part);
            body.push('\n');
        }
    }
    (subject, body)
}

fn address_text(list: Option<&Address<'_>>) -> String {
    let mut text = String::new();
    for address in list.into_iter().flat_map(Address::iter) {
        for value in [address.name.as_deref(), address.address.as_deref()]
            .into_iter()
            .flatten()
        {
            text.push_str(value);
            text.push(' ');
        }
    }
    text
}

/// The display name of the first address when present, else its address:
/// the value clients show for a `from`/`to` column (RFC 8621 §4.4.2).
fn display(list: Option<&Address<'_>>) -> String {
    let Some(first) = list.and_then(Address::first) else {
        return String::new();
    };
    match first.name.as_deref() {
        Some(name) if !name.is_empty() => name.to_owned(),
        _ => first.address.as_deref().unwrap_or("").to_owned(),
    }
}

/// Original-case subject, body text and address text for snippet highlighting.
pub(super) fn text(raw: &[u8]) -> Result<(String, String, String), &'static str> {
    let parsed = parse(raw)?;
    let (subject, body) = subject_body(&parsed);
    let addresses = [parsed.from(), parsed.to(), parsed.cc(), parsed.bcc()]
        .map(address_text)
        .concat();
    Ok((subject, body, addresses))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_indexes_addresses_headers_attachments_and_sort_keys() {
        let raw = b"From: Alice Sender <alice@example.com>\r\nTo: bob@example.org\r\nCc: =?UTF-8?B?Q2FmZQ==?= <cafe@example.net>\r\nX-Custom-Header: Custom-Value-1\r\nSubject: Re: Hello\r\nDate: Thu, 01 Jan 2026 00:00:00 +0000\r\nContent-Type: multipart/mixed; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nBody text\r\n--b\r\nContent-Type: application/pdf\r\nContent-Disposition: attachment; filename=\"a.pdf\"\r\n\r\n%PDF\r\n--b--\r\n";
        let content = Content::parse(raw).unwrap();
        assert!(content.matches("from", "alice sender"));
        assert!(content.matches("text", "alice@example.com"));
        assert!(content.matches("cc", "cafe"));
        assert!(!content.matches("to", "alice"));
        assert!(content.matches("body", "body text"));
        assert!(
            content
                .headers
                .contains(&("x-custom-header".into(), "custom-value-1".into()))
        );
        assert!(content.has_attachment);
        assert_eq!(content.sort_from, "Alice Sender");
        assert_eq!(content.sort_to, "bob@example.org");
        assert_eq!(content.sort_subject, "Re: Hello");
        assert_eq!(content.sent_at, Some(1_767_225_600));
    }
    #[test]
    fn thread_keywords_distinguish_some_from_all() {
        let state = |id: &str, keywords: &[&str]| MessageState {
            id: id.into(),
            modseq: 1,
            uids: Default::default(),
            thread: Some("t".into()),
            mailboxes: vec![],
            keywords: keywords.iter().map(|k| (*k).to_owned()).collect(),
            received_at: 0,
        };
        let states = [state("a", &["$seen", "$flagged"]), state("b", &["$seen"])];
        let threads = ThreadKeywords::build(&states);
        assert!(threads.some("t", "$flagged"));
        assert!(threads.all("t", "$seen"));
        assert!(!threads.all("t", "$flagged"));
        assert!(!threads.some("other", "$seen"));
    }
}
