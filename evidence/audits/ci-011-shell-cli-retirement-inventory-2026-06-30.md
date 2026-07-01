# CI-011 shell + CLI retirement inventory — 2026-06-30

Generated: `2026-06-30T23:12:39Z`

Task: `t_43080012` — CI-011: shell + CLI retirement inventory.

Authority: ADR-0515 (no CLI/shell, cloud-ci/console/API authority), ADR-0522 (every real-work `run:`/`sh`/Makefile target retires into the one graph), ADR-0523 (closed irreducible-glue ledger), HANDOFF.md §6.2/§6.4, and the DRIFT-001 Kanban supersession comment.

Scope: executable source surfaces and runner operations in the current worktree. Prose-only references are not executable operations; generated JSON faces are data and were not edited. Exclusions: `.worktrees/`, `.git/`, `target/`, `buck-out/`.

Machine-readable ledger: `evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json`.

## Counts

| Surface | Count | Classification counts |
|---|---:|---|
| Shell/shebang files | 52 | delete=8, irreducible_glue_ledger_entry=7, migrate_to_console_api=37 |
| GitHub Actions `run:` blocks | 27 | irreducible_glue_ledger_entry=15, migrate_to_console_api=12 |
| Makefile executable targets | 12 | migrate_to_console_api=12 |
| Rust CLI/main surfaces | 25 | delete=1, migrate_to_console_api=24 |

## Closed irreducible-glue ledger mapping

| Ledger item | ADR-0523 item | Current surfaces | Retirement pressure |
|---|---:|---|---|
| `toolchain_bootstrap` | 1 | `infra/ci/install-buck2.sh`<br>`.github/workflows/oya-ci-required.yml run blocks installing buck2/rustup toolchains` | Move downloads into buck2 toolchains/download_file where possible; remaining first bootstrap remains ledger-bound. |
| `scm_facts_emitter_edge` | 2 | `infra/ci/materialize-cloud-ci-generated-faces.sh`<br>`.github/workflows/oya-ci-required.yml materialization run blocks` | Keep edge singular; shell body can become Rust/controller, but ambient SCM read remains graph-edge ledger item. |
| `ci_checkout_fetch_depth_0` | 3 | `.github/workflows/oya-ci-required.yml uses actions/checkout with fetch-depth: 0` | Generated forge/owned-runner adapter may emit it; do not delete shallow-proofing. |
| `hardware_endpoint_cd_bring_up` | 4 | `infra/talos/installation-media/gen-media.sh`<br>`infra/talos/bare-metal/up.sh`<br>`infra/talos/local/talos-local.sh`<br>`infra/capi/init.sh` | Move automatable desired-state render/apply into GitOps/CAPI/tofu/controller/ops console; keep only physical one-shot residue. |
| `reindeer_buckify` | 5 | `scripts/ci/regen-third-party.sh`<br>`scripts/ci/third-party-buckify-handedits.patch` | Remove bash patch orchestration; generator itself remains out-of-graph by design. |

## Shell/shebang file classifications

| Path | LOC | Classification | Ledger item / target |
|---|---:|---|---|
| `.envrc` | 7 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `bin/oya` | 51 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `infra/capi/crs/render.sh` | 90 | `migrate_to_console_api` | GitOps/CAPI desired-state generator or controller API; keep rendering deterministic. |
| `infra/capi/init.sh` | 32 | `irreducible_glue_ledger_entry` | hardware_endpoint_cd_bring_up |
| `infra/ci/buck2-affected-gate.sh` | 137 | `migrate_to_console_api` | Rust cloud-ci affected-set app / generated adapter; no bash logic in CI. |
| `infra/ci/install-buck2.sh` | 34 | `irreducible_glue_ledger_entry` | toolchain_bootstrap |
| `infra/ci/materialize-cloud-ci-generated-faces.sh` | 28 | `irreducible_glue_ledger_entry` | scm_facts_emitter_edge |
| `infra/talos/bare-metal/up.sh` | 141 | `irreducible_glue_ledger_entry` | hardware_endpoint_cd_bring_up |
| `infra/talos/installation-media/gen-media.sh` | 70 | `irreducible_glue_ledger_entry` | hardware_endpoint_cd_bring_up |
| `infra/talos/local/talos-local.sh` | 423 | `irreducible_glue_ledger_entry` | hardware_endpoint_cd_bring_up |
| `infra/talos/smoke-kata.sh` | 42 | `migrate_to_console_api` | Rust smoke/conformance app or ops-console check over Talos/Kubernetes APIs. |
| `oya/intelligence/iac/cedar/guardrails-build.sh` | 79 | `migrate_to_console_api` | Rust Cedar bundle builder / policy API. |
| `run-slice.sh` | 45 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `scripts/agent-pre-push-validate.sh` | 259 | `migrate_to_console_api` | Cloud-ci Rust gate, ops console/API, or generated adapter depending on owner. |
| `scripts/branch-protection-apply.sh` | 132 | `migrate_to_console_api` | Cloud-scm/cloud-ci controller or ops console API workflow. |
| `scripts/build/build-and-push-cloud-intelligence.sh` | 100 | `migrate_to_console_api` | Buck2 OCI/image target plus GitOps/console deployment API. |
| `scripts/check-sequential-pr-merge-conflicts.sh` | 224 | `migrate_to_console_api` | Cloud-scm merge-queue/controller API. |
| `scripts/ci/oya-ci-post.sh` | 170 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `scripts/ci/regen-third-party.sh` | 49 | `irreducible_glue_ledger_entry` | reindeer_buckify |
| `scripts/evidence-secret-scan.sh` | 72 | `migrate_to_console_api` | Rust cloud-ci secret/evidence scanner. |
| `scripts/github-actions-required-secrets-check.sh` | 103 | `migrate_to_console_api` | Cloud-scm/cloud-ci controller or ops console API workflow. |
| `scripts/hooks/pre-push.sh` | 16 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `scripts/install-trivy-ci.sh` | 5 | `migrate_to_console_api` | Rust supply-chain/cloud-ci app with pinned external tool download/verification. |
| `scripts/onprem-bring-up.sh` | 12 | `migrate_to_console_api` | Ops console/API/controller; keep hardware/manual parts as runbook approvals only. |
| `scripts/onprem-host-decommission.sh` | 226 | `migrate_to_console_api` | Ops console/API/controller; keep hardware/manual parts as runbook approvals only. |
| `scripts/pr-review-workflow-pr-head-check.sh` | 189 | `migrate_to_console_api` | Cloud-ci Rust gate, ops console/API, or generated adapter depending on owner. |
| `scripts/reject-placeholder-digests.sh` | 98 | `migrate_to_console_api` | Rust cloud-ci gate app with RED/GREEN fixtures. |
| `scripts/reject-public-dev-domains.sh` | 139 | `migrate_to_console_api` | Rust cloud-ci gate app with RED/GREEN fixtures. |
| `scripts/reject-retired-grouping-wording.sh` | 103 | `migrate_to_console_api` | Rust cloud-ci gate app with RED/GREEN fixtures. |
| `scripts/repair-sequential-pr-queue.sh` | 266 | `migrate_to_console_api` | Cloud-scm merge-queue/controller API. |
| `scripts/supply-chain-adr0039.sh` | 9 | `migrate_to_console_api` | Rust supply-chain/cloud-ci app with pinned external tool download/verification. |
| `scripts/tests/github-actions-required-secrets-check.test.sh` | 74 | `migrate_to_console_api` | Rust unit/integration tests for the replacement gate/app, or delete with the retired script. |
| `scripts/tests/governance-hooks-retired-vcs-surfaces.test.sh` | 64 | `migrate_to_console_api` | Rust unit/integration tests for the replacement gate/app, or delete with the retired script. |
| `scripts/tests/pr-review-workflow-pr-head-check.test.sh` | 162 | `migrate_to_console_api` | Rust unit/integration tests for the replacement gate/app, or delete with the retired script. |
| `scripts/tests/reject-placeholder-digests.test.sh` | 70 | `migrate_to_console_api` | Rust unit/integration tests for the replacement gate/app, or delete with the retired script. |
| `scripts/tests/reject-public-dev-domains.test.sh` | 151 | `migrate_to_console_api` | Rust unit/integration tests for the replacement gate/app, or delete with the retired script. |
| `scripts/tests/reject-retired-grouping-wording.test.sh` | 47 | `migrate_to_console_api` | Rust unit/integration tests for the replacement gate/app, or delete with the retired script. |
| `scripts/tests/trigger-next-queue-automerge-required-contexts.test.sh` | 109 | `migrate_to_console_api` | Rust unit/integration tests for the replacement gate/app, or delete with the retired script. |
| `scripts/trigger-next-queue-automerge.sh` | 345 | `migrate_to_console_api` | Cloud-scm merge-queue/controller API. |
| `tools/completions/bash/_oya` | 56 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `tools/governance/adr-0221-governance-gates.sh` | 119 | `migrate_to_console_api` | Rust cloud-ci governance gate app. |
| `tools/hooks/adr-orphan-detect.sh` | 83 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/injection-content-scanner.sh` | 115 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/local-authority-enforcer.sh` | 50 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/no-cargo-enforcer.sh` | 44 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/pre-dispatch-guide.sh` | 84 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/session-start-context-inject.sh` | 10 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `tools/hooks/spec-version-pin-suggester.sh` | 74 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/stale-tool-suggester.sh` | 53 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/stop-did-you-forget-suggester.sh` | 66 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |
| `tools/hooks/userprompt-canonical-primer.sh` | 10 | `delete` | Delete after consumers are migrated/removed; do not replace with another shell wrapper. |
| `tools/hooks/vacuous-green-gate-detect.sh` | 79 | `migrate_to_console_api` | Cloud-ci Rust gate or declarative agent-runtime policy; no project shell hook authority. |

## GitHub Actions run block classification

| Path:line | Classification | Snippet |
|---|---|---|
| `.github/workflows/docs-graph-drift.yml:72` | `migrate_to_console_api` | `set -euo pipefail \| cargo build --locked -p oya-architecture-graph-generator-app \| cargo test --locked -p oya-architecture-graph-generator-app \| - name: Regenerate the dashboard` |
| `.github/workflows/docs-graph-drift.yml:77` | `migrate_to_console_api` | `set -euo pipefail \| cargo run --locked -q -p oya-architecture-graph-generator-app \ \| --bin oya-architecture-graph-generator -- --write \| - name: Fail on dashboard drift` |
| `.github/workflows/docs-graph-drift.yml:82` | `migrate_to_console_api` | `set -euo pipefail \| git diff --exit-code -- docs/architecture/product-graph.html \ \| \|\| { echo "::error::docs/architecture/product-graph.html is stale; run the generator (--write) and commit."; exit 1; }` |
| `.github/workflows/oya-ci-required.yml:58` | `irreducible_glue_ledger_entry` | `run: infra/ci/install-buck2.sh \| - name: Materialize cloud-ci generated faces` |
| `.github/workflows/oya-ci-required.yml:60` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| - name: Upload regenerated faces \| # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.` |
| `.github/workflows/oya-ci-required.yml:113` | `irreducible_glue_ledger_entry` | `run: infra/ci/install-buck2.sh \| - name: Materialize cloud-ci generated faces` |
| `.github/workflows/oya-ci-required.yml:115` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| - name: cargo test ${{ matrix.crate }}` |
| `.github/workflows/oya-ci-required.yml:117` | `migrate_to_console_api` | `run: cargo test --locked --no-fail-fast -p ${{ matrix.crate }} -- --test-threads=1 \| # ── freshness: first-diagnosis gate for the two stale-output failures from PR #662. \| #    Runs as its own fast job with no needs edge so stale Cargo.lock and stale generated` |
| `.github/workflows/oya-ci-required.yml:132` | `irreducible_glue_ledger_entry` | `run: infra/ci/install-buck2.sh \| - name: Pre-provision pinned Rust toolchain for Buck2 freshness binaries` |
| `.github/workflows/oya-ci-required.yml:134` | `irreducible_glue_ledger_entry` | `set -euo pipefail \| rustup show active-toolchain \|\| rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy \| rustup component add rustfmt clippy --toolchain 1.95.0 \| rustc --version \| cargo --version \| - name: Run freshness ga` |
| `.github/workflows/oya-ci-required.yml:141` | `migrate_to_console_api` | `set -euo pipefail \| freshness_bin="$(buck2 build //cloud/cloud-ci/gates/oya-cloud-ci-freshness-app:oya-cloud-ci-freshness-app-bin --show-output \| awk '{print $2}')" \| "${freshness_bin}" --repo-root . \| # ── registry-drift: materialized workspace == regenerated` |
| `.github/workflows/oya-ci-required.yml:165` | `irreducible_glue_ledger_entry` | `run: infra/ci/install-buck2.sh \| - name: Materialize faces then assert byte-parity` |
| `.github/workflows/oya-ci-required.yml:167` | `irreducible_glue_ledger_entry` | `infra/ci/materialize-cloud-ci-generated-faces.sh . \| cargo test --locked --no-fail-fast -p registry-drift -- --test-threads=1 \| # ── cloud-ci-firewall: the baseline ratchet (blocks only NEW debt) + the gate-registration \| #    meta-test (no in-tree gate may go` |
| `.github/workflows/oya-ci-required.yml:190` | `irreducible_glue_ledger_entry` | `run: infra/ci/install-buck2.sh \| - name: Materialize cloud-ci generated faces` |
| `.github/workflows/oya-ci-required.yml:192` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| - name: cargo test cloud-ci-firewall` |
| `.github/workflows/oya-ci-required.yml:194` | `migrate_to_console_api` | `run: cargo test --locked --no-fail-fast -p oya-cloud-ci-firewall-app -- --test-threads=1 \| # ── GENERATED OUTPUT DIFF POLICY. Generated files may be deleted to retire a tracked output, \| #    but PRs must not add/modify generated outputs as merge surfaces. Cla` |
| `.github/workflows/oya-ci-required.yml:211` | `irreducible_glue_ledger_entry` | `run: infra/ci/install-buck2.sh \| - name: Pre-provision pinned Rust toolchain for Buck2 policy binary` |
| `.github/workflows/oya-ci-required.yml:213` | `irreducible_glue_ledger_entry` | `set -euo pipefail \| rustup show active-toolchain \|\| rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy \| rustup component add rustfmt clippy --toolchain 1.95.0 \| rustc --version \| cargo --version \| - name: Reject non-delet` |
| `.github/workflows/oya-ci-required.yml:223` | `migrate_to_console_api` | `set -euo pipefail \| if [ "${EVENT_NAME}" = "push" ]; then \| echo "generated-output-diff-policy: push event; presubmit diff policy not applicable." \| exit 0 \| fi \| git fetch --no-tags --prune origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}" \| po` |
| `.github/workflows/oya-ci-required.yml:260` | `irreducible_glue_ledger_entry` | `run: infra/ci/install-buck2.sh \| # Pre-provision the pinned rust toolchain ONCE, serially, before the buck2 build. \| # The buck2 rust toolchain (toolchains/BUCK: system_rust_toolchain via the rustup shim) \| # resolves rustc/cargo/clippy per-compile-action, and` |
| `.github/workflows/oya-ci-required.yml:273` | `irreducible_glue_ledger_entry` | `set -euo pipefail \| rustup show active-toolchain \|\| rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy \| rustup component add rustfmt clippy --toolchain 1.95.0 \| rustc --version \| cargo --version \| # Warm buck-out + the bu` |
| `.github/workflows/oya-ci-required.yml:306` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication — \| # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests \| # run green, ful` |
| `.github/workflows/oya-ci-required.yml:319` | `migrate_to_console_api` | `set -euo pipefail \| # buck2 test builds its targets before running them, so a standalone \| # `buck2 build` immediately before is redundant — removed (item 4 quick win). \| buck2 test //cloud/cloud-ci/... \| - name: buck2 affected-set driver (advisory speed path)` |
| `.github/workflows/oya-ci-required.yml:326` | `migrate_to_console_api` | `set -euo pipefail \| chmod +x infra/ci/buck2-affected-gate.sh \| BUCK2=buck2 infra/ci/buck2-affected-gate.sh "origin/${{ github.base_ref \|\| 'dev' }}" \| # ── APP-SHELL GENERATED CLIENT LANE. Generated TypeScript clients are intentionally ignored \| #    in git, so` |
| `.github/workflows/oya-ci-required.yml:345` | `migrate_to_console_api` | `set -euo pipefail \| corepack enable \| corepack prepare pnpm@11.5.2 --activate \| pnpm --version \| - name: Regenerate and verify app-shell clients` |
| `.github/workflows/oya-ci-required.yml:351` | `migrate_to_console_api` | `set -euo pipefail \| pnpm --dir oya/app-shell-frontend install --frozen-lockfile \| pnpm --dir oya/app-shell-frontend codegen \| pnpm --dir oya/app-shell-frontend codegen:check \| pnpm --dir oya/app-shell-frontend typecheck \| # ── THE FAN-IN. This is the single re` |
| `.github/workflows/oya-ci-required.yml:376` | `migrate_to_console_api` | `echo "Gate results:" \| echo "  gate (matrix)     = ${{ needs.gate.result }}" \| echo "  freshness         = ${{ needs.gate-freshness.result }}" \| echo "  registry-drift    = ${{ needs.gate-registry-drift.result }}" \| echo "  cloud-ci-firewall = ${{ needs.gate-c` |

## Makefile target classification

| Target | Line | Classification | Rationale |
|---|---:|---|---|
| `help` | 13 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `bootstrap` | 24 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `install` | 26 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `plan` | 28 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `apply` | 31 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `tofu-init` | 34 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `tofu-fmt-check` | 37 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `verify` | 40 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `verify-deploy-contract` | 42 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `fleet` | 46 | `migrate_to_console_api` | Print-only pointer to hardware-gated bring-up; keep as runbook prose until console/API replaces it, not as an executable authority. |
| `ops` | 55 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |
| `check-tofu` | 60 | `migrate_to_console_api` | Make target is shell/CLI orchestration; route operation through console/API/cloud-ci generated adapter. |

## Rust CLI/main surface classification

| Path | Command | Family | Classification | Target |
|---|---|---|---|---|
| `oya/developer-sdk/crates/oya-dev-cli/src/main.rs` | `oya-dev-cli / default-run oya` | `internal_operator_cli` | `migrate_to_console_api` | Move gate/verify/doc/lint/ops/supply-chain/onprem/merge-queue operations into cloud-ci gate apps, controllers, console, or APIs; keep only local bridge feedback until deleted. |
| `bin/oya` | `oya wrapper` | `internal_operator_cli_wrapper` | `delete` | Delete with PATH shim once oya-dev-cli consumers are migrated. |
| `cloud/tenancy/crates/oya-tenant-cli/src/main.rs` | `oya` | `tenant_product_cli` | `migrate_to_console_api` | Allowed only as tenant product integration/SDK wrapper; default UX and operations should be console/API, never internal merge/ops authority. |
| `libs/oya-shared-architecture-check-cli/src/main.rs` | `` | `check_cli` | `migrate_to_console_api` | Cloud-ci Rust gate app or library invoked by oya-ci-required; no standalone check CLI authority. |
| `libs/oya-shared-bounded-contexts-check-cli/src/main.rs` | `` | `check_cli` | `migrate_to_console_api` | Cloud-ci Rust gate app or library invoked by oya-ci-required; no standalone check CLI authority. |
| `libs/oya-shared-semver-check-cli/src/main.rs` | `` | `check_cli` | `migrate_to_console_api` | Cloud-ci Rust gate app or library invoked by oya-ci-required; no standalone check CLI authority. |
| `libs/oya-shared-supply-chain-check-cli/src/main.rs` | `` | `check_cli` | `migrate_to_console_api` | Cloud-ci Rust gate app or library invoked by oya-ci-required; no standalone check CLI authority. |
| `oya/contact-center/crates/oya-contact-center-voice-routing-app/src/main.rs` | `oya-contact-center-voice-routing` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/contract-lifecycle-management/crates/oya-contract-lifecycle-management-contract-obligation-app/src/main.rs` | `oya-contract-lifecycle-management-contract-obligation` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/crm/crates/oya-crm-revenue-app/src/main.rs` | `oya-crm-revenue` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/financial-planning/crates/oya-financial-planning-forecast-scenario-app/src/main.rs` | `oya-financial-planning-forecast-scenario` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/incident-management/crates/oya-incident-management-sre-incident-command-app/src/main.rs` | `oya-incident-management-sre-incident-command` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/intelligence/crates/oya-codeview-cli/src/main.rs` | `` | `cli_binary` | `migrate_to_console_api` | Console/API/cloud-ci equivalent depending on owner. |
| `oya/marketing-automation/crates/oya-marketing-automation-campaign-journey-app/src/main.rs` | `oya-marketing-automation-campaign-journey` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/plant-maintenance/crates/oya-plant-maintenance-work-order-app/src/main.rs` | `oya-plant-maintenance-work-order` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/production-planning/crates/oya-production-planning-mrp-app/src/main.rs` | `oya-production-planning-mrp` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/quality-management/crates/oya-quality-management-inspection-app/src/main.rs` | `oya-quality-management-inspection` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/real-estate/crates/oya-real-estate-lease-app/src/main.rs` | `oya-real-estate-lease` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/supply-chain-planning/crates/oya-supply-chain-planning-network-app/src/main.rs` | `oya-supply-chain-planning-network` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/treasury/crates/oya-treasury-cash-app/src/main.rs` | `oya-treasury-cash` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/warehouse/crates/oya-warehouse-fulfillment-app/src/main.rs` | `oya-warehouse-fulfillment` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `oya/whiteboard/crates/oya-whiteboard-canvas-collaboration-app/src/main.rs` | `oya-whiteboard-canvas-collaboration` | `product_service_entrypoint` | `migrate_to_console_api` | Container/service entrypoint with declarative config; operator interaction through console/API. |
| `tools/oya-buck-test-wiring-app/src/main.rs` | `oya-buck-test-wiring` | `local_tool_cli` | `migrate_to_console_api` | Buck2/cloud-ci generator/controller API; no manual operator CLI authority. |
| `tools/oya-lane-supervisor-app/src/main.rs` | `oya-lane-supervisor` | `local_agent_bridge_cli` | `migrate_to_console_api` | Hermes Kanban / cloud-ci lane-state controller; remove local .omc JSONL bridge authority. |
| `tools/oya-xtask-metadata-augment-app/src/main.rs` | `` | `local_tool_cli` | `migrate_to_console_api` | Buck2/cloud-ci generator/controller API; no manual operator CLI authority. |

## Internal `oya-dev-cli` command-family classification

| Command family | Classification | Target |
|---|---|---|
| `gate / gate run-all / validate *` | `migrate_to_console_api` | cloud-ci Rust gate apps behind oya-ci-required |
| `verify --ci-required / --affected / --from-results / terminal-evidence` | `migrate_to_console_api` | cloud-ci required context, structured results API, terminal evidence API |
| `doc / lint / gen / catalog / check / cleanup / demo` | `migrate_to_console_api` | generators, cloud-ci drift gates, or console workflows |
| `ops / onprem` | `migrate_to_console_api` | ops console/API/controllers with audit and approvals |
| `supply-chain` | `migrate_to_console_api` | cloud-ci supply-chain gates and release controllers |
| `plan / submit / merge-queue` | `migrate_to_console_api` | cloud-scm/cloud-ci controller API / Kanban board state |

## `oya-dev-cli` top-level and `gate validate` coverage

The internal `oya-dev-cli` dispatcher was traced at `oya/developer-sdk/crates/oya-dev-cli/src/lib.rs:334-379`; its top-level dispatch surfaces are classified in the JSON ledger under `internal_oya_dev_cli_top_level_commands`. The stale usage-only `oya vcs ...` string is classified as `delete` because ADR-0363 retired the bespoke VCS ratchet and the current dispatcher has no live `vcs` branch.

`oya gate validate` was traced through `oya/developer-sdk/crates/oya-dev-cli/src/commands/gate/mod.rs`; **119** concrete validate subcommands were discovered and are all classified as `migrate_to_console_api` because each is internal governance/verification authority that must run as cloud-ci Rust gate app / `oya-ci-required` registration rather than manual CLI authority. The full subcommand list is in the JSON ledger under `internal_oya_dev_cli_gate_validate_subcommands`.

## Verification

- Scanner command: `git ls-files --cached --others --exclude-standard (excluding .worktrees/, .git/, target/, buck-out/)`.
- Coverage assertion: all 52 discovered shell/shebang files classified; all 27 workflow `run:` blocks classified; all 12 Makefile executable targets classified; 25 Rust CLI/main surfaces classified; 17 `oya-dev-cli` top-level/usage surfaces classified; 119 `oya gate validate` subcommands classified.
- No generated `*.generated.json` files were edited.
- JSON ledger was parsed successfully after write.
