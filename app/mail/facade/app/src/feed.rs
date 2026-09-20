//! `ChangeFeed` operator surface: which consumers this cell enables, and the
//! recovery subcommands, each an audited row plus one structured log line.
use mail_api::{ChangeFeed, Consumer, MetadataStore};
use mail_sqlite_store::SqliteStore;
use std::error::Error;

/// `MAIL_CONSUMERS`: comma-separated consumer names; empty by default.
pub(super) fn consumers() -> Result<Vec<Consumer>, Box<dyn Error>> {
    std::env::var("MAIL_CONSUMERS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(consumer)
        .collect()
}

fn consumer(name: &str) -> Result<Consumer, Box<dyn Error>> {
    Consumer::ALL
        .into_iter()
        .find(|c| c.name() == name)
        .ok_or_else(|| {
            let known: Vec<_> = Consumer::ALL.iter().map(|c| c.name()).collect();
            format!("unknown consumer {name:?}; known: {}", known.join(", ")).into()
        })
}

pub(super) const USAGE: &str = "mail-app reconcile DATABASE CONSUMER REASON; mail-app dead-letter DATABASE list CONSUMER | DATABASE CONSUMER TENANT ACCOUNT REVISION REASON; mail-app cursor retire DATABASE CONSUMER [TENANT] REASON; mail-app audit DATABASE [LIMIT]";

/// `args` starts at the subcommand word; `Ok(false)` means not a feed command.
pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let operator = super::convert::operator;
    match words.as_slice() {
        ["reconcile", database, consumer_name, reason] => {
            let consumer = consumer(consumer_name)?;
            let operator = operator();
            let marked = SqliteStore::open(database)?.reconcile(consumer, &operator, reason)?;
            eprintln!(
                "mail-app: event=reconcile consumer={consumer_name} re-marked={marked} operator={operator}"
            );
        }
        ["dead-letter", database, "list", consumer_name] => {
            for (dirty, reason) in
                SqliteStore::open(database)?.poisoned(consumer(consumer_name)?)?
            {
                println!("{} {} {reason}", dirty.tenant, dirty.account);
            }
        }
        [
            "dead-letter",
            database,
            consumer_name,
            tenant,
            account,
            revision,
            reason,
        ] => {
            let consumer = consumer(consumer_name)?;
            let store = SqliteStore::open(database)?;
            // The tenant is immutable per account; naming it guards a typo.
            if store.account_info(account)?.tenant != *tenant {
                return Err(format!("account {account} is not in tenant {tenant}").into());
            }
            let revision: u64 = revision.parse()?;
            let operator = operator();
            store.dead_letter(consumer, account, revision, &operator, reason)?;
            eprintln!(
                "mail-app: event=dead-letter consumer={consumer_name} tenant={tenant} account={account} revision={revision} operator={operator}"
            );
        }
        ["cursor", "retire", database, consumer_name, rest @ ..]
            if !rest.is_empty() && rest.len() <= 2 =>
        {
            let consumer = consumer(consumer_name)?;
            let (tenant, reason) = match rest {
                [tenant, reason] => (Some(*tenant), *reason),
                [reason] => (None, *reason),
                _ => unreachable!(),
            };
            let operator = operator();
            let retired =
                SqliteStore::open(database)?.retire_cursor(consumer, tenant, &operator, reason)?;
            eprintln!(
                "mail-app: event=cursor-retired consumer={consumer_name} tenant={} retired={retired} operator={operator}",
                tenant.unwrap_or("*")
            );
        }
        ["audit", database, rest @ ..] if rest.len() <= 1 => {
            let limit = rest
                .first()
                .map_or(Ok(100), |n| n.parse())
                .map_err(|_| "LIMIT must be a whole number")?;
            for row in SqliteStore::open(database)?.audit(limit)? {
                println!(
                    "{} {} {} {} {} {}",
                    row.id, row.at_utc, row.kind, row.operator, row.reason, row.detail
                );
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn consumer_names_round_trip_and_unknown_names_are_refused() {
        assert_eq!(
            super::consumer("foundry-records").unwrap(),
            mail_api::Consumer::FoundryRecords
        );
        assert!(super::consumer("records").is_err());
    }
}
