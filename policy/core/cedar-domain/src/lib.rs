//! Cedar-shaped authorization policy kernel.
//!
//! This is deliberately pure: it stores versioned policy records and evaluates
//! role + attribute predicates without network, storage, or runtime side effects.
//!
//! The [`authz_engine`] module carries the `AuthzRequest` / `AuthzDecision` /
//! `EvalLogFilter` value types that encode the Cedar evaluation contract without
//! importing any framework crates beyond `serde`.
//!
//! The [`obligations`] module carries Cedar-style annotation/obligation
//! key-value pairs that ride out with `Allow` decisions for downstream PEP
//! step-up, audit, and redaction.
//!
//! The [`rebac`] module carries the Zanzibar-style relationship-tuple
//! vocabulary: tuples, usersets, consistency tokens, and the tuple-store port.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod authorization;
pub mod authz_engine;
pub mod backbone_write;
pub mod lint;
pub mod obligations;
pub mod policy;
pub mod policy_diff;
pub mod policy_set;
pub mod rebac;
pub mod runtime_evaluator;

pub use authorization::{
    AuthorizationDecision, AuthorizationQuery, AuthorizationSubject, PolicyError,
};
pub use backbone_write::{
    BACKBONE_WRITE_POLICY_VERSION, BackboneWriteOperation, backbone_write_policy_versions,
};
pub use lint::{LintSeverity, PolicyLintFinding, PolicyLintReport, lint_policy_version};
pub use obligations::{AnnotationKind, PolicyAnnotation};
pub use policy::{
    PolicyEffect, PolicyRule, PolicyRuleInput, PolicyScope, PolicyVersion, PublishedPolicy,
};
pub use policy_diff::{ImpactReport, RuleDelta, diff_policy_versions};
pub use policy_set::PolicySet;
pub use runtime_evaluator::{
    CedarEvaluationLogEntry, CedarRuntimeError, CedarRuntimeEvaluation, CedarRuntimeEvaluator,
};
