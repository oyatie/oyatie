//! Observability tracing adapter: `tracing-subscriber` setup for executable roots.
//!
//! Pure kernels do not depend on this crate. Runtime binaries call it once at
//! process startup when they own stdout/stderr and want structured JSON tracing.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::error::Error;
use std::fmt;

use observability_domain::{
    CapabilityInvocationTraceContext, CapabilityInvocationTraceObserver,
    CapabilityInvocationTraceSpan, InvocationTraceResult, fields,
};
use tracing_subscriber::{EnvFilter, fmt as tracing_fmt};

/// Runtime tracing subscriber configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonTracingConfig {
    pub default_filter: String, // data_class: INTERNAL_ONLY
}

impl Default for JsonTracingConfig {
    fn default() -> Self {
        Self {
            default_filter: "info".to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservabilityInitError {
    InvalidDefaultFilter(String),
    SubscriberAlreadyInstalled,
}

impl fmt::Display for ObservabilityInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDefaultFilter(filter) => {
                write!(f, "invalid default tracing filter: {filter}")
            }
            Self::SubscriberAlreadyInstalled => write!(f, "tracing subscriber already installed"),
        }
    }
}

impl Error for ObservabilityInitError {}

/// `tracing` implementation of the app-facing capability invocation port.
#[derive(Clone, Debug, Default)]
pub struct TracingCapabilityInvocationObserver;

#[derive(Debug)]
struct TracingCapabilityInvocationSpan {
    span: tracing::span::EnteredSpan,
}

impl CapabilityInvocationTraceObserver for TracingCapabilityInvocationObserver {
    fn start_capability_invocation(
        &self,
        context: &CapabilityInvocationTraceContext,
    ) -> Box<dyn CapabilityInvocationTraceSpan> {
        Box::new(TracingCapabilityInvocationSpan::new(context))
    }
}

impl TracingCapabilityInvocationSpan {
    fn new(context: &CapabilityInvocationTraceContext) -> Self {
        let span = tracing::info_span!(
            observability_domain::CAPABILITY_INVOCATION_SPAN_NAME,
            service.name = context.service_name.as_str(),
            oyatie.tenant.id = context.tenant_id.as_str(),
            oyatie.tenant.region = context.tenant_region.as_str(),
            oyatie.cell.id = tracing::field::Empty,
            oyatie.cell.bound = context.cell_id.is_some(),
            oyatie.capability.id = context.capability_id.as_str(),
            oyatie.data_classes_touched = context.data_classes_touched.as_str(),
            oyatie.autonomy_tier = tracing::field::Empty,
            gen_ai.operation.name = context.operation_name.as_str(),
            gen_ai.provider.name = context.provider_name.as_str(),
            oyatie.invocation.result = tracing::field::Empty,
            error.type = tracing::field::Empty,
        )
        .entered();
        if let Some(cell_id) = context.cell_id.as_deref() {
            span.record(fields::CELL_ID, cell_id);
        }
        Self { span }
    }
}

impl CapabilityInvocationTraceSpan for TracingCapabilityInvocationSpan {
    fn record_autonomy_tier(&self, autonomy_tier: &str) {
        self.span.record(fields::AUTONOMY_TIER, autonomy_tier);
    }

    fn emit_result(&self, result: InvocationTraceResult) {
        self.span.record(fields::INVOCATION_RESULT, result.result);
        if let Some(error_type) = result.error_type {
            self.span.record(fields::ERROR_TYPE, error_type);
            tracing::event!(
                target: "application_app::observability",
                tracing::Level::WARN,
                {
                    "oyatie.invocation.result" = result.result,
                    "error.type" = error_type,
                }
            );
        } else {
            tracing::event!(
                target: "application_app::observability",
                tracing::Level::INFO,
                {
                    "oyatie.invocation.result" = result.result,
                }
            );
        }
    }
}

// ---------------------------------------------------------------------------
// SLO-breach trace emission
// ---------------------------------------------------------------------------

/// Stable field-name constants for SLO-breach spans.
///
/// General fields (`service.name`, `oyatie.tenant.id`, etc.) reuse
/// `observability_domain::fields::*`. Only SLO-specific keys are defined
/// here to avoid any duplication.
pub mod slo_fields {
    pub const SLO_NAME: &str = "slo.name";
    pub const SLO_OBJECTIVE: &str = "slo.objective";
    pub const SLO_ERROR_BUDGET_CONSUMED: &str = "slo.error_budget.consumed";
    pub const SLO_BURN_RATE_SHORT: &str = "slo.burn_rate.short";
    pub const SLO_BURN_RATE_LONG: &str = "slo.burn_rate.long";
    pub const SLO_ALERT_DECISION: &str = "slo.alert.decision";
}

/// Stable span name for SLO-breach evaluation traces.
pub const SLO_BREACH_SPAN_NAME: &str = "slo.breach.evaluate";

/// SLO identity and target objective.
#[derive(Clone, Debug)]
pub struct SloObjective {
    pub name: String, // data_class: INTERNAL_ONLY
    pub target: f64,  // data_class: INTERNAL_ONLY
}

/// Burn-rate inputs driving the alert evaluation.
#[derive(Clone, Debug)]
pub struct AlertBurnRate {
    pub short_window: f64,          // data_class: INTERNAL_ONLY
    pub long_window: f64,           // data_class: INTERNAL_ONLY
    pub error_budget_consumed: f64, // data_class: INTERNAL_ONLY
}

/// Low-cardinality alert decision resulting from the burn-rate evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertDecision {
    Page,
    Ticket,
    None,
}

impl AlertDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Ticket => "ticket",
            Self::None => "none",
        }
    }
}

/// Combined input value object for one SLO-breach trace observation.
#[derive(Clone, Debug)]
pub struct SloBreachTraceContext {
    pub objective: SloObjective,  // data_class: INTERNAL_ONLY
    pub burn_rate: AlertBurnRate, // data_class: INTERNAL_ONLY
    pub decision: AlertDecision,  // data_class: INTERNAL_ONLY
}

/// App-facing port for emitting a single SLO-breach trace span.
pub trait SloBreachTraceObserver: fmt::Debug + Send + Sync {
    fn observe(&self, ctx: &SloBreachTraceContext);
}

/// `tracing` implementation: opens a stable `info_span` carrying all SLO
/// fields and the alert decision as low-cardinality span fields.
///
/// Mirrors the `TracingCapabilityInvocationSpan` pattern: fields declared as
/// `tracing::field::Empty` in the `info_span!` macro, then recorded via
/// `span.record(...)` after entering.
#[derive(Clone, Debug, Default)]
pub struct TracingSloBreachTraceObserver;

impl SloBreachTraceObserver for TracingSloBreachTraceObserver {
    fn observe(&self, ctx: &SloBreachTraceContext) {
        let span = tracing::info_span!(
            SLO_BREACH_SPAN_NAME,
            "slo.name" = tracing::field::Empty,
            "slo.objective" = tracing::field::Empty,
            "slo.error_budget.consumed" = tracing::field::Empty,
            "slo.burn_rate.short" = tracing::field::Empty,
            "slo.burn_rate.long" = tracing::field::Empty,
            "slo.alert.decision" = tracing::field::Empty,
        );
        let entered = span.entered();
        entered.record(slo_fields::SLO_NAME, ctx.objective.name.as_str());
        entered.record(slo_fields::SLO_OBJECTIVE, ctx.objective.target);
        entered.record(
            slo_fields::SLO_ERROR_BUDGET_CONSUMED,
            ctx.burn_rate.error_budget_consumed,
        );
        entered.record(slo_fields::SLO_BURN_RATE_SHORT, ctx.burn_rate.short_window);
        entered.record(slo_fields::SLO_BURN_RATE_LONG, ctx.burn_rate.long_window);
        entered.record(slo_fields::SLO_ALERT_DECISION, ctx.decision.as_str());
        // Emit an info event inside the span so the JSON formatter writes a
        // record carrying all current-span fields (with_current_span(true)).
        tracing::info!(
            target: "oya_observability::slo_breach",
            "slo breach evaluated"
        );
        // `entered` drops here, exiting the span.
    }
}

/// No-op default: satisfies the trait, opens no span, allocates nothing.
///
/// Used when a composition root has not installed telemetry infrastructure.
#[derive(Clone, Debug, Default)]
pub struct NoopSloBreachTraceObserver;

impl SloBreachTraceObserver for NoopSloBreachTraceObserver {
    fn observe(&self, _ctx: &SloBreachTraceContext) {}
}

// ---------------------------------------------------------------------------

/// Install structured JSON tracing to stdout.
///
/// This is a process-global operation and should only be called by executable
/// composition roots. `RUST_LOG` overrides `default_filter` when present.
pub fn install_json_stdout_tracing(
    config: &JsonTracingConfig,
) -> Result<(), ObservabilityInitError> {
    let filter = build_env_filter(config, std::env::var("RUST_LOG").ok().as_deref())?;

    tracing_fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(true)
        .with_span_list(true)
        .with_writer(std::io::stdout)
        .try_init()
        .map_err(|_| ObservabilityInitError::SubscriberAlreadyInstalled)
}

fn build_env_filter(
    config: &JsonTracingConfig,
    env_filter: Option<&str>,
) -> Result<EnvFilter, ObservabilityInitError> {
    if let Some(env_filter) = env_filter
        && let Ok(filter) = EnvFilter::try_new(env_filter)
    {
        return Ok(filter);
    }
    EnvFilter::try_new(config.default_filter.as_str())
        .map_err(|_| ObservabilityInitError::InvalidDefaultFilter(config.default_filter.clone()))
}

#[cfg(test)]
mod tests {
    use super::{JsonTracingConfig, ObservabilityInitError, build_env_filter};

    #[test]
    fn invalid_default_filter_is_rejected_without_env_mutation() {
        let error = build_env_filter(
            &JsonTracingConfig {
                default_filter: "[".to_string(),
            },
            None,
        )
        .expect_err("invalid default filter must fail when no env override is supplied");
        assert_eq!(
            error,
            ObservabilityInitError::InvalidDefaultFilter("[".to_string())
        );
    }

    #[test]
    fn valid_env_filter_overrides_invalid_default_filter() {
        build_env_filter(
            &JsonTracingConfig {
                default_filter: "[".to_string(),
            },
            Some("info"),
        )
        .expect("valid env filter is accepted before fallback default");
    }
}
