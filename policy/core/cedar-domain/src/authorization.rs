//! The authorization query surface: who is asking, about what, and the
//! decision that comes back.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::obligations::PolicyAnnotation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationSubject {
    pub tenant_id: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationQuery {
    pub subject: AuthorizationSubject,
    pub action: String,
    pub resource: String,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reason: String,
    pub matched_policy: Option<String>,
    /// Cedar-style annotations collected from the matching Allow rule.
    ///
    /// # Forbid-wins invariant
    ///
    /// This field is **always empty when `allowed == false`**.  A PEP must check
    /// `allowed` first; consuming annotations on a denied decision bypasses the PDP.
    pub annotations: Vec<PolicyAnnotation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyError {
    InvalidPolicyId,
    InvalidSemver,
    EmptyRules,
    EmptyRuleField,
    VersionAlreadyExists,
    SupersedesSelf,
    SupersedesMissing,
    SupersedesScopeMismatch,
    SupersedesNotOlder,
}
