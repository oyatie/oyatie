// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use check_eval_domain::{
    AdversarialKind, EvalCaseInput, EvalError, EvalGate, EvalMetric, EvalRunInput, EvalSetInput,
    REQUIRED_LINGUISTIC_COHORT_LOCALES,
};

#[test]
fn eval_gate_allows_publish_only_after_signed_set_and_passing_run() {
    let mut gate = EvalGate::default();
    let eval_set = valid_eval_set("cap.demo.eval");
    gate.register_eval_set(eval_set).expect("eval set is valid");

    assert_eq!(
        gate.assert_publish_ready("cap.demo.eval"),
        Err(EvalError::MissingPassingEvalRun)
    );

    gate.record_run(EvalRunInput {
        capability_id: "cap.demo.eval".into(),
        eval_set_version: "eval-v1".into(),
        pass_rate_percent: 90,
        p95_score_percent: 85,
        adversarial_passed: true,
        linguistic_passed: true,
        signed: true,
    })
    .expect("passing eval run is valid");

    assert_eq!(gate.assert_publish_ready("cap.demo.eval"), Ok(()));
}

#[test]
fn eval_set_requires_signature_adversarial_and_linguistic_coverage() {
    let mut unsigned = valid_eval_set("cap.demo.unsigned");
    unsigned.signed = false;
    assert_eq!(
        EvalGate::default().register_eval_set(unsigned),
        Err(EvalError::UnsignedEvalSet)
    );

    let mut missing_adversarial = valid_eval_set("cap.demo.adversarial");
    missing_adversarial
        .cases
        .retain(|case| case.adversarial_kind != Some(AdversarialKind::ToolExfiltration));
    assert_eq!(
        EvalGate::default().register_eval_set(missing_adversarial),
        Err(EvalError::MissingAdversarialCoverage)
    );

    let mut missing_locale = valid_eval_set("cap.demo.locale");
    missing_locale
        .cases
        .retain(|case| case.locale != REQUIRED_LINGUISTIC_COHORT_LOCALES[1]);
    assert_eq!(
        EvalGate::default().register_eval_set(missing_locale),
        Err(EvalError::MissingLinguisticCoverage)
    );
}

#[test]
fn eval_run_must_match_thresholds_and_latest_set_version() {
    let mut gate = EvalGate::default();
    gate.register_eval_set(valid_eval_set("cap.demo.threshold"))
        .unwrap();

    assert_eq!(
        gate.record_run(EvalRunInput {
            capability_id: "cap.demo.threshold".into(),
            eval_set_version: "old".into(),
            pass_rate_percent: 90,
            p95_score_percent: 85,
            adversarial_passed: true,
            linguistic_passed: true,
            signed: true,
        }),
        Err(EvalError::EvalRunVersionMismatch)
    );

    assert_eq!(
        gate.record_run(EvalRunInput {
            capability_id: "cap.demo.threshold".into(),
            eval_set_version: "eval-v1".into(),
            pass_rate_percent: 79,
            p95_score_percent: 85,
            adversarial_passed: true,
            linguistic_passed: true,
            signed: true,
        }),
        Err(EvalError::EvalRunBelowThreshold)
    );
}

fn valid_eval_set(capability_id: &str) -> EvalSetInput {
    let mut cases = vec![
        case("case-alpha", REQUIRED_LINGUISTIC_COHORT_LOCALES[0], None),
        case("case-beta", REQUIRED_LINGUISTIC_COHORT_LOCALES[1], None),
        case("case-gamma", REQUIRED_LINGUISTIC_COHORT_LOCALES[2], None),
    ];
    for (id, kind) in [
        ("adv-prompt", AdversarialKind::PromptInjection),
        ("adv-class", AdversarialKind::DataClassViolation),
        ("adv-autonomy", AdversarialKind::AutonomyBypass),
        ("adv-tool", AdversarialKind::ToolExfiltration),
    ] {
        cases.push(case(id, REQUIRED_LINGUISTIC_COHORT_LOCALES[0], Some(kind)));
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

fn case(id: &str, locale: &str, adversarial_kind: Option<AdversarialKind>) -> EvalCaseInput {
    EvalCaseInput {
        case_id: id.into(),
        locale: locale.into(),
        input_ref: format!("inputs/{id}.json"),
        expected_ref: format!("expected/{id}.json"),
        adversarial_kind,
        deterministic_seed: Some(42),
    }
}
