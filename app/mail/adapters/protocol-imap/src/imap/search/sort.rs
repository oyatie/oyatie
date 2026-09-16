use super::{super::syntax::Token, criteria::Parser};
use mail_kernel::Message;
use std::cmp::Ordering;
use unicode_normalization::UnicodeNormalization;
mod subject;

pub(super) struct Key {
    field: Field,
    reverse: bool,
}
#[derive(Clone, Copy)]
enum Field {
    Arrival,
    Cc,
    Date,
    From,
    Size,
    Subject,
    To,
}
#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub(super) enum Value {
    Missing,
    Number(i64),
    Text(String),
}

impl Key {
    pub fn needs_raw(&self) -> bool {
        !matches!(self.field, Field::Arrival | Field::Size)
    }
}

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<Vec<Key>, &'static str> {
    if !matches!(parser.tokens.first(), Some(Token::Open)) {
        return Err("BAD");
    }
    parser.tokens = &parser.tokens[1..];
    let mut keys = Vec::new();
    while !matches!(parser.tokens.first(), Some(Token::Close)) {
        let word = parser.atom()?;
        let reverse = word.eq_ignore_ascii_case("REVERSE");
        let word = if reverse { parser.atom()? } else { word };
        let field = match word.to_ascii_uppercase().as_str() {
            "ARRIVAL" => Field::Arrival,
            "CC" => Field::Cc,
            "DATE" => Field::Date,
            "FROM" => Field::From,
            "SIZE" => Field::Size,
            "SUBJECT" => Field::Subject,
            "TO" => Field::To,
            _ => return Err("BAD"),
        };
        if keys.len() == 32 {
            return Err("NO [LIMIT] Sort program too large");
        }
        keys.push(Key { field, reverse });
    }
    parser.tokens = &parser.tokens[1..];
    if keys.is_empty() {
        Err("BAD")
    } else {
        Ok(keys)
    }
}

pub(super) fn values(
    keys: &[Key],
    message: &Message,
    parsed: Option<&mail_parser::Message<'_>>,
    budget: &mut usize,
) -> Result<Vec<Value>, &'static str> {
    keys.iter()
        .map(|key| {
            *budget = budget
                .checked_sub(std::mem::size_of::<Value>())
                .ok_or("NO [LIMIT] Sort memory exceeded")?;
            Ok(match key.field {
                Field::Arrival => Value::Number(message.received_at),
                Field::Size => Value::Number(message.size as i64),
                Field::Date => parsed
                    .and_then(message_date)
                    .map_or(Value::Missing, Value::Number),
                Field::Subject => match parsed.and_then(|m| m.subject()) {
                    Some(value) => Value::Text(casemap(&subject::base(value), budget)?),
                    None => Value::Missing,
                },
                Field::Cc | Field::From | Field::To => {
                    let address = parsed
                        .and_then(|m| match key.field {
                            Field::Cc => m.cc(),
                            Field::From => m.from(),
                            _ => m.to(),
                        })
                        .and_then(|a| a.first())
                        .and_then(|a| a.address.as_deref());
                    let Some(address) = address else {
                        return Ok(Value::Missing);
                    };
                    let local = address.rsplit_once('@').map_or(address, |(local, _)| local);
                    Value::Text(casemap(local, budget)?)
                }
            })
        })
        .collect()
}

pub(super) fn compare(keys: &[Key], left: &[Value], right: &[Value]) -> Ordering {
    keys.iter()
        .zip(left.iter().zip(right))
        .map(|(key, (a, b))| {
            // Stalwart ranks absence after positive indexed ranks. Descending
            // absence shares rank zero with indexed empty text.
            match (a, b) {
                (Value::Missing, Value::Missing) => return Ordering::Equal,
                (Value::Missing, Value::Text(text)) | (Value::Text(text), Value::Missing)
                    if key.reverse && text.is_empty() =>
                {
                    return Ordering::Equal;
                }
                (Value::Missing, _) => return Ordering::Greater,
                (_, Value::Missing) => return Ordering::Less,
                _ => {}
            }
            let order = a.cmp(b);
            if key.reverse { order.reverse() } else { order }
        })
        .find(|o| !o.is_eq())
        .unwrap_or(Ordering::Equal)
}

// RFC 5051 maps each scalar using UnicodeData's simple titlecase, then NFKD.
// Full case expansions (e.g. sharp s) are not simple titlecase mappings.
fn casemap(input: &str, budget: &mut usize) -> Result<String, &'static str> {
    let mut result = String::new();
    for c in input
        .chars()
        .map(|c| match unicode_case_mapping::to_titlecase(c) {
            [mapped, 0, 0] if mapped != 0 => char::from_u32(mapped).unwrap_or(c),
            _ => c,
        })
        .nfkd()
    {
        *budget = budget
            .checked_sub(c.len_utf8())
            .ok_or("NO [LIMIT] Sort memory exceeded")?;
        result.push(c);
    }
    Ok(result)
}

fn message_date(message: &mail_parser::Message<'_>) -> Option<i64> {
    let mut date = *message.date()?;
    // mail-parser deliberately wraps numeric zone fields; retain RFC 5256's
    // invalid-zone-as-UTC behavior by checking the original token first.
    if message.header_raw("Date").is_some_and(invalid_zone) {
        date.tz_hour = 0;
        date.tz_minute = 0;
    }
    sent_date(date)
}

fn invalid_zone(raw: &str) -> bool {
    let mut depth = 0;
    let mut escaped = false;
    let mut start = None;
    let mut last = "";
    for (i, c) in raw.char_indices() {
        if depth > 0 {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
            }
        } else if c.is_ascii_whitespace() || c == '(' {
            if let Some(start) = start.take() {
                last = &raw[start..i];
            }
            if c == '(' {
                depth = 1;
            }
        } else {
            start.get_or_insert(i);
        }
    }
    if let Some(start) = start {
        last = &raw[start..];
    }
    let Some(zone) = last.strip_prefix(['+', '-']) else {
        return false;
    };
    zone.len() != 4
        || !zone.bytes().all(|b| b.is_ascii_digit())
        || zone[..2].parse::<u8>().map_or(true, |hour| hour > 23)
        || zone[2..].parse::<u8>().map_or(true, |minute| minute > 59)
}

fn sent_date(mut date: mail_parser::DateTime) -> Option<i64> {
    if !super::criteria::valid_date(date.year, date.month, date.day) {
        return None;
    }
    // RFC 5256 section 2.2 retains a valid date when its time is invalid.
    if date.hour > 23 || date.minute > 59 || date.second > 60 {
        date.hour = 0;
        date.minute = 0;
        date.second = 0;
    }
    Some(date.to_timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_decomposition_cannot_exceed_remaining_sort_budget() {
        // U+FDFA compatibility decomposition expands one scalar into 18.
        let input = "\u{fdfa}";
        let mut budget = 8;
        assert!(casemap(input, &mut budget).is_err());
        assert!(budget < 8);
        let mut budget = 256;
        let mapped = casemap(input, &mut budget).unwrap();
        assert_eq!(256 - budget, mapped.len());
    }

    #[test]
    fn invalid_sent_date_is_missing_and_invalid_time_uses_midnight() {
        let parse = |date| mail_parser::DateTime::parse_rfc822(date).unwrap();
        assert_eq!(sent_date(parse("31 Feb 2024 12:00:00 +0000")), None);
        assert_eq!(sent_date(parse("29 Feb 2023 12:00:00 +0000")), None);
        assert_eq!(sent_date(parse("0 Jan 2024 12:00:00 +0000")), None);
        assert_eq!(
            sent_date(parse("1 Jan 2024 99:00:00 +0000")),
            sent_date(parse("1 Jan 2024 00:00:00 +0000"))
        );
        assert_eq!(
            sent_date(parse("29 Feb 2024 12:00:00 +0000")),
            Some(1_709_208_000)
        );
    }
}
