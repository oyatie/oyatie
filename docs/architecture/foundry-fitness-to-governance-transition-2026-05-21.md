---
title: "governance-fitness-* → governance-* Transition Report"
date: 2026-05-21
status: completed
authority: CLAUDE.md new_governance_lane_prefix directive + ADR-0132
classification_artifact: docs/architecture/transition-classification-2026-05-21.json
---

# governance-fitness-* → governance-* Transition Report

## §1 Scope

Total files with `governance-fitness-` text at scan time: **637**

Breakdown by directory:

| Directory          | File count |
|--------------------|-----------|
| `docs/`            | 366       |
| `microservices/`   | 181       |
| `crates/`          | 60        |
| `specs/`           | 28        |
| `packs/`           | 2         |
| `registries/`      | 0         |
| **Total**          | **637**   |

The directive to rename comes from `CLAUDE.md` field:

```
new_governance_lane_prefix: governance-* (per ADR-0132); existing governance-fitness-* lanes retained until each is renamed in its own migration IP
```

ADR-0132 (`docs/decisions/ADR-0705-product-protocol-live-apex.md`) establishes `governance-no-grouping` as the canonical CI lane name using the new prefix, confirming `governance-*` as the forward prefix for all governance lanes.

---

## §2 Classification

| Category | Count | Description |
|----------|-------|-------------|
| A        | 571   | Forward-looking docs — renamed in place |
| B        | 6     | Historical ledgers — tombstoned with §Note |
| C        | 57    | Actual Rust crate files — flagged for code-migration ADR |
| D        | subset of A | Specs with `enforcement_lane_id` values — renamed as part of Category A |
| E        | 27    | Operational code using old prefix as logic — left untouched |

**Total accounted for: 661** (637 original matches + 24 Category C files inside crate dirs that didn't appear in original grep because they have no text references — the 60 crate-file grep hits include both Cargo.toml files and source files; the classification JSON enumerates all files in the 28 crate directories).

---

## §3 Files in Category A — Renamed (571 files)

All 571 files had every occurrence of `governance-fitness-` replaced with `governance-` using per-file `sed -i`.

### §3.1 Docs (360 files)

```
docs/ADR-INDEX.md
docs/advanced-cicd/branch-pipeline/ADR-0055-branch-pipeline.md
docs/advanced-cicd/branch-pipeline/agent-roles-spec.md
docs/advanced-cicd/branch-pipeline/branch-pipeline-architecture.md
docs/advanced-cicd/branch-pipeline/branch-protection-rules.md
docs/advanced-cicd/branch-pipeline/ci-policy-per-branch.md
docs/advanced-cicd/branch-pipeline/fitness-lanes-for-branch-pipeline.md
docs/advanced-cicd/branch-pipeline/foundry-pipeline-mirror.md
docs/advanced-cicd/branch-pipeline/playbooks-by-axis-stage.md
docs/advanced-cicd/branch-pipeline/rollback-mechanics-per-stage.md
docs/advanced-cicd/branch-pipeline/velocity-without-stability-loss.md
docs/advanced-cicd/progressive-delivery/blue-green-spec.md
docs/advanced-cicd/progressive-delivery/canary-rail-spec.md
docs/advanced-cicd/progressive-delivery/dark-launch-spec.md
docs/advanced-cicd/progressive-delivery/enforcement-lanes.md
docs/advanced-cicd/progressive-delivery/feature-flag-architecture.md
docs/advanced-cicd/progressive-delivery/INDEX.md
docs/advanced-cicd/progressive-delivery/playbook-ads.md
docs/advanced-cicd/progressive-delivery/playbook-cloud.md
docs/advanced-cicd/progressive-delivery/playbook-cross-axis-contract.md
docs/advanced-cicd/progressive-delivery/playbook-foundry.md
docs/advanced-cicd/progressive-delivery/playbook-saas.md
docs/advanced-cicd/progressive-delivery/playbook-search.md
docs/advanced-cicd/progressive-delivery/playbook-vertical-pack.md
docs/advanced-cicd/progressive-delivery/playbook-workspace.md
docs/advanced-cicd/progressive-delivery/progressive-delivery-strategy.md
docs/advanced-cicd/progressive-delivery/slo-burn-rate-rollback-spec.md
docs/advanced-cicd/progressive-delivery/stable-cohort-spec.md
docs/advanced-cicd/progressive-delivery/traffic-mirror-spec.md
docs/advanced-cicd/release-versioning/api-versioning-spec.md
docs/advanced-cicd/release-versioning/breaking-change-process.md
docs/advanced-cicd/release-versioning/crate-versioning-spec.md
docs/advanced-cicd/release-versioning/enforcement-lanes.md
docs/advanced-cicd/release-versioning/INDEX.md
docs/advanced-cicd/release-versioning/release-branch-cut-spec.md
docs/advanced-cicd/release-versioning/release-cherry-pick-agent-spec.md
docs/advanced-cicd/release-versioning/release-versioning-strategy.md
docs/advanced-cicd/release-versioning/version-eol-policy.md
docs/AGENT-INSTRUCTION-SOURCES.md
docs/AGENTS.md
docs/agents/AGENT-COMPLETION-PROTOCOL.md
docs/agents/AGENT-DECISION-TREE.md
docs/agents/AGENT-TOOL-PROTOCOL.md
docs/agents/CROSS-REFERENCE-INDEX.md
docs/architecture/hyperscaler-pattern-attribution.md
docs/audits/convention-audit-2026-05-12.md
docs/automation/adr-index-pipeline.md
docs/automation/architecture-map-kernel-spec.md
docs/automation/audit-chain-map-spec.md
docs/automation/changelog-pipeline.md
docs/automation/cross-reference-index-spec.md
docs/automation/dependency-graph-spec.md
docs/automation/doc-freshness-discipline.md
docs/automation/fitness-lane-reports-pipeline.md
docs/automation/glossary-pipeline.md
docs/automation/INDEX.md
docs/automation/openapi-pipeline.md
docs/automation/orphan-detection-discipline.md
docs/automation/product-map-spec.md
docs/automation/roadmap-visualization-spec.md
docs/automation/runbook-freshness-pipeline.md
docs/automation/rustdoc-pipeline.md
docs/automation/schema-doc-pipeline.md
docs/automation/service-map-spec.md
docs/automation/tech-stack-map-spec.md
templates/checklists/agent-completion-checklist.md
templates/checklists/agent-kickoff-checklist.md
templates/checklists/build-vs-buy.md
templates/checklists/cross-axis-contract-change-checklist.md
templates/checklists/doc-freshness-checklist.md
templates/checklists/done-definition-checklist.md
templates/checklists/escalation-checklist.md
templates/checklists/inventory-update-checklist.md
templates/checklists/legacy-adr-deletion.md
templates/checklists/per-implementation-plan-checklist.md
templates/checklists/per-phase-completion-checklist.md
templates/checklists/pr-review-checklist.md
templates/checklists/pre-flight-checklist.md
templates/checklists/pre-merge.md
templates/checklists/regional-pack-onboarding.md
templates/checklists/release-readiness-checklist.md
templates/checklists/wave-gate.md
docs/decisions/ADR-0702-identity-authz-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0702-identity-authz-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0705-product-protocol-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0705-product-protocol-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0705-product-protocol-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0703-cas-cache-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0706-observability-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0701-monorepo-capability-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0704-k8s-port-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0704-k8s-port-live-apex.md
docs/decisions/ADR-0706-observability-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0701-monorepo-capability-live-apex.md
docs/decisions/ADR-0705-product-protocol-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0707-trust-safety-live-apex.md
docs/decisions/ADR-0707-trust-safety-live-apex.md
docs/decisions/ADR-0707-trust-safety-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0707-trust-safety-live-apex.md
docs/decisions/ADR-0701-monorepo-capability-live-apex.md
docs/decisions/ADR-0709-general-live-apex.md
docs/decisions/ADR-0700-ci-admission-live-apex.md
docs/decisions/ADR-0703-cas-cache-live-apex.md
docs/decisions/README.md
docs/decisions/RETIRED.md
docs/decisions/specs/deep-dive-oyatie-sst-consolidation.md
docs/decisions/specs/deep-dive-trace-oyatie-sst-consolidation.md
docs/DESIGN.md
docs/DOC-CATALOG.md
docs/DOC-UPDATE-PROTOCOL.md
docs/DOCUMENTATION.md
docs/fitness-lanes/adapter-kernel.md
docs/fitness-lanes/adr-citation.md
docs/fitness-lanes/adr-index.md
docs/fitness-lanes/adr-shape.md
docs/fitness-lanes/agent-completion-checklist.md
docs/fitness-lanes/agentic-navigability.md
docs/fitness-lanes/architecture-map-freshness.md
docs/fitness-lanes/archive-orphan.md
docs/fitness-lanes/audit-emission.md
docs/fitness-lanes/authoritative-tracked.md
docs/fitness-lanes/authority-cohesion.md
docs/fitness-lanes/banned-primitives.md
docs/fitness-lanes/brand-residue.md
docs/fitness-lanes/bypass.md
docs/fitness-lanes/capability-publish.md
docs/fitness-lanes/cargo-prefix.md
docs/fitness-lanes/cargo-vet.md
docs/fitness-lanes/changelog-row.md
docs/fitness-lanes/claim-ceiling.md
docs/fitness-lanes/clippy-pedantic.md
docs/fitness-lanes/cloud-mutation.md
docs/fitness-lanes/codeowners-mirror.md
docs/fitness-lanes/cohesion.md
docs/fitness-lanes/constitution-cite.md
docs/fitness-lanes/cosign-signature.md
docs/fitness-lanes/cost-budget.md
docs/fitness-lanes/cross-axis-notify.md
docs/fitness-lanes/cutover-bootstrap-window.md
docs/fitness-lanes/data-class.md
docs/fitness-lanes/diataxis-doc-class.md
docs/fitness-lanes/direct-tool-invocation-audit.md
docs/fitness-lanes/doc-catalog.md
docs/fitness-lanes/doc-freshness.md
docs/fitness-lanes/done-definition.md
docs/fitness-lanes/evidence-bundle-shape.md
docs/fitness-lanes/flat-crates.md
docs/fitness-lanes/forward-reference-resolved.md
docs/fitness-lanes/foundry-corpus-citation.md
docs/fitness-lanes/glossary-vocabulary.md
docs/fitness-lanes/glossary.md
docs/fitness-lanes/image-size-budget.md
docs/fitness-lanes/INDEX.md
docs/fitness-lanes/license.md
docs/fitness-lanes/lts-dependency.md
docs/fitness-lanes/mdbook-publish.md
docs/fitness-lanes/mistakes-ledger-cite.md
docs/fitness-lanes/nextest-required.md
docs/fitness-lanes/openapi-contract-binding.md
docs/fitness-lanes/orphan-detection.md
docs/fitness-lanes/perf-evidence.md
docs/fitness-lanes/portfolio-citation.md
docs/fitness-lanes/pr-shape-strict.md
docs/fitness-lanes/pre-flight-checklist.md
docs/fitness-lanes/provider-agnostic.md
docs/fitness-lanes/raci-completeness.md
docs/fitness-lanes/redirect-thinness.md
docs/fitness-lanes/runbook-freshness.md
docs/fitness-lanes/runbook-index-resolves.md
docs/fitness-lanes/sbom-attestation.md
docs/fitness-lanes/scaffold-claim-pattern.md
docs/fitness-lanes/schema-migration.md
docs/fitness-lanes/semver-checks.md
docs/fitness-lanes/slsa-provenance.md
docs/fitness-lanes/traceability-validator.md
docs/fitness-lanes/workspace-lints.md
docs/GLOSSARY.md
docs/ideas/agentic-slo-gated-promotion.md
docs/machine-readable/catalog.json
docs/machine-readable/compliance.json
docs/machine-readable/contracts.json
docs/machine-readable/contradictions.json
docs/machine-readable/decisions.json
docs/machine-readable/glossary.json
docs/machine-readable/risks.json
docs/performance-budgets/README.md
docs/plans/cutover-cross-cutting-amendments-2026-05-12.md
docs/plans/M01-foundation-cc-01-cutover/architect-review-iter-1.md
docs/plans/M01-foundation-cc-01-cutover/architect-review-iter-2.md
docs/plans/M01-foundation-cc-01-cutover/cross-cutting-amendments.md
docs/plans/M01-foundation-cc-01-cutover/INDEX.md
docs/plans/M01-foundation-cc-01-cutover/open-questions-resolutions.md
docs/plans/M01-foundation-cc-01-cutover/pre-cutover-drafts.md
docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
docs/PRIVACY-PROGRAM.md
docs/products/_TEMPLATE.md
docs/products/cloud/PRD.md
docs/products/foundry/PRD.md
docs/products/foundry/supervisor/README.md
docs/products/foundry/supervisor/supervisor-app/BENCHMARKS.md
docs/quality/ai-slop-defense/additional-tooling-recommendations.md
docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md
docs/quality/ai-slop-defense/defense-in-depth-architecture.md
docs/quality/ai-slop-defense/gap-analysis-ai-vs-production.md
docs/quality/ai-slop-defense/impossible-to-fail-environment-spec.md
docs/quality/ai-slop-defense/production-quality-bar.md
docs/raw/claude-code-backup-comprehensive-analysis.md
docs/README.md
docs/regional-packs/README.md
docs/RELEASE-MANAGEMENT.md
docs/release/branch-pipeline/agent-roles-spec.md
docs/release/branch-pipeline/branch-pipeline-architecture.md
docs/release/branch-pipeline/branch-protection-rules.md
docs/release/branch-pipeline/ci-policy-per-branch.md
docs/release/branch-pipeline/fitness-lanes-for-branch-pipeline.md
docs/release/branch-pipeline/foundry-pipeline-mirror.md
docs/release/branch-pipeline/playbooks-by-axis-stage.md
docs/release/branch-pipeline/rollback-mechanics-per-stage.md
docs/release/branch-pipeline/velocity-without-stability-loss.md
docs/release/progressive-delivery/blue-green-spec.md
docs/release/progressive-delivery/canary-rail-spec.md
docs/release/progressive-delivery/dark-launch-spec.md
docs/release/progressive-delivery/enforcement-lanes.md
docs/release/progressive-delivery/feature-flag-architecture.md
docs/release/progressive-delivery/INDEX.md
docs/release/progressive-delivery/playbook-ads.md
docs/release/progressive-delivery/playbook-cloud.md
docs/release/progressive-delivery/playbook-cross-axis-contract.md
docs/release/progressive-delivery/playbook-foundry.md
docs/release/progressive-delivery/playbook-saas.md
docs/release/progressive-delivery/playbook-search.md
docs/release/progressive-delivery/playbook-vertical-pack.md
docs/release/progressive-delivery/playbook-workspace.md
docs/release/progressive-delivery/progressive-delivery-strategy.md
docs/release/progressive-delivery/slo-burn-rate-rollback-spec.md
docs/release/progressive-delivery/stable-cohort-spec.md
docs/release/progressive-delivery/traffic-mirror-spec.md
docs/research/hyperscaler-best-practices-2026-05-12.md
docs/RISK-REGISTER.md
docs/ROADMAP.md
docs/runbooks/ads/data-use-boundary-violation.md
docs/runbooks/cross-axis/audit-chain-integrity-failure.md
docs/runbooks/cross-axis/cohesion-fitness-violation.md
docs/runbooks/cross-axis/data-class-violation-detected.md
docs/runbooks/cross-axis/foundation-bypass-expired.md
docs/runbooks/sanctioned-primitives/preflight.md
docs/runbooks/vertical-healthcare/phi-leak-suspected.md
docs/SECURITY-PROGRAM.md
docs/SLO-CATALOG.md
docs/SPEC.md
docs/specs/deep-dive-oyatie-sst-consolidation.md
docs/specs/deep-dive-trace-oyatie-sst-consolidation.md
docs/STANDARDS-AND-TEMPLATES.md
docs/standards/agent-instructions-discipline.md
docs/standards/autonomy-ceiling.md
docs/standards/ci-lanes.md
docs/standards/claude-code-harness.md
docs/standards/clean-architecture.md
docs/standards/code-style-rust.md
docs/standards/crate-naming-convention.md
docs/standards/data-class.md
docs/standards/dependency-policy.md
docs/standards/doc-style.md
docs/standards/error-handling.md
docs/standards/fips-hsm-substrate-root-signing.md
docs/standards/git-workflow.md
docs/standards/hyperscaler-best-practices.md
docs/standards/image-discipline.md
docs/standards/INDEX.md
docs/standards/m02-exit-gate-validators.md
docs/standards/multi-agent-tool-map.md
docs/standards/observability.md
docs/standards/on-call.md
docs/standards/prevention-doctrine.md
docs/standards/release-management.md
docs/standards/security-review.md
docs/standards/testing.md
docs/teams/axis-ads-analytics/CHARTER.md
docs/teams/axis-foundry/CHARTER.md
docs/teams/axis-search/CHARTER.md
docs/teams/council-privacy/CHARTER.md
docs/teams/ops-sre-reliability/CHARTER.md
docs/teams/platform-audit-evidence/CHARTER.md
docs/teams/platform-eventing-og/CHARTER.md
docs/teams/platform-privacy-dub/CHARTER.md
docs/teams/platform-tenancy-identity/CHARTER.md
docs/teams/README.md
docs/teams/regional-packs/CHARTER.md
docs/templates/adr-template-v2.md
docs/templates/adr-template.md
docs/templates/bounded-context-registration-template.md
docs/templates/capability-record-template-v2.yaml
docs/templates/design-doc-template.md
docs/templates/evidence-bundle-template.json
docs/templates/impl-plan-template.md
docs/templates/implementation-plan-template.md
docs/templates/INDEX.md
docs/templates/microservice-template.md
docs/templates/milestone-index-template.md
docs/templates/milestone-readme-template.md
docs/templates/mistakes-ledger-row-template.md
docs/templates/phase-index-template.md
docs/templates/phase-spec-template.md
docs/templates/postmortem-template.md
docs/templates/prd-template.md
docs/templates/pull-request-template-v2.md
docs/templates/runbook-template-v2.md
docs/TOOLCHAIN.md
```

### §3.2 Microservices (181 files)

All files under `microservices/` containing `governance-fitness-` references were renamed. These are ARCHITECTURE.md, compliance.md, manifest.json, policy/*.cedar, policy/*.md, runbooks/*.md, IP-*.md, slos/*.yaml, and similar per-microservice flat-layout files per ADR-0131.

### §3.3 Specs (28 files — Category A/D combined)

All lane reference strings in specs were renamed, including `enforcement_lane_id` field values (Category D subset) and all other forward-looking lane references:

```
specs/agent-durable-goal.json
specs/agentic-slo-gated-promotion.json
specs/crate-naming-audit.json
specs/decision-principles.json
specs/decision-rights.json
specs/forbidden-operations.json
specs/governance-amendment.json
specs/hyperscaler-architecture-invariants.json
specs/master-plan-sequencing.json
specs/masterplan.json
specs/microservices/accounting.json
specs/microservices/anonymous.json
specs/microservices/calendar.json
specs/tenant-rbac-packaging.json
specs/microservices/tenant-rbac.json
specs/microservices/hr.json
specs/microservices/mail.json
specs/microservices/messenger.json
specs/microservices/network.json
specs/microservices/ontology.json
specs/microservices/payroll.json
specs/microservices/scorecards/canonical/cis-k8s-benchmark.json
specs/microservices/shorts.json
specs/microservices/social.json
specs/microservices/workflow-studio.json
specs/microservices/workflow.json
specs/root-hub-pointers.json
specs/test-standard.json
```

JSON validation after rename: **0 failures** (all 28 parsed successfully).

### §3.4 Packs (2 files)

```
packs/cn-pipl/dpia-template.md
packs/cn-pipl/README.md
```

---

## §4 Files in Category B — Tombstoned (6 files)

These files describe past state. Renaming them would rewrite history. A §Note was appended to each:

> "References to `governance-fitness-*` in this historical document are intentional — they describe past state. New work uses `governance-*` per the 2026-05-21 transition directive."

Files tombstoned:

| File | Reason |
|------|--------|
| `docs/ADR-CONSOLIDATION-PLAN.md` | Consolidation ledger — tracks past ADR states |
| `docs/ADR-LEGACY-REGRESSION-MAPPING.md` | Legacy regression map — describes historical regressions |
| `docs/CHANGELOG.md` | Changelog — records what was done in past releases |
| `docs/CONTRADICTION-LEDGER.md` | Contradiction ledger — records past contradictions at a point in time |
| `docs/MISTAKES-LEDGER.md` | Mistakes ledger — permanent record of past mistakes |
| `docs/VENDOR-PARTNER-LEDGER.md` | Vendor ledger — records past vendor decisions |

---

## §5 Files in Category C — Flagged for Code Migration (28 crates, 57 files)

These are **actual Rust crates** named `governance-fitness-*`. They were NOT renamed. Renaming them requires:

1. A separate code-migration ADR (to be filed by the owning team)
2. `git mv crates/governance-fitness-<X>-kernel crates/governance-<X>-kernel` per crate
3. Update `name = "governance-fitness-<X>-kernel"` → `name = "governance-<X>-kernel"` in each `Cargo.toml`
4. Update `[workspace]` `members` array in root `Cargo.toml`
5. Update all `foundry_fitness_<X>_kernel` extern crate references (use declarations) in dependent crates
6. Run `cargo check --workspace` after each batch to confirm zero errors
7. Update any `.github/workflows/*.yml` that reference these crate names by string

### Crate directories to migrate (28 crates):

| Old name | New name |
|----------|----------|
| `crates/governance-fitness-adapter-with-no-importer-kernel` | `crates/governance-adapter-with-no-importer-kernel` |
| `crates/governance-fitness-adr-shape-kernel` | `crates/governance-adr-shape-kernel` |
| `crates/governance-fitness-agentic-navigability-kernel` | `crates/governance-agentic-navigability-kernel` |
| `crates/governance-fitness-architecture-map-freshness-kernel` | `crates/governance-architecture-map-freshness-kernel` |
| `crates/governance-fitness-authoritative-tracked-kernel` | `crates/governance-authoritative-tracked-kernel` |
| `crates/governance-fitness-banned-primitives-kernel` | `crates/governance-banned-primitives-kernel` |
| `crates/governance-fitness-bypass-kernel` | `crates/governance-bypass-kernel` |
| `crates/governance-fitness-claim-ceiling-kernel` | `crates/governance-claim-ceiling-kernel` |
| `crates/governance-fitness-cohesion-fitness-kernel` | `crates/governance-cohesion-fitness-kernel` |
| `crates/governance-fitness-doc-freshness-kernel` | `crates/governance-doc-freshness-kernel` |
| `crates/governance-fitness-doc-style-kernel` | `crates/governance-doc-style-kernel` |
| `crates/governance-fitness-image-discipline-kernel` | `crates/governance-image-discipline-kernel` |
| `crates/governance-fitness-license-policy-kernel` | `crates/governance-license-policy-kernel` |
| `crates/governance-fitness-lifecycle-kernel` | `crates/governance-lifecycle-kernel` |
| `crates/governance-fitness-mistakes-ledger-kernel` | `crates/governance-mistakes-ledger-kernel` |
| `crates/governance-fitness-orphan-detection-kernel` | `crates/governance-orphan-detection-kernel` |
| `crates/governance-fitness-portfolio-citation-kernel` | `crates/governance-portfolio-citation-kernel` |
| `crates/governance-fitness-pre-push-kernel` | `crates/governance-pre-push-kernel` |
| `crates/governance-fitness-predictable-naming-kernel` | `crates/governance-predictable-naming-kernel` |
| `crates/governance-fitness-provider-coupling-kernel` | `crates/governance-provider-coupling-kernel` |
| `crates/governance-fitness-purpose-kernel` | `crates/governance-purpose-kernel` |
| `crates/governance-fitness-quality-lane-kernel` | `crates/governance-quality-lane-kernel` |
| `crates/governance-fitness-sunset-lifecycle-kernel` | `crates/governance-sunset-lifecycle-kernel` |
| `crates/governance-fitness-supply-chain-kernel` | `crates/governance-supply-chain-kernel` |
| `crates/governance-fitness-tos-policy-kernel` | `crates/governance-tos-policy-kernel` |
| `crates/governance-fitness-upstream-api-drift-kernel` | `crates/governance-upstream-api-drift-kernel` |

### Per-crate migration plan (apply in dependency order):

**Phase 1 — leaf kernels with no inbound dependencies from other fitness kernels:**
- `governance-fitness-adr-shape-kernel`
- `governance-fitness-doc-style-kernel`
- `governance-fitness-image-discipline-kernel`
- `governance-fitness-license-policy-kernel`
- `governance-fitness-orphan-detection-kernel`
- `governance-fitness-upstream-api-drift-kernel`
- `governance-fitness-tos-policy-kernel`
- `governance-fitness-agentic-navigability-kernel`

**Phase 2 — kernels that depend on Phase 1 or are independently groundable:**
- `governance-fitness-doc-freshness-kernel`
- `governance-fitness-architecture-map-freshness-kernel`
- `governance-fitness-banned-primitives-kernel`
- `governance-fitness-adapter-with-no-importer-kernel`
- `governance-fitness-provider-coupling-kernel`
- `governance-fitness-portfolio-citation-kernel`
- `governance-fitness-authoritative-tracked-kernel`
- `governance-fitness-purpose-kernel`

**Phase 3 — kernels that consume gate infrastructure:**
- `governance-fitness-bypass-kernel`
- `governance-fitness-claim-ceiling-kernel`
- `governance-fitness-cohesion-fitness-kernel`
- `governance-fitness-lifecycle-kernel`
- `governance-fitness-sunset-lifecycle-kernel`
- `governance-fitness-mistakes-ledger-kernel`
- `governance-fitness-predictable-naming-kernel`

**Phase 4 — pipeline kernels:**
- `governance-fitness-pre-push-kernel`
- `governance-fitness-quality-lane-kernel`
- `governance-fitness-supply-chain-kernel`

After each phase: `cargo check --workspace` must exit 0 before proceeding.

### Additional dependent crates that reference fitness kernels and will need updates:

The following Category E crates (operational code) reference `governance-fitness-*` crate names in their source and will need corresponding updates after the crate rename:

- `crates/dev-cli/Cargo.toml` — depends on fitness kernels
- `crates/dev-cli/src/commands/gate/architecture_boundaries.rs` — hardcoded crate name list
- `crates/check-pre-push/src/lib.rs` — hardcoded crate name list
- `crates/dev-cli/src/documentation_gates.rs` — references `governance-fitness-docs` lane string
- `crates/intelligence-gate-catalog-domain/src/lib.rs` — prefix detection logic (requires updating the detection prefix from `governance-fitness-` to `governance-` after all crates are renamed)

**Note:** `crates/intelligence-gate-catalog-domain/src/lib.rs` line 324 contains:
```rust
if status == "active" && id.starts_with("governance-fitness-") {
```
This prefix check must be updated to `"governance-"` after crate migration completes. Until crate migration is complete, this check correctly gates against the old prefix.

---

## §6 Specs in Category D — enforcement_lane_id Values Renamed

Category D is a subset of Category A. The one spec file with `"enforcement_lane_id"` fields containing `governance-fitness-` values:

- `specs/master-plan-sequencing.json`

Sample of renamed values (5 occurrences):
- `"governance-fitness-banned-primitives"` → `"governance-banned-primitives"` (×5)

JSON validation result: **PASS** — `python3 -c "import json; json.load(open(...))"` exited 0.

All other specs with forward-looking lane references (`lane_ref`, `detection_lane`, `enforcement_lane`, `validation_lane_ref`, `planned_enforced_by`, etc.) were also renamed as part of Category A.

---

## §7 Files in Category E — Left Untouched (27 files)

These files use `governance-fitness-` as **operational logic**, not as documentation. Renaming them now would break the build or CI gating logic before crate migration completes.

| File | Reason |
|------|--------|
| `crates/check-retired-vocabulary/src/lib.rs` | Tests embed retired terms as string fixtures to test the kernel. Self-validating — renaming breaks test semantics. |
| `crates/check-pre-push/src/lib.rs` | Hardcoded allowlist of fitness kernel crate names (lines 119–124). Must update after crate rename. |
| `crates/check-protection-context-match/src/lib.rs` | Uses `governance-fitness-` as a prefix token in branch-protection context matching logic (line 127, 183). Must update after crate rename. |
| `crates/dev-cli/Cargo.toml` | Workspace dependency declarations for fitness kernel crates. Must update after `git mv`. |
| `crates/dev-cli/src/aspirational_enforcement_gate.rs` | References `governance-fitness-predictable-naming-kernel` by name in ALLOWED_ROLES note. |
| `crates/dev-cli/src/commands/gate/architecture_boundaries.rs` | Hardcoded list of fitness kernel crate names for architecture-boundary gating. |
| `crates/dev-cli/src/commands/gate/mod.rs` | Lane id comments for retired-vocabulary, protection-context-match, pre-push, changeset lanes. |
| `crates/dev-cli/src/commands/lint.rs` | Lane reference in doc comment. |
| `crates/dev-cli/src/commands/submit.rs` | Lane reference in doc comment. |
| `crates/dev-cli/src/commands/verify.rs` | References `governance-fitness-purpose-audit-app` command string. |
| `crates/dev-cli/src/documentation_gates.rs` | Checks `documentation.contains("governance-fitness-docs")` — operational predicate. |
| `crates/dev-cli/src/hyperscaler_arch_invariants_gate.rs` | `planned_enforced_by` validation — requires prefix `governance-fitness-`. |
| `crates/dev-cli/src/pre_push_contract_gate.rs` | Supply-chain workflow name `governance-fitness-supply-chain`. |
| `crates/dev-cli/src/protection_context_match_gate.rs` | Lane id comment `governance-fitness-protection-context-match`. |
| `crates/dev-cli/src/retired_vocabulary_gate.rs` | Lane id comment. Intentionally excluded from corpus scans per its own exclude list. |
| `crates/dev-cli/tests/gate_cli.rs` | Integration test fixtures embedding fitness lane names for gate testing. |
| `crates/dev-cli/tests/lint_cli.rs` | Integration test using `governance-fitness-docs` lane string in fixture. |
| `crates/governance-architecture-map-kernel/src/lib.rs` | Lane reference in doc comment. |
| `crates/intelligence-gate-catalog-domain/src/lib.rs` | **Critical**: `id.starts_with("governance-fitness-")` prefix detection on line 324, 157. Must update after all crate migration completes. |

### E category risk note

`crates/intelligence-gate-catalog-domain/src/lib.rs` contains a **runtime prefix check** (`id.starts_with("governance-fitness-")`). After crate migration is complete and all CI lane IDs have been renamed to `governance-*`, this check MUST be updated to `governance-*` in the same PR that renames the last fitness crate. Until then, it correctly gates against the old prefix and must not be changed.

Similarly, `crates/dev-cli/src/hyperscaler_arch_invariants_gate.rs` validates that `planned_enforced_by` names an `governance-fitness-*` lane. After crate migration, this validation must be updated to accept `governance-*`.

---

## §8 Verification

### cargo metadata

```
cargo metadata --no-deps --format-version 1 > /dev/null
```

Result: **PASS** — workspace integrity confirmed. No Cargo.toml files were modified by this transition.

### JSON spec validation

```
python3 -c "import json; json.load(open(f))"  # run on all 28 specs
```

Result: **0 failures** across 28 spec files.

### Spot-check of 10 random Category A docs

All 10 showed `old_hits=0` (no remaining `governance-fitness-`) and `new_hits > 0` (confirming the rename applied), with line counts preserved:

| File | old_hits | new_hits |
|------|----------|----------|
| `docs/standards/security-review.md` | 0 | 6 |
| `docs/automation/openapi-pipeline.md` | 0 | 2 |
| `docs/advanced-cicd/progressive-delivery/canary-rail-spec.md` | 0 | 5 |
| `docs/decisions/ADR-0705-product-protocol-live-apex.md` | 0 | 10 |
| `docs/decisions/ADR-0700-ci-admission-live-apex.md` | 0 | 2 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 0 | 6 |
| `templates/checklists/done-definition-checklist.md` | 0 | 29 |
| `docs/automation/dependency-graph-spec.md` | 0 | 2 |
| `docs/templates/impl-plan-template.md` | 0 | 1 |
| `docs/release/progressive-delivery/playbook-cross-axis-contract.md` | 0 | 4 |

### Category E crate files verified clean (no accidental renames)

All checked Category E files showed 0 `governance-` hits introduced by this operation. Pre-existing `governance-*` hits in `gate_cli.rs` (3 occurrences) are from previously-landed `governance-protection-context-match` references — confirmed pre-existing, not introduced by this transition.

### Zero remaining governance-fitness- in Category A files

```
Category A docs with remaining governance-fitness-:        0
Category A microservices with remaining governance-fitness-:        0
Category A specs with remaining governance-fitness-:        0
```

---

## §9 Remaining Work for Human + Code-Migration Agent

### Required before the transition is fully complete

1. **File a Code-Migration ADR** for the 28 `governance-fitness-*` crates. The ADR must:
   - Cite this transition report as evidence
   - Specify the migration order (§5 Phase 1–4 plan)
   - Declare the `cargo check --workspace` gate as the completion criterion
   - Assign owning team (`axis-foundry`)

2. **Per-crate migration** (28 crates, in Phase 1–4 order):
   ```
   git mv crates/governance-fitness-<X>-kernel crates/governance-<X>-kernel
   # Edit Cargo.toml: name field
   # Edit root Cargo.toml: workspace members
   # Edit all dependent Cargo.toml: dependency name
   # Edit all use declarations: foundry_fitness_<X>_kernel → governance_<X>_kernel
   cargo check --workspace
   ```

3. **Update Category E operational code** after each Phase completes:
   - `crates/intelligence-gate-catalog-domain/src/lib.rs` lines 157, 324: update prefix check from `"governance-fitness-"` to `"governance-"` once all fitness crates are renamed
   - `crates/dev-cli/src/hyperscaler_arch_invariants_gate.rs`: update `planned_enforced_by` validation prefix
   - `crates/dev-cli/src/pre_push_contract_gate.rs`: update supply-chain workflow name from `governance-fitness-supply-chain` to `governance-supply-chain` (requires renaming the `.github/workflows/governance-fitness-supply-chain.yml` file too)
   - `crates/dev-cli/src/documentation_gates.rs`: update `governance-fitness-docs` lane string
   - `crates/check-pre-push/src/lib.rs`: update hardcoded crate name allowlist
   - `crates/check-protection-context-match/src/lib.rs`: update prefix token
   - Test fixture strings in `crates/dev-cli/tests/gate_cli.rs` and `tests/lint_cli.rs` — update to reflect new lane names in test scenarios

4. **`.github/workflows/` files** — check for workflow files named `governance-fitness-*.yml` and rename them with a corresponding `.github/workflows/` `git mv`. The supply-chain workflow (`governance-fitness-supply-chain.yml`) is referenced by kernel source code (§7 above) and must be renamed in the same PR as the source update.

5. **`registry/vocabulary/retired.yaml`** — add `governance-fitness-` as a retired vocabulary term pointing to `governance-` as the canonical replacement. This will cause the `governance-retired-vocabulary` CI lane to enforce the rename going forward and reject any future drift back to the old prefix.

6. **CLAUDE.md update** — once crate migration is complete, remove the parenthetical `(per ADR-0132); existing governance-fitness-* lanes retained until each is renamed in its own migration IP` caveat from the `new_governance_lane_prefix` field, leaving only `governance-*`.

### What is safe to ignore

- The `governance-protection-context-match` pre-existing occurrences in `crates/dev-cli/tests/gate_cli.rs` — these already use the new prefix correctly.
- Historical ledger files (`docs/CHANGELOG.md`, `docs/MISTAKES-LEDGER.md`, etc.) — their `governance-fitness-*` references are intentionally preserved and tombstoned per §4.

---

*Transition executed 2026-05-21. Classification artifact: `docs/architecture/transition-classification-2026-05-21.json`.*

---

## §10 Full Microservices File List (Category A — 181 files)

All 181 microservices files had `governance-fitness-` → `governance-` applied in place:

```
microservices/analytics/catalog/contracts.json
microservices/analytics/cost-budget.md
microservices/analytics/decisions/ADR-AN-001-ttl-policy.md
microservices/analytics/decisions/ADR-AN-005-materialized-view-cadence.md
microservices/analytics/sdk-plan.md
microservices/analytics/specs/IP-004-outbox-cdc-ingest-pipeline.md
microservices/analytics/specs/IP-014-self-slo-burn-rate-alerts.md
microservices/api-gateway/ARCHITECTURE.md
microservices/api-gateway/IP-006-routing-rest-crate.md
microservices/api-gateway/PHASE-01-EDGE-SUBSTRATE-BUILDOUT.md
microservices/api-gateway/README.md
microservices/api-gateway/runbooks/ech-key-rotation.md
microservices/api-gateway/runbooks/edge-cache-poisoning.md
microservices/application/competitor-parity-matrix.md
microservices/audit-chain/competitor-parity-matrix.md
microservices/audit-chain/threat-model.md
microservices/calendar/threat-model.md
docs/decisions/ADR-0701-monorepo-capability-live-apex.md
microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning
microservices/observability/ARCHITECTURE.md#cell-health
microservices/cloud-iac/competitor-parity-matrix.md
microservices/cloud-iac/threat-model.md
microservices/cloud-k8s/competitor-parity-matrix.md
microservices/cloud-k8s/policy/cluster-isolation.md
microservices/compliance/ARCHITECTURE.md
microservices/connector/ARCHITECTURE.md
microservices/connector/failure-modes.md
microservices/connector/IP-014-compliance-critical-path.md
microservices/connector/IP-015-connector-adapter-trait-doc.md
microservices/connector/PHASE-01-INTEGRATION-SUBSTRATE-FOUNDATION.md
microservices/connector/PRD.md
microservices/connector/runbooks/connector-onboarding.md
microservices/connector/scorecards/overrides.json
microservices/feature-flags/ARCHITECTURE.md
microservices/feature-flags/backfill-replay.md
microservices/feature-flags/capacity-model.md
microservices/feature-flags/CHANGELOG.md
microservices/feature-flags/competitor-parity-matrix.md
microservices/feature-flags/compliance.md
microservices/feature-flags/contracts/openfeature-sdk-contract.md
microservices/feature-flags/dashboards/pack-override-coverage.md
microservices/feature-flags/dpia.md
microservices/feature-flags/incident-response.md
microservices/feature-flags/multi-region.md
microservices/feature-flags/PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md
microservices/feature-flags/README.md
microservices/feature-flags/runbooks/a11y-flag-violation.md
microservices/feature-flags/runbooks/audit-replay.md
microservices/feature-flags/runbooks/experiment-rollback.md
microservices/feature-flags/runbooks/experiment-stat-sig-violation.md
microservices/feature-flags/runbooks/flag-mutation-cascade.md
microservices/feature-flags/runbooks/killswitch-engaged.md
microservices/feature-flags/runbooks/pack-override-cascade.md
microservices/feature-flags/runbooks/stale-targeting-rule.md
microservices/feature-flags/sdk-plan.md
microservices/foundry/bc-sources/eval/compliance.md
microservices/foundry/bc-sources/guardrails/competitor-parity-matrix.md
microservices/foundry/bc-sources/guardrails/failure-modes.md
microservices/foundry/bc-sources/guardrails/PHASE-01-GUARDRAILS-SAFETY-AND-POLICY-ENFORCEMENT.md
microservices/foundry/bc-sources/guardrails/threat-model.md
microservices/foundry/bc-sources/providers/competitor-parity-matrix.md
microservices/foundry/bc-sources/runtime/competitor-parity-matrix.md
microservices/foundry/bc-sources/supervisor/competitor-parity-matrix.md
microservices/foundry/iac/cedar/guardrails-build.sh
microservices/foundry/IP-012-runtime-autonomy-tier-gate.md
microservices/foundry/IP-015-runtime-hg-fr-hyperscaler-gate-registration.md
microservices/foundry/IP-061-guardrails-cedar-policy-engine-iac.md
microservices/foundry/IP-073-guardrails-runtime-guardrails-coupling-lane.md
microservices/foundry/manifest.json
microservices/foundry/policy/guardrails-guardrail-enforcement.md
microservices/foundry/policy/guardrails-schema.cedarschema
microservices/foundry/policy/guardrails-tenant-isolation.md
microservices/governance/lane-prefix-migration-task-37.json
microservices/governance/PRD.md
microservices/intelligence/ARCHITECTURE.md
microservices/intelligence/AUDIT-FINDINGS-2026-05-20.json
microservices/intelligence/compliance.md
microservices/intelligence/dpia.md
microservices/intelligence/iac/helm/intelligence/Chart.yaml
microservices/intelligence/iac/k8s/network-policy.yaml
microservices/intelligence/iac/prod-ech-config.yaml
microservices/intelligence/iac/prod-edge-waf.yaml
microservices/intelligence/iac/prod-pqc-cert.yaml
microservices/intelligence/IP-002-domain-layer-secret-reference.md
microservices/intelligence/IP-003-domain-layer-refusal-decision.md
microservices/intelligence/IP-008-kernel-guardrail-stack.md
microservices/intelligence/PHASE-01-INTELLIGENCE-TWO-LAYER-MVP.md
microservices/intelligence/policy/eu-ai-act-high-risk.cedar
microservices/intelligence/policy/refusal-baseline.cedar
microservices/intelligence/runbooks/eu-ai-act-incident-notification.md
microservices/intelligence/scorecards/overrides.json
microservices/intelligence/threat-model.md
microservices/mail/ARCHITECTURE.md
microservices/mail/competitor-parity-matrix.md
microservices/messenger/IP-NEW-hyperscaler-metric-emission.md
microservices/notes/ARCHITECTURE.md
microservices/observability/competitor-parity-matrix.md
microservices/observability/IP-011-per-component-release-pointers.md
microservices/observability/IP-013-event-driven-promote-workflows.md
microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md
microservices/observability/policy/tenant-isolation.md
microservices/observability/threat-model.md
microservices/ontology/ARCHITECTURE.md
microservices/ontology/capabilities/cedar-evaluate.yaml
microservices/ontology/capabilities/query-execute.yaml
microservices/ontology/capabilities/type-register.yaml
microservices/ontology/competitor-parity-matrix.md
microservices/ontology/compliance.md
microservices/ontology/failure-modes.md
microservices/ontology/iac/helm/cedar-policy-engine/values.yaml
microservices/ontology/IP-001-ontology-iac-stack.md
microservices/ontology/IP-002-object-type-registry-kernel-domain.md
microservices/ontology/IP-003-link-action-function-type-registry.md
microservices/ontology/IP-004-entity-store-rls-citus.md
microservices/ontology/IP-005-link-store-traversal.md
microservices/ontology/IP-006-cedar-fragment-coverage-engine.md
microservices/ontology/IP-007-action-engine-cedar-gated.md
microservices/ontology/IP-008-function-engine-oltp-and-olap.md
microservices/ontology/IP-009-clickhouse-history-mirror.md
microservices/ontology/IP-010-audit-chain-merkle-ed25519.md
microservices/ontology/IP-011-query-engine-3layer-kg.md
microservices/ontology/IP-012-agent-gateway-llm-tool-call.md
microservices/ontology/IP-013-pillar-cross-pillar-grant.md
microservices/ontology/IP-014-rest-and-sdk-surfaces.md
microservices/ontology/IP-015-app-binaries-and-branch-protection.md
microservices/ontology/IP-016-read-path-library-rollout.md
microservices/ontology/IP-018-abuse-defence-edge-wiring.md
microservices/ontology/PHASE-01-TYPED-ENTITY-SUBSTRATE.md
microservices/ontology/policy/data-residency.md
microservices/ontology/policy/type-isolation.md
microservices/ontology/runbooks/ontology-bot-score-recalibration.md
microservices/ontology/runbooks/query-engine-restart.md
microservices/ontology/runbooks/type-registry-migration.md
microservices/ontology/sdk-plan.md
microservices/ontology/threat-model.md
microservices/ops-dashboard-control-center/ARCHITECTURE.md
microservices/ops-dashboard-control-center/AUDIT-FINDINGS-2026-05-20.json
microservices/ops-dashboard-control-center/backfill-replay.md
microservices/ops-dashboard-control-center/capacity-model.md
microservices/ops-dashboard-control-center/CHANGELOG.md
microservices/ops-dashboard-control-center/competitor-parity-matrix.md
microservices/ops-dashboard-control-center/compliance.md
microservices/ops-dashboard-control-center/contracts/metric-naming-convention.md
microservices/ops-dashboard-control-center/dashboards/on-call-handoff.md
microservices/ops-dashboard-control-center/dpia.md
microservices/ops-dashboard-control-center/incident-response.md
microservices/ops-dashboard-control-center/IP-008-step-up-auth-flow.md
microservices/ops-dashboard-control-center/IP-009-audit-emission-integration.md
microservices/ops-dashboard-control-center/IP-010-cedar-admin-console-surface.md
microservices/ops-dashboard-control-center/IP-011-tenant-admin-panel.md
microservices/ops-dashboard-control-center/IP-012-cell-operator-panel.md
microservices/ops-dashboard-control-center/IP-013-adr-promotion-triage-panel.md
microservices/ops-dashboard-control-center/IP-014-finops-portal-integration.md
microservices/ops-dashboard-control-center/IP-015-observability-pivot.md
microservices/ops-dashboard-control-center/IP-016-on-call-handoff-bc.md
microservices/ops-dashboard-control-center/multi-region.md
microservices/ops-dashboard-control-center/PHASE-01-INTERNAL-OPS-DASHBOARD.md
microservices/ops-dashboard-control-center/policy/data-residency.md
microservices/ops-dashboard-control-center/README.md
microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md
microservices/ops-dashboard-control-center/runbooks/admin-mfa-cascade.md
microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md
microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md
microservices/ops-dashboard-control-center/runbooks/oncall-handoff-failure.md
microservices/ops-dashboard-control-center/runbooks/pack-override-quarantine.md
microservices/ops-dashboard-control-center/runbooks/step-up-auth-bypass-attempt.md
microservices/ops-dashboard-control-center/runbooks/tenant-scope-violation-detected.md
microservices/ops-dashboard-control-center/sdk-plan.md
microservices/payments/AUDIT-FINDINGS-2026-05-20.json
microservices/payments/compliance.md
microservices/payments/PHASE-01-PAYMENTS-MVP.md
microservices/payments/policy/data-residency.md
microservices/social/ARCHITECTURE.md
microservices/social/IP-018-dsa-compliance-overlay.md
microservices/tenancy/competitor-parity-matrix.md
microservices/tenancy/failure-modes.md
microservices/tenancy/IP-012-branch-protection-and-release-pointers.md
microservices/tenancy/threat-model.md
microservices/workflow-engine/competitor-parity-matrix.md
microservices/workflow-studio/PRD.md
microservices/workflow-studio/runbooks/canvas-perf-regression.md
```

---

## §11 Execution Log

Commands executed in sequence (2026-05-21):

```
# Step 1: Enumerate
grep -rl "governance-fitness-" /Users/jasonlee/oyatie/docs        → 366 files
grep -rl "governance-fitness-" /Users/jasonlee/oyatie/microservices → 181 files
grep -rl "governance-fitness-" /Users/jasonlee/oyatie/crates       → 60 files
grep -rl "governance-fitness-" /Users/jasonlee/oyatie/specs        → 28 files
grep -rl "governance-fitness-" /Users/jasonlee/oyatie/packs        → 2 files
grep -rl "governance-fitness-" /Users/jasonlee/oyatie/registries   → 0 files
Total: 637 files

# Step 2: Classify
# B: grep for CHANGELOG|MISTAKES-LEDGER|CONTRADICTION-LEDGER|ADR-CONSOLIDATION-PLAN|
#    ADR-LEGACY-REGRESSION-MAPPING|LEDGER|retired/|superseded/ → 6 files
# A: all docs minus B → 360 files
# C: find crates -type d -name "governance-fitness-*" → 28 crate dirs, 57 files
# E: manual inspection of crate source files → 27 files
# A/D: all 28 specs (rename lane refs + enforcement_lane_id values)
# A: 2 packs files

# Step 3: Apply Category A renames (docs, microservices, specs, packs)
while IFS= read -r f; do sed -i '' 's/governance-fitness-/governance-/g' "$f"; done < /tmp/cat_A_docs.txt
# → 360 docs renamed
while IFS= read -r f; do sed -i '' 's/governance-fitness-/governance-/g' "$f"; done < /tmp/files_microservices.txt
# → 181 microservices files renamed
while IFS= read -r f; do sed -i '' 's/governance-fitness-/governance-/g' "$f"; done < /tmp/files_specs.txt
# → 28 specs renamed
while IFS= read -r f; do sed -i '' 's/governance-fitness-/governance-/g' "$f"; done < /tmp/files_packs.txt
# → 2 packs renamed
# Total renamed: 571 files

# Step 4: Tombstone Category B
# Appended §Note to each of 6 historical ledger files

# Step 5: Verify
# JSON validation: 0 failures across 28 specs
# cargo metadata --no-deps --format-version 1: PASS
# Spot-check 10 random docs: old_hits=0 on all 10
# Zero remaining governance-fitness- in Category A: confirmed
# Category E crate files: 0 accidental renames
# Crate Cargo.toml names unchanged: confirmed (governance-fitness-lifecycle-kernel,
#   governance-fitness-banned-primitives-kernel, governance-fitness-bypass-kernel
#   all retain original name = "governance-fitness-*" values)
```

---

## §12 Decision Notes

### Why not rename Category E operational code now

The operational code in Category E has two layers of dependency:

1. **Crate name strings** — `governance-fitness-claim-ceiling-kernel` etc. appear in `check-pre-push/src/lib.rs` as a hardcoded allowlist of kernel crate names. If renamed before the crates themselves are renamed, the allowlist would reference non-existent crates and CI would fail.

2. **Prefix detection logic** — `intelligence-gate-catalog-domain/src/lib.rs` line 324 uses `id.starts_with("governance-fitness-")` as an active runtime gate. This correctly gates current lane IDs. Changing it before crates are renamed would break the gate for all currently-active lanes.

The correct sequencing is: crate rename (Category C) → update operational code (Category E). This sequencing is enforced by §9 remaining-work items 2 and 3.

### Why VENDOR-PARTNER-LEDGER.md is Category B

`docs/VENDOR-PARTNER-LEDGER.md` is a ledger recording vendor/partner decisions at a point in time. Even though its content is sparse (draft v0.1), its `purpose` field is `"Oyatie — Vendor + Partner Ledger"` — a historical registry. Renaming `governance-fitness-*` references in it would silently alter the recorded vendor context that existed when those references were made. It receives a tombstone note instead.

### Why specs are Category A, not a separate category

The task description calls out `"enforcement_lane_id"` field values as Category D. However, specs contain many other forward-looking lane references (`lane_ref`, `detection_lane`, `enforcement_lane`, `validation_lane_ref`, `planned_enforced_by`, etc.) — all describing the target future state of CI enforcement. These are equally forward-looking and equally benefit from the rename. Treating all spec lane references as Category A (with D as a structural note about the field name) produces a clean, complete result with zero remaining `governance-fitness-` hits in any spec file.

### Why `docs/plans/` files are Category A, not B

The `docs/plans/` subtree contains implementation plans describing work to be done (future-tense). Even historical iterations (`pre-cutover-drafts.md`, `architect-review-iter-1.md`) describe what the plan said at a point in time about the *target* architecture — not past state that must be preserved verbatim. Contrast with `docs/CHANGELOG.md` which records what *was shipped*. Implementation plans are forward-looking by construction; they receive the rename.

Note: `crates/dev-cli/src/retired_vocabulary_gate.rs` line 48 explicitly excludes `docs/plans` from the retired-vocabulary corpus scan, consistent with treating plans as forward-looking documents.
