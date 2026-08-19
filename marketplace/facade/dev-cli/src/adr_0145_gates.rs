//! Gate dispatcher wiring for PR #143 Fix-D + ADR-0145 enforcement gates.
//!
//! Five gates land here:
//!
//! 1. `high-risk-auto-decision-refusal` — strict; SEC-MAJ-02.
//! 2. `slsa-l3-evidence-grounded` — strict; SEC-MAJ-01.
//! 3. `otel-trace-propagation` — DEFERRED (advisory); ADR-0145 Invariant 2.
//! 4. `ontology-projection-coverage` — strict; ADR-0145 Invariant 3.
//! 5. `audit-chain-seal-coverage` — DEFERRED (advisory); ADR-0145 Invariant 1.
//!
//! The advisory-mode gates return SUCCESS even when findings exist; the
//! findings are surfaced in stdout for traceability. Strict-mode
//! promotion is tracked under
//! `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-*`.
//!
//! The runner I/O for each gate scans canonical default paths in the
//! repo; arguments are accepted but optional. Tests cover the parse +
//! dispatch surface; integration with real on-disk content is exercised
//! via `tests/gate_cli.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_audit_chain_seal_coverage as audit_chain_check;
use check_high_risk_auto_decision_refusal as high_risk_refusal_check;
use check_ontology_projection_coverage as ontology_check;
use check_otel_trace_propagation as otel_check;
use check_slsa_l3_evidence_grounded as slsa_check;

use crate::service_roots::{self, ServiceSubpath};

const DEFAULT_WORKFLOWS_DIR: &str = ".github/workflows";

// ---------- Common scanning helpers ----------

/// Resolve the roots this gate scans: the explicit `--microservices-root`
/// when given, otherwise the shared registry-derived default set.
///
/// The default path FAILS when an expected root is absent. That is the
/// point: these gates previously defaulted to a hardcoded
/// `["cloud", "oya", "microservices"]`, two thirds of which no longer
/// existed, and the absence was swallowed into an empty — green — scan.
fn resolve_roots(args: &[String]) -> Result<Vec<PathBuf>, String> {
    match parse_flag_with_value(args, "--microservices-root") {
        Some(explicit) => Ok(vec![PathBuf::from(explicit)]),
        None => service_roots::default_service_roots(),
    }
}

fn read_optional_string(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn list_microservice_subpaths(root: &Path, relative: &str) -> Vec<ServiceSubpath> {
    service_roots::list_service_subpaths(root, relative)
}

/// Collect all first-level service directories from a list of root paths.
fn service_dirs_from_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                out.push(p);
            }
        }
    }
    out
}

fn parse_flag_with_value(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == flag {
            return iter.next().cloned();
        }
        if let Some(value) = a.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_string());
        }
    }
    None
}

// ---------- Gate 1: high-risk-auto-decision-refusal ----------

pub(crate) fn run_high_risk_auto_decision_refusal(args: Vec<String>) -> ExitCode {
    let roots = match resolve_roots(&args) {
        Ok(roots) => roots,
        Err(error) => {
            eprintln!("high-risk-auto-decision-refusal: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut capabilities = Vec::new();
    for root in &roots {
        for subpath in list_microservice_subpaths(root, "capabilities") {
            let Ok(entries) = fs::read_dir(&subpath.path) else {
                continue;
            };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("yaml") {
                    continue;
                }
                if let Some(contents) = read_optional_string(&p) {
                    capabilities.push(high_risk_refusal_check::CapabilityDocument {
                        path: p.to_string_lossy().to_string(),
                        microservice: subpath.microservice.clone(),
                        contents,
                    });
                }
            }
        }
    }
    let mut cedar_fragments = Vec::new();
    for root in &roots {
        for subpath in list_microservice_subpaths(root, "policy") {
            let Ok(entries) = fs::read_dir(&subpath.path) else {
                continue;
            };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("cedar") {
                    continue;
                }
                if let Some(contents) = read_optional_string(&p) {
                    cedar_fragments.push(high_risk_refusal_check::CedarPolicyDocument {
                        path: p.to_string_lossy().to_string(),
                        microservice: subpath.microservice.clone(),
                        contents,
                    });
                }
            }
        }
    }
    let (report, violations) =
        high_risk_refusal_check::audit_all_violations(capabilities, cedar_fragments);
    println!(
        "high-risk-auto-decision-refusal: {} capabilities, {} claims, {} cedar fragments, {} µservices",
        report.capabilities_checked,
        report.claims_found,
        report.cedar_fragments_checked,
        report.microservices_audited,
    );
    if violations.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "high-risk-auto-decision-refusal: {} violations:",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {v}");
        }
        ExitCode::FAILURE
    }
}

// ---------- Gate 2: slsa-l3-evidence-grounded ----------

pub(crate) fn run_slsa_l3_evidence_grounded(args: Vec<String>) -> ExitCode {
    let roots = match resolve_roots(&args) {
        Ok(roots) => roots,
        Err(error) => {
            eprintln!("slsa-l3-evidence-grounded: {error}");
            return ExitCode::FAILURE;
        }
    };
    let workflows_dir = PathBuf::from(
        parse_flag_with_value(&args, "--workflows-dir")
            .unwrap_or_else(|| DEFAULT_WORKFLOWS_DIR.to_string()),
    );

    let mut scorecards = Vec::new();
    for root in &roots {
        for subpath in list_microservice_subpaths(root, "scorecards") {
            let overrides = subpath.path.join("overrides.json");
            if let Some(contents) = read_optional_string(&overrides) {
                scorecards.push(slsa_check::ScorecardOverrideDocument {
                    path: overrides.to_string_lossy().to_string(),
                    microservice: subpath.microservice.clone(),
                    contents,
                });
            }
        }
    }
    let mut workflows = Vec::new();
    if let Ok(entries) = fs::read_dir(&workflows_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("yml") {
                continue;
            }
            if let Some(contents) = read_optional_string(&p) {
                // Use a canonical path the kernel can canonicalize.
                let canonical = format!(
                    ".github/workflows/{}",
                    p.file_name().unwrap_or_default().to_string_lossy()
                );
                workflows.push(slsa_check::WorkflowDocument {
                    path: canonical,
                    contents,
                });
            }
        }
    }

    // ADR-0361: feed the Jenkins-native SLSA grounding (the shared CI lane + the
    // captured signing evidence) so the canonical citations resolve once the
    // GitHub Actions workflows are retired.
    for jenkins_path in [
        "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy",
        "evidence/ci/slsa/README.md",
    ] {
        if let Some(contents) = read_optional_string(&PathBuf::from(jenkins_path)) {
            workflows.push(slsa_check::WorkflowDocument {
                path: jenkins_path.to_string(),
                contents,
            });
        }
    }

    let (report, violations) = slsa_check::audit_all_violations(scorecards, workflows);
    println!(
        "slsa-l3-evidence-grounded: {} scorecards, {} citations, {} workflows inspected, {} µservices",
        report.scorecards_checked,
        report.citations_checked,
        report.workflows_inspected,
        report.microservices_audited,
    );
    if violations.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "slsa-l3-evidence-grounded: {} violations:",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {v}");
        }
        ExitCode::FAILURE
    }
}

// ---------- Gate 3: otel-trace-propagation (advisory) ----------

pub(crate) fn run_otel_trace_propagation(args: Vec<String>) -> ExitCode {
    let crates_root = PathBuf::from(
        parse_flag_with_value(&args, "--crates-root").unwrap_or_else(|| "crates".to_string()),
    );

    let mut adapters: Vec<otel_check::ClientAdapterSource> = Vec::new();
    let Ok(crate_entries) = fs::read_dir(&crates_root) else {
        println!(
            "otel-trace-propagation: advisory mode — no crates/ root readable at {}",
            crates_root.display()
        );
        return ExitCode::SUCCESS;
    };

    for entry in crate_entries.flatten() {
        let p = entry.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        // Match `oya-*-adapter-grpc-*` (canonical sibling-call adapter naming).
        if !name.contains("-adapter-grpc-") {
            continue;
        }
        let microservice = name
            .strip_prefix("oya-")
            .and_then(|n| n.split('-').next())
            .unwrap_or("unknown")
            .to_string();
        let src = p.join("src").join("lib.rs");
        if let Some(contents) = read_optional_string(&src) {
            adapters.push(otel_check::ClientAdapterSource {
                path: src.to_string_lossy().to_string(),
                microservice,
                contents,
            });
        }
    }

    let report = otel_check::validate_advisory(adapters);
    println!(
        "otel-trace-propagation (DEFERRED/advisory per ADR-0145): {} adapters, {} compliant, {} µservices, {} findings",
        report.adapters_checked,
        report.adapters_with_propagation,
        report.microservices_audited,
        report.advisory_findings.len(),
    );
    for finding in &report.advisory_findings {
        println!("  advisory: {finding}");
    }
    println!(
        "(advisory mode; strict-mode promotion tracked under \
         registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-otel-propagation-validator)"
    );
    ExitCode::SUCCESS
}

// ---------- Gate 4: ontology-projection-coverage (strict) ----------

pub(crate) fn run_ontology_projection_coverage(args: Vec<String>) -> ExitCode {
    let roots = match resolve_roots(&args) {
        Ok(roots) => roots,
        Err(error) => {
            eprintln!("ontology-projection-coverage: {error}");
            return ExitCode::FAILURE;
        }
    };

    let manifests = collect_manifests_from_roots(&roots);
    let advisory_inputs: Vec<ontology_check::ManifestDocument> = manifests
        .iter()
        .map(|m| ontology_check::ManifestDocument {
            path: m.path.clone(),
            microservice: m.microservice.clone(),
            contents: m.contents.clone(),
        })
        .collect();
    let report = ontology_check::validate_strict(advisory_inputs);
    println!(
        "ontology-projection-coverage (strict per ADR-0145): {} manifests, {} with projections, {} canonical-entity owners, {} projections, {} findings",
        report.manifests_checked,
        report.manifests_with_projections,
        report.manifests_owning_entities,
        report.projections_checked,
        report.strict_findings.len(),
    );
    for finding in &report.strict_findings {
        println!("  blocker: {finding}");
    }
    if report.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

// ---------- Gate 5: audit-chain-seal-coverage (advisory) ----------

pub(crate) fn run_audit_chain_seal_coverage(args: Vec<String>) -> ExitCode {
    let roots = match resolve_roots(&args) {
        Ok(roots) => roots,
        Err(error) => {
            eprintln!("audit-chain-seal-coverage: {error}");
            return ExitCode::FAILURE;
        }
    };
    let manifests = collect_manifests_from_roots(&roots);
    let advisory_inputs: Vec<audit_chain_check::ManifestDocument> = manifests
        .iter()
        .map(|m| audit_chain_check::ManifestDocument {
            path: m.path.clone(),
            microservice: m.microservice.clone(),
            contents: m.contents.clone(),
        })
        .collect();
    let report = audit_chain_check::validate_advisory(advisory_inputs);
    println!(
        "audit-chain-seal-coverage (DEFERRED/advisory per ADR-0145): {} manifests, {} audit_enabled, {} with seals, {} findings",
        report.manifests_checked,
        report.manifests_with_audit_enabled,
        report.manifests_with_seal_events,
        report.advisory_findings.len(),
    );
    for finding in &report.advisory_findings {
        println!("  advisory: {finding}");
    }
    println!(
        "(advisory mode; strict-mode promotion tracked under \
         registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-audit-chain-seal-validator)"
    );
    ExitCode::SUCCESS
}

// ---------- Shared manifest collection ----------

struct ManifestEntry {
    path: String,
    microservice: String,
    contents: String,
}

fn collect_manifests_from_roots(roots: &[PathBuf]) -> Vec<ManifestEntry> {
    let mut out = Vec::new();
    for root in roots {
        for manifest in service_roots::list_service_files(root, "manifest.json") {
            if let Some(contents) = read_optional_string(&manifest.path) {
                out.push(ManifestEntry {
                    path: manifest.path.to_string_lossy().to_string(),
                    microservice: manifest.microservice,
                    contents,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flag_with_value_handles_separated_form() {
        let args = vec!["--microservices-root".into(), "x".into()];
        assert_eq!(
            parse_flag_with_value(&args, "--microservices-root"),
            Some("x".to_string())
        );
    }

    #[test]
    fn parse_flag_with_value_handles_equals_form() {
        let args = vec!["--microservices-root=x".into()];
        assert_eq!(
            parse_flag_with_value(&args, "--microservices-root"),
            Some("x".to_string())
        );
    }

    #[test]
    fn parse_flag_with_value_returns_none_when_absent() {
        let args = vec!["--other".into()];
        assert_eq!(parse_flag_with_value(&args, "--microservices-root"), None);
    }

    /// Both LIVE path shapes must yield a microservice name.
    ///
    /// This test previously asserted on
    /// `microservices/tasks/capabilities/T2-auto.yaml` — a path that has not
    /// existed since the `microservices/` tree was renamed away. It passed
    /// against a world that was gone, and so proved nothing while the
    /// production lookup returned `None` for every real path in the tree
    /// and every document collapsed into the empty-string microservice.
    #[test]
    fn microservice_name_for_extracts_both_live_shapes() {
        // Depth-2: <root>/<service>/capabilities — the service segment.
        assert_eq!(
            service_roots::microservice_name_for(&PathBuf::from("workflow/tasks/capabilities")),
            Some("tasks".into())
        );
        // Depth-1: <root>/capabilities — the capability root owns them.
        assert_eq!(
            service_roots::microservice_name_for(&PathBuf::from("marketplace/capabilities")),
            Some("marketplace".into())
        );
    }

    /// The pairing key must actually tie a capability document to the Cedar
    /// fragment sitting beside it. Empty-string names (the pre-fix
    /// behaviour) collapse the entire repository into one bucket and
    /// degrade the gate from "does THIS microservice's claim have a
    /// matching forbid rule in THIS microservice's policy?" to "does any
    /// forbid rule exist anywhere?".
    #[test]
    fn capability_and_policy_dirs_of_one_service_share_a_microservice_key() {
        let capability_key =
            service_roots::microservice_name_for(&PathBuf::from("workflow/tasks/capabilities"));
        let policy_key =
            service_roots::microservice_name_for(&PathBuf::from("workflow/tasks/policy"));
        assert_eq!(capability_key, policy_key);
        assert_eq!(capability_key, Some("tasks".into()));
        assert!(capability_key.is_some_and(|k| !k.is_empty()));
    }

    #[test]
    fn collect_manifests_on_nonexistent_roots_returns_empty() {
        let roots = vec![PathBuf::from("/nonexistent/path")];
        let manifests = collect_manifests_from_roots(&roots);
        assert!(manifests.is_empty());
    }
}
