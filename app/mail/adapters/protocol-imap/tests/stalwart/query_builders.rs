// RFC 5322 message builders mirroring tests/src/jmap/compliance/mod.rs
// (`build_message`, `build_multipart_*`) for the Email/query seed corpus.
use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Default)]
pub struct Msg<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub cc: &'a str,
    pub bcc: &'a str,
    pub subject: &'a str,
    pub date: String,
    pub message_id: &'a str,
    pub in_reply_to: &'a str,
    pub references: &'a str,
    pub body: String,
    pub extra: Vec<&'a str>,
}

pub fn message(o: Msg) -> String {
    let mut lines = vec![format!("From: {}", o.from), format!("To: {}", o.to)];
    if !o.cc.is_empty() {
        lines.push(format!("Cc: {}", o.cc));
    }
    if !o.bcc.is_empty() {
        lines.push(format!("Bcc: {}", o.bcc));
    }
    lines.push(format!("Subject: {}", o.subject));
    lines.push(format!("Date: {}", o.date));
    lines.push(format!("Message-ID: {}", o.message_id));
    if !o.in_reply_to.is_empty() {
        lines.push(format!("In-Reply-To: {}", o.in_reply_to));
    }
    if !o.references.is_empty() {
        lines.push(format!("References: {}", o.references));
    }
    lines.push("MIME-Version: 1.0".into());
    lines.push("Content-Type: text/plain; charset=UTF-8".into());
    lines.push("Content-Transfer-Encoding: 7bit".into());
    lines.extend(o.extra.iter().map(|h| (*h).to_owned()));
    lines.push(String::new());
    lines.push(o.body);
    lines.join("\r\n")
}

pub fn multipart(kind: &str, boundary: &str, head: Vec<String>, parts: Vec<Vec<String>>) -> String {
    let mut lines = head;
    lines.push("MIME-Version: 1.0".into());
    lines.push(format!(
        "Content-Type: multipart/{kind}; boundary=\"{boundary}\""
    ));
    lines.push(String::new());
    for part in parts {
        lines.push(format!("--{boundary}"));
        lines.extend(part);
    }
    lines.push(format!("--{boundary}--"));
    lines.join("\r\n")
}

pub fn head(
    from: &str,
    to: &str,
    cc: Option<&str>,
    subject: &str,
    date: &str,
    id: &str,
) -> Vec<String> {
    let mut lines = vec![format!("From: {from}"), format!("To: {to}")];
    if let Some(cc) = cc {
        lines.push(format!("Cc: {cc}"));
    }
    lines.extend([
        format!("Subject: {subject}"),
        format!("Date: {date}"),
        format!("Message-ID: {id}"),
    ]);
    lines
}

pub fn text(kind: &str, body: &str) -> Vec<String> {
    vec![
        format!("Content-Type: text/{kind}; charset=UTF-8"),
        "Content-Transfer-Encoding: 7bit".into(),
        String::new(),
        body.into(),
    ]
}

pub const JPEG: [u8; 22] = [
    0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x00, 0x00, 0xff, 0xd9,
];

pub fn m(
    from: &str,
    to: &str,
    subject: &str,
    date: String,
    message_id: &str,
    body: &str,
) -> String {
    message(Msg {
        from,
        to,
        subject,
        date,
        message_id,
        body: body.into(),
        ..Msg::default()
    })
}
pub const ALPHA: &str = "Project Alpha Discussion";
pub const RE: &str = "Re: Project Alpha Discussion";
pub const TU: &str = "testuser@example.com";
