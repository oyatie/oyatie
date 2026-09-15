// RFC 5256 section 2.1. The MIME parser's translated reply prefixes are
// intentionally broader than this protocol's exact base-subject grammar.
pub(super) fn base(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let mut space = false;
    for c in input.chars() {
        if matches!(c, ' ' | '\t' | '\r' | '\n') {
            space = !normalized.is_empty();
        } else {
            if space {
                normalized.push(' ');
            }
            space = false;
            normalized.push(c);
        }
    }
    let mut value = normalized.as_str();
    loop {
        value = value.trim_end_matches(' ');
        while value
            .get(value.len().saturating_sub(5)..)
            .is_some_and(|v| v.eq_ignore_ascii_case("(fwd)"))
        {
            value = value[..value.len() - 5].trim_end_matches(' ');
        }
        loop {
            let before = value;
            value = value.trim_start_matches(' ');
            let mut tail = value;
            let mut last_blob = tail;
            while let Some(rest) = blob(tail) {
                last_blob = tail;
                tail = rest;
            }
            if let Some(rest) = refwd(tail) {
                value = rest;
            } else {
                value = if tail.is_empty() { last_blob } else { tail };
            }
            if value == before {
                break;
            }
        }
        if value
            .get(..5)
            .is_some_and(|v| v.eq_ignore_ascii_case("[fwd:"))
            && value.ends_with(']')
        {
            value = &value[5..value.len() - 1];
        } else {
            return value.to_owned();
        }
    }
}

fn blob(value: &str) -> Option<&str> {
    let value = value.strip_prefix('[')?;
    let end = value.find(['[', ']'])?;
    (value.as_bytes()[end] == b']').then(|| value[end + 1..].trim_start_matches(' '))
}

fn refwd(value: &str) -> Option<&str> {
    let prefix = ["re", "fwd", "fw"].into_iter().find(|p| {
        value
            .get(..p.len())
            .is_some_and(|v| v.eq_ignore_ascii_case(p))
    })?;
    let mut rest = value[prefix.len()..].trim_start_matches(' ');
    if let Some(tail) = blob(rest) {
        rest = tail;
    }
    rest.strip_prefix(':')
}
