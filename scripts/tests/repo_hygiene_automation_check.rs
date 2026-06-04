#![allow(dead_code)]

#[path = "../ci/assert-repo-hygiene-automation.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("read {}: {}", path, error);
    })
}

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "oyatie-repo-hygiene-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap_or_else(|error| {
        panic!("create temp dir {}: {}", path.display(), error);
    });
    path
}

#[test]
fn checked_in_repo_hygiene_contract_passes() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty());
    assert_eq!(evaluation.domains_checked, 6);
    assert_eq!(evaluation.security_backlog_count, 31);
}

#[test]
fn spec_rejects_reintroduced_python_hygiene_command() {
    let mut spec = read_repo_file("specs/repo-hygiene-automation.json");
    spec = spec.replace(
        "\"buck2 build //:repo-hygiene-automation-check\"",
        "\"python3 scripts/ci/assert-repo-hygiene-automation.py --json\"",
    );
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("retired repo-hygiene Python command")),
        "{:?}",
        failures
    );
}

#[test]
fn spec_rejects_missing_security_hardening_backlog_item() {
    let spec = read_repo_file("specs/repo-hygiene-automation.json").replace(
        "\"id\": \"service_mesh_mtls\"",
        "\"id\": \"service_mesh_mtls_removed\"",
    );
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "security_hardening_backlog missing service_mesh_mtls"),
        "{:?}",
        failures
    );
}

#[test]
fn checked_in_masterplan_surfaces_do_not_recommend_retired_cargo_gate() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert!(
        evaluation
            .failures
            .iter()
            .all(|failure| !failure.contains("retired Cargo planning-closure command")),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn root_jenkinsfile_is_rejected_as_retired_ci_entrypoint() {
    let root = temp_dir("root-jenkinsfile");
    fs::write(root.join("Jenkinsfile"), "pipeline {}\n").unwrap_or_else(|error| {
        panic!("write retired Jenkinsfile fixture: {}", error);
    });
    let failures = gate::retired_root_file_failures(&root);
    let _ = fs::remove_dir_all(&root);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("retired root CI entrypoint")),
        "{:?}",
        failures
    );
}

#[test]
fn service_jenkinsfiles_are_rejected_as_retired_ci_entrypoints() {
    let root = temp_dir("service-jenkinsfile");
    for rel in ["cloud/demo/ci", "oya/demo/ci"] {
        fs::create_dir_all(root.join(rel)).unwrap_or_else(|error| {
            panic!("create service ci fixture {}: {}", rel, error);
        });
        fs::write(root.join(rel).join("Jenkinsfile"), "pipeline {}\n").unwrap_or_else(|error| {
            panic!("write service Jenkinsfile fixture {}: {}", rel, error);
        });
    }
    let failures = gate::retired_service_ci_entrypoint_failures(&root);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(failures.len(), 2, "{:?}", failures);
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains("retired service Jenkins CI entrypoint")),
        "{:?}",
        failures
    );
}

#[test]
fn retired_active_ci_substrate_paths_are_rejected() {
    let root = temp_dir("retired-active-ci-substrate");
    for rel in [
        "infra/ci/jenkins",
        "infra/ci/argocd",
        "infra/cilium/cell-boundaries",
        "infra/forge",
    ] {
        fs::create_dir_all(root.join(rel)).unwrap_or_else(|error| {
            panic!("create retired active path fixture {}: {}", rel, error);
        });
    }
    for rel in [
        "infra/ci/deploy-local.sh",
        "infra/cilium/cell-boundaries/oya-ci-jenkins-ingress.netpol.yaml",
        "infra/cilium/cell-boundaries/oya-forge-ingress.netpol.yaml",
        "infra/forge/jenkins-forgejo-token.secret.template.yaml",
        "scripts/ci/arm-auto-merge.sh",
        "scripts/tests/forgejo_auto_merge_after_ci.test.sh",
        "docs/ci/forge-of-record.md",
    ] {
        if let Some(parent) = root.join(rel).parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                panic!(
                    "create retired active file parent {}: {}",
                    parent.display(),
                    error
                );
            });
        }
        fs::write(root.join(rel), "retired\n").unwrap_or_else(|error| {
            panic!("write retired active path fixture {}: {}", rel, error);
        });
    }
    let failures = gate::retired_active_path_failures(&root);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(failures.len(), 10, "{:?}", failures);
    assert!(
        failures
            .iter()
            .all(|failure| failure.contains("retired active CI substrate path")),
        "{:?}",
        failures
    );
}

#[test]
fn active_doc_phrase_scanner_rejects_manual_bridge_statuses() {
    let failures = gate::active_doc_phrase_failures(
        "example.md",
        "Agents may post manual oya-ci-required success statuses to merge bridge PRs.",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("manual oya-ci-required success statuses")),
        "{:?}",
        failures
    );
}

#[test]
fn retired_exact_name_scanner_requires_generic_active_doc_term() {
    let failures = gate::retired_exact_name_failures(
        "docs/live-procedure.md",
        "Use Jenkins as interim CI authority for dev.",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("retired exact-name reference")),
        "{:?}",
        failures
    );
}

#[test]
fn retired_exact_name_scanner_preserves_historical_adr_provenance() {
    let failures = gate::retired_exact_name_failures(
        "docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md",
        "Jenkins and Argo CD are historical names in this ADR.",
    );
    assert!(failures.is_empty(), "{:?}", failures);
}
