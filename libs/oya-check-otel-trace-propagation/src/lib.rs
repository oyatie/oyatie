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
//! - `validate_strict` fails closed on supplied sources that do not show
//!   both a valid W3C `traceparent` value and an OTLP exporter path. This
//!   is the first runtime-proof slice; full Rust-syntax per-call parsing
//!   remains tracked under
//!   `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-otel-propagation-validator`.
//!
//! The gate dispatcher still runs this kernel in DEFERRED advisory mode per
//! the gate-deferral pattern documented in
//! `crates/oya-dev-cli/src/commands/gate/mod.rs` until the full parser covers
//! every inter-service call site.

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
pub struct StrictReport {
    pub adapters_checked: usize,
    pub adapters_with_valid_traceparent: usize,
    pub adapters_with_otlp_exporter_path: usize,
    pub microservices_audited: usize,
    pub strict_findings: Vec<TracePropagationFinding>,
}

impl StrictReport {
    pub fn is_success(&self) -> bool {
        self.adapters_checked > 0 && self.strict_findings.is_empty()
    }
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

/// Canonical tokens that prove the slice has an OTLP exporter path to a
/// collector/gateway, not just an in-process span placeholder.
pub const OTLP_EXPORTER_TOKENS: &[&str] = &[
    "OTEL_EXPORTER_OTLP_ENDPOINT",
    "OYA_OTEL_ENDPOINT",
    "otel-collector",
    "alloy.observability",
    ":4317",
    "/v1/traces",
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

/// Strict mode for the first collector/tracing slice.
///
/// This is intentionally narrower than the final ADR-0145 parser: it validates
/// supplied sources fail-closed for a real W3C `traceparent` value and an OTLP
/// exporter path to a collector. The later parser can extend this report shape
/// to reason over every outbound call site without reintroducing an
/// `unimplemented!()` green path.
pub fn validate_strict<I>(adapters: I) -> StrictReport
where
    I: IntoIterator<Item = ClientAdapterSource>,
{
    let adapters: Vec<ClientAdapterSource> = adapters.into_iter().collect();
    let mut findings = Vec::new();
    let mut with_traceparent = 0usize;
    let mut with_otlp = 0usize;
    let mut microservices = std::collections::BTreeSet::<String>::new();

    if adapters.is_empty() {
        findings.push(TracePropagationFinding {
            adapter_path: "<none>".into(),
            microservice: "<none>".into(),
            summary: "no adapter sources supplied; strict mode cannot pass by empty input".into(),
        });
    }

    for adapter in &adapters {
        microservices.insert(adapter.microservice.clone());

        let valid_traceparent = has_traceparent_header_with_valid_value(&adapter.contents);
        if valid_traceparent {
            with_traceparent += 1;
        } else {
            findings.push(TracePropagationFinding {
                adapter_path: adapter.path.clone(),
                microservice: adapter.microservice.clone(),
                summary: "no valid W3C traceparent value found for strict propagation proof".into(),
            });
        }

        let otlp_exporter_path = has_otlp_exporter_path(&adapter.contents);
        if otlp_exporter_path {
            with_otlp += 1;
        } else {
            findings.push(TracePropagationFinding {
                adapter_path: adapter.path.clone(),
                microservice: adapter.microservice.clone(),
                summary: format!(
                    "no OTLP exporter path found; expected collector/exporter token from {OTLP_EXPORTER_TOKENS:?}"
                ),
            });
        }
    }

    StrictReport {
        adapters_checked: adapters.len(),
        adapters_with_valid_traceparent: with_traceparent,
        adapters_with_otlp_exporter_path: with_otlp,
        microservices_audited: microservices.len(),
        strict_findings: findings,
    }
}

fn has_any_propagation_token(source: &str) -> bool {
    PROPAGATION_TOKENS
        .iter()
        .any(|token| source.contains(token))
}

fn has_otlp_exporter_path(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    let mentions_otlp_or_otel = lower.contains("otlp") || lower.contains("otel");
    mentions_otlp_or_otel
        && OTLP_EXPORTER_TOKENS
            .iter()
            .any(|token| source.contains(token) || lower.contains(&token.to_ascii_lowercase()))
}

fn has_traceparent_header_with_valid_value(source: &str) -> bool {
    source.contains("traceparent")
        && source
            .split(|c: char| !(c.is_ascii_hexdigit() || c == '-'))
            .any(is_valid_traceparent_candidate)
}

fn is_valid_traceparent_candidate(candidate: &str) -> bool {
    let mut parts = candidate.split('-');
    let Some(version) = parts.next() else {
        return false;
    };
    let Some(trace_id) = parts.next() else {
        return false;
    };
    let Some(parent_id) = parts.next() else {
        return false;
    };
    let Some(flags) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }

    version == "00"
        && trace_id.len() == 32
        && parent_id.len() == 16
        && flags.len() == 2
        && is_lowercase_hex(trace_id)
        && is_lowercase_hex(parent_id)
        && is_lowercase_hex(flags)
        && !is_all_zero(trace_id)
        && !is_all_zero(parent_id)
        && u8::from_str_radix(flags, 16).is_ok()
}

fn is_lowercase_hex(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_all_zero(value: &str) -> bool {
    value.bytes().all(|byte| byte == b'0')
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
    fn strict_mode_rejects_empty_input_instead_of_green_by_unimplemented() {
        let report = validate_strict(Vec::<ClientAdapterSource>::new());

        assert!(!report.is_success());
        assert_eq!(report.adapters_checked, 0);
        assert_eq!(report.strict_findings.len(), 1);
        assert!(
            report.strict_findings[0]
                .summary
                .contains("no adapter sources")
        );
    }

    #[test]
    fn strict_mode_accepts_messenger_slice_with_valid_traceparent_and_otlp_path() {
        let adapter = ClientAdapterSource {
            path: "oya/messenger/crates/oya-messenger-message-stream-rest/src/lib.rs".into(),
            microservice: "messenger".into(),
            contents: r#"
                pub const MESSENGER_DEFAULT_OTLP_ENDPOINT: &str = "http://otel-collector.observability.svc.cluster.local:4317";
                pub const MESSENGER_OTLP_EXPORTER_ENV: &str = "OYA_OTEL_ENDPOINT";
                pub const MESSENGER_TRACEPARENT_HEADER: &str = "traceparent";
                pub const MESSENGER_TRACEPARENT_CANARY: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
            "#
            .into(),
        };

        let report = validate_strict(vec![adapter]);

        assert!(
            report.is_success(),
            "unexpected findings: {:?}",
            report.strict_findings
        );
        assert_eq!(report.adapters_checked, 1);
        assert_eq!(report.adapters_with_valid_traceparent, 1);
        assert_eq!(report.adapters_with_otlp_exporter_path, 1);
        assert_eq!(report.microservices_audited, 1);
    }

    #[test]
    fn strict_mode_rejects_invalid_traceparent_even_when_otlp_path_exists() {
        let adapter = ClientAdapterSource {
            path: "oya/messenger/crates/oya-messenger-message-stream-rest/src/lib.rs".into(),
            microservice: "messenger".into(),
            contents: r#"
                const ENDPOINT: &str = "http://otel-collector.observability.svc.cluster.local:4317";
                const HEADER: &str = "traceparent";
                const TRACEPARENT: &str = "00-00000000000000000000000000000000-0000000000000000-00";
            "#
            .into(),
        };

        let report = validate_strict(vec![adapter]);

        assert!(!report.is_success());
        assert_eq!(report.adapters_with_valid_traceparent, 0);
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("valid W3C traceparent"))
        );
    }

    #[test]
    fn strict_mode_rejects_missing_otlp_exporter_path() {
        let adapter = ClientAdapterSource {
            path: "oya/messenger/crates/oya-messenger-message-stream-rest/src/lib.rs".into(),
            microservice: "messenger".into(),
            contents: r#"
                const HEADER: &str = "traceparent";
                const TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
            "#
            .into(),
        };

        let report = validate_strict(vec![adapter]);

        assert!(!report.is_success());
        assert_eq!(report.adapters_with_valid_traceparent, 1);
        assert_eq!(report.adapters_with_otlp_exporter_path, 0);
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("OTLP exporter path"))
        );
    }

    #[test]
    fn strict_mode_rejects_valid_looking_trace_id_without_traceparent_header() {
        let adapter = ClientAdapterSource {
            path: "oya/messenger/crates/oya-messenger-message-stream-rest/src/lib.rs".into(),
            microservice: "messenger".into(),
            contents: r#"
                const ENDPOINT: &str = "http://otel-collector.observability.svc.cluster.local:4317";
                const RANDOM_CANARY: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
            "#
            .into(),
        };

        let report = validate_strict(vec![adapter]);

        assert!(!report.is_success());
        assert_eq!(report.adapters_with_valid_traceparent, 0);
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("valid W3C traceparent"))
        );
    }

    #[test]
    fn propagation_tokens_non_empty() {
        assert!(!PROPAGATION_TOKENS.is_empty());
    }
}
