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
    assert_eq!(evaluation.security_backlog_count, 40);
    assert_eq!(evaluation.tracked_typescript_pnpm_mjs_count, 0);
    assert_eq!(evaluation.tracked_nonvendored_python_shell_count, 37);
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
fn spec_rejects_missing_kubernetes_native_antipattern_tool_example() {
    let spec = read_repo_file("specs/repo-hygiene-automation.json").replace(
        "\"buck2 build //:kubernetes-native-anti-pattern-check\"",
        "\"buck2 build //:kubernetes-native-anti-pattern-check-removed\"",
    );
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure
                .contains("active context drift scan missing required tool example")),
        "{:?}",
        failures
    );
}

#[test]
fn spec_rejects_pnpm_or_typescript_as_repo_authority() {
    let spec = read_repo_file("specs/repo-hygiene-automation.json")
        .replace(
            "\"pnpm_or_package_json_repo_authority\": false",
            "\"pnpm_or_package_json_repo_authority\": true",
        )
        .replace(
            "\"typescript_runtime_merge_authority\": false",
            "\"typescript_runtime_merge_authority\": true",
        );
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("pnpm/package metadata must not be repo authority")),
        "{:?}",
        failures
    );
    assert!(
        failures.iter().any(|failure| failure
            .contains("TypeScript runtime surfaces must not exist or be merge authority")),
        "{:?}",
        failures
    );
}

#[test]
fn active_policy_context_name_scanner_rejects_provenance_token_context_fields() {
    let root = temp_dir("active-policy-context-name-bad");
    let policy_dir = root.join("oya/example/cedar");
    fs::create_dir_all(&policy_dir).unwrap_or_else(|error| {
        panic!("create policy dir {}: {}", policy_dir.display(), error);
    });
    fs::write(
        policy_dir.join("policies.cedar"),
        r#"permit(principal, action, resource) when {
  context.doctrine.adr_0513 == true
};"#,
    )
    .unwrap_or_else(|error| panic!("write policy fixture: {error}"));

    let failures = gate::active_policy_context_name_failures(&root);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("active policy context field must be capability-named")),
        "{failures:?}"
    );
}

#[test]
fn active_policy_context_name_scanner_accepts_capability_named_context_fields() {
    let root = temp_dir("active-policy-context-name-good");
    let policy_dir = root.join("cloud/example/cedar");
    fs::create_dir_all(&policy_dir).unwrap_or_else(|error| {
        panic!("create policy dir {}: {}", policy_dir.display(), error);
    });
    fs::write(
        policy_dir.join("policies.cedar"),
        r#"permit(principal, action, resource) when {
  context.doctrine.buck2_prow_ci_authority == true
};"#,
    )
    .unwrap_or_else(|error| panic!("write policy fixture: {error}"));

    let failures = gate::active_policy_context_name_failures(&root);
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn tenant_rbac_packaging_rejects_retired_local_grouping_gates() {
    let packaging = read_repo_file("specs/tenant-rbac-packaging.json")
        .replace(
            "\"buck2 build //:retired-grouping-wording-check\"",
            &format!(
                "\"{}\"",
                ["scripts/", "reject-retired-grouping-wording.sh ."].concat()
            ),
        )
        .replace(
            "\"buck2 build //libs/oya-check-no-grouping:no-grouping-kernel-check\"",
            &format!(
                "\"{}\"",
                ["cargo", " test -p oya-check-no-grouping"].concat()
            ),
        );
    let failures = gate::tenant_rbac_packaging_failures(&packaging);
    for expected in [
        "retired shell grouping-wording gate",
        "Buck2 no-grouping kernel check",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn typescript_pnpm_inventory_matches_checked_in_surface() {
    let inventory = read_repo_file("registry/repo-hygiene/typescript-pnpm-surface-inventory.json");
    let files = gate::tracked_typescript_pnpm_mjs_files(Path::new(&repo_root()))
        .expect("TypeScript/pnpm surface scan should run");
    assert_eq!(files.len(), 0, "{files:?}");
    let (_count, failures) =
        gate::typescript_pnpm_surface_failures(Path::new(&repo_root()), &inventory);
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn typescript_pnpm_inventory_rejects_missing_current_file() {
    let inventory = read_repo_file("registry/repo-hygiene/typescript-pnpm-surface-inventory.json")
        .replace("\"tracked_file_count\": 0", "\"tracked_file_count\": 1");
    let (_count, failures) =
        gate::typescript_pnpm_surface_failures(Path::new(&repo_root()), &inventory);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("must record tracked_file_count 0")),
        "{failures:?}"
    );
}

#[test]
fn typescript_pnpm_surface_scan_excludes_vendored_agent_skills() {
    let root = temp_dir("typescript-pnpm-vendor-exclusion");
    fs::create_dir_all(root.join("tools/agent-skills/scripts")).unwrap_or_else(|error| {
        panic!("create vendored fixture dir: {error}");
    });
    fs::create_dir_all(root.join("scripts")).unwrap_or_else(|error| {
        panic!("create scripts fixture dir: {error}");
    });
    fs::write(
        root.join("tools/agent-skills/scripts/validate-skills.js"),
        "console.log('vendored');\n",
    )
    .unwrap_or_else(|error| panic!("write vendored fixture: {error}"));
    fs::write(
        root.join("scripts/generate-contract-docs.mjs"),
        "console.log('owned');\n",
    )
    .unwrap_or_else(|error| panic!("write owned fixture: {error}"));

    let files = gate::tracked_typescript_pnpm_mjs_files(&root)
        .expect("fixture TypeScript/pnpm scan should run");
    let _ = fs::remove_dir_all(&root);
    assert_eq!(
        files,
        vec!["scripts/generate-contract-docs.mjs".to_string()]
    );
}

#[test]
fn python_shell_inventory_matches_checked_in_surface() {
    let inventory = read_repo_file("registry/repo-hygiene/python-shell-surface-inventory.json");
    let files = gate::tracked_python_shell_files(Path::new(&repo_root()))
        .expect("Python/shell surface scan should run");
    assert_eq!(files.len(), 37, "{files:?}");
    let (_count, failures) =
        gate::python_shell_surface_failures(Path::new(&repo_root()), &inventory);
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn python_shell_inventory_rejects_missing_current_file() {
    let inventory = read_repo_file("registry/repo-hygiene/python-shell-surface-inventory.json")
        .replace(
            "tools/hooks/no-cargo-enforcer.sh",
            "tools/hooks/no-cargo-enforcer.rs",
        );
    let (_count, failures) =
        gate::python_shell_surface_failures(Path::new(&repo_root()), &inventory);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("tools/hooks/no-cargo-enforcer.sh")),
        "{failures:?}"
    );
}

#[test]
fn python_shell_surface_scan_excludes_vendored_surfaces() {
    let root = temp_dir("python-shell-vendor-exclusion");
    fs::create_dir_all(root.join("tools/agent-skills/scripts")).unwrap_or_else(|error| {
        panic!("create vendored fixture dir: {error}");
    });
    fs::create_dir_all(root.join("third-party/tooling")).unwrap_or_else(|error| {
        panic!("create third-party fixture dir: {error}");
    });
    fs::create_dir_all(root.join("scripts")).unwrap_or_else(|error| {
        panic!("create scripts fixture dir: {error}");
    });
    fs::write(
        root.join("tools/agent-skills/scripts/validate-skills.sh"),
        "echo vendored\n",
    )
    .unwrap_or_else(|error| panic!("write vendored fixture: {error}"));
    fs::write(
        root.join("third-party/tooling/upstream-helper.sh"),
        "echo third-party\n",
    )
    .unwrap_or_else(|error| panic!("write third-party fixture: {error}"));
    fs::write(root.join("scripts/pending-rewrite.py"), "print('owned')\n")
        .unwrap_or_else(|error| panic!("write owned fixture: {error}"));

    let files =
        gate::tracked_python_shell_files(&root).expect("fixture Python/shell scan should run");
    let _ = fs::remove_dir_all(&root);
    assert_eq!(files, vec!["scripts/pending-rewrite.py".to_string()]);
}

#[test]
fn spec_rejects_python_shell_as_durable_gate_authority() {
    let spec = read_repo_file("specs/repo-hygiene-automation.json").replace(
        "\"python_shell_durable_gate_authority\": false",
        "\"python_shell_durable_gate_authority\": true",
    );
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure
                .contains("Python/shell surfaces must not be durable gate authority")),
        "{:?}",
        failures
    );
}

#[test]
fn spec_rejects_missing_rust_and_buck2_pin_policy() {
    let spec = read_repo_file("specs/repo-hygiene-automation.json")
        .replace(
            "\"required_rust_stable\": \"1.96.0\"",
            "\"required_rust_stable\": \"1.95.0\"",
        )
        .replace(
            "\"required_buck2_release\": \"2026-06-01\"",
            "\"required_buck2_release\": \"2026-05-18\"",
        );
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("latest stable Rust pin")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("current Buck2 release pin")),
        "{:?}",
        failures
    );
}

#[test]
fn rust_toolchain_policy_rejects_stale_stable_or_edition() {
    let failures = gate::rust_toolchain_policy_failures(
        "channel = \"1.95.0\"",
        "[workspace.package]\nedition = \"2021\"\nrust-version = \"1.95.0\"\n",
        "rustc --edition=2021 example.rs",
        "{\"rust_toolchain\":\"1.95.0\"}",
        "channel = \"1.95.0\"",
        "Rust 1.95.0 edition 2021",
        "Rust 1.95.0 edition 2021",
        "Rust 1.95.0 edition 2021",
        "Rust 1.95.0 edition 2021",
    );
    for expected in [
        "rust-toolchain.toml",
        "Cargo.toml",
        "BUCK must not compile Rust checks with edition 2021",
        "specs/github-lane-unlocker-bridge.json",
        "specs/buck2-authority-policy.json",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn dependency_registry_policy_rejects_untracked_workspace_dependency() {
    let cargo =
        "[workspace.dependencies]\nserde = \"1\"\nfake-external = \"0.1\"\n[workspace.lints]\n";
    let rationales = r#"{"entries":{"serde":{}},"_meta":{"version_policy":"latest_upstream_stable_or_lts","tracking_policy":"all_workspace_dependencies_tracked","exception_policy":"explicit_waiver_required_for_non_latest_or_non_in_house_dependency","library_posture":"in_house_first_oya_rust_libraries"}}"#;
    let allowlist = r#"{"blessed":{"serde":{}},"_meta":{"version_policy":"latest_upstream_stable_or_lts","tracking_policy":"all_workspace_dependencies_tracked","exception_policy":"explicit_waiver_required_for_non_latest_or_non_in_house_dependency","library_posture":"in_house_first_oya_rust_libraries"}}"#;
    let failures = gate::dependency_registry_policy_failures(
        cargo,
        rationales,
        allowlist,
        "in-house latest registry/dependency-rationales.json",
        "in-house latest registry/dependency-rationales.json",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("missing workspace dependency fake-external")),
        "{:?}",
        failures
    );
}

#[test]
fn buck2_release_policy_rejects_stale_release_pin() {
    let failures = gate::buck2_release_policy_failures(
        "{\"required_buck2_release\":\"2026-05-18\"}",
        "env:\n  BUCK2_RELEASE: \"2026-05-18\"\n",
        ": \"${BUCK2_RELEASE:=2026-05-18}\"",
        "https://github.com/facebook/buck2.git",
        "genrule(name=\"latest-toolchain-pin-updater-check\")",
    );
    for expected in [
        "specs/repo-hygiene-automation.json",
        ".github/workflows/github-lane-unlocker-ci-cd.yml",
        "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
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
fn active_doc_phrase_scanner_rejects_retired_local_oya_cli_mirror_wording() {
    let failures = gate::active_doc_phrase_failures(
        "docs/ci/auto-merge-flow.md",
        "Local `oya verify`, local `oya gate`, Buck2 affected-only output, Cargo, and operator memory are not protected-branch authority.",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("Local `oya verify`, local `oya gate`")),
        "{:?}",
        failures
    );
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_local_oya_cli_output_labels() {
    let failures = gate::active_doc_phrase_failures(
        "specs/phase0-auto-merge-after-ci.json",
        r#"{"non_authority_surfaces":["local oya verify output","local oya gate output"]}"#,
    );
    for expected in ["local oya verify output", "local oya gate output"] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_exact_retired_oya_cli_pair() {
    let failures = gate::active_doc_phrase_failures(
        "docs/AGENTS.md",
        "Governance checks run elsewhere; the retired `oya gate` / `oya verify` CLI surfaces are not merge authority.",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("retired `oya gate` / `oya verify`")),
        "{:?}",
        failures
    );
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_oya_cli_list() {
    let failures = gate::active_doc_phrase_failures(
        "AGENTS.md",
        "retirement_note: the `oya git`, `oya vcs`, `oya gate`, and `oya verify` CLI surfaces are retired as merge authorities.",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("`oya git`, `oya vcs`, `oya gate`")),
        "{:?}",
        failures
    );
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_oya_vcs_projection_commands() {
    let failures = gate::active_doc_phrase_failures(
        "specs/agent-operating-contract.json",
        r#"{"required_sequence":["claim with oya vcs before edits where project guidance requires it","done and promote through oya vcs when ready"],"observability_hooks":["oya vcs status","oya vcs verify evidence strings"],"sanctioned_primitives":["oya-git","oya-vcs","oya-vcs-admission"]}"#,
    );
    for expected in [
        "claim with oya vcs before edits",
        "done and promote through oya vcs",
        "oya vcs status",
        "oya vcs verify evidence strings",
        "oya-git",
        "oya-vcs",
        "oya-vcs-admission",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_tenant_rbac_gate_refs() {
    let failures = gate::active_doc_phrase_failures(
        "specs/microservices/tenant-rbac.json",
        "test_ref: oya gate validate planning-closure; also oya gate validate product-prd-json",
    );
    for expected in [
        "oya gate validate planning-closure",
        "oya gate validate product-prd-json",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_brief_template_lifecycle() {
    let failures = gate::active_doc_phrase_failures(
        "docs/standards/brief-template.md",
        "Which PR, Jenkins contexts, and Jenkins governance lifecycle? ./bin/oya verify --ci-required && ./bin/oya gate run-all",
    );
    for expected in [
        "Jenkins contexts",
        "Jenkins governance lifecycle",
        "./bin/oya verify --ci-required",
        "./bin/oya gate run-all",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_agentic_dev_team_gate_refs() {
    let failures = gate::active_doc_phrase_failures(
        "docs/standards/agentic-dev-team-optimization.md",
        "Verification: oya gate run-all plus oya gate validate audit-chain-coverage --microservice demo via oya-dev-cli",
    );
    for expected in ["oya gate run-all", "oya gate validate", "oya-dev-cli"] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_canonical_prd_ci_cd_refs() {
    let failures = gate::active_doc_phrase_failures(
        "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
        "Agent workflow has Jenkins required checks, `oya gate` / `oya verify`, reviewer/governance approval, and reviewer/governance lifecycle.",
    );
    for expected in [
        "`oya gate` / `oya verify`",
        "reviewer/governance approval",
        "reviewer/governance lifecycle",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn retired_exact_name_scanner_rejects_retired_canonical_prd_substrate_names() {
    let failures = gate::retired_exact_name_failures(
        "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
        "CI uses Jenkins LTS and CD uses Argo CD sync.",
    );
    assert!(
        failures.iter().any(|failure| failure
            .contains("docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md:1: retired exact-name reference")),
        "{:?}",
        failures
    );
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_doc_catalog_gate_refs() {
    let failures = gate::active_doc_phrase_failures(
        "docs/DOC-CATALOG.md",
        "codeview read surface uses oya gate validate codeview-read-surface and active oya gate run-all commands stay mirrored.",
    );
    for expected in ["oya gate validate", "oya gate run-all"] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_ci_lanes_local_cli_refs() {
    let failures = gate::active_doc_phrase_failures(
        "docs/standards/ci-lanes.md",
        "The oya verify command maps local checks, active lanes are invoked from oya gate run-all, and authors run oya gate validate quality-lanes.",
    );
    for expected in [
        "oya verify command",
        "oya gate run-all",
        "oya gate validate",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_standalone_local_tool_loops() {
    let failures = gate::active_doc_phrase_failures(
        "docs/QA-TEST-STRATEGY.md",
        "Use bacon for feedback and cargo-machete for dependency sweeps before PRs.",
    );
    for expected in ["bacon", "cargo-machete"] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_active_process_doc_refs() {
    let failures = gate::active_doc_phrase_failures(
        "docs/AGENTS-OPERATING-CONTRACT.md",
        "Run ./bin/oya verify --ci-required and oya gate run-all before asking for reviewer/governance approval.",
    );
    for expected in [
        "./bin/oya verify --ci-required",
        "oya gate run-all",
        "reviewer/governance approval",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_doc_phrase_scanner_rejects_retired_vendor_and_raci_gate_refs() {
    let failures = gate::active_doc_phrase_failures(
        "docs/VENDOR-PARTNER-LEDGER.md",
        "Use oya gate validate vendor-contract-recency and oya verify command output.",
    );
    for expected in ["oya gate validate", "oya verify command"] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn retired_exact_name_scanner_rejects_retired_release_substrate_names() {
    let failures = gate::retired_exact_name_failures(
        "docs/RELEASE-MANAGEMENT.md",
        "Coordinate through Jenkins and Argo CD for active release authority.",
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
fn active_doc_phrase_scanner_is_case_insensitive() {
    let failures = gate::active_doc_phrase_failures(
        "example.md",
        "DEV REQUIRES GITHUB-LANE-UNLOCKER-REQUIRED before every merge.",
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("dev requires github-lane-unlocker-required")),
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
