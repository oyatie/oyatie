#[derive(Clone)]
pub(super) enum Item {
    Uid,
    Modseq,
    Flags,
    Date,
    Size,
    Envelope,
    Preview { lazy: bool },
    EmailId,
    ThreadId,
    ObjectId,
    Structure { extended: bool },
    Content(Content),
}
#[derive(Clone)]
pub(super) struct Content {
    pub label: String,
    pub path: Vec<usize>,
    pub section: Section,
    pub partial: Option<(usize, usize)>,
    pub peek: bool,
    pub binary: bool,
    pub size: bool,
}
#[derive(Clone)]
pub(super) enum Section {
    Whole,
    Content,
    Text,
    Header,
    Mime,
    Fields { names: Vec<String>, exclude: bool },
}
pub(super) fn parse(input: &str) -> Result<Vec<Item>, &'static str> {
    let input = input.trim();
    if input.len() > 8192 {
        return Err("BAD");
    }
    match input.to_ascii_uppercase().as_str() {
        "FAST" => return Ok(vec![Item::Flags, Item::Date, Item::Size]),
        "ALL" => return Ok(vec![Item::Flags, Item::Date, Item::Size, Item::Envelope]),
        "FULL" => {
            return Ok(vec![
                Item::Flags,
                Item::Date,
                Item::Size,
                Item::Envelope,
                Item::Structure { extended: false },
            ]);
        }
        _ => {}
    }
    let list = input.starts_with('(');
    let input = if list {
        input
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .ok_or("BAD")?
    } else {
        input
    };
    let mut items = Vec::new();
    let mut start = 0;
    let mut bracket = false;
    let mut quoted = false;
    let mut escaped = false;
    let mut literal_end = 0;
    for (i, b) in input.bytes().chain(std::iter::once(b' ')).enumerate() {
        if i < literal_end {
            continue;
        }
        if b == b'{' && bracket && !quoted {
            let marker_end = i + input[i..].find("}\r\n").ok_or("BAD")? + 1;
            let (size, _) =
                super::super::literal::specifier(&input.as_bytes()[i..marker_end]).ok_or("BAD")?;
            literal_end = marker_end
                .checked_add(2)
                .and_then(|n| n.checked_add(size))
                .ok_or("BAD")?;
            if input.get(marker_end + 2..literal_end).is_none() {
                return Err("BAD");
            }
            continue;
        }
        if b.is_ascii_control() {
            return Err("BAD");
        }
        if quoted {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
            continue;
        }
        match b {
            b'"' if bracket => quoted = true,
            b'[' if !bracket => bracket = true,
            b'[' => return Err("BAD"),
            b']' if bracket => bracket = false,
            b']' => return Err("BAD"),
            b' ' if !bracket => {
                if i > start {
                    if items.len() >= 128 {
                        return Err("BAD");
                    }
                    let text = input[start..i].to_ascii_uppercase();
                    if text == "(LAZY)" {
                        if let Some(Item::Preview { lazy }) = items.last_mut()
                            && !*lazy
                        {
                            *lazy = true;
                            start = i + 1;
                            continue;
                        }
                        return Err("BAD");
                    }
                    items.push(match text.as_str() {
                        "UID" => Item::Uid,
                        "MODSEQ" => Item::Modseq,
                        "FLAGS" => Item::Flags,
                        "INTERNALDATE" => Item::Date,
                        "RFC822.SIZE" => Item::Size,
                        "ENVELOPE" => Item::Envelope,
                        "PREVIEW" => Item::Preview { lazy: false },
                        "EMAILID" => Item::EmailId,
                        "THREADID" => Item::ThreadId,
                        "OBJECTID" => Item::ObjectId,
                        "BODY" => Item::Structure { extended: false },
                        "BODYSTRUCTURE" => Item::Structure { extended: true },
                        _ => Item::Content(content(&text).ok_or("BAD")?),
                    });
                }
                start = i + 1;
            }
            b if b.is_ascii_control() => return Err("BAD"),
            _ => {}
        }
    }
    if bracket || items.is_empty() || (!list && items.len() > 1) {
        Err("BAD")
    } else {
        Ok(items)
    }
}

fn content(text: &str) -> Option<Content> {
    let alias = match text {
        "RFC822" => Some((Section::Whole, false)),
        "RFC822.HEADER" => Some((Section::Header, true)),
        "RFC822.TEXT" => Some((Section::Text, false)),
        _ => None,
    };
    if let Some((section, peek)) = alias {
        return Some(Content {
            label: text.into(),
            path: vec![],
            section,
            partial: None,
            peek,
            binary: false,
            size: false,
        });
    }
    let (prefix, rest) = text.split_once('[')?;
    // The outer closing bracket is last; quoted field names may contain ].
    let (section, suffix) = rest.rsplit_once(']')?;
    let (peek, binary, size) = match prefix {
        "BODY" => (false, false, false),
        "BODY.PEEK" => (true, false, false),
        "BINARY" => (false, true, false),
        "BINARY.PEEK" => (true, true, false),
        "BINARY.SIZE" => (true, true, true),
        _ => return None,
    };
    let partial = if suffix.is_empty() {
        None
    } else {
        if size {
            return None;
        }
        let (offset, count) = suffix
            .strip_prefix('<')?
            .strip_suffix('>')?
            .split_once('.')?;
        let offset = number(offset, false)?;
        let count = number(count, true)?;
        Some((offset, count))
    };
    let (path, kind) = section_kind(section)?;
    if binary && !matches!(kind, Section::Whole | Section::Content) {
        return None;
    }
    let section = if let Section::Fields { names, exclude } = &kind {
        let path = path.iter().map(|n| format!("{n}.")).collect::<String>();
        format!(
            "{path}HEADER.FIELDS{} ({})",
            if *exclude { ".NOT" } else { "" },
            names
                .iter()
                .map(|name| {
                    if name.bytes().any(|b| b"(){}[]%*\"\\".contains(&b)) {
                        format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\""))
                    } else {
                        name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else {
        section.to_owned()
    };
    let label = format!(
        "{}[{}]{}",
        if size {
            "BINARY.SIZE"
        } else if binary {
            "BINARY"
        } else {
            "BODY"
        },
        section,
        partial
            .map(|(offset, _)| format!("<{offset}>"))
            .unwrap_or_default()
    );
    Some(Content {
        label,
        path,
        section: kind,
        partial,
        peek,
        binary,
        size,
    })
}

fn number(value: &str, nonzero: bool) -> Option<usize> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let number = value.parse::<u32>().ok()?;
    (!nonzero || number > 0).then_some(number as usize)
}

fn section_kind(mut input: &str) -> Option<(Vec<usize>, Section)> {
    let mut path = vec![];
    while input.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        let (head, tail) = input.split_once('.').unwrap_or((input, ""));
        if path.len() >= 32 || head.starts_with('0') {
            return None;
        }
        path.push(number(head, true)?);
        if tail.is_empty() {
            return (!input.ends_with('.')).then_some((path, Section::Content));
        }
        input = tail;
    }
    let kind = match input {
        "" if path.is_empty() => Section::Whole,
        "HEADER" => Section::Header,
        "TEXT" => Section::Text,
        "MIME" => Section::Mime,
        _ => {
            let (key, list) = input.split_once(' ')?;
            let exclude = match key {
                "HEADER.FIELDS" => false,
                "HEADER.FIELDS.NOT" => true,
                _ => return None,
            };
            let names = super::super::syntax::tokens(
                list.trim_start().strip_prefix('(')?.strip_suffix(')')?,
            )?
            .into_iter()
            .map(|token| match token {
                super::super::syntax::Token::Word(name)
                    if !name.bytes().any(|b| b"[]%*".contains(&b)) =>
                {
                    Some(name)
                }
                super::super::syntax::Token::Quoted(name) => Some(name),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
            if names.is_empty()
                || names.iter().any(|s| {
                    s.is_empty() || !s.bytes().all(|b| (33..=126).contains(&b) && b != b':')
                })
            {
                return None;
            }
            Section::Fields { names, exclude }
        }
    };
    Some((path, kind))
}
