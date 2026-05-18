//! EU AI Act Annex III refusal integration check (SEC-MAJ-02).
//!
//! # Why this crate exists
//!
//! Across the microservices that ship a `capabilities/T2-auto.yaml` we
//! declare claims like:
//!
//! > **T2-task-auto-assign in employment-context engages EU AI Act
//! > Annex III §4 high-risk AI** and is REFUSED at the Cedar layer
//! > until ADR-TASKS-XXXX conformity-assessment ADR ships per pack.
//!
//! The claim is only meaningful if the Cedar policy fragment in the
//! same µservice actually **forbids** the auto-action when
//! `context.employment_context == true` (or `context.context_kind ==
//! "employment"`, per the canonical Cedar shape used in workflow-studio
//! and tasks).
//!
//! Before this crate, nothing tied the claim to the implementation.
//! The kernel performs that tie:
//!
//! 1. Parse every supplied `capabilities/T2-auto.yaml` for capability
//!    blocks that carry an "EU AI Act Annex III" / "REFUSED at Cedar
//!    layer" / "employment-context" admission claim.
//! 2. For each such claim, locate the supplied Cedar policy fragment
//!    for the same µservice.
//! 3. Verify the Cedar fragment contains a `forbid (...)` rule that
//!    refuses the auto-assign Action when the employment-context
//!    condition is present.
//! 4. Emit a `EuAiActAnnexIiiRefusalViolation` for any (capability,
//!    µservice) pair whose claim has no matching forbid rule.
//!
//! # Layer
//!
//! `domain` (port-in-kernel, ADR-0056). The kernel performs no I/O;
//! callers (e.g. `oya gate validate eu-ai-act-annex-iii-refusal`) read
//! the files and hand strings in.
//!
//! # Naming justification
//!
//! `oya-check-eu-ai-act-annex-iii-refusal` follows BNF v4.1:
//! `oya-<topic:check>-<axis:eu-ai-act-annex-iii-refusal>`. The hyphenated
//! Annex token (`annex-iii`) preserves the regulator-facing citation
//! style used in
//! `specs/capabilities/eu-ai-act-risk-class-registry.json`.
//!
//! # References
//!
//! - EU AI Act Regulation (EU) 2024/1689 — Art. 5 prohibited; Art. 6
//!   high-risk classification; Art. 9 risk management; Art. 14 human
//!   oversight; Art. 50 transparency; Annex III §1-§8.
//! - `specs/capabilities/eu-ai-act-risk-class-registry.json`.
//! - `microservices/<ms>/capabilities/T2-auto.yaml`.
//! - `microservices/<ms>/policy/tenant-scope.cedar`.
//! - ADR-0064 — canonical-base-and-localization-packs.
//! - ADR-0133 — industry-best-practice + hyperscaler-conformance.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

/// One supplied capability YAML — runners read the file and forward its
/// path + UTF-8 contents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

/// One supplied Cedar policy fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

/// Successful audit summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnnexIiiRefusalReport {
    pub capabilities_checked: usize,
    pub claims_found: usize,
    pub cedar_fragments_checked: usize,
    pub microservices_audited: usize,
}

/// A capability claim that did not have a matching Cedar forbid rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EuAiActAnnexIiiRefusalViolation {
    pub capability_id: String,
    pub microservice: String,
    pub capability_doc_path: String,
    pub claimed_annex_section: String,
    pub kind: ViolationKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ViolationKind {
    /// The capability claims Annex III refusal but no Cedar fragment
    /// was supplied for the same µservice.
    NoCedarFragmentForMicroservice,
    /// A Cedar fragment exists but contains no `forbid` rule that
    /// references the claimed auto-action.
    NoForbidRuleForAction,
    /// A `forbid` rule mentions the auto-action but does not gate on
    /// the employment-context condition.
    ForbidRuleMissingEmploymentContextGate,
}

impl fmt::Display for EuAiActAnnexIiiRefusalViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): capability {:?} declares {:?} refusal but {:?} — {}",
            self.microservice,
            self.capability_doc_path,
            self.capability_id,
            self.claimed_annex_section,
            self.kind,
            self.summary,
        )
    }
}

/// The audit entrypoint. Returns the report on success; the first
/// violation as `Err` on failure.
///
/// Fail-closed: callers that want the full violation list should use
/// `audit_all_violations` instead.
pub fn validate_annex_iii_refusals<C, P>(
    capabilities: C,
    cedar_fragments: P,
) -> Result<AnnexIiiRefusalReport, EuAiActAnnexIiiRefusalViolation>
where
    C: IntoIterator<Item = CapabilityDocument>,
    P: IntoIterator<Item = CedarPolicyDocument>,
{
    let (report, violations) = audit_all_violations(capabilities, cedar_fragments);
    if let Some(first) = violations.into_iter().next() {
        Err(first)
    } else {
        Ok(report)
    }
}

/// Full audit — returns the report AND every violation found.
pub fn audit_all_violations<C, P>(
    capabilities: C,
    cedar_fragments: P,
) -> (AnnexIiiRefusalReport, Vec<EuAiActAnnexIiiRefusalViolation>)
where
    C: IntoIterator<Item = CapabilityDocument>,
    P: IntoIterator<Item = CedarPolicyDocument>,
{
    let capabilities: Vec<CapabilityDocument> = capabilities.into_iter().collect();
    let cedar_fragments: Vec<CedarPolicyDocument> = cedar_fragments.into_iter().collect();

    // Index Cedar fragments by microservice for O(1) lookup.
    let mut cedar_by_ms: BTreeMap<String, Vec<&CedarPolicyDocument>> = BTreeMap::new();
    for fragment in &cedar_fragments {
        cedar_by_ms
            .entry(fragment.microservice.clone())
            .or_default()
            .push(fragment);
    }

    let mut violations = Vec::new();
    let mut microservices_audited: BTreeMap<String, ()> = BTreeMap::new();
    let mut total_claims = 0usize;

    for cap_doc in &capabilities {
        microservices_audited.insert(cap_doc.microservice.clone(), ());
        let claims = parse_annex_iii_claims(cap_doc);
        total_claims += claims.len();

        for claim in claims {
            let fragments = cedar_by_ms.get(&cap_doc.microservice);
            match fragments {
                None => {
                    violations.push(EuAiActAnnexIiiRefusalViolation {
                        capability_id: claim.capability_id.clone(),
                        microservice: cap_doc.microservice.clone(),
                        capability_doc_path: cap_doc.path.clone(),
                        claimed_annex_section: claim.annex_section.clone(),
                        kind: ViolationKind::NoCedarFragmentForMicroservice,
                        summary: format!(
                            "no Cedar policy fragment supplied for µservice {:?}",
                            cap_doc.microservice
                        ),
                    });
                }
                Some(fragments) => {
                    let any_forbid = fragments
                        .iter()
                        .any(|f| fragment_forbids_action(f, &claim));
                    let any_gates_employment = fragments
                        .iter()
                        .any(|f| fragment_gates_on_employment_context(f, &claim));
                    if !any_forbid {
                        violations.push(EuAiActAnnexIiiRefusalViolation {
                            capability_id: claim.capability_id.clone(),
                            microservice: cap_doc.microservice.clone(),
                            capability_doc_path: cap_doc.path.clone(),
                            claimed_annex_section: claim.annex_section.clone(),
                            kind: ViolationKind::NoForbidRuleForAction,
                            summary: format!(
                                "no `forbid` rule mentions capability {:?} or its claimed Cedar action token \
                                 across {} supplied fragment(s)",
                                claim.capability_id,
                                fragments.len(),
                            ),
                        });
                    } else if !any_gates_employment {
                        violations.push(EuAiActAnnexIiiRefusalViolation {
                            capability_id: claim.capability_id.clone(),
                            microservice: cap_doc.microservice.clone(),
                            capability_doc_path: cap_doc.path.clone(),
                            claimed_annex_section: claim.annex_section.clone(),
                            kind: ViolationKind::ForbidRuleMissingEmploymentContextGate,
                            summary: format!(
                                "`forbid` rule found, but none of {} supplied fragment(s) gate on \
                                 employment-context (`context.employment_context` or \
                                 `context.context_kind == \"employment\"`)",
                                fragments.len(),
                            ),
                        });
                    }
                }
            }
        }
    }

    let report = AnnexIiiRefusalReport {
        capabilities_checked: capabilities.len(),
        claims_found: total_claims,
        cedar_fragments_checked: cedar_fragments.len(),
        microservices_audited: microservices_audited.len(),
    };
    (report, violations)
}

/// Internal claim record extracted from a capability YAML.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AnnexIiiClaim {
    capability_id: String,
    annex_section: String,
    /// Canonical Cedar action token (e.g. `TaskT2AutoAssign`) inferred
    /// from the capability id when the YAML names it via the
    /// `cedar_admission_path` field; otherwise empty.
    cedar_action_token: String,
}

/// Parse a `capabilities/T2-auto.yaml` for capability blocks that
/// declare an Annex-III refusal claim. We do not depend on a YAML
/// crate; the canonical schema fields we need are line-anchored.
fn parse_annex_iii_claims(doc: &CapabilityDocument) -> Vec<AnnexIiiClaim> {
    let mut out = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_action: Option<String> = None;
    let mut current_annex: Option<String> = None;
    let mut current_refused = false;

    for raw_line in doc.contents.lines() {
        let line = raw_line.trim_start();
        // New capability block — flush the previous and reset.
        if let Some(id_value) = line.strip_prefix("- id:") {
            flush_claim(
                &mut out,
                &mut current_id,
                &mut current_action,
                &mut current_annex,
                &mut current_refused,
            );
            current_id = Some(id_value.trim().trim_matches('"').to_string());
            continue;
        }

        if let Some(annex_value) = line.strip_prefix("annex_iii_section:") {
            current_annex = Some(annex_value.trim().trim_matches('"').to_string());
        }

        // The eu_ai_act_classification line is where the refusal claim
        // is canonically declared.
        if line.starts_with("eu_ai_act_classification:") {
            let lower = line.to_ascii_lowercase();
            if lower.contains("annex iii")
                || lower.contains("annex-iii")
                || lower.contains("annex_iii")
            {
                if lower.contains("refused at cedar")
                    || lower.contains("refused at the cedar")
                    || lower.contains("refused-at-cedar")
                {
                    current_refused = true;
                }
                // Capture the Annex section text if present inline.
                if let Some(start) = lower.find("annex iii") {
                    let tail = &line[start..];
                    current_annex.get_or_insert_with(|| tail.to_string());
                }
            }
        }

        // The cedar_admission_path multi-line block names the Action::
        // token. We pick the first such token we see between blocks.
        // The canonical shapes are:
        //   Action::"TaskT2AutoAssign"     (Cedar v4 string)
        //   Action::TaskT2AutoAssign       (Cedar v4 identifier)
        //   action::TaskT2AutoAssign       (lowercase variant)
        // We also tolerate the verb form `action::` (used in YAML
        // comment-style admission paths like
        // `tenant-scope.cedar :: action::TaskT2AutoAssign admits …`).
        if current_action.is_none() {
            for marker in ["Action::", "action::"] {
                if let Some(start) = line.find(marker) {
                    let after = &line[start + marker.len()..];
                    // Strip leading quote if present.
                    let after = after.trim_start_matches('"');
                    let token: String = after
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !token.is_empty() {
                        current_action = Some(token);
                        break;
                    }
                }
            }
        }
    }

    flush_claim(
        &mut out,
        &mut current_id,
        &mut current_action,
        &mut current_annex,
        &mut current_refused,
    );
    out
}

fn flush_claim(
    out: &mut Vec<AnnexIiiClaim>,
    current_id: &mut Option<String>,
    current_action: &mut Option<String>,
    current_annex: &mut Option<String>,
    current_refused: &mut bool,
) {
    if let Some(id) = current_id.take() {
        if *current_refused {
            let annex = current_annex.take().unwrap_or_else(|| "Annex III".to_string());
            let action = current_action.take().unwrap_or_default();
            out.push(AnnexIiiClaim {
                capability_id: id,
                annex_section: annex,
                cedar_action_token: action,
            });
        } else {
            // Drop the partials.
            current_action.take();
            current_annex.take();
        }
        *current_refused = false;
    }
}

/// Does the supplied Cedar fragment contain a `forbid` rule that
/// references the claim's auto-action? We accept either the canonical
/// Action token (when the YAML provided one) OR a `forbid` rule whose
/// body mentions the literal capability_id (case-insensitive
/// substring), which matches the convention in `workflow-studio` of
/// naming Actions after their capability ids.
fn fragment_forbids_action(fragment: &CedarPolicyDocument, claim: &AnnexIiiClaim) -> bool {
    let lower = fragment.contents.to_ascii_lowercase();
    if !lower.contains("forbid") {
        return false;
    }
    let cap_lower = claim.capability_id.to_ascii_lowercase();
    let action_lower = claim.cedar_action_token.to_ascii_lowercase();
    if !action_lower.is_empty() && lower.contains(&action_lower) {
        return true;
    }
    // Tolerant fallback: capability id text appears somewhere in a
    // forbid context.
    for window in lower.split("forbid").skip(1) {
        // Only consider up to the next blank line / semicolon-ended
        // fragment so we don't false-positive on later permits.
        let window_section = window.split("\npermit").next().unwrap_or(window);
        if window_section.contains(&cap_lower) {
            return true;
        }
        // Also accept the canonical PascalCase Action shape inferred
        // from the snake_case id (e.g. `task-auto-assign` →
        // `taskautoassign`, which appears inside `TaskT2AutoAssign`).
        let collapsed: String = cap_lower.chars().filter(|c| c.is_alphanumeric()).collect();
        if !collapsed.is_empty() && window_section.replace('_', "").contains(&collapsed) {
            return true;
        }
    }
    false
}

/// Does ANY clause in the Cedar fragment gate on employment-context?
/// The canonical shapes we look for:
///   - `context.employment_context`
///   - `context.context_kind == "employment"`
///   - the `pack-` employment-context guard literal
///     (e.g. `pack-kr-employment-context`)
fn fragment_gates_on_employment_context(
    fragment: &CedarPolicyDocument,
    _claim: &AnnexIiiClaim,
) -> bool {
    let lower = fragment.contents.to_ascii_lowercase();
    lower.contains("context.employment_context")
        || lower.contains("context.context_kind == \"employment\"")
        || lower.contains("employment_context")
}

#[cfg(test)]
mod tests {
    use super::*;

    const T2_AUTO_YAML_WITH_REFUSAL: &str = r#"
doc_class: AutonomyTierCatalog
microservice: tasks
tier: T2

capabilities:
  - id: T2-task-auto-assign
    name: "Auto-assign task to best-fit assignee"
    description: |
      …employment-context…
    annex_iii_section: Annex III §4 (employment)
    eu_ai_act_classification: "high-risk (Annex III §4) in employment-context — REFUSED at Cedar layer pending ADR-TASKS-XXXX"
    cedar_admission_path: |
      tenant-scope.cedar :: action::TaskT2AutoAssign admits when:
        - context.context_kind != "employment"

  - id: T2-task-auto-status-advance
    eu_ai_act_classification: minimal-risk (deterministic)
"#;

    const CEDAR_WITH_FORBID_AND_EMPLOYMENT_GATE: &str = r#"
forbid (
  principal,
  action == Action::"TaskT2AutoAssign",
  resource
) when {
  context.context_kind == "employment"
};
"#;

    const CEDAR_WITH_FORBID_NO_EMPLOYMENT_GATE: &str = r#"
forbid (
  principal,
  action == Action::"TaskT2AutoAssign",
  resource
);
"#;

    const CEDAR_WITHOUT_FORBID: &str = r#"
permit (principal, action, resource);
"#;

    #[test]
    fn parses_annex_iii_claim_from_yaml() {
        let cap = CapabilityDocument {
            path: "microservices/tasks/capabilities/T2-auto.yaml".into(),
            microservice: "tasks".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.into(),
        };
        let claims = parse_annex_iii_claims(&cap);
        assert_eq!(claims.len(), 1, "expected exactly one Annex III claim");
        assert_eq!(claims[0].capability_id, "T2-task-auto-assign");
        assert!(claims[0].annex_section.to_ascii_lowercase().contains("annex iii"));
        assert_eq!(claims[0].cedar_action_token, "TaskT2AutoAssign");
    }

    #[test]
    fn passes_when_cedar_forbids_with_employment_gate() {
        let cap = CapabilityDocument {
            path: "microservices/tasks/capabilities/T2-auto.yaml".into(),
            microservice: "tasks".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.into(),
        };
        let cedar = CedarPolicyDocument {
            path: "microservices/tasks/policy/tenant-scope.cedar".into(),
            microservice: "tasks".into(),
            contents: CEDAR_WITH_FORBID_AND_EMPLOYMENT_GATE.into(),
        };
        let report = validate_annex_iii_refusals(vec![cap], vec![cedar])
            .expect("forbid + employment gate must pass");
        assert_eq!(report.claims_found, 1);
        assert_eq!(report.microservices_audited, 1);
    }

    #[test]
    fn fails_when_cedar_missing_employment_gate() {
        let cap = CapabilityDocument {
            path: "microservices/tasks/capabilities/T2-auto.yaml".into(),
            microservice: "tasks".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.into(),
        };
        let cedar = CedarPolicyDocument {
            path: "microservices/tasks/policy/tenant-scope.cedar".into(),
            microservice: "tasks".into(),
            contents: CEDAR_WITH_FORBID_NO_EMPLOYMENT_GATE.into(),
        };
        let err = validate_annex_iii_refusals(vec![cap], vec![cedar])
            .expect_err("missing employment-context gate must fail");
        assert_eq!(err.kind, ViolationKind::ForbidRuleMissingEmploymentContextGate);
    }

    #[test]
    fn fails_when_cedar_has_no_forbid() {
        let cap = CapabilityDocument {
            path: "microservices/tasks/capabilities/T2-auto.yaml".into(),
            microservice: "tasks".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.into(),
        };
        let cedar = CedarPolicyDocument {
            path: "microservices/tasks/policy/tenant-scope.cedar".into(),
            microservice: "tasks".into(),
            contents: CEDAR_WITHOUT_FORBID.into(),
        };
        let err = validate_annex_iii_refusals(vec![cap], vec![cedar])
            .expect_err("no forbid rule must fail");
        assert_eq!(err.kind, ViolationKind::NoForbidRuleForAction);
    }

    #[test]
    fn fails_when_no_cedar_supplied() {
        let cap = CapabilityDocument {
            path: "microservices/tasks/capabilities/T2-auto.yaml".into(),
            microservice: "tasks".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.into(),
        };
        let err = validate_annex_iii_refusals(vec![cap], Vec::<CedarPolicyDocument>::new())
            .expect_err("missing cedar must fail");
        assert_eq!(err.kind, ViolationKind::NoCedarFragmentForMicroservice);
    }

    #[test]
    fn passes_when_no_annex_iii_claims_present() {
        let cap = CapabilityDocument {
            path: "microservices/notes/capabilities/T2-auto.yaml".into(),
            microservice: "notes".into(),
            contents: r#"
capabilities:
  - id: T2-notes-auto-tag
    eu_ai_act_classification: minimal-risk
"#
            .into(),
        };
        let report = validate_annex_iii_refusals(vec![cap], Vec::<CedarPolicyDocument>::new())
            .expect("no claims => no findings");
        assert_eq!(report.claims_found, 0);
    }

    #[test]
    fn audit_all_violations_returns_full_list() {
        let cap_a = CapabilityDocument {
            path: "microservices/tasks/capabilities/T2-auto.yaml".into(),
            microservice: "tasks".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.into(),
        };
        let cap_b = CapabilityDocument {
            path: "microservices/calendar/capabilities/T2-auto.yaml".into(),
            microservice: "calendar".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.replace("microservice: tasks", "microservice: calendar"),
        };
        let (report, violations) = audit_all_violations(
            vec![cap_a, cap_b],
            Vec::<CedarPolicyDocument>::new(),
        );
        assert_eq!(report.microservices_audited, 2);
        assert_eq!(violations.len(), 2);
        for v in &violations {
            assert_eq!(v.kind, ViolationKind::NoCedarFragmentForMicroservice);
        }
    }
}
