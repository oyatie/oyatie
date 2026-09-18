use mail_kernel::{Account, Mailbox};
use mail_service::MailService;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

struct Query {
    filter: Value,
    sort: Vec<(String, bool)>,
    fingerprint: String,
}
impl Query {
    fn parse(account: &str, args: &Value) -> Result<Self, &'static str> {
        let filter = if args["filter"].is_null() {
            json!({})
        } else {
            args["filter"].clone()
        };
        validate(&filter, 0)?;
        let mut sort = vec![];
        if !args["sort"].is_null() {
            for comparator in args["sort"].as_array().ok_or("invalidArguments")? {
                let property = comparator["property"].as_str().ok_or("invalidArguments")?;
                if !["name", "sortOrder"].contains(&property)
                    || (!comparator["collation"].is_null() && comparator["collation"] != "")
                {
                    return Err("unsupportedSort");
                }
                let ascending = if comparator["isAscending"].is_null() {
                    true
                } else {
                    comparator["isAscending"]
                        .as_bool()
                        .ok_or("invalidArguments")?
                };
                sort.push((property.into(), ascending));
            }
        }
        if sort.len() > 8 {
            return Err("unsupportedSort");
        }
        for property in ["sortAsTree", "filterAsTree"] {
            if !args[property].is_null() && !args[property].is_boolean() {
                return Err("invalidArguments");
            }
            if args[property] == true {
                return Err("unsupportedFilter");
            }
        }
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(json!(["Mailbox", account, filter, sort]).to_string())
        );
        Ok(Self {
            filter,
            sort,
            fingerprint,
        })
    }
    fn state(&self, revision: u64) -> String {
        format!("{revision}:{}", self.fingerprint)
    }
    fn ids(&self, mailboxes: &BTreeMap<String, Mailbox>) -> Vec<String> {
        let mut rows: Vec<_> = mailboxes
            .values()
            .filter(|m| matches(&self.filter, m))
            .collect();
        rows.sort_by(|a, b| {
            for (property, ascending) in &self.sort {
                let order = if property == "name" {
                    a.name.cmp(&b.name)
                } else {
                    a.sort_order.cmp(&b.sort_order)
                };
                if !order.is_eq() {
                    return if *ascending { order } else { order.reverse() };
                }
            }
            a.id.cmp(&b.id)
        });
        rows.into_iter().map(|m| m.id.clone()).collect()
    }
}

fn validate(filter: &Value, depth: usize) -> Result<(), &'static str> {
    if depth > 32 {
        return Err("unsupportedFilter");
    }
    let object = filter.as_object().ok_or("invalidArguments")?;
    if let Some(operator) = object.get("operator") {
        if ![json!("AND"), json!("OR"), json!("NOT")].contains(operator)
            || object.keys().any(|k| k != "operator" && k != "conditions")
        {
            return Err("unsupportedFilter");
        }
        let conditions = filter["conditions"].as_array().ok_or("invalidArguments")?;
        if conditions.len() > 256 {
            return Err("unsupportedFilter");
        }
        for condition in conditions {
            validate(condition, depth + 1)?;
        }
    } else {
        for (key, value) in object {
            let valid = match key.as_str() {
                "parentId" | "role" => value.is_null() || value.is_string(),
                "name" => value.is_string(),
                "hasAnyRole" | "isSubscribed" => value.is_boolean(),
                _ => return Err("unsupportedFilter"),
            };
            if !valid {
                return Err("invalidArguments");
            }
        }
    }
    Ok(())
}

fn matches(filter: &Value, m: &Mailbox) -> bool {
    if let Some(operator) = filter["operator"].as_str() {
        let conditions = filter["conditions"].as_array().expect("validated filter");
        return match operator {
            "AND" => conditions.iter().all(|f| matches(f, m)),
            "OR" => conditions.iter().any(|f| matches(f, m)),
            _ => !conditions.iter().any(|f| matches(f, m)),
        };
    }
    filter
        .as_object()
        .expect("validated filter")
        .iter()
        .all(|(key, value)| match key.as_str() {
            "name" => m
                .name
                .to_lowercase()
                .contains(&value.as_str().unwrap_or_default().to_lowercase()),
            "parentId" => m.parent_id.as_deref() == value.as_str(),
            "role" => m.role.as_deref() == value.as_str(),
            "hasAnyRole" => m.role.is_some() == (value == true),
            "isSubscribed" => m.is_subscribed == (value == true),
            _ => false,
        })
}

pub(super) fn query(account: &Account, args: &Value) -> Result<Value, &'static str> {
    let query = Query::parse(&account.id, args)?;
    let mailboxes = account
        .mailboxes
        .iter()
        .map(|m| (m.id.clone(), m.clone()))
        .collect();
    let ids = query.ids(&mailboxes);
    let (position, limit) = super::query::page(&ids, args)?;
    Ok(
        json!({"accountId":account.id,"queryState":query.state(account.revision),"canCalculateChanges":true,
        "position":position,"ids":ids.iter().skip(position).take(limit).collect::<Vec<_>>(),"total":ids.len()}),
    )
}

pub(super) fn changes(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let query = Query::parse(&account.id, args)?;
    let state = args["sinceQueryState"].as_str().ok_or("invalidArguments")?;
    let (revision, fingerprint) = state.split_once(':').ok_or("cannotCalculateChanges")?;
    if fingerprint != query.fingerprint {
        return Err("cannotCalculateChanges");
    }
    let since = revision.parse().map_err(|_| "cannotCalculateChanges")?;
    let max = super::changes::limit(args)?;
    if !args["upToId"].is_null() && !args["upToId"].is_string() {
        return Err("invalidArguments");
    }
    if !args["calculateTotal"].is_null() && !args["calculateTotal"].is_boolean() {
        return Err("invalidArguments");
    }
    let window = super::changes::window(service, token, account, since)?;
    let current: BTreeMap<_, _> = account
        .mailboxes
        .iter()
        .map(|m| (m.id.clone(), m.clone()))
        .collect();
    let after = query.ids(&current);
    let (ids, kind) = window.mailboxes(account);
    let (removed, added) = super::changes::delta(&window, &ids, &kind, &after, max)?;
    Ok(
        json!({"accountId":account.id,"oldQueryState":state,"newQueryState":query.state(account.revision),"removed":removed,"added":added,"total":after.len()}),
    )
}
