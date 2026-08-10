//! # cloud-ci-hub-exclusivity (Swarm Delivery Law / ADR-0711)
//!
//! Mechanical sole-owner enforcement for governed hubs. Open `integ/*` PRs MUST NOT
//! multi-own any path listed at `specs/integ-branch-envelopes.json#hubs.paths`.
//!
//! ## Forever shape
//! - Authority is a **JSON pointer**, never a re-listed path set in this crate or its
//!   policy JSON (`#anti_drift.prose_must_cite_not_enumerate`).
//! - Producer supplies hub paths + open-PR file facts as DATA; this module is a pure
//!   fail-closed evaluator (no shell, net, clock, or filesystem).
//! - Empty hub authority with `sole_owner_per_wave=true` is RED (`hub_authority_empty`),
//!   never a silent green.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// Gate id (matches policy `gate_id` + buck2 test target purpose).
pub const GATE_ID: &str = "cloud-ci-hub-exclusivity";

/// Canonical authority pointer — cite; do not re-list hub paths in prose or policy DATA.
pub const HUBS_PATHS_POINTER: &str = "specs/integ-branch-envelopes.json#hubs.paths";

/// Two or more open integ PRs touch the same hub path.
pub const CODE_MULTI_OWN_HUB: &str = "hub_multi_owned";
/// Hub authority was empty while sole-owner mode is required (fail-closed).
pub const CODE_AUTHORITY_EMPTY: &str = "hub_authority_empty";
/// Policy pointer does not match [`HUBS_PATHS_POINTER`].
pub const CODE_AUTHORITY_POINTER_MISMATCH: &str = "hub_authority_pointer_mismatch";
/// Policy `gate_id` does not match [`GATE_ID`].
pub const CODE_POLICY_GATE_ID_MISMATCH: &str = "hub_exclusivity_gate_id_mismatch";
/// Envelopes JSON hubs.paths entry was missing or non-array (fail-closed parse).
pub const CODE_HUBS_PATHS_MALFORMED: &str = "hubs_paths_malformed";
/// Malformed / unparseable open-PR file-facts payload (producer DATA).
pub const CODE_OPEN_PR_FACTS_MALFORMED: &str = "open_pr_facts_malformed";

pub const VIOLATION_CODES: [&str; 6] = [
    CODE_MULTI_OWN_HUB,
    CODE_AUTHORITY_EMPTY,
    CODE_AUTHORITY_POINTER_MISMATCH,
    CODE_POLICY_GATE_ID_MISMATCH,
    CODE_HUBS_PATHS_MALFORMED,
    CODE_OPEN_PR_FACTS_MALFORMED,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    /// Mechanical REFUSE — Claim/Land MUST NOT treat this tip as hub-clear.
    Refuse,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub findings: BTreeSet<Finding>,
}

impl Report {
    fn from_findings(findings: BTreeSet<Finding>) -> Self {
        let verdict = if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Refuse
        };
        Self { verdict, findings }
    }
}

/// Policy pack for the hub-exclusivity gate (DATA; no hub path enumeration).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubExclusivityPolicy {
    pub gate_id: String,
    /// Must equal [`HUBS_PATHS_POINTER`].
    pub hubs_paths_authority: String,
    /// Only heads with this prefix participate (default `integ/`).
    pub integ_head_ref_prefix: String,
    pub sole_owner_per_wave: bool,
}

impl HubExclusivityPolicy {
    /// Parse policy JSON. Missing fields fail closed via evaluate (empty/mismatch codes).
    pub fn from_json(value: &Value) -> Self {
        Self {
            gate_id: value
                .get("gate_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_owned(),
            hubs_paths_authority: value
                .get("hubs_paths_authority")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_owned(),
            integ_head_ref_prefix: value
                .get("integ_head_ref_prefix")
                .and_then(Value::as_str)
                .unwrap_or("integ/")
                .trim()
                .to_owned(),
            sole_owner_per_wave: value
                .get("sole_owner_per_wave")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        }
    }
}

/// Hub path set loaded from envelopes `#hubs.paths` by the producer (never hardcoded here).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HubAuthority {
    pub paths: BTreeSet<String>,
}

/// One open PR's head + changed files (producer-supplied SCM facts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPrFact {
    pub number: u64,
    pub head_ref_name: String,
    pub files: BTreeSet<String>,
}


/// Default on-disk policy pack path (relative to repo root).
pub const DEFAULT_POLICY_RELPATH: &str =
    "ci/facade/affected-target-set/hub-exclusivity-policy.json";

/// Envelopes document path that owns `#hubs.paths` (cite; do not re-list).
pub const ENVELOPES_RELPATH: &str = "specs/integ-branch-envelopes.json";

/// Parse producer-supplied open-PR file facts.
///
/// Accepted shapes (both are DATA — never instructions):
/// - Simplified fixture: `[{ "number", "head_ref_name", "files": ["path", ...] }, ...]`
/// - GitHub pulls list fragment: `[{ "number", "head": { "ref": "..." }, "files":
///   [{ "filename": "..." }, ...] | ["path", ...] }, ...]`
pub fn open_pr_facts_from_json(value: &Value) -> Result<Vec<OpenPrFact>, Finding> {
    let arr = value.as_array().ok_or_else(|| {
        Finding::new(
            CODE_OPEN_PR_FACTS_MALFORMED,
            "open_prs",
            "open-PR file facts must be a JSON array",
        )
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for (idx, entry) in arr.iter().enumerate() {
        let number = entry
            .get("number")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                Finding::new(
                    CODE_OPEN_PR_FACTS_MALFORMED,
                    format!("open_prs[{idx}].number"),
                    "each open PR fact requires a numeric `number`",
                )
            })?;
        let head_ref_name = entry
            .get("head_ref_name")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                entry
                    .get("head")
                    .and_then(|h| h.get("ref"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                Finding::new(
                    CODE_OPEN_PR_FACTS_MALFORMED,
                    format!("open_prs[{idx}].head_ref_name"),
                    "each open PR fact requires `head_ref_name` or `head.ref`",
                )
            })?;
        let files_value = entry.get("files").ok_or_else(|| {
            Finding::new(
                CODE_OPEN_PR_FACTS_MALFORMED,
                format!("open_prs[{idx}].files"),
                "each open PR fact requires a `files` array",
            )
        })?;
        let files_arr = files_value.as_array().ok_or_else(|| {
            Finding::new(
                CODE_OPEN_PR_FACTS_MALFORMED,
                format!("open_prs[{idx}].files"),
                "`files` must be a JSON array of paths or `{filename}` objects",
            )
        })?;
        let mut files = BTreeSet::new();
        for (fidx, f) in files_arr.iter().enumerate() {
            let path = f
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .or_else(|| {
                    f.get("filename")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                });
            match path {
                Some(p) => {
                    files.insert(p);
                }
                None => {
                    return Err(Finding::new(
                        CODE_OPEN_PR_FACTS_MALFORMED,
                        format!("open_prs[{idx}].files[{fidx}]"),
                        "file entries must be non-empty path strings or objects with `filename`",
                    ));
                }
            }
        }
        out.push(OpenPrFact {
            number,
            head_ref_name,
            files,
        });
    }
    Ok(out)
}

/// Load policy + envelopes authority + open-PR facts and evaluate (pure; no I/O).
pub fn evaluate_from_producer_docs(
    policy_doc: &Value,
    envelopes_doc: &Value,
    open_prs_doc: &Value,
) -> Report {
    let policy = HubExclusivityPolicy::from_json(policy_doc);
    let authority = match hubs_paths_from_envelopes(envelopes_doc) {
        Ok(a) => a,
        Err(finding) => {
            let mut findings = BTreeSet::new();
            findings.insert(finding);
            return Report::from_findings(findings);
        }
    };
    let open_prs = match open_pr_facts_from_json(open_prs_doc) {
        Ok(p) => p,
        Err(finding) => {
            let mut findings = BTreeSet::new();
            findings.insert(finding);
            return Report::from_findings(findings);
        }
    };
    evaluate(&policy, &authority, &open_prs)
}

/// Extract `#hubs.paths` from an envelopes document. Fail-closed on missing/non-array.
pub fn hubs_paths_from_envelopes(doc: &Value) -> Result<HubAuthority, Finding> {
    let paths_value = doc
        .pointer("/hubs/paths")
        .ok_or_else(|| {
            Finding::new(
                CODE_HUBS_PATHS_MALFORMED,
                HUBS_PATHS_POINTER,
                "envelopes document missing /hubs/paths — refuse rather than invent hubs",
            )
        })?;
    let arr = paths_value.as_array().ok_or_else(|| {
        Finding::new(
            CODE_HUBS_PATHS_MALFORMED,
            HUBS_PATHS_POINTER,
            "/hubs/paths must be a JSON array of path strings",
        )
    })?;
    let mut paths = BTreeSet::new();
    for (idx, entry) in arr.iter().enumerate() {
        match entry.as_str().map(str::trim) {
            Some(path) if !path.is_empty() => {
                paths.insert(path.to_owned());
            }
            _ => {
                return Err(Finding::new(
                    CODE_HUBS_PATHS_MALFORMED,
                    format!("{HUBS_PATHS_POINTER}[{idx}]"),
                    "hubs.paths entries must be non-empty strings",
                ));
            }
        }
    }
    Ok(HubAuthority { paths })
}

/// Pure evaluate: REFUSE when open integ PRs multi-own any hub path.
pub fn evaluate(
    policy: &HubExclusivityPolicy,
    authority: &HubAuthority,
    open_prs: &[OpenPrFact],
) -> Report {
    let mut findings = BTreeSet::new();

    if policy.gate_id != GATE_ID {
        findings.insert(Finding::new(
            CODE_POLICY_GATE_ID_MISMATCH,
            "gate_id",
            format!(
                "policy gate_id {:?} must equal {GATE_ID}",
                policy.gate_id
            ),
        ));
    }

    if policy.hubs_paths_authority != HUBS_PATHS_POINTER {
        findings.insert(Finding::new(
            CODE_AUTHORITY_POINTER_MISMATCH,
            "hubs_paths_authority",
            format!(
                "policy hubs_paths_authority {:?} must equal {HUBS_PATHS_POINTER} (cite pointer; do not fork the list)",
                policy.hubs_paths_authority
            ),
        ));
    }

    if policy.sole_owner_per_wave && authority.paths.is_empty() {
        findings.insert(Finding::new(
            CODE_AUTHORITY_EMPTY,
            HUBS_PATHS_POINTER,
            "hub authority is empty while sole_owner_per_wave is true — fail-closed (missing envelopes load is not green)",
        ));
    }

    if !findings.is_empty() {
        return Report::from_findings(findings);
    }

    let prefix = policy.integ_head_ref_prefix.as_str();
    let integ_prs: Vec<&OpenPrFact> = open_prs
        .iter()
        .filter(|pr| pr.head_ref_name.starts_with(prefix))
        .collect();

    // hub path -> owning integ PR numbers (stable order)
    let mut owners: BTreeMap<&str, BTreeSet<u64>> = BTreeMap::new();
    for hub in &authority.paths {
        for pr in &integ_prs {
            if pr.files.contains(hub) {
                owners.entry(hub.as_str()).or_default().insert(pr.number);
            }
        }
    }

    for (hub, pr_numbers) in owners {
        if pr_numbers.len() > 1 {
            let list = pr_numbers
                .iter()
                .map(|n| format!("#{n}"))
                .collect::<Vec<_>>()
                .join(", ");
            findings.insert(Finding::new(
                CODE_MULTI_OWN_HUB,
                hub,
                format!(
                    "hub path multi-owned by open integ PRs [{list}] — sole_owner_per_wave requires exactly one owner (REFUSE)"
                ),
            ));
        }
    }

    Report::from_findings(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> HubExclusivityPolicy {
        HubExclusivityPolicy {
            gate_id: GATE_ID.to_owned(),
            hubs_paths_authority: HUBS_PATHS_POINTER.to_owned(),
            integ_head_ref_prefix: "integ/".to_owned(),
            sole_owner_per_wave: true,
        }
    }

    fn authority(paths: &[&str]) -> HubAuthority {
        HubAuthority {
            paths: paths.iter().map(|p| (*p).to_owned()).collect(),
        }
    }

    fn pr(number: u64, head: &str, files: &[&str]) -> OpenPrFact {
        OpenPrFact {
            number,
            head_ref_name: head.to_owned(),
            files: files.iter().map(|f| (*f).to_owned()).collect(),
        }
    }

    #[test]
    fn multi_own_hub_among_open_integ_prs_refuses() {
        let report = evaluate(
            &policy(),
            &authority(&["hub/a.json", "hub/b.json"]),
            &[
                pr(1643, "integ/os", &["hub/a.json", "os/foo.rs"]),
                pr(1647, "integ/build", &["hub/a.json", "Cargo.toml"]),
                pr(1644, "integ/specs", &["hub/b.json"]),
            ],
        );
        assert_eq!(report.verdict, Verdict::Refuse);
        let multi: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.code == CODE_MULTI_OWN_HUB)
            .collect();
        assert_eq!(multi.len(), 1);
        assert_eq!(multi[0].key, "hub/a.json");
        assert!(multi[0].detail.contains("#1643"));
        assert!(multi[0].detail.contains("#1647"));
    }

    #[test]
    fn sole_owner_hub_is_green() {
        let report = evaluate(
            &policy(),
            &authority(&["hub/a.json"]),
            &[
                pr(1644, "integ/specs", &["hub/a.json"]),
                pr(1643, "integ/os", &["os/foo.rs"]),
            ],
        );
        assert_eq!(report.verdict, Verdict::Green);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn non_integ_heads_do_not_count_as_owners() {
        let report = evaluate(
            &policy(),
            &authority(&["hub/a.json"]),
            &[
                pr(1, "integ/specs", &["hub/a.json"]),
                pr(2, "feature/other", &["hub/a.json"]),
            ],
        );
        assert_eq!(report.verdict, Verdict::Green);
    }

    #[test]
    fn empty_authority_fail_closed() {
        let report = evaluate(&policy(), &authority(&[]), &[]);
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_AUTHORITY_EMPTY)
        );
    }

    #[test]
    fn pointer_mismatch_fail_closed() {
        let mut p = policy();
        p.hubs_paths_authority = "specs/forked-hubs.json#paths".to_owned();
        let report = evaluate(&p, &authority(&["hub/a.json"]), &[]);
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_AUTHORITY_POINTER_MISMATCH)
        );
    }

    #[test]
    fn gate_id_mismatch_fail_closed() {
        let mut p = policy();
        p.gate_id = "wrong".to_owned();
        let report = evaluate(&p, &authority(&["hub/a.json"]), &[]);
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_POLICY_GATE_ID_MISMATCH)
        );
    }

    #[test]
    fn hubs_paths_from_envelopes_extracts_pointer_shape() {
        let doc = json!({
            "hubs": {
                "sole_owner_per_wave": true,
                "paths": ["alpha", "beta"]
            }
        });
        let auth = hubs_paths_from_envelopes(&doc).expect("ok");
        assert_eq!(
            auth.paths,
            ["alpha".to_owned(), "beta".to_owned()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn hubs_paths_from_envelopes_missing_paths_refuses() {
        let doc = json!({ "hubs": {} });
        let err = hubs_paths_from_envelopes(&doc).expect_err("missing");
        assert_eq!(err.code, CODE_HUBS_PATHS_MALFORMED);
    }

    #[test]
    fn policy_from_json_reads_authority_pointer() {
        let value = json!({
            "gate_id": GATE_ID,
            "hubs_paths_authority": HUBS_PATHS_POINTER,
            "integ_head_ref_prefix": "integ/",
            "sole_owner_per_wave": true
        });
        let p = HubExclusivityPolicy::from_json(&value);
        assert_eq!(p.hubs_paths_authority, HUBS_PATHS_POINTER);
        assert!(p.sole_owner_per_wave);
    }

    #[test]
    fn open_pr_facts_from_simplified_fixture() {
        let doc = json!([
            {
                "number": 100,
                "head_ref_name": "integ/a",
                "files": ["Cargo.lock", "ci/x.rs"]
            }
        ]);
        let facts = open_pr_facts_from_json(&doc).expect("parse");
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].number, 100);
        assert!(facts[0].files.contains("Cargo.lock"));
    }

    #[test]
    fn open_pr_facts_from_github_shaped_fixture() {
        let doc = json!([
            {
                "number": 200,
                "head": { "ref": "integ/b" },
                "files": [{ "filename": "docs/CHANGELOG.md" }]
            }
        ]);
        let facts = open_pr_facts_from_json(&doc).expect("parse");
        assert_eq!(facts[0].head_ref_name, "integ/b");
        assert!(facts[0].files.contains("docs/CHANGELOG.md"));
    }

    #[test]
    fn evaluate_from_producer_docs_multi_own_refuses() {
        let policy = json!({
            "gate_id": GATE_ID,
            "hubs_paths_authority": HUBS_PATHS_POINTER,
            "integ_head_ref_prefix": "integ/",
            "sole_owner_per_wave": true
        });
        let envelopes = json!({
            "hubs": { "paths": ["Cargo.lock", "docs/CHANGELOG.md"] }
        });
        let open_prs = json!([
            {
                "number": 1,
                "head_ref_name": "integ/a",
                "files": ["Cargo.lock", "a.rs"]
            },
            {
                "number": 2,
                "head_ref_name": "integ/b",
                "files": ["Cargo.lock", "b.rs"]
            }
        ]);
        let report = evaluate_from_producer_docs(&policy, &envelopes, &open_prs);
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_MULTI_OWN_HUB && f.key == "Cargo.lock")
        );
    }
}
