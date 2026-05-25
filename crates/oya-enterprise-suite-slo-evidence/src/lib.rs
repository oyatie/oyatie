//! Enterprise Suite SLO evidence contract foundation.
//!
//! This control-plane crate defines the pre-cloud SLO evidence contract that a
//! later Oyatie cloud integration must satisfy before production SLO evidence is
//! claimed. It uses OpenSLO-style manifest paths, OTel metric stream names,
//! rolling error-budget windows, burn-rate alert windows, and rollback/canary
//! evidence references, but deliberately does not scrape metrics, contact an
//! OpenTelemetry collector, evaluate Prometheus queries, page an alert manager,
//! deploy cloud resources, or claim production SLO evidence.
#![forbid(unsafe_code)]

const SCHEMA_VERSION: u32 = 1;
const MIN_SUCCESS_TARGET_BPS: u16 = 9_900;
const MAX_TARGET_BPS: u16 = 10_000;
const MIN_WINDOW_DAYS: u16 = 7;
const MAX_WINDOW_DAYS: u16 = 90;
const MIN_OBJECTIVE_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnterpriseSloKind {
    Availability,
    Latency,
    Freshness,
    Correctness,
}

impl EnterpriseSloKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Availability => "availability",
            Self::Latency => "latency",
            Self::Freshness => "freshness",
            Self::Correctness => "correctness",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseSloObjective {
    pub name: &'static str,                  // data_class: PUBLIC
    pub kind: EnterpriseSloKind,             // data_class: PUBLIC
    pub user_journey: &'static str,          // data_class: INTERNAL_ONLY
    pub openslo_manifest_path: &'static str, // data_class: INTERNAL_ONLY
    pub sli_metric_name: &'static str,       // data_class: INTERNAL_ONLY
    pub target_bps: u16,                     // data_class: PUBLIC
    pub threshold_millis: Option<u32>,       // data_class: PUBLIC
    pub rolling_window_days: u16,            // data_class: PUBLIC
    pub evidence_ref: &'static str,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseSloBurnRatePolicy {
    pub fast_window_minutes: u16,        // data_class: PUBLIC
    pub slow_window_minutes: u16,        // data_class: PUBLIC
    pub page_burn_rate_threshold: u16,   // data_class: INTERNAL_ONLY
    pub ticket_burn_rate_threshold: u16, // data_class: INTERNAL_ONLY
    pub alert_policy_ref: &'static str,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseSloEvidencePlan {
    pub plan_name: &'static str,                       // data_class: PUBLIC
    pub service_name: &'static str,                    // data_class: PUBLIC
    pub objectives: Vec<EnterpriseSloObjective>,       // data_class: PUBLIC
    pub burn_rate_policy: EnterpriseSloBurnRatePolicy, // data_class: PUBLIC
    pub dashboard_ref: &'static str,                   // data_class: INTERNAL_ONLY
    pub otel_collector_ref: &'static str,              // data_class: INTERNAL_ONLY
    pub canary_evidence_ref: &'static str,             // data_class: INTERNAL_ONLY
    pub rollback_gate_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub error_budget_release_gate_required: bool,      // data_class: PUBLIC
    pub multi_window_burn_rate_alert_required: bool,   // data_class: PUBLIC
    pub openslo_manifests_required: bool,              // data_class: PUBLIC
    pub otel_metric_streams_required: bool,            // data_class: PUBLIC
    pub runtime_otel_export_attached: bool,            // data_class: INTERNAL_ONLY
    pub metrics_backend_attached: bool,                // data_class: INTERNAL_ONLY
    pub alert_manager_attached: bool,                  // data_class: INTERNAL_ONLY
    pub canary_runtime_attached: bool,                 // data_class: INTERNAL_ONLY
    pub rollback_automation_attached: bool,            // data_class: INTERNAL_ONLY
    pub production_slo_evidence_attached: bool,        // data_class: INTERNAL_ONLY
    pub multi_region_slo_evidence_attached: bool,      // data_class: INTERNAL_ONLY
    pub schema_version: u32,                           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnterpriseSloEvidenceError {
    InvalidPlanName,
    InvalidServiceName,
    MissingObjectives,
    DuplicateObjective(String),
    InvalidObjectiveName,
    InvalidManifestPath,
    InvalidMetricName,
    InvalidTarget,
    InvalidLatencyThreshold,
    InvalidWindow,
    InvalidEvidenceRef,
    InvalidBurnRateWindow,
    InvalidBurnRateThreshold,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn enterprise_suite_slo_evidence_plan() -> EnterpriseSloEvidencePlan {
    EnterpriseSloEvidencePlan {
        plan_name: "enterprise-suite-slo-evidence-plan",
        service_name: "enterprise-suite",
        objectives: vec![
            EnterpriseSloObjective {
                name: "enterprise-suite-availability",
                kind: EnterpriseSloKind::Availability,
                user_journey: "enterprise-suite-governed-api-serving",
                openslo_manifest_path: "microservices/enterprise-suite/slos/enterprise-suite-availability.openslo.yaml",
                sli_metric_name: "http.server.request.duration",
                target_bps: 9_990,
                threshold_millis: None,
                rolling_window_days: 28,
                evidence_ref: "evidence/slo/enterprise-suite/availability.jsonl",
            },
            EnterpriseSloObjective {
                name: "enterprise-suite-latency-p99",
                kind: EnterpriseSloKind::Latency,
                user_journey: "enterprise-suite-governed-api-serving",
                openslo_manifest_path: "microservices/enterprise-suite/slos/enterprise-suite-latency-p99.openslo.yaml",
                sli_metric_name: "http.server.request.duration",
                target_bps: 9_900,
                threshold_millis: Some(500),
                rolling_window_days: 28,
                evidence_ref: "evidence/slo/enterprise-suite/latency-p99.jsonl",
            },
            EnterpriseSloObjective {
                name: "enterprise-suite-audit-emission-lag-p99",
                kind: EnterpriseSloKind::Freshness,
                user_journey: "enterprise-suite-audit-chain-observability",
                openslo_manifest_path: "microservices/enterprise-suite/slos/enterprise-suite-audit-emission-lag-p99.openslo.yaml",
                sli_metric_name: "audit.chain.emit.lag",
                target_bps: 9_900,
                threshold_millis: Some(30_000),
                rolling_window_days: 28,
                evidence_ref: "evidence/slo/enterprise-suite/audit-emission-lag-p99.jsonl",
            },
            EnterpriseSloObjective {
                name: "enterprise-suite-cloud-readiness-gate-correctness",
                kind: EnterpriseSloKind::Correctness,
                user_journey: "enterprise-suite-cloud-readiness-release-gate",
                openslo_manifest_path: "microservices/enterprise-suite/slos/enterprise-suite-cloud-readiness-gate-correctness.openslo.yaml",
                sli_metric_name: "enterprise.cloud_readiness.gate.result",
                target_bps: 9_999,
                threshold_millis: None,
                rolling_window_days: 28,
                evidence_ref: "evidence/slo/enterprise-suite/cloud-readiness-gate-correctness.jsonl",
            },
        ],
        burn_rate_policy: EnterpriseSloBurnRatePolicy {
            fast_window_minutes: 60,
            slow_window_minutes: 360,
            page_burn_rate_threshold: 14,
            ticket_burn_rate_threshold: 2,
            alert_policy_ref: "alerts/enterprise-suite/slo-burn-rate",
        },
        dashboard_ref: "dashboards/enterprise-suite/slo-and-error-budget",
        otel_collector_ref: "otel/collector/enterprise-suite-dev",
        canary_evidence_ref: "evidence/canary/enterprise-suite/slo-release-gate.jsonl",
        rollback_gate_ref: "rollback/enterprise-suite/error-budget-release-gate",
        error_budget_release_gate_required: true,
        multi_window_burn_rate_alert_required: true,
        openslo_manifests_required: true,
        otel_metric_streams_required: true,
        runtime_otel_export_attached: false,
        metrics_backend_attached: false,
        alert_manager_attached: false,
        canary_runtime_attached: false,
        rollback_automation_attached: false,
        production_slo_evidence_attached: false,
        multi_region_slo_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn openslo_manifest_paths(plan: &EnterpriseSloEvidencePlan) -> Vec<&'static str> {
    plan.objectives
        .iter()
        .map(|objective| objective.openslo_manifest_path)
        .collect()
}

pub fn validate_enterprise_suite_slo_evidence_plan(
    plan: &EnterpriseSloEvidencePlan,
) -> Result<(), EnterpriseSloEvidenceError> {
    validate_slug(plan.plan_name, EnterpriseSloEvidenceError::InvalidPlanName)?;
    if plan.service_name != "enterprise-suite" {
        return Err(EnterpriseSloEvidenceError::InvalidServiceName);
    }
    if plan.objectives.len() < MIN_OBJECTIVE_COUNT {
        return Err(EnterpriseSloEvidenceError::MissingObjectives);
    }

    let mut seen = std::collections::BTreeSet::new();
    for objective in &plan.objectives {
        validate_objective(objective)?;
        if !seen.insert(objective.name) {
            return Err(EnterpriseSloEvidenceError::DuplicateObjective(
                objective.name.to_owned(),
            ));
        }
    }

    validate_burn_rate_policy(&plan.burn_rate_policy)?;
    validate_prefixed_ref(
        plan.dashboard_ref,
        "dashboards/enterprise-suite/",
        EnterpriseSloEvidenceError::InvalidEvidenceRef,
    )?;
    validate_prefixed_ref(
        plan.otel_collector_ref,
        "otel/collector/",
        EnterpriseSloEvidenceError::InvalidEvidenceRef,
    )?;
    validate_prefixed_ref(
        plan.canary_evidence_ref,
        "evidence/canary/enterprise-suite/",
        EnterpriseSloEvidenceError::InvalidEvidenceRef,
    )?;
    validate_prefixed_ref(
        plan.rollback_gate_ref,
        "rollback/enterprise-suite/",
        EnterpriseSloEvidenceError::InvalidEvidenceRef,
    )?;

    if !plan.error_budget_release_gate_required {
        return Err(EnterpriseSloEvidenceError::MissingRequiredControl(
            "error_budget_release_gate_required",
        ));
    }
    if !plan.multi_window_burn_rate_alert_required {
        return Err(EnterpriseSloEvidenceError::MissingRequiredControl(
            "multi_window_burn_rate_alert_required",
        ));
    }
    if !plan.openslo_manifests_required {
        return Err(EnterpriseSloEvidenceError::MissingRequiredControl(
            "openslo_manifests_required",
        ));
    }
    if !plan.otel_metric_streams_required {
        return Err(EnterpriseSloEvidenceError::MissingRequiredControl(
            "otel_metric_streams_required",
        ));
    }
    if plan.runtime_otel_export_attached
        || plan.metrics_backend_attached
        || plan.alert_manager_attached
        || plan.canary_runtime_attached
        || plan.rollback_automation_attached
        || plan.production_slo_evidence_attached
        || plan.multi_region_slo_evidence_attached
    {
        return Err(EnterpriseSloEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_objective(
    objective: &EnterpriseSloObjective,
) -> Result<(), EnterpriseSloEvidenceError> {
    validate_slug(
        objective.name,
        EnterpriseSloEvidenceError::InvalidObjectiveName,
    )?;
    validate_prefixed_ref(
        objective.user_journey,
        "enterprise-suite-",
        EnterpriseSloEvidenceError::InvalidObjectiveName,
    )?;
    validate_prefixed_ref(
        objective.openslo_manifest_path,
        "microservices/enterprise-suite/slos/",
        EnterpriseSloEvidenceError::InvalidManifestPath,
    )?;
    if !objective.openslo_manifest_path.ends_with(".openslo.yaml") {
        return Err(EnterpriseSloEvidenceError::InvalidManifestPath);
    }
    validate_metric_name(objective.sli_metric_name)?;
    if objective.target_bps < MIN_SUCCESS_TARGET_BPS || objective.target_bps > MAX_TARGET_BPS {
        return Err(EnterpriseSloEvidenceError::InvalidTarget);
    }
    if matches!(
        objective.kind,
        EnterpriseSloKind::Latency | EnterpriseSloKind::Freshness
    ) {
        match objective.threshold_millis {
            Some(1..=60_000) => {}
            _ => return Err(EnterpriseSloEvidenceError::InvalidLatencyThreshold),
        }
    }
    if !matches!(
        objective.kind,
        EnterpriseSloKind::Latency | EnterpriseSloKind::Freshness
    ) && objective.threshold_millis.is_some()
    {
        return Err(EnterpriseSloEvidenceError::InvalidLatencyThreshold);
    }
    if !(MIN_WINDOW_DAYS..=MAX_WINDOW_DAYS).contains(&objective.rolling_window_days) {
        return Err(EnterpriseSloEvidenceError::InvalidWindow);
    }
    validate_prefixed_ref(
        objective.evidence_ref,
        "evidence/slo/enterprise-suite/",
        EnterpriseSloEvidenceError::InvalidEvidenceRef,
    )?;
    Ok(())
}

fn validate_burn_rate_policy(
    policy: &EnterpriseSloBurnRatePolicy,
) -> Result<(), EnterpriseSloEvidenceError> {
    if policy.fast_window_minutes == 0
        || policy.slow_window_minutes <= policy.fast_window_minutes
        || policy.slow_window_minutes > 24 * 60
    {
        return Err(EnterpriseSloEvidenceError::InvalidBurnRateWindow);
    }
    if policy.page_burn_rate_threshold <= policy.ticket_burn_rate_threshold
        || policy.ticket_burn_rate_threshold == 0
    {
        return Err(EnterpriseSloEvidenceError::InvalidBurnRateThreshold);
    }
    validate_prefixed_ref(
        policy.alert_policy_ref,
        "alerts/enterprise-suite/",
        EnterpriseSloEvidenceError::InvalidEvidenceRef,
    )?;
    Ok(())
}

fn validate_slug(
    value: &str,
    error: EnterpriseSloEvidenceError,
) -> Result<(), EnterpriseSloEvidenceError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(error);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: EnterpriseSloEvidenceError,
) -> Result<(), EnterpriseSloEvidenceError> {
    if !value.starts_with(prefix)
        || value.len() <= prefix.len()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(error);
    }
    Ok(())
}

fn validate_metric_name(value: &str) -> Result<(), EnterpriseSloEvidenceError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '.' || ch == '_')
        || !value.contains('.')
    {
        return Err(EnterpriseSloEvidenceError::InvalidMetricName);
    }
    Ok(())
}

fn has_unsafe_text(value: &str) -> bool {
    value.contains('\0')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains('<')
        || value.contains('>')
        || value.contains('|')
        || value.contains(';')
        || value.contains('`')
        || value.contains('$')
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.starts_with('/') || value.starts_with('~')
}

fn has_credential_shape(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("credential")
}
