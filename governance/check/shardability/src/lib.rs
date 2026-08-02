//! Shardability check (ADR-0062 §"sharded state"; decision-principles.json DP-09).
//!
//! Every per-tenant DB table MUST declare `tenant_id` so the data can be
//! row-level-secured + per-cell sharded per the Postgres+Citus topology
//! (Bominal ADR-0117 stage 2 inherited). Tables that are global (registries
//! shared across tenants) opt out via an explicit `-- shardability: global`
//! comment on the line preceding the `CREATE TABLE`.
//!
//! Scope of this kernel: pure logic over typed [`MigrationFile`] nodes
//! pre-harvested by a runner. Detection is a lightweight regex-free text
//! scan: find `CREATE TABLE` statements, look for `tenant_id` column or
//! global opt-out marker, flag the rest.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// Execution mode for the shardability check lane.
///
/// `ReportOnly` prints violations but always returns success (exit 0) so early
/// substrate phases can track drift without blocking CI.  `Blocker` causes the
/// check to return a non-zero exit code when any violation is found; P22 flips
/// the lane to this mode once all known violations are resolved.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationFile {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub content: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub path: String,
    pub table_name: String,
    pub line: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub files_checked: usize,
    pub tables_seen: usize,
    pub tables_global: usize,
    pub violations: Vec<Violation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    EmptyPath,
    DuplicatePath { path: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "migration file with empty path"),
            Self::DuplicatePath { path } => write!(f, "duplicate migration file path: {path}"),
        }
    }
}

impl std::error::Error for Error {}

const GLOBAL_OPT_OUT_MARKER: &str = "shardability: global";

fn extract_table_name(stmt: &str) -> Option<String> {
    // Find "CREATE TABLE" then take the next identifier (skipping IF NOT EXISTS).
    let upper = stmt.to_ascii_uppercase();
    let create_pos = upper.find("CREATE TABLE")?;
    let mut tail = &stmt[create_pos + "CREATE TABLE".len()..];
    let upper_tail = tail.to_ascii_uppercase();
    if upper_tail.trim_start().starts_with("IF NOT EXISTS") {
        let after = upper_tail
            .find("IF NOT EXISTS")
            .map(|p| p + "IF NOT EXISTS".len())
            .unwrap_or(0);
        tail = &tail[after..];
    }
    let identifier: String = tail
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| {
            c.is_ascii_alphanumeric() || *c == '_' || *c == '.' || *c == '"' || *c == '`'
        })
        .collect();
    let cleaned: String = identifier
        .chars()
        .filter(|c| *c != '"' && *c != '`')
        .collect();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

pub fn check(files: &[MigrationFile]) -> Result<Report, Error> {
    let mut seen_paths = BTreeSet::new();
    let mut violations = Vec::new();
    let mut tables_seen = 0usize;
    let mut tables_global = 0usize;

    for file in files {
        if file.path.trim().is_empty() {
            return Err(Error::EmptyPath);
        }
        if !seen_paths.insert(file.path.clone()) {
            return Err(Error::DuplicatePath {
                path: file.path.clone(),
            });
        }

        let lines: Vec<&str> = file.content.lines().collect();
        let mut i = 0usize;
        while i < lines.len() {
            let line = lines[i];
            let upper = line.to_ascii_uppercase();
            if upper.contains("CREATE TABLE") {
                tables_seen += 1;

                // Check the preceding non-blank line for the global opt-out comment.
                let mut prev_idx = i;
                let mut is_global = false;
                while prev_idx > 0 {
                    prev_idx -= 1;
                    let prev = lines[prev_idx].trim();
                    if prev.is_empty() {
                        continue;
                    }
                    is_global = prev.contains(GLOBAL_OPT_OUT_MARKER);
                    break;
                }
                if is_global {
                    tables_global += 1;
                    i += 1;
                    continue;
                }

                // Collect the CREATE TABLE statement until matching `);`.
                let mut stmt = String::new();
                let mut j = i;
                while j < lines.len() {
                    stmt.push_str(lines[j]);
                    stmt.push('\n');
                    let trimmed = lines[j].trim();
                    if trimmed.ends_with(");") || trimmed.ends_with(")") {
                        break;
                    }
                    j += 1;
                }

                // tenant_id present? Match case-insensitively, must be a word boundary
                // so `not_tenant_id_extra` doesn't false-positive.
                let stmt_lower = stmt.to_ascii_lowercase();
                let has_tenant_id = stmt_lower
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .any(|tok| tok == "tenant_id");

                if !has_tenant_id {
                    let table_name = extract_table_name(&stmt)
                        .unwrap_or_else(|| format!("<unparseable at line {}>", i + 1));
                    violations.push(Violation {
                        path: file.path.clone(),
                        table_name,
                        line: (i + 1) as u32,
                    });
                }

                i = j + 1;
                continue;
            }
            i += 1;
        }
    }

    Ok(Report {
        files_checked: files.len(),
        tables_seen,
        tables_global,
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mf(path: &str, content: &str) -> MigrationFile {
        MigrationFile {
            path: path.into(),
            content: content.into(),
        }
    }

    #[test]
    fn empty_input_passes() {
        let r = check(&[]).unwrap();
        assert_eq!(r.tables_seen, 0);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn tenant_id_present_passes() {
        let r = check(&[mf(
            "migrations/001_users.sql",
            "CREATE TABLE users (\n  tenant_id UUID NOT NULL,\n  id UUID PRIMARY KEY\n);\n",
        )])
        .unwrap();
        assert_eq!(r.tables_seen, 1);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn missing_tenant_id_flagged() {
        let r = check(&[mf(
            "migrations/002_orders.sql",
            "CREATE TABLE orders (\n  id UUID PRIMARY KEY,\n  amount BIGINT\n);\n",
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].table_name, "orders");
    }

    #[test]
    fn global_opt_out_marker_excuses_table() {
        let r = check(&[mf(
            "migrations/003_currency_codes.sql",
            "-- shardability: global (ISO 4217 reference table; shared across tenants)\n\
             CREATE TABLE currency_codes (\n  code CHAR(3) PRIMARY KEY\n);\n",
        )])
        .unwrap();
        assert_eq!(r.tables_seen, 1);
        assert_eq!(r.tables_global, 1);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn if_not_exists_table_handled() {
        let r = check(&[mf(
            "migrations/004_invoices.sql",
            "CREATE TABLE IF NOT EXISTS invoices (\n  id UUID PRIMARY KEY\n);\n",
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].table_name, "invoices");
    }

    #[test]
    fn no_word_boundary_false_positive() {
        // "not_tenant_id_extra" should NOT count as tenant_id.
        let r = check(&[mf(
            "migrations/005_t.sql",
            "CREATE TABLE t (\n  not_tenant_id_extra TEXT\n);\n",
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
    }

    #[test]
    fn multiple_tables_in_one_file() {
        let r = check(&[mf(
            "migrations/006_multi.sql",
            "CREATE TABLE a (id UUID, tenant_id UUID);\n\
             CREATE TABLE b (id UUID);\n",
        )])
        .unwrap();
        assert_eq!(r.tables_seen, 2);
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].table_name, "b");
    }

    #[test]
    fn rejects_empty_path() {
        let err = check(&[mf("", "")]).unwrap_err();
        assert_eq!(err, Error::EmptyPath);
    }

    #[test]
    fn rejects_duplicate_path() {
        let err = check(&[mf("a.sql", ""), mf("a.sql", "")]).unwrap_err();
        assert!(matches!(err, Error::DuplicatePath { .. }));
    }



}
