# A list of available rules and their signatures can be found here: https://buck2.build/docs/prelude/globals/

genrule(
    name = "hello_world",
    out = "out.txt",
    cmd = "echo BUILT BY BUCK2> $OUT",
)


# Buck2 root BUCK dialect rejects def-based src helpers in this repository;
# keep current service CI files explicitly declared here, while
# specs/buck2-authority-policy.json expands command_scan_globs
# (cloud/*/ci/Jenkinsfile, oya/*/ci/Jenkinsfile) in the Python scanner.
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
        "scripts/ci/enforce-buck2-authority.py": "scripts/ci/enforce-buck2-authority.py",
        "scripts/ci/oya-ci-post.sh": "scripts/ci/oya-ci-post.sh",
        "scripts/ci/assert-pr-required-context.py": "scripts/ci/assert-pr-required-context.py",
        "scripts/tests/phase0_required_context_rollup_check.test.sh": "scripts/tests/phase0_required_context_rollup_check.test.sh",
        "scripts/ci/assert-required-status-source.py": "scripts/ci/assert-required-status-source.py",
        "scripts/tests/phase0_required_status_source_check.test.sh": "scripts/tests/phase0_required_status_source_check.test.sh",
        "scripts/ci/assert-tenant-pipeline-isolation.py": "scripts/ci/assert-tenant-pipeline-isolation.py",
        "scripts/tests/phase0_tenant_isolation_fixture_check.test.sh": "scripts/tests/phase0_tenant_isolation_fixture_check.test.sh",
        "scripts/ci/assert-override-kill-switch.py": "scripts/ci/assert-override-kill-switch.py",
        "scripts/tests/phase0_override_kill_switch_check.test.sh": "scripts/tests/phase0_override_kill_switch_check.test.sh",
        "scripts/ci/assert-trusted-target-inventory.py": "scripts/ci/assert-trusted-target-inventory.py",
        "scripts/tests/phase0_trusted_target_inventory_check.test.sh": "scripts/tests/phase0_trusted_target_inventory_check.test.sh",
        "scripts/ci/assert-result-bundle-output.py": "scripts/ci/assert-result-bundle-output.py",
        "scripts/tests/phase0_result_bundle_output_check.test.sh": "scripts/tests/phase0_result_bundle_output_check.test.sh",
        "scripts/ci/assert-automation-ratchet.py": "scripts/ci/assert-automation-ratchet.py",
        "scripts/tests/phase0_automation_ratchet_check.test.sh": "scripts/tests/phase0_automation_ratchet_check.test.sh",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
        "scripts/ci/assert-claim-ceiling.py": "scripts/ci/assert-claim-ceiling.py",
        "scripts/tests/phase0_claim_ceiling_check.test.sh": "scripts/tests/phase0_claim_ceiling_check.test.sh",
        "specs/phase0-claim-evidence-map.json": "specs/phase0-claim-evidence-map.json",
        "specs/hyperscaler-production-readiness-claim-contract.json": "specs/hyperscaler-production-readiness-claim-contract.json",
        "scripts/ci/assert-phase0-aggregate-exit.py": "scripts/ci/assert-phase0-aggregate-exit.py",
        "scripts/tests/phase0_aggregate_exit_check.test.sh": "scripts/tests/phase0_aggregate_exit_check.test.sh",
        "scripts/ci/assert-rust-testing-standard.py": "scripts/ci/assert-rust-testing-standard.py",
        "scripts/tests/rust_testing_standard_check.test.sh": "scripts/tests/rust_testing_standard_check.test.sh",
        "scripts/ci/assert-rust-llvm-coverage-runner-contract.py": "scripts/ci/assert-rust-llvm-coverage-runner-contract.py",
        "scripts/tests/rust_llvm_coverage_runner_contract_check.test.sh": "scripts/tests/rust_llvm_coverage_runner_contract_check.test.sh",
        "specs/rust-llvm-coverage-runner-contract.json": "specs/rust-llvm-coverage-runner-contract.json",
        "scripts/ci/run-rust-llvm-coverage-smoke.py": "scripts/ci/run-rust-llvm-coverage-smoke.py",
        "scripts/tests/rust_llvm_coverage_smoke_check.test.sh": "scripts/tests/rust_llvm_coverage_smoke_check.test.sh",
        "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs": "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs",
        "scripts/ci/assert-buck2-cargo-target-coverage.py": "scripts/ci/assert-buck2-cargo-target-coverage.py",
        "scripts/tests/buck2_cargo_target_coverage_check.test.sh": "scripts/tests/buck2_cargo_target_coverage_check.test.sh",
        "specs/buck2-cargo-target-coverage.json": "specs/buck2-cargo-target-coverage.json",
        "scripts/ci/assert-red-green-fixture-contract.py": "scripts/ci/assert-red-green-fixture-contract.py",
        "scripts/tests/red_green_fixture_contract_check.test.sh": "scripts/tests/red_green_fixture_contract_check.test.sh",
        "specs/red-green-fixture-contract.json": "specs/red-green-fixture-contract.json",
        "scripts/ci/assert-phase0-merge-conflict-foundation.py": "scripts/ci/assert-phase0-merge-conflict-foundation.py",
        "scripts/tests/phase0_merge_conflict_foundation_check.test.sh": "scripts/tests/phase0_merge_conflict_foundation_check.test.sh",
        "specs/generated-artifact-registry.json": "specs/generated-artifact-registry.json",
        "scripts/ci/assert-service-root-classifier.py": "scripts/ci/assert-service-root-classifier.py",
        "scripts/tests/service_root_classifier_check.test.sh": "scripts/tests/service_root_classifier_check.test.sh",
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
        "scripts/ci/assert-status-enum-drift.py": "scripts/ci/assert-status-enum-drift.py",
        "scripts/tests/status_enum_drift_check.test.sh": "scripts/tests/status_enum_drift_check.test.sh",
        "specs/status-enum-registry.json": "specs/status-enum-registry.json",
        "scripts/ci/assert-adr-hygiene.py": "scripts/ci/assert-adr-hygiene.py",
        "scripts/tests/adr_hygiene_check.test.sh": "scripts/tests/adr_hygiene_check.test.sh",
        "specs/adr-hygiene-registry.json": "specs/adr-hygiene-registry.json",
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
        "specs/fixtures/phase0-required-context-rollup/good-oya-ci-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-oya-ci-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json": "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json",
        "specs/fixtures/phase0-required-context-rollup/bad-missing-oya-ci-required.json": "specs/fixtures/phase0-required-context-rollup/bad-missing-oya-ci-required.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-failure.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-completed-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-completed-failure.json",
        "specs/fixtures/phase0-required-context-rollup/good-nested-cloud-ci-oya-ci-success.json": "specs/fixtures/phase0-required-context-rollup/good-nested-cloud-ci-oya-ci-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-missing-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-untrusted-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-untrusted-producer.json",
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
        ".github/branch-protection.yaml": ".github/branch-protection.yaml",
        "infra/branch-protection/dev.json": "infra/branch-protection/dev.json",
        "infra/ci/jenkins/reported-status-contexts.json": "infra/ci/jenkins/reported-status-contexts.json",
        "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.py": "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.py",
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
    cmd = "PYTHONDONTWRITEBYTECODE=1 OYA_REPO_ROOT=$PWD python3 scripts/ci/enforce-buck2-authority.py --policy specs/buck2-authority-policy.json > $OUT",
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
        "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.py": "scripts/tests/phase0_ci_enforcement_baseline_catalog_check.py",
        "specs/phase0-ci-enforcement-baseline.json": "specs/phase0-ci-enforcement-baseline.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-claim-evidence-map.json": "specs/phase0-claim-evidence-map.json",
        "specs/phase0-ci-enforcement-result-schema.json": "specs/phase0-ci-enforcement-result-schema.json",
        "specs/phase0-override-packet-schema.json": "specs/phase0-override-packet-schema.json",
        "specs/phase0-trusted-target-inventory-schema.json": "specs/phase0-trusted-target-inventory-schema.json",
        "specs/toolchain-tenant-isolation-fixtures.json": "specs/toolchain-tenant-isolation-fixtures.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-ci-enforcement-baseline/*.json"])} | {path: path for path in glob(["specs/fixtures/phase0-required-status-source/*.json"])},
    out = "phase0-ci-enforcement-baseline-catalog-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 OYA_REPO_ROOT=$PWD python3 scripts/tests/phase0_ci_enforcement_baseline_catalog_check.py > $OUT",
    visibility = ["PUBLIC"],
)


# AC-0.12 aggregate-exit fixture check: local/static coverage that
# Phase-0 cannot pass on a partial, omitted, unknown, or false subcondition.
# This never claims live required-context authority, P0.0 green, Phase-0
# completion, production readiness, or hyperscaler-grade status.
genrule(
    name = "phase0-aggregate-exit-check",
    srcs = {
        "scripts/ci/assert-phase0-aggregate-exit.py": "scripts/ci/assert-phase0-aggregate-exit.py",
        "scripts/tests/phase0_aggregate_exit_check.test.sh": "scripts/tests/phase0_aggregate_exit_check.test.sh",
    } | {path: path for path in glob(["specs/fixtures/phase0-exit-gate/*.json"])},
    out = "phase0-aggregate-exit-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_aggregate_exit_check.test.sh > $OUT",
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
        "scripts/ci/assert-rust-testing-standard.py": "scripts/ci/assert-rust-testing-standard.py",
        "scripts/tests/rust_testing_standard_check.test.sh": "scripts/tests/rust_testing_standard_check.test.sh",
        "docs/standards/testing.md": "docs/standards/testing.md",
    },
    out = "rust-testing-standard-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/rust_testing_standard_check.test.sh > $OUT",
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
        "scripts/ci/assert-rust-llvm-coverage-runner-contract.py": "scripts/ci/assert-rust-llvm-coverage-runner-contract.py",
        "scripts/tests/rust_llvm_coverage_runner_contract_check.test.sh": "scripts/tests/rust_llvm_coverage_runner_contract_check.test.sh",
        "specs/rust-llvm-coverage-runner-contract.json": "specs/rust-llvm-coverage-runner-contract.json",
    },
    out = "rust-llvm-coverage-runner-contract-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/rust_llvm_coverage_runner_contract_check.test.sh > $OUT",
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
        "scripts/ci/run-rust-llvm-coverage-smoke.py": "scripts/ci/run-rust-llvm-coverage-smoke.py",
        "scripts/tests/rust_llvm_coverage_smoke_check.test.sh": "scripts/tests/rust_llvm_coverage_smoke_check.test.sh",
        "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs": "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs",
    },
    out = "rust-llvm-coverage-smoke-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/rust_llvm_coverage_smoke_check.test.sh > /dev/null && PYTHONDONTWRITEBYTECODE=1 python3 scripts/ci/run-rust-llvm-coverage-smoke.py --out $OUT > /dev/null",
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
        "scripts/ci/assert-buck2-cargo-target-coverage.py": "scripts/ci/assert-buck2-cargo-target-coverage.py",
        "scripts/tests/buck2_cargo_target_coverage_check.test.sh": "scripts/tests/buck2_cargo_target_coverage_check.test.sh",
        "specs/buck2-cargo-target-coverage.json": "specs/buck2-cargo-target-coverage.json",
        "Cargo.toml": "Cargo.toml",
    } | {path: path for path in glob(["**/Cargo.toml", "**/BUCK"], exclude = ["buck-out/**", "target/**"])},
    out = "buck2-cargo-target-coverage-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/buck2_cargo_target_coverage_check.test.sh > /dev/null && PYTHONDONTWRITEBYTECODE=1 python3 scripts/ci/assert-buck2-cargo-target-coverage.py --spec specs/buck2-cargo-target-coverage.json --json > $OUT",
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
        "scripts/ci/assert-red-green-fixture-contract.py": "scripts/ci/assert-red-green-fixture-contract.py",
        "scripts/tests/red_green_fixture_contract_check.test.sh": "scripts/tests/red_green_fixture_contract_check.test.sh",
        "specs/red-green-fixture-contract.json": "specs/red-green-fixture-contract.json",
        "scripts/ci/assert-phase0-merge-conflict-foundation.py": "scripts/ci/assert-phase0-merge-conflict-foundation.py",
        "scripts/tests/phase0_merge_conflict_foundation_check.test.sh": "scripts/tests/phase0_merge_conflict_foundation_check.test.sh",
        "specs/generated-artifact-registry.json": "specs/generated-artifact-registry.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-generated-artifact-unregistered.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-generated-artifact-unregistered.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-phase1-tide-batching-claim.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-phase1-tide-batching-claim.json",
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-merge-tree-conflict.json": "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-merge-tree-conflict.json",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "BUCK": "BUCK",
        "docs/standards/testing.md": "docs/standards/testing.md",
    } | {path: path for path in glob(["scripts/ci/assert-*.py", "scripts/ci/run-rust-llvm-coverage-smoke.py", "scripts/tests/*.test.sh", "scripts/tests/*.py", "specs/fixtures/**/*.json", "specs/fixtures/**/*.rs", "specs/*.json"])},
    out = "phase0-red-green-fixture-contract-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/red_green_fixture_contract_check.test.sh > /dev/null && PYTHONDONTWRITEBYTECODE=1 python3 scripts/ci/assert-red-green-fixture-contract.py --spec specs/red-green-fixture-contract.json --json > $OUT",
    visibility = ["PUBLIC"],
)


# AC-0.15 merge-conflict foundation check: local/static seed registry evidence
# for generated artifacts, conflict taxonomy, merge-tree readiness fixtures, and
# one-lane-one-path/path-overlap fail-closed cases. This never posts statuses,
# proves full generated-output coverage, or claims Phase-1 Tide batching.
genrule(
    name = "phase0-merge-conflict-foundation-check",
    srcs = {
        "scripts/ci/assert-phase0-merge-conflict-foundation.py": "scripts/ci/assert-phase0-merge-conflict-foundation.py",
        "scripts/tests/phase0_merge_conflict_foundation_check.test.sh": "scripts/tests/phase0_merge_conflict_foundation_check.test.sh",
        "specs/generated-artifact-registry.json": "specs/generated-artifact-registry.json",
        "Cargo.toml": "Cargo.toml",
        "Cargo.lock": "Cargo.lock",
        "reindeer.toml": "reindeer.toml",
        "scripts/ci/regen-third-party.sh": "scripts/ci/regen-third-party.sh",
        "scripts/ci/third-party-buckify-handedits.patch": "scripts/ci/third-party-buckify-handedits.patch",
        "third-party/BUCK": "third-party//:BUCK",
    } | {path: path for path in glob(["specs/fixtures/phase0-merge-conflict-foundation/*.json", "third-party/fixups/**/*.toml"])},
    out = "phase0-merge-conflict-foundation-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_merge_conflict_foundation_check.test.sh > /dev/null && PYTHONDONTWRITEBYTECODE=1 python3 scripts/ci/assert-phase0-merge-conflict-foundation.py --registry specs/generated-artifact-registry.json --json > $OUT",
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
        "scripts/ci/assert-service-root-classifier.py": "scripts/ci/assert-service-root-classifier.py",
        "scripts/tests/service_root_classifier_check.test.sh": "scripts/tests/service_root_classifier_check.test.sh",
        "specs/service-inventory.json": "specs/service-inventory.json",
        "specs/phase0-structural-packets.json": "specs/phase0-structural-packets.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-service-root-classifier/*.json", "oya/*", "cloud/*", "libs/*", "packs/*", "regional-packs/*", "platforms/*"])},
    out = "service-root-classifier-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/service_root_classifier_check.test.sh > /dev/null && PYTHONDONTWRITEBYTECODE=1 python3 scripts/ci/assert-service-root-classifier.py --inventory specs/service-inventory.json --packets specs/phase0-structural-packets.json --json > $OUT",
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
        "scripts/ci/assert-status-enum-drift.py": "scripts/ci/assert-status-enum-drift.py",
        "scripts/tests/status_enum_drift_check.test.sh": "scripts/tests/status_enum_drift_check.test.sh",
        "specs/status-enum-registry.json": "specs/status-enum-registry.json",
        "specs/microservices/real-estate.json": "specs/microservices/real-estate.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-status-enum-drift/*.json", "oya/real-estate/**", "oya/analytics/**"])},
    out = "status-enum-drift-check.json",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/status_enum_drift_check.test.sh > /dev/null && PYTHONDONTWRITEBYTECODE=1 python3 scripts/ci/assert-status-enum-drift.py --registry specs/status-enum-registry.json --json > $OUT",
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


# AC-0.16 automation-ratchet fixture check: local/static coverage that
# every Phase-0 rule row is classified, mapped, fixture-backed, and not routed
# back to oya CLI authority. This never claims live required-context authority.
genrule(
    name = "phase0-automation-ratchet-check",
    srcs = {
        "scripts/ci/assert-automation-ratchet.py": "scripts/ci/assert-automation-ratchet.py",
        "scripts/tests/phase0_automation_ratchet_check.test.sh": "scripts/tests/phase0_automation_ratchet_check.test.sh",
        "specs/phase0-automation-matrix.json": "specs/phase0-automation-matrix.json",
        "specs/phase0-automation-coverage-registry.json": "specs/phase0-automation-coverage-registry.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-automation-ratchet/*.json"])},
    out = "phase0-automation-ratchet-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_automation_ratchet_check.test.sh > $OUT",
    visibility = ["PUBLIC"],
)


# AC-0.17 claim-ceiling fixture check: local/static coverage that
# regulated readiness/enforcement/security/completion language maps to allowed
# evidence tiers or explicit target/non-claim labels. This never claims live
# required-context authority, production readiness, or hyperscaler-grade status.
genrule(
    name = "phase0-claim-ceiling-check",
    srcs = {
        "scripts/ci/assert-claim-ceiling.py": "scripts/ci/assert-claim-ceiling.py",
        "scripts/tests/phase0_claim_ceiling_check.test.sh": "scripts/tests/phase0_claim_ceiling_check.test.sh",
        "specs/phase0-claim-evidence-map.json": "specs/phase0-claim-evidence-map.json",
        "specs/hyperscaler-production-readiness-claim-contract.json": "specs/hyperscaler-production-readiness-claim-contract.json",
    } | {path: path for path in glob(["specs/fixtures/phase0-claim-ceiling/*.json"])},
    out = "phase0-claim-ceiling-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_claim_ceiling_check.test.sh > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 structured result-bundle fixture check: local/static coverage
# that schema-conforming RED/false-green result bundles cannot imply live
# required-context authority or Phase-0 completion. This never posts statuses.
genrule(
    name = "phase0-result-bundle-output-check",
    srcs = {
        "scripts/ci/assert-result-bundle-output.py": "scripts/ci/assert-result-bundle-output.py",
        "scripts/tests/phase0_result_bundle_output_check.test.sh": "scripts/tests/phase0_result_bundle_output_check.test.sh",
        "specs/phase0-ci-enforcement-result-schema.json": "specs/phase0-ci-enforcement-result-schema.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json",
    },
    out = "phase0-result-bundle-output-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_result_bundle_output_check.test.sh > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 trusted target-inventory fixture check: local/static GOOD/BAD
# coverage that candidate PR bytes cannot author the required Buck2 target
# inventory. This does not claim live cloud-ci/controller target authority.
genrule(
    name = "phase0-trusted-target-inventory-check",
    srcs = {
        "scripts/ci/assert-trusted-target-inventory.py": "scripts/ci/assert-trusted-target-inventory.py",
        "scripts/tests/phase0_trusted_target_inventory_check.test.sh": "scripts/tests/phase0_trusted_target_inventory_check.test.sh",
        "scripts/ci/assert-result-bundle-output.py": "scripts/ci/assert-result-bundle-output.py",
        "scripts/tests/phase0_result_bundle_output_check.test.sh": "scripts/tests/phase0_result_bundle_output_check.test.sh",
        "specs/phase0-trusted-target-inventory-schema.json": "specs/phase0-trusted-target-inventory-schema.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json",
    },
    out = "phase0-trusted-target-inventory-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_trusted_target_inventory_check.test.sh > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 tenant-pipeline isolation fixture check: local/static GOOD/BAD coverage
# for the 11 required separation surfaces. This does not claim live tenant
# isolation until trusted cloud-ci/oya-ci runs the gate on candidate SHAs.
genrule(
    name = "phase0-tenant-isolation-fixture-check",
    srcs = {
        "scripts/ci/assert-tenant-pipeline-isolation.py": "scripts/ci/assert-tenant-pipeline-isolation.py",
        "scripts/tests/phase0_tenant_isolation_fixture_check.test.sh": "scripts/tests/phase0_tenant_isolation_fixture_check.test.sh",
        "specs/toolchain-tenant-isolation-fixtures.json": "specs/toolchain-tenant-isolation-fixtures.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.3-bad-cross-tenant-shared-cache.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.3-bad-cross-tenant-shared-cache.json",
    },
    out = "phase0-tenant-isolation-fixture-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_tenant_isolation_fixture_check.test.sh > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 override/kill-switch fixture check: local/static GOOD/BAD coverage
# for TTL, reviewer, audit-chain, owner, blast-radius, revert/fix follow-up,
# affected context/gate, and no-new-oya-CLI fields. This does not claim live
# protected-flow override authority.
genrule(
    name = "phase0-override-kill-switch-check",
    srcs = {
        "scripts/ci/assert-override-kill-switch.py": "scripts/ci/assert-override-kill-switch.py",
        "scripts/tests/phase0_override_kill_switch_check.test.sh": "scripts/tests/phase0_override_kill_switch_check.test.sh",
        "specs/phase0-override-packet-schema.json": "specs/phase0-override-packet-schema.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json",
        "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json",
    },
    out = "phase0-override-kill-switch-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_override_kill_switch_check.test.sh > $OUT",
    visibility = ["PUBLIC"],
)


# P0.0 required-status source binding check: GitHub branch-protection
# required_status_checks.checks must bind oya-ci-required to the trusted
# cloud-ci/oya-ci source app before any green claim. This target is fixture-only
# local evidence; the live dev read remains a RED artifact until the app is bound.
genrule(
    name = "phase0-required-status-source-check",
    srcs = {
        "scripts/ci/assert-required-status-source.py": "scripts/ci/assert-required-status-source.py",
        "scripts/tests/phase0_required_status_source_check.test.sh": "scripts/tests/phase0_required_status_source_check.test.sh",
    } | {path: path for path in glob(["specs/fixtures/phase0-required-status-source/*.json"])},
    out = "phase0-required-status-source-check.txt",
    cmd = "PYTHONDONTWRITEBYTECODE=1 bash scripts/tests/phase0_required_status_source_check.test.sh > $OUT",
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
# required-context drift and non-squash merge methods before arming auto-merge.
# This is local/static evidence over a fake gh CLI, not a live GitHub mutation.
genrule(
    name = "github-auto-merge-after-ci-check",
    srcs = {
        "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh": "scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh",
        "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh": "scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh",
        "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh": "scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh",
        "scripts/tests/phase0_required_context_rollup_check.test.sh": "scripts/tests/phase0_required_context_rollup_check.test.sh",
        "scripts/ci/assert-pr-required-context.py": "scripts/ci/assert-pr-required-context.py",
        "specs/fixtures/phase0-required-context-rollup/good-oya-ci-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-oya-ci-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json": "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json",
        "specs/fixtures/phase0-required-context-rollup/bad-missing-oya-ci-required.json": "specs/fixtures/phase0-required-context-rollup/bad-missing-oya-ci-required.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-failure.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-completed-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-completed-failure.json",
        "specs/fixtures/phase0-required-context-rollup/good-nested-cloud-ci-oya-ci-success.json": "specs/fixtures/phase0-required-context-rollup/good-nested-cloud-ci-oya-ci-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-missing-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-untrusted-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-untrusted-producer.json",
        "scripts/trigger-next-queue-automerge.sh": "scripts/trigger-next-queue-automerge.sh",
        "scripts/check-sequential-pr-merge-conflicts.sh": "scripts/check-sequential-pr-merge-conflicts.sh",
        "infra/branch-protection/dev.json": "infra/branch-protection/dev.json",
    },
    out = "github-auto-merge-after-ci-check.txt",
    cmd = "(bash scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh && bash scripts/tests/trigger-next-queue-automerge-conflict-guard.test.sh && bash scripts/tests/check_sequential_pr_merge_conflicts_fetch_remote.test.sh && bash scripts/tests/phase0_required_context_rollup_check.test.sh) > $OUT",
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
        "scripts/tests/phase0_required_context_rollup_check.test.sh": "scripts/tests/phase0_required_context_rollup_check.test.sh",
        "scripts/ci/assert-pr-required-context.py": "scripts/ci/assert-pr-required-context.py",
        "specs/fixtures/phase0-required-context-rollup/good-oya-ci-required-success.json": "specs/fixtures/phase0-required-context-rollup/good-oya-ci-required-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json": "specs/fixtures/phase0-required-context-rollup/bad-no-checks-reported.json",
        "specs/fixtures/phase0-required-context-rollup/bad-missing-oya-ci-required.json": "specs/fixtures/phase0-required-context-rollup/bad-missing-oya-ci-required.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-failure.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-completed-failure.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-completed-failure.json",
        "specs/fixtures/phase0-required-context-rollup/good-nested-cloud-ci-oya-ci-success.json": "specs/fixtures/phase0-required-context-rollup/good-nested-cloud-ci-oya-ci-success.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-missing-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-missing-producer.json",
        "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-untrusted-producer.json": "specs/fixtures/phase0-required-context-rollup/bad-oya-ci-required-success-untrusted-producer.json",
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
