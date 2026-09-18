use super::filter_message::{Candidate, Content};
use serde_json::{Value, json};
use std::cmp::Ordering;

/// Sort properties advertised as `emailQuerySortOptions` (RFC 8621 §4.4.2).
pub(super) const PROPERTIES: [&str; 7] = [
    "receivedAt",
    "size",
    "from",
    "to",
    "subject",
    "sentAt",
    "hasKeyword",
];

#[derive(Clone, PartialEq)]
enum Key {
    ReceivedAt,
    Size,
    From,
    To,
    Subject,
    SentAt,
    HasKeyword(String),
}

#[derive(Clone)]
pub(super) struct Sort {
    key: Key,
    ascending: bool,
}
impl Sort {
    /// Parses the `sort` argument; absent or empty means receivedAt descending.
    pub fn parse(value: &Value) -> Result<Vec<Self>, &'static str> {
        let newest_first = vec![Self {
            key: Key::ReceivedAt,
            ascending: false,
        }];
        if value.is_null() {
            return Ok(newest_first);
        }
        let list = value.as_array().ok_or("invalidArguments")?;
        if list.len() > 8 {
            return Err("unsupportedSort");
        }
        let mut sorts = Vec::new();
        for item in list {
            let object = item.as_object().ok_or("invalidArguments")?;
            if object
                .keys()
                .any(|k| !["property", "isAscending", "collation", "keyword"].contains(&k.as_str()))
                || (!item["collation"].is_null() && item["collation"] != "")
            {
                return Err("unsupportedSort");
            }
            let key = match item["property"].as_str().ok_or("invalidArguments")? {
                "receivedAt" => Key::ReceivedAt,
                "size" => Key::Size,
                "from" => Key::From,
                "to" => Key::To,
                "subject" => Key::Subject,
                "sentAt" => Key::SentAt,
                "hasKeyword" => Key::HasKeyword(
                    item["keyword"]
                        .as_str()
                        .ok_or("invalidArguments")?
                        .to_owned(),
                ),
                _ => return Err("unsupportedSort"),
            };
            if !matches!(key, Key::HasKeyword(_)) && !item["keyword"].is_null() {
                return Err("invalidArguments");
            }
            let ascending = if item["isAscending"].is_null() {
                true
            } else {
                item["isAscending"].as_bool().ok_or("invalidArguments")?
            };
            sorts.push(Self { key, ascending });
        }
        Ok(if sorts.is_empty() {
            newest_first
        } else {
            sorts
        })
    }
    /// True when any key needs the parsed message bytes.
    pub fn content(sorts: &[Self]) -> bool {
        sorts
            .iter()
            .any(|s| matches!(s.key, Key::From | Key::To | Key::Subject | Key::SentAt))
    }
    /// True when every key is immutable for a stored message, so retained
    /// items keep their relative order between two query states.
    pub fn stable(sorts: &[Self]) -> bool {
        sorts.iter().all(|s| s.key == Key::ReceivedAt)
    }
    /// Canonical form for the query-state fingerprint.
    pub fn canonical(sorts: &[Self]) -> Value {
        json!(
            sorts
                .iter()
                .map(|s| {
                    let (property, keyword) = match &s.key {
                        Key::ReceivedAt => ("receivedAt", None),
                        Key::Size => ("size", None),
                        Key::From => ("from", None),
                        Key::To => ("to", None),
                        Key::Subject => ("subject", None),
                        Key::SentAt => ("sentAt", None),
                        Key::HasKeyword(keyword) => ("hasKeyword", Some(keyword)),
                    };
                    json!([property, keyword, s.ascending])
                })
                .collect::<Vec<_>>()
        )
    }
    pub fn compare(sorts: &[Self], a: &Candidate<'_>, b: &Candidate<'_>) -> Ordering {
        for sort in sorts {
            let order = sort.key.compare(a, b);
            let order = if sort.ascending {
                order
            } else {
                order.reverse()
            };
            if order != Ordering::Equal {
                return order;
            }
        }
        a.state.id.cmp(&b.state.id)
    }
}
impl Key {
    fn compare(&self, a: &Candidate<'_>, b: &Candidate<'_>) -> Ordering {
        match self {
            Self::ReceivedAt => a.state.received_at.cmp(&b.state.received_at),
            Self::Size => a.size.cmp(&b.size),
            Self::From => text(a, |c| &c.sort_from).cmp(text(b, |c| &c.sort_from)),
            Self::To => text(a, |c| &c.sort_to).cmp(text(b, |c| &c.sort_to)),
            Self::Subject => text(a, |c| &c.sort_subject).cmp(text(b, |c| &c.sort_subject)),
            Self::SentAt => a
                .content
                .and_then(|c| c.sent_at)
                .cmp(&b.content.and_then(|c| c.sent_at)),
            Self::HasKeyword(keyword) => a
                .state
                .keywords
                .contains(keyword)
                .cmp(&b.state.keywords.contains(keyword)),
        }
    }
}

fn text<'a>(c: &Candidate<'a>, pick: fn(&Content) -> &str) -> &'a str {
    c.content.map(pick).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn keys(value: Value) -> Result<Vec<Sort>, &'static str> {
        Sort::parse(&value)
    }
    #[test]
    fn parse_accepts_advertised_properties_and_refuses_others() {
        for property in PROPERTIES {
            let sort = keys(json!([{"property":property,"keyword":"$seen"}]));
            if property == "hasKeyword" {
                assert!(sort.is_ok());
            } else {
                assert_eq!(sort.err(), Some("invalidArguments"));
                assert!(keys(json!([{"property":property}])).is_ok());
            }
        }
        assert_eq!(
            keys(json!([{"property":"threadId"}])).err(),
            Some("unsupportedSort")
        );
        assert_eq!(
            keys(json!([{"property":"hasKeyword"}])).err(),
            Some("invalidArguments")
        );
        assert_eq!(
            keys(json!([{"property":"size","collation":"i;octet"}])).err(),
            Some("unsupportedSort")
        );
        assert!(Sort::stable(&keys(Value::Null).unwrap()));
        assert!(!Sort::stable(&keys(json!([{"property":"size"}])).unwrap()));
        assert!(Sort::content(
            &keys(json!([{"property":"sentAt"}])).unwrap()
        ));
    }
    #[test]
    fn descending_has_keyword_puts_flagged_first_then_falls_through() {
        let state = |id: &str, keywords: &[&str], received_at: i64| mail_kernel::MessageState {
            id: id.into(),
            modseq: 1,
            uids: Default::default(),
            thread: None,
            mailboxes: vec![],
            keywords: keywords.iter().map(|k| (*k).to_owned()).collect(),
            received_at,
        };
        let sorts = keys(json!([
            {"property":"hasKeyword","keyword":"$flagged","isAscending":false},
            {"property":"receivedAt","isAscending":true}
        ]))
        .unwrap();
        let (plain, flagged, older) = (
            state("a", &[], 10),
            state("b", &["$flagged"], 20),
            state("c", &[], 5),
        );
        let candidate = |state| Candidate {
            state,
            size: None,
            content: None,
            threads: None,
        };
        assert_eq!(
            Sort::compare(&sorts, &candidate(&flagged), &candidate(&plain)),
            Ordering::Less
        );
        assert_eq!(
            Sort::compare(&sorts, &candidate(&older), &candidate(&plain)),
            Ordering::Less
        );
    }
}
