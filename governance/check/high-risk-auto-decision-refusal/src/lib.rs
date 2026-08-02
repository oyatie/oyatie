//! High-risk auto-decision refusal integration check (SEC-MAJ-02).
//!
//! # Why this crate exists
//!
//! Across microservices that ship `capabilities/T2-auto.yaml`, product teams
//! can declare that a capability becomes a high-risk automated decision in a
//! sensitive context and is therefore refused at the Cedar layer until the
//! tenant has the required conformity evidence for its active regulatory pack.
//!
//! The claim is only meaningful if the Cedar policy fragment in the same
//! microservice actually **forbids** the auto-action when
//! `context.employment_context == true` (or `context.context_kind ==
//! "employment"`, per the canonical Cedar shape used in workflow-studio and
//! tasks).
//!
//! The kernel performs that tie:
//!
//! 1. Parse every supplied `capabilities/T2-auto.yaml` for capability blocks
//!    that carry a high-risk / refused-at-Cedar / employment-context admission
//!    claim.
//! 2. For each such claim, locate the supplied Cedar policy fragment for the
//!    same microservice.
//! 3. Verify the Cedar fragment contains a `forbid (...)` rule that refuses the
//!    auto-action when the employment-context condition is present.
//! 4. Emit a `HighRiskAutoDecisionRefusalViolation` for any (capability,
//!    microservice) pair whose claim has no matching forbid rule.
//!
//! # Layer
//!
//! `domain` (port-in-kernel, ADR-0056). The kernel performs no I/O; callers
//! (for example, `oya gate validate high-risk-auto-decision-refusal`) read the
//! files and hand strings in.
//!
//! # Naming justification
//!
//! `check-high-risk-auto-decision-refusal` follows the ADR-0532/0533 de-branded grammar:
//! `<group:check>-<axis:high-risk-auto-decision-refusal>`. The axis is a
//! product-level duty rather than a jurisdiction identifier, keeping the
//! canonical gate reusable by any regulatory pack that has a high-risk
//! automated-decision refusal rule.
//!
//! # References
//!
//! - `microservices/<ms>/capabilities/T2-auto.yaml`.
//! - `microservices/<ms>/policy/tenant-scope.cedar`.
//! - ADR-0064 — canonical-base-and-localization-packs.
//! - ADR-0133 — industry-best-practice + hyperscaler-conformance.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

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
pub struct HighRiskAutoDecisionRefusalReport {
    pub capabilities_checked: usize,
    pub claims_found: usize,
    pub cedar_fragments_checked: usize,
    pub microservices_audited: usize,
}

/// A capability claim that did not have a matching Cedar forbid rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HighRiskAutoDecisionRefusalViolation {
    pub capability_id: String,
    pub microservice: String,
    pub capability_doc_path: String,
    pub claimed_risk_anchor: String,
    pub kind: ViolationKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ViolationKind {
    /// The capability claims a high-risk refusal but no Cedar fragment
    /// was supplied for the same microservice.
    NoCedarFragmentForMicroservice,
    /// A Cedar fragment exists but contains no `forbid` rule that
    /// references the claimed auto-action.
    NoForbidRuleForAction,
    /// A `forbid` rule mentions the auto-action but does not gate on
    /// the employment-context condition.
    ForbidRuleMissingEmploymentContextGate,
}

impl fmt::Display for HighRiskAutoDecisionRefusalViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): capability {:?} declares {:?} refusal but {:?} — {}",
            self.microservice,
            self.capability_doc_path,
            self.capability_id,
            self.claimed_risk_anchor,
            self.kind,
            self.summary,
        )
    }
}

/// The audit entrypoint. Returns the report on success; the first violation as
/// `Err` on failure.
///
/// Fail-closed: callers that want the full violation list should use
/// `audit_all_violations` instead.
pub fn validate_high_risk_auto_decision_refusals<C, P>(
    capabilities: C,
    cedar_fragments: P,
) -> Result<HighRiskAutoDecisionRefusalReport, HighRiskAutoDecisionRefusalViolation>
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
) -> (
    HighRiskAutoDecisionRefusalReport,
    Vec<HighRiskAutoDecisionRefusalViolation>,
)
where
    C: IntoIterator<Item = CapabilityDocument>,
    P: IntoIterator<Item = CedarPolicyDocument>,
{
    let capabilities: Vec<CapabilityDocument> = capabilities.into_iter().collect();
    let cedar_fragments: Vec<CedarPolicyDocument> = cedar_fragments.into_iter().collect();

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
        let claims = parse_high_risk_refusal_claims(cap_doc);
        total_claims += claims.len();

        for claim in claims {
            let fragments = cedar_by_ms.get(&cap_doc.microservice);
            match fragments {
                None => {
                    violations.push(HighRiskAutoDecisionRefusalViolation {
                        capability_id: claim.capability_id.clone(),
                        microservice: cap_doc.microservice.clone(),
                        capability_doc_path: cap_doc.path.clone(),
                        claimed_risk_anchor: claim.risk_anchor_section.clone(),
                        kind: ViolationKind::NoCedarFragmentForMicroservice,
                        summary: format!(
                            "no Cedar policy fragment supplied for microservice {:?}",
                            cap_doc.microservice
                        ),
                    });
                }
                Some(fragments) => {
                    let any_forbid = fragments.iter().any(|f| fragment_forbids_action(f, &claim));
                    let any_gates_employment = fragments
                        .iter()
                        .any(|f| fragment_gates_on_employment_context(f, &claim));
                    if !any_forbid {
                        violations.push(HighRiskAutoDecisionRefusalViolation {
                            capability_id: claim.capability_id.clone(),
                            microservice: cap_doc.microservice.clone(),
                            capability_doc_path: cap_doc.path.clone(),
                            claimed_risk_anchor: claim.risk_anchor_section.clone(),
                            kind: ViolationKind::NoForbidRuleForAction,
                            summary: format!(
                                "no `forbid` rule mentions capability {:?} or its claimed Cedar action token across {} supplied fragment(s)",
                                claim.capability_id,
                                fragments.len(),
                            ),
                        });
                    } else if !any_gates_employment {
                        violations.push(HighRiskAutoDecisionRefusalViolation {
                            capability_id: claim.capability_id.clone(),
                            microservice: cap_doc.microservice.clone(),
                            capability_doc_path: cap_doc.path.clone(),
                            claimed_risk_anchor: claim.risk_anchor_section.clone(),
                            kind: ViolationKind::ForbidRuleMissingEmploymentContextGate,
                            summary: format!(
                                "`forbid` rule found, but none of {} supplied fragment(s) gate on employment-context (`context.employment_context` or `context.context_kind == \"employment\"`)",
                                fragments.len(),
                            ),
                        });
                    }
                }
            }
        }
    }

    let report = HighRiskAutoDecisionRefusalReport {
        capabilities_checked: capabilities.len(),
        claims_found: total_claims,
        cedar_fragments_checked: cedar_fragments.len(),
        microservices_audited: microservices_audited.len(),
    };
    (report, violations)
}

/// Internal claim record extracted from a capability YAML.
#[derive(Clone, Debug, Eq, PartialEq)]
struct HighRiskRefusalClaim {
    capability_id: String,
    risk_anchor_section: String,
    /// Canonical Cedar action token (e.g. `TaskT2AutoAssign`) inferred from the
    /// capability id when the YAML names it via the `cedar_admission_path`
    /// field; otherwise empty.
    cedar_action_token: String,
}

/// Parse a `capabilities/T2-auto.yaml` for capability blocks that declare a
/// high-risk refused-at-Cedar claim. We do not depend on a YAML crate; the
/// canonical schema fields we need are line-anchored, with tolerant support for
/// multi-line `risk_classification` values.
fn parse_high_risk_refusal_claims(doc: &CapabilityDocument) -> Vec<HighRiskRefusalClaim> {
    let mut out = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_action: Option<String> = None;
    let mut current_risk_anchor: Option<String> = None;
    let mut current_refused = false;

    for raw_line in doc.contents.lines() {
        let line = raw_line.trim_start();
        if let Some(id_value) = line.strip_prefix("- id:") {
            flush_claim(
                &mut out,
                &mut current_id,
                &mut current_action,
                &mut current_risk_anchor,
                &mut current_refused,
            );
            current_id = Some(id_value.trim().trim_matches('"').to_string());
            continue;
        }

        if let Some(anchor_value) = line.strip_prefix("risk_anchor_section:") {
            current_risk_anchor = Some(anchor_value.trim().trim_matches('"').to_string());
        }

        if line_declares_high_risk_refusal(line) {
            current_refused = true;
            current_risk_anchor
                .get_or_insert_with(|| risk_anchor_from_line(line).unwrap_or_default());
        }

        if current_action.is_none() {
            for marker in ["Action::", "action::"] {
                if let Some(start) = line.find(marker) {
                    let after = &line[start + marker.len()..];
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
        &mut current_risk_anchor,
        &mut current_refused,
    );
    out
}

fn line_declares_high_risk_refusal(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let high_risk =
        lower.contains("high-risk") || lower.contains("high_risk") || lower.contains("high risk");
    let refused = lower.contains("refused at cedar")
        || lower.contains("refused at the cedar")
        || lower.contains("refused-at-cedar");
    high_risk && refused
}

fn risk_anchor_from_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.trim_matches('"').to_string())
}

fn flush_claim(
    out: &mut Vec<HighRiskRefusalClaim>,
    current_id: &mut Option<String>,
    current_action: &mut Option<String>,
    current_risk_anchor: &mut Option<String>,
    current_refused: &mut bool,
) {
    if let Some(id) = current_id.take() {
        if *current_refused {
            let risk_anchor = current_risk_anchor
                .take()
                .filter(|anchor| !anchor.trim().is_empty())
                .unwrap_or_else(|| "high-risk auto-decision".to_string());
            let action = current_action.take().unwrap_or_default();
            out.push(HighRiskRefusalClaim {
                capability_id: id,
                risk_anchor_section: risk_anchor,
                cedar_action_token: action,
            });
        } else {
            current_action.take();
            current_risk_anchor.take();
        }
        *current_refused = false;
    }
}

/// Does the supplied Cedar fragment contain a `forbid` rule that references the
/// claim's auto-action? We accept either the canonical Action token (when the
/// YAML provided one) OR a `forbid` rule whose body mentions the literal
/// capability_id (case-insensitive substring).
fn fragment_forbids_action(fragment: &CedarPolicyDocument, claim: &HighRiskRefusalClaim) -> bool {
    let lower = fragment.contents.to_ascii_lowercase();
    if !lower.contains("forbid") {
        return false;
    }
    let cap_lower = claim.capability_id.to_ascii_lowercase();
    let action_lower = claim.cedar_action_token.to_ascii_lowercase();
    if !action_lower.is_empty() && lower.contains(&action_lower) {
        return true;
    }
    let normalized_action: String = action_lower
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect();
    if !normalized_action.is_empty()
        && lower
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .contains(&normalized_action)
    {
        return true;
    }
    for window in lower.split("forbid").skip(1) {
        let window_section = window.split("\npermit").next().unwrap_or(window);
        if window_section.contains(&cap_lower) {
            return true;
        }
        let collapsed: String = cap_lower.chars().filter(|c| c.is_alphanumeric()).collect();
        if !collapsed.is_empty() && window_section.replace('_', "").contains(&collapsed) {
            return true;
        }
    }
    false
}

/// Does any clause in the Cedar fragment gate on employment-context?
fn fragment_gates_on_employment_context(
    fragment: &CedarPolicyDocument,
    _claim: &HighRiskRefusalClaim,
) -> bool {
    let lower = fragment.contents.to_ascii_lowercase();
    lower.contains("context.employment_context")
        || lower.contains("context.context_kind == \"employment\"")
        || lower.contains("resource.context_kind == \"employment\"")
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
    risk_anchor_section: employment-context high-risk auto-decision
    risk_classification: "high-risk in employment-context — REFUSED at Cedar layer pending conformity evidence"
    cedar_admission_path: |
      tenant-scope.cedar :: action::TaskT2AutoAssign admits when:
        - context.context_kind != "employment"

  - id: T2-task-auto-status-advance
    risk_classification: minimal-risk (deterministic)
"#;

    const T2_AUTO_YAML_WITH_MULTILINE_REFUSAL: &str = r#"
capabilities:
  - id: T2-mail-dlp-quarantine
    risk_anchor_section: employment-context high-risk auto-decision
    risk_classification:
      - default: limited-risk
      - high-risk: when employment context is detected — REFUSED at Cedar layer pending conformity evidence
    cedar_admission_path: |
      tenant-scope.cedar :: action::MailDlpQuarantine admits when:
        - context.context_kind != "employment"
"#;

    const CEDAR_WITH_FORBID_AND_EMPLOYMENT_GATE: &str = r#"
forbid (
  principal,
  action == Action::"task_t2_auto_assign",
  resource
) when {
  resource.context_kind == "employment"
};
"#;

    const CEDAR_WITH_MULTILINE_FORBID: &str = r#"
forbid (
  principal,
  action == Action::"MailDlpQuarantine",
  resource
) when {
  context.employment_context == true
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
    fn parses_high_risk_refusal_claim_from_yaml() {
        let cap = CapabilityDocument {
            path: "microservices/tasks/capabilities/T2-auto.yaml".into(),
            microservice: "tasks".into(),
            contents: T2_AUTO_YAML_WITH_REFUSAL.into(),
        };
        let claims = parse_high_risk_refusal_claims(&cap);
        assert_eq!(
            claims.len(),
            1,
            "expected exactly one high-risk refusal claim"
        );
        assert_eq!(claims[0].capability_id, "T2-task-auto-assign");
        assert_eq!(
            claims[0].risk_anchor_section,
            "employment-context high-risk auto-decision"
        );
        assert_eq!(claims[0].cedar_action_token, "TaskT2AutoAssign");
    }

    #[test]
    fn parses_multiline_high_risk_refusal_claim_from_yaml() {
        let cap = CapabilityDocument {
            path: "microservices/mail/capabilities/T2-auto.yaml".into(),
            microservice: "mail".into(),
            contents: T2_AUTO_YAML_WITH_MULTILINE_REFUSAL.into(),
        };
        let cedar = CedarPolicyDocument {
            path: "microservices/mail/policy/tenant-scope.cedar".into(),
            microservice: "mail".into(),
            contents: CEDAR_WITH_MULTILINE_FORBID.into(),
        };
        let report = validate_high_risk_auto_decision_refusals(vec![cap], vec![cedar])
            .expect("multi-line refusal + employment gate must pass");
        assert_eq!(report.claims_found, 1);
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
        let report = validate_high_risk_auto_decision_refusals(vec![cap], vec![cedar])
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
        let err = validate_high_risk_auto_decision_refusals(vec![cap], vec![cedar])
            .expect_err("missing employment-context gate must fail");
        assert_eq!(
            err.kind,
            ViolationKind::ForbidRuleMissingEmploymentContextGate
        );
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
        let err = validate_high_risk_auto_decision_refusals(vec![cap], vec![cedar])
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
        let err =
            validate_high_risk_auto_decision_refusals(vec![cap], Vec::<CedarPolicyDocument>::new())
                .expect_err("missing cedar must fail");
        assert_eq!(err.kind, ViolationKind::NoCedarFragmentForMicroservice);
    }

    #[test]
    fn passes_when_no_high_risk_refusal_claims_present() {
        let cap = CapabilityDocument {
            path: "microservices/notes/capabilities/T2-auto.yaml".into(),
            microservice: "notes".into(),
            contents: r#"
capabilities:
  - id: T2-notes-auto-tag
    risk_classification: minimal-risk
"#
            .into(),
        };
        let report =
            validate_high_risk_auto_decision_refusals(vec![cap], Vec::<CedarPolicyDocument>::new())
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
            contents: T2_AUTO_YAML_WITH_REFUSAL
                .replace("microservice: tasks", "microservice: calendar"),
        };
        let (report, violations) =
            audit_all_violations(vec![cap_a, cap_b], Vec::<CedarPolicyDocument>::new());
        assert_eq!(report.microservices_audited, 2);
        assert_eq!(violations.len(), 2);
        for v in &violations {
            assert_eq!(v.kind, ViolationKind::NoCedarFragmentForMicroservice);
        }
    }
}
