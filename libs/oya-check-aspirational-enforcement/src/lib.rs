//! Aspirational enforcement claim validator.
//!
//! The kernel is intentionally I/O-free. Runners provide corpus documents and
//! the known enforcement surfaces from crates, workflows, quality lanes, and branch
//! protection. The validator fails only explicit binding claims; advisory and
//! proposed lane mentions remain allowed.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `panic!()`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspirationalDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KnownEnforcementSurfaces {
    /// Check-gate CAPABILITY IDENTITIES (`check-<topic>`) — the `capability:`
    /// facet of `registry/catalog/<crate>.yaml`, NOT a crate-name prefix. The
    /// facet is invariant under relocation (`libs/oya-check-foo` ->
    /// `governance/check/foo`), so a rename cannot empty this set the way a
    /// `starts_with("oya-check-")` directory scan could.
    /// data_class: INTERNAL_ONLY
    pub check_capabilities: BTreeSet<String>,
    pub workflow_contexts: BTreeSet<String>, // data_class: INTERNAL_ONLY
    pub quality_lane_contexts: BTreeSet<String>, // data_class: INTERNAL_ONLY
    pub branch_required_contexts: BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// All lane ids DECLARED in the quality-lane registry (any status). A
    /// binding enforcement claim that references a governance lane NOT declared
    /// here is treated as advisory/planned (a future lane), not a violation —
    /// only declared-but-unresolved lanes fail (ADR-0362 (a): planned refs are
    /// advisory regardless of prefix). data_class: INTERNAL_ONLY
    pub declared_lane_ids: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspirationalReport {
    pub documents_checked: usize, // data_class: INTERNAL_ONLY
    pub lines_checked: usize,     // data_class: INTERNAL_ONLY
    pub binding_mentions: usize,  // data_class: INTERNAL_ONLY
    /// Check-capability identities observed anywhere in the corpus, binding or
    /// not. This is the gate's MEASURED SITE COUNT for the check family. A
    /// full-repo scan that observes zero sites is a broken scan, not a clean
    /// repo; runners MUST fail on it rather than report clean.
    /// data_class: INTERNAL_ONLY
    pub check_sites: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspirationalViolation {
    pub path: String,                // data_class: INTERNAL_ONLY
    pub line: usize,                 // data_class: INTERNAL_ONLY
    pub token: String,               // data_class: INTERNAL_ONLY
    pub kind: AspirationalIssueKind, // data_class: INTERNAL_ONLY
    pub summary: String,             // data_class: INTERNAL_ONLY
    pub fix: String,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AspirationalIssueKind {
    MissingCrate,
    MissingWorkflow,
    MissingQualityLane,
    MissingRequiredContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingBindingContext {
    indent: usize,
    requires_branch_context: bool,
}

impl fmt::Display for AspirationalViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{} {:?}: {} ({}) fix: {}",
            self.path, self.line, self.kind, self.summary, self.token, self.fix
        )
    }
}

pub fn validate_aspirational_enforcement<D>(
    documents: D,
    known: &KnownEnforcementSurfaces,
) -> Result<AspirationalReport, Vec<AspirationalViolation>>
where
    D: IntoIterator<Item = AspirationalDocument>,
{
    let mut documents_checked = 0usize;
    let mut lines_checked = 0usize;
    let mut binding_mentions = 0usize;
    let mut check_sites = 0usize;
    let mut violations = Vec::new();

    for document in documents {
        documents_checked += 1;
        let mut pending_binding_context = None::<PendingBindingContext>;
        for (index, raw_line) in document.contents.lines().enumerate() {
            lines_checked += 1;
            let line_number = index + 1;
            let lower = raw_line.to_ascii_lowercase();
            let indent = raw_line.len() - raw_line.trim_start().len();
            let stripped = raw_line.trim_start();
            if let Some(pending) = pending_binding_context
                && !stripped.is_empty()
                && indent <= pending.indent
                && !stripped.starts_with("- ")
                && !starts_multiline_binding_header(&lower)
            {
                pending_binding_context = None;
            }
            let tokens = enforcement_tokens(raw_line);
            check_sites += tokens
                .iter()
                .filter(|token| token.starts_with("check-"))
                .count();
            let advisory = is_advisory_context(&lower);
            let current_line_is_binding = is_binding_context(&lower) && !advisory;
            let pending_context = pending_binding_context.filter(|_| !advisory);
            if starts_multiline_binding_header(&lower) && !advisory {
                pending_binding_context = Some(PendingBindingContext {
                    indent,
                    requires_branch_context: requires_branch_context(&lower),
                });
            }
            if tokens.is_empty() || (!current_line_is_binding && pending_context.is_none()) {
                continue;
            }
            let branch_context_required = requires_branch_context(&lower)
                || pending_context
                    .map(|pending| pending.requires_branch_context)
                    .unwrap_or(false);
            for token in tokens {
                binding_mentions += 1;
                if token.starts_with("check-") {
                    if !known.check_capabilities.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token: token.clone(),
                            kind: AspirationalIssueKind::MissingCrate,
                            summary: "binding claim references a missing check capability"
                                .to_string(),
                            fix: "add the catalog capability facet or mark the claim advisory/proposed".to_string(),
                        });
                    }
                } else if token.starts_with("oya-governance-")
                    && known.declared_lane_ids.contains(&token)
                {
                    // Only DECLARED governance lanes are validated. A binding
                    // claim referencing an undeclared oya-governance-* lane is a
                    // planned/future lane => advisory (falls through, no
                    // violation), per ADR-0362 (a).
                    if !known.workflow_contexts.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token: token.clone(),
                            kind: AspirationalIssueKind::MissingWorkflow,
                            summary: "binding claim references a missing workflow/job context"
                                .to_string(),
                            fix: "add the workflow/job or mark the claim advisory/proposed"
                                .to_string(),
                        });
                    }
                    if !known.quality_lane_contexts.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token: token.clone(),
                            kind: AspirationalIssueKind::MissingQualityLane,
                            summary:
                                "binding claim references a missing active quality-lane registry row"
                                    .to_string(),
                            fix: "add the active quality-lane registry row or mark the claim advisory/proposed"
                                .to_string(),
                        });
                    }
                    if branch_context_required && !known.branch_required_contexts.contains(&token) {
                        violations.push(AspirationalViolation {
                            path: document.path.clone(),
                            line: line_number,
                            token,
                            kind: AspirationalIssueKind::MissingRequiredContext,
                            summary: "required-check claim is absent from branch protection"
                                .to_string(),
                            fix: "add the exact context to branch protection or remove the required-check claim"
                                .to_string(),
                        });
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(AspirationalReport {
            documents_checked,
            lines_checked,
            binding_mentions,
            check_sites,
        })
    } else {
        Err(violations)
    }
}

/// Maximal `[A-Za-z0-9_-]` runs, i.e. identifier tokens delimited at real word
/// boundaries. Anchoring on runs (rather than a bare substring `find`) is what
/// lets `check-<topic>` be recognised WITHOUT substring-colliding inside
/// `oya-check-<topic>`: a run yields at most one identity.
fn identifier_runs(line: &str) -> impl Iterator<Item = &str> {
    line.split(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '-' || character == '_')
    })
    .filter(|run| !run.is_empty())
}

/// Positions in which a bare, unbranded `check-<topic>` reads as a claim about
/// a gate rather than as a word in a sentence: the scalar VALUE of the line,
/// and the contents of any backtick code span. Corpus prose that collides with
/// the identity shape (`check-fail`, `check-family`, `check-runs`,
/// `check-run-rename`) sits in neither.
fn claim_positions(line: &str) -> BTreeSet<&str> {
    let mut positions = BTreeSet::from([scalar_value(line)]);
    let mut rest = line;
    while let Some((_, after_open)) = rest.split_once('`') {
        let Some((span, after_close)) = after_open.split_once('`') else {
            break;
        };
        positions.insert(span.trim());
        rest = after_close;
    }
    positions
}

/// The scalar a line asserts, with a leading list dash, a `key:` prefix and
/// surrounding quotes removed.
fn scalar_value(line: &str) -> &str {
    let mut value = line.trim();
    value = value.strip_prefix("- ").unwrap_or(value).trim();
    if let Some((key, rest)) = value.split_once(':')
        && !key.contains(char::is_whitespace)
    {
        value = rest.trim();
    }
    value.trim_matches('"').trim_matches('\'').trim()
}

fn enforcement_tokens(line: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    let positions = claim_positions(line);
    for run in identifier_runs(line) {
        // Governance lanes keep the branded `oya-governance-*` id: that string
        // IS the literal status context compared against branch protection and
        // the lane registry, so it is not a relocation target.
        if let Some(topic) = run.strip_prefix("oya-governance-") {
            insert_token(&mut tokens, "oya-governance-", topic);
            continue;
        }
        // Check gates are keyed on CAPABILITY IDENTITY (`check-<topic>`, the
        // `capability:` facet of registry/catalog/<crate>.yaml), never on a
        // crate-name prefix. The legacy branded `oya-check-<topic>` spelling
        // and the relocated bare `check-<topic>` spelling normalise to the
        // same identity, so relocating libs/oya-check-* under governance/check/
        // cannot empty this scan.
        if let Some(topic) = run.strip_prefix("oya-check-") {
            // Brand-anchored, so unambiguous even mid-prose ("enforced by
            // `oya-check-http-stack`").
            insert_token(&mut tokens, "check-", topic);
            continue;
        }
        // The bare form is NOT brand-anchored and collides with ordinary
        // hyphenated English and GitHub vocabulary — `check-fail`,
        // `check-family`, `check-runs`, `check-run-rename` all appear in the
        // live corpus as prose. Restricting it to a claim POSITION keeps it a
        // claim about a gate and not a word in a sentence.
        if let Some(topic) = run.strip_prefix("check-")
            && positions.contains(run)
        {
            insert_token(&mut tokens, "check-", topic);
        }
    }
    tokens
}

fn insert_token(tokens: &mut BTreeSet<String>, prefix: &str, topic: &str) {
    // A bare `oya-check-*` / `check-*` family wildcard is not a claim about any
    // one gate; the `*` is not an identifier char, so it arrives here as an
    // empty topic.
    if topic.is_empty() || topic.ends_with('-') || topic.ends_with('_') {
        return;
    }
    tokens.insert(format!("{prefix}{topic}"));
}

fn is_binding_context(lower: &str) -> bool {
    [
        "blocks merge",
        "blocking",
        "branch protection",
        "enforced by",
        "enforced_by",
        "required check",
        "required status",
        "shall",
        "status: active",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_advisory_context(lower: &str) -> bool {
    if [
        "not advisory",
        "not merely advisory",
        "not just advisory",
        "no longer advisory",
        "not planned",
        "not proposed",
        "not candidate",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return false;
    }
    [
        "advisory",
        "backlog",
        "candidate",
        "not active",
        "not enforced",
        "not required",
        "not yet",
        "planned",
        "proposed",
        "retired",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn requires_branch_context(lower: &str) -> bool {
    lower.contains("branch protection")
        || lower.contains("blocks merge")
        || lower.contains("blocking")
        || lower.contains("required check")
        || lower.contains("required status")
}

fn starts_multiline_binding_header(lower: &str) -> bool {
    let stripped = lower.trim_start();
    stripped.starts_with("enforced_by:")
        || stripped.starts_with("\"enforced_by\":")
        || stripped.starts_with("'enforced_by':")
        || stripped.starts_with("required check:")
        || stripped.starts_with("required status:")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known() -> KnownEnforcementSurfaces {
        KnownEnforcementSurfaces {
            check_capabilities: BTreeSet::from(["check-real".to_string()]),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            branch_required_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            // Both declared lanes; "missing" is declared-but-unresolved so the
            // missing-* violation tests still fire, while undeclared tokens are
            // advisory (ADR-0362 (a)).
            declared_lane_ids: BTreeSet::from([
                "oya-governance-real".to_string(),
                "oya-governance-missing".to_string(),
            ]),
        }
    }

    fn doc(contents: &str) -> AspirationalDocument {
        AspirationalDocument {
            path: "docs/decisions/ADR-9999.md".to_string(),
            contents: contents.to_string(),
        }
    }

    #[test]
    fn accepts_binding_claims_with_real_surfaces() {
        let report = validate_aspirational_enforcement(
            [doc(
                "enforced_by: oya-check-real\nrequired check: oya-governance-real\n",
            )],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 2);
    }

    #[test]
    fn accepts_planned_missing_lane_mentions() {
        let report = validate_aspirational_enforcement(
            [doc(
                "candidate validator oya-governance-missing remains planned and advisory\n",
            )],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 0);
    }

    #[test]
    fn treats_undeclared_governance_enforced_by_as_advisory() {
        // ADR-0362 (a): a binding `enforced_by:` claim referencing an
        // oya-governance-* lane that is NOT declared in the registry is a
        // planned/future lane => advisory, not a violation.
        let report = validate_aspirational_enforcement(
            [doc("enforced_by: oya-governance-doc-rigor\n")],
            &known(),
        )
        .expect("undeclared governance lane ref is advisory, not a violation");
        // The token is seen on a binding line (counted) but not flagged, because
        // the lane is undeclared => treated as planned/advisory.
        assert_eq!(report.binding_mentions, 1);
    }

    #[test]
    fn accepts_non_binding_workflow_mentions() {
        let report = validate_aspirational_enforcement(
            [doc(
                "workflow catalog documents oya-governance-missing as future context\n",
            )],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 0);
    }

    #[test]
    fn ignores_wildcard_prefix_mentions() {
        let report = validate_aspirational_enforcement(
            [doc("required check family oya-check-*\n")],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 0);
    }

    #[test]
    fn resolves_the_relocated_check_capability_spelling() {
        // Post-relocation corpora spell the gate by its capability identity
        // (`check-real`), not by the legacy crate name. Both must resolve
        // against the SAME known set, or the relocation empties the scan.
        let report =
            validate_aspirational_enforcement([doc("enforced_by: check-real\n")], &known()).unwrap();
        assert_eq!(report.binding_mentions, 1);
        assert_eq!(report.check_sites, 1);
    }

    #[test]
    fn rejects_missing_capability_under_the_relocated_spelling() {
        let violations =
            validate_aspirational_enforcement([doc("enforced_by: check-missing\n")], &known())
                .unwrap_err();
        assert_eq!(violations[0].kind, AspirationalIssueKind::MissingCrate);
        assert_eq!(violations[0].token, "check-missing");
    }

    #[test]
    fn bare_check_prefix_does_not_substring_collide_inside_oya_check() {
        // The naive fix — adding a bare `check-` substring prefix alongside
        // `oya-check-` — makes `oya-check-real` yield BOTH `oya-check-real`
        // and a spurious nested `check-real`, double-counting every legacy
        // site. Token-boundary anchoring must yield exactly one identity.
        let report = validate_aspirational_enforcement(
            [doc("enforced_by: oya-check-real\n")],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 1);
        assert_eq!(report.check_sites, 1);
    }

    #[test]
    fn both_spellings_normalise_to_one_identity() {
        assert_eq!(
            enforcement_tokens("enforced_by: oya-check-real"),
            BTreeSet::from(["check-real".to_string()])
        );
        assert_eq!(
            enforcement_tokens("enforced_by: check-real"),
            BTreeSet::from(["check-real".to_string()]),
            "legacy and relocated spellings must be the same site, not two"
        );
        assert_eq!(
            enforcement_tokens("enforced_by:\n- check-real".lines().nth(1).unwrap()),
            BTreeSet::from(["check-real".to_string()]),
            "the relocated spelling must resolve in a YAML list value too"
        );
        assert_eq!(
            enforcement_tokens("\"enforced_by\": \"check-real\""),
            BTreeSet::from(["check-real".to_string()]),
            "the relocated spelling must resolve in a JSON value too"
        );
        assert_eq!(
            enforcement_tokens("axum is sanctioned; shall be enforced by `check-real` per ADR-0090"),
            BTreeSet::from(["check-real".to_string()]),
            "a backticked code span is a claim position: today's prose claims \
             read `oya-check-<topic>`, and relocation turns them into the bare form"
        );
    }

    #[test]
    fn identity_match_is_anchored_at_word_boundaries() {
        // A token that merely ENDS with the identity shape is not a site.
        assert!(enforcement_tokens("enforced_by: notacheck-real").is_empty());
        assert!(enforcement_tokens("enforced_by: prefixoya-check-real").is_empty());
    }

    #[test]
    fn bare_check_prefix_in_prose_is_not_a_claim() {
        // Every one of these is a real line from the corpus. The bare
        // `check-<topic>` form is not brand-anchored, so outside the scalar
        // VALUE position it is ordinary English / GitHub vocabulary, not a
        // claim about a gate. Matching it there over-fires — the opposite
        // error to the emptied scan, and just as wrong.
        for prose in [
            "  - oya-governance-x (new lane; refuses changes that conflate fmt-fail with check-fail in the exit code)",
            "  - oya-governance-y (new lane; the canonical prefixes for check-family lanes are exhaustive)",
            "a `workflow_call` reusable workflow renames check-runs to `<caller>/<job>`, breaking the required context",
            "respecting the verified `workflow_call` check-run-rename caveat",
        ] {
            let tokens = enforcement_tokens(prose);
            assert!(
                !tokens.iter().any(|token| token.starts_with("check-")),
                "prose must not read as a check claim: {prose} -> {tokens:?}"
            );
        }
    }

    #[test]
    fn counts_check_sites_on_non_binding_lines_too() {
        // The measured site count is the scan's liveness signal, so it counts
        // every observed identity — not only the ones on binding lines.
        let report = validate_aspirational_enforcement(
            [doc("the catalog documents oya-check-other as advisory\n")],
            &known(),
        )
        .unwrap();
        assert_eq!(report.binding_mentions, 0);
        assert_eq!(report.check_sites, 1);
    }

    #[test]
    fn rejects_missing_check_crates() {
        let violations =
            validate_aspirational_enforcement([doc("enforced_by: oya-check-missing\n")], &known())
                .unwrap_err();
        assert_eq!(violations[0].kind, AspirationalIssueKind::MissingCrate);
    }

    #[test]
    fn rejects_missing_workflow_contexts() {
        let violations = validate_aspirational_enforcement(
            [doc("required check: oya-governance-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingWorkflow)
        );
    }

    #[test]
    fn rejects_missing_quality_lane_contexts() {
        let known = KnownEnforcementSurfaces {
            check_capabilities: BTreeSet::new(),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::new(),
            branch_required_contexts: BTreeSet::new(),
            declared_lane_ids: BTreeSet::from(["oya-governance-real".to_string()]),
        };
        let violations =
            validate_aspirational_enforcement([doc("enforced_by: oya-governance-real\n")], &known)
                .unwrap_err();
        assert_eq!(
            violations[0].kind,
            AspirationalIssueKind::MissingQualityLane
        );
    }

    #[test]
    fn rejects_missing_branch_required_contexts() {
        let known = KnownEnforcementSurfaces {
            check_capabilities: BTreeSet::new(),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            branch_required_contexts: BTreeSet::new(),
            declared_lane_ids: BTreeSet::from(["oya-governance-real".to_string()]),
        };
        let violations = validate_aspirational_enforcement(
            [doc(
                "branch protection required check: oya-governance-real\n",
            )],
            &known,
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingRequiredContext)
        );
    }

    #[test]
    fn rejects_negated_advisory_binding_claims() {
        let violations = validate_aspirational_enforcement(
            [doc(
                "required check: oya-governance-missing is active, not advisory\n",
            )],
            &known(),
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingWorkflow)
        );
    }

    #[test]
    fn rejects_multiline_enforced_by_claims() {
        let violations = validate_aspirational_enforcement(
            [doc("enforced_by:\n  - oya-check-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert_eq!(violations[0].kind, AspirationalIssueKind::MissingCrate);
    }

    #[test]
    fn rejects_same_indent_yaml_enforced_by_claims() {
        let violations = validate_aspirational_enforcement(
            [doc("enforced_by:\n- oya-check-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert_eq!(violations[0].kind, AspirationalIssueKind::MissingCrate);
    }

    #[test]
    fn rejects_same_indent_yaml_required_check_claims() {
        let violations = validate_aspirational_enforcement(
            [doc("required check:\n- oya-governance-missing\n")],
            &known(),
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingWorkflow)
        );
    }

    #[test]
    fn rejects_same_indent_yaml_required_status_claims_without_branch_context() {
        let known = KnownEnforcementSurfaces {
            check_capabilities: BTreeSet::new(),
            workflow_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            quality_lane_contexts: BTreeSet::from(["oya-governance-real".to_string()]),
            branch_required_contexts: BTreeSet::new(),
            declared_lane_ids: BTreeSet::from(["oya-governance-real".to_string()]),
        };
        let violations = validate_aspirational_enforcement(
            [doc("required status:\n- oya-governance-real\n")],
            &known,
        )
        .unwrap_err();
        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == AspirationalIssueKind::MissingRequiredContext)
        );
    }
}
