//! `oya gate validate saga-shape` runner (ADR-0222).
//!
//! Reads every `cloud/<svc>/specs/saga-*.json`, `oya/<svc>/specs/saga-*.json`,
//! and `microservices/<axis>/specs/saga-*.json` file plus the
//! canonical schema at `specs/saga-shape.json` and delegates to the
//! kernel validator in `oya-check-saga-shape`.
//!
//! Advisory mode: when zero saga definitions are discovered the gate
//! passes with a 0/0 report (per the deferred-gate pattern). Strict
//! promotion follows the ADR-0222 backlog at
//! `registry/saga-shape/migration-backlog.tsv`.

use std::fs;
use std::path::PathBuf;

use oya_check_saga_shape::{
    canonical_microservice_catalog, validate_saga_shape, AuditClass, CompensationKind,
    IdempotencyKeyStrategy, RollbackStrategy, SagaDefinition, SagaStep,
};
use serde_json::Value;

use crate::usage;

const DEFAULT_SCHEMA_PATH: &str = "specs/saga-shape.json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SagaShapeValidateArgs {
    service_roots: Vec<PathBuf>,
    schema_path: PathBuf,
    allow_empty: bool,
}

impl Default for SagaShapeValidateArgs {
    fn default() -> Self {
        Self {
            // Empty means "not yet resolved": the shared, registry-derived
            // default set is resolved in `parse_saga_shape_validate_args`,
            // where an absent expected root can be reported as an error
            // instead of being defaulted into an empty scan.
            service_roots: Vec::new(),
            schema_path: PathBuf::from(DEFAULT_SCHEMA_PATH),
            allow_empty: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SagaShapeReport {
    pub sagas_checked: usize,
    pub steps_checked: usize,
    pub advisory_count: usize,
}

pub(crate) fn parse_saga_shape_validate_args(
    args: Vec<String>,
) -> Result<SagaShapeValidateArgs, String> {
    let mut parsed = SagaShapeValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--allow-empty" => {
                parsed.allow_empty = true;
            }
            "--strict" => {
                parsed.allow_empty = false;
            }
            "--microservices-root" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                // Single explicit root overrides the multi-root default.
                parsed.service_roots = vec![PathBuf::from(value)];
            }
            "--schema" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.schema_path = PathBuf::from(value);
            }
            _ => return Err(usage()),
        }
    }
    if parsed.service_roots.is_empty() {
        // No explicit root: resolve the shared, registry-derived default
        // set. This propagates an error when an expected root is absent
        // rather than scanning nothing and passing.
        parsed.service_roots = crate::service_roots::default_service_roots()?;
    }
    Ok(parsed)
}

pub(crate) fn validate_saga_shape_gate(
    args: SagaShapeValidateArgs,
) -> Result<SagaShapeReport, String> {
    // Schema presence check — the gate refuses to run if the canonical
    // schema is missing (operator misconfiguration vs no-sagas-yet
    // advisory case).
    if !args.schema_path.exists() {
        return Err(format!(
            "saga-shape schema not found at {} (per ADR-0222)",
            args.schema_path.display()
        ));
    }

    let saga_files = discover_saga_files(&args.service_roots)?;
    if saga_files.is_empty() && args.allow_empty {
        return Ok(SagaShapeReport {
            sagas_checked: 0,
            steps_checked: 0,
            advisory_count: 0,
        });
    }
    if saga_files.is_empty() && !args.allow_empty {
        return Err("no saga definitions found and --strict requested".into());
    }

    let mut sagas: Vec<SagaDefinition> = Vec::with_capacity(saga_files.len());
    for path in &saga_files {
        let definition = read_saga_definition(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        sagas.push(definition);
    }

    let catalog = canonical_microservice_catalog();
    match validate_saga_shape(sagas, &catalog) {
        Ok(report) => Ok(SagaShapeReport {
            sagas_checked: report.sagas_checked,
            steps_checked: report.steps_checked,
            advisory_count: report.advisories.len(),
        }),
        Err(violations) => {
            let detail = violations
                .iter()
                .map(|violation| format!("  - {violation}"))
                .collect::<Vec<_>>()
                .join("\n");
            Err(format!(
                "saga shape validation failed ({} violations):\n{detail}",
                violations.len()
            ))
        }
    }
}

/// Discover `saga-*.json` specs under every service root, in BOTH layout
/// shapes: `<root>/specs/` and `<root>/<service>/specs/`. The predecessor
/// walked the depth-2 shape only.
fn discover_saga_files(service_roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for root in service_roots {
        for specs in crate::service_roots::list_service_subpaths(root, "specs") {
            let spec_entries = fs::read_dir(&specs.path)
                .map_err(|error| format!("cannot read {}: {error}", specs.path.display()))?;
            for spec_entry in spec_entries {
                let spec_entry =
                    spec_entry.map_err(|error| format!("spec entry unreadable: {error}"))?;
                let path = spec_entry.path();
                if !path.is_file() {
                    continue;
                }
                let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if file_name.starts_with("saga-") && file_name.ends_with(".json") {
                    paths.push(path);
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn read_saga_definition(path: &PathBuf) -> Result<SagaDefinition, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("unreadable saga file: {error}"))?;
    let root: Value = serde_json::from_str(&contents)
        .map_err(|error| format!("saga file is not valid JSON: {error}"))?;
    let object = root
        .as_object()
        .ok_or_else(|| "saga file root must be an object".to_string())?;

    let saga_id = required_string(object, "saga_id")?;
    let axis = required_string(object, "axis")?;
    let owner_team = required_string(object, "owner_team")?;
    let version = required_string(object, "version")?;
    let rollback_strategy = optional_string(object, "rollback_strategy")
        .unwrap_or_else(|| "reverse-order-compensation".to_string());
    let rollback_strategy = match rollback_strategy.as_str() {
        "reverse-order-compensation" => RollbackStrategy::ReverseOrderCompensation,
        "no-compensation-readonly" => RollbackStrategy::NoCompensationReadOnly,
        "best-effort-with-page" => RollbackStrategy::BestEffortWithPage,
        other => return Err(format!("unknown rollback_strategy {other}")),
    };
    let audit_chain_emit = object
        .get("audit_chain_emit")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);

    let steps_value = object
        .get("steps")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "saga.steps must be an array".to_string())?;
    let mut steps = Vec::with_capacity(steps_value.len());
    for step_value in steps_value {
        steps.push(read_step(step_value)?);
    }

    Ok(SagaDefinition {
        saga_id,
        axis,
        owner_team,
        version,
        steps,
        rollback_strategy,
        audit_chain_emit,
        source_path: path.to_string_lossy().to_string(),
    })
}

fn read_step(step_value: &Value) -> Result<SagaStep, String> {
    let object = step_value
        .as_object()
        .ok_or_else(|| "saga.step must be an object".to_string())?;
    let step_id = required_string(object, "step_id")?;
    let target_microservice = required_string(object, "target_microservice")?;
    let forward_action_object = object
        .get("forward_action")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "saga.step.forward_action must be an object".to_string())?;
    let forward_action_capability_ref = required_string(forward_action_object, "capability_ref")?;
    let compensation_object = object
        .get("compensation_action")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "saga.step.compensation_action must be an object".to_string())?;
    let compensation_kind = match required_string(compensation_object, "kind")?.as_str() {
        "Cancel" => CompensationKind::Cancel,
        "Refund" => CompensationKind::Refund,
        "Retry" => CompensationKind::Retry,
        "NoopWithEvidence" => CompensationKind::NoopWithEvidence,
        "Custom" => CompensationKind::Custom,
        other => return Err(format!("unknown compensation_action.kind {other}")),
    };
    let compensation_capability_ref = optional_string(compensation_object, "capability_ref");
    let idempotency_key_strategy = match required_string(object, "idempotency_key_strategy")?
        .as_str()
    {
        "saga-step-attempt" => IdempotencyKeyStrategy::SagaStepAttempt,
        "request-body-hash" => IdempotencyKeyStrategy::RequestBodyHash,
        "client-supplied" => IdempotencyKeyStrategy::ClientSupplied,
        other => return Err(format!("unknown idempotency_key_strategy {other}")),
    };
    let timeout_budget_ms = object
        .get("timeout_budget_ms")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "saga.step.timeout_budget_ms must be an integer".to_string())?;
    let retry_object = object
        .get("retry_policy")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "saga.step.retry_policy must be an object".to_string())?;
    let retry_max_attempts = retry_object
        .get("max_attempts")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "saga.step.retry_policy.max_attempts must be an integer".to_string())?;
    let retry_backoff_ms = retry_object
        .get("backoff_ms")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "saga.step.retry_policy.backoff_ms must be an integer".to_string())?;
    let audit_class = match required_string(object, "audit_class")?.as_str() {
        "ReadOnly" => AuditClass::ReadOnly,
        "WriteIdempotent" => AuditClass::WriteIdempotent,
        "WriteNonIdempotent" => AuditClass::WriteNonIdempotent,
        "SideEffectIrreversible" => AuditClass::SideEffectIrreversible,
        other => return Err(format!("unknown audit_class {other}")),
    };

    Ok(SagaStep {
        step_id,
        target_microservice,
        forward_action_capability_ref,
        compensation_kind,
        compensation_capability_ref,
        idempotency_key_strategy,
        timeout_budget_ms: timeout_budget_ms as u32,
        retry_max_attempts: retry_max_attempts as u8,
        retry_backoff_ms: retry_backoff_ms as u32,
        audit_class,
    })
}

fn required_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<String, String> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| format!("required string field `{key}` missing"))
}

fn optional_string(object: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}
