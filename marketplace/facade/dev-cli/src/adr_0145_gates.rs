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

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_audit_chain_seal_coverage as audit_chain_check;
use check_high_risk_auto_decision_refusal as high_risk_refusal_check;
use check_ontology_projection_coverage as ontology_check;
use check_otel_trace_propagation as otel_check;
use check_slsa_l3_evidence_grounded as slsa_check;

/// Canonical service roots scanned when no explicit `--microservices-root` is given.
const DEFAULT_SERVICE_ROOTS: &[&str] = &["cloud", "oya", "microservices"];
const DEFAULT_WORKFLOWS_DIR: &str = ".github/workflows";

// ---------- Common scanning helpers ----------

fn microservice_name_for(path: &Path) -> Option<String> {
    // microservices/<ms>/... — return <ms>.
    let mut iter = path.components();
    while let Some(c) = iter.next() {
        if c.as_os_str() == OsStr::new("microservices")
            && let Some(next) = iter.next()
        {
            return Some(next.as_os_str().to_string_lossy().to_string());
        }
    }
    None
}

fn read_optional_string(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn list_microservice_subpaths(root: &Path, relative: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let candidate = p.join(relative);
        if candidate.exists() {
            out.push(candidate);
        }
    }
    out
}

/// Collect all first-level service directories from a list of root paths.
fn service_dirs_from_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else { continue; };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() { out.push(p); }
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
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };

    let mut capabilities = Vec::new();
    for root in &roots {
    for path in list_microservice_subpaths(root, "capabilities") {
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        let microservice = microservice_name_for(&path).unwrap_or_default();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            if let Some(contents) = read_optional_string(&p) {
                capabilities.push(high_risk_refusal_check::CapabilityDocument {
                    path: p.to_string_lossy().to_string(),
                    microservice: microservice.clone(),
                    contents,
                });
            }
        }
    }

    }
    let mut cedar_fragments = Vec::new();
    for root in &roots {
    for path in list_microservice_subpaths(root, "policy") {
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        let microservice = microservice_name_for(&path).unwrap_or_default();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("cedar") {
                continue;
            }
            if let Some(contents) = read_optional_string(&p) {
                cedar_fragments.push(high_risk_refusal_check::CedarPolicyDocument {
                    path: p.to_string_lossy().to_string(),
                    microservice: microservice.clone(),
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
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
    };
    let workflows_dir = PathBuf::from(
        parse_flag_with_value(&args, "--workflows-dir")
            .unwrap_or_else(|| DEFAULT_WORKFLOWS_DIR.to_string()),
    );

    let mut scorecards = Vec::new();
    for root in &roots {
    for path in list_microservice_subpaths(root, "scorecards") {
        let microservice = microservice_name_for(&path).unwrap_or_default();
        let overrides = path.join("overrides.json");
        if let Some(contents) = read_optional_string(&overrides) {
            scorecards.push(slsa_check::ScorecardOverrideDocument {
                path: overrides.to_string_lossy().to_string(),
                microservice,
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
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
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
    let explicit_root = parse_flag_with_value(&args, "--microservices-root");
    let roots: Vec<PathBuf> = if let Some(r) = explicit_root {
        vec![PathBuf::from(r)]
    } else {
        DEFAULT_SERVICE_ROOTS.iter().map(PathBuf::from).collect()
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
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let manifest = p.join("manifest.json");
            if let Some(contents) = read_optional_string(&manifest) {
                let microservice = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                out.push(ManifestEntry {
                    path: manifest.to_string_lossy().to_string(),
                    microservice,
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

    #[test]
    fn microservice_name_for_extracts_correctly() {
        let path = PathBuf::from("microservices/tasks/capabilities/T2-auto.yaml");
        assert_eq!(microservice_name_for(&path), Some("tasks".into()));
    }

    #[test]
    fn collect_manifests_on_nonexistent_roots_returns_empty() {
        let roots = vec![PathBuf::from("/nonexistent/path")];
        let manifests = collect_manifests_from_roots(&roots);
        assert!(manifests.is_empty());
    }
}
