#![allow(dead_code)]

#[path = "../ci/generate-oya-ci-prowjob-registry.rs"]
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
        "oyatie-prowjob-registry-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap_or_else(|error| {
        panic!("create temp dir {}: {}", path.display(), error);
    });
    path
}

fn copy_registry_fixture(root: &Path) {
    for rel in [
        "specs/oya-ci-prowjob-registry.json",
        "specs/root-hub-pointers.json",
        "specs/ci/prow-jobs/platform-governance.json",
        "specs/ci/prow-jobs/platform-scm-ci-cd.json",
        "specs/ci/prow-jobs/platform-release-conveyor.json",
        "specs/generated/oya-ci-prowjob-registry.generated.yaml",
        "specs/generated/oya-ci-controller-config.generated.yaml",
    ] {
        let destination = root.join(rel);
        fs::create_dir_all(destination.parent().unwrap()).unwrap_or_else(|error| {
            panic!("create fixture parent {}: {}", destination.display(), error);
        });
        fs::write(&destination, read_repo_file(rel)).unwrap_or_else(|error| {
            panic!("write fixture {}: {}", destination.display(), error);
        });
    }
}

#[test]
fn checked_in_prowjob_registry_passes() {
    let evaluation = gate::evaluate(&repo_root());
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty(), "{:?}", evaluation.failures);
    assert_eq!(evaluation.shards_checked, 3);
    assert!(evaluation.jobs_checked >= 7);
    assert_eq!(evaluation.required_context, "oya-ci-required");
    assert_eq!(
        evaluation.generated_controller_config,
        "specs/generated/oya-ci-controller-config.generated.yaml"
    );
    assert!(evaluation.local_static_only);
    assert!(!evaluation.live_authority_claimed);
}

#[test]
fn registry_requires_controller_owned_required_context() {
    let registry = read_repo_file("specs/oya-ci-prowjob-registry.json").replace(
        "\"controller_owned_required_context\": \"oya-ci-required\"",
        "\"controller_owned_required_context\": \"github-lane-unlocker-required\"",
    );
    let failures = gate::registry_text_failures(&registry);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("controller-owned oya-ci-required")),
        "{:?}",
        failures
    );
}

#[test]
fn shard_rejects_non_buck2_durable_job_command() {
    let mut failures = Vec::new();
    let shard = read_repo_file("specs/ci/prow-jobs/platform-governance.json").replace(
        "buck2 build //:repo-hygiene-automation-check",
        "scripts/ci/manual-status.sh oya-ci-required",
    );
    let jobs = gate::parse_shard(
        "specs/ci/prow-jobs/platform-governance.json",
        &shard,
        &mut failures,
    );
    assert!(!jobs.is_empty(), "fixture should still parse jobs");
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("non-Buck2 command is not allowed")),
        "{:?}",
        failures
    );
}

#[test]
fn generated_output_drift_is_rejected() {
    let root = temp_dir("generated-drift");
    copy_registry_fixture(&root);
    let generated_path = root.join("specs/generated/oya-ci-prowjob-registry.generated.yaml");
    let mut generated = fs::read_to_string(&generated_path).unwrap_or_else(|error| {
        panic!("read generated fixture: {}", error);
    });
    generated.push_str("# stale hand edit\n");
    fs::write(&generated_path, generated).unwrap_or_else(|error| {
        panic!("write generated fixture: {}", error);
    });

    let evaluation = gate::evaluate(&root);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains("generated output is stale")),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn generated_controller_config_drift_is_rejected() {
    let root = temp_dir("controller-config-drift");
    copy_registry_fixture(&root);
    let generated_path = root.join("specs/generated/oya-ci-controller-config.generated.yaml");
    let mut generated = fs::read_to_string(&generated_path).unwrap_or_else(|error| {
        panic!("read generated controller config fixture: {}", error);
    });
    generated.push_str("# stale hand edit\n");
    fs::write(&generated_path, generated).unwrap_or_else(|error| {
        panic!("write generated controller config fixture: {}", error);
    });

    let evaluation = gate::evaluate(&root);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains("generated controller config is stale")),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn generated_controller_config_requires_kubernetes_native_security_defaults() {
    let root = temp_dir("controller-config-security");
    copy_registry_fixture(&root);
    let generated_path = root.join("specs/generated/oya-ci-controller-config.generated.yaml");
    let generated = fs::read_to_string(&generated_path)
        .unwrap_or_else(|error| panic!("read generated controller config fixture: {}", error))
        .replace(
            "workloadIdentityRequired: true",
            "workloadIdentityRequired: false",
        )
        .replace(
            "nodeMetadataAccess: \"blocked\"",
            "nodeMetadataAccess: \"allowed\"",
        )
        .replace(
            "defaultDenyNetworkPolicy: true",
            "defaultDenyNetworkPolicy: false",
        );
    fs::write(&generated_path, generated).unwrap_or_else(|error| {
        panic!("write generated controller config fixture: {}", error);
    });

    let evaluation = gate::evaluate(&root);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains("generated controller config is stale")),
        "{:?}",
        evaluation.failures
    );
}
