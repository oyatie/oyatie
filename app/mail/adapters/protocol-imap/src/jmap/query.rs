use mail_kernel::{Account, MessageState};
use mail_service::MailService;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

struct Query {
    filter: super::filter::Filter,
    ascending: bool,
    fingerprint: String,
}

impl Query {
    fn parse(account: &str, args: &Value) -> Result<Self, &'static str> {
        let filter = super::filter::Filter::parse(&args["filter"])?;
        let mut ascending = false;
        if !args["sort"].is_null() {
            let sort = args["sort"].as_array().ok_or("invalidArguments")?;
            if sort.len() > 1 {
                return Err("unsupportedSort");
            }
            if let Some(sort) = sort.first() {
                let object = sort.as_object().ok_or("invalidArguments")?;
                if sort["property"] != "receivedAt"
                    || object
                        .keys()
                        .any(|k| !["property", "isAscending", "collation"].contains(&k.as_str()))
                    || (!sort["collation"].is_null() && sort["collation"] != "")
                {
                    return Err("unsupportedSort");
                }
                ascending = if sort["isAscending"].is_null() {
                    true
                } else {
                    sort["isAscending"].as_bool().ok_or("invalidArguments")?
                };
            }
        }
        if !args["collapseThreads"].is_null() && !args["collapseThreads"].is_boolean() {
            return Err("invalidArguments");
        }
        if args["collapseThreads"] == true {
            return Err("unsupportedFilter");
        }
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(json!([account, args["filter"], ascending]).to_string())
        );
        Ok(Self {
            filter,
            ascending,
            fingerprint,
        })
    }

    fn ids(
        &self,
        states: impl IntoIterator<Item = MessageState>,
        content: Option<&BTreeSet<String>>,
    ) -> Vec<String> {
        let mut messages: Vec<_> = states
            .into_iter()
            .filter(|m| {
                content.map_or_else(
                    || self.filter.matches(m, "", "", ""),
                    |ids| ids.contains(&m.id),
                )
            })
            .collect();
        messages.sort_by(|a, b| {
            let order = a.received_at.cmp(&b.received_at);
            (if self.ascending {
                order
            } else {
                order.reverse()
            })
            .then_with(|| a.id.cmp(&b.id))
        });
        messages.into_iter().map(|m| m.id).collect()
    }

    fn state(&self, revision: u64) -> String {
        format!("{revision}:{}", self.fingerprint)
    }
}

pub(super) fn query(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let query = Query::parse(&account.id, args)?;
    let mut content = None;
    if query.filter.content() {
        let mut remaining = super::filter::MAX_SCAN_BYTES;
        let mut matched = BTreeSet::new();
        if account.messages.len() > 10000 {
            return Err("limit");
        }
        for message in &account.messages {
            query.filter.charge(message.size, &mut remaining)?;
            let raw = service
                .download(token, &account.id, &message.id)
                .map_err(super::method::error)?;
            let (subject, body, addresses) = super::filter::text(&raw)?;
            if query.filter.matches(
                &message.state(),
                &subject.to_lowercase(),
                &body.to_lowercase(),
                &addresses.to_lowercase(),
            ) {
                matched.insert(message.id.clone());
            }
        }
        content = Some(matched);
    }
    let ids = query.ids(account.messages.iter().map(|m| m.state()), content.as_ref());
    let (position, limit) = page(&ids, args)?;
    Ok(
        json!({"accountId":account.id,"queryState":query.state(account.revision),"canCalculateChanges":!query.filter.content(),
        "position":position,"ids":ids.iter().skip(position).take(limit).collect::<Vec<_>>(),"total":ids.len()}),
    )
}

pub(super) fn page(ids: &[String], args: &Value) -> Result<(usize, usize), &'static str> {
    let position = if !args["anchor"].is_null() {
        let anchor = args["anchor"].as_str().ok_or("invalidArguments")?;
        let index = ids
            .iter()
            .position(|id| id == anchor)
            .ok_or("anchorNotFound")? as i64;
        let offset = if args["anchorOffset"].is_null() {
            0
        } else {
            args["anchorOffset"].as_i64().ok_or("invalidArguments")?
        };
        index.saturating_add(offset).max(0) as usize
    } else {
        let position = if args["position"].is_null() {
            0
        } else {
            args["position"].as_i64().ok_or("invalidArguments")?
        };
        if position < 0 {
            (ids.len() as i64).saturating_add(position).max(0) as usize
        } else {
            position as usize
        }
    }
    .min(ids.len());
    let limit = if args["limit"].is_null() {
        256
    } else {
        args["limit"].as_u64().ok_or("invalidArguments")?.min(256) as usize
    };
    if !args["calculateTotal"].is_null() && !args["calculateTotal"].is_boolean() {
        return Err("invalidArguments");
    }
    Ok((position, limit))
}

pub(super) fn changes(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let query = Query::parse(&account.id, args)?;
    if query.filter.content() {
        return Err("cannotCalculateChanges");
    }
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
    let changes = super::changes::history(service, token, account, since)?;
    let mut old: BTreeMap<_, _> = account
        .messages
        .iter()
        .map(|m| (m.id.clone(), m.state()))
        .collect();
    for change in changes.iter().rev() {
        if let Some(before) = &change.before {
            old.insert(change.id.clone(), before.clone());
        } else {
            old.remove(&change.id);
        }
    }
    let before = query.ids(old.into_values(), None);
    let after = query.ids(account.messages.iter().map(|m| m.state()), None);
    let before_set: BTreeSet<_> = before.iter().collect();
    let after_set: BTreeSet<_> = after.iter().collect();
    // receivedAt and IDs are immutable, so retained items preserve relative order.
    let removed: Vec<_> = before.iter().filter(|id| !after_set.contains(id)).collect();
    let added: Vec<_> = after
        .iter()
        .enumerate()
        .filter(|(_, id)| !before_set.contains(id))
        .map(|(index, id)| json!({"id":id,"index":index}))
        .collect();
    if removed.len() + added.len() > max {
        return Err("tooManyChanges");
    }
    Ok(
        json!({"accountId":account.id,"oldQueryState":state,"newQueryState":query.state(account.revision),
        "removed":removed,"added":added,"total":after.len()}),
    )
}
