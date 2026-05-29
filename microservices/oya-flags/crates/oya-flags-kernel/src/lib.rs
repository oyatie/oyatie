//! # oya-flags-kernel
//!
//! Pure-Rust OpenFeature evaluation state machine (ADR-0481).
//! No I/O, no HTTP, no async. Sub-ms flag resolution target.
//!
//! ## Deliverable coverage
//!
//! - [`FlagKey`] — validated flag identifier value object
//! - [`EvaluationContext`] — tenant + subject context for targeting rules
//! - [`FlagValue`] — typed flag variant (bool / string / int / float / object)
//! - [`EvaluationResult`] — resolved value + reason + error code envelope
//! - [`FlagResolver`] — trait seam for adapter implementations
//!
//! ## Performance invariant (ADR-0481)
//!
//! All evaluation paths are synchronous and allocation-minimal. The sub-ms
//! target is enforced by the SLO at `microservices/oya-flags/slos/availability.openslo.yaml`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All kernel-level flag evaluation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// The requested flag key does not exist in the store.
    FlagNotFound(String),
    /// The flag exists but is disabled (targeting off).
    FlagDisabled,
    /// The requested type does not match the flag's stored type.
    TypeMismatch { expected: &'static str, actual: &'static str },
    /// The evaluation context was invalid or incomplete.
    InvalidContext(String),
    /// A general error from an adapter implementation.
    General(String),
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelError::FlagNotFound(key) => write!(f, "flag not found: {key}"),
            KernelError::FlagDisabled => f.write_str("flag is disabled"),
            KernelError::TypeMismatch { expected, actual } => {
                write!(f, "type mismatch: expected {expected}, got {actual}")
            }
            KernelError::InvalidContext(why) => write!(f, "invalid evaluation context: {why}"),
            KernelError::General(msg) => write!(f, "flag evaluation error: {msg}"),
        }
    }
}

impl std::error::Error for KernelError {}

pub type Result<T> = std::result::Result<T, KernelError>;

// ---------------------------------------------------------------------------
// FlagKey value object
// ---------------------------------------------------------------------------

/// A validated feature flag identifier.
///
/// Must be non-empty and contain only ASCII alphanumeric characters, hyphens,
/// underscores, and forward slashes (namespace separator).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlagKey(String);

impl FlagKey {
    /// Construct a [`FlagKey`] from a string, validating the format.
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        if key.is_empty() {
            return Err(KernelError::InvalidContext("flag key must not be empty".to_owned()));
        }
        if !key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/')) {
            return Err(KernelError::InvalidContext(format!(
                "flag key {key:?} contains invalid characters"
            )));
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FlagKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------
// EvaluationContext
// ---------------------------------------------------------------------------

/// Tenant + subject context forwarded with every flag resolution request.
///
/// Targeting rules in the flag store use these fields for per-tenant /
/// per-subject rollout segmentation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationContext {
    /// Tenant identifier (mandatory per dogfood doctrine — no bypass).
    pub tenant_id: String, // data_class: TENANT_CONFIG
    /// Optional subject identifier (user, service account, workload).
    pub subject_id: Option<String>, // data_class: TENANT_CONFIG
}

impl EvaluationContext {
    pub fn new(tenant_id: impl Into<String>) -> Result<Self> {
        let tenant_id = tenant_id.into();
        if tenant_id.is_empty() {
            return Err(KernelError::InvalidContext(
                "tenant_id must not be empty".to_owned(),
            ));
        }
        Ok(Self { tenant_id, subject_id: None })
    }

    pub fn with_subject(mut self, subject_id: impl Into<String>) -> Self {
        self.subject_id = Some(subject_id.into());
        self
    }
}

// ---------------------------------------------------------------------------
// FlagValue — typed variant
// ---------------------------------------------------------------------------

/// A resolved flag value. Mirrors the OpenFeature type system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FlagValue {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
}

impl FlagValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            FlagValue::Bool(_) => "bool",
            FlagValue::String(_) => "string",
            FlagValue::Int(_) => "int",
            FlagValue::Float(_) => "float",
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            FlagValue::Bool(v) => Ok(*v),
            other => Err(KernelError::TypeMismatch {
                expected: "bool",
                actual: other.type_name(),
            }),
        }
    }

    pub fn as_string(&self) -> Result<&str> {
        match self {
            FlagValue::String(v) => Ok(v.as_str()),
            other => Err(KernelError::TypeMismatch {
                expected: "string",
                actual: other.type_name(),
            }),
        }
    }

    pub fn as_int(&self) -> Result<i64> {
        match self {
            FlagValue::Int(v) => Ok(*v),
            other => Err(KernelError::TypeMismatch {
                expected: "int",
                actual: other.type_name(),
            }),
        }
    }

    pub fn as_float(&self) -> Result<f64> {
        match self {
            FlagValue::Float(v) => Ok(*v),
            other => Err(KernelError::TypeMismatch {
                expected: "float",
                actual: other.type_name(),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// EvaluationReason — OpenFeature resolution reason
// ---------------------------------------------------------------------------

/// Why a particular flag value was returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationReason {
    /// The static default was returned (flag not configured for context).
    Default,
    /// A targeting rule matched the evaluation context.
    TargetingMatch,
    /// The flag was returned from a cache layer.
    Cached,
    /// The flag was disabled; default value returned.
    Disabled,
    /// An error occurred; default value returned.
    Error,
    /// Unknown reason (provider did not specify).
    Unknown,
}

// ---------------------------------------------------------------------------
// EvaluationResult
// ---------------------------------------------------------------------------

/// The complete result of a flag resolution, mirroring the OpenFeature spec.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// The resolved value (or default on error).
    pub value: FlagValue,
    /// Why this value was returned.
    pub reason: EvaluationReason,
    /// The variant/variation key that matched, if any.
    pub variant: Option<String>,
    /// Non-fatal error message if reason is Error.
    pub error_message: Option<String>,
}

impl EvaluationResult {
    pub fn resolved(value: FlagValue, reason: EvaluationReason) -> Self {
        Self { value, reason, variant: None, error_message: None }
    }

    pub fn default_value(value: FlagValue) -> Self {
        Self {
            value,
            reason: EvaluationReason::Default,
            variant: None,
            error_message: None,
        }
    }

    pub fn error(default: FlagValue, message: impl Into<String>) -> Self {
        Self {
            value: default,
            reason: EvaluationReason::Error,
            variant: None,
            error_message: Some(message.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// FlagResolver trait seam
// ---------------------------------------------------------------------------

/// Core evaluation port. Adapter crates implement this against a flag store
/// (e.g. in-memory, PostgreSQL, or a remote OFREP upstream).
pub trait FlagResolver {
    /// Resolve a flag to its typed value for the given context.
    ///
    /// Implementations MUST:
    /// 1. Return the default value (not Err) when the flag is disabled.
    /// 2. Never panic on the evaluation path.
    /// 3. Complete in sub-ms (ADR-0481 latency target).
    fn resolve(
        &self,
        key: &FlagKey,
        default: FlagValue,
        ctx: &EvaluationContext,
    ) -> EvaluationResult;
}

/// In-memory stub resolver — always returns the default value.
/// Used by tests and the app binary before a real adapter is wired.
pub struct DefaultFlagResolver;

impl FlagResolver for DefaultFlagResolver {
    fn resolve(
        &self,
        _key: &FlagKey,
        default: FlagValue,
        _ctx: &EvaluationContext,
    ) -> EvaluationResult {
        EvaluationResult::default_value(default)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_key_valid() {
        let k = FlagKey::new("my-flag/v1").unwrap();
        assert_eq!(k.as_str(), "my-flag/v1");
    }

    #[test]
    fn flag_key_empty_rejected() {
        assert!(FlagKey::new("").is_err());
    }

    #[test]
    fn flag_key_invalid_char_rejected() {
        assert!(FlagKey::new("flag with spaces").is_err());
    }

    #[test]
    fn evaluation_context_requires_tenant() {
        assert!(EvaluationContext::new("").is_err());
    }

    #[test]
    fn evaluation_context_with_subject() {
        let ctx = EvaluationContext::new("tenant-1").unwrap().with_subject("user-42");
        assert_eq!(ctx.tenant_id, "tenant-1");
        assert_eq!(ctx.subject_id.as_deref(), Some("user-42"));
    }

    #[test]
    fn flag_value_type_coercion_bool() {
        let v = FlagValue::Bool(true);
        assert_eq!(v.as_bool().unwrap(), true);
        assert!(v.as_string().is_err());
    }

    #[test]
    fn flag_value_type_name() {
        assert_eq!(FlagValue::Bool(false).type_name(), "bool");
        assert_eq!(FlagValue::String("x".to_owned()).type_name(), "string");
        assert_eq!(FlagValue::Int(0).type_name(), "int");
        assert_eq!(FlagValue::Float(0.0).type_name(), "float");
    }

    #[test]
    fn default_resolver_returns_default() {
        let resolver = DefaultFlagResolver;
        let key = FlagKey::new("feature/dark-launch").unwrap();
        let ctx = EvaluationContext::new("oyatie-dogfood").unwrap();
        let result = resolver.resolve(&key, FlagValue::Bool(false), &ctx);
        assert_eq!(result.value, FlagValue::Bool(false));
        assert_eq!(result.reason, EvaluationReason::Default);
    }

    #[test]
    fn evaluation_result_error_carries_message() {
        let r = EvaluationResult::error(FlagValue::Bool(false), "store unavailable");
        assert_eq!(r.reason, EvaluationReason::Error);
        assert_eq!(r.error_message.as_deref(), Some("store unavailable"));
    }
}
