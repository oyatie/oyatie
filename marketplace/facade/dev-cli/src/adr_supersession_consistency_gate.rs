//! `oya gate validate adr-supersession-consistency`.
//!
//! Enforces that ADR supersession links are bidirectional. The defect this
//! closes (#6b): an ADR can declare `supersedes: [ADR-X]` in its front-matter
//! while ADR-X still carries an empty `superseded_by: []`, leaving a dangling
//! one-directional link that the ADR index and masterplan mirror then project
//! inconsistently.
//!
//! Invariant (pure link reciprocity over ADR<->ADR edges — it never inspects or
//! mutates `status`, so it cannot force a false status flip):
//!
//!  - FORWARD: for every ADR X whose `supersedes` lists an ADR id Y, ADR Y's
//!    `superseded_by` MUST contain X. A superseding ADR that is itself only
//!    `proposed`/`draft` expresses forward-looking INTENT, not an in-force
//!    supersession, so it does not yet obligate the target (exempted).
//!  - REVERSE: for every ADR Y whose `superseded_by` lists an ADR id X, ADR X's
//!    `supersedes` MUST contain Y. A `superseded_by` back-link is a factual
//!    "this ADR is retired" claim, so it always obligates the forward link.
//!
//! Non-ADR targets (e.g. `microservices/cell/PRD.md` path entries, or amendment
//! doc references) carry no front-matter and are skipped. ADR ids are extracted
//! from both bare ids (`ADR-0140`), filename references (`ADR-0140-slug.md`),
//! and quoted descriptive entries (`"ADR-0015 (partial — …)"`).
//!
//! ADR-0083 Tier-3 posture: this gate is panic-free — every fallible step
//! returns `Result`/`ExitCode`, with no `unwrap`/`expect`/`panic` in
//! non-test code.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::adr_planning_frontmatter::{frontmatter_list, frontmatter_scalar, read_frontmatter};

/// Statuses for which a *superseding* ADR's forward `supersedes` edge does NOT
/// yet obligate the target to back-link (the supersession is intent, not yet in
/// force). Compared case-insensitively.
const NOT_IN_FORCE_STATUSES: &[&str] = &["proposed", "draft", "drafted", "rejected", "withdrawn"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdrSupersessionConsistencyArgs {
    pub(crate) decisions_dir: PathBuf,
}

pub(crate) fn parse_adr_supersession_consistency_args(
    args: Vec<String>,
) -> Result<AdrSupersessionConsistencyArgs, String> {
    let mut parsed = AdrSupersessionConsistencyArgs {
        decisions_dir: PathBuf::from("docs/decisions"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--decisions-dir" => {
                parsed.decisions_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--decisions-dir requires a value".to_string())?,
                );
            }
            other => {
                return Err(format!(
                    "adr-supersession-consistency: unknown flag {other:?}; allowed: --decisions-dir"
                ));
            }
        }
    }
    Ok(parsed)
}

/// One ADR's supersession-relevant front-matter, with link targets already
/// resolved to ADR ids (non-ADR targets dropped).
#[derive(Clone, Debug, Eq, PartialEq)]
struct AdrLinks {
    id: String,
    status: String,
    supersedes: BTreeSet<String>,
    superseded_by: BTreeSet<String>,
}

#[derive(Default, Debug, Eq, PartialEq)]
pub(crate) struct AdrSupersessionConsistencyReport {
    pub(crate) adrs_checked: usize,
    pub(crate) edges_checked: usize,
    pub(crate) failures: Vec<String>,
}

/// Extract an `ADR-NNNN` id from a raw front-matter list entry, or `None` for a
/// non-ADR target. Path-shaped entries (containing `/`, e.g.
/// `microservices/cell/PRD.md`) are treated as non-ADR targets because they
/// cannot carry front-matter back-links.
fn adr_id_of(entry: &str) -> Option<String> {
    let entry = entry.trim();
    if entry.contains('/') {
        return None;
    }
    // Find a leading `ADR-` then exactly four ascii digits.
    let start = entry.find("ADR-")?;
    let digits_start = start + 4;
    let digits: String = entry[digits_start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.len() != 4 {
        return None;
    }
    Some(format!("ADR-{digits}"))
}

fn resolve_ids(entries: &[String]) -> BTreeSet<String> {
    entries.iter().filter_map(|e| adr_id_of(e)).collect()
}

/// Read every `docs/decisions/ADR-*.md`, parse the supersession front-matter,
/// and build the id -> links map (sorted by id for deterministic reporting).
fn read_adr_links(decisions_dir: &Path) -> Result<BTreeMap<String, AdrLinks>, String> {
    let entries = fs::read_dir(decisions_dir).map_err(|error| {
        format!(
            "ADR decisions dir unreadable {}: {error}",
            decisions_dir.display()
        )
    })?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("ADR decisions dir entry unreadable: {error}"))?;
        let path = entry.path();
        let is_adr = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ADR-") && name.ends_with(".md"));
        if is_adr {
            paths.push(path);
        }
    }
    paths.sort();

    let mut links: BTreeMap<String, AdrLinks> = BTreeMap::new();
    for path in &paths {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let Some(id) = adr_id_of(file_name) else {
            continue;
        };
        let contents = fs::read_to_string(path)
            .map_err(|error| format!("ADR unreadable {}: {error}", path.display()))?;
        let Some(frontmatter) = read_frontmatter(&contents) else {
            continue;
        };
        let status = frontmatter_scalar(frontmatter, "status").unwrap_or_default();
        let supersedes = resolve_ids(&frontmatter_list(frontmatter, "supersedes"));
        let superseded_by = resolve_ids(&frontmatter_list(frontmatter, "superseded_by"));
        links.insert(
            id.clone(),
            AdrLinks {
                id,
                status,
                supersedes,
                superseded_by,
            },
        );
    }
    if links.is_empty() {
        return Err(format!(
            "ADR decisions dir contains no ADR-NNNN markdown files: {}",
            decisions_dir.display()
        ));
    }
    Ok(links)
}

fn status_in_force(status: &str) -> bool {
    let status = status.trim().to_ascii_lowercase();
    !NOT_IN_FORCE_STATUSES.contains(&status.as_str())
}

pub(crate) fn validate_adr_supersession_consistency_gate(
    args: AdrSupersessionConsistencyArgs,
) -> Result<AdrSupersessionConsistencyReport, String> {
    let links = read_adr_links(&args.decisions_dir)?;
    let mut report = AdrSupersessionConsistencyReport {
        adrs_checked: links.len(),
        ..AdrSupersessionConsistencyReport::default()
    };

    // FORWARD: X supersedes Y (X in force) => Y.superseded_by contains X.
    for source in links.values() {
        for target_id in &source.supersedes {
            report.edges_checked += 1;
            let Some(target) = links.get(target_id) else {
                report.failures.push(format!(
                    "[MISSING_TARGET] {} supersedes {target_id}, but {target_id} ADR file not found in {}",
                    source.id,
                    args.decisions_dir.display()
                ));
                continue;
            };
            if !status_in_force(&source.status) {
                // Proposed/draft superseder: intent only, does not yet obligate.
                continue;
            }
            if !target.superseded_by.contains(&source.id) {
                report.failures.push(format!(
                    "[FORWARD_NO_BACKLINK] {} (status={}) supersedes {target_id}, but {target_id}.superseded_by does not contain {}",
                    source.id, source.status, source.id
                ));
            }
        }
    }

    // REVERSE: Y superseded_by X => X.supersedes contains Y.
    for target in links.values() {
        for source_id in &target.superseded_by {
            report.edges_checked += 1;
            let Some(source) = links.get(source_id) else {
                report.failures.push(format!(
                    "[MISSING_SOURCE] {} is superseded_by {source_id}, but {source_id} ADR file not found in {}",
                    target.id,
                    args.decisions_dir.display()
                ));
                continue;
            };
            if !source.supersedes.contains(&target.id) {
                report.failures.push(format!(
                    "[REVERSE_NO_FORWARD] {} is superseded_by {source_id}, but {source_id}.supersedes does not contain {}",
                    target.id, target.id
                ));
            }
        }
    }

    report.failures.sort();
    report.failures.dedup();
    Ok(report)
}

pub(crate) fn run_adr_supersession_consistency(args: Vec<String>) -> ExitCode {
    let parsed = match parse_adr_supersession_consistency_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match validate_adr_supersession_consistency_gate(parsed) {
        Ok(report) => {
            if report.failures.is_empty() {
                println!(
                    "adr-supersession-consistency validation passed: {} ADRs, {} supersession edge(s) all bidirectional",
                    report.adrs_checked, report.edges_checked
                );
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "adr-supersession-consistency validation failed: {} one-directional supersession link(s) [{} ADRs, {} edges]",
                    report.failures.len(),
                    report.adrs_checked,
                    report.edges_checked
                );
                for failure in &report.failures {
                    eprintln!("  - {failure}");
                }
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("adr-supersession-consistency validation error: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_adr(dir: &Path, id: &str, frontmatter_body: &str) {
        let path = dir.join(format!("{id}-fixture.md"));
        let contents = format!("---\nid: {id}\n{frontmatter_body}\n---\n\n# {id}: fixture\n");
        fs::write(path, contents).expect("write fixture ADR");
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("oya-adr-supersession-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp decisions dir");
        dir
    }

    fn run(dir: &Path) -> AdrSupersessionConsistencyReport {
        validate_adr_supersession_consistency_gate(AdrSupersessionConsistencyArgs {
            decisions_dir: dir.to_path_buf(),
        })
        .expect("gate runs")
    }

    #[test]
    fn bidirectional_pair_passes() {
        let dir = temp_dir("ok");
        write_adr(
            &dir,
            "ADR-0001",
            "status: Accepted\nsupersedes: [ADR-0002]\nsuperseded_by: []",
        );
        write_adr(
            &dir,
            "ADR-0002",
            "status: Superseded\nsupersedes: []\nsuperseded_by: [ADR-0001]",
        );
        let report = run(&dir);
        assert!(
            report.failures.is_empty(),
            "expected pass, got {:?}",
            report.failures
        );
        assert_eq!(report.adrs_checked, 2);
        assert_eq!(report.edges_checked, 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn forward_link_without_backlink_fails() {
        let dir = temp_dir("fwd");
        // ADR-0001 (in force) supersedes ADR-0002, but ADR-0002 does not
        // back-link it.
        write_adr(
            &dir,
            "ADR-0001",
            "status: Accepted\nsupersedes: [ADR-0002]\nsuperseded_by: []",
        );
        write_adr(
            &dir,
            "ADR-0002",
            "status: Accepted\nsupersedes: []\nsuperseded_by: []",
        );
        let report = run(&dir);
        assert_eq!(
            report.failures.len(),
            1,
            "expected one failure, got {:?}",
            report.failures
        );
        assert!(
            report.failures[0].contains("FORWARD_NO_BACKLINK")
                && report.failures[0].contains("ADR-0001")
                && report.failures[0].contains("ADR-0002"),
            "unexpected failure: {:?}",
            report.failures
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reverse_link_without_forward_fails() {
        let dir = temp_dir("rev");
        // ADR-0002 claims it is superseded_by ADR-0001, but ADR-0001 does not
        // list it in supersedes.
        write_adr(
            &dir,
            "ADR-0001",
            "status: Accepted\nsupersedes: []\nsuperseded_by: []",
        );
        write_adr(
            &dir,
            "ADR-0002",
            "status: Superseded\nsupersedes: []\nsuperseded_by: [ADR-0001]",
        );
        let report = run(&dir);
        assert_eq!(
            report.failures.len(),
            1,
            "expected one failure, got {:?}",
            report.failures
        );
        assert!(
            report.failures[0].contains("REVERSE_NO_FORWARD"),
            "unexpected failure: {:?}",
            report.failures
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn proposed_superseder_does_not_obligate_target() {
        let dir = temp_dir("proposed");
        // A *Proposed* ADR-0001 declaring supersession is intent only; ADR-0002
        // is not obligated to back-link, so this passes.
        write_adr(
            &dir,
            "ADR-0001",
            "status: Proposed\nsupersedes: [ADR-0002]\nsuperseded_by: []",
        );
        write_adr(
            &dir,
            "ADR-0002",
            "status: Accepted\nsupersedes: []\nsuperseded_by: []",
        );
        let report = run(&dir);
        assert!(
            report.failures.is_empty(),
            "proposed superseder should be exempt, got {:?}",
            report.failures
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn non_adr_target_is_ignored() {
        let dir = temp_dir("nonadr");
        // Path-shaped supersession target carries no front-matter, so it is not
        // checked for a back-link.
        write_adr(
            &dir,
            "ADR-0001",
            "status: Accepted\nsupersedes:\n  - microservices/cell/PRD.md\nsuperseded_by: []",
        );
        let report = run(&dir);
        assert!(
            report.failures.is_empty(),
            "non-ADR target should be ignored, got {:?}",
            report.failures
        );
        assert_eq!(report.edges_checked, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn filename_and_quoted_entries_resolve_to_ids() {
        assert_eq!(adr_id_of("ADR-0140").as_deref(), Some("ADR-0140"));
        assert_eq!(
            adr_id_of("ADR-0107-tools-implicit-app-convention.md").as_deref(),
            Some("ADR-0107")
        );
        assert_eq!(
            adr_id_of("ADR-0015 (partial — only the split)").as_deref(),
            Some("ADR-0015")
        );
        assert_eq!(adr_id_of("microservices/cell/PRD.md"), None);
        assert_eq!(adr_id_of("not-an-adr"), None);
    }
}
