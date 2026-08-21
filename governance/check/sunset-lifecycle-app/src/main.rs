//! Foundry sunset-lifecycle fitness dev-CLI.
//!
//! Walks the repo for sunset clauses across three surfaces:
//!
//! 1. **ADR frontmatter** — `docs/decisions/*.md` files whose YAML
//!    frontmatter carries `sunset_at` / `sunset_milestone` /
//!    `deprecation_at` / `removal_at` / `sunset_topic` keys.
//! 2. **Spec JSON `_sunset` objects** — `specs/*.json`
//!    files containing a top-level `"_sunset": { ... }` member.
//! 3. **Cargo manifest sunset metadata** — `[package.metadata.oya.sunset]`
//!    section in any workspace crate's `Cargo.toml`.
//!
//! Discovered clauses are fed into the I/O-free
//! [`check_sunset_lifecycle_kernel::evaluate`] kernel
//! together with `now` (configurable via `--now YYYY-MM-DD`, defaults to
//! today UTC at startup) and the set of reached milestones (configurable
//! via repeated `--reached-milestone <id>`).
//!
//! Exits with code 0 when no violations; non-zero otherwise.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use check_sunset_lifecycle_kernel::{
    Date, LifecycleState, SunsetClause, Violation, evaluate,
};

const DEFAULT_ADR_ROOT: &str = "docs/decisions";
const DEFAULT_SPECS_ROOT: &str = "specs";
const DEFAULT_CRATES_ROOT: &str = "crates";
const DEFAULT_TOOLS_ROOT: &str = "tools";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = match Options::parse(&args) {
        Ok(o) => o,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    let clauses = match discover_all(&options) {
        Ok(c) => c,
        Err(message) => {
            eprintln!("sunset-lifecycle error: {message}");
            return ExitCode::FAILURE;
        }
    };
    let violations = evaluate(&clauses, options.now, &options.reached_milestones);
    print_report(&clauses, &violations);
    if violations.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

struct Options {
    adr_root: PathBuf,
    specs_root: PathBuf,
    crates_root: PathBuf,
    tools_root: PathBuf,
    now: Date,
    reached_milestones: Vec<String>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut adr_root = PathBuf::from(DEFAULT_ADR_ROOT);
        let mut specs_root = PathBuf::from(DEFAULT_SPECS_ROOT);
        let mut crates_root = PathBuf::from(DEFAULT_CRATES_ROOT);
        let mut tools_root = PathBuf::from(DEFAULT_TOOLS_ROOT);
        let mut now: Option<Date> = None;
        let mut reached_milestones: Vec<String> = Vec::new();

        let mut i = 0usize;
        while i < args.len() {
            match args[i].as_str() {
                "--adr-root" => {
                    i += 1;
                    adr_root = PathBuf::from(arg(args, i, "--adr-root")?);
                }
                "--specs-root" => {
                    i += 1;
                    specs_root = PathBuf::from(arg(args, i, "--specs-root")?);
                }
                "--crates-root" => {
                    i += 1;
                    crates_root = PathBuf::from(arg(args, i, "--crates-root")?);
                }
                "--tools-root" => {
                    i += 1;
                    tools_root = PathBuf::from(arg(args, i, "--tools-root")?);
                }
                "--now" => {
                    i += 1;
                    let value = arg(args, i, "--now")?;
                    now = Some(
                        Date::parse_iso(value)
                            .ok_or_else(|| format!("--now expects YYYY-MM-DD, got `{value}`"))?,
                    );
                }
                "--reached-milestone" => {
                    i += 1;
                    reached_milestones.push(arg(args, i, "--reached-milestone")?.to_string());
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument `{other}`\n{}", usage())),
            }
            i += 1;
        }
        let now = now.unwrap_or_else(today_utc);
        Ok(Self {
            adr_root,
            specs_root,
            crates_root,
            tools_root,
            now,
            reached_milestones,
        })
    }
}

fn arg<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a str, String> {
    args.get(i)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn usage() -> String {
    "usage: oya-governance-sunset-lifecycle-app \
     [--adr-root PATH] [--specs-root PATH] [--crates-root PATH] [--tools-root PATH] \
     [--now YYYY-MM-DD] [--reached-milestone <id>]..."
        .into()
}

/// Compute today's date in UTC from system time without pulling in chrono.
fn today_utc() -> Date {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    // ADR-0083 Tier 1: use the kernel's infallible `Date::epoch()` constant
    // constructor (sibling of `Date::new`) — encodes 1970-01-01 statically
    // without `.expect()`.
    Date::epoch().add_days(days)
}

fn discover_all(options: &Options) -> Result<Vec<SunsetClause>, String> {
    let mut out = Vec::new();
    out.extend(discover_adr(&options.adr_root)?);
    out.extend(discover_specs(&options.specs_root)?);
    out.extend(discover_cargo_metadata(&options.crates_root)?);
    out.extend(discover_cargo_metadata(&options.tools_root)?);
    Ok(out)
}

// ---------- ADR frontmatter discovery ----------

fn discover_adr(root: &Path) -> Result<Vec<SunsetClause>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries = fs::read_dir(root).map_err(|e| format!("read_dir({}): {e}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("entry under {}: {e}", root.display()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let contents =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let map = extract_yaml_frontmatter(&contents)
            .map(parse_yaml_flat_scalars)
            .unwrap_or_default();
        if !map_carries_sunset_signal(&map, &contents) {
            continue;
        }
        let fallback_topic = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed-sunset")
            .to_string();
        out.push(clause_from_map(
            &map,
            &path.display().to_string(),
            &fallback_topic,
        ));
    }
    Ok(out)
}

/// Strip the leading `---\n...\n---\n` block from a markdown file.
/// Returns `None` when no frontmatter is present.
fn extract_yaml_frontmatter(contents: &str) -> Option<&str> {
    let rest = contents.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Parse a flat key/value YAML block (no nesting, no lists). Sufficient
/// for ADR frontmatter sunset fields. Lines that don't fit `key: value`
/// shape are ignored.
fn parse_yaml_flat_scalars(block: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(|c: char| !c.is_whitespace()) {
            continue;
        }
        let Some(colon) = trimmed.find(':') else {
            continue;
        };
        let key = trimmed[..colon].trim().to_string();
        let value = trimmed[colon + 1..]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !key.is_empty() && !value.is_empty() {
            map.insert(key, value);
        }
    }
    map
}

fn map_carries_sunset_signal(map: &BTreeMap<String, String>, contents: &str) -> bool {
    if map.contains_key("sunset_at")
        || map.contains_key("sunset_milestone")
        || map.contains_key("sunset_topic")
        || map.contains_key("removal_at")
    {
        return true;
    }
    // Prose hint: a `sunset_note:` block or "sunset" in a status line
    // means the doc DECLARES a sunset intent but lacks machine-readable
    // fields — exactly the MissingFields lane target.
    if map.contains_key("sunset_note") {
        return true;
    }
    if contents.contains("\nsunset_note:") || contents.contains("\nsunset:") {
        return true;
    }
    // Body-level prose mentions (e.g. "sunset 2026-05-15", "sunset clause",
    // "scheduled to sunset", "deprecation lead time", "sunset window") are
    // intent declarations without machine-readable schema. The lane SHOULD
    // surface them as MISSING_FIELDS so authors upgrade them to the
    // ADR-0108 schema. Conservative substring match — false positives are
    // an acceptable cost given the lane's WARN-only baseline.
    has_body_sunset_prose(contents)
}

fn has_body_sunset_prose(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    lower.contains("scheduled to sunset")
        || lower.contains(" sunset clause")
        || lower.contains(" sunset window")
        || lower.contains(", sunset 20")
        || lower.contains("sunset 2026")
        || lower.contains("sunset 2027")
        || lower.contains("sunset 2028")
        || lower.contains("sunset_at:")
        || lower.contains("sunset_milestone:")
        || lower.contains("sunset date")
        || lower.contains("sunset-bounded")
}

fn clause_from_map(
    map: &BTreeMap<String, String>,
    location: &str,
    fallback_topic: &str,
) -> SunsetClause {
    let sunset_at = map.get("sunset_at").and_then(|v| Date::parse_iso(v));
    let deprecation_at = map.get("deprecation_at").and_then(|v| Date::parse_iso(v));
    let removal_at = map.get("removal_at").and_then(|v| Date::parse_iso(v));
    let sunset_milestone = map.get("sunset_milestone").cloned();
    let sunset_topic = map
        .get("sunset_topic")
        .cloned()
        .or_else(|| map.get("id").cloned())
        .unwrap_or_else(|| fallback_topic.to_string());
    let has_deprecation_marker = map
        .get("status")
        .map(|s| {
            let lower = s.to_ascii_lowercase();
            lower.contains("deprecated") || lower.contains("superseded")
        })
        .unwrap_or(false);
    SunsetClause {
        location: location.to_string(),
        sunset_at,
        sunset_milestone,
        deprecation_at,
        removal_at,
        sunset_topic,
        has_deprecation_marker,
    }
}

// ---------- Spec JSON `_sunset` discovery ----------

fn discover_specs(root: &Path) -> Result<Vec<SunsetClause>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries = fs::read_dir(root).map_err(|e| format!("read_dir({}): {e}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("entry under {}: {e}", root.display()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let contents =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if let Some(clause) = parse_json_sunset_block(&contents, &path.display().to_string()) {
            out.push(clause);
            continue;
        }
        // No machine-readable `_sunset` object, but the file references
        // sunset in prose -> MissingFields candidate.
        if has_body_sunset_prose(&contents) {
            out.push(SunsetClause {
                location: format!("{}#prose", path.display()),
                sunset_at: None,
                sunset_milestone: None,
                deprecation_at: None,
                removal_at: None,
                sunset_topic: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unnamed-sunset")
                    .to_string(),
                has_deprecation_marker: false,
            });
        }
    }
    Ok(out)
}

/// Extract a top-level `"_sunset": { ... }` object via brace-matched
/// substring, then parse the keys we care about. Deliberately
/// dependency-free — JSON `_sunset` objects use the flat scalar shape
/// defined in ADR-0108 (no nesting beyond the object itself).
fn parse_json_sunset_block(contents: &str, location: &str) -> Option<SunsetClause> {
    let key = "\"_sunset\"";
    let key_index = contents.find(key)?;
    let after = &contents[key_index + key.len()..];
    let brace_open = after.find('{')?;
    let body = &after[brace_open + 1..];
    let mut depth = 1usize;
    let mut end = 0usize;
    for (i, ch) in body.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let object_body = &body[..end];
    let map = parse_json_flat_scalars(object_body);
    let sunset_at = map.get("sunset_at").and_then(|v| Date::parse_iso(v));
    let deprecation_at = map.get("deprecation_at").and_then(|v| Date::parse_iso(v));
    let removal_at = map.get("removal_at").and_then(|v| Date::parse_iso(v));
    let sunset_milestone = map.get("sunset_milestone").cloned();
    let sunset_topic = map
        .get("sunset_topic")
        .cloned()
        .unwrap_or_else(|| "unnamed-sunset".to_string());
    let has_deprecation_marker = map
        .get("status")
        .map(|s| {
            let lower = s.to_ascii_lowercase();
            lower.contains("deprecated") || lower.contains("superseded")
        })
        .unwrap_or(false);
    Some(SunsetClause {
        location: format!("{location}#_sunset"),
        sunset_at,
        sunset_milestone,
        deprecation_at,
        removal_at,
        sunset_topic,
        has_deprecation_marker,
    })
}

/// Parse top-level `"key": "value"` and `"key": "<date>"` pairs from a
/// flat JSON object body. Sufficient for the `_sunset` shape per
/// ADR-0108.
fn parse_json_flat_scalars(body: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut chars = body.chars().peekable();
    let mut buf = String::new();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            buf.clear();
            for next in chars.by_ref() {
                if next == '"' {
                    break;
                }
                buf.push(next);
            }
            let key = buf.clone();
            // Skip to the next `"`
            for next in chars.by_ref() {
                if next == ':' {
                    break;
                }
            }
            // Read leading whitespace until `"` or non-string
            let mut value = String::new();
            let mut in_string = false;
            for next in chars.by_ref() {
                if !in_string {
                    if next == '"' {
                        in_string = true;
                        continue;
                    }
                    if next == ',' || next == '}' {
                        break;
                    }
                } else {
                    if next == '"' {
                        break;
                    }
                    value.push(next);
                }
            }
            if !key.is_empty() && !value.is_empty() {
                map.insert(key, value);
            }
        }
    }
    map
}

// ---------- Cargo manifest `[package.metadata.oya.sunset]` discovery ----------

fn discover_cargo_metadata(root: &Path) -> Result<Vec<SunsetClause>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries = fs::read_dir(root).map_err(|e| format!("read_dir({}): {e}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("entry under {}: {e}", root.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let contents = fs::read_to_string(&manifest)
            .map_err(|e| format!("read {}: {e}", manifest.display()))?;
        if let Some(clause) = parse_cargo_sunset_section(&contents, &manifest.display().to_string())
        {
            out.push(clause);
        }
    }
    Ok(out)
}

/// Locate the `[package.metadata.oya.sunset]` section in a Cargo manifest
/// and parse its scalar entries. Section ends at the next `[...]` header
/// or EOF.
fn parse_cargo_sunset_section(contents: &str, location: &str) -> Option<SunsetClause> {
    let header = "[package.metadata.oya.sunset]";
    let start = contents.find(header)?;
    let rest = &contents[start + header.len()..];
    // Section ends at next `[` at line start.
    let mut section = String::new();
    for line in rest.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') && !section.is_empty() {
            break;
        }
        section.push_str(line);
        section.push('\n');
    }
    let map = parse_toml_flat_scalars(&section);
    let sunset_at = map.get("sunset_at").and_then(|v| Date::parse_iso(v));
    let deprecation_at = map.get("deprecation_at").and_then(|v| Date::parse_iso(v));
    let removal_at = map.get("removal_at").and_then(|v| Date::parse_iso(v));
    let sunset_milestone = map.get("sunset_milestone").cloned();
    let sunset_topic = map
        .get("sunset_topic")
        .cloned()
        .unwrap_or_else(|| "unnamed-sunset".to_string());
    let has_deprecation_marker = map
        .get("status")
        .map(|s| {
            let lower = s.to_ascii_lowercase();
            lower.contains("deprecated") || lower.contains("superseded")
        })
        .unwrap_or(false);
    Some(SunsetClause {
        location: format!("{location}#package.metadata.oya.sunset"),
        sunset_at,
        sunset_milestone,
        deprecation_at,
        removal_at,
        sunset_topic,
        has_deprecation_marker,
    })
}

fn parse_toml_flat_scalars(block: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
            continue;
        }
        let Some(eq) = trimmed.find('=') else {
            continue;
        };
        let key = trimmed[..eq].trim().to_string();
        let value = trimmed[eq + 1..]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !key.is_empty() && !value.is_empty() {
            map.insert(key, value);
        }
    }
    map
}

// ---------- Reporting ----------

fn print_report(clauses: &[SunsetClause], violations: &[Violation]) {
    let mut by_state: BTreeMap<&'static str, usize> = BTreeMap::new();
    for v in violations {
        let label = match v.state {
            LifecycleState::SunsetReached => "SUNSET_REACHED (should-be-deprecated)",
            LifecycleState::RemovalReached => "REMOVAL_REACHED (must-be-removed)",
            LifecycleState::MissingFields => "MISSING_FIELDS (needs-schema-upgrade)",
            LifecycleState::PreSunset | LifecycleState::Deprecated => "OK",
        };
        *by_state.entry(label).or_insert(0) += 1;
    }
    if violations.is_empty() {
        println!(
            "sunset-lifecycle ok: clauses_scanned={} violations=0",
            clauses.len()
        );
        return;
    }
    eprintln!(
        "sunset-lifecycle FAIL: clauses_scanned={} violations={}",
        clauses.len(),
        violations.len()
    );
    for (label, count) in &by_state {
        eprintln!("  {label}: {count}");
    }
    for v in violations {
        let overdue = v
            .days_overdue
            .map(|d| format!(" days_overdue={d}"))
            .unwrap_or_default();
        eprintln!("  - {} state={:?}{}", v.clause_location, v.state, overdue);
        eprintln!("      action: {}", v.expected_action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn discovers_adr_with_yaml_sunset_fields() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("decisions");
        fs::create_dir_all(&dir).expect("mkdir");
        let adr_body = "---\n\
            id: ADR-9999\n\
            sunset_at: 2026-04-15\n\
            sunset_topic: test-thing\n\
            status: Accepted\n\
            ---\n\n\
            # ADR-9999\n";
        fs::write(dir.join("ADR-9999-test.md"), adr_body).expect("write");
        let clauses = discover_adr(&dir).expect("discover");
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0].sunset_at, Date::parse_iso("2026-04-15"));
        assert_eq!(clauses[0].sunset_topic, "test-thing");
        assert!(!clauses[0].has_deprecation_marker);
    }

    #[test]
    fn discovers_adr_without_frontmatter_via_body_prose_as_missing_fields() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("decisions");
        fs::create_dir_all(&dir).expect("mkdir");
        // No YAML frontmatter; body-only prose mentions sunset 2026.
        let body = "# ADR-1111\n\nThis surface is scheduled to sunset 2026-06-01.\n";
        fs::write(dir.join("ADR-1111-no-frontmatter.md"), body).expect("write");
        let clauses = discover_adr(&dir).expect("discover");
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0].sunset_topic, "ADR-1111-no-frontmatter");
        let violations = evaluate(&clauses, Date::parse_iso("2026-05-15").unwrap(), &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].state, LifecycleState::MissingFields);
    }

    #[test]
    fn discovers_adr_with_prose_only_sunset_note_as_missing_fields() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("decisions");
        fs::create_dir_all(&dir).expect("mkdir");
        let body = "---\n\
            id: ADR-8888\n\
            sunset_note: scheduled to retire once VCS lands\n\
            ---\n\n\
            # ADR-8888\n";
        fs::write(dir.join("ADR-8888.md"), body).expect("write");
        let clauses = discover_adr(&dir).expect("discover");
        assert_eq!(clauses.len(), 1);
        assert!(clauses[0].sunset_at.is_none());
        assert!(clauses[0].sunset_milestone.is_none());
        let violations = evaluate(&clauses, Date::parse_iso("2026-05-15").unwrap(), &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].state, LifecycleState::MissingFields);
    }

    #[test]
    fn discovers_spec_json_sunset_block() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join("specs");
        fs::create_dir_all(&dir).expect("mkdir");
        let body = r#"{
  "name": "demo",
  "_sunset": {
    "sunset_at": "2026-04-01",
    "sunset_topic": "json-test",
    "removal_at": "2026-07-15"
  }
}"#;
        fs::write(dir.join("demo.json"), body).expect("write");
        let clauses = discover_specs(&dir).expect("discover");
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0].sunset_at, Date::parse_iso("2026-04-01"));
        assert_eq!(clauses[0].removal_at, Date::parse_iso("2026-07-15"));
        assert_eq!(clauses[0].sunset_topic, "json-test");
    }

    #[test]
    fn discovers_cargo_manifest_sunset_metadata() {
        let tmp = TempDir::new().expect("tempdir");
        let crate_dir = tmp.path().join("crates").join("fake-crate");
        fs::create_dir_all(&crate_dir).expect("mkdir");
        let manifest = "[package]\n\
            name = \"fake-crate\"\n\n\
            [package.metadata.oya.sunset]\n\
            sunset_at = \"2026-03-01\"\n\
            sunset_topic = \"cargo-test\"\n\
            status = \"Deprecated\"\n\n\
            [dependencies]\n";
        fs::write(crate_dir.join("Cargo.toml"), manifest).expect("write");
        let clauses = discover_cargo_metadata(&tmp.path().join("crates")).expect("discover");
        assert_eq!(clauses.len(), 1);
        assert_eq!(clauses[0].sunset_at, Date::parse_iso("2026-03-01"));
        assert_eq!(clauses[0].sunset_topic, "cargo-test");
        assert!(clauses[0].has_deprecation_marker);
    }

    #[test]
    fn options_parses_now_and_milestones() {
        let args = [
            "--now".to_string(),
            "2026-05-15".to_string(),
            "--reached-milestone".to_string(),
            "M01-P08-merge".to_string(),
            "--reached-milestone".to_string(),
            "M01-P07-merge".to_string(),
        ];
        let options = Options::parse(&args).expect("parse");
        assert_eq!(options.now, Date::parse_iso("2026-05-15").unwrap());
        assert_eq!(
            options.reached_milestones,
            vec!["M01-P08-merge".to_string(), "M01-P07-merge".to_string(),]
        );
    }

    #[test]
    fn options_rejects_invalid_now() {
        let args = ["--now".to_string(), "not-a-date".to_string()];
        match Options::parse(&args) {
            Ok(_) => panic!("must reject invalid --now value"),
            Err(message) => assert!(
                message.contains("--now expects YYYY-MM-DD"),
                "unexpected error message: {message}"
            ),
        }
    }
}
