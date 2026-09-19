//! LIST (RFC 3501, LIST-EXTENDED RFC 5258, LIST-STATUS RFC 5819, SPECIAL-USE
//! RFC 6154, CHILDREN RFC 3348) and the IMAP4rev1 LSUB command.
use super::{
    folders,
    mailboxes::{decode, quote},
    response::Output,
    status,
    syntax::{Token, tokens},
};
use mail_kernel::{Account, Mailbox};

#[derive(Default)]
struct Request {
    subscribed: bool,
    special_use: bool,
    recursive: bool,
    return_subscribed: bool,
    status: Option<Vec<String>>,
    patterns: Vec<String>,
}

pub(super) fn execute(
    account: &Account,
    parts: &[String],
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    let verb = parts[1].to_ascii_uppercase();
    let tokens = tokens(&parts[2]).ok_or("BAD")?;
    let mut request = parse(&tokens, output.utf8)?;
    if verb == "LSUB" {
        if request.subscribed || request.special_use || request.status.is_some() {
            return Err("BAD");
        }
        request.subscribed = true;
    }
    if request
        .status
        .as_ref()
        .is_some_and(|items| items.iter().any(|i| i == "OBJECTID"))
    {
        super::objectid::activate(output);
    }
    // RFC 3501 §6.3.8: an empty name asks for the hierarchy root.
    if request.patterns.iter().all(String::is_empty) {
        output.extend_from_slice(format!("* {verb} (\\Noselect) \"/\" \"\"\r\n").as_bytes());
        return Ok(None);
    }
    for mailbox in &account.mailboxes {
        let path = account.mailbox_path(&mailbox.id).ok_or("NO")?;
        if !request.patterns.iter().any(|p| folders::matches(p, &path)) {
            continue;
        }
        let selected = selects(&request, mailbox);
        let mut childinfo = Vec::new();
        if request.recursive {
            let descendants: Vec<_> = account
                .mailboxes
                .iter()
                .filter(|m| descends(account, m, &mailbox.id))
                .collect();
            if request.subscribed && descendants.iter().any(|m| m.is_subscribed) {
                childinfo.push("\"SUBSCRIBED\"");
            }
            if request.special_use && descendants.iter().any(|m| folders::attribute(m).is_some()) {
                childinfo.push("\"SPECIAL-USE\"");
            }
        }
        if !selected && childinfo.is_empty() {
            continue;
        }
        let mut attributes = vec![if account
            .mailboxes
            .iter()
            .any(|m| m.parent_id.as_deref() == Some(&mailbox.id))
        {
            "\\HasChildren"
        } else {
            "\\HasNoChildren"
        }];
        if (request.subscribed || request.return_subscribed) && mailbox.is_subscribed {
            attributes.push("\\Subscribed");
        }
        attributes.extend(folders::attribute(mailbox));
        let extended = if childinfo.is_empty() {
            String::new()
        } else {
            format!(" (\"CHILDINFO\" ({}))", childinfo.join(" "))
        };
        output.extend_from_slice(
            format!(
                "* {verb} ({}) \"/\" {}{extended}\r\n",
                attributes.join(" "),
                quote(&path, output.utf8)
            )
            .as_bytes(),
        );
        if let Some(items) = &request.status
            && selected
        {
            status::report(account, mailbox, items, output);
        }
    }
    Ok(None)
}

fn selects(request: &Request, mailbox: &Mailbox) -> bool {
    (!request.subscribed && !request.special_use)
        || (request.subscribed && mailbox.is_subscribed)
        || (request.special_use && folders::attribute(mailbox).is_some())
}

fn descends(account: &Account, mailbox: &Mailbox, ancestor: &str) -> bool {
    let mut parent = mailbox.parent_id.as_deref();
    let mut depth = 0;
    while let Some(id) = parent {
        if id == ancestor {
            return true;
        }
        depth += 1;
        if depth > account.mailboxes.len() {
            return false;
        }
        parent = account
            .mailboxes
            .iter()
            .find(|m| m.id == id)
            .and_then(|m| m.parent_id.as_deref());
    }
    false
}

fn name(token: &Token, utf8: bool) -> Result<String, &'static str> {
    let (Token::Word(value) | Token::Quoted(value)) = token else {
        return Err("BAD");
    };
    if utf8 {
        Ok(value.clone())
    } else {
        decode(value).ok_or("BAD")
    }
}

/// Splits off a parenthesised group that starts at `tokens[0]`.
fn group(tokens: &[Token]) -> Result<(&[Token], &[Token]), &'static str> {
    let end = tokens
        .iter()
        .position(|t| matches!(t, Token::Close))
        .ok_or("BAD")?;
    Ok((&tokens[1..end], &tokens[end + 1..]))
}

fn parse(tokens: &[Token], utf8: bool) -> Result<Request, &'static str> {
    let mut request = Request::default();
    let mut rest = tokens;
    if let Some(Token::Open) = rest.first() {
        let (options, tail) = group(rest)?;
        for option in options {
            let Token::Word(word) = option else {
                return Err("BAD");
            };
            match word.to_ascii_uppercase().as_str() {
                "SUBSCRIBED" => request.subscribed = true,
                "SPECIAL-USE" => request.special_use = true,
                "RECURSIVEMATCH" => request.recursive = true,
                "REMOTE" => {}
                _ => return Err("BAD"),
            }
        }
        rest = tail;
    }
    if request.recursive && !(request.subscribed || request.special_use) {
        return Err("BAD");
    }
    let reference = name(rest.first().ok_or("BAD")?, utf8)?;
    rest = &rest[1..];
    let patterns: &[Token] = match rest.first().ok_or("BAD")? {
        Token::Open => {
            let (patterns, tail) = group(rest)?;
            rest = tail;
            patterns
        }
        _ => {
            let (pattern, tail) = rest.split_at(1);
            rest = tail;
            pattern
        }
    };
    for pattern in patterns {
        request
            .patterns
            .push(format!("{reference}{}", name(pattern, utf8)?));
    }
    if request.patterns.is_empty() {
        return Err("BAD");
    }
    if rest.is_empty() {
        return Ok(request);
    }
    let [Token::Word(word), Token::Open, options @ ..] = rest else {
        return Err("BAD");
    };
    if !word.eq_ignore_ascii_case("RETURN") {
        return Err("BAD");
    }
    let mut options: &[Token] = options;
    loop {
        match options {
            [Token::Close] => return Ok(request),
            [Token::Word(option), tail @ ..] => {
                options = tail;
                match option.to_ascii_uppercase().as_str() {
                    "SUBSCRIBED" => request.return_subscribed = true,
                    "CHILDREN" | "SPECIAL-USE" => {}
                    "STATUS" if matches!(options.first(), Some(Token::Open)) => {
                        let (items, tail) = group(options)?;
                        request.status = Some(status::items(items)?);
                        options = tail;
                    }
                    _ => return Err("BAD"),
                }
            }
            _ => return Err("BAD"),
        }
    }
}
