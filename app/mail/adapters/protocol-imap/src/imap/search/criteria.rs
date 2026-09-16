use super::super::syntax::{Token, keyword, sequence_set};

pub(super) enum Criterion {
    All,
    Never,
    And(Vec<Self>),
    Or(Box<Self>, Box<Self>),
    Not(Box<Self>),
    Nor(Vec<Self>),
    Flag(String),
    Size {
        larger: bool,
        value: usize,
    },
    Date {
        sent: bool,
        order: std::cmp::Ordering,
        value: (u16, u8, u8),
    },
    Header(String, String),
    Body {
        headers: bool,
        value: String,
    },
    Set {
        uid: bool,
        ranges: Vec<(u32, u32)>,
    },
    Saved,
    Modseq(u64),
    Identity {
        thread: bool,
        value: String,
    },
}

impl Criterion {
    pub fn uses_sequences(&self) -> bool {
        match self {
            Self::Set { uid: false, .. } => true,
            Self::And(items) | Self::Nor(items) => items.iter().any(Self::uses_sequences),
            Self::Or(a, b) => a.uses_sequences() || b.uses_sequences(),
            Self::Not(item) => item.uses_sequences(),
            _ => false,
        }
    }
    pub fn has_modseq(&self) -> bool {
        match self {
            Self::Modseq(_) => true,
            Self::And(items) | Self::Nor(items) => items.iter().any(Self::has_modseq),
            Self::Or(a, b) => a.has_modseq() || b.has_modseq(),
            Self::Not(item) => item.has_modseq(),
            _ => false,
        }
    }
    pub fn needs_raw(&self) -> bool {
        match self {
            Self::Header(..) | Self::Body { .. } | Self::Date { sent: true, .. } => true,
            Self::And(items) | Self::Nor(items) => items.iter().any(Self::needs_raw),
            Self::Or(a, b) => a.needs_raw() || b.needs_raw(),
            Self::Not(item) => item.needs_raw(),
            _ => false,
        }
    }
}

pub(super) struct Parser<'a> {
    pub tokens: &'a [Token],
    pub largest: u32,
    pub largest_uid: u32,
}
impl Parser<'_> {
    pub fn atom(&mut self) -> Result<String, &'static str> {
        if !matches!(self.tokens.first(), Some(Token::Word(_))) {
            return Err("BAD");
        }
        self.text()
    }

    pub fn text(&mut self) -> Result<String, &'static str> {
        let (token, rest) = self.tokens.split_first().ok_or("BAD")?;
        self.tokens = rest;
        match token {
            Token::Word(value) | Token::Quoted(value) => Ok(value.clone()),
            _ => Err("BAD"),
        }
    }
    pub fn and(&mut self, depth: usize) -> Result<Criterion, &'static str> {
        let mut items = Vec::new();
        while !self.tokens.is_empty() && !matches!(self.tokens[0], Token::Close) {
            match self.one(depth)? {
                Criterion::And(nested) => items.extend(nested),
                item => items.push(item),
            }
        }
        if items.is_empty() {
            Err("BAD")
        } else {
            Ok(Criterion::And(items))
        }
    }
    fn one(&mut self, depth: usize) -> Result<Criterion, &'static str> {
        if depth > 32 {
            return Err("BAD");
        }
        let (token, rest) = self.tokens.split_first().ok_or("BAD")?;
        self.tokens = rest;
        let word = match token {
            Token::Open => {
                let value = self.and(depth + 1)?;
                if !matches!(self.tokens.first(), Some(Token::Close)) {
                    return Err("BAD");
                }
                self.tokens = &self.tokens[1..];
                return Ok(value);
            }
            Token::Word(word) => word.to_ascii_uppercase(),
            _ => return Err("BAD"),
        };
        Ok(match word.as_str() {
            "ALL" | "OLD" => Criterion::All,
            "RECENT" | "NEW" => Criterion::Never,
            "OR" => Criterion::Or(
                Box::new(self.one(depth + 1)?),
                Box::new(self.one(depth + 1)?),
            ),
            "NOT" => match self.one(depth + 1)? {
                // The oracle flattens conjunctive groups into NOT's filter list.
                Criterion::And(items) => Criterion::Nor(items),
                item => Criterion::Not(Box::new(item)),
            },
            "ANSWERED" | "DELETED" | "DRAFT" | "FLAGGED" | "SEEN" => {
                Criterion::Flag(format!("${}", word.to_ascii_lowercase()))
            }
            "UNANSWERED" | "UNDELETED" | "UNDRAFT" | "UNFLAGGED" | "UNSEEN" => {
                Criterion::Not(Box::new(Criterion::Flag(format!(
                    "${}",
                    word[2..].to_ascii_lowercase()
                ))))
            }
            "KEYWORD" | "UNKEYWORD" => {
                let flag = self.text()?;
                if flag.is_empty()
                    || flag
                        .bytes()
                        .any(|b| b <= 32 || b >= 127 || b"(){}%*\"\\]".contains(&b))
                {
                    return Err("BAD");
                }
                let value = Criterion::Flag(keyword(&flag).ok_or("BAD")?);
                if word == "UNKEYWORD" {
                    Criterion::Not(Box::new(value))
                } else {
                    value
                }
            }
            "LARGER" | "SMALLER" => {
                let size = self.text()?;
                if size.is_empty() || !size.bytes().all(|b| b.is_ascii_digit()) {
                    return Err("BAD");
                }
                Criterion::Size {
                    larger: word == "LARGER",
                    value: size.parse::<u32>().map_err(|_| "BAD")? as usize,
                }
            }
            "BEFORE" | "ON" | "SINCE" | "SENTBEFORE" | "SENTON" | "SENTSINCE" => {
                let value = date(&self.text()?).ok_or("BAD")?;
                let sent = word.starts_with("SENT");
                let order = if word.ends_with("BEFORE") {
                    std::cmp::Ordering::Less
                } else if word.ends_with("SINCE") {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                };
                Criterion::Date { sent, order, value }
            }
            "BCC" | "CC" | "FROM" | "TO" | "SUBJECT" => {
                Criterion::Header(word, self.text()?.to_lowercase())
            }
            "HEADER" => {
                let name = self.text()?;
                if name.is_empty() || !name.bytes().all(|b| (33..=126).contains(&b) && b != b':') {
                    return Err("BAD");
                }
                Criterion::Header(name, self.text()?.to_lowercase())
            }
            "BODY" | "TEXT" => Criterion::Body {
                headers: word == "TEXT",
                value: self.text()?.to_lowercase(),
            },
            "UID" => {
                let set = self.text()?;
                if set == "$" {
                    Criterion::Saved
                } else {
                    Criterion::Set {
                        uid: true,
                        ranges: sequence_set(&set, self.largest_uid).ok_or("BAD")?,
                    }
                }
            }
            "$" => Criterion::Saved,
            "MODSEQ" => {
                let mut value = self.text()?;
                if super::super::condstore::number(&value).is_err() {
                    let flag = value
                        .strip_prefix("/flags/")
                        .and_then(keyword)
                        .ok_or("BAD")?;
                    if !mail_kernel::valid_keywords(&[flag])
                        || !matches!(
                            self.atom()?.to_ascii_lowercase().as_str(),
                            "all" | "shared" | "priv"
                        )
                    {
                        return Err("BAD");
                    }
                    value = self.text()?;
                }
                Criterion::Modseq(super::super::condstore::number(&value)?)
            }
            "EMAILID" | "THREADID" => {
                let value = self.text()?;
                if value.is_empty()
                    || value.len() > 255
                    || !value
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
                {
                    return Err("BAD");
                }
                Criterion::Identity {
                    thread: word == "THREADID",
                    value,
                }
            }
            _ => Criterion::Set {
                uid: false,
                ranges: sequence_set(&word, self.largest).ok_or("BAD")?,
            },
        })
    }
}

fn date(input: &str) -> Option<(u16, u8, u8)> {
    let mut parts = input.split('-');
    let day = parts.next()?;
    let month = parts.next()?;
    let year = parts.next()?;
    if parts.next().is_some()
        || day.is_empty()
        || day.len() > 2
        || year.len() != 4
        || !day.bytes().chain(year.bytes()).all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let day = day.parse::<u8>().ok()?;
    let year = year.parse::<u16>().ok()?;
    let month = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ]
    .iter()
    .position(|m| month.eq_ignore_ascii_case(m))? as u8
        + 1;
    valid_date(year, month, day).then_some((year, month, day))
}

pub(super) fn valid_date(year: u16, month: u8, day: u8) -> bool {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    year > 0 && (1..=12).contains(&month) && day > 0 && day <= days
}
