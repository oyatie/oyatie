//! Pure guardrail decision kernel for Intelligence requests.
//!
//! The kernel is intentionally fail-closed: missing classifier output, high-risk
//! findings, and always-blocked safety categories produce deny decisions with
//! deterministic evidence references.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GuardrailCategory {
    Benign,
    ChildSafety,
    CredentialLeakage,
    HateHarassment,
    PromptInjection,
    RegulatedFinancial,
    RegulatedMedical,
    SelfHarm,
    SexualContent,
    Violence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailFinding {
    pub category: GuardrailCategory, // data_class: INTERNAL_ONLY
    pub risk_level: RiskLevel,       // data_class: INTERNAL_ONLY
    pub reason: String,              // data_class: INTERNAL_ONLY
    pub evidence_ref: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailRequest {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub content_ref: String,             // data_class: INTERNAL_ONLY
    pub findings: Vec<GuardrailFinding>, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailAllow {
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailDeny {
    pub refusal_reasons: Vec<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuardrailDecision {
    Allow(GuardrailAllow),
    Deny(GuardrailDeny),
}

pub fn decide_guardrail(request: &GuardrailRequest) -> GuardrailDecision {
    let mut evidence_refs = vec![request.request_evidence_ref.clone()];
    let mut refusal_reasons = Vec::new();

    if request.findings.is_empty() {
        refusal_reasons.push("guardrail classification missing; request denied closed".to_owned());
    }

    for finding in &request.findings {
        evidence_refs.push(finding.evidence_ref.clone());
        if finding_requires_refusal(finding) {
            refusal_reasons.push(refusal_reason_for(finding));
        }
    }

    let evidence_refs = sorted_unique(evidence_refs);
    let refusal_reasons = sorted_unique(refusal_reasons);

    if refusal_reasons.is_empty() {
        GuardrailDecision::Allow(GuardrailAllow { evidence_refs })
    } else {
        GuardrailDecision::Deny(GuardrailDeny {
            refusal_reasons,
            evidence_refs,
        })
    }
}

fn finding_requires_refusal(finding: &GuardrailFinding) -> bool {
    finding.risk_level == RiskLevel::High || is_always_blocked_category(finding.category)
}

fn is_always_blocked_category(category: GuardrailCategory) -> bool {
    matches!(
        category,
        GuardrailCategory::ChildSafety
            | GuardrailCategory::CredentialLeakage
            | GuardrailCategory::PromptInjection
    )
}

fn refusal_reason_for(finding: &GuardrailFinding) -> String {
    let reason = finding.reason.trim();
    if reason.is_empty() {
        format!(
            "guardrail finding {:?}/{:?} requires refusal",
            finding.category, finding.risk_level
        )
    } else {
        reason.to_owned()
    }
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with_finding(finding: GuardrailFinding) -> GuardrailRequest {
        GuardrailRequest {
            tenant_id: "ten_a".to_owned(),
            content_ref: "content:2".to_owned(),
            findings: vec![finding],
            request_evidence_ref: "req:2".to_owned(),
        }
    }

    #[test]
    fn denies_high_risk_and_preserves_refusal_reason() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::RegulatedMedical,
            risk_level: RiskLevel::High,
            reason: "medical diagnosis request requires licensed escalation".to_owned(),
            evidence_ref: "classifier:2".to_owned(),
        });

        assert_eq!(
            decide_guardrail(&request),
            GuardrailDecision::Deny(GuardrailDeny {
                refusal_reasons: vec![
                    "medical diagnosis request requires licensed escalation".to_owned()
                ],
                evidence_refs: vec!["classifier:2".to_owned(), "req:2".to_owned()],
            })
        );
    }

    #[test]
    fn blank_high_risk_reason_still_denies_with_fallback_reason() {
        let request = request_with_finding(GuardrailFinding {
            category: GuardrailCategory::PromptInjection,
            risk_level: RiskLevel::High,
            reason: " ".to_owned(),
            evidence_ref: "classifier:3".to_owned(),
        });

        assert_eq!(
            decide_guardrail(&request),
            GuardrailDecision::Deny(GuardrailDeny {
                refusal_reasons: vec![
                    "guardrail finding PromptInjection/High requires refusal".to_owned()
                ],
                evidence_refs: vec!["classifier:3".to_owned(), "req:2".to_owned()],
            })
        );
    }
}
