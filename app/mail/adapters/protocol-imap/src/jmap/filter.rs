use super::filter_message::Candidate;
pub(super) use super::filter_message::text;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Quantifier {
    Some,
    None,
    All,
}

/// RFC 8621 §4.4.1 Email/query filter tree. Leaves needing message bytes are
/// `Text`, `Header` and `Attachment`; every other leaf reads stored metadata.
#[derive(Clone, Debug)]
pub(super) enum Filter {
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Vec<Self>),
    Mailbox(String),
    MailboxOtherThan(Vec<String>),
    Before(i64),
    After(i64),
    MinSize(usize),
    MaxSize(usize),
    Keyword(bool, String),
    Thread(Quantifier, String),
    Attachment(bool),
    Text(String, String),
    Header(String, Option<String>),
}
impl Filter {
    pub fn parse(value: &Value) -> Result<Self, &'static str> {
        Self::parse_at(value, 0, &mut 64)
    }
    fn parse_at(value: &Value, depth: usize, remaining: &mut usize) -> Result<Self, &'static str> {
        if depth > 16 || *remaining == 0 {
            return Err("unsupportedFilter");
        }
        *remaining -= 1;
        if value.is_null() {
            return Ok(Self::All(vec![]));
        }
        let object = value.as_object().ok_or("invalidArguments")?;
        if object.contains_key("operator") {
            if object
                .keys()
                .any(|key| !["operator", "conditions"].contains(&key.as_str()))
            {
                return Err("invalidArguments");
            }
            let conditions = value["conditions"].as_array().ok_or("invalidArguments")?;
            let mut filters = Vec::new();
            for condition in conditions {
                if !condition.is_object() {
                    return Err("invalidArguments");
                }
                filters.push(Self::parse_at(condition, depth + 1, remaining)?);
            }
            return match value["operator"].as_str() {
                Some("AND") => Ok(Self::All(filters)),
                Some("OR") => Ok(Self::Any(filters)),
                Some("NOT") => Ok(Self::Not(filters)),
                _ => Err("invalidArguments"),
            };
        }
        let mut filters = Vec::new();
        for (property, value) in object {
            filters.push(Self::condition(property, value)?);
        }
        Ok(Self::All(filters))
    }
    fn condition(property: &str, value: &Value) -> Result<Self, &'static str> {
        let size = || {
            usize::try_from(value.as_u64().ok_or("invalidArguments")?)
                .map_err(|_| "invalidArguments")
        };
        let date = || {
            let text = value.as_str().ok_or("invalidArguments")?;
            mail_parser::DateTime::parse_rfc3339(text)
                .filter(mail_parser::DateTime::is_valid)
                .map(|date| date.to_timestamp())
                .ok_or("invalidArguments")
        };
        Ok(match property {
            "inMailbox" => Self::Mailbox(string(value)?),
            "inMailboxOtherThan" => {
                let ids = value.as_array().ok_or("invalidArguments")?;
                if ids.len() > 256 {
                    return Err("unsupportedFilter");
                }
                Self::MailboxOtherThan(ids.iter().map(string).collect::<Result<_, _>>()?)
            }
            "before" => Self::Before(date()?),
            "after" => Self::After(date()?),
            "minSize" => Self::MinSize(size()?),
            "maxSize" => Self::MaxSize(size()?),
            "hasKeyword" => Self::Keyword(true, string(value)?),
            "notKeyword" => Self::Keyword(false, string(value)?),
            "someInThreadHaveKeyword" => Self::Thread(Quantifier::Some, string(value)?),
            "noneInThreadHaveKeyword" => Self::Thread(Quantifier::None, string(value)?),
            "allInThreadHaveKeyword" => Self::Thread(Quantifier::All, string(value)?),
            "hasAttachment" => Self::Attachment(value.as_bool().ok_or("invalidArguments")?),
            "text" | "subject" | "body" | "from" | "to" | "cc" | "bcc" => {
                Self::Text(property.to_owned(), string(value)?.to_lowercase())
            }
            "header" => {
                let parts = value.as_array().ok_or("invalidArguments")?;
                let (name, needle) = match parts.as_slice() {
                    [name] => (string(name)?, None),
                    [name, needle] => (string(name)?, Some(string(needle)?.to_lowercase())),
                    _ => return Err("invalidArguments"),
                };
                if name.is_empty() || !name.bytes().all(|b| (33..=126).contains(&b) && b != b':') {
                    return Err("invalidArguments");
                }
                Self::Header(name.to_lowercase(), needle)
            }
            _ => return Err("unsupportedFilter"),
        })
    }
    fn any(&self, leaf: &dyn Fn(&Self) -> bool) -> bool {
        match self {
            Self::All(v) | Self::Any(v) | Self::Not(v) => v.iter().any(|f| f.any(leaf)),
            _ => leaf(self),
        }
    }
    /// True when evaluation needs the parsed message bytes.
    pub fn content(&self) -> bool {
        self.any(&|f| matches!(f, Self::Text(..) | Self::Header(..) | Self::Attachment(_)))
    }
    /// True when evaluation needs the stored message size.
    pub fn sized(&self) -> bool {
        self.any(&|f| matches!(f, Self::MinSize(_) | Self::MaxSize(_)))
    }
    /// True when evaluation needs the keywords of every thread member.
    pub fn threaded(&self) -> bool {
        self.any(&|f| matches!(f, Self::Thread(..)))
    }
    pub fn matches(&self, c: &Candidate<'_>) -> bool {
        match self {
            Self::All(v) => v.iter().all(|f| f.matches(c)),
            Self::Any(v) => v.iter().any(|f| f.matches(c)),
            Self::Not(v) => !v.iter().any(|f| f.matches(c)),
            Self::Mailbox(id) => c.state.mailboxes.contains(id),
            Self::MailboxOtherThan(ids) => c.state.mailboxes.iter().any(|m| !ids.contains(m)),
            Self::Before(instant) => c.state.received_at < *instant,
            Self::After(instant) => c.state.received_at >= *instant,
            Self::MinSize(bytes) => c.size.is_some_and(|size| size >= *bytes),
            Self::MaxSize(bytes) => c.size.is_some_and(|size| size < *bytes),
            Self::Keyword(has, keyword) => c.state.keywords.contains(keyword) == *has,
            Self::Thread(quantifier, keyword) => c.threads.is_some_and(|threads| {
                let thread = c.thread_id();
                match quantifier {
                    Quantifier::Some => threads.some(thread, keyword),
                    Quantifier::None => !threads.some(thread, keyword),
                    Quantifier::All => threads.all(thread, keyword),
                }
            }),
            Self::Attachment(has) => c.content.is_some_and(|x| x.has_attachment == *has),
            Self::Text(property, term) => c.content.is_some_and(|x| x.matches(property, term)),
            Self::Header(name, needle) => c.content.is_some_and(|x| {
                x.headers.iter().any(|(header, value)| {
                    header == name && needle.as_ref().is_none_or(|needle| value.contains(needle))
                })
            }),
        }
    }
    pub fn weight(&self) -> usize {
        match self {
            Self::All(v) | Self::Any(v) | Self::Not(v) => {
                v.iter().map(Self::weight).sum::<usize>().max(1)
            }
            _ => 1,
        }
    }
    pub fn charge(&self, size: usize, remaining: &mut usize) -> Result<(), &'static str> {
        *remaining = remaining
            .checked_sub(size.checked_mul(self.weight()).ok_or("limit")?)
            .ok_or("limit")?;
        Ok(())
    }
    pub fn terms<'a>(&'a self, property: &str, terms: &mut Vec<&'a str>) {
        match self {
            Self::All(v) | Self::Any(v) => {
                for filter in v {
                    filter.terms(property, terms);
                }
            }
            Self::Text(field, term)
                if (field == "text" || field == property) && !term.is_empty() =>
            {
                terms.push(term)
            }
            _ => {}
        }
    }
}

fn string(value: &Value) -> Result<String, &'static str> {
    let text = value.as_str().ok_or("invalidArguments")?;
    if text.len() > 4096 {
        return Err("unsupportedFilter");
    }
    Ok(text.to_owned())
}

pub(super) const MAX_SCAN_BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn scan_budget_charges_filter_complexity_without_overflow() {
        let filter =
            Filter::parse(&json!({"operator":"OR","conditions":[{"text":"one"},{"text":"two"}]}))
                .unwrap();
        let mut remaining = 100;
        assert_eq!(filter.charge(50, &mut remaining), Ok(()));
        assert_eq!(remaining, 0);
        assert_eq!(filter.charge(1, &mut remaining), Err("limit"));
        assert_eq!(filter.charge(usize::MAX, &mut remaining), Err("limit"));
    }
    #[test]
    fn metadata_conditions_follow_rfc_8621_boundaries() {
        let state = mail_kernel::MessageState {
            id: "e1".into(),
            modseq: 1,
            uids: Default::default(),
            thread: None,
            mailboxes: vec!["inbox".into()],
            keywords: vec!["$seen".into()],
            received_at: 1_767_225_600,
        };
        let candidate = Candidate {
            state: &state,
            size: Some(1000),
            content: None,
            threads: None,
        };
        for (filter, expected) in [
            (json!({"before":"2026-01-01T00:00:00Z"}), false),
            (json!({"after":"2026-01-01T00:00:00+00:00"}), true),
            (json!({"minSize":1000}), true),
            (json!({"maxSize":1000}), false),
            (json!({"hasKeyword":"$seen","notKeyword":"$flagged"}), true),
            (json!({"hasKeyword":"$Seen"}), false),
            (json!({"inMailboxOtherThan":["inbox"]}), false),
            (json!({"inMailboxOtherThan":["archive"]}), true),
        ] {
            let filter = Filter::parse(&filter).unwrap();
            assert!(!filter.content());
            assert_eq!(filter.matches(&candidate), expected, "{filter:?}");
        }
        assert!(
            Filter::parse(&json!({"hasAttachment":true}))
                .unwrap()
                .content()
        );
        for invalid in [
            json!({"before":"yesterday"}),
            json!({"header":[]}),
            json!({"header":["a:b"]}),
            json!({"minSize":-1}),
        ] {
            assert_eq!(Filter::parse(&invalid).err(), Some("invalidArguments"));
        }
        assert_eq!(
            Filter::parse(&json!({"unknown":"x"})).err(),
            Some("unsupportedFilter")
        );
    }
}
