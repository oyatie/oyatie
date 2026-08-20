// Reference implementation — feature-flags MockProvider for Rust unit tests.
//
// This file is canonical. Copy-paste into `tests/common/feature_flag_mock.rs`
// when a unit test must evaluate a flag without standing up the side-car.
//
// Doctrine references:
//   - ADR-0159  Runtime feature-flag substrate
//   - ADR-0145  Three-tier evaluator latency budget
//   - microservices/feature-flags/PRD.md §Personas (engineer test path)

use async_trait::async_trait;
use feature_flags_client::{Context, EvalError, EvalResult, FFProvider};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-process mock provider — zero side-car, zero Cedar.
///
/// Use in `cargo nextest` paths where you want a deterministic flag result
/// without taking the side-car dependency. NOT a substitute for
/// `feature-flags-local` in integration tests; this is unit-scope only.
pub struct MockProvider {
    bool_overrides: RwLock<HashMap<String, bool>>,
    string_overrides: RwLock<HashMap<String, String>>,
    number_overrides: RwLock<HashMap<String, f64>>,
    json_overrides: RwLock<HashMap<String, Value>>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            bool_overrides: RwLock::new(HashMap::new()),
            string_overrides: RwLock::new(HashMap::new()),
            number_overrides: RwLock::new(HashMap::new()),
            json_overrides: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_bool(self, key: &str, value: bool) -> Self {
        self.bool_overrides.write().unwrap().insert(key.to_owned(), value);
        self
    }

    pub fn with_string(self, key: &str, value: &str) -> Self {
        self.string_overrides.write().unwrap().insert(key.to_owned(), value.to_owned());
        self
    }

    pub fn with_number(self, key: &str, value: f64) -> Self {
        self.number_overrides.write().unwrap().insert(key.to_owned(), value);
        self
    }

    pub fn with_json(self, key: &str, value: Value) -> Self {
        self.json_overrides.write().unwrap().insert(key.to_owned(), value);
        self
    }
}

#[async_trait]
impl FFProvider for MockProvider {
    async fn eval_bool(&self, key: &str, default: bool, _ctx: &Context) -> Result<EvalResult<bool>, EvalError> {
        let v = self.bool_overrides.read().unwrap().get(key).copied().unwrap_or(default);
        Ok(EvalResult { value: v, variant: if v { "on" } else { "off" }.into(), reason: "MOCK".into() })
    }

    async fn eval_string(&self, key: &str, default: &str, _ctx: &Context) -> Result<EvalResult<String>, EvalError> {
        let v = self.string_overrides.read().unwrap().get(key).cloned().unwrap_or_else(|| default.to_owned());
        Ok(EvalResult { value: v.clone(), variant: v, reason: "MOCK".into() })
    }

    async fn eval_number(&self, key: &str, default: f64, _ctx: &Context) -> Result<EvalResult<f64>, EvalError> {
        let v = self.number_overrides.read().unwrap().get(key).copied().unwrap_or(default);
        Ok(EvalResult { value: v, variant: v.to_string(), reason: "MOCK".into() })
    }

    async fn eval_json(&self, key: &str, default: Value, _ctx: &Context) -> Result<EvalResult<Value>, EvalError> {
        let v = self.json_overrides.read().unwrap().get(key).cloned().unwrap_or(default);
        Ok(EvalResult { value: v.clone(), variant: "json".into(), reason: "MOCK".into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feature_flags_client::Context;

    #[tokio::test]
    async fn override_returns_set_value() {
        let p = MockProvider::new().with_bool("k", true);
        let ctx = Context::for_tenant("t1");
        let r = p.eval_bool("k", false, &ctx).await.unwrap();
        assert!(r.value);
        assert_eq!(r.variant, "on");
        assert_eq!(r.reason, "MOCK");
    }

    #[tokio::test]
    async fn unset_falls_back_to_default() {
        let p = MockProvider::new();
        let ctx = Context::for_tenant("t1");
        let r = p.eval_bool("k", false, &ctx).await.unwrap();
        assert!(!r.value);
    }

    #[tokio::test]
    async fn json_override_round_trip() {
        let p = MockProvider::new().with_json("k", serde_json::json!({"x": 1}));
        let ctx = Context::for_tenant("t1");
        let r = p.eval_json("k", Value::Null, &ctx).await.unwrap();
        assert_eq!(r.value, serde_json::json!({"x": 1}));
    }
}
