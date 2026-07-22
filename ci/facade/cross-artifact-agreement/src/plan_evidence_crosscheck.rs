//! Plan-vs-evidence cross-check lane of the masterplan evidence gate
//! (masterplan v2 consolidation, Sub-AC 4.3).
//!
//! `evaluate_masterplan_v2_plan_evidence_drift` (lib.rs) proves evidence
//! POLICY: status claims carry evidence refs and legacy/local planning stores
//! are never laundered into evidence via a static deny list. This module
//! proves evidence CONTENT: every masterplan work-item status claim is
//! cross-checked against RECORDED completion evidence, failing closed on
//!
//! - **unevidenced 'done' claims** — a verified-completion status (`done`,
//!   `done-verified`, `complete`, …) or an `evidence-attached` external import
//!   whose evidence set contains no *recorded completion evidence class*:
//!   a merged commit (`git:<40-hex>`), a merged-PR / gate-run record
//!   (`https://github.com/<owner>/<repo>/pull|commit|actions/runs/…`), or a
//!   tracked product-completion packet under `evidence/`;
//! - **dangling evidence pointers** — a path-class evidence ref that does not
//!   resolve inside the tracked-tree resolution universe (the committed
//!   scm-facts face `tracked_paths`);
//! - **evidence pointing at retired surfaces** — a ref whose path matches a
//!   `surface_dispositions` row dispositioned `absorbed` or
//!   `retired-git-history-only` (exact rows, `#fragment` rows, and `/**` glob
//!   rows), derived MECHANICALLY from the masterplan itself, never from a
//!   hand list;
//! - **malformed recorded-evidence refs** — a truncated `git:` sha, a
//!   non-GitHub URL, or an unknown URI scheme masquerading as evidence.
//!
//! The evaluator is pure: the caller assembles the `plan_evidence_crosscheck`
//! corpus from the tree (`tracked_paths` from the committed scm-facts face) —
//! the evaluator itself does no I/O. Missing or malformed corpus sections fail
//! closed: a status claim the gate cannot cross-check is never admitted as
//! evidenced. Carve-outs live as DATA, never as evaluator branches.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::Finding;

/// Validator id recorded by `masterplan_v2.evidence_state_policy.plan_evidence_crosscheck`.
pub const PLAN_EVIDENCE_CROSSCHECK_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-plan-evidence-crosscheck";

/// The blocking violation code this lane emits.
pub const UNRECORDED_EVIDENCE_CODE: &str = "masterplan_plan_evidence_unrecorded";

const DISPOSITION_ABSORBED: &str = "absorbed";
const DISPOSITION_RETIRED_GIT_HISTORY_ONLY: &str = "retired-git-history-only";
const COMPLETION_PACKET_PREFIX: &str = "evidence/";
const GIT_REF_SCHEME: &str = "git:";
const SINGULAR_EVIDENCE_FIELDS: [&str; 5] = [
    "evidence_ref",
    "evidence_path",
    "merged_pr_ref",
    "gate_run_ref",
    "review_ref",
];

fn unrecorded(key: &str) -> Finding {
    Finding::new(UNRECORDED_EVIDENCE_CODE, key)
}

/// How a validated evidence ref counts toward a completion claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceClass {
    /// Recorded completion evidence: merged commit, merged-PR/gate-run
    /// record, or tracked product-completion packet under `evidence/`.
    Completion,
    /// Valid provenance/supporting evidence (e.g. a tracked source ref) that
    /// does NOT prove completion on its own.
    Supporting,
    /// The ref failed validation (a finding was already recorded).
    Invalid,
}

/// A retired surface derived from `masterplan_v2.surface_dispositions`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RetiredSurface {
    /// Normalized lowercase path (fragment and glob suffix stripped).
    path: String,
    /// `Some(fragment)` when the disposition row retires only a fragment of
    /// an otherwise-live artifact (e.g. `#v1-legacy-fragments`).
    fragment: Option<String>,
    /// True when the row is a `/**` glob: `path` is a prefix matcher.
    glob_prefix: bool,
}

/// Evaluate the plan-vs-evidence cross-check corpus against the masterplan.
/// `masterplan` is the `/specs/masterplan.json` document (or a fixture
/// mirroring it); `corpus` is assembled by the caller:
///
/// ```jsonc
/// {
///   "tracked_paths": ["evidence/goals/….json", "docs/MASTERPLAN.md", …]
/// }
/// ```
///
/// `tracked_paths` is the resolution universe for path-class evidence refs —
/// on the live tree it is the committed scm-facts face `tracked_paths`, so a
/// deleted or never-committed evidence record fails closed as a dangling
/// pointer.
pub fn evaluate_masterplan_plan_evidence_crosscheck(
    masterplan: &Value,
    corpus: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if !corpus.is_object() {
        findings.insert(unrecorded("<malformed-plan-evidence-crosscheck-corpus>"));
        return findings;
    }
    let Some(v2) = masterplan.get("masterplan_v2") else {
        findings.insert(unrecorded("<missing-masterplan_v2>"));
        return findings;
    };
    if !v2.is_object() {
        findings.insert(unrecorded("<malformed-masterplan_v2>"));
        return findings;
    }

    let Some(tracked) = collect_tracked_paths(corpus, &mut findings) else {
        return findings;
    };

    if !crosscheck_declaration_present(v2) {
        findings.insert(unrecorded(
            "masterplan_v2.evidence_state_policy.plan_evidence_crosscheck",
        ));
    }

    let retired = retired_surfaces(v2);

    evaluate_work_item_claims(v2.get("work_items"), &tracked, &retired, &mut findings);
    evaluate_external_import_claims(
        v2.get("external_work_item_claim_imports"),
        &tracked,
        &retired,
        &mut findings,
    );

    findings
}

/// The corpus resolution universe. `None` means the corpus is unusable and a
/// fail-closed sentinel has been recorded.
fn collect_tracked_paths(
    corpus: &Value,
    findings: &mut BTreeSet<Finding>,
) -> Option<BTreeSet<String>> {
    let Some(entries) = corpus.get("tracked_paths").and_then(Value::as_array) else {
        findings.insert(unrecorded("<missing-tracked-paths>"));
        return None;
    };
    let mut tracked = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let Some(path) = entry.as_str().map(str::trim).filter(|p| !p.is_empty()) else {
            findings.insert(unrecorded(&format!(
                "plan_evidence_crosscheck.tracked_paths[{index}]"
            )));
            continue;
        };
        tracked.insert(normalize_ref_path(path));
    }
    Some(tracked)
}

/// The masterplan must name this lane as a declared validator so the
/// cross-check cannot be silently unwired from the evidence-state policy.
fn crosscheck_declaration_present(v2: &Value) -> bool {
    v2.get("evidence_state_policy")
        .and_then(|policy| policy.get("plan_evidence_crosscheck"))
        .and_then(|crosscheck| crosscheck.get("validator"))
        .and_then(Value::as_str)
        == Some(PLAN_EVIDENCE_CROSSCHECK_VALIDATOR)
}

/// Derive the retired-surface matchers mechanically from
/// `surface_dispositions`: every row dispositioned `absorbed` or
/// `retired-git-history-only` retires its path as an evidence destination.
fn retired_surfaces(v2: &Value) -> Vec<RetiredSurface> {
    let Some(surfaces) = v2.get("surface_dispositions").and_then(Value::as_array) else {
        return Vec::new();
    };
    surfaces
        .iter()
        .filter(|surface| {
            surface
                .get("disposition")
                .and_then(Value::as_str)
                .is_some_and(|disposition| {
                    disposition == DISPOSITION_ABSORBED
                        || disposition == DISPOSITION_RETIRED_GIT_HISTORY_ONLY
                })
        })
        .filter_map(|surface| surface.get("path").and_then(Value::as_str))
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let (raw_path, fragment) = match path.split_once('#') {
                Some((base, fragment)) => (base, Some(fragment.to_ascii_lowercase())),
                None => (path, None),
            };
            let (raw_path, glob_prefix) = match raw_path.strip_suffix("**") {
                Some(prefix) => (prefix, true),
                None => (raw_path, false),
            };
            RetiredSurface {
                path: normalize_ref_path(raw_path).to_ascii_lowercase(),
                fragment,
                glob_prefix,
            }
        })
        .collect()
}

fn evaluate_work_item_claims(
    work_items: Option<&Value>,
    tracked: &BTreeSet<String>,
    retired: &[RetiredSurface],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(work_items) = work_items.and_then(Value::as_array) else {
        findings.insert(unrecorded("masterplan_v2.work_items"));
        return;
    };

    for (index, item) in work_items.iter().enumerate() {
        let key = non_empty_field(item, "id")
            .map(str::to_owned)
            .unwrap_or_else(|| format!("work_items[{index}]"));
        let scoped_key = format!("{key}@work_items[{index}]");
        let has_completion_evidence =
            validate_claim_evidence(item, &scoped_key, tracked, retired, findings);

        let claims_verified_completion = non_empty_field(item, "status")
            .is_some_and(is_verified_completion_claim)
            || non_empty_field(item, "evidence_state").is_some_and(is_verified_completion_claim);
        if claims_verified_completion && !has_completion_evidence {
            findings.insert(unrecorded(&format!(
                "{scoped_key}.recorded-completion-evidence"
            )));
        }
    }
}

fn evaluate_external_import_claims(
    claims: Option<&Value>,
    tracked: &BTreeSet<String>,
    retired: &[RetiredSurface],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(claims) = claims else {
        return;
    };
    let Some(claims) = claims.as_array() else {
        findings.insert(unrecorded("masterplan_v2.external_work_item_claim_imports"));
        return;
    };

    for (index, claim) in claims.iter().enumerate() {
        if !claim.is_object() {
            findings.insert(unrecorded(&format!(
                "external_work_item_claim_imports[{index}]"
            )));
            continue;
        }
        let key = non_empty_field(claim, "external_work_item_id")
            .or_else(|| non_empty_field(claim, "work_item_id"))
            .or_else(|| non_empty_field(claim, "claim_id"))
            .map(str::to_owned)
            .unwrap_or_else(|| format!("external_work_item_claim_imports[{index}]"));
        let scoped_key = format!("{key}@external_work_item_claim_imports[{index}]");
        let has_completion_evidence =
            validate_claim_evidence(claim, &scoped_key, tracked, retired, findings);

        let claims_evidence_attached = non_empty_field(claim, "evidence_state")
            .is_some_and(is_evidence_attached_claim)
            || non_empty_field(claim, "masterplan_status")
                .is_some_and(is_verified_completion_claim)
            || non_empty_field(claim, "status").is_some_and(is_verified_completion_claim);
        if claims_evidence_attached && !has_completion_evidence {
            findings.insert(unrecorded(&format!(
                "{scoped_key}.recorded-completion-evidence"
            )));
        }
    }
}

/// Validate every evidence ref carried by one claim (the `evidence_refs`
/// array plus the singular evidence fields). Returns true when at least one
/// ref is recorded completion evidence.
fn validate_claim_evidence(
    claim: &Value,
    scoped_key: &str,
    tracked: &BTreeSet<String>,
    retired: &[RetiredSurface],
    findings: &mut BTreeSet<Finding>,
) -> bool {
    let mut has_completion_evidence = false;

    if let Some(refs) = claim.get("evidence_refs") {
        match refs.as_array() {
            Some(refs) => {
                for (index, evidence_ref) in refs.iter().enumerate() {
                    let ref_key = format!("{scoped_key}.evidence_refs[{index}]");
                    let Some(evidence_ref) = evidence_ref
                        .as_str()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    else {
                        findings.insert(unrecorded(&format!("{ref_key}.malformed")));
                        continue;
                    };
                    let class =
                        classify_evidence_ref(evidence_ref, &ref_key, tracked, retired, findings);
                    has_completion_evidence |= class == EvidenceClass::Completion;
                }
            }
            None => {
                findings.insert(unrecorded(&format!("{scoped_key}.evidence_refs")));
            }
        }
    }

    for field in SINGULAR_EVIDENCE_FIELDS {
        let Some(raw) = claim.get(field) else {
            continue;
        };
        let ref_key = format!("{scoped_key}.{field}");
        let Some(evidence_ref) = raw.as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            findings.insert(unrecorded(&format!("{ref_key}.malformed")));
            continue;
        };
        let class = classify_evidence_ref(evidence_ref, &ref_key, tracked, retired, findings);
        has_completion_evidence |= class == EvidenceClass::Completion;
    }

    has_completion_evidence
}

/// Validate and classify one evidence ref. Recording a finding and returning
/// `Invalid` are always paired.
fn classify_evidence_ref(
    evidence_ref: &str,
    ref_key: &str,
    tracked: &BTreeSet<String>,
    retired: &[RetiredSurface],
    findings: &mut BTreeSet<Finding>,
) -> EvidenceClass {
    if let Some(sha) = evidence_ref.strip_prefix(GIT_REF_SCHEME) {
        if is_full_lowercase_hex_sha(sha) {
            return EvidenceClass::Completion;
        }
        findings.insert(unrecorded(&format!("{ref_key}.malformed-git-ref")));
        return EvidenceClass::Invalid;
    }

    if evidence_ref.starts_with("https://") || evidence_ref.starts_with("http://") {
        if is_recorded_github_evidence_url(evidence_ref) {
            return EvidenceClass::Completion;
        }
        findings.insert(unrecorded(&format!("{ref_key}.unrecorded-url")));
        return EvidenceClass::Invalid;
    }

    if evidence_ref.contains("://") {
        findings.insert(unrecorded(&format!("{ref_key}.unrecorded-scheme")));
        return EvidenceClass::Invalid;
    }

    // Path-class ref: `<path>[#fragment]`.
    let (raw_path, fragment) = match evidence_ref.split_once('#') {
        Some((base, fragment)) => (base, Some(fragment.to_ascii_lowercase())),
        None => (evidence_ref, None),
    };
    let normalized = normalize_ref_path(raw_path);

    if ref_path_is_retired(&normalized, fragment.as_deref(), retired) {
        findings.insert(unrecorded(&format!("{ref_key}.retired-surface")));
        return EvidenceClass::Invalid;
    }
    if !tracked.contains(&normalized) {
        findings.insert(unrecorded(&format!("{ref_key}.unresolved")));
        return EvidenceClass::Invalid;
    }
    if normalized.starts_with(COMPLETION_PACKET_PREFIX) {
        return EvidenceClass::Completion;
    }
    EvidenceClass::Supporting
}

fn ref_path_is_retired(
    normalized_path: &str,
    fragment: Option<&str>,
    retired: &[RetiredSurface],
) -> bool {
    let lower = normalized_path.to_ascii_lowercase();
    retired.iter().any(|surface| {
        if surface.glob_prefix {
            return lower.starts_with(&surface.path);
        }
        if lower != surface.path {
            return false;
        }
        match &surface.fragment {
            // Fragment-scoped row: only that fragment of the artifact is
            // retired; refs to other fragments (or the whole file) survive.
            Some(retired_fragment) => fragment == Some(retired_fragment.as_str()),
            None => true,
        }
    })
}

/// Strip leading `/` and `./` so refs, dispositions, and tracked paths agree
/// on repo-relative form. A `~` home prefix is intentionally preserved: it
/// can only ever match a retired `~/…` glob row, never the tracked tree.
fn normalize_ref_path(path: &str) -> String {
    let mut normalized = path.trim();
    loop {
        if let Some(stripped) = normalized.strip_prefix("./") {
            normalized = stripped;
            continue;
        }
        if normalized.starts_with('/') {
            normalized = normalized.trim_start_matches('/');
            continue;
        }
        break;
    }
    normalized.to_owned()
}

fn is_full_lowercase_hex_sha(sha: &str) -> bool {
    sha.len() == 40
        && sha
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// Recorded GitHub completion/gate evidence: a merged-PR, merged-commit, or
/// actions-run record under `https://github.com/<owner>/<repo>/…`.
fn is_recorded_github_evidence_url(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
    else {
        return false;
    };
    let rest = rest.split_once('#').map_or(rest, |(base, _)| base);
    let segments: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() < 4 {
        return false;
    }
    match segments[2] {
        "pull" => segments[3].chars().all(|c| c.is_ascii_digit()),
        "commit" => is_full_lowercase_hex_sha(segments[3]),
        "actions" => {
            segments.len() >= 5
                && segments[3] == "runs"
                && segments[4].chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_verified_completion_claim(status: &str) -> bool {
    matches!(
        normalized_token(status).as_str(),
        "done"
            | "complete"
            | "completed"
            | "doneverified"
            | "verifieddone"
            | "evidenceattacheddone"
    )
}

fn is_evidence_attached_claim(status: &str) -> bool {
    normalized_token(status) == "evidenceattached"
}

fn normalized_token(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn non_empty_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::{Value, json};

    use super::*;

    fn green_masterplan() -> Value {
        json!({
            "masterplan_v2": {
                "evidence_state_policy": {
                    "plan_evidence_crosscheck": {
                        "validator": PLAN_EVIDENCE_CROSSCHECK_VALIDATOR
                    }
                },
                "surface_dispositions": [
                    {"path": "/specs/masterplan.json", "disposition": "canonical-authority"},
                    {"path": "/specs/masterplan.json#v1-legacy-fragments", "disposition": "absorbed"},
                    {"path": "docs/ROADMAP.md", "disposition": "retired-git-history-only"},
                    {"path": ".omc/**", "disposition": "retired-git-history-only"},
                    {"path": "~/.gjc/**", "disposition": "retired-git-history-only"},
                    {"path": "plan/legacy-notes.md", "disposition": "retired-git-history-only"}
                ],
                "work_items": [
                    {
                        "id": "MPV2-0000",
                        "status": "done-verified",
                        "evidence_refs": [
                            "git:ee1773cc5a244f7a78715ab38d2676372dfb3e91",
                            "evidence/goals/proof-run.json#stage_review"
                        ]
                    },
                    {
                        "id": "MPV2-0001",
                        "status": "in-progress",
                        "evidence_refs": [
                            "cloud/gates/app/src/lib.rs#evaluator"
                        ]
                    }
                ],
                "external_work_item_claim_imports": [
                    {
                        "external_work_item_id": "t_0001",
                        "evidence_state": "evidence-attached",
                        "evidence_refs": [
                            "https://github.com/jason931225/oyatie/pull/1054#issuecomment-1"
                        ]
                    },
                    {
                        "external_work_item_id": "t_0002",
                        "evidence_state": "claimed-done-unverified",
                        "evidence_refs": []
                    }
                ]
            }
        })
    }

    fn green_corpus() -> Value {
        json!({
            "tracked_paths": [
                "evidence/goals/proof-run.json",
                "cloud/gates/app/src/lib.rs",
                "plan/legacy-notes.md",
                "specs/masterplan.json"
            ]
        })
    }

    fn keys(findings: &std::collections::BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|f| f.key.clone()).collect()
    }

    #[test]
    fn green_masterplan_and_corpus_produce_no_findings() {
        let findings =
            evaluate_masterplan_plan_evidence_crosscheck(&green_masterplan(), &green_corpus());
        assert!(findings.is_empty(), "expected green, got {findings:?}");
    }

    #[test]
    fn every_finding_uses_the_single_unrecorded_code() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] = json!([
            "git:short",
            "https://example.com/build/1",
            "ftp://host/file",
            "evidence/goals/missing.json",
            "docs/ROADMAP.md",
            42
        ]);
        masterplan["masterplan_v2"]["evidence_state_policy"] = json!({});
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert!(!findings.is_empty());
        assert!(
            findings
                .iter()
                .all(|finding| finding.code == UNRECORDED_EVIDENCE_CODE),
            "single-code contract broken: {findings:?}"
        );
    }

    #[test]
    fn malformed_corpus_fails_closed() {
        let findings =
            evaluate_masterplan_plan_evidence_crosscheck(&green_masterplan(), &json!([]));
        assert_eq!(
            keys(&findings),
            vec!["<malformed-plan-evidence-crosscheck-corpus>"]
        );
    }

    #[test]
    fn missing_masterplan_v2_fails_closed() {
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&json!({}), &green_corpus());
        assert_eq!(keys(&findings), vec!["<missing-masterplan_v2>"]);
    }

    #[test]
    fn missing_tracked_paths_fails_closed() {
        let findings = evaluate_masterplan_plan_evidence_crosscheck(
            &green_masterplan(),
            &json!({"tracked_paths": "not-an-array"}),
        );
        assert_eq!(keys(&findings), vec!["<missing-tracked-paths>"]);
    }

    #[test]
    fn non_string_tracked_path_entries_are_flagged() {
        let findings = evaluate_masterplan_plan_evidence_crosscheck(
            &green_masterplan(),
            &json!({"tracked_paths": [
                7,
                "evidence/goals/proof-run.json",
                "cloud/gates/app/src/lib.rs",
                "plan/legacy-notes.md",
                "specs/masterplan.json"
            ]}),
        );
        assert_eq!(
            keys(&findings),
            vec!["plan_evidence_crosscheck.tracked_paths[0]"]
        );
    }

    #[test]
    fn missing_crosscheck_declaration_is_flagged() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["evidence_state_policy"] = json!({});
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec!["masterplan_v2.evidence_state_policy.plan_evidence_crosscheck"]
        );
    }

    #[test]
    fn missing_work_items_fail_closed() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]
            .as_object_mut()
            .unwrap()
            .remove("work_items");
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(keys(&findings), vec!["masterplan_v2.work_items"]);
    }

    #[test]
    fn done_claim_with_only_supporting_evidence_is_unrecorded() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] =
            json!(["cloud/gates/app/src/lib.rs#evaluator"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec!["MPV2-0000@work_items[0].recorded-completion-evidence"]
        );
    }

    #[test]
    fn done_claim_with_no_evidence_is_unrecorded() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] = json!([]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec!["MPV2-0000@work_items[0].recorded-completion-evidence"]
        );
    }

    #[test]
    fn dangling_evidence_path_is_unresolved_and_discounted() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] =
            json!(["evidence/goals/deleted-packet.json#stage_review"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec![
                "MPV2-0000@work_items[0].evidence_refs[0].unresolved",
                "MPV2-0000@work_items[0].recorded-completion-evidence",
            ]
        );
    }

    #[test]
    fn evidence_at_exact_retired_surface_is_flagged() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][1]["evidence_refs"] =
            json!(["plan/legacy-notes.md#closing-status"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec!["MPV2-0001@work_items[1].evidence_refs[0].retired-surface"]
        );
    }

    #[test]
    fn evidence_under_retired_glob_surface_is_flagged() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][1]["evidence_refs"] =
            json!([".omc/ultragoal/goals.json", "~/.gjc/state/run.json"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec![
                "MPV2-0001@work_items[1].evidence_refs[0].retired-surface",
                "MPV2-0001@work_items[1].evidence_refs[1].retired-surface",
            ]
        );
    }

    #[test]
    fn fragment_scoped_retirement_spares_other_fragments() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][1]["evidence_refs"] = json!([
            "specs/masterplan.json#masterplan_v2",
            "/specs/masterplan.json#v1-legacy-fragments"
        ]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec!["MPV2-0001@work_items[1].evidence_refs[1].retired-surface"]
        );
    }

    #[test]
    fn malformed_git_ref_is_flagged_and_discounted() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] = json!(["git:abc123"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec![
                "MPV2-0000@work_items[0].evidence_refs[0].malformed-git-ref",
                "MPV2-0000@work_items[0].recorded-completion-evidence",
            ]
        );
    }

    #[test]
    fn non_github_url_is_flagged_and_discounted() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] =
            json!(["https://example.com/ci/run/9"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec![
                "MPV2-0000@work_items[0].evidence_refs[0].unrecorded-url",
                "MPV2-0000@work_items[0].recorded-completion-evidence",
            ]
        );
    }

    #[test]
    fn github_actions_run_and_commit_urls_count_as_recorded_completion_evidence() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] = json!([
            "https://github.com/jason931225/oyatie/actions/runs/28447071285/job/84304557825",
            "https://github.com/jason931225/oyatie/commit/ee1773cc5a244f7a78715ab38d2676372dfb3e91"
        ]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert!(findings.is_empty(), "expected green, got {findings:?}");
    }

    #[test]
    fn external_claim_with_only_supporting_evidence_is_unrecorded() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["external_work_item_claim_imports"][0]["evidence_refs"] =
            json!(["cloud/gates/app/src/lib.rs#evaluator"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec!["t_0001@external_work_item_claim_imports[0].recorded-completion-evidence"]
        );
    }

    #[test]
    fn external_claim_tracked_completion_packet_satisfies_evidence_attachment() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["external_work_item_claim_imports"][0]["evidence_refs"] =
            json!(["evidence/goals/proof-run.json"]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert!(findings.is_empty(), "expected green, got {findings:?}");
    }

    #[test]
    fn malformed_external_imports_fail_closed() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["external_work_item_claim_imports"] = json!("not-an-array");
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec!["masterplan_v2.external_work_item_claim_imports"]
        );

        masterplan["masterplan_v2"]["external_work_item_claim_imports"] = json!([42]);
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(keys(&findings), vec!["external_work_item_claim_imports[0]"]);
    }

    #[test]
    fn singular_merged_pr_field_counts_and_is_validated() {
        let mut masterplan = green_masterplan();
        masterplan["masterplan_v2"]["work_items"][0]["evidence_refs"] = json!([]);
        masterplan["masterplan_v2"]["work_items"][0]["merged_pr_ref"] =
            json!("https://github.com/jason931225/oyatie/pull/1121");
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert!(findings.is_empty(), "expected green, got {findings:?}");

        masterplan["masterplan_v2"]["work_items"][0]["merged_pr_ref"] = json!("pull-1121");
        let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &green_corpus());
        assert_eq!(
            keys(&findings),
            vec![
                "MPV2-0000@work_items[0].merged_pr_ref.unresolved",
                "MPV2-0000@work_items[0].recorded-completion-evidence",
            ]
        );
    }
}
