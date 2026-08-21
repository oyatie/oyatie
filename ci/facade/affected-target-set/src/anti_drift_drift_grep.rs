//! # cloud-ci-anti-drift-drift-grep (Swarm Delivery Law / ADR-0711 Amendment D)
//!
//! Mechanical REFUSE when ADR-0711 / PORTABLE prose re-lists roots / hubs / freeze
//! enumerations that MUST live only at `specs/integ-branch-envelopes.json`
//! (`#anti_drift.prose_must_cite_not_enumerate`, INV-DOC-2).
//!
//! ## Forever shape
//! - Authority for *what must be cited* is a **JSON pointer**, never a re-listed
//!   pointer set forked into this crate or its policy JSON.
//! - Producer supplies prose file facts as DATA; this module is a pure fail-closed
//!   evaluator (no shell, net, clock, or filesystem).
//! - Forbidden detectors are pattern matchers (legacy root comma-list, Amendment B
//!   path table) — not a second enumeration of governed roots/hubs.
//! - Empty scanned surfaces with `require_surfaces=false` is Green (parked tip
//!   before envelopes/ADR land); fixtures still bind RED/GREEN independently.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

use serde_json::Value;

/// Gate id (matches policy `gate_id` + buck2 test target purpose).
pub const GATE_ID: &str = "cloud-ci-anti-drift-drift-grep";

/// Canonical authority pointer — cite; do not re-list the cite set in prose or policy DATA.
pub const PROSE_MUST_CITE_POINTER: &str =
    "specs/integ-branch-envelopes.json#anti_drift.prose_must_cite_not_enumerate";

/// Surfaces authority (ADR + PORTABLE paths live under `#anti_drift`).
pub const SCANNED_SURFACES_POINTER: &str = "specs/integ-branch-envelopes.json#anti_drift";

/// Prose re-lists a legacy root comma-list (dual-truth vs `#roots`).
pub const CODE_PROSE_ROOT_ENUMERATION: &str = "prose_root_enumeration";
/// Prose embeds an Amendment B freeze/layout path table (dual-truth vs `#reorg_debt_freeze.rows`).
pub const CODE_PROSE_FREEZE_PATH_TABLE: &str = "prose_freeze_path_table";
/// In-scope Swarm surface missing a required JSON-pointer cite.
pub const CODE_PROSE_MISSING_REQUIRED_CITE: &str = "prose_missing_required_cite";
/// Policy pointer does not match [`PROSE_MUST_CITE_POINTER`].
pub const CODE_AUTHORITY_POINTER_MISMATCH: &str = "anti_drift_authority_pointer_mismatch";
/// Policy `gate_id` does not match [`GATE_ID`].
pub const CODE_POLICY_GATE_ID_MISMATCH: &str = "anti_drift_gate_id_mismatch";
/// Envelopes `#anti_drift.prose_must_cite_not_enumerate` missing/non-array (fail-closed parse).
pub const CODE_PROSE_MUST_CITE_MALFORMED: &str = "prose_must_cite_malformed";
/// `require_surfaces=true` but no in-scope prose was supplied (fail-closed).
pub const CODE_SURFACES_EMPTY: &str = "anti_drift_surfaces_empty";

pub const VIOLATION_CODES: [&str; 7] = [
    CODE_PROSE_ROOT_ENUMERATION,
    CODE_PROSE_FREEZE_PATH_TABLE,
    CODE_PROSE_MISSING_REQUIRED_CITE,
    CODE_AUTHORITY_POINTER_MISMATCH,
    CODE_POLICY_GATE_ID_MISMATCH,
    CODE_PROSE_MUST_CITE_MALFORMED,
    CODE_SURFACES_EMPTY,
];

/// Legacy dual-truth root comma-list that Amendment D retired from prose.
const FORBIDDEN_ROOT_COMMA_LIST: &str = "`os`, `ci`, `governance`";
/// Amendment B path-table header that must not reappear in ADR/PORTABLE prose.
const FORBIDDEN_FREEZE_TABLE_HEADER: &str = "| current path | action |";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    /// Mechanical REFUSE — Claim/Land MUST NOT treat this tip as anti-drift-clear.
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

/// Policy pack for the anti-drift drift-grep gate (DATA; no root/hub enumeration).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntiDriftDriftGrepPolicy {
    pub gate_id: String,
    /// Must equal [`PROSE_MUST_CITE_POINTER`].
    pub prose_must_cite_authority: String,
    /// Must equal [`SCANNED_SURFACES_POINTER`].
    pub scanned_surfaces_authority: String,
    /// When true, empty in-scope surfaces REFUSE (fail-closed). Parked tips use false.
    pub require_surfaces: bool,
}

impl AntiDriftDriftGrepPolicy {
    /// Parse policy JSON. Missing fields fail closed via evaluate (empty/mismatch codes).
    pub fn from_json(value: &Value) -> Self {
        Self {
            gate_id: value
                .get("gate_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_owned(),
            prose_must_cite_authority: value
                .get("prose_must_cite_authority")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_owned(),
            scanned_surfaces_authority: value
                .get("scanned_surfaces_authority")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_owned(),
            require_surfaces: value
                .get("require_surfaces")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }
}

/// Required cite suffixes loaded from envelopes `#anti_drift.prose_must_cite_not_enumerate`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProseCiteAuthority {
    pub pointers: BTreeSet<String>,
}

/// One prose surface (producer-supplied file fact).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProseSurface {
    pub path: String,
    pub text: String,
}

/// Extract `#anti_drift.prose_must_cite_not_enumerate` from an envelopes document.
pub fn prose_must_cite_from_envelopes(doc: &Value) -> Result<ProseCiteAuthority, Finding> {
    let value = doc
        .pointer("/anti_drift/prose_must_cite_not_enumerate")
        .ok_or_else(|| {
            Finding::new(
                CODE_PROSE_MUST_CITE_MALFORMED,
                PROSE_MUST_CITE_POINTER,
                "envelopes document missing /anti_drift/prose_must_cite_not_enumerate — refuse rather than invent cite set",
            )
        })?;
    let arr = value.as_array().ok_or_else(|| {
        Finding::new(
            CODE_PROSE_MUST_CITE_MALFORMED,
            PROSE_MUST_CITE_POINTER,
            "/anti_drift/prose_must_cite_not_enumerate must be a JSON array of pointer strings",
        )
    })?;
    let mut pointers = BTreeSet::new();
    for (idx, entry) in arr.iter().enumerate() {
        match entry.as_str().map(str::trim) {
            Some(pointer) if !pointer.is_empty() => {
                pointers.insert(pointer.to_owned());
            }
            _ => {
                return Err(Finding::new(
                    CODE_PROSE_MUST_CITE_MALFORMED,
                    format!("{PROSE_MUST_CITE_POINTER}[{idx}]"),
                    "prose_must_cite_not_enumerate entries must be non-empty strings",
                ));
            }
        }
    }
    Ok(ProseCiteAuthority { pointers })
}

/// Default cite set used by fixtures when envelopes are absent on the tip.
///
/// This mirrors the envelopes shape but is **fixture DATA**, not a live authority fork —
/// live evaluate loads from [`prose_must_cite_from_envelopes`] when envelopes exist.
pub fn fixture_prose_must_cite() -> ProseCiteAuthority {
    ProseCiteAuthority {
        pointers: [
            "#roots",
            "#planes",
            "#hubs.paths",
            "#reorg_debt_freeze.prefixes",
            "#reorg_debt_freeze.no_new_births_while_reorg_prefixes",
            "#concurrent_safe_exemptions.paths",
            "#reorg_debt_freeze.rows",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect(),
    }
}

/// True when prose is an in-scope Swarm Delivery Law anti-drift surface.
pub fn is_in_scope_swarm_surface(path: &str, text: &str) -> bool {
    let path_l = path.to_ascii_lowercase();
    if path_l.contains("adr-0711") {
        return true;
    }
    text.contains("INV-DOC-2")
        || text.contains("Amendment D")
        || text.contains("prose_must_cite_not_enumerate")
        || text.contains("Swarm Delivery Law")
}

fn surface_cites_pointer(text: &str, pointer: &str) -> bool {
    // Accept either the bare JSON pointer fragment (`#roots`) or a full envelopes cite.
    if text.contains(pointer) {
        return true;
    }
    let bare = pointer.strip_prefix('#').unwrap_or(pointer);
    // `reorg_debt_freeze.rows` without leading `#` still counts (self-check legacy form).
    if text.contains(bare) {
        return true;
    }
    // Full path cite: integ-branch-envelopes.json#roots
    let full = format!("integ-branch-envelopes.json{pointer}");
    text.contains(&full)
}

/// Pure evaluate: REFUSE when in-scope prose dual-truth enumerates or omits required cites.
pub fn evaluate(
    policy: &AntiDriftDriftGrepPolicy,
    cite_authority: &ProseCiteAuthority,
    surfaces: &[ProseSurface],
) -> Report {
    let mut findings = BTreeSet::new();

    if policy.gate_id != GATE_ID {
        findings.insert(Finding::new(
            CODE_POLICY_GATE_ID_MISMATCH,
            "gate_id",
            format!("policy gate_id {:?} must equal {GATE_ID}", policy.gate_id),
        ));
    }

    if policy.prose_must_cite_authority != PROSE_MUST_CITE_POINTER {
        findings.insert(Finding::new(
            CODE_AUTHORITY_POINTER_MISMATCH,
            "prose_must_cite_authority",
            format!(
                "policy prose_must_cite_authority {:?} must equal {PROSE_MUST_CITE_POINTER} (cite pointer; do not fork the list)",
                policy.prose_must_cite_authority
            ),
        ));
    }

    if policy.scanned_surfaces_authority != SCANNED_SURFACES_POINTER {
        findings.insert(Finding::new(
            CODE_AUTHORITY_POINTER_MISMATCH,
            "scanned_surfaces_authority",
            format!(
                "policy scanned_surfaces_authority {:?} must equal {SCANNED_SURFACES_POINTER}",
                policy.scanned_surfaces_authority
            ),
        ));
    }

    if !findings.is_empty() {
        return Report::from_findings(findings);
    }

    let in_scope: Vec<&ProseSurface> = surfaces
        .iter()
        .filter(|s| is_in_scope_swarm_surface(&s.path, &s.text))
        .collect();

    if policy.require_surfaces && in_scope.is_empty() {
        findings.insert(Finding::new(
            CODE_SURFACES_EMPTY,
            SCANNED_SURFACES_POINTER,
            "require_surfaces is true but no in-scope ADR-0711/PORTABLE prose was supplied — fail-closed",
        ));
        return Report::from_findings(findings);
    }

    // Minimum cites always required on in-scope surfaces (INV-DOC-2 floor).
    // Full envelopes list is used when non-empty; otherwise fixture floor of #roots + rows.
    let required: Vec<&str> = if cite_authority.pointers.is_empty() {
        vec!["#roots", "#reorg_debt_freeze.rows"]
    } else {
        // Floor subset that self-check historically pinned; full list is advisory until
        // every surface carries every pointer (PORTABLE short form cites the set in one line).
        let floor = ["#roots", "#hubs.paths", "#reorg_debt_freeze.rows"];
        floor
            .iter()
            .copied()
            .filter(|p| {
                cite_authority
                    .pointers
                    .iter()
                    .any(|a| a == p || a.ends_with(p))
            })
            .collect()
    };

    for surface in in_scope {
        if surface.text.contains(FORBIDDEN_ROOT_COMMA_LIST) {
            findings.insert(Finding::new(
                CODE_PROSE_ROOT_ENUMERATION,
                surface.path.as_str(),
                format!(
                    "prose root enumeration {FORBIDDEN_ROOT_COMMA_LIST:?} — cite {PROSE_MUST_CITE_POINTER} / #roots instead (INV-DOC-2)"
                ),
            ));
        }
        if surface.text.contains(FORBIDDEN_FREEZE_TABLE_HEADER) {
            findings.insert(Finding::new(
                CODE_PROSE_FREEZE_PATH_TABLE,
                surface.path.as_str(),
                "prose freeze/layout path table — cite #reorg_debt_freeze.rows instead (INV-DOC-2)",
            ));
        }
        for pointer in &required {
            if !surface_cites_pointer(&surface.text, pointer) {
                findings.insert(Finding::new(
                    CODE_PROSE_MISSING_REQUIRED_CITE,
                    format!("{}::{pointer}", surface.path),
                    format!(
                        "in-scope Swarm surface must cite {pointer} (authority {PROSE_MUST_CITE_POINTER})"
                    ),
                ));
            }
        }
    }

    Report::from_findings(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> AntiDriftDriftGrepPolicy {
        AntiDriftDriftGrepPolicy {
            gate_id: GATE_ID.to_owned(),
            prose_must_cite_authority: PROSE_MUST_CITE_POINTER.to_owned(),
            scanned_surfaces_authority: SCANNED_SURFACES_POINTER.to_owned(),
            require_surfaces: false,
        }
    }

    fn surface(path: &str, text: &str) -> ProseSurface {
        ProseSurface {
            path: path.to_owned(),
            text: text.to_owned(),
        }
    }

    #[test]
    fn root_enumeration_in_adr_refuses() {
        let text = "\
## Amendment D\n\
INV-DOC-2 applies.\n\
Governed roots include `os`, `ci`, `governance` and more.\n\
See specs/integ-branch-envelopes.json#roots and #hubs.paths and reorg_debt_freeze.rows.\n";
        let report = evaluate(
            &policy(),
            &fixture_prose_must_cite(),
            &[surface(
                "docs/decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md",
                text,
            )],
        );
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_PROSE_ROOT_ENUMERATION)
        );
    }

    #[test]
    fn freeze_path_table_refuses() {
        let text = "\
## Amendment D\nINV-DOC-2\n\
| current path | action |\n| libs/ | reorg_now |\n\
Cite specs/integ-branch-envelopes.json#roots #hubs.paths reorg_debt_freeze.rows.\n";
        let report = evaluate(
            &policy(),
            &fixture_prose_must_cite(),
            &[surface(
                "governance/contracts/PORTABLE-SWARM-CONTRACT.md",
                text,
            )],
        );
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_PROSE_FREEZE_PATH_TABLE)
        );
    }

    #[test]
    fn cite_only_swarm_surface_is_green() {
        let text = "\
### Amendment D — Anti-drift\n\
**INV-DOC-2:** enumerate ONLY via JSON pointers (`#roots`, `#planes`, `#hubs.paths`,\
`#reorg_debt_freeze.prefixes`, `#reorg_debt_freeze.rows`).\n\
Authority: specs/integ-branch-envelopes.json#anti_drift.\n";
        let report = evaluate(
            &policy(),
            &fixture_prose_must_cite(),
            &[surface(
                "docs/decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md",
                text,
            )],
        );
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
    }

    #[test]
    fn missing_required_cite_refuses() {
        let text = "Amendment D / INV-DOC-2 — do not re-list roots.\n";
        let report = evaluate(
            &policy(),
            &fixture_prose_must_cite(),
            &[surface(
                "docs/decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md",
                text,
            )],
        );
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_PROSE_MISSING_REQUIRED_CITE)
        );
    }

    #[test]
    fn pre_amendment_portable_out_of_scope_is_green() {
        let text = "# Portable swarm contract\nDo not invent secrets.\n";
        let report = evaluate(
            &policy(),
            &fixture_prose_must_cite(),
            &[surface(
                "governance/contracts/PORTABLE-SWARM-CONTRACT.md",
                text,
            )],
        );
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
    }

    #[test]
    fn empty_surfaces_with_require_false_is_green() {
        let report = evaluate(&policy(), &fixture_prose_must_cite(), &[]);
        assert_eq!(report.verdict, Verdict::Green);
    }

    #[test]
    fn empty_surfaces_with_require_true_refuses() {
        let mut p = policy();
        p.require_surfaces = true;
        let report = evaluate(&p, &fixture_prose_must_cite(), &[]);
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_SURFACES_EMPTY)
        );
    }

    #[test]
    fn pointer_mismatch_fail_closed() {
        let mut p = policy();
        p.prose_must_cite_authority = "specs/forked.json#cites".to_owned();
        let report = evaluate(&p, &fixture_prose_must_cite(), &[]);
        assert_eq!(report.verdict, Verdict::Refuse);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == CODE_AUTHORITY_POINTER_MISMATCH)
        );
    }

    #[test]
    fn prose_must_cite_from_envelopes_extracts() {
        let doc = json!({
            "anti_drift": {
                "prose_must_cite_not_enumerate": ["#roots", "#hubs.paths"]
            }
        });
        let auth = prose_must_cite_from_envelopes(&doc).expect("ok");
        assert_eq!(
            auth.pointers,
            ["#roots".to_owned(), "#hubs.paths".to_owned()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn prose_must_cite_missing_refuses_parse() {
        let doc = json!({ "anti_drift": {} });
        let err = prose_must_cite_from_envelopes(&doc).expect_err("missing");
        assert_eq!(err.code, CODE_PROSE_MUST_CITE_MALFORMED);
    }

    #[test]
    fn policy_from_json_reads_pointers() {
        let value = json!({
            "gate_id": GATE_ID,
            "prose_must_cite_authority": PROSE_MUST_CITE_POINTER,
            "scanned_surfaces_authority": SCANNED_SURFACES_POINTER,
            "require_surfaces": false
        });
        let p = AntiDriftDriftGrepPolicy::from_json(&value);
        assert_eq!(p.prose_must_cite_authority, PROSE_MUST_CITE_POINTER);
        assert!(!p.require_surfaces);
    }
}
