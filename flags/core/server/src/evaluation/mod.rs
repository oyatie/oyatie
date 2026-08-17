//! evaluation subsystem for the `flags` capability server.
//!
//! Capability-first decomposition (ADR-0562 reorg): the pure, cloud-agnostic evaluation engine has
//! been lifted out of this bundled server into the `flags-evaluation-domain` crate at
//! `flags/core/evaluation-domain` (behind the `flags/*/*` workspace glob). This module is now the
//! thin SEAM the server uses to evaluate flags: it re-exports the domain so the OFREP/gRPC/REST
//! faces resolve flags through one deterministic engine, while the engine itself carries zero
//! cloud/persistence/identity/runtime coupling.
//!
//! The flag SOURCE (storage/cloud/identity adapters) is DEFERRED behind
//! [`flags_evaluation_domain::FlagSource`] (clean-arch ports-in-core per ADR-0570); this server
//! depends on the domain and will inject a concrete `FlagSource` once the storage adapter lands.

pub use flags_evaluation_domain::{
    AttrValue, Condition, EvalErrorCode, Evaluation, EvaluationContext, Flag, FlagKey, FlagSource,
    FlagSourceError, FlagValue, Operand, Operator, Reason, Rollout, Rule, RuleOutcome,
    TOTAL_BASIS_POINTS, Variant, VariantKey, evaluate,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_evaluation_seam_resolves_through_domain_engine() {
        // Smoke test that the server's evaluation seam reaches the deterministic domain engine.
        let flag = Flag {
            key: "smoke".into(),
            enabled: false,
            variants: vec![
                Variant {
                    key: "on".into(),
                    value: FlagValue::Bool(true),
                },
                Variant {
                    key: "off".into(),
                    value: FlagValue::Bool(false),
                },
            ],
            rules: vec![],
            default_rollout: None,
            default_variant: "off".into(),
            off_variant: "off".into(),
        };
        let ev = evaluate(&flag, &EvaluationContext::for_key("u1"));
        assert_eq!(ev.reason, Reason::Disabled);
        assert_eq!(ev.variant, "off");
    }
}
