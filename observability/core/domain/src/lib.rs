//! Observability kernel: stable telemetry vocabulary and data-class-safe log exposure rules.
//!
//! This crate is intentionally pure. Runtime crates bind these names to concrete
//! tracing subscribers/exporters; kernels and app services use the vocabulary to
//! avoid string drift and accidental sensitive-content telemetry.

pub mod severity;
pub mod slo;

pub use severity::{Severity, UnknownSeverityLabel, should_emit};
pub use slo::budget::{
    BudgetWindow, burn_rate, classify_budget_windows, error_budget_remaining_ratio,
};
pub use slo::{
    AlertDecision, InvalidSLOObjective, PAGE_BUDGET_CONSUMED_MIN, PAGE_BURN_RATE_THRESHOLD,
    SLOObjective, TICKET_BUDGET_CONSUMED_MIN, TICKET_BURN_RATE_THRESHOLD, classify_burn_rate,
    slo_fields,
};

use std::fmt;

use data_boundary_kernel::{
    DataClass, DataClassification, OperationalDataClass, SubjectDataMarker,
};

/// Stable telemetry field names used by Oyatie spans and log records.
pub mod fields {
    pub const SERVICE_NAME: &str = "service.name";
    pub const SERVICE_VERSION: &str = "service.version";
    pub const DEPLOYMENT_ENVIRONMENT: &str = "deployment.environment";
    pub const TENANT_ID: &str = "oyatie.tenant.id";
    pub const TENANT_REGION: &str = "oyatie.tenant.region";
    pub const CELL_ID: &str = "oyatie.cell.id";
    pub const CELL_BOUND: &str = "oyatie.cell.bound";
    pub const CAPABILITY_ID: &str = "oyatie.capability.id";
    pub const DATA_CLASSES_TOUCHED: &str = "oyatie.data_classes_touched";
    pub const AUTONOMY_TIER: &str = "oyatie.autonomy_tier";
    pub const INVOCATION_RESULT: &str = "oyatie.invocation.result";
    pub const ERROR_TYPE: &str = "error.type";
    pub const GEN_AI_OPERATION_NAME: &str = "gen_ai.operation.name";
    pub const GEN_AI_PROVIDER_NAME: &str = "gen_ai.provider.name";
}

/// Stable span name for Foundry capability invocations.
pub const CAPABILITY_INVOCATION_SPAN_NAME: &str = "foundry.capability.invoke";

/// Stable GenAI operation label for capability invocation traces.
pub const CAPABILITY_INVOCATION_OPERATION_NAME: &str = "capability_invocation";

/// Stable provider label for Oyatie's Foundry runtime.
pub const FOUNDRY_PROVIDER_NAME: &str = "oya-foundry";

/// App-produced value object describing a capability invocation trace.
///
/// Concrete telemetry frameworks consume this at adapter/runtime edges. Keeping
/// it here prevents orchestration crates from constructing `tracing` spans or
/// OpenTelemetry records directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvocationTraceContext {
    pub service_name: String,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_region: String,        // data_class: INTERNAL_ONLY
    pub cell_id: Option<String>,      // data_class: INTERNAL_ONLY
    pub capability_id: String,        // data_class: INTERNAL_ONLY
    pub data_classes_touched: String, // data_class: INTERNAL_ONLY
    pub operation_name: String,       // data_class: INTERNAL_ONLY
    pub provider_name: String,        // data_class: INTERNAL_ONLY
}

/// Low-cardinality invocation result emitted to the active invocation trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvocationTraceResult {
    pub result: &'static str,             // data_class: INTERNAL_ONLY
    pub error_type: Option<&'static str>, // data_class: INTERNAL_ONLY
}

/// App-facing port for creating a concrete capability-invocation trace.
pub trait CapabilityInvocationTraceObserver: fmt::Debug + Send + Sync {
    fn start_capability_invocation(
        &self,
        context: &CapabilityInvocationTraceContext,
    ) -> Box<dyn CapabilityInvocationTraceSpan>;
}

/// App-facing handle for recording invocation fields without framework types.
pub trait CapabilityInvocationTraceSpan {
    fn record_autonomy_tier(&self, autonomy_tier: &str);
    fn emit_result(&self, result: InvocationTraceResult);
}

/// No-op observer used when a composition root has not installed telemetry.
///
/// This is a deliberate clean-architecture default, not error masking:
/// application crates stay framework-free, production/runtime composition can
/// inject a concrete adapter, and tests cover both this null implementation and
/// the `tracing` adapter behavior.
#[derive(Clone, Debug, Default)]
pub struct NoopCapabilityInvocationTraceObserver;

#[derive(Debug)]
struct NoopCapabilityInvocationTraceSpan;

impl CapabilityInvocationTraceObserver for NoopCapabilityInvocationTraceObserver {
    fn start_capability_invocation(
        &self,
        _context: &CapabilityInvocationTraceContext,
    ) -> Box<dyn CapabilityInvocationTraceSpan> {
        Box::new(NoopCapabilityInvocationTraceSpan)
    }
}

impl CapabilityInvocationTraceSpan for NoopCapabilityInvocationTraceSpan {
    fn record_autonomy_tier(&self, _autonomy_tier: &str) {}

    fn emit_result(&self, _result: InvocationTraceResult) {}
}

/// Whether a value with a data class may be emitted into logs/traces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryLogExposure {
    /// Raw value may be emitted when operationally necessary.
    Raw,
    /// Value must be redacted before emission.
    Redact,
    /// Value must never be emitted as observability payload.
    Forbid,
}

impl TelemetryLogExposure {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Redact => "redact",
            Self::Forbid => "forbid",
        }
    }
}

/// A redacted telemetry value with the source data class retained as evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedactedTelemetryValue {
    pub value: String,                             // data_class: INTERNAL_ONLY
    pub source_classification: DataClassification, // data_class: INTERNAL_ONLY
    /// Legacy compatibility label for telemetry/evidence consumers that still
    /// store hash-stable `DataClass` values while call sites migrate to
    /// `DataClassification`.
    pub source_data_class: DataClass, // data_class: INTERNAL_ONLY
    pub exposure: TelemetryLogExposure,            // data_class: INTERNAL_ONLY
}

/// Decide log exposure for an individual field classification.
pub const fn log_exposure_for_classification(
    classification: DataClassification,
) -> TelemetryLogExposure {
    let classification = classification.normalized();
    match classification {
        DataClassification::Operational(OperationalDataClass::Audit) => TelemetryLogExposure::Raw,
        DataClassification::Operational(OperationalDataClass::Secret) => {
            TelemetryLogExposure::Forbid
        }
        DataClassification::SubjectMarker(SubjectDataMarker::Children) => {
            TelemetryLogExposure::Redact
        }
        DataClassification::Privacy(data_class) => match data_class.data_class() {
            DataClass::Public | DataClass::InternalOnly | DataClass::Usage => {
                TelemetryLogExposure::Raw
            }
            DataClass::PiiIdentifying
            | DataClass::PiiSensitive
            | DataClass::PiiQuasiIdentifier
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
            | DataClass::BehavioralTenantProduct
            | DataClass::BehavioralAds
            | DataClass::DeclaredPreference
            | DataClass::SearchQuery => TelemetryLogExposure::Redact,
            DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23 => TelemetryLogExposure::Forbid,
            // Unreachable through public constructors; retain the historical
            // conservative mapping in case old serialized values are adapted
            // through a compatibility shim before reaching this function.
            DataClass::Audit => TelemetryLogExposure::Raw,
            DataClass::Children => TelemetryLogExposure::Redact,
            DataClass::Secret => TelemetryLogExposure::Forbid,
        },
    }
}

/// Decide log exposure for a legacy data-class label.
///
/// New code should call [`log_exposure_for_classification`]. This wrapper is
/// retained for payloads that still carry hash-stable or persisted `DataClass`
/// values during the classification split.
pub const fn legacy_log_exposure_for_data_class(data_class: DataClass) -> TelemetryLogExposure {
    log_exposure_for_classification(DataClassification::from_data_class(data_class))
}

#[deprecated(
    note = "use log_exposure_for_classification for canonical typed access or legacy_log_exposure_for_data_class for the compatibility projection"
)]
pub const fn log_exposure_for(data_class: DataClass) -> TelemetryLogExposure {
    legacy_log_exposure_for_data_class(data_class)
}

/// Return true when every classification may appear in an operational span as a class label.
pub fn classification_labels_are_safe(classes: &[DataClassification]) -> bool {
    classes.iter().all(|classification| {
        !matches!(
            log_exposure_for_classification(*classification),
            TelemetryLogExposure::Forbid
        )
    })
}

/// Return true when every legacy data class may appear in an operational span as a class label.
pub fn legacy_data_class_labels_are_safe(classes: &[DataClass]) -> bool {
    classes.iter().all(|data_class| {
        !matches!(
            legacy_log_exposure_for_data_class(*data_class),
            TelemetryLogExposure::Forbid
        )
    })
}

#[deprecated(
    note = "use classification_labels_are_safe for canonical typed access or legacy_data_class_labels_are_safe for the compatibility projection"
)]
pub fn class_labels_are_safe(classes: &[DataClass]) -> bool {
    legacy_data_class_labels_are_safe(classes)
}

/// Replacement label when a span would otherwise reveal a forbidden data-class label.
pub const FORBIDDEN_DATA_CLASS_PRESENT_LABEL: &str = "FORBIDDEN_DATA_CLASS_PRESENT";

/// Convert typed classification labels into a span-safe value for telemetry fields.
pub fn telemetry_data_classifications_label(classes: &[DataClassification]) -> String {
    if classification_labels_are_safe(classes) {
        data_classifications_label(classes)
    } else {
        FORBIDDEN_DATA_CLASS_PRESENT_LABEL.to_string()
    }
}

/// Convert legacy data-class labels into a span-safe value for telemetry fields.
pub fn legacy_telemetry_data_classes_label(classes: &[DataClass]) -> String {
    if legacy_data_class_labels_are_safe(classes) {
        legacy_data_classes_label(classes)
    } else {
        FORBIDDEN_DATA_CLASS_PRESENT_LABEL.to_string()
    }
}

#[deprecated(
    note = "use telemetry_data_classifications_label for canonical typed access or legacy_telemetry_data_classes_label for the compatibility projection"
)]
pub fn telemetry_data_classes_label(classes: &[DataClass]) -> String {
    legacy_telemetry_data_classes_label(classes)
}

/// Convert typed classification labels into a stable, low-cardinality span value.
pub fn data_classifications_label(classes: &[DataClassification]) -> String {
    classes
        .iter()
        .map(|classification| classification_label(*classification))
        .collect::<Vec<_>>()
        .join(",")
}

/// Convert legacy data-class labels into a stable, low-cardinality span value.
pub fn legacy_data_classes_label(classes: &[DataClass]) -> String {
    classes
        .iter()
        .map(|data_class| legacy_data_class_label(*data_class))
        .collect::<Vec<_>>()
        .join(",")
}

#[deprecated(
    note = "use data_classifications_label for canonical typed access or legacy_data_classes_label for the compatibility projection"
)]
pub fn data_classes_label(classes: &[DataClass]) -> String {
    legacy_data_classes_label(classes)
}

/// Convert one legacy data class into the canonical telemetry label.
pub const fn legacy_data_class_label(data_class: DataClass) -> &'static str {
    data_class.label()
}

#[deprecated(
    note = "use classification_label for canonical typed access or legacy_data_class_label for the compatibility projection"
)]
pub const fn data_class_label(data_class: DataClass) -> &'static str {
    legacy_data_class_label(data_class)
}

/// Convert one classification into the canonical telemetry label.
pub const fn classification_label(classification: DataClassification) -> &'static str {
    classification.label()
}

/// Redact a value according to the typed classification before it reaches telemetry.
pub fn redact_classification_for_telemetry(
    value: impl Into<String>,
    classification: impl Into<DataClassification>,
) -> RedactedTelemetryValue {
    let classification = classification.into().normalized();
    let exposure = log_exposure_for_classification(classification);
    let value = match exposure {
        TelemetryLogExposure::Raw => value.into(),
        TelemetryLogExposure::Redact | TelemetryLogExposure::Forbid => "[REDACTED]".to_string(),
    };
    RedactedTelemetryValue {
        value,
        source_classification: classification,
        source_data_class: classification.compatibility_data_class(),
        exposure,
    }
}

/// Redact a value according to a legacy data-class label before it reaches telemetry.
pub fn legacy_redact_for_telemetry(
    value: impl Into<String>,
    data_class: DataClass,
) -> RedactedTelemetryValue {
    redact_classification_for_telemetry(value, DataClassification::from(data_class))
}

#[deprecated(
    note = "use redact_classification_for_telemetry for canonical typed access or legacy_redact_for_telemetry for the compatibility projection"
)]
pub fn redact_for_telemetry(
    value: impl Into<String>,
    data_class: DataClass,
) -> RedactedTelemetryValue {
    legacy_redact_for_telemetry(value, data_class)
}

#[cfg(test)]
mod tests {
    use super::{
        CAPABILITY_INVOCATION_OPERATION_NAME, CapabilityInvocationTraceContext,
        CapabilityInvocationTraceObserver, FORBIDDEN_DATA_CLASS_PRESENT_LABEL,
        FOUNDRY_PROVIDER_NAME, InvocationTraceResult, NoopCapabilityInvocationTraceObserver,
        TelemetryLogExposure, classification_labels_are_safe, data_classifications_label,
        legacy_data_class_labels_are_safe, legacy_data_classes_label,
        legacy_log_exposure_for_data_class, legacy_redact_for_telemetry,
        legacy_telemetry_data_classes_label, log_exposure_for_classification,
        redact_classification_for_telemetry, telemetry_data_classifications_label,
    };
    use data_boundary_kernel::{
        DataClass, DataClassification, OperationalDataClass, SubjectDataMarker,
    };

    #[test]
    fn noop_invocation_trace_observer_accepts_app_trace_calls() {
        let observer = NoopCapabilityInvocationTraceObserver;
        let context = CapabilityInvocationTraceContext {
            service_name: "oya-foundation-app".to_string(),
            tenant_id: "ten_noop".to_string(),
            tenant_region: "region-alpha1".to_string(),
            cell_id: Some("cell-noop".to_string()),
            capability_id: "cap.noop".to_string(),
            data_classes_touched: "INTERNAL_ONLY".to_string(),
            operation_name: CAPABILITY_INVOCATION_OPERATION_NAME.to_string(),
            provider_name: FOUNDRY_PROVIDER_NAME.to_string(),
        };
        assert_eq!(context.tenant_region, "region-alpha1");

        let span = observer.start_capability_invocation(&context);

        span.record_autonomy_tier("T2");
        span.emit_result(InvocationTraceResult {
            result: "succeeded",
            error_type: None,
        });
    }

    #[test]
    fn data_class_labels_are_stable_low_cardinality_values() {
        assert_eq!(
            legacy_data_classes_label(&[DataClass::InternalOnly, DataClass::PiiIdentifying]),
            "INTERNAL_ONLY,PII_IDENTIFYING"
        );
        assert_eq!(
            legacy_data_classes_label(&[
                DataClass::PiiQuasiIdentifier,
                DataClass::FinancialRegulatedCredit,
                DataClass::SensitivePipaArticle23,
            ]),
            "PII_QUASI_IDENTIFIER,FINANCIAL_REGULATED_CREDIT,SENSITIVE_PIPA_ART23"
        );
        #[allow(deprecated)]
        {
            assert_eq!(
                super::data_classes_label(&[DataClass::InternalOnly, DataClass::PiiIdentifying]),
                legacy_data_classes_label(&[DataClass::InternalOnly, DataClass::PiiIdentifying])
            );
        }
        assert_eq!(
            data_classifications_label(&[
                DataClassification::from(DataClass::InternalOnly),
                DataClassification::from(OperationalDataClass::Audit),
                DataClassification::from(SubjectDataMarker::Children),
            ]),
            "INTERNAL_ONLY,AUDIT,CHILDREN"
        );
    }

    #[test]
    fn telemetry_exposure_blocks_sensitive_payloads_by_classification() {
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::Public)),
            TelemetryLogExposure::Raw
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::PiiIdentifying)),
            TelemetryLogExposure::Redact
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::Phi)),
            TelemetryLogExposure::Forbid
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::Pci)),
            TelemetryLogExposure::Forbid
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(
                DataClass::SensitivePipaArticle23
            )),
            TelemetryLogExposure::Forbid
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::SearchQuery)),
            TelemetryLogExposure::Redact
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(SubjectDataMarker::Children)),
            TelemetryLogExposure::Redact
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(OperationalDataClass::Audit)),
            TelemetryLogExposure::Raw
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(OperationalDataClass::Secret)),
            TelemetryLogExposure::Forbid
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::Audit)),
            TelemetryLogExposure::Raw
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::Children)),
            TelemetryLogExposure::Redact
        );
        assert_eq!(
            log_exposure_for_classification(DataClassification::from(DataClass::Secret)),
            TelemetryLogExposure::Forbid
        );

        let pii = redact_classification_for_telemetry(
            "person@example.test",
            DataClassification::from(DataClass::PiiIdentifying),
        );
        assert_eq!(pii.value, "[REDACTED]");
        assert_eq!(pii.exposure, TelemetryLogExposure::Redact);

        let secret =
            redact_classification_for_telemetry("secret-token", OperationalDataClass::Secret);
        assert_eq!(secret.value, "[REDACTED]");
        assert_eq!(
            secret.source_classification,
            DataClassification::from(OperationalDataClass::Secret)
        );
        assert_eq!(secret.source_data_class, DataClass::Secret);
        assert_eq!(secret.exposure, TelemetryLogExposure::Forbid);

        let legacy_data_class_marker = redact_classification_for_telemetry(
            "secret-token",
            DataClassification::from(DataClass::Secret),
        );
        assert_eq!(
            legacy_data_class_marker.source_classification,
            DataClassification::from(OperationalDataClass::Secret)
        );
        assert_eq!(
            legacy_data_class_marker.source_data_class,
            DataClass::Secret
        );
        assert_eq!(
            legacy_data_class_marker.exposure,
            TelemetryLogExposure::Forbid
        );
    }

    #[test]
    fn legacy_data_class_exposure_wrappers_preserve_bootstrap_semantics() {
        assert_eq!(
            legacy_log_exposure_for_data_class(DataClass::Audit),
            TelemetryLogExposure::Raw
        );
        assert_eq!(
            legacy_log_exposure_for_data_class(DataClass::Children),
            TelemetryLogExposure::Redact
        );
        assert_eq!(
            legacy_log_exposure_for_data_class(DataClass::Secret),
            TelemetryLogExposure::Forbid
        );

        let secret = legacy_redact_for_telemetry("secret-token", DataClass::Secret);
        assert_eq!(secret.value, "[REDACTED]");
        assert_eq!(
            secret.source_classification,
            DataClassification::from(OperationalDataClass::Secret)
        );
        assert_eq!(secret.exposure, TelemetryLogExposure::Forbid);

        #[allow(deprecated)]
        {
            assert_eq!(
                super::log_exposure_for(DataClass::Secret),
                legacy_log_exposure_for_data_class(DataClass::Secret)
            );
            assert_eq!(
                super::redact_for_telemetry("secret-token", DataClass::Secret),
                legacy_redact_for_telemetry("secret-token", DataClass::Secret)
            );
        }
    }

    #[test]
    fn only_class_labels_for_non_forbidden_classes_are_marked_safe() {
        assert!(legacy_data_class_labels_are_safe(&[
            DataClass::InternalOnly,
            DataClass::BehavioralTenantProduct
        ]));
        assert!(!legacy_data_class_labels_are_safe(&[
            DataClass::InternalOnly,
            DataClass::Pci
        ]));
        assert_eq!(
            legacy_telemetry_data_classes_label(&[
                DataClass::InternalOnly,
                DataClass::BehavioralTenantProduct
            ]),
            "INTERNAL_ONLY,BEHAVIORAL_TENANT_PRODUCT"
        );
        assert_eq!(
            legacy_telemetry_data_classes_label(&[DataClass::InternalOnly, DataClass::Pci]),
            FORBIDDEN_DATA_CLASS_PRESENT_LABEL
        );
        #[allow(deprecated)]
        {
            assert_eq!(
                super::class_labels_are_safe(&[DataClass::InternalOnly]),
                legacy_data_class_labels_are_safe(&[DataClass::InternalOnly])
            );
            assert_eq!(
                super::telemetry_data_classes_label(&[DataClass::InternalOnly]),
                legacy_telemetry_data_classes_label(&[DataClass::InternalOnly])
            );
        }
        assert!(classification_labels_are_safe(&[
            DataClassification::from(DataClass::InternalOnly),
            DataClassification::from(OperationalDataClass::Audit),
        ]));
        assert!(!classification_labels_are_safe(&[
            DataClassification::from(DataClass::InternalOnly),
            DataClassification::from(OperationalDataClass::Secret),
        ]));
        assert_eq!(
            telemetry_data_classifications_label(&[
                DataClassification::from(DataClass::InternalOnly),
                DataClassification::from(OperationalDataClass::Audit),
            ]),
            "INTERNAL_ONLY,AUDIT"
        );
        assert_eq!(
            telemetry_data_classifications_label(&[
                DataClassification::from(DataClass::InternalOnly),
                DataClassification::from(OperationalDataClass::Secret),
            ]),
            FORBIDDEN_DATA_CLASS_PRESENT_LABEL
        );
    }
}
