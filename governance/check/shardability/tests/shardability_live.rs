// governance-check-shardability LIVE-TREE gate (ADR-0062 "sharded state"; DP-09).
//
// The `#[cfg(test)] mod tests` inside src/lib.rs proves the kernel correct on hand-written
// migration text. It says nothing about this repository, and until this file existed nothing did:
// the crate's only Cargo consumer was marketplace/facade/dev-cli, which no workflow invokes, so
// every case the doctrine had ever run was a fixture. Those fixture tests stay exactly where they
// are — this target is ADDED beside them, never in place of them.
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Walk failures are ERRORS, never omitted observations: a migration dropped from the
// census because its contents failed to read would quietly shrink the frozen map, and a shrink
// reads as repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_shardability::{MigrationFile, Report, check};

const POLICY_PATH: &str = "governance/check/shardability/shardability-policy.json";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

struct Policy {
    min_tracked_files: usize,
    min_sql_files: usize,
    min_tables_seen: usize,
    frozen_unsharded_tables: BTreeMap<String, usize>,
}

struct Observed {
    report: Report,
    tracked_files: usize,
    /// `<file>::<table>` -> how many `CREATE TABLE` statements in that file declare that table
    /// without a `tenant_id` column and without the `-- shardability: global` opt-out.
    ///
    /// Deliberately NOT line-anchored. The kernel reports a line number, but a line number moves
    /// whenever anything above it is edited, so a line-keyed baseline forces a blind re-freeze on
    /// edits that changed nothing about shardability — and an edit above a guarded statement can
    /// leave no legal edit at all. Deliberately NOT a bare set either: a migration that declares
    /// the same table name twice (a rewrite appended below the original, say) would be invisible
    /// to a set, so the multiplicity is carried as the value.
    unsharded_tables: BTreeMap<String, usize>,
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {POLICY_PATH})");
}

fn load_policy(root: &Path) -> Policy {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let number = |key: &str| -> usize {
        usize::try_from(
            doc[key]
                .as_u64()
                .unwrap_or_else(|| panic!("policy field {key} missing or not a number")),
        )
        .expect("policy number fits usize")
    };
    let frozen = doc["frozen_unsharded_tables"]
        .as_object()
        .expect("policy field frozen_unsharded_tables missing or not an object")
        .iter()
        .map(|(key, value)| {
            let count = usize::try_from(
                value
                    .as_u64()
                    .unwrap_or_else(|| panic!("frozen_unsharded_tables[{key}] is not a number")),
            )
            .expect("count fits usize");
            (key.clone(), count)
        })
        .collect();
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_sql_files: number("min_sql_files"),
        min_tables_seen: number("min_tables_seen"),
        frozen_unsharded_tables: frozen,
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
///
/// Walking the working tree instead would measure a different corpus than CI does the moment an
/// ignored `*.sql` exists on disk (a dumped schema, a scratch migration), and with the map pinned
/// by equality that is a red gate CI cannot reproduce.
fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| format!("git ls-files failed to start: {e}"))?;
    if !out.status.success() {
        return Err(format!("git ls-files exited with {}", out.status));
    }
    let text =
        String::from_utf8(out.stdout).map_err(|e| format!("git ls-files output not UTF-8: {e}"))?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect())
}

fn read_tracked(root: &Path, relative: &str) -> Result<Option<String>, String> {
    let path = root.join(relative);
    // Every failure below is an ERROR, never an omitted observation.
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("metadata {relative} failed: {e} (tracked but unreadable)"))?;
    if !metadata.is_file() {
        return Ok(None); // a tracked symlink to a directory carries no DDL
    }
    if metadata.len() > MAX_SCANNED_BYTES {
        return Err(format!(
            "{relative} is {} bytes, over the {MAX_SCANNED_BYTES}-byte scan cap — raise the cap \
             deliberately rather than dropping the migration from the census",
            metadata.len()
        ));
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("read {relative} failed: {e}"))?;
    // LOSSY, never skipped: a non-UTF-8 byte in a comment still leaves the DDL keywords intact.
    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let mut migrations: Vec<MigrationFile> = Vec::new();
    for relative in &tracked {
        if !relative.ends_with(".sql") {
            continue;
        }
        let Some(content) = read_tracked(root, relative)? else {
            continue;
        };
        migrations.push(MigrationFile {
            path: relative.clone(),
            content,
        });
    }

    let report = check(&migrations).map_err(|e| format!("kernel refused the live corpus: {e}"))?;

    let mut unsharded_tables: BTreeMap<String, usize> = BTreeMap::new();
    for violation in &report.violations {
        *unsharded_tables
            .entry(format!("{}::{}", violation.path, violation.table_name))
            .or_default() += 1;
    }

    Ok(Observed {
        report,
        tracked_files: tracked.len(),
        unsharded_tables,
    })
}

/// The live walk, done ONCE for the whole binary: it is a pure function of the tree, so every test
/// re-walking it would recompute the same answer over the whole tracked file list.
fn live() -> &'static (Policy, Observed) {
    static LIVE: OnceLock<(Policy, Observed)> = OnceLock::new();
    LIVE.get_or_init(|| {
        let root = repo_root();
        let policy = load_policy(&root);
        let observed = observe(&root).expect("live walk");
        (policy, observed)
    })
}

fn census(observed: &Observed) -> String {
    let mut out = format!(
        "census: {} tracked *.sql migrations over {} tracked files; {} CREATE TABLE statements, {} \
         opted out as `-- shardability: global`, {} unsharded across {} (file, table) keys\n",
        observed.report.files_checked,
        observed.tracked_files,
        observed.report.tables_seen,
        observed.report.tables_global,
        observed.report.violations.len(),
        observed.unsharded_tables.len(),
    );
    for violation in &observed.report.violations {
        out.push_str(&format!(
            "  {}:{} {}\n",
            violation.path, violation.line, violation.table_name
        ));
    }
    out
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// A ratchet pinned by equality cannot distinguish "the schema was repaired" from "the walk
/// collapsed"; both drive the observed map toward empty. These floors are the machine oracle that
/// separates them. Every floor counts SUBJECT MATERIAL — tracked paths, migration files, CREATE
/// TABLE statements — never findings, so adding `tenant_id` to a table moves the frozen map and
/// leaves all three floors exactly where they are. No floor here can red on honest progress.
///
/// There is deliberately NO floor on `tables_global` (the opt-out count) and none on the violation
/// count. Both have zero as a legitimate target, and a floor on a term whose target is zero reds
/// precisely when the work succeeds, which gets the guard deleted rather than the corpus fixed.
#[test]
fn the_migration_corpus_is_intact() {
    let (policy, observed) = live();
    assert!(
        observed.tracked_files >= policy.min_tracked_files,
        "git ls-files returned {} tracked paths, below the floor of {} — the corpus walk is broken \
         and every count below is meaningless\n{}",
        observed.tracked_files,
        policy.min_tracked_files,
        census(observed)
    );
    assert!(
        observed.report.files_checked >= policy.min_sql_files,
        "{} tracked *.sql migrations found, below the floor of {}. Schema does not disappear in \
         bulk; a drop here is a narrowed scan, and a narrowed scan reports a perfectly sharded \
         database it never read\n{}",
        observed.report.files_checked,
        policy.min_sql_files,
        census(observed)
    );
    assert!(
        observed.report.tables_seen >= policy.min_tables_seen,
        "{} CREATE TABLE statements parsed, below the floor of {} — the DDL scan stopped matching \
         the corpus it governs, so its verdict is not evidence\n{}",
        observed.report.tables_seen,
        policy.min_tables_seen,
        census(observed)
    );
}

/// THE GATE: a SHRINK-ONLY, TWO-SIDED ratchet on the MAP of `(file, table)` unsharded tables.
///
/// Keys, not a count. ADR-0062 requires every per-tenant table to carry `tenant_id` so rows can be
/// RLS-scoped and Citus-sharded per cell; a table that is genuinely cross-tenant opts out with an
/// explicit `-- shardability: global` comment on the line above its `CREATE TABLE`. A count would
/// tell a reviewer that the number moved and nothing about which table moved; `<file>::<table>`
/// names the migration and the relation, and is reviewable on its face — small enough here that a
/// reviewer can open every entry.
///
/// TWO-SIDED, over the UNION of both key sets. A new unsharded table appears above its pin and
/// blocks; a repaired one falls below its pin and ALSO blocks, forcing the pin down in the same
/// change so the win is recorded. The union matters: iterating only the frozen keys makes a NEW
/// key invisible, which is the same hole as an unratcheted count, one level down.
///
/// WHAT THIS DOES NOT DETECT, stated at the strength the mechanism has: renaming an unsharded
/// table in place swaps one key for another and reads as one repair plus one regression, which is
/// exactly what it is but says nothing about whether the rename fixed anything. And the kernel is
/// a text scanner, not a SQL parser — a table created by `EXECUTE format(...)` or by an ORM
/// migration written in Rust is outside this subject entirely.
#[test]
fn unsharded_tables_equal_the_frozen_map() {
    let (policy, observed) = live();

    let keys: BTreeMap<&String, ()> = policy
        .frozen_unsharded_tables
        .keys()
        .chain(observed.unsharded_tables.keys())
        .map(|key| (key, ()))
        .collect();
    let drift: Vec<String> = keys
        .into_keys()
        .filter_map(|key| {
            let seen = observed.unsharded_tables.get(key).copied().unwrap_or(0);
            let want = policy
                .frozen_unsharded_tables
                .get(key)
                .copied()
                .unwrap_or(0);
            (seen != want).then(|| format!("  {key}: observed {seen}, frozen {want}"))
        })
        .collect();

    assert!(
        drift.is_empty(),
        "shardability drift, per (file, table). ABOVE the pin: a CREATE TABLE declares no \
         `tenant_id`, so its rows cannot be RLS-scoped or per-cell sharded — add the column, or, \
         if the relation really is cross-tenant, put `-- shardability: global (<reason>)` on the \
         line above the statement. BELOW the pin: lower `frozen_unsharded_tables` in THIS change \
         so the win is recorded, or discover that the scan narrowed and is reporting green over \
         migrations it stopped reading. Re-derive by RUNNING this gate and reading 'observed N' \
         from these lines; never by arithmetic on the old values:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against REAL corpus text, not just fixtures.
///
/// A green ratchet proves nothing on its own: a caller that silently produced zero findings would
/// satisfy every assertion above by reporting a perfectly sharded schema. This appends a genuine
/// `CREATE TABLE` with no `tenant_id` to a copy of a REAL tracked migration's text and asserts the
/// kernel reddens — and asserts the new violation NAMES THE PLANTED TABLE, so the test cannot pass
/// on some pre-existing finding that happened to be there already. It then re-runs the identical
/// statement carrying the `-- shardability: global` marker and asserts it does not fire, so the
/// documented opt-out is proven to be a real escape hatch rather than a rule nobody can satisfy.
#[test]
fn injecting_an_unsharded_table_into_a_real_migration_reddens_the_gate() {
    let root = repo_root();
    let tracked = tracked_files(&root).expect("git ls-files");

    let subject = tracked
        .iter()
        .find(|relative| relative.ends_with(".sql"))
        .expect("no tracked *.sql migration exists; this gate has no subject");
    let body = read_tracked(&root, subject)
        .expect("read migration")
        .expect("migration is a file");

    let baseline = check(&[MigrationFile {
        path: subject.clone(),
        content: body.clone(),
    }])
    .expect("kernel accepts the unmodified migration");

    // The probe table name is unique to this test, so the assertion below cannot be satisfied by
    // any table that already exists in the corpus.
    const PROBE: &str = "shardability_live_probe_relation";
    let planted = format!(
        "{body}\nCREATE TABLE {PROBE} (\n  probe_id UUID PRIMARY KEY,\n  payload TEXT NOT NULL\n);\n"
    );
    let reddened = check(&[MigrationFile {
        path: subject.clone(),
        content: planted,
    }])
    .expect("kernel accepts the planted migration");

    assert_eq!(
        reddened.violations.len(),
        baseline.violations.len() + 1,
        "a CREATE TABLE with no tenant_id appended to the live migration {subject} did not raise \
         the violation count"
    );
    assert!(
        reddened
            .violations
            .iter()
            .any(|violation| violation.table_name == PROBE),
        "the violation count rose but no finding NAMES the planted table {PROBE}; the gate \
         reddened for some other reason and this proves nothing about the plant"
    );

    let opted_out = format!(
        "{body}\n-- shardability: global (probe; cross-tenant by construction)\nCREATE TABLE \
         {PROBE} (\n  probe_id UUID PRIMARY KEY,\n  payload TEXT NOT NULL\n);\n"
    );
    let suppressed = check(&[MigrationFile {
        path: subject.clone(),
        content: opted_out,
    }])
    .expect("kernel accepts the opted-out migration");
    assert_eq!(
        suppressed.violations.len(),
        baseline.violations.len(),
        "the `-- shardability: global` marker failed to excuse the identical statement on live \
         migration text, so the frozen map is pinned against a rule that has no usable opt-out"
    );
    assert_eq!(
        suppressed.tables_global,
        baseline.tables_global + 1,
        "the opted-out table was not counted as a global opt-out; it was silently dropped from \
         the census instead, which is a hole, not an exemption"
    );

    println!(
        "mutation proof: {subject} {} violations -> {} with an unsharded {PROBE} planted, back to \
         {} when the same statement carries `-- shardability: global`",
        baseline.violations.len(),
        reddened.violations.len(),
        suppressed.violations.len()
    );
}

/// Evidence, always printed, so a reader can tell a repaired corpus from a collapsed walk without
/// re-running anything.
#[test]
fn live_census_is_reported() {
    let (_, observed) = live();
    println!("{}", census(observed));
    assert!(observed.report.files_checked > 0);
}
