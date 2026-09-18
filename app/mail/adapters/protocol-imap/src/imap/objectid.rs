use super::{response::Output, syntax::Token};
use mail_kernel::{Account, Mailbox};

pub(super) fn object_id(kind: &str, account: &str, id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update((account.len() as u64).to_be_bytes());
    hash.update(account.as_bytes());
    hash.update(id.as_bytes());
    format!("{kind}{:x}", hash.finalize())
}

#[derive(Default)]
pub(super) struct Identifiers {
    account: Option<String>,
    mailbox: Option<String>,
}

pub(super) fn activate(output: &mut Output) {
    if !output.objectid {
        output.objectid = true;
        output.extend_from_slice(b"* ENABLED OBJECTID+\r\n");
    }
}

pub(super) fn compound(account: &Account, mailbox: &Mailbox) -> String {
    format!(
        "(ACCOUNTID {} MAILBOXID {})",
        account.id,
        object_id("F", &account.id, &mailbox.id)
    )
}

impl Identifiers {
    pub fn find<'a>(&self, account: &'a Account) -> Option<&'a Mailbox> {
        // Resolve only within the authenticated, policy-authorized snapshot.
        // An unknown or foreign identifier falls back to that account's name.
        if self.account.as_deref() != Some(account.id.as_str()) {
            return None;
        }
        let id = self.mailbox.as_deref()?;
        account
            .mailboxes
            .iter()
            .find(|m| object_id("F", &account.id, &m.id) == id)
    }
}

// Strip only top-level OBJECTID options, leaving CONDSTORE/QRESYNC parsing
// unchanged. The shared tokenizer bounds this non-recursive pass to 1024 tokens.
pub(super) fn options(
    tokens: Vec<Token>,
) -> Result<(Vec<Token>, Option<Identifiers>), &'static str> {
    if tokens.is_empty() {
        return Ok((tokens, None));
    }
    if !matches!(tokens.first(), Some(Token::Open)) || !matches!(tokens.last(), Some(Token::Close))
    {
        return Err("BAD");
    }
    let mut tokens = tokens.into_iter().peekable();
    let mut retained = Vec::new();
    let mut depth = 0_usize;
    let mut identifiers = None;
    while let Some(token) = tokens.next() {
        if depth == 1 && matches!(&token, Token::Word(w) if w.eq_ignore_ascii_case("OBJECTID")) {
            let mut parsed = Identifiers::default();
            if matches!(tokens.peek(), Some(Token::Open)) {
                tokens.next();
                loop {
                    let key = match tokens.next() {
                        Some(Token::Word(key)) => key,
                        Some(Token::Close) => break,
                        _ => return Err("BAD"),
                    };
                    let value = match tokens.next() {
                        Some(Token::Word(value) | Token::Quoted(value)) => value,
                        _ => return Err("BAD"),
                    };
                    if key.eq_ignore_ascii_case("ACCOUNTID") {
                        parsed.account = Some(value);
                    } else if key.eq_ignore_ascii_case("MAILBOXID") {
                        parsed.mailbox = Some(value);
                    }
                }
            }
            identifiers = Some(parsed);
            continue;
        }
        match token {
            Token::Open => depth += 1,
            Token::Close => depth = depth.checked_sub(1).ok_or("BAD")?,
            _ => {}
        }
        if depth == 0 && tokens.peek().is_some() {
            return Err("BAD");
        }
        retained.push(token);
    }
    if depth != 0 {
        return Err("BAD");
    }
    Ok((retained, identifiers))
}
