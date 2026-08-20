//! Canonical cloud-ci run observability packet validation (GH-1003).
//!
//! This lives in the legacy `ci/facade` firewall crate only because that is the
//! current required-context enforcement path; the stable contracts are the top-level
//! `specs/cloud-ci-run-observability-{packet,status}.schema.json` specs and the eventual
//! de-branded destination is an `observability/core` run-observability kernel.

use std::collections::BTreeSet;

use serde_json::Value;

pub const PACKET_SCHEMA_VERSION: &str = "cloud-ci-run-observability-packet/v1";
pub const PACKET_SCHEMA_PATH: &str = "specs/cloud-ci-run-observability-packet.schema.json";
pub const STATUS_SCHEMA_VERSION: &str = "cloud-ci-run-observability-status/v1";
pub const STATUS_SCHEMA_PATH: &str = "specs/cloud-ci-run-observability-status.schema.json";
pub const STATUS_VALUES: [&str; 6] = [
    "queued",
    "running",
    "passed",
    "failed",
    "cancelled",
    "timed_out",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketVerdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PacketFinding {
    pub code: String,
    pub key: String,
    pub remediation: String,
}

impl PacketFinding {
    fn new(code: &str, key: impl Into<String>, remediation: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            remediation: remediation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketReport {
    pub verdict: PacketVerdict,
    pub findings: BTreeSet<PacketFinding>,
}

impl PacketReport {
    fn from_findings(findings: BTreeSet<PacketFinding>) -> Self {
        let verdict = if findings.is_empty() {
            PacketVerdict::Green
        } else {
            PacketVerdict::Red
        };
        Self { verdict, findings }
    }
}

pub fn validate_packet(packet: &Value) -> PacketReport {
    let mut findings = BTreeSet::new();
    let Some(obj) = packet.as_object() else {
        findings.insert(PacketFinding::new(
            "packet_not_object",
            "$",
            "Emit the cloud-ci run observability packet as a JSON object.",
        ));
        return PacketReport::from_findings(findings);
    };

    require_exact_str(
        obj.get("schema_version"),
        PACKET_SCHEMA_VERSION,
        "schema_version",
        "packet_schema_version_invalid",
        &mut findings,
    );
    let run = obj.get("run");
    let run_provider = str_at(run.and_then(|run| run.get("provider")));
    let run_id = str_at(run.and_then(|run| run.get("run_id")));
    let attempt = run
        .and_then(|run| run.get("attempt"))
        .and_then(Value::as_u64);
    let conclusion = str_at(run.and_then(|run| run.get("conclusion")));
    let failure_required = matches!(conclusion, Some("failed" | "infra_red"));

    require_non_empty_str(
        obj.get("packet_id"),
        "packet_id",
        "packet_id_missing",
        &mut findings,
    );
    if let Some(packet_id) = str_at(obj.get("packet_id")) {
        if !is_canonical_packet_id_shape(packet_id) {
            findings.insert(PacketFinding::new(
                "packet_id_unstable",
                "packet_id",
                "Use a stable packet id of the form cloud-ci-run-packet:<provider>:<run-id>:attempt-<n>.",
            ));
        } else if let (Some(provider), Some(run_id), Some(attempt)) =
            (run_provider, run_id, attempt)
        {
            let expected = canonical_packet_id(provider, run_id, attempt);
            if packet_id != expected {
                findings.insert(PacketFinding::new(
                    "packet_id_run_mismatch",
                    "packet_id",
                    "Tie packet_id to the packet run fields exactly: cloud-ci-run-packet:<provider>:<run_id>:attempt-<attempt>.",
                ));
            }
        }
    }

    validate_producer(obj.get("producer"), &mut findings);
    validate_run(obj.get("run"), &mut findings);
    validate_context_binding(obj.get("producer"), obj.get("run"), &mut findings);
    validate_subject(obj.get("subject"), &mut findings);

    let artifact_ids = validate_artifacts(obj.get("artifacts"), run_id, &mut findings);
    let diagnostic_ids =
        validate_diagnostics(obj.get("diagnostics"), failure_required, &mut findings);
    validate_retention(obj.get("retention"), &mut findings);
    validate_transitions(
        obj.get("transitions"),
        conclusion,
        run_id,
        &artifact_ids,
        &diagnostic_ids,
        &mut findings,
    );
    validate_diagnosability(
        obj.get("diagnosability"),
        failure_required,
        &artifact_ids,
        &diagnostic_ids,
        &mut findings,
    );

    PacketReport::from_findings(findings)
}

pub fn validate_status(status: &Value) -> PacketReport {
    let mut findings = BTreeSet::new();
    let Some(obj) = status.as_object() else {
        findings.insert(PacketFinding::new(
            "status_not_object",
            "$",
            "Emit the cloud-ci run observability status API payload as a JSON object.",
        ));
        return PacketReport::from_findings(findings);
    };

    require_exact_str(
        obj.get("schema_version"),
        STATUS_SCHEMA_VERSION,
        "schema_version",
        "status_schema_version_invalid",
        &mut findings,
    );

    let run = obj.get("run");
    let run_provider = str_at(run.and_then(|run| run.get("provider")));
    let run_id = str_at(run.and_then(|run| run.get("run_id")));
    let attempt = run
        .and_then(|run| run.get("attempt"))
        .and_then(Value::as_u64);
    let status_value = str_at(obj.get("status"));
    let failure_required = matches!(status_value, Some("failed" | "timed_out"));

    validate_status_id(
        obj.get("status_id"),
        run_provider,
        run_id,
        attempt,
        &mut findings,
    );
    validate_status_producer(obj.get("producer"), &mut findings);
    validate_status_run(run, &mut findings);
    validate_context_binding(obj.get("producer"), run, &mut findings);
    validate_subject(obj.get("subject"), &mut findings);
    validate_status_value(obj.get("status"), &mut findings);
    validate_status_phase(obj.get("phase"), status_value, &mut findings);
    validate_gate_summary(obj.get("gate_summary"), status_value, &mut findings);
    let artifact_refs = validate_status_refs(
        obj.get("artifact_refs"),
        "artifact_refs",
        StatusRefKind::Artifact,
        &mut findings,
    );
    let diagnostic_refs = validate_status_refs(
        obj.get("diagnostic_refs"),
        "diagnostic_refs",
        StatusRefKind::Diagnostic,
        &mut findings,
    );
    validate_status_correlation(obj.get("correlation"), run_id, &mut findings);
    validate_status_retention(obj.get("retention"), &mut findings);
    validate_status_diagnosability(
        obj.get("diagnosability"),
        failure_required,
        artifact_refs,
        diagnostic_refs,
        &mut findings,
    );

    PacketReport::from_findings(findings)
}

fn validate_status_id(
    value: Option<&Value>,
    provider: Option<&str>,
    run_id: Option<&str>,
    attempt: Option<u64>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    require_non_empty_str(value, "status_id", "status_id_missing", findings);
    let Some(status_id) = str_at(value) else {
        return;
    };
    if !is_canonical_status_id_shape(status_id) {
        findings.insert(PacketFinding::new(
            "status_id_unstable",
            "status_id",
            "Use a stable status id of the form cloud-ci-run-status:<provider>:<run-id>:attempt-<n>.",
        ));
        return;
    }
    if let (Some(provider), Some(run_id), Some(attempt)) = (provider, run_id, attempt) {
        let expected = canonical_status_id(provider, run_id, attempt);
        if status_id != expected {
            findings.insert(PacketFinding::new(
                "status_id_run_mismatch",
                "status_id",
                "Tie status_id to the status run fields exactly: cloud-ci-run-status:<provider>:<run_id>:attempt-<attempt>.",
            ));
        }
    }
}

fn canonical_status_id(provider: &str, run_id: &str, attempt: u64) -> String {
    format!("cloud-ci-run-status:{provider}:{run_id}:attempt-{attempt}")
}

fn is_canonical_status_id_shape(status_id: &str) -> bool {
    let mut parts = status_id.split(':');
    matches!(parts.next(), Some("cloud-ci-run-status"))
        && parts
            .next()
            .is_some_and(|provider| !provider.is_empty() && provider.bytes().all(is_stable_id_byte))
        && parts
            .next()
            .is_some_and(|run_id| !run_id.is_empty() && run_id.bytes().all(is_stable_id_byte))
        && parts.next().is_some_and(|attempt| {
            attempt
                .strip_prefix("attempt-")
                .is_some_and(|value| !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()))
        })
        && parts.next().is_none()
}

fn validate_status_producer(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, "producer", findings) else {
        return;
    };
    require_allowed_str(
        obj.get("required_context"),
        &["cloud-ci-required", "oya-ci-required"],
        "producer.required_context",
        "producer_required_context_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("merge_authority_context"),
        &["cloud-ci-required", "oya-ci-required"],
        "producer.merge_authority_context",
        "producer_merge_authority_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("control_plane"),
        &["trusted-cloud-ci-controller", "github-actions-bridge"],
        "producer.control_plane",
        "producer_control_plane_invalid",
        findings,
    );
    require_non_empty_str(
        obj.get("status_api"),
        "producer.status_api",
        "status_api_missing",
        findings,
    );
}

fn validate_status_run(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, "run", findings) else {
        return;
    };
    require_non_empty_str(obj.get("run_id"), "run.run_id", "run_id_missing", findings);
    require_positive_u64(
        obj.get("attempt"),
        "run.attempt",
        "run_attempt_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("provider"),
        &["github-actions", "owned-cloud-ci"],
        "run.provider",
        "run_provider_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("status_context"),
        &["cloud-ci-required", "oya-ci-required"],
        "run.status_context",
        "run_status_context_invalid",
        findings,
    );
    require_non_empty_str(
        obj.get("correlation_id"),
        "run.correlation_id",
        "run_correlation_id_missing",
        findings,
    );
    require_non_empty_str(
        obj.get("started_at"),
        "run.started_at",
        "run_started_at_missing",
        findings,
    );
    require_non_empty_str(
        obj.get("updated_at"),
        "run.updated_at",
        "run_updated_at_missing",
        findings,
    );
}

fn validate_status_value(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    match str_at(value) {
        Some(text) if STATUS_VALUES.contains(&text) => {}
        Some(text) => {
            findings.insert(PacketFinding::new(
                "status_value_invalid",
                "status",
                format!(
                    "invalid cloud-ci run observability status: {text}; expected {}",
                    STATUS_VALUES.join("|")
                ),
            ));
        }
        None => {
            findings.insert(PacketFinding::new(
                "status_value_missing",
                "status",
                format!(
                    "Emit one of the canonical status values: {}.",
                    STATUS_VALUES.join("|")
                ),
            ));
        }
    }
}

fn validate_status_phase(
    value: Option<&Value>,
    status_value: Option<&str>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(obj) = object_at(value, "phase", findings) else {
        return;
    };
    require_non_empty_str(
        obj.get("phase_id"),
        "phase.phase_id",
        "phase_id_missing",
        findings,
    );
    require_non_empty_str(
        obj.get("name"),
        "phase.name",
        "phase_name_missing",
        findings,
    );
    require_allowed_str(
        obj.get("state"),
        &STATUS_VALUES,
        "phase.state",
        "phase_state_invalid",
        findings,
    );
    require_non_empty_str(
        obj.get("started_at"),
        "phase.started_at",
        "phase_started_at_missing",
        findings,
    );
    require_non_empty_str(
        obj.get("updated_at"),
        "phase.updated_at",
        "phase_updated_at_missing",
        findings,
    );
    if let (Some(status), Some(phase_state)) = (status_value, str_at(obj.get("state")))
        && matches!(status, "passed" | "failed" | "cancelled" | "timed_out")
        && phase_state != status
    {
        findings.insert(PacketFinding::new(
            "terminal_phase_status_mismatch",
            "phase.state",
            "A terminal run status must carry the same terminal state on the current phase projection.",
        ));
    }
}

fn validate_gate_summary(
    value: Option<&Value>,
    status_value: Option<&str>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(obj) = object_at(value, "gate_summary", findings) else {
        return;
    };
    let mut total_parts = 0_u64;
    for key in [
        "queued",
        "running",
        "passed",
        "failed",
        "cancelled",
        "timed_out",
    ] {
        require_u64(
            obj.get(key),
            format!("gate_summary.{key}"),
            "gate_summary_count_missing",
            findings,
        );
        total_parts += obj.get(key).and_then(Value::as_u64).unwrap_or(0);
    }
    require_u64(
        obj.get("total"),
        "gate_summary.total",
        "gate_summary_total_missing",
        findings,
    );
    if let Some(total) = obj.get("total").and_then(Value::as_u64)
        && total != total_parts
    {
        findings.insert(PacketFinding::new(
            "gate_summary_total_mismatch",
            "gate_summary.total",
            "Make gate_summary.total equal the sum of queued/running/passed/failed/cancelled/timed_out.",
        ));
    }
    if matches!(status_value, Some("failed" | "timed_out")) {
        require_non_empty_str(
            obj.get("primary_failed_gate_id"),
            "gate_summary.primary_failed_gate_id",
            "primary_failed_gate_missing",
            findings,
        );
        require_allowed_str(
            obj.get("failure_taxonomy"),
            &[
                "code_regression",
                "policy_violation",
                "infra_red",
                "operator_waiver_required",
                "cancelled",
                "flake_suspected",
                "timeout",
            ],
            "gate_summary.failure_taxonomy",
            "failure_taxonomy_invalid",
            findings,
        );
    }
}

#[derive(Debug, Clone, Copy)]
enum StatusRefKind {
    Artifact,
    Diagnostic,
}

fn validate_status_refs(
    value: Option<&Value>,
    key: &str,
    kind: StatusRefKind,
    findings: &mut BTreeSet<PacketFinding>,
) -> usize {
    let Some(items) = array_at(value, key, findings) else {
        return 0;
    };
    let mut valid_count = 0_usize;
    for (index, item) in items.iter().enumerate() {
        let Some(text) = str_at(Some(item)).filter(|text| !text.is_empty()) else {
            findings.insert(PacketFinding::new(
                "status_ref_invalid",
                format!("{key}[{index}]"),
                "Emit every status reference as a non-empty stable typed artifact or diagnostic id.",
            ));
            continue;
        };
        match kind {
            StatusRefKind::Artifact => {
                if text.contains("/actions/runs/") {
                    findings.insert(PacketFinding::new(
                        "status_artifact_ref_raw_actions_log",
                        format!("{key}[{index}]"),
                        "Reference typed gate/status artifacts; do not use raw GitHub Actions run logs as status API refs.",
                    ));
                }
                if is_status_artifact_ref(text) {
                    valid_count += 1;
                } else {
                    findings.insert(PacketFinding::new(
                        "status_artifact_ref_invalid",
                        format!("{key}[{index}]"),
                        "Use canonical status artifact refs: artifact:<gate-report|step-report|redacted-diagnostics|status-packet>:<stable-key>.",
                    ));
                }
            }
            StatusRefKind::Diagnostic => {
                if is_status_diagnostic_ref(text) {
                    valid_count += 1;
                } else {
                    findings.insert(PacketFinding::new(
                        "status_diagnostic_ref_invalid",
                        format!("{key}[{index}]"),
                        "Use canonical status diagnostic refs: diag:<gate_id>:<stable-diagnostic-key>.",
                    ));
                }
            }
        }
    }
    valid_count
}

fn is_status_artifact_ref(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("artifact:") else {
        return false;
    };
    let Some((kind, tail)) = rest.split_once(':') else {
        return false;
    };
    matches!(
        kind,
        "gate-report" | "step-report" | "redacted-diagnostics" | "status-packet"
    ) && !tail.is_empty()
        && tail.bytes().all(is_status_artifact_ref_byte)
}

fn is_status_diagnostic_ref(value: &str) -> bool {
    let mut parts = value.split(':');
    matches!(parts.next(), Some("diag"))
        && parts
            .next()
            .is_some_and(|gate_id| !gate_id.is_empty() && gate_id.bytes().all(is_stable_id_byte))
        && parts.next().is_some_and(|diagnostic_key| {
            !diagnostic_key.is_empty() && diagnostic_key.bytes().all(is_stable_id_byte)
        })
        && parts.next().is_none()
}

fn is_status_artifact_ref_byte(byte: u8) -> bool {
    is_stable_id_byte(byte) || byte == b':'
}

fn validate_status_correlation(
    value: Option<&Value>,
    run_id: Option<&str>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(obj) = object_at(value, "correlation", findings) else {
        return;
    };
    require_non_empty_str(
        obj.get("packet_id"),
        "correlation.packet_id",
        "correlation_packet_id_missing",
        findings,
    );
    if let (Some(run_id), Some(packet_id)) = (run_id, str_at(obj.get("packet_id")))
        && !packet_id.contains(run_id)
    {
        findings.insert(PacketFinding::new(
            "correlation_packet_run_mismatch",
            "correlation.packet_id",
            "Tie correlation.packet_id to the same stable run id as the status run projection.",
        ));
    }
    require_non_empty_str(
        obj.get("status_artifact_uri"),
        "correlation.status_artifact_uri",
        "status_artifact_uri_missing",
        findings,
    );
}

fn validate_status_retention(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, "retention", findings) else {
        return;
    };
    require_positive_u64(
        obj.get("status_ttl_days"),
        "retention.status_ttl_days",
        "status_ttl_days_invalid",
        findings,
    );
    require_non_empty_str(
        obj.get("expires_at"),
        "retention.expires_at",
        "retention_expires_at_missing",
        findings,
    );
    let Some(pii) = object_at(obj.get("pii_policy"), "retention.pii_policy", findings) else {
        return;
    };
    require_exact_bool(
        pii.get("redaction_required"),
        true,
        "retention.pii_policy.redaction_required",
        "status_redaction_required_missing",
        findings,
    );
    require_allowed_str(
        pii.get("diagnostics_pii"),
        &["redacted-or-none"],
        "retention.pii_policy.diagnostics_pii",
        "status_diagnostics_pii_invalid",
        findings,
    );
}

fn validate_status_diagnosability(
    value: Option<&Value>,
    failure_required: bool,
    artifact_ref_count: usize,
    diagnostic_ref_count: usize,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(obj) = object_at(value, "diagnosability", findings) else {
        return;
    };
    require_exact_bool(
        obj.get("actions_log_scrape_required"),
        false,
        "diagnosability.actions_log_scrape_required",
        "actions_log_scrape_required",
        findings,
    );
    require_exact_bool(
        obj.get("first_diagnosis_from_status_api"),
        true,
        "diagnosability.first_diagnosis_from_status_api",
        "status_api_diagnosis_not_declared",
        findings,
    );
    if failure_required {
        if artifact_ref_count == 0 {
            findings.insert(PacketFinding::new(
                "status_failure_artifact_refs_missing",
                "artifact_refs",
                "Failed/timed-out status projections must include typed artifact refs for first diagnosis.",
            ));
        }
        if diagnostic_ref_count == 0 {
            findings.insert(PacketFinding::new(
                "status_failure_diagnostic_refs_missing",
                "diagnostic_refs",
                "Failed/timed-out status projections must include redacted diagnostic refs for first diagnosis.",
            ));
        }
    }
}

fn canonical_packet_id(provider: &str, run_id: &str, attempt: u64) -> String {
    format!("cloud-ci-run-packet:{provider}:{run_id}:attempt-{attempt}")
}

fn is_canonical_packet_id_shape(packet_id: &str) -> bool {
    let mut parts = packet_id.split(':');
    matches!(parts.next(), Some("cloud-ci-run-packet"))
        && parts
            .next()
            .is_some_and(|provider| !provider.is_empty() && provider.bytes().all(is_stable_id_byte))
        && parts
            .next()
            .is_some_and(|run_id| !run_id.is_empty() && run_id.bytes().all(is_stable_id_byte))
        && parts.next().is_some_and(|attempt| {
            attempt
                .strip_prefix("attempt-")
                .is_some_and(|value| !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()))
        })
        && parts.next().is_none()
}

fn validate_context_binding(
    producer: Option<&Value>,
    run: Option<&Value>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let required_context = str_at(producer.and_then(|producer| producer.get("required_context")));
    let merge_authority_context =
        str_at(producer.and_then(|producer| producer.get("merge_authority_context")));
    let status_context = str_at(run.and_then(|run| run.get("status_context")));
    if let (Some(required_context), Some(merge_authority_context), Some(status_context)) =
        (required_context, merge_authority_context, status_context)
        && (required_context != merge_authority_context || required_context != status_context)
    {
        findings.insert(PacketFinding::new(
            "context_binding_mismatch",
            "producer.required_context",
            "Bind producer.required_context, producer.merge_authority_context, and run.status_context to one canonical required context; do not silently mix cloud-ci-required and oya-ci-required.",
        ));
    }
}

fn validate_producer(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, "producer", findings) else {
        return;
    };
    require_allowed_str(
        obj.get("required_context"),
        &["cloud-ci-required", "oya-ci-required"],
        "producer.required_context",
        "producer_required_context_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("merge_authority_context"),
        &["cloud-ci-required", "oya-ci-required"],
        "producer.merge_authority_context",
        "producer_merge_authority_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("control_plane"),
        &["trusted-cloud-ci-controller", "github-actions-bridge"],
        "producer.control_plane",
        "producer_control_plane_invalid",
        findings,
    );
    require_non_empty_str(
        obj.get("workflow"),
        "producer.workflow",
        "producer_workflow_missing",
        findings,
    );
    require_exact_bool(
        obj.get("retired_evidence_authority"),
        false,
        "producer.retired_evidence_authority",
        "retired_evidence_authority_forbidden",
        findings,
    );
}

fn validate_run(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, "run", findings) else {
        return;
    };
    require_non_empty_str(obj.get("run_id"), "run.run_id", "run_id_missing", findings);
    require_positive_u64(
        obj.get("attempt"),
        "run.attempt",
        "run_attempt_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("provider"),
        &["github-actions", "owned-cloud-ci"],
        "run.provider",
        "run_provider_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("status_context"),
        &["cloud-ci-required", "oya-ci-required"],
        "run.status_context",
        "run_status_context_invalid",
        findings,
    );
    require_allowed_str(
        obj.get("conclusion"),
        &["passed", "failed", "cancelled", "infra_red"],
        "run.conclusion",
        "run_conclusion_invalid",
        findings,
    );
    require_non_empty_str(
        obj.get("started_at"),
        "run.started_at",
        "run_started_at_missing",
        findings,
    );
    require_non_empty_str(
        obj.get("completed_at"),
        "run.completed_at",
        "run_completed_at_missing",
        findings,
    );
}

fn validate_subject(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, "subject", findings) else {
        return;
    };
    require_non_empty_str(
        obj.get("repository"),
        "subject.repository",
        "repository_missing",
        findings,
    );
    require_positive_u64(
        obj.get("pr_number"),
        "subject.pr_number",
        "pr_number_invalid",
        findings,
    );
    for field in ["head_ref", "base_ref"] {
        require_non_empty_str(
            obj.get(field),
            format!("subject.{field}"),
            "git_ref_missing",
            findings,
        );
    }
    for field in ["head_sha", "base_sha", "merge_base_sha"] {
        match str_at(obj.get(field)) {
            Some(value) if is_git_sha(value) => {}
            _ => {
                findings.insert(PacketFinding::new(
                    "git_sha_invalid",
                    format!("subject.{field}"),
                    "Emit every head/base/merge-base revision as a full 40-hex git object id.",
                ));
            }
        }
    }
}

fn validate_transitions(
    value: Option<&Value>,
    conclusion: Option<&str>,
    run_id: Option<&str>,
    artifact_ids: &BTreeSet<String>,
    diagnostic_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(items) = array_at(value, "transitions", findings) else {
        return;
    };
    if items.len() < 3 {
        findings.insert(PacketFinding::new(
            "transition_history_too_short",
            "transitions",
            "Emit run, gate, and step transitions so operators can reconstruct status without logs.",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut has_failed_transition = false;
    for (index, item) in items.iter().enumerate() {
        let key = format!("transitions[{index}]");
        let Some(obj) = item.as_object() else {
            findings.insert(PacketFinding::new(
                "transition_not_object",
                key,
                "Emit each transition as an object with stable ids, state, timestamp, and duration.",
            ));
            continue;
        };
        let transition_id = str_at(obj.get("transition_id"));
        match transition_id {
            Some(id) if !id.is_empty() => {
                if !ids.insert(id.to_owned()) {
                    findings.insert(PacketFinding::new(
                        "transition_id_duplicate",
                        format!("transitions[{index}].transition_id"),
                        "Transition ids must be stable and unique inside the packet.",
                    ));
                }
            }
            _ => {
                findings.insert(PacketFinding::new(
                    "transition_id_missing",
                    format!("transitions[{index}].transition_id"),
                    "Emit a stable transition id for every run/gate/step state change.",
                ));
            }
        };
        let transition_type = str_at(obj.get("transition_type"));
        require_allowed_str(
            obj.get("transition_type"),
            &["run", "gate", "step"],
            format!("transitions[{index}].transition_type"),
            "transition_type_invalid",
            findings,
        );
        let state = str_at(obj.get("state"));
        require_allowed_str(
            obj.get("state"),
            &[
                "queued",
                "running",
                "passed",
                "failed",
                "skipped",
                "infra_red",
                "waived",
            ],
            format!("transitions[{index}].state"),
            "transition_state_invalid",
            findings,
        );
        validate_transition_id_shape(
            transition_id,
            transition_type,
            state,
            run_id,
            obj,
            index,
            findings,
        );
        require_non_empty_str(
            obj.get("at"),
            format!("transitions[{index}].at"),
            "transition_timestamp_missing",
            findings,
        );
        require_u64(
            obj.get("duration_ms"),
            format!("transitions[{index}].duration_ms"),
            "transition_duration_invalid",
            findings,
        );
        if matches!(state, Some("failed" | "infra_red")) {
            has_failed_transition = true;
            validate_failure(
                obj.get("failure"),
                &format!("transitions[{index}].failure"),
                artifact_ids,
                diagnostic_ids,
                findings,
            );
        }
    }
    if matches!(conclusion, Some("failed" | "infra_red")) && !has_failed_transition {
        findings.insert(PacketFinding::new(
            "failed_run_without_failed_transition",
            "transitions",
            "A failed required-context run must name the failed gate or step and include typed failure detail.",
        ));
    }
}

fn validate_failure(
    value: Option<&Value>,
    key: &str,
    artifact_ids: &BTreeSet<String>,
    diagnostic_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(obj) = object_at(value, key, findings) else {
        return;
    };
    require_allowed_str(
        obj.get("taxonomy"),
        &[
            "code_regression",
            "policy_violation",
            "infra_red",
            "operator_waiver_required",
            "cancelled",
            "flake_suspected",
        ],
        format!("{key}.taxonomy"),
        "failure_taxonomy_invalid",
        findings,
    );
    for field in ["code", "summary", "remediation"] {
        require_non_empty_str(
            obj.get(field),
            format!("{key}.{field}"),
            "failure_detail_missing",
            findings,
        );
    }
    validate_ref_array(
        obj.get("artifact_refs"),
        &format!("{key}.artifact_refs"),
        "failure_artifact_refs_missing",
        artifact_ids,
        true,
        findings,
    );
    validate_ref_array(
        obj.get("diagnostic_refs"),
        &format!("{key}.diagnostic_refs"),
        "failure_diagnostic_refs_missing",
        diagnostic_ids,
        true,
        findings,
    );
}

fn validate_artifacts(
    value: Option<&Value>,
    run_id: Option<&str>,
    findings: &mut BTreeSet<PacketFinding>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(items) = array_at(value, "artifacts", findings) else {
        return ids;
    };
    if items.is_empty() {
        findings.insert(PacketFinding::new(
            "artifact_list_empty",
            "artifacts",
            "Emit typed gate/status artifacts for the run packet; raw logs are not sufficient.",
        ));
    }
    for (index, item) in items.iter().enumerate() {
        let key = format!("artifacts[{index}]");
        let Some(obj) = item.as_object() else {
            findings.insert(PacketFinding::new(
                "artifact_not_object",
                key,
                "Emit each artifact pointer as an object with kind, URI, digest, and retention policy.",
            ));
            continue;
        };
        if let Some(id) = str_at(obj.get("artifact_id")).filter(|id| !id.is_empty()) {
            if !ids.insert(id.to_owned()) {
                findings.insert(PacketFinding::new(
                    "artifact_id_duplicate",
                    format!("artifacts[{index}].artifact_id"),
                    "Artifact ids must be stable and unique inside the packet.",
                ));
            }
        } else {
            findings.insert(PacketFinding::new(
                "artifact_id_missing",
                format!("artifacts[{index}].artifact_id"),
                "Emit a stable artifact id for every typed artifact pointer.",
            ));
        }
        let kind = str_at(obj.get("kind"));
        require_allowed_str(
            obj.get("kind"),
            &[
                "gate-report",
                "step-report",
                "redacted-diagnostics",
                "status-packet",
            ],
            format!("artifacts[{index}].kind"),
            "artifact_kind_invalid",
            findings,
        );
        validate_artifact_id_shape(
            str_at(obj.get("artifact_id")),
            kind,
            run_id,
            index,
            findings,
        );
        match str_at(obj.get("uri")) {
            Some(uri) if !uri.is_empty() => {
                if uri.contains("/actions/runs/") {
                    findings.insert(PacketFinding::new(
                        "raw_actions_log_artifact_forbidden",
                        format!("artifacts[{index}].uri"),
                        "Point at typed cloud-ci status/gate artifacts; do not make Actions job logs the status API.",
                    ));
                }
            }
            _ => {
                findings.insert(PacketFinding::new(
                    "artifact_uri_missing",
                    format!("artifacts[{index}].uri"),
                    "Emit a typed artifact URI for every gate report, diagnostic bundle, or status packet.",
                ));
            }
        }
        match str_at(obj.get("digest_sha256")) {
            Some(value) if is_sha256_digest(value) => {}
            _ => {
                findings.insert(PacketFinding::new(
                    "artifact_digest_invalid",
                    format!("artifacts[{index}].digest_sha256"),
                    "Seal every artifact pointer with a canonical sha256:<64-hex> digest.",
                ));
            }
        }
        validate_artifact_retention(
            obj.get("retention"),
            &format!("artifacts[{index}].retention"),
            findings,
        );
    }
    ids
}

fn validate_artifact_retention(
    value: Option<&Value>,
    key: &str,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(obj) = object_at(value, key, findings) else {
        return;
    };
    require_non_empty_str(
        obj.get("policy"),
        format!("{key}.policy"),
        "artifact_retention_policy_missing",
        findings,
    );
    require_non_empty_str(
        obj.get("expires_at"),
        format!("{key}.expires_at"),
        "artifact_retention_expiry_missing",
        findings,
    );
    if !matches!(obj.get("contains_pii"), Some(Value::Bool(_))) {
        findings.insert(PacketFinding::new(
            "artifact_pii_flag_missing",
            format!("{key}.contains_pii"),
            "Declare whether the typed artifact contains PII and list the PII classes when true.",
        ));
    }
    if !matches!(obj.get("pii_classes"), Some(Value::Array(_))) {
        findings.insert(PacketFinding::new(
            "artifact_pii_classes_missing",
            format!("{key}.pii_classes"),
            "Declare artifact PII classes as an array, empty when the artifact is PII-free.",
        ));
    }
}

fn validate_diagnostics(
    value: Option<&Value>,
    failure_required: bool,
    findings: &mut BTreeSet<PacketFinding>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(items) = array_at(value, "diagnostics", findings) else {
        return ids;
    };
    if failure_required && items.is_empty() {
        findings.insert(PacketFinding::new(
            "diagnostics_empty",
            "diagnostics",
            "Emit redacted typed diagnostics so failed gates are diagnosable without raw logs.",
        ));
    }
    for (index, item) in items.iter().enumerate() {
        let key = format!("diagnostics[{index}]");
        let Some(obj) = item.as_object() else {
            findings.insert(PacketFinding::new(
                "diagnostic_not_object",
                key,
                "Emit each diagnostic as an object with id, gate, severity, code, redacted message, and redaction metadata.",
            ));
            continue;
        };
        if let Some(id) = str_at(obj.get("diagnostic_id")).filter(|id| !id.is_empty()) {
            if !ids.insert(id.to_owned()) {
                findings.insert(PacketFinding::new(
                    "diagnostic_id_duplicate",
                    format!("diagnostics[{index}].diagnostic_id"),
                    "Diagnostic ids must be stable and unique inside the packet.",
                ));
            }
        } else {
            findings.insert(PacketFinding::new(
                "diagnostic_id_missing",
                format!("diagnostics[{index}].diagnostic_id"),
                "Emit a stable diagnostic id for every diagnostic entry.",
            ));
        }
        require_non_empty_str(
            obj.get("gate_id"),
            format!("diagnostics[{index}].gate_id"),
            "diagnostic_gate_id_missing",
            findings,
        );
        require_allowed_str(
            obj.get("severity"),
            &["info", "warning", "error"],
            format!("diagnostics[{index}].severity"),
            "diagnostic_severity_invalid",
            findings,
        );
        require_non_empty_str(
            obj.get("code"),
            format!("diagnostics[{index}].code"),
            "diagnostic_code_missing",
            findings,
        );
        validate_diagnostic_id_shape(
            str_at(obj.get("diagnostic_id")),
            str_at(obj.get("gate_id")),
            index,
            findings,
        );
        match str_at(obj.get("message_redacted")) {
            Some(message) if !message.is_empty() => {
                if contains_secret_like_text(message) {
                    findings.insert(PacketFinding::new(
                        "diagnostic_secret_unredacted",
                        format!("diagnostics[{index}].message_redacted"),
                        "Redact tenant, idempotency, token, Authorization, subject, and correlation values before emitting diagnostics.",
                    ));
                }
            }
            _ => {
                findings.insert(PacketFinding::new(
                    "diagnostic_message_missing",
                    format!("diagnostics[{index}].message_redacted"),
                    "Emit a human-actionable redacted diagnostic message.",
                ));
            }
        }
        validate_redaction(
            obj.get("redaction"),
            &format!("diagnostics[{index}].redaction"),
            findings,
        );
    }
    ids
}

fn validate_redaction(value: Option<&Value>, key: &str, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, key, findings) else {
        return;
    };
    require_exact_bool(
        obj.get("applied"),
        true,
        format!("{key}.applied"),
        "diagnostic_redaction_not_applied",
        findings,
    );
    if !matches!(obj.get("redacted_fields"), Some(Value::Array(_))) {
        findings.insert(PacketFinding::new(
            "diagnostic_redacted_fields_missing",
            format!("{key}.redacted_fields"),
            "List redacted diagnostic field classes; use an empty array only when no sensitive fields were present.",
        ));
    }
}

fn validate_retention(value: Option<&Value>, findings: &mut BTreeSet<PacketFinding>) {
    let Some(obj) = object_at(value, "retention", findings) else {
        return;
    };
    require_positive_u64(
        obj.get("packet_ttl_days"),
        "retention.packet_ttl_days",
        "packet_ttl_invalid",
        findings,
    );
    require_non_empty_str(
        obj.get("expires_at"),
        "retention.expires_at",
        "packet_retention_expiry_missing",
        findings,
    );
    let Some(policy) = object_at(obj.get("pii_policy"), "retention.pii_policy", findings) else {
        return;
    };
    require_exact_bool(
        policy.get("redaction_required"),
        true,
        "retention.pii_policy.redaction_required",
        "packet_redaction_policy_disabled",
        findings,
    );
    require_exact_str(
        policy.get("diagnostics_pii"),
        "redacted-or-none",
        "retention.pii_policy.diagnostics_pii",
        "packet_diagnostics_pii_policy_invalid",
        findings,
    );
    require_exact_str(
        policy.get("raw_log_storage"),
        "not-a-status-api-dependency",
        "retention.pii_policy.raw_log_storage",
        "raw_log_storage_policy_invalid",
        findings,
    );
}

fn validate_diagnosability(
    value: Option<&Value>,
    failure_required: bool,
    artifact_ids: &BTreeSet<String>,
    diagnostic_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(obj) = object_at(value, "diagnosability", findings) else {
        return;
    };
    require_exact_bool(
        obj.get("actions_log_scrape_required"),
        false,
        "diagnosability.actions_log_scrape_required",
        "actions_log_scrape_required",
        findings,
    );
    if failure_required {
        require_non_empty_str(
            obj.get("primary_failed_gate_id"),
            "diagnosability.primary_failed_gate_id",
            "primary_failed_gate_missing",
            findings,
        );
    } else if matches!(str_at(obj.get("primary_failed_gate_id")), Some(value) if !value.is_empty())
    {
        findings.insert(PacketFinding::new(
            "passed_packet_primary_failed_gate_forbidden",
            "diagnosability.primary_failed_gate_id",
            "Passed/cancelled packets must not invent a primary failed gate.",
        ));
    }
    validate_ref_array(
        obj.get("artifact_refs"),
        "diagnosability.artifact_refs",
        "diagnosability_artifact_refs_missing",
        artifact_ids,
        failure_required,
        findings,
    );
    validate_ref_array(
        obj.get("diagnostic_refs"),
        "diagnosability.diagnostic_refs",
        "diagnosability_diagnostic_refs_missing",
        diagnostic_ids,
        failure_required,
        findings,
    );
    if !failure_required
        && let Some(items) = obj.get("diagnostic_refs").and_then(Value::as_array)
        && !items.is_empty()
    {
        findings.insert(PacketFinding::new(
            "passed_packet_diagnostic_refs_forbidden",
            "diagnosability.diagnostic_refs",
            "Passed/cancelled packets must not carry failed-gate diagnostic references.",
        ));
    }
}

fn validate_ref_array(
    value: Option<&Value>,
    key: &str,
    code: &str,
    valid_ids: &BTreeSet<String>,
    require_non_empty: bool,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(items) = array_at(value, key, findings) else {
        return;
    };
    if require_non_empty && items.is_empty() {
        findings.insert(PacketFinding::new(
            code,
            key,
            "Reference at least one typed artifact or diagnostic id; raw log scraping is not a status API.",
        ));
    }
    for (index, item) in items.iter().enumerate() {
        match item.as_str() {
            Some(id) if valid_ids.contains(id) => {}
            Some(_) => {
                findings.insert(PacketFinding::new(
                    "packet_reference_unknown",
                    format!("{key}[{index}]"),
                    "Every artifact_refs/diagnostic_refs entry must point at a typed artifact/diagnostic in the same packet.",
                ));
            }
            None => {
                findings.insert(PacketFinding::new(
                    "packet_reference_not_string",
                    format!("{key}[{index}]"),
                    "Reference ids must be strings.",
                ));
            }
        };
    }
}

fn validate_transition_id_shape(
    transition_id: Option<&str>,
    transition_type: Option<&str>,
    state: Option<&str>,
    run_id: Option<&str>,
    obj: &serde_json::Map<String, Value>,
    index: usize,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let Some(transition_id) = transition_id.filter(|value| !value.is_empty()) else {
        return;
    };
    let (Some(transition_type), Some(state)) = (transition_type, state) else {
        return;
    };
    let expected = match transition_type {
        "run" => run_id.map(|run_id| format!("run:{run_id}:{state}")),
        "gate" => str_at(obj.get("gate_id")).map(|gate_id| format!("gate:{gate_id}:{state}")),
        "step" => match (str_at(obj.get("gate_id")), str_at(obj.get("step_id"))) {
            (Some(gate_id), Some(step_id)) => Some(format!("step:{gate_id}:{step_id}:{state}")),
            _ => None,
        },
        _ => None,
    };
    match expected {
        Some(expected) if transition_id == expected => {}
        Some(_) => {
            findings.insert(PacketFinding::new(
                "transition_id_unstable",
                format!("transitions[{index}].transition_id"),
                "Use canonical transition ids: run:<run_id>:<state>, gate:<gate_id>:<state>, or step:<gate_id>:<step_id>:<state>.",
            ));
        }
        None => {
            findings.insert(PacketFinding::new(
                "transition_id_context_missing",
                format!("transitions[{index}].transition_id"),
                "Provide the run_id/gate_id/step_id fields needed to derive the canonical transition id.",
            ));
        }
    };
}

fn validate_artifact_id_shape(
    artifact_id: Option<&str>,
    kind: Option<&str>,
    run_id: Option<&str>,
    index: usize,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let (Some(artifact_id), Some(kind)) = (artifact_id.filter(|value| !value.is_empty()), kind)
    else {
        return;
    };
    let prefix = format!("artifact:{kind}:");
    if !artifact_id.starts_with(&prefix) || artifact_id[prefix.len()..].is_empty() {
        findings.insert(PacketFinding::new(
            "artifact_id_unstable",
            format!("artifacts[{index}].artifact_id"),
            "Use canonical artifact ids: artifact:<kind>:<stable-run-or-gate-key>.",
        ));
        return;
    }
    if kind == "status-packet"
        && let Some(run_id) = run_id
    {
        let expected = format!("artifact:status-packet:{run_id}");
        if artifact_id != expected {
            findings.insert(PacketFinding::new(
                "artifact_id_run_mismatch",
                format!("artifacts[{index}].artifact_id"),
                "Tie the status-packet artifact id to the run id exactly: artifact:status-packet:<run_id>.",
            ));
        }
    }
}

fn validate_diagnostic_id_shape(
    diagnostic_id: Option<&str>,
    gate_id: Option<&str>,
    index: usize,
    findings: &mut BTreeSet<PacketFinding>,
) {
    let (Some(diagnostic_id), Some(gate_id)) =
        (diagnostic_id.filter(|value| !value.is_empty()), gate_id)
    else {
        return;
    };
    let prefix = format!("diag:{gate_id}:");
    let tail = diagnostic_id.strip_prefix(&prefix);
    if !tail.is_some_and(|tail| !tail.is_empty() && tail.bytes().all(is_stable_id_byte)) {
        findings.insert(PacketFinding::new(
            "diagnostic_id_unstable",
            format!("diagnostics[{index}].diagnostic_id"),
            "Use canonical diagnostic ids: diag:<gate_id>:<stable-diagnostic-key>.",
        ));
    }
}

fn object_at<'a>(
    value: Option<&'a Value>,
    key: impl Into<String>,
    findings: &mut BTreeSet<PacketFinding>,
) -> Option<&'a serde_json::Map<String, Value>> {
    match value.and_then(Value::as_object) {
        Some(obj) => Some(obj),
        None => {
            findings.insert(PacketFinding::new(
                "packet_object_missing",
                key.into(),
                "Emit the required object with the canonical cloud-ci run observability packet shape.",
            ));
            None
        }
    }
}

fn array_at<'a>(
    value: Option<&'a Value>,
    key: impl Into<String>,
    findings: &mut BTreeSet<PacketFinding>,
) -> Option<&'a Vec<Value>> {
    match value.and_then(Value::as_array) {
        Some(items) => Some(items),
        None => {
            findings.insert(PacketFinding::new(
                "packet_array_missing",
                key.into(),
                "Emit the required array with the canonical cloud-ci run observability packet shape.",
            ));
            None
        }
    }
}

fn str_at(value: Option<&Value>) -> Option<&str> {
    value.and_then(Value::as_str)
}

fn require_non_empty_str(
    value: Option<&Value>,
    key: impl Into<String>,
    code: &str,
    findings: &mut BTreeSet<PacketFinding>,
) {
    if !matches!(str_at(value), Some(text) if !text.is_empty()) {
        findings.insert(PacketFinding::new(
            code,
            key,
            "Emit this field as a non-empty string in the canonical packet.",
        ));
    }
}

fn require_exact_str(
    value: Option<&Value>,
    expected: &str,
    key: impl Into<String>,
    code: &str,
    findings: &mut BTreeSet<PacketFinding>,
) {
    if str_at(value) != Some(expected) {
        findings.insert(PacketFinding::new(
            code,
            key,
            "Emit the exact canonical enum/string value required by the packet contract.",
        ));
    }
}

fn require_allowed_str(
    value: Option<&Value>,
    allowed: &[&str],
    key: impl Into<String>,
    code: &str,
    findings: &mut BTreeSet<PacketFinding>,
) {
    match str_at(value) {
        Some(text) if allowed.contains(&text) => {}
        _ => {
            findings.insert(PacketFinding::new(
                code,
                key,
                "Emit one of the canonical enum values defined by the packet contract.",
            ));
        }
    }
}

fn require_exact_bool(
    value: Option<&Value>,
    expected: bool,
    key: impl Into<String>,
    code: &str,
    findings: &mut BTreeSet<PacketFinding>,
) {
    if value.and_then(Value::as_bool) != Some(expected) {
        findings.insert(PacketFinding::new(
            code,
            key,
            "Emit the exact boolean required by the packet contract.",
        ));
    }
}

fn require_positive_u64(
    value: Option<&Value>,
    key: impl Into<String>,
    code: &str,
    findings: &mut BTreeSet<PacketFinding>,
) {
    match value.and_then(Value::as_u64) {
        Some(value) if value > 0 => {}
        _ => {
            findings.insert(PacketFinding::new(
                code,
                key,
                "Emit this field as a positive integer.",
            ));
        }
    }
}

fn require_u64(
    value: Option<&Value>,
    key: impl Into<String>,
    code: &str,
    findings: &mut BTreeSet<PacketFinding>,
) {
    if value.and_then(Value::as_u64).is_none() {
        findings.insert(PacketFinding::new(
            code,
            key,
            "Emit this field as a non-negative integer.",
        ));
    }
}

fn is_stable_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
}

fn is_git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn contains_secret_like_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("authorization:")
        || lower.contains("bearer ")
        || lower.contains("ghp_")
        || lower.contains("token=")
        || lower.contains("idempotency-key")
        || lower.contains("tenant_secret")
}
