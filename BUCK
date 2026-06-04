# A list of available rules and their signatures can be found here: https://buck2.build/docs/prelude/globals/

genrule(
    name = "hello_world",
    out = "out.txt",
    cmd = "echo BUILT BY BUCK2> $OUT",
)


# Buck2 root BUCK dialect rejects def-based src helpers in this repository;
# keep current service CI files explicitly declared here, while
# specs/buck2-authority-policy.json expands command_scan_globs
# (cloud/*/ci/Jenkinsfile, oya/*/ci/Jenkinsfile) in the legacy policy scanner.
genrule(
    name = "buck2-authority-policy-check",
    srcs = {
        ".github/workflows/backbone-microservices-ci.yml": ".github/workflows/backbone-microservices-ci.yml",
        "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy": "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy",
        "infra/ci/buck2-affected-gate.sh": "infra/ci/buck2-affected-gate.sh",
        "infra/ci/jenkins/farmwide-seed.groovy": "infra/ci/jenkins/farmwide-seed.groovy",
        "infra/ci/jenkins/codegen-measure-seed.groovy": "infra/ci/jenkins/codegen-measure-seed.groovy",
        "infra/ci/jenkins/parallel-lanes-seed.groovy": "infra/ci/jenkins/parallel-lanes-seed.groovy",
        "infra/ci/jenkins/smoke-seed.groovy": "infra/ci/jenkins/smoke-seed.groovy",
        "infra/ci/jenkins/agent-image/Dockerfile": "infra/ci/jenkins/agent-image/Dockerfile",
        "scripts/branch-protection-apply.sh": "scripts/branch-protection-apply.sh",
        "scripts/onprem-bring-up.sh": "scripts/onprem-bring-up.sh",
        "scripts/install-trivy-ci.sh": "scripts/install-trivy-ci.sh",
        "scripts/supply-chain-adr0039.sh": "scripts/supply-chain-adr0039.sh",
        "scripts/hooks/pre-push.sh": "scripts/hooks/pre-push.sh",
        "scripts/ci/enforce-buck2-authority.rs": "scripts/ci/enforce-buck2-authority.rs",
        "scripts/ci/oya-ci-post.sh": "scripts/ci/oya-ci-post.sh",
        "scripts/ci/assert-pr-required-context.rs": "scripts/ci/assert-pr-required-context.rs",
        "scripts/tests/phase0_required_context_rollup_check.rs": "scripts/tests/phase0_required_context_rollup_check.rs",
        "scripts/ci/assert-required-status-source.rs": "scripts/ci/assert-required-status-source.rs",
        "scripts/tests/phase0_required_status_source_check.rs": "scripts/tests/phase0_required_status_source_check.rs",
        "scripts/ci/assert-tenant-pipeline-isolation.rs": "scripts/ci/assert-tenant-pipeline-isolation.rs",
        "scripts/tests/phase0_tenant_isolation_fixture_check.rs": "scripts/tests/phase0_tenant_isolation_fixture_check.rs",
        "scripts/ci/assert-override-kill-switch.rs": "scripts/ci/assert-override-kill-switch.rs",
        "scripts/tests/phase0_override_kill_switch_check.rs": "scripts/tests/phase0_override_kill_switch_check.rs",
        "scripts/ci/assert-trusted-target-inventory.rs": "scripts/ci/assert-trusted-target-inventory.rs",
        "scripts/tests/phase0_trusted_target_inventory_check.rs": "scripts/tests/phase0_trusted_target_inventory_check.rs",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "scripts/tests/phase0_result_bundle_output_check.rs": "scripts/tests/phase0_result_bundle_output_check.rs",
        "scripts/ci/assert-automation-ratchet.rs": "scripts/ci/assert-automation-ratchet.rs",
        "scripts/tests/phase0_automation_ratchet_check.rs": "scripts/tests/phase0_automation_ratchet_check.rs",
        "scripts/tests/buck2_authority_policy_check.rs": "scripts/tests/buck2_authority_policy_check.rs",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/oya-ci-prow-capability-parity.json": "specs/oya-ci-prow-capability-parity.json",
        "specs/root-hub-pointers.json": "specs/root-hub-pointers.json",
        ".github/workflows/github-lane-unlocker-ci-cd.yml": ".github/workflows/github-lane-unlocker-ci-cd.yml",
        ".github/branch-protection.yaml": ".github/branch-protection.yaml",
        "infra/branch-protection/dev.json": "infra/branch-protection/dev.json",
        "scripts/ci/assert-github-lane-unlocker-bridge.rs": "scripts/ci/assert-github-lane-unlocker-bridge.rs",
        "scripts/tests/github_lane_unlocker_bridge_check.rs": "scripts/tests/github_lane_unlocker_bridge_check.rs",
        "scripts/ci/github-actions-lane-unlocker-bootstrap.sh": "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
        "rust-toolchain.toml": "rust-toolchain.toml",
        "specs/github-lane-unlocker-bridge.json": "specs/github-lane-unlocker-bridge.json",
        "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md": "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
        "docs/ci/github-actions-lane-unlocker.md": "docs/ci/github-actions-lane-unlocker.md",
        "scripts/ci/assert-third-party-durable-handedits.rs": "scripts/ci/assert-third-party-durable-handedits.rs",
        "scripts/tests/third_party_durable_handedits_check.rs": "scripts/tests/third_party_durable_handedits_check.rs",
        "scripts/ci/assert-repo-hygiene-automation.rs": "scripts/ci/assert-repo-hygiene-automation.rs",
        "scripts/tests/repo_hygiene_automation_check.rs": "scripts/tests/repo_hygiene_automation_check.rs",
        "specs/repo-hygiene-automation.json": "specs/repo-hygiene-automation.json",
        "specs/retired-external-substrate-registry.json": "specs/retired-external-substrate-registry.json",
        "tools/oya-doc-staleness-inventory-app/BUCK": "//tools/oya-doc-staleness-inventory-app:BUCK",
        "tools/oya-doc-staleness-inventory-app/Cargo.toml": "//tools/oya-doc-staleness-inventory-app:cargo-manifest",
        "tools/oya-doc-staleness-inventory-app/src/lib.rs": "//tools/oya-doc-staleness-inventory-app:lib-src",
        "tools/oya-doc-staleness-inventory-app/src/main.rs": "//tools/oya-doc-staleness-inventory-app:main-src",
        "docs/DOC-CATALOG.md": "docs/DOC-CATALOG.md",
        "README.md": "README.md",
        "AGENTS.md": "AGENTS.md",
        "CLAUDE.md": "CLAUDE.md",
        "docs/AGENTS.md": "docs/AGENTS.md",
        "scripts/ci/assert-claim-ceiling.rs": "scripts/ci/assert-claim-ceiling.rs",
        "scripts/tests/phase0_claim_ceiling_check.rs": "scripts/tests/phase0_claim_ceiling_check.rs",
        "specs/phase0-claim-evidence-map.json": "specs/phase0-claim-evidence-map.json",
        "specs/hyperscaler-production-readiness-claim-contract.json": "specs/hyperscaler-production-readiness-claim-contract.json",
        "scripts/ci/assert-phase0-aggregate-exit.rs": "scripts/ci/assert-phase0-aggregate-exit.rs",
        "scripts/tests/phase0_aggregate_exit_check.rs": "scripts/tests/phase0_aggregate_exit_check.rs",
        "scripts/ci/assert-rust-testing-standard.rs": "scripts/ci/assert-rust-testing-standard.rs",
        "scripts/tests/rust_testing_standard_check.rs": "scripts/tests/rust_testing_standard_check.rs",
        "scripts/ci/assert-rust-llvm-coverage-runner-contract.rs": "scripts/ci/assert-rust-llvm-coverage-runner-contract.rs",
        "scripts/tests/rust_llvm_coverage_runner_contract_check.rs": "scripts/tests/rust_llvm_coverage_runner_contract_check.rs",
        "specs/rust-llvm-coverage-runner-contract.json": "specs/rust-llvm-coverage-runner-contract.json",
        "scripts/ci/run-rust-llvm-coverage-smoke.rs": "scripts/ci/run-rust-llvm-coverage-smoke.rs",
        "scripts/tests/rust_llvm_coverage_smoke_check.rs": "scripts/tests/rust_llvm_coverage_smoke_check.rs",
        "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs": "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs",
        "scripts/ci/assert-buck2-cargo-target-coverage.rs": "scripts/ci/assert-buck2-cargo-target-coverage.rs",
        "scripts/tests/buck2_cargo_target_coverage_check.rs": "scripts/tests/buck2_cargo_target_coverage_check.rs",
        "specs/buck2-cargo-target-coverage.json": "specs/buck2-cargo-target-coverage.json",
        "scripts/ci/assert-red-green-fixture-contract.rs": "scripts/ci/assert-red-green-fixture-contract.rs",
        "scripts/tests/red_green_fixture_contract_check.rs": "scripts/tests/red_green_fixture_contract_check.rs",
        "specs/red-green-fixture-contract.json": "specs/red-green-fixture-contract.json",
        "scripts/ci/assert-phase0-merge-conflict-foundation.rs": "scripts/ci/assert-phase0-merge-conflict-foundation.rs",
        "scripts/tests/phase0_merge_conflict_foundation_check.rs": "scripts/tests/phase0_merge_conflict_foundation_check.rs",
        "specs/generated-artifact-registry.json": "specs/generated-artifact-registry.json",
        "scripts/ci/assert-service-root-classifier.rs": "scripts/ci/assert-service-root-classifier.rs",
        "scripts/tests/service_root_classifier_check.rs": "scripts/tests/service_root_classifier_check.rs",
        "specs/service-inventory.json": "specs/service-inventory.json",
        "specs/phase0-structural-packets.json": "specs/phase0-structural-packets.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-good-seed.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-good-seed.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-missing-inventory-entry.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-missing-inventory-entry.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-outside-closed-world.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-outside-closed-world.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-layout-sprawl.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-layout-sprawl.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-real-token-live-field.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-real-token-live-field.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-structural-packet-family.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-structural-packet-family.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-duplicate-service.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-duplicate-service.json",
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-underscore-crate.json": "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-underscore-crate.json",
        "scripts/ci/assert-status-enum-drift.rs": "scripts/ci/assert-status-enum-drift.rs",
        "scripts/tests/status_enum_drift_check.rs": "scripts/tests/status_enum_drift_check.rs",
        "specs/status-enum-registry.json": "specs/status-enum-registry.json",
        "scripts/ci/assert-adr-hygiene.py": "scripts/ci/assert-adr-hygiene.py",
        "scripts/tests/adr_hygiene_check.test.sh": "scripts/tests/adr_hygiene_check.test.sh",
        "specs/adr-hygiene-registry.json": "specs/adr-hygiene-registry.json",
        "scripts/ci/assert-language-discipline.rs": "scripts/ci/assert-language-discipline.rs",
        "scripts/tests/language_discipline_check.rs": "scripts/tests/language_discipline_check.rs",
        "specs/language-discipline-registry.json": "specs/language-discipline-registry.json",
        "scripts/ci/assert-d1-seam-contracts.rs": "scripts/ci/assert-d1-seam-contracts.rs",
        "scripts/tests/d1_seam_contracts_check.rs": "scripts/tests/d1_seam_contracts_check.rs",
        "specs/d1-seam-contracts-registry.json": "specs/d1-seam-contracts-registry.json",
        "contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto": "contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto",
        "contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto": "contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto",
        "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-good-d1-seam-contracts.json": "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-good-d1-seam-contracts.json",
        "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-bad-missing-consistency-token.json": "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-bad-missing-consistency-token.json",
        "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-bad-proto-required-or-frozen-topology.json": "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-bad-proto-required-or-frozen-topology.json",
        "scripts/ci/assert-effective-dating-kernel.rs": "scripts/ci/assert-effective-dating-kernel.rs",
        "scripts/tests/effective_dating_kernel_check.rs": "scripts/tests/effective_dating_kernel_check.rs",
        "specs/effective-dating-kernel-registry.json": "specs/effective-dating-kernel-registry.json",
        "oya/ontology/crates/oya-ontology-kernel/src/effective_dating.rs": "//oya/ontology/crates/oya-ontology-kernel:effective-dating-src",
        "oya/ontology/crates/oya-ontology-kernel/src/lib.rs": "//oya/ontology/crates/oya-ontology-kernel:lib-src",
        "oya/ontology/crates/oya-ontology-kernel/BUCK": "//oya/ontology/crates/oya-ontology-kernel:BUCK",
        "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-good-effective-dating-kernel.json": "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-good-effective-dating-kernel.json",
        "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-overlapping-valid-time.json": "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-overlapping-valid-time.json",
        "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-clock-skew-nondeterministic.json": "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-clock-skew-nondeterministic.json",
        "scripts/ci/assert-cross-artifact-agreement.rs": "scripts/ci/assert-cross-artifact-agreement.rs",
        "scripts/tests/cross_artifact_agreement_check.rs": "scripts/tests/cross_artifact_agreement_check.rs",
        "specs/cross-artifact-agreement-registry.json": "specs/cross-artifact-agreement-registry.json",
        "specs/decision-propagation-packets.json": "specs/decision-propagation-packets.json",
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-good-cross-artifact-agreement.json": "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-good-cross-artifact-agreement.json",
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-missing-masterplan-roadmap.json": "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-missing-masterplan-roadmap.json",
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-unreconciled-idea-refine-output.json": "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-unreconciled-idea-refine-output.json",
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-generated-decisions-divergence.json": "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-generated-decisions-divergence.json",
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-missing-register-packet.json": "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-missing-register-packet.json",
        "scripts/ci/assert-structural-lock-revert.rs": "scripts/ci/assert-structural-lock-revert.rs",
        "scripts/tests/structural_lock_revert_check.rs": "scripts/tests/structural_lock_revert_check.rs",
        "specs/structural-lock-revert-registry.json": "specs/structural-lock-revert-registry.json",
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-good-serialized-structural-lock-revert.json": "specs/fixtures/phase0-structural-lock-revert/tc-0.9-good-serialized-structural-lock-revert.json",
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-missing-protected-revert-evidence.json": "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-missing-protected-revert-evidence.json",
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-overlapping-structural-lanes.json": "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-overlapping-structural-lanes.json",
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-mechanical-lock-claim.json": "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-mechanical-lock-claim.json",
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-stale-lock-ttl.json": "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-stale-lock-ttl.json",
        "scripts/ci/assert-d1-read-your-writes-xfail.rs": "scripts/ci/assert-d1-read-your-writes-xfail.rs",
        "scripts/tests/d1_read_your_writes_xfail_check.rs": "scripts/tests/d1_read_your_writes_xfail_check.rs",
        "specs/d1-read-your-writes-xfail-registry.json": "specs/d1-read-your-writes-xfail-registry.json",
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-good-xfail-classified-read-your-writes.json": "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-good-xfail-classified-read-your-writes.json",
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-misclassified-green-without-phase2.json": "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-misclassified-green-without-phase2.json",
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-missing-consistency-token.json": "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-missing-consistency-token.json",
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-phase2-green-claim-without-live-evidence.json": "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-phase2-green-claim-without-live-evidence.json",
        "scripts/ci/assert-who-gates-gates.rs": "scripts/ci/assert-who-gates-gates.rs",
        "scripts/tests/who_gates_gates_check.rs": "scripts/tests/who_gates_gates_check.rs",
        "specs/who-gates-gates-registry.json": "specs/who-gates-gates-registry.json",
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-good-known-bad-meta-gate.json": "specs/fixtures/phase0-who-gates-gates/tc-0.11-good-known-bad-meta-gate.json",
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-known-bad-fixture.json": "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-known-bad-fixture.json",
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-vacuous-pass-condition.json": "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-vacuous-pass-condition.json",
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-self-mutation-test.json": "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-self-mutation-test.json",
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-oya-cli-authority-route.json": "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-oya-cli-authority-route.json",
        "specs/fixtures/phase0-language-discipline/tc-0.4-good-allowlisted-bootstrap-shell-edit.json": "specs/fixtures/phase0-language-discipline/tc-0.4-good-allowlisted-bootstrap-shell-edit.json",
        "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-python-under-scripts.json": "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-python-under-scripts.json",
        "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-shell-test-sprawl.json": "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-shell-test-sprawl.json",
        "specs/fixtures/phase0-language-discipline/tc-0.4-good-non-script-change.json": "specs/fixtures/phase0-language-discipline/tc-0.4-good-non-script-change.json",
        "docs/decisions/ADR-0377-forgejo-board-git-ref-cas-fallback.md": "docs/decisions/ADR-0377-forgejo-board-git-ref-cas-fallback.md",
        "docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md": "docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md",
        "specs/fixtures/phase0-status-enum-drift/tc-status-enum-good-aligned.json": "specs/fixtures/phase0-status-enum-drift/tc-status-enum-good-aligned.json",
        "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-invalid-status-value.json": "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-invalid-status-value.json",
        "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-spec-without-code.json": "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-spec-without-code.json",
        "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-code-without-spec.json": "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-code-without-spec.json",
        "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-status-drift.json": "specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-status-drift.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-generated-artifact-unregistered.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-generated-artifact-unregistered.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-phase1-tide-batching-claim.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-phase1-tide-batching-claim.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-merge-tree-conflict.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-merge-tree-conflict.json",
        "specs/fixtures/phase0-required-context-rollup/good-github-lane-unlocker-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json": "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json",
        "specs/fixtures/phase0-required-context-rollup/bad-missing-github-lane-unlocker-required.json": "specs/fixtures/phase0-required-context-rollup/bad-missing-github-lane-unlocker-required.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-failure.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-completed-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-completed-failure.json",
        "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json",
        "scripts/ci/regen-third-party.sh": "scripts/ci/regen-third-party.sh",
        "scripts/gen_first_party_buck.py": "scripts/gen_first_party_buck.py",
        "scripts/agent-pre-push-validate.sh": "scripts/agent-pre-push-validate.sh",
        "scripts/build/build-and-push-cloud-intelligence.sh": "scripts/build/build-and-push-cloud-intelligence.sh",
        "cloud/cloud-intelligence/iac/oci/BUCK": "//cloud/cloud-intelligence/iac/oci:BUCK",
        "scripts/asyncapi-lint.mjs": "scripts/asyncapi-lint.mjs",
        "scripts/proto-lint.mjs": "scripts/proto-lint.mjs",
        "scripts/validate-adr-shape.mjs": "scripts/validate-adr-shape.mjs",
        "scripts/validate-foundry-phase00-evidence.mjs": "scripts/validate-foundry-phase00-evidence.mjs",
        "tools/hooks/no-cargo-enforcer.sh": "tools/hooks/no-cargo-enforcer.sh",
        "Makefile": "Makefile",
        "docs/TOOLCHAIN.md": "docs/TOOLCHAIN.md",
        "docs/standards/code-style.md": "docs/standards/code-style.md",
        "docs/standards/rust-release-optimization.md": "docs/standards/rust-release-optimization.md",
        "docs/standards/testing.md": "docs/standards/testing.md",
        "docs/standards/ci-lanes.md": "docs/standards/ci-lanes.md",
        "docs/standards/release-management.md": "docs/standards/release-management.md",
        "docs/standards/multi-agent-tool-map.md": "docs/standards/multi-agent-tool-map.md",
        "docs/security.md": "docs/security.md",
        "cloud/cloud-intelligence/Dockerfile": "cloud/cloud-intelligence/Dockerfile",
        "oya/ci-webhook-gateway/Dockerfile": "//oya/ci-webhook-gateway:Dockerfile",
        "oya/governance/iac/build/Dockerfile.distroless-rust": "oya/governance/iac/build/Dockerfile.distroless-rust",
        "oya/application/crates/oya-application-shell-frontend-prototype/client-manifest.json": "//oya/application/crates/oya-application-shell-frontend-prototype:client-manifest.json",
        "infra/ci/jenkins/reported-status-contexts.json": "infra/ci/jenkins/reported-status-contexts.json",
        "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs": "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs",
        "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh": "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh",
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh": "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh": "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh",
        "scripts/ci/arm-auto-merge.sh": "scripts/ci/arm-auto-merge.sh",
        "scripts/trigger-next-queue-automerge.sh": "scripts/trigger-next-queue-automerge.sh",
        "scripts/check-sequential-pr-merge-conflicts.sh": "scripts/check-sequential-pr-merge-conflicts.sh",
        "scripts/tests/forgejo_auto_merge_after_ci.test.sh": "scripts/tests/forgejo_auto_merge_after_ci.test.sh",
        "scripts/tests/phase0_auto_merge_after_ci_contract_check.py": "scripts/tests/phase0_auto_merge_after_ci_contract_check.py",
        "docs/ci/auto-merge-flow.md": "docs/ci/auto-merge-flow.md",
        "docs/ci/forge-of-record.md": "docs/ci/forge-of-record.md",
        "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md": "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md",
        "docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md": "docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md",
        "specs/phase0-auto-merge-after-ci.json": "specs/phase0-auto-merge-after-ci.json",
        "oya/ci-tide/crates/oya-ci-tide-kernel/src/lib.rs": "//oya/ci-tide/crates/oya-ci-tide-kernel:lib-src",
        "oya/ci-tide/crates/oya-ci-tide-app/src/lib.rs": "//oya/ci-tide/crates/oya-ci-tide-app:lib-src",
        "oya/ci-tide/crates/oya-ci-tide-forgejo-adapter/src/lib.rs": "//oya/ci-tide/crates/oya-ci-tide-forgejo-adapter:lib-src",
        "docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md": "docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md",
        "docs/decisions/ADR-0361-jenkins-native-cicd-revamp-execution.md": "docs/decisions/ADR-0361-jenkins-native-cicd-revamp-execution.md",
        "docs/decisions/ADR-0392-buck2-canonical-build-graph.md": "docs/decisions/ADR-0392-buck2-canonical-build-graph.md",
        "docs/decisions/ADR-0408-buck2-driven-ci-cd.md": "docs/decisions/ADR-0408-buck2-driven-ci-cd.md",
        "docs/decisions/ADR-0515-buck2-native-oci-static-musl-base.md": "docs/decisions/ADR-0515-buck2-native-oci-static-musl-base.md",
        "docs/decisions/ADR-0360-ci-pipeline-optimization-program.md": "docs/decisions/ADR-0360-ci-pipeline-optimization-program.md",
        "docs/decisions/ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate.md": "docs/decisions/ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate.md",
        "docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md": "docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md",
        "BUCK": "BUCK",
        "specs/buck2-authority-policy.json": "specs/buck2-authority-policy.json",
        "cloud/cell-lifecycle/ci/Jenkinsfile": "cloud/cell-lifecycle/ci/Jenkinsfile",
        "cloud/cell-rebalancer/ci/Jenkinsfile": "cloud/cell-rebalancer/ci/Jenkinsfile",
        "cloud/cloud-billing-tax/ci/Jenkinsfile": "cloud/cloud-billing-tax/ci/Jenkinsfile",
        "cloud/cloud-billing/ci/Jenkinsfile": "cloud/cloud-billing/ci/Jenkinsfile",
        "cloud/cloud-data/ci/Jenkinsfile": "cloud/cloud-data/ci/Jenkinsfile",
        "cloud/cloud-iac/ci/Jenkinsfile": "cloud/cloud-iac/ci/Jenkinsfile",
        "cloud/cloud-iam/ci/Jenkinsfile": "cloud/cloud-iam/ci/Jenkinsfile",
        "cloud/cloud-intelligence/ci/Jenkinsfile": "cloud/cloud-intelligence/ci/Jenkinsfile",
        "cloud/cloud-k8s/ci/Jenkinsfile": "cloud/cloud-k8s/ci/Jenkinsfile",
        "cloud/cloud-kms/ci/Jenkinsfile": "cloud/cloud-kms/ci/Jenkinsfile",
        "cloud/cloud-network-dns/ci/Jenkinsfile": "cloud/cloud-network-dns/ci/Jenkinsfile",
        "cloud/cloud-network/ci/Jenkinsfile": "cloud/cloud-network/ci/Jenkinsfile",
        "cloud/cloud-secrets/ci/Jenkinsfile": "cloud/cloud-secrets/ci/Jenkinsfile",
        "cloud/cloud-storage/ci/Jenkinsfile": "cloud/cloud-storage/ci/Jenkinsfile",
        "cloud/tenancy/ci/Jenkinsfile": "cloud/tenancy/ci/Jenkinsfile",
        "oya/accounting/ci/Jenkinsfile": "oya/accounting/ci/Jenkinsfile",
        "oya/analytics/ci/Jenkinsfile": "oya/analytics/ci/Jenkinsfile",
        "oya/api-gateway/ci/Jenkinsfile": "oya/api-gateway/ci/Jenkinsfile",
        "oya/application/ci/Jenkinsfile": "oya/application/ci/Jenkinsfile",
        "oya/audit-chain/ci/Jenkinsfile": "oya/audit-chain/ci/Jenkinsfile",
        "oya/calendar/ci/Jenkinsfile": "oya/calendar/ci/Jenkinsfile",
        "oya/comms-email/ci/Jenkinsfile": "oya/comms-email/ci/Jenkinsfile",
        "oya/community/ci/Jenkinsfile": "oya/community/ci/Jenkinsfile",
        "oya/compliance/ci/Jenkinsfile": "oya/compliance/ci/Jenkinsfile",
        "oya/connector/ci/Jenkinsfile": "oya/connector/ci/Jenkinsfile",
        "oya/consent-graph/ci/Jenkinsfile": "oya/consent-graph/ci/Jenkinsfile",
        "oya/contact-center/ci/Jenkinsfile": "oya/contact-center/ci/Jenkinsfile",
        "oya/contract-lifecycle-management/ci/Jenkinsfile": "oya/contract-lifecycle-management/ci/Jenkinsfile",
        "oya/crm/ci/Jenkinsfile": "oya/crm/ci/Jenkinsfile",
        "oya/data-pipeline/ci/Jenkinsfile": "oya/data-pipeline/ci/Jenkinsfile",
        "oya/data-warehouse/ci/Jenkinsfile": "oya/data-warehouse/ci/Jenkinsfile",
        "oya/design-collaboration/ci/Jenkinsfile": "oya/design-collaboration/ci/Jenkinsfile",
        "oya/detection/ci/Jenkinsfile": "oya/detection/ci/Jenkinsfile",
        "oya/developer-sdk/ci/Jenkinsfile": "oya/developer-sdk/ci/Jenkinsfile",
        "oya/diagnostics/ci/Jenkinsfile": "oya/diagnostics/ci/Jenkinsfile",
        "oya/docs/ci/Jenkinsfile": "oya/docs/ci/Jenkinsfile",
        "oya/drive/ci/Jenkinsfile": "oya/drive/ci/Jenkinsfile",
        "oya/emergency/ci/Jenkinsfile": "oya/emergency/ci/Jenkinsfile",
        "oya/emr/ci/Jenkinsfile": "oya/emr/ci/Jenkinsfile",
        "oya/feature-flags/ci/Jenkinsfile": "oya/feature-flags/ci/Jenkinsfile",
        "oya/financial-planning/ci/Jenkinsfile": "oya/financial-planning/ci/Jenkinsfile",
        "oya/finops-portal/ci/Jenkinsfile": "oya/finops-portal/ci/Jenkinsfile",
        "oya/forms/ci/Jenkinsfile": "oya/forms/ci/Jenkinsfile",
        "oya/global-trade/ci/Jenkinsfile": "oya/global-trade/ci/Jenkinsfile",
        "oya/governance/ci/Jenkinsfile": "oya/governance/ci/Jenkinsfile",
        "oya/healthcare-integration/ci/Jenkinsfile": "oya/healthcare-integration/ci/Jenkinsfile",
        "oya/hr/ci/Jenkinsfile": "oya/hr/ci/Jenkinsfile",
        "oya/identity/ci/Jenkinsfile": "oya/identity/ci/Jenkinsfile",
        "oya/imaging/ci/Jenkinsfile": "oya/imaging/ci/Jenkinsfile",
        "oya/incident-management/ci/Jenkinsfile": "oya/incident-management/ci/Jenkinsfile",
        "oya/intelligence/ci/Jenkinsfile": "oya/intelligence/ci/Jenkinsfile",
        "oya/itsm/ci/Jenkinsfile": "oya/itsm/ci/Jenkinsfile",
        "oya/learning-management/ci/Jenkinsfile": "oya/learning-management/ci/Jenkinsfile",
        "oya/mail/ci/Jenkinsfile": "oya/mail/ci/Jenkinsfile",
        "oya/marketing-automation/ci/Jenkinsfile": "oya/marketing-automation/ci/Jenkinsfile",
        "oya/marketplace/ci/Jenkinsfile": "oya/marketplace/ci/Jenkinsfile",
        "oya/meet/ci/Jenkinsfile": "oya/meet/ci/Jenkinsfile",
        "oya/messenger/ci/Jenkinsfile": "oya/messenger/ci/Jenkinsfile",
        "oya/notes/ci/Jenkinsfile": "oya/notes/ci/Jenkinsfile",
        "oya/observability/ci/Jenkinsfile": "oya/observability/ci/Jenkinsfile",
        "oya/ontology/ci/Jenkinsfile": "oya/ontology/ci/Jenkinsfile",
        "oya/ops-dashboard-control-center/ci/Jenkinsfile": "oya/ops-dashboard-control-center/ci/Jenkinsfile",
        "oya/patient-monitoring/ci/Jenkinsfile": "oya/patient-monitoring/ci/Jenkinsfile",
        "oya/payments/ci/Jenkinsfile": "oya/payments/ci/Jenkinsfile",
        "oya/payroll/ci/Jenkinsfile": "oya/payroll/ci/Jenkinsfile",
        "oya/performance-management/ci/Jenkinsfile": "oya/performance-management/ci/Jenkinsfile",
        "oya/pharmacy/ci/Jenkinsfile": "oya/pharmacy/ci/Jenkinsfile",
        "oya/plant-maintenance/ci/Jenkinsfile": "oya/plant-maintenance/ci/Jenkinsfile",
        "oya/plugin-app-store/ci/Jenkinsfile": "oya/plugin-app-store/ci/Jenkinsfile",
        "oya/production-planning/ci/Jenkinsfile": "oya/production-planning/ci/Jenkinsfile",
        "oya/quality-management/ci/Jenkinsfile": "oya/quality-management/ci/Jenkinsfile",
        "oya/real-estate/ci/Jenkinsfile": "oya/real-estate/ci/Jenkinsfile",
        "oya/recordings/ci/Jenkinsfile": "oya/recordings/ci/Jenkinsfile",
        "oya/sheets/ci/Jenkinsfile": "oya/sheets/ci/Jenkinsfile",
        "oya/sites/ci/Jenkinsfile": "oya/sites/ci/Jenkinsfile",
        "oya/slides/ci/Jenkinsfile": "oya/slides/ci/Jenkinsfile",
        "oya/social/ci/Jenkinsfile": "oya/social/ci/Jenkinsfile",
        "oya/supply-chain-planning/ci/Jenkinsfile": "oya/supply-chain-planning/ci/Jenkinsfile",
        "oya/tasks/ci/Jenkinsfile": "oya/tasks/ci/Jenkinsfile",
        "oya/tenant-rbac/ci/Jenkinsfile": "oya/tenant-rbac/ci/Jenkinsfile",
        "oya/translate/ci/Jenkinsfile": "oya/translate/ci/Jenkinsfile",
        "oya/treasury/ci/Jenkinsfile": "oya/treasury/ci/Jenkinsfile",
        "oya/warehouse/ci/Jenkinsfile": "oya/warehouse/ci/Jenkinsfile",
        "oya/whiteboard/ci/Jenkinsfile": "oya/whiteboard/ci/Jenkinsfile",
        "oya/workflow-engine/ci/Jenkinsfile": "oya/workflow-engine/ci/Jenkinsfile",
        "oya/workflow-studio/ci/Jenkinsfile": "oya/workflow-studio/ci/Jenkinsfile",
        "oya/workplace-integration/ci/Jenkinsfile": "oya/workplace-integration/ci/Jenkinsfile",
    },
    out = "buck2-authority-policy-check.json",
    cmd = "mkdir -p $TMP/buck2-authority-policy && rustc --edition=2021 -D warnings scripts/ci/enforce-buck2-authority.rs -o $TMP/buck2-authority-policy/enforce-buck2-authority && OYA_REPO_ROOT=$PWD $TMP/buck2-authority-policy/enforce-buck2-authority --policy specs/buck2-authority-policy.json > $OUT",
    visibility = ["PUBLIC"],
)


# P00 temporary GitHub/GitHub Actions lane-unlocker contract. This target is
# local/static evidence only: it validates the bridge spec, workflow, branch
# protection shadow, root-hub pointers, ADR, procedure, and Buck2 policy without
# mutating live GitHub, Kubernetes, or deployment state.
genrule(
    name = "github-lane-unlocker-bridge-check",
    srcs = {
        ".github/workflows/github-lane-unlocker-ci-cd.yml": ".github/workflows/github-lane-unlocker-ci-cd.yml",
        ".github/branch-protection.yaml": ".github/branch-protection.yaml",
        "infra/branch-protection/dev.json": "infra/branch-protection/dev.json",
        "scripts/ci/assert-github-lane-unlocker-bridge.rs": "scripts/ci/assert-github-lane-unlocker-bridge.rs",
        "scripts/tests/github_lane_unlocker_bridge_check.rs": "scripts/tests/github_lane_unlocker_bridge_check.rs",
        "scripts/ci/github-actions-lane-unlocker-bootstrap.sh": "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
        "rust-toolchain.toml": "rust-toolchain.toml",
        "specs/github-lane-unlocker-bridge.json": "specs/github-lane-unlocker-bridge.json",
        "specs/buck2-authority-policy.json": "specs/buck2-authority-policy.json",
        "specs/root-hub-pointers.json": "specs/root-hub-pointers.json",
        "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md": "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
        "docs/ci/github-actions-lane-unlocker.md": "docs/ci/github-actions-lane-unlocker.md",
    },
    out = "github-lane-unlocker-bridge-check.json",
    cmd = "mkdir -p $TMP/github-lane-unlocker-bridge && rustc --edition=2021 -D warnings scripts/tests/github_lane_unlocker_bridge_check.rs --test -o $TMP/github-lane-unlocker-bridge/github_lane_unlocker_bridge_check && OYA_REPO_ROOT=$PWD $TMP/github-lane-unlocker-bridge/github_lane_unlocker_bridge_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-github-lane-unlocker-bridge.rs -o $TMP/github-lane-unlocker-bridge/assert-github-lane-unlocker-bridge && OYA_REPO_ROOT=$PWD $TMP/github-lane-unlocker-bridge/assert-github-lane-unlocker-bridge --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)

# Third-party generated BUCK graph durable hand-edit check. Reindeer remains the
# metadata-to-BUCK generator, but active CI must assert the checked-in graph
# directly instead of running a Python mutator plus git diff.
genrule(
    name = "third-party-durable-handedits-check",
    srcs = {
        ".github/workflows/github-lane-unlocker-ci-cd.yml": ".github/workflows/github-lane-unlocker-ci-cd.yml",
        "BUCK": "BUCK",
        "infra/ci/buck2-affected-gate.sh": "infra/ci/buck2-affected-gate.sh",
        "scripts/ci/assert-third-party-durable-handedits.rs": "scripts/ci/assert-third-party-durable-handedits.rs",
        "scripts/tests/third_party_durable_handedits_check.rs": "scripts/tests/third_party_durable_handedits_check.rs",
        "scripts/ci/regen-third-party.sh": "scripts/ci/regen-third-party.sh",
        "third-party/BUCK": "third-party//:BUCK",
    },
    out = "third-party-durable-handedits-check.json",
    cmd = "mkdir -p $TMP/third-party-durable-handedits && rustc --edition=2021 -D warnings scripts/tests/third_party_durable_handedits_check.rs --test -o $TMP/third-party-durable-handedits/third_party_durable_handedits_check && OYA_REPO_ROOT=$PWD $TMP/third-party-durable-handedits/third_party_durable_handedits_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-third-party-durable-handedits.rs -o $TMP/third-party-durable-handedits/assert-third-party-durable-handedits && OYA_REPO_ROOT=$PWD $TMP/third-party-durable-handedits/assert-third-party-durable-handedits --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)

# P00 repo hygiene automation contract. This is local/static evidence for
# git/worktree, branch/merge, repository publication, disk/workspace,
# Kubernetes workload, and documentation-sprawl hygiene. It never deletes files,
# mutates live branch protection, or scales Kubernetes workloads.
genrule(
    name = "repo-hygiene-automation-check",
    srcs = {
        "scripts/ci/assert-repo-hygiene-automation.rs": "scripts/ci/assert-repo-hygiene-automation.rs",
        "scripts/tests/repo_hygiene_automation_check.rs": "scripts/tests/repo_hygiene_automation_check.rs",
        "specs/repo-hygiene-automation.json": "specs/repo-hygiene-automation.json",
        "specs/retired-external-substrate-registry.json": "specs/retired-external-substrate-registry.json",
        "specs/root-hub-pointers.json": "specs/root-hub-pointers.json",
        "specs/github-lane-unlocker-bridge.json": "specs/github-lane-unlocker-bridge.json",
        "specs/masterplan.json": "specs/masterplan.json",
        "specs/master-plan-sequencing.json": "specs/master-plan-sequencing.json",
        "tools/oya-doc-staleness-inventory-app/BUCK": "//tools/oya-doc-staleness-inventory-app:BUCK",
        "tools/oya-doc-staleness-inventory-app/Cargo.toml": "//tools/oya-doc-staleness-inventory-app:cargo-manifest",
        "tools/oya-doc-staleness-inventory-app/src/lib.rs": "//tools/oya-doc-staleness-inventory-app:lib-src",
        "tools/oya-doc-staleness-inventory-app/src/main.rs": "//tools/oya-doc-staleness-inventory-app:main-src",
        ".github/workflows/github-lane-unlocker-ci-cd.yml": ".github/workflows/github-lane-unlocker-ci-cd.yml",
        ".github/branch-protection.yaml": ".github/branch-protection.yaml",
        "infra/branch-protection/dev.json": "infra/branch-protection/dev.json",
        "docs/ci/github-actions-lane-unlocker.md": "docs/ci/github-actions-lane-unlocker.md",
        "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md": "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
        "docs/DOC-CATALOG.md": "docs/DOC-CATALOG.md",
        "docs/MASTERPLAN.md": "docs/MASTERPLAN.md",
        "docs/AGENTS.md": "docs/AGENTS.md",
        "README.md": "README.md",
        "AGENTS.md": "AGENTS.md",
        "CLAUDE.md": "CLAUDE.md",
        "BUCK": "BUCK",
    },
    out = "repo-hygiene-automation-check.json",
    cmd = "mkdir -p $TMP/repo-hygiene-automation && rustc --edition=2021 -D warnings scripts/tests/repo_hygiene_automation_check.rs --test -o $TMP/repo-hygiene-automation/repo_hygiene_automation_check && OYA_REPO_ROOT=$PWD $TMP/repo-hygiene-automation/repo_hygiene_automation_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-repo-hygiene-automation.rs -o $TMP/repo-hygiene-automation/assert-repo-hygiene-automation && OYA_REPO_ROOT=$PWD $TMP/repo-hygiene-automation/assert-repo-hygiene-automation --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)

# P0.0 Buck2-authority parity fixture guard: mutation-style RED/GREEN checks
# for upstream Prow parity rows, explicit upstream waivers, live-authority
# boundaries, and root-hub discoverability. Fixture mode narrows broad
# repository scans so this target stays sandboxable; the full scan remains
# //:buck2-authority-policy-check above.
genrule(
    name = "buck2-authority-policy-fixture-check",
    srcs = {
        "scripts/ci/enforce-buck2-authority.rs": "scripts/ci/enforce-buck2-authority.rs",
        "scripts/tests/buck2_authority_policy_check.rs": "scripts/tests/buck2_authority_policy_check.rs",
        "specs/buck2-authority-policy.json": "specs/buck2-authority-policy.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/oya-ci-prow-capability-parity.json": "specs/oya-ci-prow-capability-parity.json",
        "specs/root-hub-pointers.json": "specs/root-hub-pointers.json",
    },
    out = "buck2-authority-policy-fixture-check.txt",
    cmd = "mkdir -p $TMP/buck2-authority-policy-fixture && rustc --edition=2021 -D warnings scripts/tests/buck2_authority_policy_check.rs --test -o $TMP/buck2-authority-policy-fixture/buck2_authority_policy_check && OYA_REPO_ROOT=$PWD $TMP/buck2-authority-policy-fixture/buck2_authority_policy_check > $OUT",
    visibility = ["PUBLIC"],
)

# P0.0 baseline catalog closure: every checked-in executable fixture under
# specs/fixtures/phase0-ci-enforcement-baseline must be listed by the baseline
# packet and reachable through Buck2, not operator memory. The fixture glob is
# intentional: newly added fixtures must enter the Buck action sandbox even when
# the baseline forgets to catalog them.
genrule(
    name = "phase0-ci-enforcement-baseline-catalog-check",
    srcs = {
        "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs": "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs",
        "specs/phase0-ci-enforcement-baseline.json": "specs/phase0-ci-enforcement-baseline.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-claim-evidence-map.json": "specs/phase0-claim-evidence-map.json",
        "specs/red-green-fixture-contract.json": "specs/red-green-fixture-contract.json",
        "specs/phase0-ci-enforcement-result-schema.json": "specs/phase0-ci-enforcement-result-schema.json",
        "specs/phase0-override-packet-schema.json": "specs/phase0-override-packet-schema.json",
        "specs/phase0-trusted-target-inventory-schema.json": "specs/phase0-trusted-target-inventory-schema.json",
        "specs/toolchain-tenant-isolation-fixtures.json": "specs/toolchain-tenant-isolation-fixtures.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-ci-enforcement-baseline/*.json"])} | {path: path for path in glob(["specs/fixtures/phase0-required-status-source/*.json"])},
    out = "phase0-ci-enforcement-baseline-catalog-check.json",
    cmd = "mkdir -p $TMP/phase0-ci-baseline-catalog && rustc --edition=2021 -D warnings scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs --test -o $TMP/phase0-ci-baseline-catalog/phase0_ci_enforcement_baseline_catalog_check_tests && OYA_REPO_ROOT=$PWD $TMP/phase0-ci-baseline-catalog/phase0_ci_enforcement_baseline_catalog_check_tests > /dev/null && rustc --edition=2021 -D warnings scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs -o $TMP/phase0-ci-baseline-catalog/phase0_ci_enforcement_baseline_catalog_check && OYA_REPO_ROOT=$PWD $TMP/phase0-ci-baseline-catalog/phase0_ci_enforcement_baseline_catalog_check --json > $OUT",
    visibility = ["PUBLIC"],
)


# AC-0.12 aggregate-exit fixture check: local/static coverage that
# Phase-0 cannot pass on a partial, omitted, unknown, or false subcondition.
# This never claims live required-context authority, P0.0 green, Phase-0
# completion, production readiness, or hyperscaler-grade status.
genrule(
    name = "phase0-aggregate-exit-check",
    srcs = {
        "scripts/ci/assert-phase0-aggregate-exit.rs": "scripts/ci/assert-phase0-aggregate-exit.rs",
        "scripts/tests/phase0_aggregate_exit_check.rs": "scripts/tests/phase0_aggregate_exit_check.rs",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-exit-gate/*.json"])},
    out = "phase0-aggregate-exit-check.txt",
    cmd = "mkdir -p $TMP/phase0-aggregate-exit && rustc --edition=2021 -D warnings scripts/tests/phase0_aggregate_exit_check.rs --test -o $TMP/phase0-aggregate-exit/phase0_aggregate_exit_check && OYA_REPO_ROOT=$PWD $TMP/phase0-aggregate-exit/phase0_aggregate_exit_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-phase0-aggregate-exit.rs -o $TMP/phase0-aggregate-exit/assert-phase0-aggregate-exit && OYA_REPO_ROOT=$PWD $TMP/phase0-aggregate-exit/assert-phase0-aggregate-exit --json > $OUT",
    visibility = ["PUBLIC"],
)


# Rust testing-standard drift check: local/static coverage that the documented
# coverage and mutation-testing standard remains Buck2-native, Tarpaulin is not
# canonical, and local Cargo mutation stays advisory unless captured by Buck2 or
# trusted cloud-ci/oya-ci evidence. This does not implement the coverage runner,
# run mutation testing, or claim live Phase-0 authority.
genrule(
    name = "rust-testing-standard-check",
    srcs = {
        "scripts/ci/assert-rust-testing-standard.rs": "scripts/ci/assert-rust-testing-standard.rs",
        "scripts/tests/rust_testing_standard_check.rs": "scripts/tests/rust_testing_standard_check.rs",
        "docs/standards/testing.md": "docs/standards/testing.md",
    },
    out = "rust-testing-standard-check.txt",
    cmd = "mkdir -p $TMP/rust-testing-standard && rustc --edition=2021 -D warnings scripts/tests/rust_testing_standard_check.rs --test -o $TMP/rust-testing-standard/rust_testing_standard_check && OYA_REPO_ROOT=$PWD $TMP/rust-testing-standard/rust_testing_standard_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-rust-testing-standard.rs -o $TMP/rust-testing-standard/assert-rust-testing-standard && OYA_REPO_ROOT=$PWD $TMP/rust-testing-standard/assert-rust-testing-standard --doc docs/standards/testing.md --json > $OUT",
    visibility = ["PUBLIC"],
)


# Rust LLVM coverage-runner contract check: local/static target-shape coverage
# for future Buck2-native source-based coverage lanes. This validates the
# rustc/LLVM/Buck2 evidence contract and explicit non-claim boundary; it does
# not run tests, generate coverage reports, post statuses, or claim live Phase-0
# authority.
genrule(
    name = "rust-llvm-coverage-runner-contract-check",
    srcs = {
        "scripts/ci/assert-rust-llvm-coverage-runner-contract.rs": "scripts/ci/assert-rust-llvm-coverage-runner-contract.rs",
        "scripts/tests/rust_llvm_coverage_runner_contract_check.rs": "scripts/tests/rust_llvm_coverage_runner_contract_check.rs",
        "specs/rust-llvm-coverage-runner-contract.json": "specs/rust-llvm-coverage-runner-contract.json",
    },
    out = "rust-llvm-coverage-runner-contract-check.txt",
    cmd = "mkdir -p $TMP/rust-llvm-coverage-runner-contract && rustc --edition=2021 -D warnings scripts/tests/rust_llvm_coverage_runner_contract_check.rs --test -o $TMP/rust-llvm-coverage-runner-contract/rust_llvm_coverage_runner_contract_check && OYA_REPO_ROOT=$PWD $TMP/rust-llvm-coverage-runner-contract/rust_llvm_coverage_runner_contract_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-rust-llvm-coverage-runner-contract.rs -o $TMP/rust-llvm-coverage-runner-contract/assert-rust-llvm-coverage-runner-contract && OYA_REPO_ROOT=$PWD $TMP/rust-llvm-coverage-runner-contract/assert-rust-llvm-coverage-runner-contract --spec specs/rust-llvm-coverage-runner-contract.json --json > $OUT",
    visibility = ["PUBLIC"],
)


# Rust LLVM coverage smoke check: Buck2-owned local fixture evidence that the
# active Rust toolchain can emit .profraw via rustc source-based coverage and
# merge/export it with rustup-sysroot llvm-profdata/llvm-cov. This is a fixture
# smoke only; production coverage budgets and live required-context authority
# remain unproven until trusted cloud-ci/oya-ci runs the real coverage lane.
genrule(
    name = "rust-llvm-coverage-smoke-check",
    srcs = {
        "scripts/ci/run-rust-llvm-coverage-smoke.rs": "scripts/ci/run-rust-llvm-coverage-smoke.rs",
        "scripts/tests/rust_llvm_coverage_smoke_check.rs": "scripts/tests/rust_llvm_coverage_smoke_check.rs",
        "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs": "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs",
    },
    out = "rust-llvm-coverage-smoke-check.json",
    cmd = "mkdir -p $TMP/rust-llvm-coverage-smoke && rustc --edition=2021 -D warnings scripts/tests/rust_llvm_coverage_smoke_check.rs --test -o $TMP/rust-llvm-coverage-smoke/rust_llvm_coverage_smoke_check && OYA_REPO_ROOT=$PWD $TMP/rust-llvm-coverage-smoke/rust_llvm_coverage_smoke_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/run-rust-llvm-coverage-smoke.rs -o $TMP/rust-llvm-coverage-smoke/run-rust-llvm-coverage-smoke && OYA_REPO_ROOT=$PWD $TMP/rust-llvm-coverage-smoke/run-rust-llvm-coverage-smoke --out $OUT > /dev/null",
    visibility = ["PUBLIC"],
)


# AC-0.13 Buck2/Cargo target-coverage check: local/static evidence that
# every Cargo workspace lib/bin target root has a checked-in Buck2 rust
# crate_root mapping, including parent-BUCK mappings such as tools/oci/BUCK.
# This measures target graph coverage only; it does not run Cargo, generate
# source-line coverage, or claim live cloud-ci/oya-ci authority.
genrule(
    name = "buck2-cargo-target-coverage-check",
    srcs = {
        "scripts/ci/assert-buck2-cargo-target-coverage.rs": "scripts/ci/assert-buck2-cargo-target-coverage.rs",
        "scripts/tests/buck2_cargo_target_coverage_check.rs": "scripts/tests/buck2_cargo_target_coverage_check.rs",
        "specs/buck2-cargo-target-coverage.json": "specs/buck2-cargo-target-coverage.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "Cargo.toml": "Cargo.toml",
    } | {path: path for path in glob(["**/Cargo.toml", "**/BUCK", "**/src/lib.rs", "**/src/main.rs", "**/src/bin/**/*.rs"], exclude = ["buck-out/**", "target/**"])},
    out = "buck2-cargo-target-coverage-check.json",
    cmd = "mkdir -p $TMP/buck2-cargo-target-coverage && rustc --edition=2021 -D warnings scripts/tests/buck2_cargo_target_coverage_check.rs --test -o $TMP/buck2-cargo-target-coverage/buck2_cargo_target_coverage_check && OYA_REPO_ROOT=$PWD $TMP/buck2-cargo-target-coverage/buck2_cargo_target_coverage_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-buck2-cargo-target-coverage.rs -o $TMP/buck2-cargo-target-coverage/assert-buck2-cargo-target-coverage && OYA_REPO_ROOT=$PWD $TMP/buck2-cargo-target-coverage/assert-buck2-cargo-target-coverage --spec specs/buck2-cargo-target-coverage.json --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.14 RED/GREEN fixture contract check: local/static registry evidence that
# Phase-0 gate targets keep explicit GOOD and BAD fixture/probe coverage plus
# non-claim markers. This does not run live CI, post statuses, or claim Phase-0
# completion.
genrule(
    name = "phase0-red-green-fixture-contract-check",
    srcs = {
        "scripts/ci/assert-red-green-fixture-contract.rs": "scripts/ci/assert-red-green-fixture-contract.rs",
        "scripts/tests/red_green_fixture_contract_check.rs": "scripts/tests/red_green_fixture_contract_check.rs",
        "specs/red-green-fixture-contract.json": "specs/red-green-fixture-contract.json",
        "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs": "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.rs",
        "scripts/ci/assert-pr-required-context.rs": "scripts/ci/assert-pr-required-context.rs",
        "scripts/tests/phase0_required_context_rollup_check.rs": "scripts/tests/phase0_required_context_rollup_check.rs",
        "scripts/ci/assert-required-status-source.rs": "scripts/ci/assert-required-status-source.rs",
        "scripts/tests/phase0_required_status_source_check.rs": "scripts/tests/phase0_required_status_source_check.rs",
        "scripts/ci/assert-tenant-pipeline-isolation.rs": "scripts/ci/assert-tenant-pipeline-isolation.rs",
        "scripts/tests/phase0_tenant_isolation_fixture_check.rs": "scripts/tests/phase0_tenant_isolation_fixture_check.rs",
        "scripts/ci/assert-override-kill-switch.rs": "scripts/ci/assert-override-kill-switch.rs",
        "scripts/tests/phase0_override_kill_switch_check.rs": "scripts/tests/phase0_override_kill_switch_check.rs",
        "scripts/ci/assert-trusted-target-inventory.rs": "scripts/ci/assert-trusted-target-inventory.rs",
        "scripts/tests/phase0_trusted_target_inventory_check.rs": "scripts/tests/phase0_trusted_target_inventory_check.rs",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "scripts/tests/phase0_result_bundle_output_check.rs": "scripts/tests/phase0_result_bundle_output_check.rs",
        "scripts/ci/assert-phase0-aggregate-exit.rs": "scripts/ci/assert-phase0-aggregate-exit.rs",
        "scripts/tests/phase0_aggregate_exit_check.rs": "scripts/tests/phase0_aggregate_exit_check.rs",
        "scripts/ci/assert-automation-ratchet.rs": "scripts/ci/assert-automation-ratchet.rs",
        "scripts/tests/phase0_automation_ratchet_check.rs": "scripts/tests/phase0_automation_ratchet_check.rs",
        "scripts/ci/assert-claim-ceiling.rs": "scripts/ci/assert-claim-ceiling.rs",
        "scripts/tests/phase0_claim_ceiling_check.rs": "scripts/tests/phase0_claim_ceiling_check.rs",
        "scripts/ci/assert-buck2-cargo-target-coverage.rs": "scripts/ci/assert-buck2-cargo-target-coverage.rs",
        "scripts/tests/buck2_cargo_target_coverage_check.rs": "scripts/tests/buck2_cargo_target_coverage_check.rs",
        "scripts/ci/assert-phase0-merge-conflict-foundation.rs": "scripts/ci/assert-phase0-merge-conflict-foundation.rs",
        "scripts/tests/phase0_merge_conflict_foundation_check.rs": "scripts/tests/phase0_merge_conflict_foundation_check.rs",
        "scripts/ci/assert-service-root-classifier.rs": "scripts/ci/assert-service-root-classifier.rs",
        "scripts/tests/service_root_classifier_check.rs": "scripts/tests/service_root_classifier_check.rs",
        "scripts/ci/assert-status-enum-drift.rs": "scripts/ci/assert-status-enum-drift.rs",
        "scripts/tests/status_enum_drift_check.rs": "scripts/tests/status_enum_drift_check.rs",
        "specs/generated-artifact-registry.json": "specs/generated-artifact-registry.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-generated-artifact-unregistered.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-generated-artifact-unregistered.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-phase1-tide-batching-claim.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-phase1-tide-batching-claim.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-merge-tree-conflict.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-merge-tree-conflict.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "BUCK": "BUCK",
        "docs/standards/testing.md": "docs/standards/testing.md",
        "scripts/ci/assert-rust-testing-standard.rs": "scripts/ci/assert-rust-testing-standard.rs",
        "scripts/tests/rust_testing_standard_check.rs": "scripts/tests/rust_testing_standard_check.rs",
        "scripts/ci/assert-language-discipline.rs": "scripts/ci/assert-language-discipline.rs",
        "scripts/tests/language_discipline_check.rs": "scripts/tests/language_discipline_check.rs",
        "scripts/ci/assert-d1-seam-contracts.rs": "scripts/ci/assert-d1-seam-contracts.rs",
        "scripts/tests/d1_seam_contracts_check.rs": "scripts/tests/d1_seam_contracts_check.rs",
        "scripts/ci/assert-effective-dating-kernel.rs": "scripts/ci/assert-effective-dating-kernel.rs",
        "scripts/tests/effective_dating_kernel_check.rs": "scripts/tests/effective_dating_kernel_check.rs",
        "scripts/ci/assert-cross-artifact-agreement.rs": "scripts/ci/assert-cross-artifact-agreement.rs",
        "scripts/tests/cross_artifact_agreement_check.rs": "scripts/tests/cross_artifact_agreement_check.rs",
        "scripts/ci/assert-structural-lock-revert.rs": "scripts/ci/assert-structural-lock-revert.rs",
        "scripts/tests/structural_lock_revert_check.rs": "scripts/tests/structural_lock_revert_check.rs",
        "scripts/ci/assert-d1-read-your-writes-xfail.rs": "scripts/ci/assert-d1-read-your-writes-xfail.rs",
        "scripts/tests/d1_read_your_writes_xfail_check.rs": "scripts/tests/d1_read_your_writes_xfail_check.rs",
        "scripts/ci/assert-who-gates-gates.rs": "scripts/ci/assert-who-gates-gates.rs",
        "scripts/tests/who_gates_gates_check.rs": "scripts/tests/who_gates_gates_check.rs",
        "scripts/ci/assert-rust-llvm-coverage-runner-contract.rs": "scripts/ci/assert-rust-llvm-coverage-runner-contract.rs",
        "scripts/tests/rust_llvm_coverage_runner_contract_check.rs": "scripts/tests/rust_llvm_coverage_runner_contract_check.rs",
        "scripts/ci/run-rust-llvm-coverage-smoke.rs": "scripts/ci/run-rust-llvm-coverage-smoke.rs",
        "scripts/tests/rust_llvm_coverage_smoke_check.rs": "scripts/tests/rust_llvm_coverage_smoke_check.rs",
        "contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto": "contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto",
        "contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto": "contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto",
    } | {path: path for path in glob(["scripts/ci/assert-*.py", "scripts/tests/*.test.sh", "scripts/tests/*.py", "specs/fixtures/**/*.json", "specs/fixtures/**/*.rs", "specs/*.json"])},
    out = "phase0-red-green-fixture-contract-check.json",
    cmd = "mkdir -p $TMP/phase0-red-green-fixture-contract && rustc --edition=2021 -D warnings scripts/tests/red_green_fixture_contract_check.rs --test -o $TMP/phase0-red-green-fixture-contract/red_green_fixture_contract_check && OYA_REPO_ROOT=$PWD $TMP/phase0-red-green-fixture-contract/red_green_fixture_contract_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-red-green-fixture-contract.rs -o $TMP/phase0-red-green-fixture-contract/assert-red-green-fixture-contract && OYA_REPO_ROOT=$PWD $TMP/phase0-red-green-fixture-contract/assert-red-green-fixture-contract --spec specs/red-green-fixture-contract.json --json > $OUT",
    visibility = ["PUBLIC"],
)


# AC-0.15 merge-conflict foundation check: local/static seed registry evidence
# for generated artifacts, conflict taxonomy, merge-tree readiness fixtures, and
# one-lane-one-path/path-overlap fail-closed cases. This never posts statuses,
# proves full generated-output coverage, or claims Phase-1 Tide batching.
genrule(
    name = "phase0-merge-conflict-foundation-check",
    srcs = {
        "scripts/ci/assert-phase0-merge-conflict-foundation.rs": "scripts/ci/assert-phase0-merge-conflict-foundation.rs",
        "scripts/tests/phase0_merge_conflict_foundation_check.rs": "scripts/tests/phase0_merge_conflict_foundation_check.rs",
        "specs/generated-artifact-registry.json": "specs/generated-artifact-registry.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "Cargo.toml": "Cargo.toml",
        "Cargo.lock": "Cargo.lock",
        "reindeer.toml": "reindeer.toml",
        "scripts/ci/regen-third-party.sh": "scripts/ci/regen-third-party.sh",
        "scripts/ci/third-party-buckify-handedits.patch": "scripts/ci/third-party-buckify-handedits.patch",
        "third-party/BUCK": "third-party//:BUCK",
    } | {path: path for path in glob(["specs/fixtures/phase0-merge-conflict-foundation/*.json", "third-party/fixups/**/*.toml"])},
    out = "phase0-merge-conflict-foundation-check.json",
    cmd = "mkdir -p $TMP/phase0-merge-conflict-foundation && rustc --edition=2021 -D warnings scripts/tests/phase0_merge_conflict_foundation_check.rs --test -o $TMP/phase0-merge-conflict-foundation/phase0_merge_conflict_foundation_check && OYA_REPO_ROOT=$PWD $TMP/phase0-merge-conflict-foundation/phase0_merge_conflict_foundation_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-phase0-merge-conflict-foundation.rs -o $TMP/phase0-merge-conflict-foundation/assert-phase0-merge-conflict-foundation && OYA_REPO_ROOT=$PWD $TMP/phase0-merge-conflict-foundation/assert-phase0-merge-conflict-foundation --registry specs/generated-artifact-registry.json --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.1/P0.6/AC-0.7 service-root classifier check: local/static seed
# inventory evidence for closed-world roots, structural packet readiness,
# service-layout sprawl fixtures, retired REAL status-token rejection, duplicate
# service roots, and kebab-case crate naming. This never posts statuses, proves
# post-migration pure split, or claims Phase-0 completion.
genrule(
    name = "service-root-classifier-check",
    srcs = {
        "scripts/ci/assert-service-root-classifier.rs": "scripts/ci/assert-service-root-classifier.rs",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "scripts/tests/service_root_classifier_check.rs": "scripts/tests/service_root_classifier_check.rs",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/service-inventory.json": "specs/service-inventory.json",
        "specs/phase0-structural-packets.json": "specs/phase0-structural-packets.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-service-root-classifier/*.json", "oya/*", "cloud/*", "libs/*", "packs/*", "regional-packs/*", "platforms/*"])},
    out = "service-root-classifier-check.json",
    cmd = "mkdir -p $TMP/service-root-classifier && rustc --edition=2021 -D warnings scripts/tests/service_root_classifier_check.rs --test -o $TMP/service-root-classifier/service_root_classifier_check && OYA_REPO_ROOT=$PWD $TMP/service-root-classifier/service_root_classifier_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-service-root-classifier.rs -o $TMP/service-root-classifier/assert-service-root-classifier && $TMP/service-root-classifier/assert-service-root-classifier --inventory specs/service-inventory.json --packets specs/phase0-structural-packets.json --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.2 status-enum drift check: local/static seed evidence for the
# 3-axis decision/maturity/constraint status enum, retired REAL live-field
# rejection, and spec/code/manifest drift fixtures. This never posts statuses,
# proves full manifest/PRD conformance, or claims Phase-0 completion.
genrule(
    name = "status-enum-drift-check",
    srcs = {
        "scripts/ci/assert-status-enum-drift.rs": "scripts/ci/assert-status-enum-drift.rs",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "scripts/tests/status_enum_drift_check.rs": "scripts/tests/status_enum_drift_check.rs",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/status-enum-registry.json": "specs/status-enum-registry.json",
        "specs/microservices/real-estate.json": "specs/microservices/real-estate.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-status-enum-drift/*.json", "oya/real-estate/**", "oya/analytics/**"])},
    out = "status-enum-drift-check.json",
    cmd = "mkdir -p $TMP/status-enum-drift && rustc --edition=2021 -D warnings scripts/tests/status_enum_drift_check.rs --test -o $TMP/status-enum-drift/status_enum_drift_check && OYA_REPO_ROOT=$PWD $TMP/status-enum-drift/status_enum_drift_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-status-enum-drift.rs -o $TMP/status-enum-drift/assert-status-enum-drift && $TMP/status-enum-drift/assert-status-enum-drift --registry specs/status-enum-registry.json --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.3 ADR hygiene check: local/static evidence for duplicate ADR-number
# rejection, ADR-0511 -> ADR-0513 supersession, and active-doc stale
# canonical-reference linting. This never regenerates the full ADR index, posts
# statuses, or claims Phase-0 completion.
genrule(
    name = "adr-hygiene-check",
    srcs = {
        "scripts/ci/assert-adr-hygiene.py": "scripts/ci/assert-adr-hygiene.py",
        "scripts/tests/adr_hygiene_check.test.sh": "scripts/tests/adr_hygiene_check.test.sh",
        "specs/adr-hygiene-registry.json": "specs/adr-hygiene-registry.json",
        "docs/decisions/ADR-0377-forgejo-board-git-ref-cas-fallback.md": "docs/decisions/ADR-0377-forgejo-board-git-ref-cas-fallback.md",
        "docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md": "docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md",
        "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md": "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md",
        "docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md": "docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md",
        "docs/research/kafka-reeval-2026-05-28.md": "docs/research/kafka-reeval-2026-05-28.md",
        "docs/standards/logging-tracing.md": "docs/standards/logging-tracing.md",
    } | {path: path for path in glob(["specs/fixtures/phase0-adr-hygiene/*.json", "docs/decisions/ADR-*.md", "docs/standards/*.md", "specs/*.json"])},
    out = "adr-hygiene-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/adr_hygiene_check.test.sh > /dev/null && PYTHONDONTWRITEBYTECODE=1 python3 scripts/ci/assert-adr-hygiene.py --registry specs/adr-hygiene-registry.json --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.4 language-discipline check: Rust/Buck2 local/static evidence that new
# candidate-authored .py/.sh test sprawl is blocked outside the allowlist and
# the T0.4 cloud-check backlog inventory is preserved. This never posts
# statuses, scans a live PR, or claims Phase-0 completion.
genrule(
    name = "language-discipline-check",
    srcs = {
        "scripts/ci/assert-language-discipline.rs": "scripts/ci/assert-language-discipline.rs",
        "scripts/tests/language_discipline_check.rs": "scripts/tests/language_discipline_check.rs",
        "specs/language-discipline-registry.json": "specs/language-discipline-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-language-discipline/*.json"])},
    out = "language-discipline-check.json",
    cmd = "mkdir -p $TMP/language-discipline && rustc --edition=2021 -D warnings scripts/tests/language_discipline_check.rs --test -o $TMP/language-discipline/language_discipline_check && OYA_REPO_ROOT=$PWD $TMP/language-discipline/language_discipline_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-language-discipline.rs -o $TMP/language-discipline/assert-language-discipline && $TMP/language-discipline/assert-language-discipline --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.5/AC-0.10 D1 seam contract check: Rust/Buck2 local/static evidence that
# the shape-only A2a/A2b proto3 contracts exist, carry the proto-optional
# consistency_token seam, require token presence in Phase-0 fixtures, and keep
# topology-bearing fields conformance-gated. This never runs live D1
# conformance, posts statuses, or claims Phase-0 completion.
genrule(
    name = "d1-seam-contracts-check",
    srcs = {
        "scripts/ci/assert-d1-seam-contracts.rs": "scripts/ci/assert-d1-seam-contracts.rs",
        "scripts/tests/d1_seam_contracts_check.rs": "scripts/tests/d1_seam_contracts_check.rs",
        "specs/d1-seam-contracts-registry.json": "specs/d1-seam-contracts-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto": "contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto",
        "contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto": "contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto",
    } | {path: path for path in glob(["specs/fixtures/phase0-d1-seam-contracts/*.json"])},
    out = "d1-seam-contracts-check.json",
    cmd = "mkdir -p $TMP/d1-seam-contracts && rustc --edition=2021 -D warnings scripts/tests/d1_seam_contracts_check.rs --test -o $TMP/d1-seam-contracts/d1_seam_contracts_check && OYA_REPO_ROOT=$PWD $TMP/d1-seam-contracts/d1_seam_contracts_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-d1-seam-contracts.rs -o $TMP/d1-seam-contracts/assert-d1-seam-contracts && $TMP/d1-seam-contracts/assert-d1-seam-contracts --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)

# AC-0.6 effective-dating kernel check: Rust/Buck2 local/static evidence
# that ontology-kernel exposes a bitemporal effective-dated type, property
# fixtures cover valid-time x transaction-time as-of behavior, overlapping
# ranges are rejected, and non-monotonic transaction-time inserts are
# deterministic. This never posts statuses or claims Phase-0 completion.
genrule(
    name = "effective-dating-kernel-check",
    srcs = {
        "scripts/ci/assert-effective-dating-kernel.rs": "scripts/ci/assert-effective-dating-kernel.rs",
        "scripts/tests/effective_dating_kernel_check.rs": "scripts/tests/effective_dating_kernel_check.rs",
        "specs/effective-dating-kernel-registry.json": "specs/effective-dating-kernel-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "oya/ontology/crates/oya-ontology-kernel/src/effective_dating.rs": "//oya/ontology/crates/oya-ontology-kernel:effective-dating-src",
        "oya/ontology/crates/oya-ontology-kernel/src/lib.rs": "//oya/ontology/crates/oya-ontology-kernel:lib-src",
        "oya/ontology/crates/oya-ontology-kernel/BUCK": "//oya/ontology/crates/oya-ontology-kernel:BUCK",
        "effective-dating-kernel-tests.txt": "//oya/ontology/crates/oya-ontology-kernel:effective-dating-kernel-tests",
    } | {path: path for path in glob(["specs/fixtures/phase0-effective-dating-kernel/*.json"])},
    out = "effective-dating-kernel-check.json",
    cmd = "mkdir -p $TMP/effective-dating-kernel && rustc --edition=2021 -D warnings scripts/tests/effective_dating_kernel_check.rs --test -o $TMP/effective-dating-kernel/effective_dating_kernel_check && OYA_REPO_ROOT=$PWD $TMP/effective-dating-kernel/effective_dating_kernel_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-effective-dating-kernel.rs -o $TMP/effective-dating-kernel/assert-effective-dating-kernel && $TMP/effective-dating-kernel/assert-effective-dating-kernel --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.8 cross-artifact agreement check: Rust/Buck2 local/static
# evidence that backlog decision register #1..#21 has executable
# ADR/spec/masterplan/roadmap propagation packets, with fixture-backed REDs
# for missing agreement entries, generated artifact divergence, unreconciled
# idea-refine output, and packet omissions. This never posts statuses, adds an
# oya CLI surface, or claims Phase-0 completion.
genrule(
    name = "cross-artifact-agreement-check",
    srcs = {
        "scripts/ci/assert-cross-artifact-agreement.rs": "scripts/ci/assert-cross-artifact-agreement.rs",
        "scripts/tests/cross_artifact_agreement_check.rs": "scripts/tests/cross_artifact_agreement_check.rs",
        "specs/cross-artifact-agreement-registry.json": "specs/cross-artifact-agreement-registry.json",
        "specs/decision-propagation-packets.json": "specs/decision-propagation-packets.json",
        "docs/decisions/ADR-0365-automated-adr-lifecycle-and-propagation.md": "docs/decisions/ADR-0365-automated-adr-lifecycle-and-propagation.md",
        "docs/machine-readable/masterplan.generated.json": "docs/machine-readable/masterplan.generated.json",
        "docs/machine-readable/board-sync.generated.json": "docs/machine-readable/board-sync.generated.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-cross-artifact-agreement/*.json"])},
    out = "cross-artifact-agreement-check.json",
    cmd = "mkdir -p $TMP/cross-artifact-agreement && rustc --edition=2021 -D warnings scripts/tests/cross_artifact_agreement_check.rs --test -o $TMP/cross-artifact-agreement/cross_artifact_agreement_check && OYA_REPO_ROOT=$PWD $TMP/cross-artifact-agreement/cross_artifact_agreement_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-cross-artifact-agreement.rs -o $TMP/cross-artifact-agreement/assert-cross-artifact-agreement && $TMP/cross-artifact-agreement/assert-cross-artifact-agreement --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.9 structural-lock/revert check: Rust/Buck2 local/static evidence that
# structural artifact edits carry serialized path ownership, protected-flow
# revert evidence, RED fixtures for overlap/stale/false-authority cases, and
# explicit advisory-until-required-context boundaries. This never posts statuses
# or claims mechanical lock, P0.0 green, or Phase-0 completion.
genrule(
    name = "structural-lock-revert-check",
    srcs = {
        "scripts/ci/assert-structural-lock-revert.rs": "scripts/ci/assert-structural-lock-revert.rs",
        "scripts/tests/structural_lock_revert_check.rs": "scripts/tests/structural_lock_revert_check.rs",
        "specs/structural-lock-revert-registry.json": "specs/structural-lock-revert-registry.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-structural-lock-revert/*.json"])},
    out = "structural-lock-revert-check.json",
    cmd = "mkdir -p $TMP/structural-lock-revert && rustc --edition=2021 -D warnings scripts/tests/structural_lock_revert_check.rs --test -o $TMP/structural-lock-revert/structural_lock_revert_check && OYA_REPO_ROOT=$PWD $TMP/structural-lock-revert/structural_lock_revert_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-structural-lock-revert.rs -o $TMP/structural-lock-revert/assert-structural-lock-revert && $TMP/structural-lock-revert/assert-structural-lock-revert --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.10b D1 read-your-writes XFAIL check: Rust/Buck2 local/static
# evidence that the in-memory probe stays consistency-token aware and XFAIL
# classified until Phase-2 live D1 conformance evidence lands. This never posts
# statuses, mutates branch protection, adds an `oya` CLI surface, or claims
# P0.0 green / Phase-0 completion.
genrule(
    name = "d1-read-your-writes-xfail-check",
    srcs = {
        "scripts/ci/assert-d1-read-your-writes-xfail.rs": "scripts/ci/assert-d1-read-your-writes-xfail.rs",
        "scripts/tests/d1_read_your_writes_xfail_check.rs": "scripts/tests/d1_read_your_writes_xfail_check.rs",
        "specs/d1-read-your-writes-xfail-registry.json": "specs/d1-read-your-writes-xfail-registry.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-d1-read-your-writes-xfail/*.json"])},
    out = "d1-read-your-writes-xfail-check.json",
    cmd = "mkdir -p $TMP/d1-read-your-writes-xfail && rustc --edition=2021 -D warnings scripts/tests/d1_read_your_writes_xfail_check.rs --test -o $TMP/d1-read-your-writes-xfail/d1_read_your_writes_xfail_check && OYA_REPO_ROOT=$PWD $TMP/d1-read-your-writes-xfail/d1_read_your_writes_xfail_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-d1-read-your-writes-xfail.rs -o $TMP/d1-read-your-writes-xfail/assert-d1-read-your-writes-xfail && $TMP/d1-read-your-writes-xfail/assert-d1-read-your-writes-xfail --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)


# AC-0.11 who-gates-the-gates meta-check: Rust/Buck2 local/static
# evidence that every integrity gate carries known-bad fixtures,
# self-mutation probes, and non-vacuous pass conditions before its evidence can
# count toward Phase-0. This never posts statuses, mutates branch protection,
# adds an `oya` CLI surface, or claims P0.0 green / Phase-0 completion.
genrule(
    name = "who-gates-gates-check",
    srcs = {
        "scripts/ci/assert-who-gates-gates.rs": "scripts/ci/assert-who-gates-gates.rs",
        "scripts/tests/who_gates_gates_check.rs": "scripts/tests/who_gates_gates_check.rs",
        "specs/who-gates-gates-registry.json": "specs/who-gates-gates-registry.json",
        "specs/red-green-fixture-contract.json": "specs/red-green-fixture-contract.json",
        "scripts/tests/red_green_fixture_contract_check.rs": "scripts/tests/red_green_fixture_contract_check.rs",
        "BUCK": "BUCK",
    } | {path: path for path in glob(["specs/fixtures/phase0-who-gates-gates/*.json"])},
    out = "who-gates-gates-check.json",
    cmd = "mkdir -p $TMP/who-gates-gates && rustc --edition=2021 -D warnings scripts/tests/who_gates_gates_check.rs --test -o $TMP/who-gates-gates/who_gates_gates_check && OYA_REPO_ROOT=$PWD $TMP/who-gates-gates/who_gates_gates_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-who-gates-gates.rs -o $TMP/who-gates-gates/assert-who-gates-gates && $TMP/who-gates-gates/assert-who-gates-gates --json > $OUT",
    cacheable = False,
    remote = False,
    repo_relative_root = True,
    visibility = ["PUBLIC"],
)

# AC-0.16 automation-ratchet fixture check: local/static coverage that
# every Phase-0 rule row is classified, mapped, fixture-backed, and not routed
# back to oya CLI authority. This never claims live required-context authority.
genrule(
    name = "phase0-automation-ratchet-check",
    srcs = {
        "scripts/ci/assert-automation-ratchet.rs": "scripts/ci/assert-automation-ratchet.rs",
        "scripts/tests/phase0_automation_ratchet_check.rs": "scripts/tests/phase0_automation_ratchet_check.rs",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-automation-ratchet/*.json"])},
    out = "phase0-automation-ratchet-check.txt",
    cmd = "mkdir -p $TMP/phase0-automation-ratchet && rustc --edition=2021 -D warnings scripts/tests/phase0_automation_ratchet_check.rs --test -o $TMP/phase0-automation-ratchet/phase0_automation_ratchet_check && OYA_REPO_ROOT=$PWD $TMP/phase0-automation-ratchet/phase0_automation_ratchet_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-automation-ratchet.rs -o $TMP/phase0-automation-ratchet/assert-automation-ratchet && OYA_REPO_ROOT=$PWD $TMP/phase0-automation-ratchet/assert-automation-ratchet --json > $OUT",
    visibility = ["PUBLIC"],
)


# AC-0.17 claim-ceiling fixture check: local/static coverage that
# regulated readiness/enforcement/security/completion language maps to allowed
# evidence tiers or explicit target/non-claim labels. This never claims live
# required-context authority, production readiness, or hyperscaler-grade status.
genrule(
    name = "phase0-claim-ceiling-check",
    srcs = {
        "scripts/ci/assert-claim-ceiling.rs": "scripts/ci/assert-claim-ceiling.rs",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "scripts/tests/phase0_claim_ceiling_check.rs": "scripts/tests/phase0_claim_ceiling_check.rs",
        "specs/phase0-claim-evidence-map.json": "specs/phase0-claim-evidence-map.json",
        "specs/hyperscaler-production-readiness-claim-contract.json": "specs/hyperscaler-production-readiness-claim-contract.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-claim-ceiling/*.json"])},
    out = "phase0-claim-ceiling-check.txt",
    cmd = "mkdir -p $TMP/phase0-claim-ceiling && rustc --edition=2021 -D warnings scripts/tests/phase0_claim_ceiling_check.rs --test -o $TMP/phase0-claim-ceiling/phase0_claim_ceiling_check && OYA_REPO_ROOT=$PWD $TMP/phase0-claim-ceiling/phase0_claim_ceiling_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-claim-ceiling.rs -o $TMP/phase0-claim-ceiling/assert-claim-ceiling && $TMP/phase0-claim-ceiling/assert-claim-ceiling --json > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 structured result-bundle fixture check: local/static coverage
# that schema-conforming RED/false-green result bundles cannot imply live
# required-context authority or Phase-0 completion. This never posts statuses.
genrule(
    name = "phase0-result-bundle-output-check",
    srcs = {
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "scripts/tests/phase0_result_bundle_output_check.rs": "scripts/tests/phase0_result_bundle_output_check.rs",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-ci-enforcement-result-schema.json": "specs/phase0-ci-enforcement-result-schema.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json",
    },
    out = "phase0-result-bundle-output-check.txt",
    cmd = "mkdir -p $TMP/phase0-result-bundle-output && rustc --edition=2021 -D warnings scripts/tests/phase0_result_bundle_output_check.rs --test -o $TMP/phase0-result-bundle-output/phase0_result_bundle_output_check && OYA_REPO_ROOT=$PWD $TMP/phase0-result-bundle-output/phase0_result_bundle_output_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-result-bundle-output.rs -o $TMP/phase0-result-bundle-output/assert-result-bundle-output && $TMP/phase0-result-bundle-output/assert-result-bundle-output --json > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 trusted target-inventory fixture check: local/static GOOD/BAD
# coverage that candidate PR bytes cannot author the required Buck2 target
# inventory. This does not claim live cloud-ci/controller target authority.
genrule(
    name = "phase0-trusted-target-inventory-check",
    srcs = {
        "scripts/ci/assert-trusted-target-inventory.rs": "scripts/ci/assert-trusted-target-inventory.rs",
        "scripts/tests/phase0_trusted_target_inventory_check.rs": "scripts/tests/phase0_trusted_target_inventory_check.rs",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "scripts/tests/phase0_result_bundle_output_check.rs": "scripts/tests/phase0_result_bundle_output_check.rs",
        "specs/phase0-trusted-target-inventory-schema.json": "specs/phase0-trusted-target-inventory-schema.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json",
    },
    out = "phase0-trusted-target-inventory-check.txt",
    cmd = "mkdir -p $TMP/phase0-trusted-target-inventory && rustc --edition=2021 -D warnings scripts/tests/phase0_trusted_target_inventory_check.rs --test -o $TMP/phase0-trusted-target-inventory/phase0_trusted_target_inventory_check && OYA_REPO_ROOT=$PWD $TMP/phase0-trusted-target-inventory/phase0_trusted_target_inventory_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-trusted-target-inventory.rs -o $TMP/phase0-trusted-target-inventory/assert-trusted-target-inventory && $TMP/phase0-trusted-target-inventory/assert-trusted-target-inventory --json > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 tenant-pipeline isolation fixture check: local/static GOOD/BAD coverage
# for the 11 required separation surfaces. This does not claim live tenant
# isolation until trusted cloud-ci/oya-ci runs the gate on candidate SHAs.
genrule(
    name = "phase0-tenant-isolation-fixture-check",
    srcs = {
        "scripts/ci/assert-tenant-pipeline-isolation.rs": "scripts/ci/assert-tenant-pipeline-isolation.rs",
        "scripts/tests/phase0_tenant_isolation_fixture_check.rs": "scripts/tests/phase0_tenant_isolation_fixture_check.rs",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/toolchain-tenant-isolation-fixtures.json": "specs/toolchain-tenant-isolation-fixtures.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.3-bad-cross-tenant-shared-cache.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.3-bad-cross-tenant-shared-cache.json",
    },
    out = "phase0-tenant-isolation-fixture-check.txt",
    cmd = "mkdir -p $TMP/phase0-tenant-isolation && rustc --edition=2021 -D warnings scripts/tests/phase0_tenant_isolation_fixture_check.rs --test -o $TMP/phase0-tenant-isolation/phase0_tenant_isolation_fixture_check && OYA_REPO_ROOT=$PWD $TMP/phase0-tenant-isolation/phase0_tenant_isolation_fixture_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-tenant-pipeline-isolation.rs -o $TMP/phase0-tenant-isolation/assert-tenant-pipeline-isolation && $TMP/phase0-tenant-isolation/assert-tenant-pipeline-isolation --json > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 override/kill-switch fixture check: local/static GOOD/BAD coverage
# for TTL, reviewer, audit-chain, owner, blast-radius, revert/fix follow-up,
# affected context/gate, and no-new-oya-CLI fields. This does not claim live
# protected-flow override authority.
genrule(
    name = "phase0-override-kill-switch-check",
    srcs = {
        "scripts/ci/assert-override-kill-switch.rs": "scripts/ci/assert-override-kill-switch.rs",
        "scripts/tests/phase0_override_kill_switch_check.rs": "scripts/tests/phase0_override_kill_switch_check.rs",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-override-packet-schema.json": "specs/phase0-override-packet-schema.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json",
    },
    out = "phase0-override-kill-switch-check.txt",
    cmd = "mkdir -p $TMP/phase0-override-kill-switch && rustc --edition=2021 -D warnings scripts/tests/phase0_override_kill_switch_check.rs --test -o $TMP/phase0-override-kill-switch/phase0_override_kill_switch_check && OYA_REPO_ROOT=$PWD $TMP/phase0-override-kill-switch/phase0_override_kill_switch_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-override-kill-switch.rs -o $TMP/phase0-override-kill-switch/assert-override-kill-switch && $TMP/phase0-override-kill-switch/assert-override-kill-switch --json > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 required-status source binding check: GitHub branch-protection
# required_status_checks.checks must bind oya-ci-required to the trusted
# cloud-ci/oya-ci source app before any green claim. This target is fixture-only
# local evidence; the live dev read remains a RED artifact until the app is bound.
genrule(
    name = "phase0-required-status-source-check",
    srcs = {
        "scripts/ci/assert-required-status-source.rs": "scripts/ci/assert-required-status-source.rs",
        "scripts/tests/phase0_required_status_source_check.rs": "scripts/tests/phase0_required_status_source_check.rs",
    } | {path: path for path in glob(["specs/fixtures/phase0-required-status-source/*.json"])},
    out = "phase0-required-status-source-check.txt",
    cmd = "mkdir -p $TMP/phase0-required-status-source && rustc --edition=2021 -D warnings scripts/tests/phase0_required_status_source_check.rs --test -o $TMP/phase0-required-status-source/phase0_required_status_source_check && OYA_REPO_ROOT=$PWD $TMP/phase0-required-status-source/phase0_required_status_source_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-required-status-source.rs -o $TMP/phase0-required-status-source/assert-required-status-source && $TMP/phase0-required-status-source/assert-required-status-source --input specs/fixtures/phase0-required-status-source/good-bound-expected-source-app.json --expected-app-id 12345 --json > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 auto-merge-after-CI bridge check: Forgejo dry-run payload must use the
# cloud-ci/oya-ci required context, schedule merge after checks pass, and pin the
# PR head SHA. This is local/static evidence, not a live Forgejo mutation.
genrule(
    name = "forgejo-auto-merge-after-ci-check",
    srcs = {
        "scripts/tests/forgejo_auto_merge_after_ci.test.sh": "scripts/tests/forgejo_auto_merge_after_ci.test.sh",
        "scripts/ci/arm-auto-merge.sh": "scripts/ci/arm-auto-merge.sh",
        "docs/ci/auto-merge-flow.md": "docs/ci/auto-merge-flow.md",
        "docs/ci/forge-of-record.md": "docs/ci/forge-of-record.md",
    },
    out = "forgejo-auto-merge-after-ci-check.txt",
    cmd = "bash scripts/tests/forgejo_auto_merge_after_ci.test.sh > $OUT",
    visibility = ["PUBLIC"],
)

# P0.0 GitHub bootstrap mirror auto-merge check: live scheduling must refuse
# github-lane-unlocker-required drift and non-squash merge methods before
# arming auto-merge. This is local/static evidence over a fake gh CLI, not a
# live GitHub mutation.
genrule(
    name = "github-auto-merge-after-ci-check",
    srcs = {
        "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh": "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh",
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh": "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh": "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh",
        "scripts/tests/phase0_required_context_rollup_check.rs": "scripts/tests/phase0_required_context_rollup_check.rs",
        "scripts/ci/assert-pr-required-context.rs": "scripts/ci/assert-pr-required-context.rs",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "specs/fixtures/phase0-required-context-rollup/good-github-lane-unlocker-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json": "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json",
        "specs/fixtures/phase0-required-context-rollup/bad-missing-github-lane-unlocker-required.json": "specs/fixtures/phase0-required-context-rollup/bad-missing-github-lane-unlocker-required.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-failure.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-completed-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-completed-failure.json",
        "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json",
        "scripts/trigger-next-queue-automerge.sh": "scripts/trigger-next-queue-automerge.sh",
        "scripts/check-sequential-pr-merge-conflicts.sh": "scripts/check-sequential-pr-merge-conflicts.sh",
        "infra/branch-protection/dev.json": "infra/branch-protection/dev.json",
    },
    out = "github-auto-merge-after-ci-check.txt",
    cmd = "(bash scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh && bash scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh && bash scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh && mkdir -p $TMP/required-context-rollup && rustc --edition=2021 -D warnings scripts/tests/phase0_required_context_rollup_check.rs --test -o $TMP/required-context-rollup/phase0_required_context_rollup_check && OYA_REPO_ROOT=$PWD $TMP/required-context-rollup/phase0_required_context_rollup_check > /dev/null && rustc --edition=2021 -D warnings scripts/ci/assert-pr-required-context.rs -o $TMP/required-context-rollup/assert-pr-required-context && $TMP/required-context-rollup/assert-pr-required-context --input specs/fixtures/phase0-required-context-rollup/good-github-lane-unlocker-required-success.json --json > /dev/null) > $OUT",
    visibility = ["PUBLIC"],
)

# P0.0 auto-merge-after-CI contract check: closes the checked-in Forgejo/GitHub
# auto-merge contract over scripts, docs, Tide adapter code, and Buck2 policy.
genrule(
    name = "phase0-auto-merge-after-ci-contract-check",
    srcs = {
        "scripts/tests/phase0_auto_merge_after_ci_contract_check.py": "scripts/tests/phase0_auto_merge_after_ci_contract_check.py",
        "specs/phase0-auto-merge-after-ci.json": "specs/phase0-auto-merge-after-ci.json",
        "specs/buck2-authority-policy.json": "specs/buck2-authority-policy.json",
        "scripts/ci/arm-auto-merge.sh": "scripts/ci/arm-auto-merge.sh",
        "scripts/trigger-next-queue-automerge.sh": "scripts/trigger-next-queue-automerge.sh",
        "scripts/check-sequential-pr-merge-conflicts.sh": "scripts/check-sequential-pr-merge-conflicts.sh",
        "scripts/tests/forgejo_auto_merge_after_ci.test.sh": "scripts/tests/forgejo_auto_merge_after_ci.test.sh",
        "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh": "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh",
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh": "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh": "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh",
        "scripts/tests/phase0_required_context_rollup_check.rs": "scripts/tests/phase0_required_context_rollup_check.rs",
        "scripts/ci/assert-pr-required-context.rs": "scripts/ci/assert-pr-required-context.rs",
        "scripts/ci/assert-result-bundle-output.rs": "scripts/ci/assert-result-bundle-output.rs",
        "specs/fixtures/phase0-required-context-rollup/good-github-lane-unlocker-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json": "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json",
        "specs/fixtures/phase0-required-context-rollup/bad-missing-github-lane-unlocker-required.json": "specs/fixtures/phase0-required-context-rollup/bad-missing-github-lane-unlocker-required.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-failure.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-completed-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-completed-failure.json",
        "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-nested-github-lane-unlocker-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-github-lane-unlocker-required-success-untrusted-producer.json",
        "docs/ci/auto-merge-flow.md": "docs/ci/auto-merge-flow.md",
        "docs/ci/forge-of-record.md": "docs/ci/forge-of-record.md",
        "oya/ci-tide/crates/oya-ci-tide-kernel/src/lib.rs": "//oya/ci-tide/crates/oya-ci-tide-kernel:lib-src",
        "oya/ci-tide/crates/oya-ci-tide-app/src/lib.rs": "//oya/ci-tide/crates/oya-ci-tide-app:lib-src",
        "oya/ci-tide/crates/oya-ci-tide-forgejo-adapter/src/lib.rs": "//oya/ci-tide/crates/oya-ci-tide-forgejo-adapter:lib-src",
    },
    out = "phase0-auto-merge-after-ci-contract-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 OYA_REPO_ROOT=$PWD python3 scripts/tests/phase0_auto_merge_after_ci_contract_check.py > $OUT",
    visibility = ["PUBLIC"],
)
