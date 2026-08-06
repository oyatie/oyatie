# Shell + Python script replacement audit — 2026-05-15

> Auditor: general-purpose subagent  
> Working dir: `/Users/jasonlee/oyatie` (branch `oya-m02-m03-fanout`)  
> Authority: user directive 2026-05-15 ("we wanted to fully replace shell scripts and python scripts"; "audit for any shell scripts or python scripts (make sure we replace them all)") + `feedback_no_exceptions_canonical` + ADR-0083 (Rust as canonical implementation language for every `oya-*` crate).  
> Inventory rule: every `.sh` / `.py` in the repo (excluding `target/`, `.git/`, `.grit/worktrees/`, `node_modules/`, `.omc/state/`) is a canonical-naming **extension** waiting to be replaced with a Rust binary.

## Summary

- Total shell scripts (`.sh`) in scope: **10**
- Total python scripts (`.py`) in scope: **6**
- Total in-scope files: **16**
- Total in-scope LOC: **1,540**
- External vendor (out of scope): **0** (no third-party shell/python under in-scope paths)
- Workflows embedding shell (`run:` blocks invoking `.sh` / `python3`): **2** (`pr-tests.yml`, `supply-chain.yml`)

### By category

| Category | Count | LOC | Notes |
|---|---:|---:|---|
| A — test fixture | 2 | 8 | Both under `crates/oya-check-dependency-seam/tests/fixtures/rust-default-language/failing/`. The fixtures themselves are the "this should be Rust per P15" failing-detection corpus — see "Caveat A" below. |
| B — repo hook / `scripts/` | 11 | 1,209 | All under `scripts/`, plus `.omc/hooks/grit-claim-intent-gate.sh`. Mix of Bash wrappers + heredoc'd Python. |
| C — workflow embedded shell | 0 standalone | n/a | All workflow `run:` blocks invoke category-B scripts; replacing B replaces C transitively. |
| D — build script / xtask | 0 | 0 | None present. (`scripts/check.sh` plays the role of a Rust-orchestrated gate runner, not a Cargo build script.) |
| E — demo / one-shot | 1 | 100 | `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh` — historical grit-protocol demo; `grit` is on the sunset path (CLAUDE.md). |
| F — external vendor (OOS) | 0 | 0 | No vendored third-party shell/python found in scope. |

### Caveat A — test-fixture "exceptions" are the corpus

`crates/oya-check-dependency-seam/tests/fixtures/rust-default-language/failing/legacy-setup.sh` (5 LOC) and `render-helper.py` (3 LOC) **exist precisely to verify that `check_rust_default_language` flags non-Rust extensions**. The seam check confirmed by `crates/oya-check-dependency-seam/tests/per_subcheck_unit_tests.rs:67-88`. Replacing them with Rust would defeat the test. Resolution: leave in place, document in the audit, and ensure the lane fail-builds on **production** `.sh`/`.py` outside this fixture path. These two files are the only canonical extension in this audit; everything else is replacement work.

---

## Findings

### `scripts/` (11 files, ~1,209 LOC)

1. **`scripts/check.sh`** (100 LOC) — category B — purpose: pre-merge gate runner that orchestrates ~50 `cargo run -p oya-dev-cli -- gate validate …` invocations plus 4 sibling `scripts/*.{sh,py}` calls plus a heredoc'd Python JSON-parse smoke test. Replacement: **extend `oya-dev-cli` with a top-level `gate run-all` (or `check`) subcommand** that wraps the identical gate-validate sequence + parity-check heredoc. Estimated effort: **M (1-3 hr)** — mostly mechanical Rust list-of-subcommands; the only design call is the heartbeat printer (lines 9-23), which becomes a small `tokio::time::interval` printer in Rust. Replace BEFORE: B-3, B-4, B-5, B-7, B-9, B-10 (they're invoked from here).

2. **`scripts/check-architecture-boundaries.sh`** (270 LOC, of which 268 LOC are heredoc'd Python) — category B — purpose: workspace-package validation (oya- prefix, `crates/`/`tools/` layout, catalog records, role-based dependency edges, legacy-dir ban). Self-test inline. Already invoked from `scripts/check.sh:90-91`. Replacement: **new `[[bin]] arch-boundaries` under `oya-dev-cli` OR (cleaner) a dedicated `oya-check-architecture-app` crate** (per ADR-0107/canonical-app-layer). Consumes `cargo metadata` + parses `registry/catalog/*.yaml`. Estimated effort: **M (1-3 hr)** — Python is straight-line, no surprising stdlib use; Rust analog uses `cargo_metadata` crate + `serde_yaml`.

3. **`scripts/check-oya-vcs-admission.sh`** (222 LOC, of which 169 LOC heredoc'd Python) — category B — purpose: Oya VCS PR3 admission gate; validates `specs/*.json` shape + branch-protection YAML + supply-chain workflow text + cargo metadata + audit-chain JSONL coverage; finishes with `cargo test`/`cargo run` smoke for `oya-dev-cli vcs claim/verify/done/promote`. Replacement: **`oya-vcs-admission-gate-app` crate**. Estimated effort: **M-L (2-4 hr)** — many small spec-shape assertions; ~10 file reads + 30 boolean checks.

4. **`scripts/check-oya-vcs-provider-execution.sh`** (182 LOC, of which 113 LOC heredoc'd Python) — category B — purpose: credential-safe provider-execution proof for Oya VCS admission lane; calls real `trivy fs`/`trivy config`, validates Argo `application.json` shape, emits provider-execution evidence JSON. Replacement: **`oya-vcs-provider-execution-gate-app` crate** (sibling of B-3); shells out to `trivy` (canonical external tool) but everything else becomes typed Rust. Estimated effort: **M (1-3 hr)** — clear single-file scope, deterministic outputs.

5. **`scripts/check-product-index.py`** (50 LOC) — category B — purpose: parse `docs/products/README.md` axis-product table + verify `docs/machine-readable/catalog.json` mirrors it. Replacement: **extend existing `oya-dev-cli gate validate doc-catalog` lane** OR add `gate validate product-index` subcommand. Estimated effort: **S (under 30 min)** — 50 LOC of plain text-split + JSON-parse; one-pass port.

6. **`scripts/check-stage0-application-shell-prereqs.py`** (120 LOC) — category B — purpose: M02 Stage-0 application-shell prereq check; verifies workspace `[workspace.members]` contains `crates/oya-application-app`, `cargo metadata` edition/rust-version pinned. Replacement: **extend `oya-dev-cli gate validate stage0-application-shell` lane**. Estimated effort: **S (under 30 min)** — uses `cargo_metadata` crate; no surprises.

7. **`scripts/audit-master-plan-completion.py`** (84 LOC) — category B — purpose: status-honesty audit for `specs/masterplan.json` (no phase-complete with incomplete IPs; complete IPs must have evidence JSON). Replacement: **extend `oya-dev-cli gate validate masterplan-status-honesty` lane**. Estimated effort: **S (under 30 min)** — `serde_json::Value` + `std::fs::read_dir`; trivial.

8. **`scripts/render-m02-exit-checklist.py`** (105 LOC) — category B — purpose: render the M02 exit-gate checklist markdown from canonical P22 inputs (phase-spec, impl-plan, INDEX); `--check` mode verifies parity. Replacement: **extend `oya-dev-cli doc render-m02-exit-checklist` subcommand** (alongside the existing `doc mdbook` / `doc openapi` / `doc adr-index` family). Estimated effort: **S (under 30 min)** — pure string templating + parity diff.

9. **`scripts/render-master-plan-ledger.py`** (119 LOC) — category B — purpose: render compact status ledger from `specs/masterplan.json`; validates IPs declare `execution_unit=ChangeSet` + `changeset_contract=claimable-verifiable-bundleable-promotable`; `--check` mode verifies parity. Replacement: **extend `oya-dev-cli doc render-master-plan-ledger` subcommand**. Estimated effort: **S (under 30 min)** — JSON walk + `collections.Counter` → `HashMap<(K,V), usize>`.

10. **`scripts/install-trivy-ci.sh`** (28 LOC) — category B — purpose: install pinned Trivy 0.70.0 binary on CI runner with sha256 verification. Replacement: **`tools/oya-install-trivy-app` Rust binary** invoked from `.github/workflows/pr-tests.yml:122` + `supply-chain.yml:34`. Estimated effort: **S (under 30 min)** — `reqwest` + `sha2` + `tar`; ~80 LOC Rust. ⚠️ **External-facing flag**: CI workflows directly reference this path — replacement requires a same-PR workflow update.

11. **`scripts/supply-chain-adr0039.sh`** (78 LOC) — category B — purpose: ADR-0039 release-time supply-chain lane; runs `trivy fs`/`trivy config`/SBOM emit, then `cosign sign`/`cosign verify`/`cosign attest` against signed evidence. Replacement: **`oya-supply-chain-adr0039-app` crate** (or extend an existing release-lane crate); shells out to `trivy` + `cosign` (canonical external tools). Estimated effort: **M (1-3 hr)** — straight-line tool wrapper with deterministic args; awk YAML-parsing block on lines 24-43 becomes `serde_yaml` parse of `registry/release/images.yaml`. ⚠️ **External-facing flag**: `.github/workflows/oya-governance-supply-chain.yml:54` references this path.

12. **`scripts/hooks/pre-push-repoctl.sh`** (3 LOC) — category B — purpose: thin shell wrapper that invokes `cargo run -p oya-dev-cli --bin repoctl -- pre-push "$@"`. Replacement: **delete and replace with a `.git/hooks/pre-push` symlink to `cargo run -p oya-dev-cli --bin repoctl -- pre-push`** OR install hook directly via `oya-dev-cli hook install`. Estimated effort: **S (under 5 min)** — trivial deletion.

### `.omc/hooks/` (1 file, 71 LOC)

13. **`.omc/hooks/grit-claim-intent-gate.sh`** (71 LOC) — category B — purpose: Claude-Code `PreToolUse` hook that JSON-parses stdin and rejects `grit claim` / `grit begin` invocations on `crates/oya-*` paths missing both a phase-id regex match and an Accepted-plan/ADR regex match. **Sunset-coupled**: `grit` itself is on the sunset path (CLAUDE.md "Both [grit + oya-tooling-agent-read] are scheduled to sunset once Oya VCS (M01-P07) and Foundry go live"). Replacement options: (a) port to **`tools/oya-grit-claim-intent-gate-app`** Rust binary now and delete when grit retires; (b) skip — let it die when grit dies. **Recommended: option (b)** — replacement cost (~30 min) is wasted against a sunsetting tool. Estimated effort if (a): **S (under 30 min)**.

### `docs/runbooks/` (1 file, 100 LOC) — category E

14. **`docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh`** (100 LOC) — category E — purpose: one-shot historical demo: spawns `grit watch`, parallel `grit claim` from two agents, negative-claim from third, validates locks released. Demo evidence already captured at `/evidence/agentic-pipeline/ip-010-parallel-claim-demo-transcript/`. Replacement: **delete** (already-captured historical evidence; `grit` on sunset path). Estimated effort: **S (under 5 min)** — pure deletion.

### `crates/.../tests/fixtures/` (2 files, 8 LOC) — category A

15. **`crates/oya-check-dependency-seam/tests/fixtures/rust-default-language/failing/legacy-setup.sh`** (5 LOC) — category A — fixture for `check_rust_default_language`'s "non-Rust extension is flagged" path. **KEEP** (deleting breaks the failing-detection test). Documented as canonical fixture in this audit.

16. **`crates/oya-check-dependency-seam/tests/fixtures/rust-default-language/failing/render-helper.py`** (3 LOC) — category A — fixture for `check_rust_default_language`. **KEEP** (same reason as #15).

### Workflows (no standalone shell to replace)

- `.github/workflows/pr-tests.yml` lines 122, 127: invokes B-10 + B-3 (replace those, workflow auto-flips).
- `.github/workflows/oya-governance-supply-chain.yml` lines 34, 36, 54: invokes B-10 + B-4 + B-11.
- Other workflows (`cosign.yml`, `sbom.yml`, `slsa.yml`, `release-musl.yml`, `release-evidence-pack.yml`, `_template-*.yml`) contain only `cargo`/`gh`/native-action `run:` blocks — no `.sh`/`.py` references and no embedded heredoc logic that warrants separate replacement.

---

## Replacement sequencing (ordered by ROI)

| # | Target | Effort | ROI driver |
|--:|---|---|---|
| 1 | **Top 5 — `scripts/check.sh` → `oya-dev-cli gate run-all`** | M | Single entry point for ~50 gates; eliminates a transitively-blocking shell layer; unblocks #2-#9 because most of them are invoked from here. |
| 2 | **Top 5 — `scripts/check-architecture-boundaries.sh` → arch-boundaries Rust** | M | Heaviest heredoc'd Python in the repo (268 LOC); already has 8-case self-test ready to port to `#[test]` modules. |
| 3 | **Top 5 — `scripts/check-oya-vcs-admission.sh` → `oya-vcs-admission-gate-app`** | M-L | Directly referenced by `branch-protection.yaml` + `pr-tests.yml`; canonical Foundry-VCS family already exists; high-visibility lane. |
| 4 | **Top 5 — `scripts/check-oya-vcs-provider-execution.sh` → `oya-vcs-provider-execution-gate-app`** | M | Sibling of #3; same effort tier; both go together. |
| 5 | **Top 5 — Easy-wins bundle (B-5, B-6, B-7, B-8, B-9, B-12)** | S × 6 (~2-3 hr total) | Six S-tier scripts at ~30 min each = one focused PR adding 5 `oya-dev-cli` subcommands + deleting one wrapper; minimal risk; reduces script-file count from 16 to 6. |
| 6 | `scripts/install-trivy-ci.sh` → `tools/oya-install-trivy-app` | S | Touches CI workflows; do alongside #3/#4 since they share Trivy. |
| 7 | `scripts/supply-chain-adr0039.sh` → `oya-supply-chain-adr0039-app` | M | Release-time lane; lower frequency; can land after #6. |
| 8 | `docs/runbooks/.../grit-parallel-claim-demo.sh` — **delete** | S | Pure deletion; do anytime. |
| 9 | `.omc/hooks/grit-claim-intent-gate.sh` — **skip until grit sunsets** | n/a | Negative ROI to port a sunsetting tool's hook. |

### Cross-cutting consolidation opportunities

- **`cargo metadata` consumers**: B-2, B-3, B-6 all parse `cargo metadata` output. A shared helper in `oya-dev-cli`'s gate substrate (likely already present) reduces duplication.
- **`specs/*.json` shape-checkers**: B-3, B-7, B-9 all read & assert shape on master-plan / sequencing / VCS-replacement JSONs. Candidate for a shared `oya-specs-shape-kernel` library.
- **`render --check` parity pattern**: B-8 and B-9 implement the same "render-to-string, diff against on-disk, fail-build if stale" pattern. Move to a `oya-dev-cli doc render` substrate that takes a renderer trait.

---

## Effort breakdown

| Tier | Files | Est. cumulative |
|---|--:|---|
| Easy wins (S) | 7 (B-5, B-6, B-7, B-8, B-9, B-10, B-12) + 1 deletion (E-14) | ~3-4 hr |
| Medium (M) | 6 (B-1, B-2, B-3, B-4, B-11) + 1 M-L (B-3 alt) | ~12-18 hr (~2 days) |
| Skip | 1 (`.omc/hooks/grit-claim-intent-gate.sh` — sunset coupled) | 0 hr |
| KEEP (test fixtures) | 2 | 0 hr |
| **Total replacement effort** | 14 files | **~2-3 dev-days** end-to-end |

---

## External-facing references requiring deprecation-notice path

| Script | External consumer | Mitigation |
|---|---|---|
| `scripts/check.sh` | `docs/AGENTS.md`, multiple ADRs, `docs/standards/*.md` cite as the pre-merge gate runner. | Add `gate run-all` Rust subcommand; keep `scripts/check.sh` as a 3-line `exec`-style wrapper for one release; cite in ADR; sunset in next minor. |
| `scripts/install-trivy-ci.sh` | `.github/workflows/pr-tests.yml`, `supply-chain.yml`. | Replace in same PR as B-10 (workflow + script flip together). |
| `scripts/supply-chain-adr0039.sh` | `.github/workflows/oya-governance-supply-chain.yml`, ADR-0039, release runbooks. | Same-PR workflow update; ADR amendment recording the Rust replacement crate. |
| `scripts/check-oya-vcs-admission.sh` | `.github/branch-protection.yaml`, `.github/workflows/pr-tests.yml`. | Same-PR update of branch-protection + workflow + script removal. |
| `scripts/check-oya-vcs-provider-execution.sh` | `.github/branch-protection.yaml`, `.github/workflows/oya-governance-supply-chain.yml`. | Same-PR update. |
| `scripts/hooks/pre-push-repoctl.sh` | Local-developer git hooks (no CI binding). | Delete and document `cargo run -p oya-dev-cli --bin repoctl -- pre-push` as the canonical hook command. **Update 2026-05-15:** `oya verify` top-level subcommand added as the canonical local-developer fold (per user directive "pre-push should really just be part of some other check/validate"). Full deletion of the .sh + retirement of the `repoctl pre-push` binary surface requires updating the `oya-check-pre-push` contract kernel (currently encodes `repoctl pre-push` as `CANONICAL_PRE_PUSH_COMMAND`); tracked as follow-up to avoid colliding with concurrent contract-kernel work. |

All other in-scope scripts are internal-only.

---

## Out-of-scope / vendor

None. No third-party `.sh`/`.py` shipped under in-scope paths. (The 8 `.grit/worktrees/<branch>/scripts/*` entries are duplicate working copies of the same `scripts/` files — counted once via the in-scope inventory.)

---

## Follow-up — `ALLOWED_DEPENDENCY_ROLES` reconciliation (Wave 2 row B-2 amendment 2026-05-15)

The architecture-boundaries gate (`crates/oya-dev-cli/src/commands/gate/architecture_boundaries.rs::allowed_dependency_roles()`) ports the legacy Python `ALLOWED_DEPENDENCY_ROLES` table verbatim. The table contains 11 role keys that pre-date the 13-value canonical enum (ADR-0105 + ADR-0106). The migration is staged in three follow-ups (cited in ADR-0105 §"Amendment 2026-05-15 — `ALLOWED_DEPENDENCY_ROLES` reconciliation"):

1. **Migrate 22 `application` catalog records → `usecase`** (paired with the 6 workspace-crate renames in ADR-0106). Touches `registry/catalog/*.yaml` in lockstep with each crate rename.
2. **Migrate 6 `runtime` catalog records → `app`** (paired with ADR-0056 §"Concrete migration"). Each `*-runtime` crate renames to `*-app`; catalog record's `role:` flips at the same time.
3. **Remove 4 `test` catalog records** — test-only crates take canonical layer suffixes per the predictable-naming kernel; the `test` role is not in the canonical 13-value enum.

After all three follow-ups land, `ALLOWED_DEPENDENCY_ROLES` is updated to drop `application`/`runtime`/`test` and add `cli`/`grpc`/`graphql`/`sdk`/`usecase`. Tracking row B-2 supersedes itself once this is complete.

Source-of-truth catalog tally (2026-05-15): 22 `application`, 6 `runtime`, 4 `test` records remain in `registry/catalog/*.yaml`.

---

## Open questions for user

1. **Sunset-coupled hook (`.omc/hooks/grit-claim-intent-gate.sh`)** — port to Rust now, or let it die with `grit` retirement? Recommendation: skip.
2. **Test-fixture `.sh`/`.py` (cat-A files)** — keep as canonical detection corpus, or rewrite the seam check to use synthetic in-test strings rather than on-disk fixtures? Recommendation: keep (current design is more truthful to the actual filesystem scan the lane performs in production).
3. **`scripts/check.sh` transitional wrapper** — keep a 3-line shell wrapper for one release after `gate run-all` lands, or rip in the same PR? Default: rip; cite `gate run-all` in `docs/AGENTS.md` + ADR.
4. **Granularity** — one mega-PR replacing all 14 files, or one PR per script (easy wins bundle + per-medium-script PRs)? Recommendation: one easy-wins PR (~6 S-tier in one shot) + one PR per medium-tier script; one mega-PR is too review-heavy.

---

## Sources

- Inventory command: `find . -type f \( -name "*.sh" -o -name "*.py" \) -not -path "./target/*" -not -path "./.git/*" -not -path "./node_modules/*" -not -path "./.omc/state/*" -not -path "./.grit/worktrees/*"`.
- LOC: `wc -l` over inventory output.
- Workflow embedded-shell references: `grep -l "scripts/" .github/workflows/*.yml`.
- Test-fixture consumer: `crates/oya-check-dependency-seam/tests/per_subcheck_unit_tests.rs:30,67-88` and `crates/oya-check-dependency-seam/src/lib.rs:119`.
- Authorities cited: `/Users/jasonlee/oyatie/CLAUDE.md`; `docs/decisions/ADR-0700-ci-admission-live-apex.md`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_exceptions_canonical.md`.

---

## 2026-06-26 update — Python/MJS to Rust retirement inventory (worker-6)

Captured at: 2026-06-26T11:13:55Z

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
| `scripts/asyncapi-lint.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | temporary_legacy_bridge | 3 | evidence/foundation/m01-p03-ip-002-audit-contracts.json, evidence/audits/doc-antipattern-audit-1778808000.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/emit_rust_tests.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 2 | docs/ideas/affected-gated-migration-engine.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/gen_first_party_buck.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 7 | oya/oya-meter/BUCK, docs/decisions/ADR-0700-ci-admission-live-apex.md, oya/oya-flags/BUCK, oya/oya-identity/BUCK ... |
| `scripts/generate-erp-second-pass-docs.mjs` | MJS | Lane 3 / worker-3 (root doc-generator MJS scripts) | temporary_legacy_bridge | 2 | docs/standards/anti-patterns.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/generate-marketplace-workplace-doc-set.mjs` | MJS | Lane 3 / worker-3 (root doc-generator MJS scripts) | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/proto-lint.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | temporary_legacy_bridge | 3 | evidence/foundation/m01-p03-ip-002-audit-contracts.json, evidence/audits/doc-antipattern-audit-1778808000.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_control_plane_operation_contract_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 2 | specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_enforceability_facets_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 2 | specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_hyperscaler_parity_taxonomy_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 3 | specs/language-discipline-registry.json, specs/cloud-hyperscaler-parity-taxonomy.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_observability_slo_evidence_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 3 | specs/cloud-observability-slo-evidence-contract.json, specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 3 | specs/language-discipline-registry.json, specs/cloud-production-quality-kit-evidence-backlog.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/tests/cloud_resource_contract_parity_catalog_check.py` | Python | Lane 5 / worker-5 (cloud Python validator scripts) | temporary_legacy_bridge | 2 | specs/language-discipline-registry.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/validate-adr-shape.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | temporary_legacy_bridge | 4 | evidence/foundation/m01-p01-ip-001-data-use-boundary-adr.json, evidence/audits/doc-antipattern-audit-1778808000.json, docs/audit/initial-sweep-2026-06-06/_execution/prelane-0.7/00-GOVERNANCE-BOOTSTRAP.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `scripts/validate-foundry-phase00-evidence.mjs` | MJS | Lane 1 / worker-1 (root MJS lint shim retirement) | temporary_legacy_bridge | 4 | docs/products/foundry/PHASE-00-SPEC.md, evidence/audits/doc-antipattern-audit-1778808000.json, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json, docs/audit/initial-sweep-2026-06-06/FOUNDRY-PROSE-SCRUB-MAP.md |
| `tools/anchor-sweep/inject_anchors.py` | Python | Shared overflow (unclaimed candidate) | temporary_legacy_bridge | 2 | docs/audit/initial-sweep-2026-06-06/backlog-reconciliation/20-verify-foundry-hygiene.md, cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `tools/buck/apply-thirdparty-patches.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |
| `tools/buck2/gen-first-party-buck.py` | Python | Lane 4 / worker-4 (Python/Buck generator and patch scripts) | temporary_legacy_bridge | 1 | cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json |

## Worker-6 collision guidance

- Do not edit `cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json` from worker-6 unless taking a specific overflow item and coordinating shared-file ownership.
- Lane 1 owns the four root MJS lint shims and their shared policy updates.
- Generated `*.generated.json` baselines mention legacy paths but must remain read-only; refresh through the materializer if a gate requires it.
- Best next unclaimed slice after lane queues progress: `tools/anchor-sweep/inject_anchors.py`, but verify callers and policy first.
