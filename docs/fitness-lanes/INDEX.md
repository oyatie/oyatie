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
| license | existing | STANDARD/repo-license-policy | oya-foundry-fitness-license-kernel | tools/oya-foundry-fitness-license | `cargo run -p oya-foundry-fitness-license` | 200 | BLOCKER |
| data-class | existing | STANDARD/data-class-tagging | oya-foundry-fitness-data-class-kernel | tools/oya-foundry-fitness-data-class | `cargo run -p oya-foundry-fitness-data-class` | 600 | BLOCKER |
| cohesion | existing | STANDARD/cross-axis-cohesion | oya-foundry-cohesion-fitness-kernel | tools/oya-foundry-fitness-cohesion | `cargo run -p oya-foundry-fitness-cohesion` | 300 | BLOCKER |
| glossary | existing | STANDARD/glossary-required-terms | oya-foundry-fitness-glossary-kernel | tools/oya-foundry-fitness-glossary | `cargo run -p oya-foundry-fitness-glossary` | 700 | HIGH |
| adr-citation | existing | STANDARD/adr-citation | oya-foundry-adr-citation-kernel | tools/oya-foundry-fitness-adr-citation | `cargo run -p oya-foundry-fitness-adr-citation` | 800 | BLOCKER |
| brand-residue | existing | STANDARD/brand-hygiene | oya-foundry-brand-residue-kernel | tools/oya-foundry-fitness-brand-residue | `cargo run -p oya-foundry-fitness-brand-residue` | 900 | HIGH |
| bypass | existing | STANDARD/no-silent-bypass | oya-foundry-bypass-kernel | tools/oya-foundry-fitness-bypass | `cargo run -p oya-foundry-fitness-bypass` | 700 | BLOCKER |
| flat-crates | existing | STANDARD/flat-workspace | oya-foundry-fitness-flat-crates-kernel | tools/oya-foundry-fitness-flat-crates | `cargo run -p oya-foundry-fitness-flat-crates` | 100 | BLOCKER |
| runbook-index-resolves | existing | STANDARD/runbook-index | oya-foundry-fitness-runbook-index-kernel | tools/oya-foundry-fitness-runbook-index-resolves | `cargo run -p oya-foundry-fitness-runbook-index-resolves` | 250 | HIGH |
| doc-catalog | existing | STANDARD/doc-catalog | oya-foundry-catalog-kernel | tools/oya-foundry-fitness-doc-catalog | `cargo run -p oya-foundry-fitness-doc-catalog` | 400 | BLOCKER |
| authority-cohesion | existing | STANDARD/single-source-authority | oya-foundry-authority-cohesion-kernel | tools/oya-foundry-fitness-authority-cohesion | `cargo run -p oya-foundry-fitness-authority-cohesion` | 350 | BLOCKER |
| mistakes-ledger-cite | existing | STANDARD/mistakes-ledger | oya-foundry-fitness-mistakes-ledger-kernel | tools/oya-foundry-fitness-mistakes-ledger-cite | `cargo run -p oya-foundry-fitness-mistakes-ledger-cite` | 600 | HIGH |
| adr-shape | existing | TEMPLATE/adr-template | oya-foundry-fitness-adr-shape-kernel | tools/oya-foundry-fitness-adr-shape | `cargo run -p oya-foundry-fitness-adr-shape` | 250 | BLOCKER |
| audit-emission | existing | STANDARD/audit-chain | oya-foundry-fitness-audit-emission-kernel | tools/oya-foundry-fitness-audit-emission | `cargo run -p oya-foundry-fitness-audit-emission` | 700 | BLOCKER |
| schema-migration | existing | STANDARD/schema-migration | oya-foundry-fitness-schema-migration-kernel | tools/oya-foundry-fitness-schema-migration | `cargo run -p oya-foundry-fitness-schema-migration` | 200 | BLOCKER |
| perf-evidence | existing | STANDARD/perf-evidence | oya-foundry-fitness-perf-evidence-kernel | tools/oya-foundry-fitness-perf-evidence | `cargo run -p oya-foundry-fitness-perf-evidence` | 300 | HIGH |
| traceability-validator | existing | STANDARD/traceability-chain | oya-foundry-fitness-traceability-kernel | tools/oya-foundry-fitness-traceability-validator | `cargo run -p oya-foundry-fitness-traceability-validator` | 1500 | BLOCKER |
| redirect-thinness | existing | STANDARD/redirect-shape | oya-foundry-fitness-redirect-thinness-kernel | tools/oya-foundry-fitness-redirect-thinness | `cargo run -p oya-foundry-fitness-redirect-thinness` | 200 | MED |
| cross-axis-notify | existing | STANDARD/cross-axis-notify | oya-foundry-fitness-cross-axis-notify-kernel | tools/oya-foundry-fitness-cross-axis-notify | `cargo run -p oya-foundry-fitness-cross-axis-notify` | 250 | HIGH |
| capability-publish | existing | STANDARD/capability-map | oya-foundry-capability-kernel | tools/oya-foundry-fitness-capability-publish | `cargo run -p oya-foundry-fitness-capability-publish` | 400 | HIGH |
| portfolio-citation | new-directive | Directive A1 (bidirectional bominal<->oyatie PRD cite) | oya-foundry-fitness-portfolio-citation-kernel | tools/oya-foundry-fitness-portfolio-citation | `cargo run -p oya-foundry-fitness-portfolio-citation` | 800 | HIGH |
| banned-primitives | new-directive | Directive 12 (sanctioned primitives) | oya-foundry-fitness-banned-primitives-kernel | tools/oya-foundry-fitness-banned-primitives | `cargo run -p oya-foundry-fitness-banned-primitives` | 500 | BLOCKER |
| archive-orphan | retired | ADR-0118 / ADR-0116 / M-CC-P11 replacement | retired | retired | retired; no CI invocation | 0 | retired |
| authoritative-tracked | new-directive | Directive A8 | oya-foundry-fitness-authoritative-tracked-kernel | tools/oya-foundry-fitness-authoritative-tracked | `cargo run -p oya-foundry-fitness-authoritative-tracked` | 400 | BLOCKER |
| agentic-navigability | new-directive | Directive 10 (navigability) | oya-foundry-fitness-agentic-navigability-kernel | tools/oya-foundry-fitness-agentic-navigability | `cargo run -p oya-foundry-fitness-agentic-navigability` | 600 | HIGH |
| orphan-detection | new-directive | Directive 10 (purpose) | oya-foundry-fitness-orphan-detection-kernel | tools/oya-foundry-fitness-orphan-detection | `cargo run -p oya-foundry-fitness-orphan-detection` | 1100 | HIGH |
| doc-freshness | new-directive | Directive 10 (staleness) | oya-foundry-fitness-doc-freshness-kernel | tools/oya-foundry-fitness-doc-freshness | `cargo run -p oya-foundry-fitness-doc-freshness` | 700 | MED |
| lts-dependency | new-directive | Directive 8 (LTS pin) | oya-foundry-fitness-lts-dependency-kernel | tools/oya-foundry-fitness-lts-dependency | `cargo run -p oya-foundry-fitness-lts-dependency` | 1800 | BLOCKER |
| provider-agnostic | new-directive | Directive 4 (provider-agnostic) | oya-foundry-fitness-provider-agnostic-kernel | tools/oya-foundry-fitness-provider-agnostic | `cargo run -p oya-foundry-fitness-provider-agnostic` | 900 | BLOCKER |
| image-size-budget | new-directive | Directive 5 (image budget) | oya-foundry-fitness-image-size-budget-kernel | tools/oya-foundry-fitness-image-size-budget | `cargo run -p oya-foundry-fitness-image-size-budget` | 2000 | HIGH |
| sbom-attestation | new-hyperscaler | hyperscaler/SBOM | oya-foundry-fitness-sbom-attestation-kernel | tools/oya-foundry-fitness-sbom-attestation | `cargo run -p oya-foundry-fitness-sbom-attestation` | 1500 | HIGH |
| cosign-signature | new-hyperscaler | hyperscaler/Cosign | oya-foundry-fitness-cosign-signature-kernel | tools/oya-foundry-fitness-cosign-signature | `cargo run -p oya-foundry-fitness-cosign-signature` | 1800 | BLOCKER |
| slsa-provenance | new-hyperscaler | hyperscaler/SLSA L2+ | oya-foundry-fitness-slsa-provenance-kernel | tools/oya-foundry-fitness-slsa-provenance | `cargo run -p oya-foundry-fitness-slsa-provenance` | 1200 | HIGH |
| cargo-vet | new-hyperscaler | hyperscaler/cargo-vet | oya-foundry-fitness-cargo-vet-kernel | tools/oya-foundry-fitness-cargo-vet | `cargo run -p oya-foundry-fitness-cargo-vet` | 1500 | HIGH |
| semver-checks | new-rust | hyperscaler/semver-checks | oya-foundry-api-semver-kernel | tools/oya-foundry-fitness-semver-checks | `cargo run -p oya-foundry-fitness-semver-checks` | 2500 | BLOCKER |
| clippy-pedantic | new-rust | hyperscaler/clippy-pedantic | oya-foundry-fitness-clippy-pedantic-kernel | tools/oya-foundry-fitness-clippy-pedantic | `cargo run -p oya-foundry-fitness-clippy-pedantic` | 3500 | BLOCKER |
| nextest-required | new-rust | hyperscaler/nextest | oya-foundry-fitness-nextest-required-kernel | tools/oya-foundry-fitness-nextest-required | `cargo run -p oya-foundry-fitness-nextest-required` | 1700 | HIGH |
| foundry-corpus-citation | new-directive | MASTERPLAN P3.5 (corpus cite) | oya-foundry-fitness-foundry-corpus-citation-kernel | tools/oya-foundry-fitness-foundry-corpus-citation | `cargo run -p oya-foundry-fitness-foundry-corpus-citation` | 500 | HIGH |
| architecture-map-freshness | new-directive | Directive 11 (visualization) | oya-foundry-fitness-architecture-map-freshness-kernel | tools/oya-foundry-fitness-architecture-map-freshness | `cargo run -p oya-foundry-fitness-architecture-map-freshness` | 350 | MED |
| direct-tool-invocation-audit | new-directive | Directive 12 (icm record) | oya-foundry-fitness-direct-tool-invocation-kernel | tools/oya-foundry-fitness-direct-tool-invocation-audit | `cargo run -p oya-foundry-fitness-direct-tool-invocation-audit` | 800 | HIGH |
| diataxis-doc-class | new-directive | Directive 10 (class shape) | oya-foundry-fitness-diataxis-doc-class-kernel | tools/oya-foundry-fitness-diataxis-doc-class | `cargo run -p oya-foundry-fitness-diataxis-doc-class` | 700 | MED |
| runbook-freshness | new-directive | Directive 10 (incident link) | oya-foundry-fitness-runbook-freshness-kernel | tools/oya-foundry-fitness-runbook-freshness | `cargo run -p oya-foundry-fitness-runbook-freshness` | 600 | MED |
| evidence-bundle-shape | new-template | TEMPLATE/phase-00-evidence | oya-foundry-fitness-evidence-bundle-shape-kernel | tools/oya-foundry-fitness-evidence-bundle-shape | `cargo run -p oya-foundry-fitness-evidence-bundle-shape` | 350 | BLOCKER |
| pre-flight-checklist | new-checklist | CHECKLIST/pre-flight | oya-foundry-fitness-pre-flight-checklist-kernel | tools/oya-foundry-fitness-pre-flight-checklist | `cargo run -p oya-foundry-fitness-pre-flight-checklist` | 200 | BLOCKER |
| done-definition | new-checklist | CHECKLIST/definition-of-done | oya-foundry-fitness-done-definition-kernel | tools/oya-foundry-fitness-done-definition | `cargo run -p oya-foundry-fitness-done-definition` | 250 | BLOCKER |
| pr-shape-strict | new-standard | STANDARD/pr-shape | oya-foundry-fitness-pr-shape-strict-kernel | tools/oya-foundry-fitness-pr-shape-strict | `cargo run -p oya-foundry-fitness-pr-shape-strict` | 150 | BLOCKER |
| agent-completion-checklist | new-checklist | CHECKLIST/agent-completion + ADR-0054 | oya-foundry-fitness-agent-completion-checklist-kernel | tools/oya-foundry-fitness-agent-completion-checklist | `cargo run -p oya-foundry-fitness-agent-completion-checklist` | 600 | BLOCKER |
| scaffold-claim-pattern | new-adr | ADR-0054 | oya-foundry-fitness-scaffold-claim-pattern-kernel | tools/oya-foundry-fitness-scaffold-claim-pattern | `cargo run -p oya-foundry-fitness-scaffold-claim-pattern` | 400 | HIGH |
| cutover-bootstrap-window | new-adr | ADR-0053 | oya-foundry-fitness-cutover-bootstrap-window-kernel | tools/oya-foundry-fitness-cutover-bootstrap-window | `cargo run -p oya-foundry-fitness-cutover-bootstrap-window` | 500 | HIGH |
| workspace-lints | new-rust | hyperscaler/workspace lints | oya-foundry-fitness-workspace-lints-kernel | tools/oya-foundry-fitness-workspace-lints | `cargo run -p oya-foundry-fitness-workspace-lints` | 250 | HIGH |
| forward-reference-resolved | new-standard | STANDARD/wave-gates | oya-foundry-fitness-forward-reference-resolved-kernel | tools/oya-foundry-fitness-forward-reference-resolved | `cargo run -p oya-foundry-fitness-forward-reference-resolved` | 500 | BLOCKER |
| raci-completeness | new-standard | STANDARD/raci-ownership | oya-foundry-fitness-raci-completeness-kernel | tools/oya-foundry-fitness-raci-completeness | `cargo run -p oya-foundry-fitness-raci-completeness` | 200 | HIGH |
| glossary-vocabulary | new-standard | STANDARD/vocabulary-retirement | oya-foundry-fitness-glossary-vocabulary-kernel | tools/oya-foundry-fitness-glossary-vocabulary | `cargo run -p oya-foundry-fitness-glossary-vocabulary` | 500 | HIGH |
| mdbook-publish | new-standard | STANDARD/doc-publish | oya-foundry-fitness-mdbook-publish-kernel | tools/oya-foundry-fitness-mdbook-publish | `cargo run -p oya-foundry-fitness-mdbook-publish` | 1100 | HIGH |
| openapi-contract-binding | new-standard | STANDARD/api-contract-binding | oya-foundry-fitness-openapi-contract-binding-kernel | tools/oya-foundry-fitness-openapi-contract-binding | `cargo run -p oya-foundry-fitness-openapi-contract-binding` | 1200 | BLOCKER |
| changelog-row | new-standard | STANDARD/changelog-discipline | oya-foundry-fitness-changelog-row-kernel | tools/oya-foundry-fitness-changelog-row | `cargo run -p oya-foundry-fitness-changelog-row` | 300 | HIGH |
| adr-index | existing-extension | STANDARD/adr-index | oya-foundry-adr-index-kernel | tools/oya-foundry-fitness-adr-index | `cargo run -p oya-foundry-fitness-adr-index` | 250 | HIGH |
| cargo-prefix | existing-extension | STANDARD/cargo-prefix | oya-foundry-cargo-prefix-kernel | tools/oya-foundry-fitness-cargo-prefix | `cargo run -p oya-foundry-fitness-cargo-prefix` | 150 | BLOCKER |
| codeowners-mirror | existing-extension | STANDARD/codeowners-mirror | oya-foundry-codeowners-mirror-kernel | tools/oya-foundry-fitness-codeowners-mirror | `cargo run -p oya-foundry-fitness-codeowners-mirror` | 300 | HIGH |
| constitution-cite | existing-extension | STANDARD/constitution-derivation | oya-foundry-constitution-cite-kernel | tools/oya-foundry-fitness-constitution-cite | `cargo run -p oya-foundry-fitness-constitution-cite` | 350 | HIGH |
| claim-ceiling | existing-extension | STANDARD/claim-ceiling | oya-foundry-claim-ceiling-kernel | tools/oya-foundry-fitness-claim-ceiling | `cargo run -p oya-foundry-fitness-claim-ceiling` | 300 | HIGH |
| cost-budget | existing-extension | STANDARD/cost-budget | oya-foundry-cost-budget-kernel | tools/oya-foundry-fitness-cost-budget | `cargo run -p oya-foundry-fitness-cost-budget` | 600 | MED |
| cloud-mutation | existing-extension | STANDARD/cloud-mutation | oya-foundry-cloud-mutation-kernel | tools/oya-foundry-fitness-cloud-mutation | `cargo run -p oya-foundry-fitness-cloud-mutation` | 800 | BLOCKER |
| adapter-kernel | existing-extension | STANDARD/adapter-shape | oya-foundry-adapter-kernel | tools/oya-foundry-fitness-adapter-kernel | `cargo run -p oya-foundry-fitness-adapter-kernel` | 800 | BLOCKER |

Total: 64 lanes. See sibling `<lane-id>.md` for kernel sketch, failure modes, and runtime budget.

## CHANGELOG

| date | change |
| --- | --- |
| 2026-05-12 | 64 fitness-lane specs landed in `docs/fitness-lanes/`; kernel implementations in Stage 3 |

| 2026-05-16 | Retired archive-orphan lane after ADR-0116 made M-CC-P11 the canonical VCS substrate; removed its workspace crates, runner, catalog entries, and one-time archive payload. |
