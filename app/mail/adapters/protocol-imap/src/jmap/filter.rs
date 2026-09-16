use mail_kernel::MessageState;
use serde_json::Value;

#[derive(Clone)]
pub(super) enum Filter {
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Vec<Self>),
    Mailbox(String),
    Text(String, String),
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
            if !matches!(property.as_str(), "inMailbox" | "text" | "subject" | "body") {
                return Err("unsupportedFilter");
            }
            let value = value.as_str().ok_or("invalidArguments")?;
            if value.len() > 4096 {
                return Err("unsupportedFilter");
            }
            filters.push(if property == "inMailbox" {
                Self::Mailbox(value.into())
            } else {
                Self::Text(property.clone(), value.to_lowercase())
            });
        }
        Ok(Self::All(filters))
    }
    pub fn content(&self) -> bool {
        match self {
            Self::All(v) | Self::Any(v) | Self::Not(v) => v.iter().any(Self::content),
            Self::Text(..) => true,
            Self::Mailbox(_) => false,
        }
    }
    pub fn matches(
        &self,
        message: &MessageState,
        subject: &str,
        body: &str,
        addresses: &str,
    ) -> bool {
        match self {
            Self::All(v) => v
                .iter()
                .all(|f| f.matches(message, subject, body, addresses)),
            Self::Any(v) => v
                .iter()
                .any(|f| f.matches(message, subject, body, addresses)),
            Self::Not(v) => !v
                .iter()
                .any(|f| f.matches(message, subject, body, addresses)),
            Self::Mailbox(id) => message.mailboxes.contains(id),
            Self::Text(property, term) => {
                (property != "body" && subject.contains(term))
                    || (property != "subject" && body.contains(term))
                    || (property == "text" && addresses.contains(term))
            }
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

pub(super) const MAX_SCAN_BYTES: usize = 64 * 1024 * 1024;
pub(super) fn text(raw: &[u8]) -> Result<(String, String, String), &'static str> {
    let parsed = mail_parser::MessageParser::default()
        .parse(raw)
        .ok_or("serverFail")?;
    if parsed.parts.len() > 1000 {
        return Err("tooLarge");
    }
    let subject = parsed.subject().unwrap_or_default().to_owned();
    let mut body = String::new();
    for index in 0..parsed.text_body.len().max(parsed.html_body.len()) {
        if let Some(part) = parsed.body_text(index) {
            body.push_str(&part);
            body.push('\n');
        }
    }
    let mut addresses = String::new();
    for list in [parsed.from(), parsed.to(), parsed.cc(), parsed.bcc()]
        .into_iter()
        .flatten()
    {
        for address in list.iter() {
            for value in [address.name.as_deref(), address.address.as_deref()]
                .into_iter()
                .flatten()
            {
                addresses.push_str(value);
                addresses.push(' ');
            }
        }
    }
    Ok((subject, body, addresses))
}

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
}
