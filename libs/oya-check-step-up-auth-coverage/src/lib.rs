//! Advisory gate: scan OpenAPI 3.x specs for missing `x-acr-required` on
//! sensitive mutating operations.
//!
//! Authority: ADR-0189 (step-up authentication ACR classes).
//!
//! Heuristic:
//!
//! - Any operation whose HTTP method is POST / PUT / PATCH / DELETE on a
//!   path that mutates state SHOULD declare `x-acr-required: routine |
//!   elevated | sensitive | critical`.
//! - GET / HEAD / OPTIONS may declare it (read-elevated paths), but it is
//!   not required.
//! - An operation tagged `x-acr-exempt: true` is intentional and skipped
//!   (with a rationale string captured in the report).
//! - The "minimum required" for sensitive paths is derived by name
//!   heuristics: paths containing `secret`, `key`, `rotate`, `revoke`,
//!   `delete-tenant`, `transfer`, `payment`, `admin/*` are FLAGGED if the
//!   operation declares only `routine` or omits the extension; they
//!   should declare at least `sensitive`.
//!
//! The crate is read-only — it produces a [`StepUpAuthCoverageReport`]
//! and never mutates the spec.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

const SENSITIVE_PATH_NEEDLES: &[&str] = &[
    "/secret",
    "/secrets",
    "/key",
    "/keys",
    "/rotate",
    "/revoke",
    "/delete-tenant",
    "/transfer",
    "/payment",
    "/payments",
    "/admin",
    "/billing/currency",
    "/residency",
];

const MUTATING_METHODS: &[&str] = &["post", "put", "patch", "delete"];

/// A finding emitted by the gate. Tier indicates severity:
/// - `Missing` — operation MUST declare `x-acr-required` and does not.
/// - `BelowFloor` — declares routine on a sensitive-path; should be ≥sensitive.
/// - `UnknownValue` — declared but not one of the four canonical levels.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub path: String,
    pub method: String,
    pub operation_id: Option<String>,
    pub finding_kind: FindingKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    MissingOnMutating,
    BelowFloorOnSensitivePath,
    UnknownValue,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StepUpAuthCoverageReport {
    pub spec_source: String,
    pub findings: Vec<Finding>,
    pub operations_inspected: usize,
    pub operations_with_declaration: usize,
}

impl StepUpAuthCoverageReport {
    pub fn ok(&self) -> bool {
        self.findings.is_empty()
    }
}

const KNOWN_ACR: &[&str] = &["routine", "elevated", "sensitive", "critical"];

/// Run the gate against a YAML or JSON OpenAPI 3.x spec body.
pub fn scan(
    spec_source: impl Into<String>,
    body: &str,
) -> Result<StepUpAuthCoverageReport, ScanError> {
    let spec: serde_yaml::Value = serde_yaml::from_str(body)
        .map_err(|e| ScanError::Parse(format!("yaml/json parse: {e}")))?;
    let spec_source = spec_source.into();

    let paths = spec
        .get("paths")
        .and_then(|p| p.as_mapping())
        .ok_or_else(|| ScanError::Parse("missing top-level `paths`".to_owned()))?;

    let mut findings = Vec::new();
    let mut ops_inspected = 0usize;
    let mut ops_declared = 0usize;

    for (path_key, path_item) in paths {
        let Some(path_str) = path_key.as_str() else {
            continue;
        };
        let Some(item_map) = path_item.as_mapping() else {
            continue;
        };
        for (method_key, op) in item_map {
            let Some(method_str) = method_key.as_str() else {
                continue;
            };
            let method_lc = method_str.to_ascii_lowercase();
            if !MUTATING_METHODS.contains(&method_lc.as_str())
                && !matches!(method_lc.as_str(), "get" | "head" | "options")
            {
                continue;
            }
            if !MUTATING_METHODS.contains(&method_lc.as_str()) {
                // read-only methods are inspected only to count declarations,
                // never required to declare.
                ops_inspected += 1;
                if op_has_acr_decl(op) {
                    ops_declared += 1;
                }
                continue;
            }
            ops_inspected += 1;
            let op_id = op
                .get("operationId")
                .and_then(|v| v.as_str())
                .map(String::from);
            let exempt = op
                .get("x-acr-exempt")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if exempt {
                continue;
            }
            let acr = op.get("x-acr-required").and_then(|v| v.as_str());
            let path_is_sensitive = is_sensitive_path(path_str);
            match acr {
                None => {
                    findings.push(Finding {
                        path: path_str.to_owned(),
                        method: method_lc.clone(),
                        operation_id: op_id.clone(),
                        finding_kind: FindingKind::MissingOnMutating,
                        message: format!(
                            "operation {method_lc} {path_str} mutates state but lacks `x-acr-required`. Declare one of: routine, elevated, sensitive, critical (per ADR-0189)."
                        ),
                    });
                }
                Some(v) if !KNOWN_ACR.contains(&v) => {
                    ops_declared += 1;
                    findings.push(Finding {
                        path: path_str.to_owned(),
                        method: method_lc.clone(),
                        operation_id: op_id.clone(),
                        finding_kind: FindingKind::UnknownValue,
                        message: format!(
                            "operation {method_lc} {path_str} declares `x-acr-required: {v}` — not in canonical enum (routine|elevated|sensitive|critical)."
                        ),
                    });
                }
                Some(v) => {
                    ops_declared += 1;
                    if path_is_sensitive && (v == "routine" || v == "elevated") {
                        findings.push(Finding {
                            path: path_str.to_owned(),
                            method: method_lc.clone(),
                            operation_id: op_id.clone(),
                            finding_kind: FindingKind::BelowFloorOnSensitivePath,
                            message: format!(
                                "operation {method_lc} {path_str} touches a sensitive resource (path matches sensitive heuristic) but declares only `{v}`. Floor for this class is `sensitive` (per ADR-0189)."
                            ),
                        });
                    }
                }
            }
        }
    }

    Ok(StepUpAuthCoverageReport {
        spec_source,
        findings,
        operations_inspected: ops_inspected,
        operations_with_declaration: ops_declared,
    })
}

fn is_sensitive_path(p: &str) -> bool {
    let lc = p.to_ascii_lowercase();
    SENSITIVE_PATH_NEEDLES.iter().any(|n| lc.contains(n))
}

fn op_has_acr_decl(op: &serde_yaml::Value) -> bool {
    op.get("x-acr-required").and_then(|v| v.as_str()).is_some()
}

#[derive(Debug)]
pub enum ScanError {
    Parse(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(s) => write!(f, "step-up-auth-coverage parse: {s}"),
        }
    }
}

impl std::error::Error for ScanError {}
