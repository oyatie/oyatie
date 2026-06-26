# Python/MJS to Rust retirement inventory — worker-6

Generated at: 2026-06-26T11:08:08Z

Scope: tracked `*.py` and `*.mjs` files only. Generated artifacts were excluded from reference counts and were not edited.

Open PR collision precheck: `gh pr list --base dev --state open --limit 100 --json number,headRefName,title,mergeStateStatus,url` returned `[]` before this evidence slice.

## Summary

- Tracked Python files: 14
- Tracked MJS files: 9
- Policy-exceptioned tracked Python/MJS files: 19
- Unowned candidates needing policy/product review before action: 1

## Inventory

| Path | Kind | Owner lane / status | Rust-first policy status | Exact-path refs | Ref examples |
|---|---|---|---|---:|---|
| `cloud/cloud-k8s/tests/test_runtime_substrate_validation.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `infra/seaweedfs/tests/test_seaweedfs_manifest.py` | Python | Shared overflow / infra bridge candidate | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `oya/app-shell-frontend/scripts/codegen-check.mjs` | MJS | Lane 2 / worker-2 (app-shell frontend MJS scripts) | not listed | 1 | evidence/multispectrum/pr-633-generated-output-hygiene-20260609-1781033990.json |
| `oya/app-shell-frontend/scripts/run-vinxi.mjs` | MJS | Lane 2 / worker-2 (app-shell frontend MJS scripts) | not listed | 0 | — |
| `oya/app-shell-frontend/scripts/shell-contract-check.mjs` | MJS | Lane 2 / worker-2 (app-shell frontend MJS scripts) | not listed | 0 | — |
| `oya/feature-flags/reference-implementations/python-asyncio-client.py` | Python | Unowned/reference SDK candidate (needs product-policy review before delete) | not listed | 0 | — |
| `scripts/asyncapi-lint.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | rust_backed_compatibility_shim | 3 | evidence/foundation/m01-p03-ip-002-audit-contracts.json, evidence/audits/doc-antipattern-audit-1778808000.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/emit_rust_tests.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 2 | docs/ideas/affected-gated-migration-engine.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/gen_first_party_buck.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 7 | docs/decisions/ADR-0565-zero-graphql-in-the-owned-api-surface.md, oya/oya-identity/BUCK, oya/oya-meter/BUCK, oya/oya-flags/BUCK ... |
| `scripts/generate-erp-second-pass-docs.mjs` | MJS | Lane 3 / worker-3 (root doc-generator MJS scripts) | temporary_legacy_bridge | 2 | docs/standards/anti-patterns.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/generate-marketplace-workplace-doc-set.mjs` | MJS | Lane 3 / worker-3 (root doc-generator MJS scripts) | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/proto-lint.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | rust_backed_compatibility_shim | 3 | evidence/audits/doc-antipattern-audit-1778808000.json, evidence/foundation/m01-p03-ip-002-audit-contracts.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_control_plane_operation_contract_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 2 | specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_enforceability_facets_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 2 | specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_hyperscaler_parity_taxonomy_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 3 | specs/language-discipline-registry.json, specs/cloud-hyperscaler-parity-taxonomy.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_observability_slo_evidence_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 3 | specs/cloud-observability-slo-evidence-contract.json, specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 3 | specs/language-discipline-registry.json, specs/cloud-production-quality-kit-evidence-backlog.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_resource_contract_parity_catalog_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 2 | specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/validate-adr-shape.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | rust_backed_compatibility_shim | 5 | evidence/foundation/m01-p01-ip-001-data-use-boundary-adr.json, evidence/audits/doc-antipattern-audit-1778808000.json, docs/audit/initial-sweep-2026-06-06/_execution/prelane-0.7/00-GOVERNANCE-BOOTSTRAP.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/src/lib.rs ... |
| `scripts/validate-foundry-phase00-evidence.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | rust_backed_compatibility_shim | 4 | evidence/audits/doc-antipattern-audit-1778808000.json, docs/products/foundry/PHASE-00-SPEC.md, docs/audit/initial-sweep-2026-06-06/FOUNDRY-PROSE-SCRUB-MAP.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `tools/anchor-sweep/inject_anchors.py` | Python | Shared overflow (unclaimed candidate) | temporary_legacy_bridge | 2 | docs/audit/initial-sweep-2026-06-06/backlog-reconciliation/20-verify-foundry-hygiene.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `tools/buck/apply-thirdparty-patches.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `tools/buck2/gen-first-party-buck.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |

## Worker-6 collision guidance

- Do not edit `cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json` from worker-6 unless taking a specific overflow item and coordinating shared-file ownership.
- Lane 1 owns the four root MJS lint shims and their shared policy updates.
- Generated `*.generated.json` baselines mention legacy paths but must remain read-only; refresh through the materializer if a gate requires it.
- Best next unclaimed slice after lane queues progress: `tools/anchor-sweep/inject_anchors.py`, but verify callers and policy first.
