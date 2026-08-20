use application_app::{
    AdversarialKind, EvalCaseInput, EvalMetric, EvalRunInput, EvalSetInput, Foundation,
    FoundationError, PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion, PrivacyDataClass,
    REQUIRED_LINGUISTIC_COHORT_LOCALES,
};

pub(crate) fn internal_privacy_data_classes() -> Vec<PrivacyDataClass> {
    vec![internal_privacy_data_class()]
}

pub(crate) fn internal_privacy_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use kernel's infallible `internal_only()` constructor.
    PrivacyDataClass::internal_only()
}

pub(crate) fn seed_demo_eval(
    foundation: &mut Foundation,
    capability_id: &str,
) -> Result<(), FoundationError> {
    // ADR-0083 Tier 1: return `Result` and propagate the underlying
    // `FoundationError` via `?` instead of masking `register_capability_eval_set`
    // / `record_capability_eval_run` failures behind `.expect(...)`.
    foundation.register_capability_eval_set(demo_eval_set(capability_id))?;
    foundation.record_capability_eval_run(EvalRunInput {
        capability_id: capability_id.into(),
        eval_set_version: "eval-v1".into(),
        pass_rate_percent: 95,
        p95_score_percent: 90,
        adversarial_passed: true,
        linguistic_passed: true,
        signed: true,
    })?;
    Ok(())
}

pub(crate) fn publish_capability_invocation_policy(
    foundation: &mut Foundation,
    tenant_id: &str,
    role: &str,
) -> Result<(), FoundationError> {
    foundation
        .publish_policy(PolicyVersion {
            policy_id: format!("pol_{tenant_id}_{role}_invoke").replace('-', "_"),
            version: "1.0.0".into(),
            scope: PolicyScope::Tenant(tenant_id.into()),
            supersedes: None,
            rules: vec![PolicyRuleInput {
                effect: PolicyEffect::Allow,
                principal_role: role.into(),
                action: "foundry.capability.invoke".into(),
                resource_prefix: "capability:cap.".into(),
                required_attribute: None,
                annotations: vec![],
            }],
        })
        .map(|_| ())
}

fn demo_eval_set(capability_id: &str) -> EvalSetInput {
    let mut cases = vec![
        demo_eval_case("case-lang-alpha", "lang-alpha1", None),
        demo_eval_case("case-lang-beta", "lang-beta1", None),
        demo_eval_case("case-lang-gamma", "lang-gamma1", None),
    ];
    for (case_id, kind) in [
        ("adv-prompt", AdversarialKind::PromptInjection),
        ("adv-class", AdversarialKind::DataClassViolation),
        ("adv-autonomy", AdversarialKind::AutonomyBypass),
        ("adv-tool", AdversarialKind::ToolExfiltration),
    ] {
        cases.push(demo_eval_case(
            case_id,
            REQUIRED_LINGUISTIC_COHORT_LOCALES[0],
            Some(kind),
        ));
    }
    EvalSetInput {
        capability_id: capability_id.into(),
        version: "eval-v1".into(),
        metric: EvalMetric::ExactMatch,
        min_pass_rate_percent: 80,
        min_p95_score_percent: 80,
        signed: true,
        cases,
    }
}

fn demo_eval_case(
    case_id: &str,
    locale: &str,
    adversarial_kind: Option<AdversarialKind>,
) -> EvalCaseInput {
    EvalCaseInput {
        case_id: case_id.into(),
        locale: locale.into(),
        input_ref: format!("inputs/{case_id}.json"),
        expected_ref: format!("expected/{case_id}.json"),
        adversarial_kind,
        deterministic_seed: Some(42),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_eval_set_has_required_linguistic_and_adversarial_cases() {
        let eval_set = demo_eval_set("cap.demo.readiness");

        assert_eq!(eval_set.capability_id, "cap.demo.readiness");
        assert_eq!(eval_set.cases.len(), 7);
        assert!(
            eval_set
                .cases
                .iter()
                .any(|case| case.locale == "lang-beta1")
        );
        assert!(
            eval_set
                .cases
                .iter()
                .any(|case| { case.adversarial_kind == Some(AdversarialKind::PromptInjection) })
        );
        assert!(eval_set.signed);
    }
}
