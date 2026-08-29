//! The shim must be a complete stand-in, not a convenience subset.
//!
//! Consumers reach this crate only through the glob re-export in `src/lib.rs`.
//! Today they happen to name root items only, so a shim that forwarded just
//! those would compile and look correct — and would silently deny a later
//! consumer the module paths the crate has always exposed. That failure would
//! surface as "the crate lost `rebac`" long after this move, so it is pinned
//! here instead: every module and a representative of every kind of item is
//! named through the shim path.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_policy_cedar_domain::obligations::{AnnotationKind, PolicyAnnotation};
use iam_policy_cedar_domain::policy_diff::{ImpactReport, RuleDelta, diff_policy_versions};
use iam_policy_cedar_domain::rebac::{
    RebacObjectRef, RebacRelation, RebacSubjectRef, RebacTenantScope, RebacTuple, RebacTupleStore,
    SnapshotToken, UsersetRewrite, Zookie,
};
use iam_policy_cedar_domain::{
    AuthorizationDecision, AuthorizationQuery, AuthorizationSubject, CedarRuntimeEvaluator,
    PolicyEffect, PolicyError, PolicyLintReport, PolicyRule, PolicyRuleInput, PolicyScope,
    PolicySet, PolicyVersion, PublishedPolicy, lint_policy_version,
};

#[test]
fn every_module_is_reachable_through_the_shim() {
    let tenant = RebacTenantScope::new("ten_shim").expect("tenant scope is valid");
    let tuple = RebacTuple::parse(tenant, "doc:readme#viewer@user:alice")
        .expect("canonical tuple parses through the shim");
    assert_eq!(tuple.to_canonical_string(), "doc:readme#viewer@user:alice");

    let relation = RebacRelation::new("viewer").expect("relation is valid");
    let rewrite = UsersetRewrite::union(vec![
        UsersetRewrite::this(),
        UsersetRewrite::computed_userset(relation),
    ])
    .expect("a non-empty union is valid");
    rewrite.validate().expect("the rewrite tree validates");

    // Named so a narrowed shim fails to compile rather than fails a run.
    let _ = std::mem::size_of::<RebacObjectRef>();
    let _ = std::mem::size_of::<RebacSubjectRef>();
    let _ = std::mem::size_of::<SnapshotToken>();
    let _ = std::mem::size_of::<Zookie>();
    let _ = AnnotationKind::Obligation;
    let _ = std::mem::size_of::<PolicyAnnotation>();
    let _ = std::mem::size_of::<ImpactReport>();
    let _ = std::mem::size_of::<RuleDelta>();
    let _: fn(&PolicyVersion, &PolicyVersion) -> ImpactReport = diff_policy_versions;
    fn _tuple_store_port_is_nameable<T: RebacTupleStore>() {}
}

#[test]
fn every_root_item_is_reachable_through_the_shim() {
    let mut policies = PolicySet::default();
    assert!(policies.get("pol_absent", "1.0.0").is_none());

    let _: fn(&PolicyVersion) -> PolicyLintReport = lint_policy_version;
    let _ = CedarRuntimeEvaluator::default();
    let _ = PolicyEffect::Allow;
    let _ = PolicyScope::Global;
    let _ = PolicyError::InvalidPolicyId;
    let _ = std::mem::size_of::<PolicyRule>();
    let _ = std::mem::size_of::<PolicyRuleInput>();
    let _ = std::mem::size_of::<PublishedPolicy>();
    let _ = std::mem::size_of::<AuthorizationQuery>();
    let _ = std::mem::size_of::<AuthorizationSubject>();
    let _ = std::mem::size_of::<AuthorizationDecision>();
}
