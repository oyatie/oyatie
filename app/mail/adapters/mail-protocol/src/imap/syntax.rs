use mail_kernel::Message;

#[derive(Debug)]
pub(super) enum Token {
    Word(String),
    Quoted(String),
    Open,
    Close,
}

pub(super) fn tokens(value: &str) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ' ' {
            continue;
        }
        if tokens.len() == 1024 || c.is_control() {
            return None;
        }
        tokens.push(match c {
            '(' => Token::Open,
            ')' => Token::Close,
            '{' => {
                let mut marker = String::from("{");
                loop {
                    let c = chars.next()?;
                    marker.push(c);
                    if c == '}' {
                        break;
                    }
                    if marker.len() > 24 {
                        return None;
                    }
                }
                let (size, _) = super::literal::specifier(marker.as_bytes())?;
                if chars.next()? != '\r' || chars.next()? != '\n' {
                    return None;
                }
                let mut text = String::new();
                while text.len() < size {
                    let c = chars.next()?;
                    if c == '\0' {
                        return None;
                    }
                    text.push(c);
                }
                if text.len() != size || chars.peek().is_some_and(|c| !matches!(c, ' ' | ')')) {
                    return None;
                }
                Token::Quoted(text)
            }
            '"' => {
                let mut text = String::new();
                loop {
                    match chars.next()? {
                        '"' => break,
                        '\\' => {
                            let c = chars.next()?;
                            if !matches!(c, '\\' | '"') {
                                return None;
                            }
                            text.push(c);
                        }
                        c if !c.is_control() => text.push(c),
                        _ => return None,
                    }
                }
                if chars.peek().is_some_and(|c| !matches!(c, ' ' | ')')) {
                    return None;
                }
                Token::Quoted(text)
            }
            c => {
                let mut text = String::from(c);
                while let Some(&c) = chars.peek() {
                    if matches!(c, ' ' | '(' | ')') {
                        break;
                    }
                    text.push(c);
                    chars.next();
                }
                if text
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '{' | '"' | '\\'))
                {
                    return None;
                }
                Token::Word(text)
            }
        });
    }
    Some(tokens)
}

pub(super) fn command_parts(value: &str) -> Option<Vec<String>> {
    // FETCH owns its nested list/astring grammar. Preserve its argument bytes;
    // the shared flat atom parser is for commands such as LOGIN and CREATE.
    let mut remaining = value;
    let mut prefix = Vec::new();
    for _ in 0..3 {
        remaining = remaining.trim_start_matches(' ');
        let (word, tail) = remaining.split_once(' ').unwrap_or((remaining, ""));
        prefix.push(word.to_owned());
        remaining = tail;
        if prefix.len() == 2 && word.eq_ignore_ascii_case("AUTHENTICATE") {
            for token in tokens(remaining)? {
                let Token::Word(value) = token else {
                    return None;
                };
                prefix.push(value);
            }
            return Some(prefix);
        }
        if prefix.len() == 2
            && matches!(
                word.to_ascii_uppercase().as_str(),
                "STATUS" | "SELECT" | "EXAMINE"
            )
        {
            let (mailbox, arguments) = astring_prefix(remaining.trim_start_matches(' '))?;
            prefix.push(mailbox);
            prefix.push(arguments.to_owned());
            return Some(prefix);
        }
        let nested = word.eq_ignore_ascii_case("FETCH")
            || word.eq_ignore_ascii_case("SEARCH")
            || word.eq_ignore_ascii_case("SORT")
            || word.eq_ignore_ascii_case("THREAD");
        let fetch = prefix.len() == 2 && nested
            || prefix.len() == 3 && prefix[1].eq_ignore_ascii_case("UID") && nested;
        if fetch {
            remaining = remaining.trim_start_matches(' ');
            if word.eq_ignore_ascii_case("SEARCH")
                || word.eq_ignore_ascii_case("SORT")
                || word.eq_ignore_ascii_case("THREAD")
            {
                prefix.push(remaining.to_owned());
                return Some(prefix);
            }
            let (set, args) = remaining.split_once(' ').unwrap_or((remaining, ""));
            prefix.push(set.to_owned());
            prefix.push(args.to_owned());
            return Some(prefix);
        }
    }
    let append = prefix
        .get(1)
        .is_some_and(|verb| verb.eq_ignore_ascii_case("APPEND"));
    let mut remaining = value;
    let mut parts = Vec::new();
    while !remaining.is_empty() {
        remaining = remaining.trim_start_matches(' ');
        if remaining.is_empty() {
            break;
        }
        // Only APPEND may leave its terminal body marker for the body reader.
        if append
            && parts.len() >= 3
            && !remaining.contains(' ')
            && (remaining.starts_with('{')
                || remaining.starts_with("(~{")
                || remaining.starts_with("({")
                || remaining.starts_with("~{"))
            && remaining.ends_with('}')
        {
            parts.push(remaining.to_owned());
            break;
        }
        let (value, rest) = astring_prefix(remaining)?;
        parts.push(value);
        remaining = rest;
        if parts.len() > 1024 {
            return None;
        }
    }
    Some(parts)
}

fn astring_prefix(input: &str) -> Option<(String, &str)> {
    if input.starts_with('{') {
        let marker_end = input.find("}\r\n")? + 1;
        let (size, _) = super::literal::specifier(&input.as_bytes()[..marker_end])?;
        let start = marker_end + 2;
        let end = start.checked_add(size)?;
        let value = input.get(start..end)?;
        let rest = input.get(end..)?;
        if value.contains('\0') || (!rest.is_empty() && !rest.starts_with(' ')) {
            return None;
        }
        return Some((value.to_owned(), rest.trim_start_matches(' ')));
    }
    let end = if input.starts_with('"') {
        let mut escaped = false;
        input.char_indices().skip(1).find_map(|(i, c)| {
            if escaped {
                escaped = false;
                None
            } else if c == '\\' {
                escaped = true;
                None
            } else if c == '"' {
                Some(i + 1)
            } else {
                None
            }
        })?
    } else {
        input.find(' ').unwrap_or(input.len())
    };
    let (value, rest) = input.split_at(end);
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    if !value.starts_with('"') && value.contains('{') {
        return None;
    }
    let value = crate::wire::atoms(value)?.pop()?;
    Some((value, rest.trim_start_matches(' ')))
}

pub(super) fn sequence_set(input: &str, largest: u32) -> Option<Vec<(u32, u32)>> {
    input
        .split(',')
        .map(|part| {
            let number = |s: &str| {
                if s == "*" {
                    Some(largest)
                } else {
                    (!s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                        .then(|| s.parse::<u32>().ok())
                        .flatten()
                }
            };
            let (left, right) = part.split_once(':').unwrap_or((part, part));
            let a = number(left)?;
            let b = number(right)?;
            Some((a.min(b), a.max(b)))
        })
        .collect()
}

pub(super) fn keyword(flag: &str) -> Option<String> {
    Some(
        match flag.to_ascii_lowercase().as_str() {
            "\\seen" => "$seen",
            "\\answered" => "$answered",
            "\\flagged" => "$flagged",
            "\\deleted" => "$deleted",
            "\\draft" => "$draft",
            "\\junk" => "$junk",
            "\\notjunk" => "$notjunk",
            _ if !flag.starts_with('\\') => flag,
            _ => return None,
        }
        .to_owned(),
    )
}

pub(super) fn flags(message: &Message) -> String {
    message
        .keywords
        .iter()
        .map(|k| match k.as_str() {
            "$seen" => "\\Seen",
            "$answered" => "\\Answered",
            "$flagged" => "\\Flagged",
            "$deleted" => "\\Deleted",
            "$draft" => "\\Draft",
            _ => k,
        })
        .collect::<Vec<_>>()
        .join(" ")
}
