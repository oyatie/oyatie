//! Oya VCS admission-gate dev-CLI (Wave 3 replacement for
//! `scripts/check-oya-vcs-admission.sh`; audit row B-3).
//!
//! Composition root: reads filesystem artifacts, runs `cargo metadata`,
//! feeds typed inputs into the kernel, then executes the canonical
//! command-surface smoke (cargo test + `oya vcs` claim/verify/done/promote
//! against the admission cutover evidence).
//!
//! # Naming justification
//!
//! - `oya-vcs-admission-gate-app` —
//!   v4 BNF `oya-<product:foundry>-<topic:vcs-admission-gate>-<layer:app>`;
//!   13-value layer-enum suffix `app` (composition-root binary tool surface
//!   per ADR-0105 §"Amendment 2026-05-15 — `tools/` canonical-suffix binding").
//!   The `gate` token is a topic word, not a layer.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use oya_vcs_admission_gate_kernel::{AdmissionInputs, AdmissionReport, validate_admission};
use serde_json::Value;

const ROOT_HUB_POINTERS: &str = "specs/root-hub-pointers.json";
const MASTER_PLAN_SEQUENCING: &str = "specs/master-plan-sequencing.json";
const MULTISPECTRUM_REVIEW: &str = "specs/multispectrum-review.json";
const GITOPS_VCS_REPLACEMENT: &str = "specs/gitops-vcs-replacement.json";
const BRANCH_PROTECTION_YAML: &str = ".github/branch-protection.yaml";
// ADR-0361: the retired GitHub Actions workflows (pr-tests.yml +
// oya-governance-supply-chain.yml) are superseded by the single
// Jenkins-native pipeline lane. Both admission checks now read the
// canonical pipeline source, which declares the oya-vcs-admission and
// oya-vcs-provider-execution stages.
const PR_TESTS_WORKFLOW: &str = "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy";
const SUPPLY_CHAIN_WORKFLOW: &str = "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy";
const AUDIT_CHAIN_JSONL: &str = "evidence/audit-chain.jsonl";
const MULTISPECTRUM_EVIDENCE_DIR: &str = "evidence/multispectrum";
const VCS_ADMISSION_EVIDENCE: &str =
    "evidence/gitops-vcs/oya-vcs-admission-cutover-2026-05-15.json";

const VCS_PACKAGES: &[&str] = &[
    "oya-vcs-kernel",
    "oya-vcs-ast-index-kernel",
    "oya-vcs-lockstore-adapter",
    "oya-vcs-changebundle-kernel",
    "oya-vcs-polyglot-indexer-adapter",
    "oya-vcs-test-standard-gate-kernel",
    "oya-vcs-promotion-controller-kernel",
    "oya-vcs-review-mergequeue-kernel",
    "oya-vcs-cli-ratchet-kernel",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!(
                "oya-vcs-admission-gate-app: metadata and authority checks passed; cargo tests + CLI smoke passed"
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("check-oya-vcs-admission: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), GateError> {
    let root = env::current_dir().map_err(|e| GateError::Io(format!("cwd: {e}")))?;

    let root_hub_pointers = read_json(&root, ROOT_HUB_POINTERS)?;
    let master_plan_sequencing = read_json(&root, MASTER_PLAN_SEQUENCING)?;
    let multispectrum_review = read_json(&root, MULTISPECTRUM_REVIEW)?;
    let gitops_vcs_replacement = read_json(&root, GITOPS_VCS_REPLACEMENT)?;

    let provider_evidence_ref = gitops_vcs_replacement
        .get("current_ci_admission_lane")
        .and_then(|c| c.get("provider_evidence_ref"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let provider_evidence_path = provider_evidence_ref
        .split_once('#')
        .map(|(left, _)| left)
        .unwrap_or(provider_evidence_ref.as_str())
        .to_string();
    let provider_evidence = if provider_evidence_path.is_empty() {
        Value::Null
    } else {
        read_json(&root, &provider_evidence_path)?
    };

    let provider_execution_proof_ref = gitops_vcs_replacement
        .get("current_ci_admission_lane")
        .and_then(|c| c.get("provider_execution_proof_ref"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let provider_execution_proof_path = provider_execution_proof_ref
        .split_once('#')
        .map(|(left, _)| left)
        .unwrap_or(provider_execution_proof_ref.as_str())
        .to_string();
    let provider_execution_proof = if provider_execution_proof_path.is_empty() {
        Value::Null
    } else {
        read_json(&root, &provider_execution_proof_path)?
    };

    let branch_protection_yaml = read_text(&root, BRANCH_PROTECTION_YAML)?;
    let pr_tests_workflow = read_text(&root, PR_TESTS_WORKFLOW)?;
    let supply_chain_workflow = read_text(&root, SUPPLY_CHAIN_WORKFLOW)?;
    let audit_chain_jsonl = read_text(&root, AUDIT_CHAIN_JSONL)?;

    let workspace_packages = cargo_metadata_packages(&root)?;
    let multispectrum_evidence = load_multispectrum_evidence(&root)?;

    let inputs = AdmissionInputs {
        root_hub_pointers: &root_hub_pointers,
        master_plan_sequencing: &master_plan_sequencing,
        multispectrum_review: &multispectrum_review,
        gitops_vcs_replacement: &gitops_vcs_replacement,
        provider_evidence_ref: &provider_evidence_ref,
        provider_evidence: &provider_evidence,
        provider_execution_proof_ref: &provider_execution_proof_ref,
        provider_execution_proof: &provider_execution_proof,
        branch_protection_yaml: &branch_protection_yaml,
        pr_tests_workflow: &pr_tests_workflow,
        supply_chain_workflow: &supply_chain_workflow,
        workspace_packages: &workspace_packages,
        audit_chain_jsonl: &audit_chain_jsonl,
        multispectrum_evidence: &multispectrum_evidence,
    };

    let report = validate_admission(&inputs);
    fail_if_dirty(&report)?;

    // Provider-execution proof sub-call: the legacy script re-ran
    // `scripts/check-oya-vcs-provider-execution.sh --mode check` at this
    // point. Now invoked via the Rust binary; trivy + Argo manifest
    // validation happens inside that crate.
    run_command(
        &root,
        Command::new("cargo").args([
            "run",
            "-q",
            "-p",
            "oya-vcs-provider-execution-gate-app",
            "--",
            "--mode",
            "check",
        ]),
        "provider-execution-gate (--mode check)",
    )?;

    // Targeted Oya VCS package tests.
    let mut cargo_test_args: Vec<&str> = vec!["test"];
    for pkg in VCS_PACKAGES {
        cargo_test_args.push("-p");
        cargo_test_args.push(pkg);
    }
    run_command(
        &root,
        Command::new("cargo").args(&cargo_test_args),
        "cargo test (vcs packages)",
    )?;

    // Dev-CLI vcs subcommand tests.
    run_command(
        &root,
        Command::new("cargo").args(["test", "-p", "oya-dev-cli", "vcs"]),
        "cargo test -p oya-dev-cli vcs",
    )?;

    // Command-surface smoke (claim / verify / done / promote).
    smoke_vcs_command(
        &root,
        &[
            "vcs",
            "--format",
            "json",
            "claim",
            "--agent",
            "admission-gate",
            "--intent",
            "Oya VCS admission CLI smoke",
            "specs/gitops-vcs-replacement.json::foundry_agentic_pipeline_integration_plan",
        ],
    )?;
    smoke_vcs_command(
        &root,
        &[
            "vcs",
            "--format",
            "json",
            "verify",
            "--agent",
            "admission-gate",
            "--evidence",
            VCS_ADMISSION_EVIDENCE,
        ],
    )?;
    smoke_vcs_command(
        &root,
        &[
            "vcs",
            "--format",
            "json",
            "done",
            "--agent",
            "admission-gate",
            "--evidence",
            VCS_ADMISSION_EVIDENCE,
        ],
    )?;
    smoke_vcs_command(
        &root,
        &[
            "vcs",
            "--format",
            "json",
            "promote",
            "--agent",
            "admission-gate",
            "--bundle",
            "bundle_oya_vcs_admission_cutover",
            "--environment",
            "ci-preview",
            "--evidence",
            VCS_ADMISSION_EVIDENCE,
        ],
    )?;

    Ok(())
}

fn smoke_vcs_command(root: &Path, vcs_args: &[&str]) -> Result<(), GateError> {
    let mut args: Vec<&str> = vec!["run", "-q", "-p", "oya-dev-cli", "--"];
    args.extend_from_slice(vcs_args);
    let output = Command::new("cargo")
        .args(&args)
        .current_dir(root)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| GateError::Io(format!("spawning cargo run oya-dev-cli: {e}")))?;
    if !output.status.success() {
        return Err(GateError::CommandFailed {
            label: format!("oya-dev-cli {vcs_args:?}"),
            status: output.status.code(),
        });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<Value>(&stdout).map_err(|e| {
        GateError::Io(format!(
            "oya-dev-cli {vcs_args:?} did not emit valid JSON: {e}"
        ))
    })?;
    Ok(())
}

fn read_text(root: &Path, relative: &str) -> Result<String, GateError> {
    let path = root.join(relative);
    fs::read_to_string(&path).map_err(|e| GateError::Io(format!("read {}: {e}", path.display())))
}

fn read_json(root: &Path, relative: &str) -> Result<Value, GateError> {
    let text = read_text(root, relative)?;
    serde_json::from_str(&text).map_err(|e| GateError::Parse {
        path: relative.to_string(),
        detail: e.to_string(),
    })
}

fn cargo_metadata_packages(root: &Path) -> Result<Vec<String>, GateError> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| GateError::Io(format!("spawning cargo metadata: {e}")))?;
    if !output.status.success() {
        return Err(GateError::CommandFailed {
            label: "cargo metadata --no-deps".to_string(),
            status: output.status.code(),
        });
    }
    let parsed: Value = serde_json::from_slice(&output.stdout).map_err(|e| GateError::Parse {
        path: "<cargo metadata>".to_string(),
        detail: e.to_string(),
    })?;
    let names = parsed
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| GateError::Io("cargo metadata: missing packages array".to_string()))?
        .iter()
        .filter_map(|pkg| {
            pkg.get("name")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
        })
        .collect();
    Ok(names)
}

fn load_multispectrum_evidence(root: &Path) -> Result<Vec<(String, Value)>, GateError> {
    let dir = root.join(MULTISPECTRUM_EVIDENCE_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| GateError::Io(format!("read_dir {}: {e}", dir.display())))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.is_file() && p.extension().and_then(OsStr::to_str) == Some("json"))
        .collect();
    paths.sort();
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let text = fs::read_to_string(&path)
            .map_err(|e| GateError::Io(format!("read {}: {e}", path.display())))?;
        let parsed: Value = serde_json::from_str(&text).map_err(|e| GateError::Parse {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?;
        out.push((path.display().to_string(), parsed));
    }
    Ok(out)
}

fn fail_if_dirty(report: &AdmissionReport) -> Result<(), GateError> {
    if report.is_clean() {
        Ok(())
    } else {
        Err(GateError::Admission(report.clone()))
    }
}

fn run_command(root: &Path, cmd: &mut Command, label: &str) -> Result<(), GateError> {
    let status = cmd
        .current_dir(root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| GateError::Io(format!("spawning {label}: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(GateError::CommandFailed {
            label: label.to_string(),
            status: status.code(),
        })
    }
}

#[derive(Debug)]
enum GateError {
    Io(String),
    Parse { path: String, detail: String },
    Admission(AdmissionReport),
    CommandFailed { label: String, status: Option<i32> },
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GateError::Io(message) => write!(f, "{message}"),
            GateError::Parse { path, detail } => {
                write!(f, "could not parse {path}: {detail}")
            }
            GateError::Admission(report) => {
                writeln!(
                    f,
                    "admission gate FAILED ({} violation(s))",
                    report.violations.len()
                )?;
                for violation in &report.violations {
                    writeln!(f, "  - [{}] {}", violation.code, violation.detail)?;
                }
                Ok(())
            }
            GateError::CommandFailed { label, status } => match status {
                Some(code) => write!(f, "{label}: exited with status {code}"),
                None => write!(f, "{label}: terminated by signal"),
            },
        }
    }
}

impl std::error::Error for GateError {}
