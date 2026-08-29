//! Hand-rolled runtime evaluator over `PolicySet`, with its own evaluation
//! log. Distinct from the `cedar-policy`-backed PDP engine.

use std::collections::BTreeMap;

use crate::authorization::PolicyError;
use crate::authorization::{AuthorizationDecision, AuthorizationQuery, AuthorizationSubject};
use crate::authz_engine;
use crate::backbone_write::backbone_write_policy_versions;
use crate::policy::{PolicyEffect, PolicyVersion};
use crate::policy_set::PolicySet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CedarRuntimeError {
    MissingTenantId,
    MissingAction,
    MissingResourceType,
    MissingAuditCorrelation,
    InvalidRoleContext,
    Policy(PolicyError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarEvaluationLogEntry {
    pub decision_ref: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: Option<String>,      // data_class: INTERNAL_ONLY
    pub action: String,                    // data_class: INTERNAL_ONLY
    pub resource_type: String,             // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>,       // data_class: INTERNAL_ONLY
    pub effect: PolicyEffect,              // data_class: INTERNAL_ONLY
    pub determining_policies: Vec<String>, // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,      // data_class: INTERNAL_ONLY
    pub reason: String,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarRuntimeEvaluation {
    pub decision_ref: String,                  // data_class: INTERNAL_ONLY
    pub decision: authz_engine::AuthzDecision, // data_class: INTERNAL_ONLY
    pub log_entry: CedarEvaluationLogEntry,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarRuntimeEvaluator {
    policy_set: PolicySet,             // data_class: INTERNAL_ONLY
    log: Vec<CedarEvaluationLogEntry>, // data_class: INTERNAL_ONLY
    next_decision_sequence: u64,       // data_class: INTERNAL_ONLY
}

impl Default for CedarRuntimeEvaluator {
    fn default() -> Self {
        Self {
            policy_set: PolicySet::default(),
            log: Vec::new(),
            next_decision_sequence: 1,
        }
    }
}

impl CedarRuntimeEvaluator {
    pub fn from_policy_versions(
        versions: impl IntoIterator<Item = PolicyVersion>,
    ) -> Result<Self, CedarRuntimeError> {
        let mut policy_set = PolicySet::default();
        for version in versions {
            policy_set
                .publish(version)
                .map_err(CedarRuntimeError::Policy)?;
        }
        Ok(Self {
            policy_set,
            log: Vec::new(),
            next_decision_sequence: 1,
        })
    }

    pub fn with_backbone_write_policies(
        tenant_id: impl Into<String>,
    ) -> Result<Self, CedarRuntimeError> {
        Self::from_policy_versions(backbone_write_policy_versions(tenant_id))
    }

    pub fn evaluate(
        &mut self,
        request: authz_engine::AuthzRequest,
        audit_correlation_id: impl Into<String>,
    ) -> Result<CedarRuntimeEvaluation, CedarRuntimeError> {
        validate_authz_request(&request)?;
        let audit_correlation_id = audit_correlation_id.into();
        if audit_correlation_id.trim().is_empty() {
            return Err(CedarRuntimeError::MissingAuditCorrelation);
        }
        let query = authorization_query_from_authz_request(&request)?;
        let authorization = self.policy_set.authorize(&query);
        let decision = authz_decision_from_authorization(&authorization);
        let decision_ref = self.next_decision_ref(&decision);
        let log_entry = CedarEvaluationLogEntry {
            decision_ref: decision_ref.clone(),
            tenant_id: request.tenant_id,
            principal_id: request.principal_id,
            action: request.action,
            resource_type: request.resource_type,
            resource_id: request.resource_id,
            effect: decision.effect,
            determining_policies: decision.determining_policies.clone(),
            audit_correlation_id,
            reason: authorization.reason,
        };
        self.log.push(log_entry.clone());
        Ok(CedarRuntimeEvaluation {
            decision_ref,
            decision,
            log_entry,
        })
    }

    pub fn eval_log(&self, filter: &authz_engine::EvalLogFilter) -> Vec<CedarEvaluationLogEntry> {
        self.log
            .iter()
            .filter(|entry| {
                filter
                    .principal_id
                    .as_ref()
                    .is_none_or(|principal_id| entry.principal_id.as_ref() == Some(principal_id))
                    && filter.effect.is_none_or(|effect| entry.effect == effect)
                    && filter
                        .resource_type
                        .as_ref()
                        .is_none_or(|resource_type| entry.resource_type == *resource_type)
            })
            .take(filter.limit as usize)
            .cloned()
            .collect()
    }

    pub fn log_len(&self) -> usize {
        self.log.len()
    }

    fn next_decision_ref(&mut self, decision: &authz_engine::AuthzDecision) -> String {
        let effect = match decision.effect {
            PolicyEffect::Allow => "allow",
            PolicyEffect::Deny => "deny",
        };
        let policy_ref = decision
            .determining_policies
            .first()
            .map_or("default", String::as_str);
        let sequence = self.next_decision_sequence;
        self.next_decision_sequence += 1;
        format!("cedar:{effect}:{policy_ref}:{sequence}")
    }
}

fn validate_authz_request(request: &authz_engine::AuthzRequest) -> Result<(), CedarRuntimeError> {
    if request.tenant_id.trim().is_empty() {
        return Err(CedarRuntimeError::MissingTenantId);
    }
    if request.action.trim().is_empty() {
        return Err(CedarRuntimeError::MissingAction);
    }
    if request.resource_type.trim().is_empty() {
        return Err(CedarRuntimeError::MissingResourceType);
    }
    Ok(())
}

fn authorization_query_from_authz_request(
    request: &authz_engine::AuthzRequest,
) -> Result<AuthorizationQuery, CedarRuntimeError> {
    Ok(AuthorizationQuery {
        subject: AuthorizationSubject {
            tenant_id: request.tenant_id.clone(),
            roles: roles_from_context(request)?,
        },
        action: request.action.clone(),
        resource: resource_ref_from_request(request),
        attributes: string_attributes_from_context(request),
    })
}

fn roles_from_context(
    request: &authz_engine::AuthzRequest,
) -> Result<Vec<String>, CedarRuntimeError> {
    let Some(value) = request.context.get("roles") else {
        return Ok(vec![request.principal_type.as_cedar_str().to_string()]);
    };
    match value {
        serde_json::Value::String(role) if !role.trim().is_empty() => Ok(vec![role.clone()]),
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| match value {
                serde_json::Value::String(role) if !role.trim().is_empty() => Ok(role.clone()),
                _ => Err(CedarRuntimeError::InvalidRoleContext),
            })
            .collect(),
        _ => Err(CedarRuntimeError::InvalidRoleContext),
    }
}

fn resource_ref_from_request(request: &authz_engine::AuthzRequest) -> String {
    match request.resource_id.as_ref() {
        Some(resource_id) => format!("{}:{resource_id}", request.resource_type),
        None => request.resource_type.clone(),
    }
}

fn string_attributes_from_context(
    request: &authz_engine::AuthzRequest,
) -> BTreeMap<String, String> {
    request
        .context
        .iter()
        .filter_map(|(key, value)| match value {
            serde_json::Value::String(value) => Some((key.clone(), value.clone())),
            serde_json::Value::Bool(value) => Some((key.clone(), value.to_string())),
            serde_json::Value::Number(value) => Some((key.clone(), value.to_string())),
            _ => None,
        })
        .collect()
}

fn authz_decision_from_authorization(
    authorization: &AuthorizationDecision,
) -> authz_engine::AuthzDecision {
    match (authorization.allowed, authorization.matched_policy.clone()) {
        (true, Some(policy_id)) => authz_engine::AuthzDecision::allow(vec![policy_id]),
        (false, Some(policy_id)) => authz_engine::AuthzDecision::explicit_deny(vec![policy_id]),
        _ => authz_engine::AuthzDecision::default_deny(),
    }
}
