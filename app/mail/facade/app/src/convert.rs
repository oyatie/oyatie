//! `mail-app convert DATABASE --backup-verified PATH | --backup-into PATH`:
//! one exclusive lock from open to the audit line; every refusal exits
//! non-zero with the converter's own message.
use mail_sqlite_store::convert::Converter;
use std::{path::Path, time::Duration};

pub(super) fn run(
    database: &str,
    mode: &str,
    backup: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let database = Path::new(database);
    let backup = Path::new(backup);
    let operator = operator();
    let converter = Converter::open(database)?;
    let conversion = match mode {
        "--backup-verified" => converter.convert(backup, &operator)?,
        "--backup-into" => {
            converter.backup_into(backup)?;
            converter.convert(backup, &operator)?
        }
        other => return Err(format!("unknown convert option {other}").into()),
    };
    println!(
        "{}",
        audit(
            conversion.accounts,
            conversion.elapsed,
            &conversion.backup_path,
            &conversion.backup_sha256,
            &operator
        )
    );
    Ok(())
}

/// One line the operator can file: what changed, how long, and the backup
/// that restores it.
fn audit(accounts: u64, elapsed: Duration, backup: &Path, sha256: &str, operator: &str) -> String {
    format!(
        "mail-app: converted {accounts} accounts in {:.3} s; backup {} sha256 {sha256}; operator {operator}",
        elapsed.as_secs_f64(),
        backup.display(),
    )
}

/// `user@host` recorded in `schema_version.operator`; std only, so the host
/// comes from the environment, `/etc/hostname`, or the `hostname` tool.
fn operator() -> String {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| "unknown".into());
    let host = std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
        })
        .map(|h| h.trim().to_owned())
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "unknown-host".into());
    format!("{user}@{host}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_line_names_accounts_elapsed_backup_and_digest() {
        let line = audit(
            3,
            Duration::from_millis(1500),
            Path::new("/var/backups/mail.sqlite"),
            &"ab".repeat(32),
            "alice@mail-1",
        );
        assert_eq!(
            line,
            format!(
                "mail-app: converted 3 accounts in 1.500 s; backup /var/backups/mail.sqlite sha256 {}; operator alice@mail-1",
                "ab".repeat(32)
            )
        );
    }

    #[test]
    fn operator_is_user_at_host() {
        let operator = operator();
        let (user, host) = operator.split_once('@').unwrap();
        assert!(!user.is_empty() && !host.is_empty(), "{operator}");
        assert!(!operator.contains('\n'));
    }
}
