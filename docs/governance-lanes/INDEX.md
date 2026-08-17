---
doc_status: published
---

# Fitness-Lane Enforcement Catalogue (INDEX)

- status: Accepted
- date: 2026-05-12

Canonical catalogue of CI-enforced fitness lanes for the oyatie SST repo.
Each row points at a `lane-spec.md` sibling file. Source of truth for lane id, kernel crate, runner binary, CI invocation, runtime budget, and severity.
Severity follows CONTRADICTION-LEDGER scale: BLOCKER (fails merge), HIGH (warns + 7d auto-blocker), MED/LOW (advisory).

Runtime tier:
- light (<= 100 ms) — every PR, pre-push
- pr-time (100 ms–1.5 s) — every PR in matrix
- heavy (> 1.5 s) — nightly + merge-queue gate

| lane id | category | enforces | kernel crate | runner | CI invocation | runtime (ms) | severity |
| --- | --- | --- | --- | --- | --- | --- | --- |
| license | existing | STANDARD/repo-license-policy | oya-governance-license-kernel | tools/oya-governance-license | `cargo run -p oya-governance-license` | 200 | BLOCKER |
| data-class | existing | STANDARD/data-class-tagging | oya-governance-data-class-kernel | tools/oya-governance-data-class | `cargo run -p oya-governance-data-class` | 600 | BLOCKER |
| cohesion | existing | STANDARD/cross-axis-cohesion | oya-governance-cohesion-fitness-kernel | tools/oya-governance-cohesion | `cargo run -p oya-governance-cohesion` | 300 | BLOCKER |
| glossary | existing | STANDARD/glossary-required-terms | oya-governance-glossary-kernel | tools/oya-governance-glossary | `cargo run -p oya-governance-glossary` | 700 | HIGH |
| adr-citation | existing | STANDARD/adr-citation | oya-governance-adr-citation-kernel | tools/oya-governance-adr-citation | `cargo run -p oya-governance-adr-citation` | 800 | BLOCKER |
| brand-residue | existing | STANDARD/brand-hygiene | oya-governance-brand-residue-kernel | tools/oya-governance-brand-residue | `cargo run -p oya-governance-brand-residue` | 900 | HIGH |
| bypass | existing | STANDARD/no-silent-bypass | oya-intelligence-bypass-kernel | tools/oya-governance-bypass | `cargo run -p oya-governance-bypass` | 700 | BLOCKER |
| flat-crates | existing | STANDARD/flat-workspace | oya-governance-flat-crates-kernel | tools/oya-governance-flat-crates | `cargo run -p oya-governance-flat-crates` | 100 | BLOCKER |
| runbook-index-resolves | existing | STANDARD/runbook-index | oya-governance-runbook-index-kernel | tools/oya-governance-runbook-index-resolves | `cargo run -p oya-governance-runbook-index-resolves` | 250 | HIGH |
| doc-catalog | existing | STANDARD/doc-catalog | oya-intelligence-catalog-kernel | tools/oya-governance-doc-catalog | `cargo run -p oya-governance-doc-catalog` | 400 | BLOCKER |
| authority-cohesion | existing | STANDARD/single-source-authority | oya-governance-authority-cohesion-kernel | tools/oya-governance-authority-cohesion | `cargo run -p oya-governance-authority-cohesion` | 350 | BLOCKER |
| mistakes-ledger-cite | existing | STANDARD/mistakes-ledger | oya-governance-mistakes-ledger-kernel | tools/oya-governance-mistakes-ledger-cite | `cargo run -p oya-governance-mistakes-ledger-cite` | 600 | HIGH |
| adr-shape | existing | TEMPLATE/adr-template | oya-governance-adr-shape-kernel | tools/oya-governance-adr-shape | `cargo run -p oya-governance-adr-shape` | 250 | BLOCKER |
| audit-emission | existing | STANDARD/audit-chain | oya-governance-audit-emission-kernel | tools/oya-governance-audit-emission | `cargo run -p oya-governance-audit-emission` | 700 | BLOCKER |
| schema-migration | existing | STANDARD/schema-migration | oya-governance-schema-migration-kernel | tools/oya-governance-schema-migration | `cargo run -p oya-governance-schema-migration` | 200 | BLOCKER |
| perf-evidence | existing | STANDARD/perf-evidence | oya-governance-perf-evidence-kernel | tools/oya-governance-perf-evidence | `cargo run -p oya-governance-perf-evidence` | 300 | HIGH |
| traceability-validator | retired | ADR-0716 D4 | retired | retired | retired; no CI invocation | 0 | retired |
| redirect-thinness | existing | STANDARD/redirect-shape | oya-governance-redirect-thinness-kernel | tools/oya-governance-redirect-thinness | `cargo run -p oya-governance-redirect-thinness` | 200 | MED |
| cross-axis-notify | existing | STANDARD/cross-axis-notify | oya-governance-cross-axis-notify-kernel | tools/oya-governance-cross-axis-notify | `cargo run -p oya-governance-cross-axis-notify` | 250 | HIGH |
| capability-publish | existing | STANDARD/capability-map | oya-intelligence-capability-kernel | tools/oya-governance-capability-publish | `cargo run -p oya-governance-capability-publish` | 400 | HIGH |
| portfolio-citation | new-directive | Directive A1 (bidirectional bominal<->oyatie PRD cite) | oya-governance-portfolio-citation-kernel | tools/oya-governance-portfolio-citation | `cargo run -p oya-governance-portfolio-citation` | 800 | HIGH |
| banned-primitives | new-directive | Directive 12 (sanctioned primitives) | oya-governance-banned-primitives-kernel | tools/oya-governance-banned-primitives | `cargo run -p oya-governance-banned-primitives` | 500 | BLOCKER |
| archive-orphan | retired | ADR-0118 / ADR-0116 / M01-P18 replacement | retired | retired | retired; no CI invocation | 0 | retired |
| authoritative-tracked | new-directive | Directive A8 | oya-governance-authoritative-tracked-kernel | tools/oya-governance-authoritative-tracked | `cargo run -p oya-governance-authoritative-tracked` | 400 | BLOCKER |
| agentic-navigability | new-directive | Directive 10 (navigability) | oya-governance-agentic-navigability-kernel | tools/oya-governance-agentic-navigability | `cargo run -p oya-governance-agentic-navigability` | 600 | HIGH |
| orphan-detection | new-directive | Directive 10 (purpose) | oya-governance-orphan-detection-kernel | tools/oya-governance-orphan-detection | `cargo run -p oya-governance-orphan-detection` | 1100 | HIGH |
| doc-freshness | new-directive | Directive 10 (staleness) | oya-governance-doc-freshness-kernel | tools/oya-governance-doc-freshness | `cargo run -p oya-governance-doc-freshness` | 700 | MED |
| lts-dependency | new-directive | Directive 8 (LTS pin) | oya-governance-lts-dependency-kernel | tools/oya-governance-lts-dependency | `cargo run -p oya-governance-lts-dependency` | 1800 | BLOCKER |
| provider-agnostic | new-directive | Directive 4 (provider-agnostic) | oya-governance-provider-agnostic-kernel | tools/oya-governance-provider-agnostic | `cargo run -p oya-governance-provider-agnostic` | 900 | BLOCKER |
| image-size-budget | new-directive | Directive 5 (image budget) | oya-governance-image-size-budget-kernel | tools/oya-governance-image-size-budget | `cargo run -p oya-governance-image-size-budget` | 2000 | HIGH |
| sbom-attestation | absorbed-by-cloud-ci | hyperscaler/SBOM | cloud-ci-supply-chain-audit | cloud/cloud-ci/gates/oya-cloud-ci-supply-chain-audit-app | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-supply-chain-audit-app:oya-cloud-ci-supply-chain-audit-app-gate` | 1500 | BLOCKER |
| cosign-signature | absorbed-by-cloud-ci | hyperscaler/Cosign | cloud-ci-supply-chain-audit | cloud/cloud-ci/gates/oya-cloud-ci-supply-chain-audit-app | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-supply-chain-audit-app:oya-cloud-ci-supply-chain-audit-app-gate` | 1800 | BLOCKER |
| slsa-provenance | absorbed-by-cloud-ci | hyperscaler/SLSA L2+ | cloud-ci-supply-chain-audit | cloud/cloud-ci/gates/oya-cloud-ci-supply-chain-audit-app | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-supply-chain-audit-app:oya-cloud-ci-supply-chain-audit-app-gate` | 1200 | BLOCKER |
| cargo-vet | retired-until-inputs | hyperscaler/cargo-vet | retired | retired | retired; no CI invocation; current dependency/advisory authority is `cloud-ci-supply-chain-audit` | 0 | retired |
| semver-checks | new-rust | hyperscaler/semver-checks | oya-intelligence-api-semver-kernel | tools/oya-governance-semver-checks | `cargo run -p oya-governance-semver-checks` | 2500 | BLOCKER |
| clippy-pedantic | new-rust | hyperscaler/clippy-pedantic | oya-governance-clippy-pedantic-kernel | tools/oya-governance-clippy-pedantic | `cargo run -p oya-governance-clippy-pedantic` | 3500 | BLOCKER |
| nextest-required | new-rust | hyperscaler/nextest | oya-governance-nextest-required-kernel | tools/oya-governance-nextest-required | `cargo run -p oya-governance-nextest-required` | 1700 | HIGH |
| foundry-corpus-citation | new-directive | MASTERPLAN P3.5 (corpus cite) | oya-governance-foundry-corpus-citation-kernel | tools/oya-governance-foundry-corpus-citation | `cargo run -p oya-governance-foundry-corpus-citation` | 500 | HIGH |
| architecture-map-freshness | new-directive | Directive 11 (visualization) | oya-governance-architecture-map-freshness-kernel | tools/oya-governance-architecture-map-freshness | `cargo run -p oya-governance-architecture-map-freshness` | 350 | MED |
| diataxis-doc-class | new-directive | Directive 10 (class shape) | oya-governance-diataxis-doc-class-kernel | tools/oya-governance-diataxis-doc-class | `cargo run -p oya-governance-diataxis-doc-class` | 700 | MED |
| runbook-freshness | new-directive | Directive 10 (incident link) | oya-governance-runbook-freshness-kernel | tools/oya-governance-runbook-freshness | `cargo run -p oya-governance-runbook-freshness` | 600 | MED |
| evidence-bundle-shape | new-template | TEMPLATE/phase-00-evidence | oya-governance-evidence-bundle-shape-kernel | tools/oya-governance-evidence-bundle-shape | `cargo run -p oya-governance-evidence-bundle-shape` | 350 | BLOCKER |
| pre-flight-checklist | new-checklist | CHECKLIST/pre-flight | oya-governance-pre-flight-checklist-kernel | tools/oya-governance-pre-flight-checklist | `cargo run -p oya-governance-pre-flight-checklist` | 200 | BLOCKER |
| done-definition | new-checklist | CHECKLIST/definition-of-done | oya-governance-done-definition-kernel | tools/oya-governance-done-definition | `cargo run -p oya-governance-done-definition` | 250 | BLOCKER |
| pr-shape-strict | new-standard | STANDARD/pr-shape | oya-governance-pr-shape-strict-kernel | tools/oya-governance-pr-shape-strict | `cargo run -p oya-governance-pr-shape-strict` | 150 | BLOCKER |
| agent-completion-checklist | new-checklist | CHECKLIST/agent-completion + ADR-0054 | oya-governance-agent-completion-checklist-kernel | tools/oya-governance-agent-completion-checklist | `cargo run -p oya-governance-agent-completion-checklist` | 600 | BLOCKER |
| scaffold-claim-pattern | new-adr | ADR-0054 | oya-governance-scaffold-claim-pattern-kernel | tools/oya-governance-scaffold-claim-pattern | `cargo run -p oya-governance-scaffold-claim-pattern` | 400 | HIGH |
| cutover-bootstrap-window | new-adr | ADR-0053 | oya-governance-cutover-bootstrap-window-kernel | tools/oya-governance-cutover-bootstrap-window | `cargo run -p oya-governance-cutover-bootstrap-window` | 500 | HIGH |
| workspace-lints | new-rust | hyperscaler/workspace lints | oya-governance-workspace-lints-kernel | tools/oya-governance-workspace-lints | `cargo run -p oya-governance-workspace-lints` | 250 | HIGH |
| forward-reference-resolved | new-standard | STANDARD/wave-gates | oya-governance-forward-reference-resolved-kernel | tools/oya-governance-forward-reference-resolved | `cargo run -p oya-governance-forward-reference-resolved` | 500 | BLOCKER |
| raci-completeness | new-standard | STANDARD/raci-ownership | oya-governance-raci-completeness-kernel | tools/oya-governance-raci-completeness | `cargo run -p oya-governance-raci-completeness` | 200 | HIGH |
| glossary-vocabulary | new-standard | STANDARD/vocabulary-retirement | oya-governance-glossary-vocabulary-kernel | tools/oya-governance-glossary-vocabulary | `cargo run -p oya-governance-glossary-vocabulary` | 500 | HIGH |
| mdbook-publish | new-standard | STANDARD/doc-publish | oya-governance-mdbook-publish-kernel | tools/oya-governance-mdbook-publish | `cargo run -p oya-governance-mdbook-publish` | 1100 | HIGH |
| openapi-contract-binding | new-standard | STANDARD/api-contract-binding | oya-governance-openapi-contract-binding-kernel | tools/oya-governance-openapi-contract-binding | `cargo run -p oya-governance-openapi-contract-binding` | 1200 | BLOCKER |
| changelog-row | new-standard | STANDARD/changelog-discipline | oya-governance-changelog-row-kernel | tools/oya-governance-changelog-row | `cargo run -p oya-governance-changelog-row` | 300 | HIGH |
| adr-index | existing-extension | STANDARD/adr-index | oya-governance-adr-index-kernel | tools/oya-governance-adr-index | `cargo run -p oya-governance-adr-index` | 250 | HIGH |
| cargo-prefix | existing-extension | STANDARD/cargo-prefix | oya-governance-cargo-prefix-kernel | tools/oya-governance-cargo-prefix | `cargo run -p oya-governance-cargo-prefix` | 150 | BLOCKER |
| codeowners-mirror | existing-extension | STANDARD/codeowners-mirror | CodeownersMirrorFitnessReport (kernel contract) | tools/oya-governance-codeowners-mirror | `cargo run -p oya-governance-codeowners-mirror` | 300 | HIGH |
| constitution-cite | existing-extension | STANDARD/constitution-derivation | oya-governance-constitution-cite-kernel | tools/oya-governance-constitution-cite | `cargo run -p oya-governance-constitution-cite` | 350 | HIGH |
| claim-ceiling | existing-extension | STANDARD/claim-ceiling | oya-governance-claim-ceiling-kernel | tools/oya-governance-claim-ceiling | `cargo run -p oya-governance-claim-ceiling` | 300 | HIGH |
| cost-budget | existing-extension | STANDARD/cost-budget | oya-intelligence-cost-budget-kernel | tools/oya-governance-cost-budget | `cargo run -p oya-governance-cost-budget` | 600 | MED |
| cloud-mutation | existing-extension | STANDARD/cloud-mutation | oya-intelligence-cloud-mutation-kernel | tools/oya-governance-cloud-mutation | `cargo run -p oya-governance-cloud-mutation` | 800 | BLOCKER |
| adapter-kernel | existing-extension | STANDARD/adapter-shape | oya-intelligence-adapter-kernel | tools/oya-governance-adapter-kernel | `cargo run -p oya-governance-adapter-kernel` | 800 | BLOCKER |

Total: 64 lanes. See sibling `<lane-id>.md` for kernel sketch, failure modes, and runtime budget.

## CHANGELOG

| date | change |
| --- | --- |
| 2026-05-12 | 64 fitness-lane specs landed in `docs/governance-lanes/`; kernel implementations in Stage 3 |

| 2026-05-16 | Retired archive-orphan lane after ADR-0116 made M01-P18 the canonical VCS substrate; removed its workspace crates, runner, catalog entries, and one-time archive payload. |
