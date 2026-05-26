//! Shared parser for the generative ADR planning front-matter (ADR-0364 §2).
//!
//! ADR front-matter is YAML between the leading `---` fences. This module
//! extracts the machine-extractable planning fields used by the masterplan
//! generator (`oya gen masterplan`, D3) and the completeness gate
//! (`adr-planning-completeness`, D2):
//!
//! ```yaml
//! ---
//! id: ADR-0364
//! status: Accepted
//! planning_impact: true
//! milestone: M-PLANNING-SSOT
//! depends_on: [ADR-0363]
//! deliverables:
//!   - id: ADR-0364-D1
//!     description: "..."
//!     exit_criteria: "..."
//!     verified_by: "oya lint adr-shape"
//! ---
//! ```
//!
//! This is a focused YAML-subset reader (not a general YAML engine): it handles
//! the leading-fence front-matter, scalar keys, inline `[a, b]` lists, block
//! `- item` lists, and the `deliverables:` block of `- key: value` maps. It
//! reuses the same `---`-fence convention as the existing `adr-index` and
//! `planning-ssot-coverage` front-matter readers.

use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanningDeliverable {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) exit_criteria: String,
    pub(crate) verified_by: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanningAdr {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) milestone: String,
    pub(crate) depends_on: Vec<String>,
    /// Whether the `deliverables:` key was present at all (even if empty).
    /// D2 distinguishes "has deliverables but incomplete" (FAIL) from "no
    /// deliverables field" (ADVISORY backfill, deferred to ADR-0364 D7).
    pub(crate) has_deliverables_field: bool,
    pub(crate) deliverables: Vec<PlanningDeliverable>,
    pub(crate) path: String,
}

/// Read every `docs/decisions/ADR-*.md`, parse front-matter, and return the
/// ADRs with `planning_impact: true`, sorted by id for determinism.
pub(crate) fn read_planning_impact_adrs(decisions_dir: &Path) -> Result<Vec<PlanningAdr>, String> {
    let entries = fs::read_dir(decisions_dir).map_err(|error| {
        format!(
            "ADR decisions dir unreadable {}: {error}",
            decisions_dir.display()
        )
    })?;
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("ADR decisions dir entry unreadable: {error}"))?;
        let path = entry.path();
        let is_adr = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.starts_with("ADR-") && name.ends_with(".md") && !name.contains("-amendment-")
            });
        if is_adr {
            paths.push(path);
        }
    }
    paths.sort();

    let mut adrs = Vec::new();
    for path in &paths {
        let contents = fs::read_to_string(path)
            .map_err(|error| format!("ADR unreadable {}: {error}", path.display()))?;
        let Some(frontmatter) = read_frontmatter(&contents) else {
            continue;
        };
        if !frontmatter_flag(frontmatter, "planning_impact") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let id = file_name.get(0..8).unwrap_or_default().to_string();
        adrs.push(parse_planning_adr(
            frontmatter,
            id,
            format!("docs/decisions/{file_name}"),
        ));
    }
    Ok(adrs)
}

/// Extract the YAML front-matter between the leading `---` fences.
pub(crate) fn read_frontmatter(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn parse_planning_adr(frontmatter: &str, id: String, path: String) -> PlanningAdr {
    let status = frontmatter_scalar(frontmatter, "status").unwrap_or_default();
    let milestone = frontmatter_scalar(frontmatter, "milestone").unwrap_or_default();
    let depends_on = frontmatter_list(frontmatter, "depends_on");
    let (has_deliverables_field, deliverables) = parse_deliverables(frontmatter);
    PlanningAdr {
        id,
        status,
        milestone,
        depends_on,
        has_deliverables_field,
        deliverables,
        path,
    }
}

/// Top-level keys in the front-matter are unindented. Returns `true` when
/// `<key>: true`.
fn frontmatter_flag(frontmatter: &str, key: &str) -> bool {
    frontmatter_scalar(frontmatter, key).as_deref() == Some("true")
}

/// Read a top-level scalar (unindented `<key>: <value>`).
pub(crate) fn frontmatter_scalar(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((found_key, value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() == key {
            return Some(clean_scalar(value));
        }
    }
    None
}

/// Read a top-level list — either inline `[a, b]` or a block of `- item`
/// lines that follow `<key>:`.
fn frontmatter_list(frontmatter: &str, key: &str) -> Vec<String> {
    let mut lines = frontmatter.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((found_key, value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() != key {
            continue;
        }
        let value = value.trim();
        if value.starts_with('[') {
            return parse_inline_list(value);
        }
        if !value.is_empty() {
            return vec![clean_scalar(value)];
        }
        // Block list: collect following indented `- item` lines.
        let mut items = Vec::new();
        while let Some(next) = lines.peek() {
            let trimmed = next.trim_start();
            let indented = next.starts_with(char::is_whitespace);
            if let Some(item) = trimmed.strip_prefix("- ") {
                if !indented {
                    break;
                }
                items.push(clean_scalar(item));
                lines.next();
            } else if indented && trimmed.is_empty() {
                lines.next();
            } else {
                break;
            }
        }
        return items;
    }
    Vec::new()
}

fn parse_inline_list(value: &str) -> Vec<String> {
    let inner = value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(clean_scalar)
        .filter(|item| !item.is_empty())
        .collect()
}

/// Parse the `deliverables:` block: a sequence of `- key: value` maps. Returns
/// `(field_present, deliverables)`. A new deliverable starts at each `- id:`
/// (or any `- <key>:`); subsequent same-indent `key: value` lines extend it.
fn parse_deliverables(frontmatter: &str) -> (bool, Vec<PlanningDeliverable>) {
    let mut lines = frontmatter.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((found_key, value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() != "deliverables" {
            continue;
        }
        // Inline empty list `deliverables: []` => present but empty.
        if value.trim().starts_with('[') {
            return (true, Vec::new());
        }
        // Block: collect indented map records.
        let mut records: Vec<Vec<(String, String)>> = Vec::new();
        let mut current: Option<Vec<(String, String)>> = None;
        while let Some(next) = lines.peek() {
            if !next.starts_with(char::is_whitespace) || next.trim().is_empty() {
                // dedent back to a top-level key (or blank) => block ended.
                if next.trim().is_empty() {
                    lines.next();
                    continue;
                }
                break;
            }
            let trimmed = next.trim_start();
            if let Some(rest) = trimmed.strip_prefix("- ") {
                if let Some(record) = current.take() {
                    records.push(record);
                }
                let mut record = Vec::new();
                if let Some((k, v)) = rest.split_once(':') {
                    record.push((k.trim().to_string(), clean_scalar(v)));
                }
                current = Some(record);
            } else if let Some((k, v)) = trimmed.split_once(':')
                && let Some(record) = current.as_mut()
            {
                record.push((k.trim().to_string(), clean_scalar(v)));
            }
            lines.next();
        }
        if let Some(record) = current.take() {
            records.push(record);
        }
        let deliverables = records
            .into_iter()
            .map(|record| {
                let get = |key: &str| {
                    record
                        .iter()
                        .find(|(k, _)| k == key)
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default()
                };
                PlanningDeliverable {
                    id: get("id"),
                    description: get("description"),
                    exit_criteria: get("exit_criteria"),
                    verified_by: get("verified_by"),
                }
            })
            .collect();
        return (true, deliverables);
    }
    // No `deliverables:` key in the front-matter.
    (false, Vec::new())
}

fn clean_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"---
id: ADR-0364
status: Accepted
planning_impact: true
milestone: M-PLANNING-SSOT
depends_on: [ADR-0363]
deliverables:
  - id: ADR-0364-D1
    description: "the contract"
    exit_criteria: "ADR present"
    verified_by: "oya lint adr-shape"
  - id: ADR-0364-D2
    description: "completeness gate"
    exit_criteria: "gate green"
    verified_by: "oya gate validate adr-planning-completeness"
purpose: do the thing
---
# ADR-0364: title
"#;

    #[test]
    fn extracts_planning_fields() {
        let fm = read_frontmatter(SAMPLE).expect("frontmatter");
        assert!(frontmatter_flag(fm, "planning_impact"));
        assert_eq!(
            frontmatter_scalar(fm, "milestone").as_deref(),
            Some("M-PLANNING-SSOT")
        );
        assert_eq!(frontmatter_list(fm, "depends_on"), vec!["ADR-0363"]);
    }

    #[test]
    fn parses_deliverable_block() {
        let fm = read_frontmatter(SAMPLE).expect("frontmatter");
        let (present, deliverables) = parse_deliverables(fm);
        assert!(present);
        assert_eq!(deliverables.len(), 2);
        assert_eq!(deliverables[0].id, "ADR-0364-D1");
        assert_eq!(deliverables[0].description, "the contract");
        assert_eq!(
            deliverables[1].verified_by,
            "oya gate validate adr-planning-completeness"
        );
    }

    #[test]
    fn no_deliverables_field_is_absent() {
        let fm = "id: ADR-0001\nplanning_impact: true\nmilestone: M1";
        let (present, deliverables) = parse_deliverables(fm);
        assert!(!present);
        assert!(deliverables.is_empty());
    }

    #[test]
    fn empty_inline_deliverables_is_present_but_empty() {
        let fm = "id: ADR-0001\ndeliverables: []";
        let (present, deliverables) = parse_deliverables(fm);
        assert!(present);
        assert!(deliverables.is_empty());
    }

    #[test]
    fn non_frontmatter_returns_none() {
        assert!(read_frontmatter("# no frontmatter").is_none());
    }
}
