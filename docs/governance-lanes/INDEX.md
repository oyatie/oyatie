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
| license | existing | STANDARD/repo-license-policy | governance-license-kernel | tools/governance-license | `cargo run -p governance-license` | 200 | BLOCKER |
| data-class | existing | STANDARD/data-class-tagging | governance-data-class-kernel | tools/governance-data-class | `cargo run -p governance-data-class` | 600 | BLOCKER |
| cohesion | existing | STANDARD/cross-axis-cohesion | governance-cohesion-fitness-kernel | tools/governance-cohesion | `cargo run -p governance-cohesion` | 300 | BLOCKER |
| glossary | existing | STANDARD/glossary-required-terms | governance-glossary-kernel | tools/governance-glossary | `cargo run -p governance-glossary` | 700 | HIGH |
| brand-residue | existing | STANDARD/brand-hygiene | governance-brand-residue-kernel | tools/governance-brand-residue | `cargo run -p governance-brand-residue` | 900 | HIGH |
| bypass | existing | STANDARD/no-silent-bypass | intelligence-bypass-kernel | tools/governance-bypass | `cargo run -p governance-bypass` | 700 | BLOCKER |
| flat-crates | existing | STANDARD/flat-workspace | governance-flat-crates-kernel | tools/governance-flat-crates | `cargo run -p governance-flat-crates` | 100 | BLOCKER |
| runbook-index-resolves | existing | STANDARD/runbook-index | governance-runbook-index-kernel | tools/governance-runbook-index-resolves | `cargo run -p governance-runbook-index-resolves` | 250 | HIGH |
| doc-catalog | existing | STANDARD/doc-catalog | intelligence-catalog-kernel | tools/governance-doc-catalog | `cargo run -p governance-doc-catalog` | 400 | BLOCKER |
| authority-cohesion | existing | STANDARD/single-source-authority | governance-authority-cohesion-kernel | tools/governance-authority-cohesion | `cargo run -p governance-authority-cohesion` | 350 | BLOCKER |
| mistakes-ledger-cite | existing | STANDARD/mistakes-ledger | governance-mistakes-ledger-kernel | tools/governance-mistakes-ledger-cite | `cargo run -p governance-mistakes-ledger-cite` | 600 | HIGH |
| adr-shape | existing | TEMPLATE/adr-template | governance-adr-shape-kernel | tools/governance-adr-shape | `cargo run -p governance-adr-shape` | 250 | BLOCKER |
| audit-emission | existing | STANDARD/audit-chain | governance-audit-emission-kernel | tools/governance-audit-emission | `cargo run -p governance-audit-emission` | 700 | BLOCKER |
| schema-migration | existing | STANDARD/schema-migration | governance-schema-migration-kernel | tools/governance-schema-migration | `cargo run -p governance-schema-migration` | 200 | BLOCKER |
| perf-evidence | existing | STANDARD/perf-evidence | governance-perf-evidence-kernel | tools/governance-perf-evidence | `cargo run -p governance-perf-evidence` | 300 | HIGH |
| traceability-validator | retired | ADR-0716 D4 | retired | retired | retired; no CI invocation | 0 | retired |
| redirect-thinness | existing | STANDARD/redirect-shape | governance-redirect-thinness-kernel | tools/governance-redirect-thinness | `cargo run -p governance-redirect-thinness` | 200 | MED |
| cross-axis-notify | existing | STANDARD/cross-axis-notify | governance-cross-axis-notify-kernel | tools/governance-cross-axis-notify | `cargo run -p governance-cross-axis-notify` | 250 | HIGH |
| capability-publish | existing | STANDARD/capability-map | intelligence-capability-kernel | tools/governance-capability-publish | `cargo run -p governance-capability-publish` | 400 | HIGH |
| portfolio-citation | new-directive | Directive A1 (bidirectional bominal<->oyatie PRD cite) | governance-portfolio-citation-kernel | tools/governance-portfolio-citation | `cargo run -p governance-portfolio-citation` | 800 | HIGH |
| banned-primitives | new-directive | Directive 12 (sanctioned primitives) | governance-banned-primitives-kernel | tools/governance-banned-primitives | `cargo run -p governance-banned-primitives` | 500 | BLOCKER |
| archive-orphan | retired | ADR-0118 / ADR-0116 / M01-P18 replacement | retired | retired | retired; no CI invocation | 0 | retired |
| authoritative-tracked | new-directive | Directive A8 | governance-authoritative-tracked-kernel | tools/governance-authoritative-tracked | `cargo run -p governance-authoritative-tracked` | 400 | BLOCKER |
| agentic-navigability | new-directive | Directive 10 (navigability) | governance-agentic-navigability-kernel | tools/governance-agentic-navigability | `cargo run -p governance-agentic-navigability` | 600 | HIGH |
| orphan-detection | new-directive | Directive 10 (purpose) | governance-orphan-detection-kernel | tools/governance-orphan-detection | `cargo run -p governance-orphan-detection` | 1100 | HIGH |
| doc-freshness | new-directive | Directive 10 (staleness) | governance-doc-freshness-kernel | tools/governance-doc-freshness | `cargo run -p governance-doc-freshness` | 700 | MED |
| lts-dependency | new-directive | Directive 8 (LTS pin) | governance-lts-dependency-kernel | tools/governance-lts-dependency | `cargo run -p governance-lts-dependency` | 1800 | BLOCKER |
| provider-agnostic | new-directive | Directive 4 (provider-agnostic) | governance-provider-agnostic-kernel | tools/governance-provider-agnostic | `cargo run -p governance-provider-agnostic` | 900 | BLOCKER |
| image-size-budget | new-directive | Directive 5 (image budget) | governance-image-size-budget-kernel | tools/governance-image-size-budget | `cargo run -p governance-image-size-budget` | 2000 | HIGH |
| sbom-attestation | absorbed-by-cloud-ci | hyperscaler/SBOM | cloud-ci-supply-chain-audit | cloud/cloud-ci/gates/cloud-ci-supply-chain-audit-app | `buck2 test //cloud/cloud-ci/gates/cloud-ci-supply-chain-audit-app:cloud-ci-supply-chain-audit-app-gate` | 1500 | BLOCKER |
| cosign-signature | absorbed-by-cloud-ci | hyperscaler/Cosign | cloud-ci-supply-chain-audit | cloud/cloud-ci/gates/cloud-ci-supply-chain-audit-app | `buck2 test //cloud/cloud-ci/gates/cloud-ci-supply-chain-audit-app:cloud-ci-supply-chain-audit-app-gate` | 1800 | BLOCKER |
| slsa-provenance | absorbed-by-cloud-ci | hyperscaler/SLSA L2+ | cloud-ci-supply-chain-audit | cloud/cloud-ci/gates/cloud-ci-supply-chain-audit-app | `buck2 test //cloud/cloud-ci/gates/cloud-ci-supply-chain-audit-app:cloud-ci-supply-chain-audit-app-gate` | 1200 | BLOCKER |
| cargo-vet | retired-until-inputs | hyperscaler/cargo-vet | retired | retired | retired; no CI invocation; current dependency/advisory authority is `cloud-ci-supply-chain-audit` | 0 | retired |
| semver-checks | new-rust | hyperscaler/semver-checks | intelligence-api-semver-kernel | tools/governance-semver-checks | `cargo run -p governance-semver-checks` | 2500 | BLOCKER |
| clippy-pedantic | new-rust | hyperscaler/clippy-pedantic | governance-clippy-pedantic-kernel | tools/governance-clippy-pedantic | `cargo run -p governance-clippy-pedantic` | 3500 | BLOCKER |
| nextest-required | new-rust | hyperscaler/nextest | governance-nextest-required-kernel | tools/governance-nextest-required | `cargo run -p governance-nextest-required` | 1700 | HIGH |
| foundry-corpus-citation | new-directive | MASTERPLAN P3.5 (corpus cite) | governance-foundry-corpus-citation-kernel | tools/governance-foundry-corpus-citation | `cargo run -p governance-foundry-corpus-citation` | 500 | HIGH |
| architecture-map-freshness | new-directive | Directive 11 (visualization) | governance-architecture-map-freshness-kernel | tools/governance-architecture-map-freshness | `cargo run -p governance-architecture-map-freshness` | 350 | MED |
| diataxis-doc-class | new-directive | Directive 10 (class shape) | governance-diataxis-doc-class-kernel | tools/governance-diataxis-doc-class | `cargo run -p governance-diataxis-doc-class` | 700 | MED |
| runbook-freshness | new-directive | Directive 10 (incident link) | governance-runbook-freshness-kernel | tools/governance-runbook-freshness | `cargo run -p governance-runbook-freshness` | 600 | MED |
| evidence-bundle-shape | new-template | TEMPLATE/phase-00-evidence | governance-evidence-bundle-shape-kernel | tools/governance-evidence-bundle-shape | `cargo run -p governance-evidence-bundle-shape` | 350 | BLOCKER |
| pre-flight-checklist | new-checklist | CHECKLIST/pre-flight | governance-pre-flight-checklist-kernel | tools/governance-pre-flight-checklist | `cargo run -p governance-pre-flight-checklist` | 200 | BLOCKER |
| done-definition | new-checklist | CHECKLIST/definition-of-done | governance-done-definition-kernel | tools/governance-done-definition | `cargo run -p governance-done-definition` | 250 | BLOCKER |
| pr-shape-strict | new-standard | STANDARD/pr-shape | governance-pr-shape-strict-kernel | tools/governance-pr-shape-strict | `cargo run -p governance-pr-shape-strict` | 150 | BLOCKER |
| agent-completion-checklist | new-checklist | CHECKLIST/agent-completion + ADR-0054 | governance-agent-completion-checklist-kernel | tools/governance-agent-completion-checklist | `cargo run -p governance-agent-completion-checklist` | 600 | BLOCKER |
| scaffold-claim-pattern | new-adr | ADR-0054 | governance-scaffold-claim-pattern-kernel | tools/governance-scaffold-claim-pattern | `cargo run -p governance-scaffold-claim-pattern` | 400 | HIGH |
| cutover-bootstrap-window | new-adr | ADR-0053 | governance-cutover-bootstrap-window-kernel | tools/governance-cutover-bootstrap-window | `cargo run -p governance-cutover-bootstrap-window` | 500 | HIGH |
| workspace-lints | new-rust | hyperscaler/workspace lints | governance-workspace-lints-kernel | tools/governance-workspace-lints | `cargo run -p governance-workspace-lints` | 250 | HIGH |
| forward-reference-resolved | new-standard | STANDARD/wave-gates | governance-forward-reference-resolved-kernel | tools/governance-forward-reference-resolved | `cargo run -p governance-forward-reference-resolved` | 500 | BLOCKER |
| raci-completeness | new-standard | STANDARD/raci-ownership | governance-raci-completeness-kernel | tools/governance-raci-completeness | `cargo run -p governance-raci-completeness` | 200 | HIGH |
| glossary-vocabulary | new-standard | STANDARD/vocabulary-retirement | governance-glossary-vocabulary-kernel | tools/governance-glossary-vocabulary | `cargo run -p governance-glossary-vocabulary` | 500 | HIGH |
| mdbook-publish | new-standard | STANDARD/doc-publish | governance-mdbook-publish-kernel | tools/governance-mdbook-publish | `cargo run -p governance-mdbook-publish` | 1100 | HIGH |
| openapi-contract-binding | new-standard | STANDARD/api-contract-binding | governance-openapi-contract-binding-kernel | tools/governance-openapi-contract-binding | `cargo run -p governance-openapi-contract-binding` | 1200 | BLOCKER |
| changelog-row | new-standard | STANDARD/changelog-discipline | governance-changelog-row-kernel | tools/governance-changelog-row | `cargo run -p governance-changelog-row` | 300 | HIGH |
| adr-index | existing-extension | STANDARD/adr-index | governance-adr-index-kernel | tools/governance-adr-index | `cargo run -p governance-adr-index` | 250 | HIGH |
| cargo-prefix | existing-extension | STANDARD/cargo-prefix | governance-cargo-prefix-kernel | tools/governance-cargo-prefix | `cargo run -p governance-cargo-prefix` | 150 | BLOCKER |
| codeowners-mirror | existing-extension | STANDARD/codeowners-mirror | CodeownersMirrorFitnessReport (kernel contract) | tools/governance-codeowners-mirror | `cargo run -p governance-codeowners-mirror` | 300 | HIGH |
| constitution-cite | existing-extension | STANDARD/constitution-derivation | governance-constitution-cite-kernel | tools/governance-constitution-cite | `cargo run -p governance-constitution-cite` | 350 | HIGH |
| claim-ceiling | existing-extension | STANDARD/claim-ceiling | governance-claim-ceiling-kernel | tools/governance-claim-ceiling | `cargo run -p governance-claim-ceiling` | 300 | HIGH |
| cost-budget | existing-extension | STANDARD/cost-budget | intelligence-cost-budget-kernel | tools/governance-cost-budget | `cargo run -p governance-cost-budget` | 600 | MED |
| cloud-mutation | existing-extension | STANDARD/cloud-mutation | intelligence-cloud-mutation-kernel | tools/governance-cloud-mutation | `cargo run -p governance-cloud-mutation` | 800 | BLOCKER |
| adapter-kernel | existing-extension | STANDARD/adapter-shape | intelligence-adapter-kernel | tools/governance-adapter-kernel | `cargo run -p governance-adapter-kernel` | 800 | BLOCKER |

Total: 64 lanes. See sibling `<lane-id>.md` for kernel sketch, failure modes, and runtime budget.

## CHANGELOG

| date | change |
| --- | --- |
| 2026-05-12 | 64 fitness-lane specs landed in `docs/governance-lanes/`; kernel implementations in Stage 3 |

| 2026-05-16 | Retired archive-orphan lane after ADR-0116 made M01-P18 the canonical VCS substrate; removed its workspace crates, runner, catalog entries, and one-time archive payload. |
