//! OpenTelemetry trace-propagation enforcement check (ADR-0145 Invariant 2).
//!
//! # Why this crate exists
//!
//! ADR-0145 Invariant 2 requires every inter-µservice call to propagate
//! the W3C traceparent header. A check kernel grounds the claim by
//! scanning per-µservice gRPC client adapters for the canonical
//! traceparent-injection token set.
//!
//! # Skeleton scope
//!
//! This crate ships the kernel API + advisory-mode evaluation:
//!
//! - `audit_all_findings` accepts a list of `ClientAdapterSource`
//!   documents (UTF-8 contents of `crates/oya-*-adapter-grpc-*` source
//!   files supplied by the runner) and returns findings.
//! - `validate_advisory` returns an advisory `AdvisoryReport`
//!   summarizing missing propagation without erroring.
//! - `validate_strict` is `unimplemented!()` until the
//!   production-mode parser lands (tracked under
//!   `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-otel-propagation-validator`).
//!
//! Until the strict mode ships, the gate dispatcher runs this kernel
//! in DEFERRED advisory mode per the gate-deferral pattern documented
//! in `crates/oya-dev-cli/src/commands/gate/mod.rs`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// One supplied gRPC client adapter source (`crates/oya-*-adapter-grpc-*`
/// or equivalent) that the runner reads off-disk and forwards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientAdapterSource {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryReport {
    pub adapters_checked: usize,
    pub adapters_with_propagation: usize,
    pub microservices_audited: usize,
    pub advisory_findings: Vec<TracePropagationFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracePropagationFinding {
    pub adapter_path: String,
    pub microservice: String,
    pub summary: String,
}

impl fmt::Display for TracePropagationFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {}",
            self.microservice, self.adapter_path, self.summary
        )
    }
}

/// Canonical token set the kernel detects to confirm propagation. A
/// client adapter that mentions any of these is treated as compliant
/// in advisory mode. The strict-mode (not yet implemented) parser will
/// require the W3C-canonical `traceparent` header injection on every
/// outbound call.
pub const PROPAGATION_TOKENS: &[&str] = &[
    "traceparent",
    "TraceContextPropagator",
    "TextMapPropagator",
    "inject_context",
    concat!("opentelemetry", "::global::get_text_map_propagator"),
    "oya_shared_tracing_client_kernel",
];

/// Advisory entrypoint. Returns the advisory report; does NOT error.
pub fn validate_advisory<I>(adapters: I) -> AdvisoryReport
where
    I: IntoIterator<Item = ClientAdapterSource>,
{
    let adapters: Vec<ClientAdapterSource> = adapters.into_iter().collect();
    let mut findings = Vec::new();
    let mut compliant = 0usize;
    let mut microservices = std::collections::BTreeSet::<String>::new();

    for adapter in &adapters {
        microservices.insert(adapter.microservice.clone());
        if has_any_propagation_token(&adapter.contents) {
            compliant += 1;
        } else {
            findings.push(TracePropagationFinding {
                adapter_path: adapter.path.clone(),
                microservice: adapter.microservice.clone(),
                summary: format!(
                    "no canonical propagation token found; expected one of {PROPAGATION_TOKENS:?}"
                ),
            });
        }
    }

    AdvisoryReport {
        adapters_checked: adapters.len(),
        adapters_with_propagation: compliant,
        microservices_audited: microservices.len(),
        advisory_findings: findings,
    }
}

/// Strict mode. Currently `unimplemented!()` — the dispatcher runs the
/// advisory mode while we author the strict-mode parser per the
/// placeholder-debt follow-up record.
pub fn validate_strict<I>(_adapters: I) -> !
where
    I: IntoIterator<Item = ClientAdapterSource>,
{
    unimplemented!(
        "oya-check-otel-trace-propagation: strict-mode validator not yet implemented; \
         tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-otel-propagation-validator"
    )
}

fn has_any_propagation_token(source: &str) -> bool {
    PROPAGATION_TOKENS
        .iter()
        .any(|token| source.contains(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisory_reports_compliant_when_token_present() {
        let adapter = ClientAdapterSource {
            path: "crates/oya-network-jobs-handoff-adapter-grpc-client/src/lib.rs".into(),
            microservice: "network".into(),
            contents: "let mut headers = BTreeMap::new();\noya_shared_tracing_client_kernel::inject(&mut headers);".into(),
        };
        let report = validate_advisory(vec![adapter]);
        assert_eq!(report.adapters_checked, 1);
        assert_eq!(report.adapters_with_propagation, 1);
        assert!(report.advisory_findings.is_empty());
    }

    #[test]
    fn advisory_reports_finding_when_no_token() {
        let adapter = ClientAdapterSource {
            path: "crates/oya-tasks-adapter-grpc-client/src/lib.rs".into(),
            microservice: "tasks".into(),
            contents: "// no propagation here\nfn call() {}".into(),
        };
        let report = validate_advisory(vec![adapter]);
        assert_eq!(report.adapters_checked, 1);
        assert_eq!(report.adapters_with_propagation, 0);
        assert_eq!(report.advisory_findings.len(), 1);
    }

    #[test]
    #[should_panic(expected = "strict-mode validator not yet implemented")]
    fn strict_mode_panics_until_authored() {
        validate_strict(Vec::<ClientAdapterSource>::new());
    }

    #[test]
    fn propagation_tokens_non_empty() {
        assert!(!PROPAGATION_TOKENS.is_empty());
    }
}
