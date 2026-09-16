use super::{storage, submission_history as history};
use mail_api::{SubmissionFailure, SubmissionPage};
use mail_kernel::{Error, SubmissionFilter, SubmissionQuery, SubmissionSortField};
use rusqlite::{Connection, params_from_iter, types::Value};

pub(super) fn query(
    db: &Connection,
    account: &str,
    revision: Option<u64>,
    query: &SubmissionQuery,
) -> Result<SubmissionPage, SubmissionFailure> {
    if query.limit > 10000 || query.sort.len() > 3 {
        return Err(Error::OverQuota.into());
    }
    let latest = history::revision(db, account)?;
    let revision = revision.unwrap_or(latest);
    let floor = history::floor(db, account)?;
    if revision > latest || revision < floor {
        return Err(Error::Conflict.into());
    }
    if revision > floor {
        let exists: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM submission_versions WHERE account=?1 AND revision=?2)",
                rusqlite::params![account, revision],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if !exists {
            return Err(Error::Conflict.into());
        }
    }
    let mut values = vec![Value::Text(account.into()), Value::Integer(revision as i64)];
    let filter = filter(&query.filter, &mut values, 0, &mut 0)?;
    let predicate = format!(
        "account=?1 AND revision<=?2 AND (until_revision IS NULL OR until_revision>?2) AND state IS NOT NULL AND ({filter})"
    );
    let total: usize = db
        .query_row(
            &format!("SELECT count(*) FROM submission_versions WHERE {predicate}"),
            params_from_iter(values.iter()),
            |r| r.get(0),
        )
        .map_err(storage)?;
    if total > 10000 {
        return Err(Error::OverQuota.into());
    }
    let mut sort = Vec::new();
    for (field, ascending) in &query.sort {
        let field = match field {
            SubmissionSortField::EmailId => "email_id",
            SubmissionSortField::ThreadId => "thread_id",
            SubmissionSortField::SentAt => "send_at",
        };
        sort.push(format!(
            "{field} {}",
            if *ascending { "ASC" } else { "DESC" }
        ));
    }
    if sort.is_empty() {
        sort.push("send_at ASC".into());
    }
    sort.push("id ASC".into());
    let sort = sort.join(",");
    let position = if let Some(anchor) = &query.anchor {
        if anchor.len() > 256 {
            return Err(Error::Invalid.into());
        }
        let mut stmt = db
            .prepare(&format!(
                "SELECT id FROM submission_versions WHERE {predicate} ORDER BY {sort} LIMIT 10000"
            ))
            .map_err(storage)?;
        let mut position = None;
        for (index, id) in stmt
            .query_map(params_from_iter(values.iter()), |r| r.get::<_, String>(0))
            .map_err(storage)?
            .enumerate()
        {
            if id.map_err(storage)? == *anchor {
                position = Some(index as i64);
                break;
            }
        }
        position
            .ok_or(SubmissionFailure::AnchorNotFound)?
            .saturating_add(query.anchor_offset)
            .max(0) as usize
    } else if query.position < 0 {
        (total as i64).saturating_add(query.position).max(0) as usize
    } else {
        query.position as usize
    };
    let mut stmt=db.prepare(&format!("SELECT id FROM submission_versions WHERE {predicate} ORDER BY {sort} LIMIT {} OFFSET {position}",query.limit)).map_err(storage)?;
    let ids = stmt
        .query_map(params_from_iter(values.iter()), |r| r.get(0))
        .map_err(storage)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage)?;
    let position = if query.anchor.is_none() && position > total {
        0
    } else {
        position
    };
    Ok(SubmissionPage {
        revision,
        ids,
        position,
        total,
    })
}

fn filter(
    value: &SubmissionFilter,
    values: &mut Vec<Value>,
    depth: usize,
    nodes: &mut usize,
) -> Result<String, Error> {
    *nodes += 1;
    if depth > 16 || *nodes > 256 || values.len() > 512 {
        return Err(Error::OverQuota);
    }
    Ok(match value {
        SubmissionFilter::All => "1".into(),
        SubmissionFilter::And(parts)
        | SubmissionFilter::Or(parts)
        | SubmissionFilter::Not(parts) => {
            if parts.is_empty() {
                return Err(Error::Invalid);
            }
            let parts = parts
                .iter()
                .map(|v| filter(v, values, depth + 1, nodes))
                .collect::<Result<Vec<_>, _>>()?;
            let is_or = matches!(value, SubmissionFilter::Or(_) | SubmissionFilter::Not(_));
            let expression = format!("({})", parts.join(if is_or { ") OR (" } else { ") AND (" }));
            if matches!(value, SubmissionFilter::Not(_)) {
                format!("NOT ({expression})")
            } else {
                expression
            }
        }
        SubmissionFilter::IdentityIds(ids)
        | SubmissionFilter::EmailIds(ids)
        | SubmissionFilter::ThreadIds(ids) => {
            if ids.len() > 256
                || values.len() + ids.len() > 512
                || ids.iter().any(|id| id.len() > 256)
            {
                return Err(Error::OverQuota);
            }
            let column = match value {
                SubmissionFilter::IdentityIds(_) => "identity_id",
                SubmissionFilter::EmailIds(_) => "email_id",
                _ => "thread_id",
            };
            if ids.is_empty() {
                "0".into()
            } else {
                let mut parameters = Vec::new();
                for id in ids {
                    values.push(Value::Text(id.clone()));
                    parameters.push(format!("?{}", values.len()));
                }
                format!("{column} IN ({})", parameters.join(","))
            }
        }
        SubmissionFilter::UndoStatus(status) => {
            values.push(Value::Text(history::status(*status).into()));
            format!("undo=?{}", values.len())
        }
        SubmissionFilter::Before(time) | SubmissionFilter::After(time) => {
            values.push(Value::Integer(*time));
            format!(
                "send_at {} ?{}",
                if matches!(value, SubmissionFilter::Before(_)) {
                    "<"
                } else {
                    ">"
                },
                values.len()
            )
        }
    })
}
