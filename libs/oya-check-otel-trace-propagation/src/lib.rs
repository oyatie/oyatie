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
//! - `validate_strict` fails closed unless it receives structured runtime
//!   evidence emitted from a request path: a W3C `traceparent`, OTLP exporter
//!   binding, and the static matched route template used as the low-cardinality
//!   label.
//!
//! The gate dispatcher still runs this kernel in DEFERRED advisory mode per
//! the gate-deferral pattern documented in
//! `crates/oya-dev-cli/src/commands/gate/mod.rs` until the full parser covers
//! every inter-service call site.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
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
    /// Compatibility count for existing gate output: in strict mode this is the
    /// number of evidence records supplied, not source files scanned.
    pub adapters_checked: usize,
    pub adapters_with_valid_traceparent: usize,
    pub adapters_with_otlp_exporter_path: usize,
    pub runtime_evidence_checked: usize,
    pub microservices_audited: usize,
    pub strict_findings: Vec<TracePropagationFinding>,
}

impl StrictReport {
    pub fn is_success(&self) -> bool {
        self.adapters_checked > 0
            && self.runtime_evidence_checked == self.adapters_checked
            && self.strict_findings.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTraceEvidence {
    pub evidence_path: String,
    pub microservice: String,
    pub method: String,
    pub request_path: String,
    pub route_template: String,
    pub path_captures: BTreeMap<String, String>,
    pub traceparent_header: String,
    pub traceparent: String,
    pub otlp_exporter_env: String,
    pub otlp_endpoint: String,
    pub otlp_protocol: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StrictTraceEvidence {
    Runtime(RuntimeTraceEvidence),
    SourceText(ClientAdapterSource),
}

impl From<RuntimeTraceEvidence> for StrictTraceEvidence {
    fn from(value: RuntimeTraceEvidence) -> Self {
        Self::Runtime(value)
    }
}

impl From<ClientAdapterSource> for StrictTraceEvidence {
    fn from(value: ClientAdapterSource) -> Self {
        Self::SourceText(value)
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
/// in advisory mode. Strict mode rejects source text and requires
/// structured runtime evidence.
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
    let mut microservices = BTreeSet::<String>::new();

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
/// Source text is intentionally rejected even when it contains plausible
/// `traceparent` and OTLP constants. Callers must supply a runtime evidence
/// record emitted from a request path so the validator can enforce route-label
/// cardinality and exporter binding as data.
pub fn validate_strict<I, E>(evidence: I) -> StrictReport
where
    I: IntoIterator<Item = E>,
    E: Into<StrictTraceEvidence>,
{
    let evidence: Vec<StrictTraceEvidence> = evidence.into_iter().map(Into::into).collect();
    let mut findings = Vec::new();
    let mut with_traceparent = 0usize;
    let mut with_otlp = 0usize;
    let mut runtime_evidence = 0usize;
    let mut microservices = BTreeSet::<String>::new();

    if evidence.is_empty() {
        findings.push(TracePropagationFinding {
            adapter_path: "<none>".into(),
            microservice: "<none>".into(),
            summary: "no runtime trace evidence supplied; strict mode cannot pass by empty input"
                .into(),
        });
    }

    for item in &evidence {
        match item {
            StrictTraceEvidence::SourceText(source) => {
                microservices.insert(source.microservice.clone());
                findings.push(TracePropagationFinding {
                    adapter_path: source.path.clone(),
                    microservice: source.microservice.clone(),
                    summary: "strict mode requires runtime trace evidence; source text/static constants are not accepted"
                        .into(),
                });
            }
            StrictTraceEvidence::Runtime(record) => {
                runtime_evidence += 1;
                microservices.insert(record.microservice.clone());

                if has_valid_traceparent_binding(record) {
                    with_traceparent += 1;
                } else {
                    findings.push(TracePropagationFinding {
                        adapter_path: record.evidence_path.clone(),
                        microservice: record.microservice.clone(),
                        summary:
                            "runtime evidence did not carry a valid W3C traceparent header/value"
                                .into(),
                    });
                }

                if has_valid_otlp_exporter_binding(record) {
                    with_otlp += 1;
                } else {
                    findings.push(TracePropagationFinding {
                        adapter_path: record.evidence_path.clone(),
                        microservice: record.microservice.clone(),
                        summary: format!(
                            "runtime evidence did not carry an OTLP exporter binding; expected env/endpoint/protocol from {OTLP_EXPORTER_TOKENS:?}"
                        ),
                    });
                }

                if !has_low_cardinality_route_template(record) {
                    findings.push(TracePropagationFinding {
                        adapter_path: record.evidence_path.clone(),
                        microservice: record.microservice.clone(),
                        summary:
                            "runtime evidence route template leaks raw path/capture data or is not a static route label"
                                .into(),
                    });
                }
            }
        }
    }

    StrictReport {
        adapters_checked: evidence.len(),
        adapters_with_valid_traceparent: with_traceparent,
        adapters_with_otlp_exporter_path: with_otlp,
        runtime_evidence_checked: runtime_evidence,
        microservices_audited: microservices.len(),
        strict_findings: findings,
    }
}

fn has_any_propagation_token(source: &str) -> bool {
    PROPAGATION_TOKENS
        .iter()
        .any(|token| source.contains(token))
}

fn has_valid_traceparent_binding(record: &RuntimeTraceEvidence) -> bool {
    record.traceparent_header == "traceparent"
        && is_valid_traceparent_candidate(&record.traceparent)
}

fn has_valid_otlp_exporter_binding(record: &RuntimeTraceEvidence) -> bool {
    let env = record.otlp_exporter_env.as_str();
    let protocol = record.otlp_protocol.to_ascii_lowercase();

    matches!(env, "OTEL_EXPORTER_OTLP_ENDPOINT" | "OYA_OTEL_ENDPOINT")
        && is_allowed_otlp_endpoint_for_protocol(&record.otlp_endpoint, &protocol)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OtlpEndpointParts<'a> {
    host: String,
    port: Option<&'a str>,
    path: &'a str,
}

fn is_allowed_otlp_endpoint_for_protocol(endpoint: &str, protocol: &str) -> bool {
    let Some(parts) = parse_otlp_http_endpoint(endpoint) else {
        return false;
    };
    if !is_allowed_otlp_collector_host(&parts.host) {
        return false;
    }

    match protocol {
        "otlp/grpc" => parts.port == Some("4317") && matches!(parts.path, "" | "/"),
        "otlp/http" | "otlp/http/protobuf" => {
            let port_ok = match parts.port {
                None | Some("4317") | Some("4318") => true,
                Some(_) => false,
            };
            port_ok && parts.path == "/v1/traces"
        }
        _ => false,
    }
}

fn parse_otlp_http_endpoint(endpoint: &str) -> Option<OtlpEndpointParts<'_>> {
    let endpoint = endpoint.trim();
    let endpoint_lower = endpoint.to_ascii_lowercase();
    let rest = if endpoint_lower.starts_with("http://") {
        &endpoint["http://".len()..]
    } else if endpoint_lower.starts_with("https://") {
        &endpoint["https://".len()..]
    } else {
        return None;
    };

    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty()
        || authority.contains('@')
        || authority.contains('[')
        || authority.contains(']')
    {
        return None;
    }

    let suffix = &rest[authority_end..];
    if suffix.starts_with('?') || suffix.starts_with('#') {
        return None;
    }
    let path_end = suffix.find(['?', '#']).unwrap_or(suffix.len());
    if path_end != suffix.len() {
        return None;
    }
    let path = &suffix[..path_end];

    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port))
            if !host.is_empty()
                && !port.is_empty()
                && port.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            (host, Some(port))
        }
        Some(_) => return None,
        None => (authority, None),
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() || host.contains(':') {
        return None;
    }

    Some(OtlpEndpointParts { host, port, path })
}

fn is_allowed_otlp_collector_host(host: &str) -> bool {
    matches!(
        host,
        "otel-collector"
            | "otel-collector.observability"
            | "otel-collector.observability.svc"
            | "otel-collector.observability.svc.cluster.local"
            | "alloy.observability"
            | "alloy.observability.svc"
            | "alloy.observability.svc.cluster.local"
    )
}

fn has_low_cardinality_route_template(record: &RuntimeTraceEvidence) -> bool {
    let template = record.route_template.as_str();
    let request_path = record.request_path.as_str();
    if template.is_empty()
        || !template.starts_with('/')
        || request_path.is_empty()
        || !request_path.starts_with('/')
        || template.contains('?')
        || request_path.contains('?')
        || template.contains("://")
        || request_path.contains("://")
        || template.chars().any(char::is_control)
        || request_path.chars().any(char::is_control)
    {
        return false;
    }

    let template_segments: Vec<&str> = template.split('/').collect();
    let request_segments: Vec<&str> = request_path.split('/').collect();
    if template_segments.len() != request_segments.len() {
        return false;
    }

    let mut used_captures = BTreeSet::<String>::new();
    for (template_segment, request_segment) in template_segments.iter().zip(request_segments.iter())
    {
        let placeholder = match route_placeholder_name(template_segment) {
            Ok(placeholder) => placeholder,
            Err(()) => return false,
        };

        if let Some(name) = placeholder {
            let Some(value) = record.path_captures.get(name) else {
                return false;
            };
            if value.is_empty() || value != request_segment {
                return false;
            }
            used_captures.insert(name.to_string());
        } else {
            if template_segment != request_segment {
                return false;
            }
            if record
                .path_captures
                .values()
                .any(|value| !value.is_empty() && value == template_segment)
            {
                return false;
            }
        }
    }

    used_captures.len() == record.path_captures.len()
}

fn route_placeholder_name(segment: &str) -> Result<Option<&str>, ()> {
    let has_open = segment.contains('{');
    let has_close = segment.contains('}');
    if !has_open && !has_close {
        return Ok(None);
    }

    if segment.starts_with('{') && segment.ends_with('}') && segment.len() > 2 {
        let name = &segment[1..segment.len() - 1];
        if name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Ok(Some(name));
        }
    }

    Err(())
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

    const VALID_TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
    const OTLP_ENDPOINT: &str = "http://otel-collector.observability.svc.cluster.local:4317";

    fn runtime_evidence() -> RuntimeTraceEvidence {
        RuntimeTraceEvidence {
            evidence_path: "runtime://messenger/post-message".into(),
            microservice: "messenger".into(),
            method: "POST".into(),
            request_path: "/channels/c/messages".into(),
            route_template: "/channels/{channel_id}/messages".into(),
            path_captures: BTreeMap::from([("channel_id".into(), "c".into())]),
            traceparent_header: "traceparent".into(),
            traceparent: VALID_TRACEPARENT.into(),
            otlp_exporter_env: "OYA_OTEL_ENDPOINT".into(),
            otlp_endpoint: OTLP_ENDPOINT.into(),
            otlp_protocol: "otlp/grpc".into(),
        }
    }

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
        let report = validate_strict(Vec::<RuntimeTraceEvidence>::new());

        assert!(!report.is_success());
        assert_eq!(report.adapters_checked, 0);
        assert_eq!(report.runtime_evidence_checked, 0);
        assert_eq!(report.strict_findings.len(), 1);
        assert!(
            report.strict_findings[0]
                .summary
                .contains("no runtime trace evidence")
        );
    }

    #[test]
    fn strict_mode_rejects_static_source_text_even_with_valid_looking_tokens() {
        let adapter = ClientAdapterSource {
            path: "comms/facade/messenger-stream-rest/src/lib.rs".into(),
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

        assert!(!report.is_success());
        assert_eq!(report.adapters_checked, 1);
        assert_eq!(report.runtime_evidence_checked, 0);
        assert_eq!(report.adapters_with_valid_traceparent, 0);
        assert_eq!(report.adapters_with_otlp_exporter_path, 0);
        assert!(report.strict_findings.iter().any(|finding| {
            finding.summary.contains("runtime") && finding.summary.contains("source text")
        }));
    }

    #[test]
    fn strict_mode_accepts_runtime_request_evidence_with_traceparent_and_otlp_path() {
        let report = validate_strict(vec![runtime_evidence()]);

        assert!(
            report.is_success(),
            "unexpected findings: {:?}",
            report.strict_findings
        );
        assert_eq!(report.adapters_checked, 1);
        assert_eq!(report.runtime_evidence_checked, 1);
        assert_eq!(report.adapters_with_valid_traceparent, 1);
        assert_eq!(report.adapters_with_otlp_exporter_path, 1);
        assert_eq!(report.microservices_audited, 1);
    }

    #[test]
    fn strict_mode_rejects_invalid_traceparent_even_when_otlp_path_exists() {
        let mut evidence = runtime_evidence();
        evidence.traceparent = "00-00000000000000000000000000000000-0000000000000000-00".into();

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert_eq!(report.adapters_with_valid_traceparent, 0);
        assert_eq!(report.adapters_with_otlp_exporter_path, 1);
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("valid W3C traceparent"))
        );
    }

    #[test]
    fn strict_mode_rejects_missing_otlp_exporter_path() {
        let mut evidence = runtime_evidence();
        evidence.otlp_endpoint = "http://example.invalid:8080".into();

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert_eq!(report.adapters_with_valid_traceparent, 1);
        assert_eq!(report.adapters_with_otlp_exporter_path, 0);
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("OTLP exporter binding"))
        );
    }

    #[test]
    fn strict_mode_rejects_collector_token_outside_endpoint_host() {
        let mut evidence = runtime_evidence();
        evidence.otlp_endpoint =
            "https://telemetry.example.invalid/otel-collector/v1/traces".into();
        evidence.otlp_protocol = "otlp/http/protobuf".into();

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert_eq!(report.adapters_with_valid_traceparent, 1);
        assert_eq!(report.adapters_with_otlp_exporter_path, 0);
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("OTLP exporter binding"))
        );
    }

    #[test]
    fn strict_mode_accepts_allowed_alloy_http_trace_endpoint() {
        let mut evidence = runtime_evidence();
        evidence.otlp_endpoint = "https://alloy.observability.svc.cluster.local/v1/traces".into();
        evidence.otlp_protocol = "otlp/http/protobuf".into();

        let report = validate_strict(vec![evidence]);

        assert!(
            report.is_success(),
            "unexpected findings: {:?}",
            report.strict_findings
        );
        assert_eq!(report.adapters_with_valid_traceparent, 1);
        assert_eq!(report.adapters_with_otlp_exporter_path, 1);
    }

    #[test]
    fn strict_mode_rejects_raw_path_route_template_cardinality_leak() {
        let mut evidence = runtime_evidence();
        evidence.route_template = "/channels/c/messages".into();

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("route template"))
        );
    }

    #[test]
    fn strict_mode_rejects_unknown_otlp_protocol_even_with_collector_endpoint() {
        let mut evidence = runtime_evidence();
        evidence.otlp_protocol = "otlp/custom".into();

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert_eq!(report.adapters_with_valid_traceparent, 1);
        assert_eq!(report.adapters_with_otlp_exporter_path, 0);
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("OTLP exporter binding"))
        );
    }

    #[test]
    fn strict_mode_rejects_template_placeholder_without_bound_capture() {
        let mut evidence = runtime_evidence();
        evidence.path_captures.clear();

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("route template"))
        );
    }

    #[test]
    fn strict_mode_rejects_capture_value_that_did_not_come_from_request_path() {
        let mut evidence = runtime_evidence();
        evidence
            .path_captures
            .insert("channel_id".into(), "other-channel".into());

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("route template"))
        );
    }

    #[test]
    fn strict_mode_rejects_unbound_extra_capture_values() {
        let mut evidence = runtime_evidence();
        evidence
            .path_captures
            .insert("message_id".into(), "m".into());

        let report = validate_strict(vec![evidence]);

        assert!(!report.is_success());
        assert!(
            report
                .strict_findings
                .iter()
                .any(|finding| finding.summary.contains("route template"))
        );
    }

    #[test]
    fn strict_mode_rejects_valid_looking_trace_id_without_traceparent_header() {
        let mut evidence = runtime_evidence();
        evidence.traceparent_header = "x-correlation-id".into();

        let report = validate_strict(vec![evidence]);

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
