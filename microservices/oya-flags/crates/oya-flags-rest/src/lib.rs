//! # oya-flags-rest
//!
//! REST adapter for the oya-flags OpenFeature provider (ADR-0481).
//! Exposes the OpenFeature Remote Evaluation Protocol (OFREP) surface:
//!
//!   POST /ofrep/v1/evaluate/flags/{key}
//!
//! All HTTP wiring lives in oya-flags-app; this crate provides the
//! request/response types and the evaluation dispatch logic.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_flags_kernel::{EvaluationContext, EvaluationReason, EvaluationResult, FlagKey, FlagValue, KernelError};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// OFREP wire types
// ---------------------------------------------------------------------------

/// OFREP POST body: `{"context": {"targetingKey": "...", "tenant_id": "..."}}`
#[derive(Debug, Deserialize)]
pub struct OFREPRequest {
    pub context: OFREPContext,
}

#[derive(Debug, Deserialize)]
pub struct OFREPContext {
    #[serde(rename = "targetingKey")]
    pub targeting_key: Option<String>,
    pub tenant_id: Option<String>,
}

/// OFREP success response.
#[derive(Debug, Serialize)]
pub struct OFREPResponse {
    pub key: String,
    pub reason: String,
    pub variant: Option<String>,
    pub value: serde_json::Value,
}

/// OFREP error response.
#[derive(Debug, Serialize)]
pub struct OFREPErrorResponse {
    pub key: String,
    pub error_code: String,
    pub error_details: String,
}

// ---------------------------------------------------------------------------
// Dispatch helper
// ---------------------------------------------------------------------------

/// Build an [`EvaluationContext`] from an OFREP request context.
pub fn build_eval_context(req_ctx: &OFREPContext) -> Result<EvaluationContext, KernelError> {
    let tenant_id = req_ctx
        .tenant_id
        .clone()
        .or_else(|| req_ctx.targeting_key.clone())
        .ok_or_else(|| {
            KernelError::InvalidContext("tenant_id or targetingKey required".to_owned())
        })?;
    let mut ctx = EvaluationContext::new(tenant_id)?;
    if let Some(ref subject) = req_ctx.targeting_key {
        ctx = ctx.with_subject(subject.clone());
    }
    Ok(ctx)
}

/// Convert an [`EvaluationResult`] to the OFREP JSON wire representation.
pub fn result_to_response(key: &FlagKey, result: &EvaluationResult) -> OFREPResponse {
    let value = flag_value_to_json(&result.value);
    OFREPResponse {
        key: key.as_str().to_owned(),
        reason: reason_to_str(result.reason).to_owned(),
        variant: result.variant.clone(),
        value,
    }
}

fn flag_value_to_json(v: &FlagValue) -> serde_json::Value {
    match v {
        FlagValue::Bool(b) => serde_json::Value::Bool(*b),
        FlagValue::String(s) => serde_json::Value::String(s.clone()),
        FlagValue::Int(i) => serde_json::Value::Number((*i).into()),
        FlagValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
    }
}

fn reason_to_str(reason: EvaluationReason) -> &'static str {
    match reason {
        EvaluationReason::Default => "DEFAULT",
        EvaluationReason::TargetingMatch => "TARGETING_MATCH",
        EvaluationReason::Cached => "CACHED",
        EvaluationReason::Disabled => "DISABLED",
        EvaluationReason::Error => "ERROR",
        EvaluationReason::Unknown => "UNKNOWN",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oya_flags_kernel::{FlagValue, EvaluationResult, EvaluationReason, FlagKey};

    #[test]
    fn build_eval_context_from_tenant_id() {
        let req_ctx = OFREPContext {
            targeting_key: None,
            tenant_id: Some("tenant-abc".to_owned()),
        };
        let ctx = build_eval_context(&req_ctx).unwrap();
        assert_eq!(ctx.tenant_id, "tenant-abc");
    }

    #[test]
    fn build_eval_context_missing_both_errors() {
        let req_ctx = OFREPContext { targeting_key: None, tenant_id: None };
        assert!(build_eval_context(&req_ctx).is_err());
    }

    #[test]
    fn result_to_response_bool() {
        let key = FlagKey::new("dark-launch").unwrap();
        let result = EvaluationResult::resolved(FlagValue::Bool(true), EvaluationReason::TargetingMatch);
        let resp = result_to_response(&key, &result);
        assert_eq!(resp.key, "dark-launch");
        assert_eq!(resp.reason, "TARGETING_MATCH");
        assert_eq!(resp.value, serde_json::Value::Bool(true));
    }
}
