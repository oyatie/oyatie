// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_capability_domain::{AutonomyTier, Capability, CapabilityAction};
use intelligence_policy_domain::{
    AutonomyCapReason, AutonomyCapSource, AutonomyCeilingInputs, AutonomyVerdict, TenantPolicy,
    agentic_ads_cap, subject_class_cap, vertical_pack_cap,
};
use data_boundary_kernel::{AgeBand, DataClass, PrivacyDataClass, SubjectClass};

#[test]
fn tenant_policy_emits_replayable_autonomy_decision_inputs() {
    let policy = TenantPolicy::new("ten_alpha".into(), AutonomyTier::T2Advisory);

    let allowed = policy.evaluate(&capability("cap.demo.view", AutonomyTier::T1ViewOnly));
    assert_eq!(allowed.tenant_id, "ten_alpha");
    assert_eq!(allowed.capability_id, "cap.demo.view");
    assert_eq!(allowed.configured_ceiling, AutonomyTier::T2Advisory);
    assert_eq!(allowed.tenant_configured_ceiling, AutonomyTier::T2Advisory);
    assert_eq!(allowed.principal_ceiling, AutonomyTier::T2Advisory);
    assert_eq!(allowed.capability_required_cap, AutonomyTier::T1ViewOnly);
    assert_eq!(allowed.agentic_ads_cap, AutonomyTier::T4AutoExecute);
    assert_eq!(allowed.vertical_pack_cap, AutonomyTier::T4AutoExecute);
    assert_eq!(allowed.subject_class, SubjectClass::Adult);
    assert_eq!(allowed.subject_class_cap, AutonomyTier::T4AutoExecute);
    assert_eq!(allowed.denial_threshold, AutonomyTier::T2Advisory);
    assert_eq!(allowed.effective_ceiling, AutonomyTier::T1ViewOnly);
    assert_eq!(allowed.required_tier, AutonomyTier::T1ViewOnly);
    assert_eq!(allowed.verdict, AutonomyVerdict::Allow);
    assert_eq!(allowed.blocking_cap_source, None);
    assert_eq!(allowed.blocking_cap_reason, None);
    assert_eq!(
        allowed.lowering_cap_source,
        AutonomyCapSource::CapabilityRequired
    );
    assert_eq!(
        allowed.lowering_cap_reason,
        AutonomyCapReason::CapabilityRequiredTier
    );
    assert!(allowed.allowed());

    let denied = policy.evaluate(&capability(
        "cap.demo.execute",
        AutonomyTier::T3ExecuteWithApproval,
    ));
    assert_eq!(denied.denial_threshold, AutonomyTier::T2Advisory);
    assert_eq!(denied.effective_ceiling, AutonomyTier::T2Advisory);
    assert_eq!(denied.verdict, AutonomyVerdict::Deny);
    assert_eq!(
        denied.blocking_cap_source,
        Some(AutonomyCapSource::TenantConfigured)
    );
    assert_eq!(
        denied.blocking_cap_reason,
        Some(AutonomyCapReason::TenantConfiguredCeiling)
    );
    assert!(!denied.allowed());
}

#[test]
fn tenant_policy_uses_lowest_effective_autonomy_ceiling() {
    let policy = TenantPolicy::new("ten_alpha".into(), AutonomyTier::T4AutoExecute);

    let denied_by_principal = policy.evaluate_with_principal_ceiling(
        &capability("cap.demo.approve", AutonomyTier::T3ExecuteWithApproval),
        AutonomyTier::T2Advisory,
    );
    assert_eq!(
        denied_by_principal.effective_ceiling,
        AutonomyTier::T2Advisory
    );
    assert_eq!(denied_by_principal.verdict, AutonomyVerdict::Deny);
    assert_eq!(
        denied_by_principal.blocking_cap_source,
        Some(AutonomyCapSource::Principal)
    );

    let allowed_by_effective = policy.evaluate_with_principal_ceiling(
        &capability("cap.demo.view", AutonomyTier::T2Advisory),
        AutonomyTier::T2Advisory,
    );
    assert_eq!(allowed_by_effective.verdict, AutonomyVerdict::Allow);
    assert_eq!(
        allowed_by_effective.effective_ceiling,
        AutonomyTier::T2Advisory
    );
}

#[test]
fn autonomy_inputs_default_absent_caps_to_t4_and_break_ties_deterministically() {
    let policy = TenantPolicy::new("ten_alpha".into(), AutonomyTier::T1ViewOnly);
    let decision = policy.evaluate_inputs(AutonomyCeilingInputs::new(
        "ten_alpha".into(),
        "cap.demo.tie".into(),
        AutonomyTier::T1ViewOnly,
        AutonomyTier::T1ViewOnly,
        AutonomyTier::T3ExecuteWithApproval,
        AutonomyTier::T4AutoExecute,
        AutonomyTier::T4AutoExecute,
        SubjectClass::Adult,
        AutonomyTier::T4AutoExecute,
    ));

    assert_eq!(decision.agentic_ads_cap, AutonomyTier::T4AutoExecute);
    assert_eq!(decision.vertical_pack_cap, AutonomyTier::T4AutoExecute);
    assert_eq!(decision.subject_class_cap, AutonomyTier::T4AutoExecute);
    assert_eq!(decision.verdict, AutonomyVerdict::Deny);
    assert_eq!(
        decision.blocking_cap_source,
        Some(AutonomyCapSource::TenantConfigured)
    );
    assert_eq!(
        decision.lowering_cap_source,
        AutonomyCapSource::TenantConfigured
    );
}

#[test]
fn vertical_pack_caps_health_and_financial_regulated_data_with_neutral_markers() {
    let healthcare_phi = capability_with_classes(
        "cap.healthcare.chart",
        AutonomyTier::T4AutoExecute,
        vec![privacy_class(DataClass::Phi)],
    );
    assert_eq!(
        vertical_pack_cap(&["clinical-safety-pack".into()], &healthcare_phi),
        AutonomyTier::T2Advisory
    );

    let healthcare_article_23 = capability_with_classes(
        "cap.healthcare.article-23",
        AutonomyTier::T4AutoExecute,
        vec![
            privacy_class(DataClass::SensitivePipaArticle23),
            privacy_class(DataClass::PipaArticle23),
        ],
    );
    assert_eq!(
        vertical_pack_cap(&["protected-health-pack".into()], &healthcare_article_23),
        AutonomyTier::T2Advisory
    );

    let fintech_pci = capability_with_classes(
        "cap.fintech.charge",
        AutonomyTier::T4AutoExecute,
        vec![privacy_class(DataClass::Pci)],
    );
    assert_eq!(
        vertical_pack_cap(&["payment-card-pack".into()], &fintech_pci),
        AutonomyTier::T2Advisory
    );

    let fintech_credit = capability_with_classes(
        "cap.fintech.credit-report",
        AutonomyTier::T4AutoExecute,
        vec![
            privacy_class(DataClass::FinancialRegulatedCredit),
            privacy_class(DataClass::Financial),
        ],
    );
    assert_eq!(
        vertical_pack_cap(&["regulated-credit-pack".into()], &fintech_credit),
        AutonomyTier::T2Advisory
    );

    let internal = capability("cap.demo.internal", AutonomyTier::T4AutoExecute);
    assert_eq!(
        vertical_pack_cap(
            &["clinical-safety-pack".into(), "payment-card-pack".into()],
            &internal
        ),
        AutonomyTier::T4AutoExecute
    );
}

#[test]
fn subject_class_cap_uses_typed_subject_class_not_data_class_bootstrap() {
    assert_eq!(
        subject_class_cap(SubjectClass::Minor {
            age_band: AgeBand::Under14
        }),
        AutonomyTier::T1ViewOnly
    );
    assert_eq!(
        subject_class_cap(SubjectClass::Vulnerable),
        AutonomyTier::T2Advisory
    );
    assert_eq!(
        subject_class_cap(SubjectClass::Elderly),
        AutonomyTier::T2Advisory
    );
    assert_eq!(
        subject_class_cap(SubjectClass::Adult),
        AutonomyTier::T4AutoExecute
    );
    assert_eq!(
        subject_class_cap(SubjectClass::Authority),
        AutonomyTier::T4AutoExecute
    );
}

#[test]
fn typed_agentic_ads_actions_cap_bid_and_budget_adjust_to_t1() {
    let bid = capability_with_action(
        "cap.ads.bid",
        "ads",
        CapabilityAction::AdsBid,
        AutonomyTier::T4AutoExecute,
        vec![privacy_class(DataClass::InternalOnly)],
    );
    assert_eq!(agentic_ads_cap(&bid), AutonomyTier::T1ViewOnly);

    let adjust = capability_with_action(
        "cap.ads.budget-adjust",
        "ads",
        CapabilityAction::AdsBudgetAdjust,
        AutonomyTier::T4AutoExecute,
        vec![privacy_class(DataClass::InternalOnly)],
    );
    assert_eq!(agentic_ads_cap(&adjust), AutonomyTier::T1ViewOnly);

    let unrelated_ads_namespace = capability_with_action(
        "cap.ads.report",
        "ads",
        CapabilityAction::Other,
        AutonomyTier::T4AutoExecute,
        vec![privacy_class(DataClass::InternalOnly)],
    );
    assert_eq!(
        agentic_ads_cap(&unrelated_ads_namespace),
        AutonomyTier::T4AutoExecute
    );
}

fn capability(id: &str, tier: AutonomyTier) -> Capability {
    capability_with_classes(id, tier, vec![privacy_class(DataClass::InternalOnly)])
}

fn capability_with_classes(
    id: &str,
    tier: AutonomyTier,
    privacy_classes: Vec<PrivacyDataClass>,
) -> Capability {
    capability_with_action(id, "demo", CapabilityAction::Other, tier, privacy_classes)
}

fn capability_with_action(
    id: &str,
    namespace: &str,
    action: CapabilityAction,
    tier: AutonomyTier,
    privacy_classes: Vec<PrivacyDataClass>,
) -> Capability {
    Capability::new_with_action(
        id.into(),
        namespace.into(),
        action,
        tier,
        privacy_classes,
        "oyatie.foundry.capability.invoked".into(),
    )
    .unwrap()
}

fn privacy_class(data_class: DataClass) -> PrivacyDataClass {
    PrivacyDataClass::new(data_class).expect("test fixture uses privacy-program data classes")
}
