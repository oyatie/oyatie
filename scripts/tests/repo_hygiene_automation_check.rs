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
    assert_eq!(evaluation.active_context_scan_files, 26);
    assert_eq!(evaluation.active_template_scan_files, 30);
    assert_eq!(evaluation.retired_exact_name_scan_files, 36);
}

#[test]
fn retired_cli_registry_specs_reject_active_command_authority() {
    let workspace_hygiene = read_repo_file("specs/workspace-hygiene.json").replace(
        "\"command\": \"buck2 build //:repo-hygiene-automation-check\"",
        "\"command\": \"oya gate validate workspace-hygiene\"",
    );
    let feature_flag = read_repo_file("specs/feature-flag-substrate-canonical.json").replace(
        "planned Rust/Buck2/Prow validator",
        "oya gate validate feature-flag-lifecycle",
    );
    let multi_region = read_repo_file("specs/multi-region-disposition-canonical.json")
        .replace(
            "planned Rust/Buck2/Prow validator: multi-region-disposition",
            "oya gate validate multi-region-disposition",
        )
        .replace(
            "planned Rust/Buck2/Prow validator: sovereign-tenant-pin",
            "oya gate validate sovereign-tenant-pin",
        );
    let microservice_migration = read_repo_file("specs/microservice-migration-tooling.json")
        .replace(
            "future Rust/Buck2/Prow migration job",
            "oya dev migrate-microservice --rollback",
        );
    let retired_vocabulary = read_repo_file("registry/vocabulary/retired.yaml").replace(
        "retired CLI remains tombstone/provenance only",
        "do not revive oya vcs",
    );
    let docs_pipeline = read_repo_file("registry/docs/pipeline.tsv")
        .replace("documentation capability: openapi", "oya doc openapi");
    let documentation_system_kernel =
        read_repo_file("libs/oya-check-documentation-system/src/lib.rs").replace(
            "documented_command must name a documentation capability",
            "documented_command must name an oya doc subcommand",
        );

    let failures = gate::retired_cli_registry_spec_failures(
        &workspace_hygiene,
        &feature_flag,
        &multi_region,
        &microservice_migration,
        &retired_vocabulary,
        &docs_pipeline,
        &documentation_system_kernel,
    );

    for expected in [
        "oya gate validate workspace-hygiene",
        "oya gate validate feature-flag-lifecycle",
        "oya gate validate multi-region-disposition",
        "oya gate validate sovereign-tenant-pin",
        "oya dev migrate-microservice --rollback",
        "do not revive oya vcs",
        "oya doc",
        "documented_command must name an oya doc subcommand",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn retired_cli_registry_specs_require_buck2_prow_replacements() {
    let workspace_hygiene = read_repo_file("specs/workspace-hygiene.json").replace(
        "\"cleanup_commands\": []",
        "\"cleanup_commands\": [\"manual-clean\"]",
    );
    let feature_flag = read_repo_file("specs/feature-flag-substrate-canonical.json")
        .replace("planned Rust/Buck2/Prow validator", "planned validator");
    let multi_region = read_repo_file("specs/multi-region-disposition-canonical.json")
        .replace(
            "planned Rust/Buck2/Prow validator: multi-region-disposition",
            "planned validator: multi-region-disposition",
        )
        .replace(
            "planned Rust/Buck2/Prow validator: sovereign-tenant-pin",
            "planned validator: sovereign-tenant-pin",
        );
    let microservice_migration = read_repo_file("specs/microservice-migration-tooling.json")
        .replace(
            "future Rust/Buck2/Prow migration job",
            "future migration job",
        );
    let retired_vocabulary = read_repo_file("registry/vocabulary/retired.yaml").replace(
        "retired CLI remains tombstone/provenance only",
        "retired CLI tombstone",
    );
    let docs_pipeline = read_repo_file("registry/docs/pipeline.tsv").replace(
        "documentation capability: openapi",
        "documentation generator: openapi",
    );
    let documentation_system_kernel =
        read_repo_file("libs/oya-check-documentation-system/src/lib.rs")
            .replace("documentation capability: ", "documentation generator: ");

    let failures = gate::retired_cli_registry_spec_failures(
        &workspace_hygiene,
        &feature_flag,
        &multi_region,
        &microservice_migration,
        &retired_vocabulary,
        &docs_pipeline,
        &documentation_system_kernel,
    );

    for expected in [
        "cleanup commands must stay empty",
        "planned Rust/Buck2/Prow validator",
        "planned Rust/Buck2/Prow validator: multi-region-disposition",
        "planned Rust/Buck2/Prow validator: sovereign-tenant-pin",
        "future Rust/Buck2/Prow migration job",
        "retired CLI remains tombstone/provenance only",
        "documentation capability: openapi",
        "documentation capability: ",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn retired_compatibility_catalog_rejects_active_authority_wording() {
    let gate_catalog = read_repo_file("libs/oya-governance-gate-catalog-domain/src/lib.rs")
        .replace(
            "Retired Foundry gate-catalog compatibility domain",
            "Foundry gate-catalog canonical domain — single source of truth",
        )
        .replace(
            "historical lift: the catalog below mirrors the retired",
            "source-of-truth lift: the catalog below mirrors the retired",
        )
        .replace(
            "Active CI/CD and merge readiness are Buck2/Prow/Kubernetes-native; this list\n/// is not part of merge gating.",
            "This list is the required merge substrate.",
        );
    let quality_lane = read_repo_file("libs/oya-check-quality-lane/src/lib.rs").replace(
        "retired compatibility wired-commands corpus",
        "canonical wired-commands catalog",
    );
    let failures = gate::retired_compatibility_catalog_failures(&gate_catalog, &quality_lane);

    for expected in [
        "Foundry gate-catalog canonical domain",
        "single source of truth",
        "required merge substrate",
        "canonical wired-commands catalog",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn retired_compatibility_catalog_requires_status_and_notice() {
    let gate_catalog = read_repo_file("libs/oya-governance-gate-catalog-domain/src/lib.rs")
        .replace("retired_compatibility_catalog", "active_catalog")
        .replace(
            "historical compatibility only; not CI/merge authority",
            "active authority",
        );
    let quality_lane = read_repo_file("libs/oya-check-quality-lane/src/lib.rs")
        .replace("compatibility/provenance only", "active authority");
    let failures = gate::retired_compatibility_catalog_failures(&gate_catalog, &quality_lane);

    for expected in [
        "retired_compatibility_catalog",
        "historical compatibility only; not CI/merge authority",
        "compatibility/provenance only",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn active_foundry_shared_surface_rejects_retired_root_guidance() {
    let readme = read_repo_file("README.md").replace(
        "SaaS, Workspace, Vertical, Intelligence, Cloud",
        "SaaS, Workspace, Vertical, Foundry, Cloud",
    );
    let root_hub = read_repo_file("specs/root-hub-pointers.json").replace(
        "\"owner_team\": \"council-architecture + platform-governance\"",
        "\"owner_team\": \"council-architecture + axis-foundry\"",
    );
    let sequencing = read_repo_file("specs/master-plan-sequencing.json").replace(
        "\"owner_team\": \"council-architecture + platform-governance\"",
        "\"owner_team\": \"council-architecture + axis-foundry\"",
    );
    let doc_agents = read_repo_file("docs/AGENTS.md")
        .replace(
            "intelligence/governance capabilities",
            "Foundry capabilities",
        )
        .replace(
            "Capability records + metering events consumed by capability runtimes.",
            "Capability records + metering events (Foundry-consumed).",
        );
    let failures =
        gate::active_foundry_shared_surface_failures(&readme, &root_hub, &sequencing, &doc_agents);

    for expected in [
        "Vertical, Foundry, Cloud",
        "axis-foundry",
        "Foundry capabilities",
        "Foundry-consumed",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn runbook_promotion_gate_scan_rejects_retired_jenkins_oya_gate_authority() {
    let root = temp_dir("runbook-promotion-gate");
    let runbook_dir = root.join("oya/example/runbooks");
    fs::create_dir_all(&runbook_dir).unwrap_or_else(|error| {
        panic!("create {}: {}", runbook_dir.display(), error);
    });

    fs::write(
        runbook_dir.join("stale.md"),
        "Promotion requires Jenkins + `oya gate run-all --ci-required` required.\n\
         Comment says green Jenkins CI and `oya gate run-all --ci-required`.\n\
         Attach Jenkins green, `oya gate run-all --ci-required` and `oya verify --ci-required` evidence attached.\n\
         We require Jenkins + `oya gate run-all --ci-required` before merge.\n",
    )
    .unwrap_or_else(|error| panic!("write stale runbook: {error}"));
    fs::write(
        runbook_dir.join("clean.md"),
        "Promotion requires `oya-ci-required` + Buck2 evidence before merge.\n",
    )
    .unwrap_or_else(|error| panic!("write clean runbook: {error}"));

    let failures = gate::runbook_promotion_gate_failures(&root);

    for expected in [
        "stale.md",
        "Jenkins + `oya gate run-all --ci-required` required",
        "green Jenkins CI and `oya gate run-all --ci-required`",
        "Jenkins green, `oya gate run-all --ci-required` and `oya verify --ci-required` evidence attached",
        "require Jenkins + `oya gate run-all --ci-required` before merge",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
    assert!(
        failures.iter().all(|failure| !failure.contains("clean.md")),
        "{failures:?}"
    );
}

#[test]
fn cloud_network_dns_authority_scan_rejects_superseded_ci_cd_phrases() {
    let root = temp_dir("cloud-network-dns-authority");
    let service_dir = root.join("cloud/cloud-network-dns");
    fs::create_dir_all(&service_dir).unwrap_or_else(|error| {
        panic!("create {}: {}", service_dir.display(), error);
    });

    fs::write(
        service_dir.join("README.md"),
        "The canonical local pre-push verifier is still active.\n\
         Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates.\n\
         ArgoCD is the canonical GitOps CD orchestrator.\n",
    )
    .unwrap_or_else(|error| panic!("write stale cloud-network-dns doc: {error}"));
    fs::write(
        service_dir.join("clean.md"),
        "ADR-0513 Buck2/Prow `oya-ci-required` evidence plus native release-conveyor seams are active.\n",
    )
    .unwrap_or_else(|error| panic!("write clean cloud-network-dns doc: {error}"));

    let failures = gate::cloud_network_dns_authority_failures(&root);

    for expected in [
        "README.md",
        "canonical local pre-push verifier",
        "Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates",
        "ArgoCD is the canonical GitOps CD orchestrator",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
    assert!(
        failures.iter().all(|failure| !failure.contains("clean.md")),
        "{failures:?}"
    );
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
fn vendored_agent_skills_guidance_rejects_retired_authority_phrases() {
    let mut guidance = read_repo_file("tools/agent-skills/AGENTS.md");
    guidance.push_str("\nJenkins CI + oya gate run-all\n");
    let failures = gate::active_doc_phrase_failures("tools/agent-skills/AGENTS.md", &guidance);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("Jenkins CI + oya gate run-all")),
        "{failures:?}"
    );
}

#[test]
fn vendored_agent_skills_guidance_rejects_retired_exact_names() {
    let mut guidance = read_repo_file("tools/agent-skills/AGENTS.md");
    guidance.push_str("\nRetain Jenkins as canonical.\n");
    let failures = gate::retired_exact_name_failures("tools/agent-skills/AGENTS.md", &guidance);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("retired exact-name reference")),
        "{failures:?}"
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
fn active_template_phrase_scanner_rejects_retired_shared_template_commands() {
    let failures = gate::active_template_phrase_failures(
        "templates/pull-request-template.md",
        "Run oya verify, oya gate validate, pnpm test (Node 20), cargo nextest run, cargo deny check, cargo public-api, grit claim, grit done, and oya-tooling-agent-read run-evidence.",
    );
    for expected in [
        "oya verify",
        "oya gate validate",
        "pnpm",
        "Node 20",
        "cargo nextest run",
        "cargo deny check",
        "cargo public-api",
        "grit claim",
        "grit done",
        "oya-tooling-agent-read",
    ] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
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
fn active_doc_phrase_scanner_rejects_retired_doc_generation_cli_refs() {
    let failures = gate::active_doc_phrase_failures(
        "docs/DOCUMENTATION.md",
        "The documentation registry says oya doc openapi and oya doc adr-index are active commands.",
    );
    assert!(
        failures.iter().any(|failure| failure.contains("oya doc ")),
        "{failures:?}"
    );
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

#[test]
fn workplace_integration_authority_scan_rejects_retired_cli_and_helm_authority() {
    let root = temp_dir("workplace-integration-authority");
    let runbook_dir = root.join("oya/workplace-integration/runbooks");
    let helm_dir = root.join("oya/workplace-integration/iac/k8s/helm/templates");
    std::fs::create_dir_all(&runbook_dir).unwrap();
    std::fs::create_dir_all(&helm_dir).unwrap();
    std::fs::write(
        runbook_dir.join("stale.md"),
        "Trigger from CI when `cargo run -p oya-dev-cli -- gate validate workplace --production-snapshot` exits non-zero.\n",
    )
    .unwrap();
    std::fs::write(
        helm_dir.join("deployment.yaml"),
        "metadata:\n  labels:\n    app.kubernetes.io/managed-by: Helm\n",
    )
    .unwrap();

    let failures = gate::workplace_integration_authority_failures(&root);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("cargo run -p oya-dev-cli -- gate validate")),
        "{failures:?}"
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("first-party Helm chart directory must not exist")),
        "{failures:?}"
    );

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn cell_lifecycle_authority_scan_rejects_retired_cli_and_helm_authority() {
    let root = temp_dir("cell-lifecycle-authority");
    let ip_dir = root.join("cloud/cell-lifecycle/IPs");
    let helm_dir = root.join("cloud/cell-lifecycle/iac/k8s/helm/templates");
    std::fs::create_dir_all(&ip_dir).unwrap();
    std::fs::create_dir_all(&helm_dir).unwrap();
    std::fs::write(
        ip_dir.join("stale.md"),
        "Citation verification runs `cargo run -q -p oya-dev-cli -- gate validate adr-citation`.\n",
    )
    .unwrap();
    std::fs::write(
        helm_dir.join("deployment.yaml"),
        "metadata:\n  labels:\n    app.kubernetes.io/managed-by: Helm\n",
    )
    .unwrap();

    let failures = gate::cell_lifecycle_authority_failures(&root);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("cargo run -q -p oya-dev-cli -- gate validate")),
        "{failures:?}"
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("first-party Helm chart directory must not exist")),
        "{failures:?}"
    );

    std::fs::remove_dir_all(root).ok();
}
