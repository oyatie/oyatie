use super::filter::{Filter, MAX_SCAN_BYTES};
use super::filter_message::{Candidate, Content, ThreadKeywords, thread_id};
use super::sort::Sort;
use mail_kernel::{Account, Message, MessageState};
use mail_service::MailService;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

struct Query {
    filter: Filter,
    sorts: Vec<Sort>,
    collapse: bool,
    fingerprint: String,
}

struct Entry {
    state: MessageState,
    size: Option<usize>,
    content: Option<Content>,
}
impl Entry {
    fn candidate<'a>(&'a self, threads: Option<&'a ThreadKeywords>) -> Candidate<'a> {
        Candidate {
            state: &self.state,
            size: self.size,
            content: self.content.as_ref(),
            threads,
        }
    }
}

impl Query {
    fn parse(account: &str, args: &Value) -> Result<Self, &'static str> {
        let filter = Filter::parse(&args["filter"])?;
        let sorts = Sort::parse(&args["sort"])?;
        let collapse = match &args["collapseThreads"] {
            Value::Null => false,
            Value::Bool(collapse) => *collapse,
            _ => return Err("invalidArguments"),
        };
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(
                json!([account, args["filter"], Sort::canonical(&sorts), collapse]).to_string()
            )
        );
        Ok(Self {
            filter,
            sorts,
            collapse,
            fingerprint,
        })
    }

    /// True when filtering or sorting needs the message bytes.
    fn content(&self) -> bool {
        self.filter.content() || Sort::content(&self.sorts)
    }

    /// True when both query states can be rebuilt from stored metadata with
    /// retained items keeping their relative order (Email/queryChanges).
    fn stateful(&self) -> bool {
        !self.content() && !self.filter.sized() && !self.collapse && Sort::stable(&self.sorts)
    }

    fn order(&self, mut entries: Vec<Entry>, threads: Option<&ThreadKeywords>) -> Vec<String> {
        entries.sort_by(|a, b| {
            Sort::compare(&self.sorts, &a.candidate(threads), &b.candidate(threads))
        });
        if self.collapse {
            // The first message of each thread in sort order represents it.
            let mut seen = BTreeSet::new();
            entries.retain(|e| seen.insert(thread_id(&e.state).to_owned()));
        }
        entries.into_iter().map(|e| e.state.id).collect()
    }

    fn ids(&self, states: impl IntoIterator<Item = MessageState>) -> Vec<String> {
        let states: Vec<_> = states.into_iter().collect();
        let threads = self
            .filter
            .threaded()
            .then(|| ThreadKeywords::build(&states));
        let entries = states
            .into_iter()
            .map(|state| Entry {
                state,
                size: None,
                content: None,
            })
            .filter(|e| self.filter.matches(&e.candidate(threads.as_ref())))
            .collect();
        self.order(entries, threads.as_ref())
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
    if query.content() && account.messages.len() > 10000 {
        return Err("limit");
    }
    let states: Vec<_> = account.messages.iter().map(Message::state).collect();
    let threads = query
        .filter
        .threaded()
        .then(|| ThreadKeywords::build(&states));
    let mut remaining = MAX_SCAN_BYTES;
    let mut entries = Vec::new();
    for (message, state) in account.messages.iter().zip(states) {
        let mut entry = Entry {
            state,
            size: Some(message.size),
            content: None,
        };
        // Metadata-only filters settle membership before any body is read;
        // content filters must scan every message within the byte budget.
        if !query.filter.content() && !query.filter.matches(&entry.candidate(threads.as_ref())) {
            continue;
        }
        if query.content() {
            query.filter.charge(message.size, &mut remaining)?;
            let raw = service
                .download(token, &account.id, &message.id)
                .map_err(super::method::error)?;
            entry.content = Some(Content::parse(&raw)?);
            if query.filter.content() && !query.filter.matches(&entry.candidate(threads.as_ref())) {
                continue;
            }
        }
        entries.push(entry);
    }
    let ids = query.order(entries, threads.as_ref());
    let (position, limit) = page(&ids, args)?;
    Ok(
        json!({"accountId":account.id,"queryState":query.state(account.revision),"canCalculateChanges":query.stateful(),
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
    if !query.stateful() {
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
    let before = query.ids(old.into_values());
    let after = query.ids(account.messages.iter().map(Message::state));
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
