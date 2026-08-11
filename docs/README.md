---
doc_class: Index
shape: canonical-hub
length_cap: 2200
authority_tier: 0
purpose: Canonical entry point for all project documentation; substantive navigation hub for agents, engineers, and stakeholders.
doc_status: published
last_gardened_at: 2026-05-20
gardened_by: codex-docs-hub-gardening
substance_bar: ">=1500 lines; intern-buildability per docs/standards/documentation-rigor.md section 1.1"
authority_chain_declaration: |
  system / developer / user instructions
    > /specs/root-hub-pointers.json
    > docs/AGENTS.md (operating contract until explicit /specs/agent-operating-contract.json PHASE-5 promotion evidence)
    > installed agent-runtime skill and role catalog (for Codex: ~/.codex/skills + ~/.codex/agents; project .codex overlays only when intentionally checked in)
    > machine-readable specs and registries under /specs, /registry, /evidence, and /templates
    > docs/ authority files during markdown-retirement compatibility
    > optional local multi-model delivery kit under .grok/ (mm-delivery; not merge authority; ADR-0515 owns oya-ci-required)
    > installed agent-runtime skill docs and external/upstream skill documentation (informational only; not vendored into this repo)
    > repo-root Redirect-class files (non-authoritative; lane-thin)
    > working drafts and retired harness brand surfaces (.omc/.omx/.gjc residual, claude-code-harness tombstone; never authoritative)
excludes:
  - path: docs/AGENTS.md
    reason: Agent operating contract; this hub points to it but does not duplicate it.
  - path: /specs/root-hub-pointers.json
    reason: Machine-readable entry-point registry; this hub is the human-readable navigation projection.
  - path: docs/standards/documentation-rigor.md
    reason: Rigor standard; this hub follows it and points readers there for the full rule.
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Oyatie Docs Hub

This file is the canonical human-readable entry point for `/Users/jasonlee/oyatie/docs`. The repo-root `README.md` is intentionally thin; agents, engineers, auditors, and stakeholders use this hub to reach the substantive source of truth without relying on memory, Slack, or guesses.

The navigation rule is simple: start from the authority chain, choose the section that matches the question, read the top canonical docs first, then follow the extended library only as needed. This hub favors resolving links over summarizing everything inline; the goal is six-hops reachability with enough substance that a new contributor knows which corpus shelf owns each answer.

Cross-link policy for this hub: every Markdown link in this file must resolve to an existing file, directory, or local section. If a requested target does not exist, this hub names it in code text instead of creating a broken link. During this gardening pass, `microservices/intelligence/spec/` was requested but does not exist; use the **intelligence** capability surfaces + `specs/microservices/foundry.json` retirement tombstone (Foundry product shelf deleted; do not cite Foundry as live authority).

<a id="top-navigation"></a>
**Top Navigation**

1. [§0 Operating Map](#section-0-operating-map)
2. [§1 Architecture](#section-1-architecture)
3. [§2 Decisions](#section-2-decisions)
4. [§3 Products](#section-3-products)
5. [§4 User Journeys](#section-4-user-journeys)
6. [§5 Personas](#section-5-personas)
7. [§6 Standards](#section-6-standards)
8. [§7 Onboarding](#section-7-onboarding)
9. [§8 Governance Pipeline](#section-8-governance-pipeline)
10. [§9 Compliance Packs](#section-9-compliance-packs)
11. [§10 Capability Tiers](#section-10-capability-tiers)
12. [§11 Governance Crates](#section-11-governance-crates)
13. [§12 Glossary](#section-12-glossary)
14. [§13 Wave Sequence](#section-13-wave-sequence)

<a id="section-0-operating-map"></a>
## §0 Operating Map

### Section Purpose

This section tells a cold reader how to use the corpus without stepping around the authority chain. It is the orientation shelf for repo-root redirects, agent operating rules, machine-readable pointers, and the documentation rigor standard that makes this hub more than a link dump.

### Canonical Docs (Top 10)

1. [root-hub-pointers.json](../specs/root-hub-pointers.json) - machine-readable pointer registry that tells agents where authority starts.
2. [AGENTS.md](AGENTS.md) - operating contract for humans and agents before any repo change.
3. [docs/README.md](README.md) - this canonical human-readable navigation hub for the docs corpus.
4. [repo README.md](../README.md) - thin repo-root redirect that points humans toward docs and specs.
5. [documentation-rigor.md](standards/documentation-rigor.md) - intern-buildability and hyperscaler-grade documentation bar.
6. [DOC-CATALOG.md](DOC-CATALOG.md) - lifecycle and ownership protocol for canonical documents.
7. [ADR-INDEX.md](ADR-INDEX.md) - generated decision index and ADR freshness pointer.
8. [GLOSSARY.md](GLOSSARY.md) - canonical vocabulary and retired-term guardrail.
9. [masterplan.json](../specs/masterplan.json) - machine-readable master plan authority for sequencing and readiness claims.
10. [markdown-retirement-policy.json](../specs/markdown-retirement-policy.json) - machine-readable policy for which Markdown survives and why.

### Related Sections

Related sections: [§1 Architecture](#section-1-architecture), [§2 Decisions](#section-2-decisions), [§6 Standards](#section-6-standards), [§13 Wave Sequence](#section-13-wave-sequence).

### When To Read

- Read this first when you enter the repository cold.
- Read this when a task asks for canonical truth instead of a stale summary.
- Read this when deciding whether a doc is authority, compatibility projection, machine-readable source, or working draft.
- Read this before editing any docs hub, catalog, or agent-facing control surface.

### Root-Level Docs And Hub Files

1. [ADR-CONSOLIDATION-PLAN.md](ADR-CONSOLIDATION-PLAN.md) - ADR Consolidation Plan: canonical navigation target for this shelf.
2. [ADR-INDEX.md](ADR-INDEX.md) - ADR Index: canonical navigation target for this shelf.
3. [ADR-LEGACY-REGRESSION-MAPPING.md](ADR-LEGACY-REGRESSION-MAPPING.md) - ADR Legacy Regression Mapping: canonical navigation target for this shelf.
4. [AGENT-INSTRUCTION-SOURCES.md](AGENT-INSTRUCTION-SOURCES.md) - Agent Instruction Sources: canonical navigation target for this shelf.
5. [AGENTS.md](AGENTS.md) - Agents: canonical navigation target for this shelf.
6. [AGENTS-OPERATING-CONTRACT.md](AGENTS-OPERATING-CONTRACT.md) - Agents Operating Contract Doctrine References: canonical navigation target for this shelf.
7. [CHANGELOG.md](CHANGELOG.md) - Changelog: canonical navigation target for this shelf.
8. [COMPETITIVE-GAP-ANALYSIS.md](COMPETITIVE-GAP-ANALYSIS.md) - Competitive Gap Analysis: canonical navigation target for this shelf.
9. [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md) - Compliance Matrix: canonical navigation target for this shelf.
10. [CONTRADICTION-LEDGER.md](CONTRADICTION-LEDGER.md) - Contradiction Ledger: canonical navigation target for this shelf.
11. [DESIGN.md](DESIGN.md) - Design: canonical navigation target for this shelf.
12. [DOC-CATALOG.md](DOC-CATALOG.md) - Doc Catalog: canonical navigation target for this shelf.
13. [DOC-COVERAGE.md](DOC-COVERAGE.md) - Doc Coverage: canonical navigation target for this shelf.
14. [DOC-UPDATE-PROTOCOL.md](DOC-UPDATE-PROTOCOL.md) - Doc Update Protocol: canonical navigation target for this shelf.
15. [DOCUMENTATION.md](DOCUMENTATION.md) - Documentation: canonical navigation target for this shelf.
16. [FINOPS-PLAN.md](FINOPS-PLAN.md) - Finops Plan: canonical navigation target for this shelf.
17. [GLOSSARY.md](GLOSSARY.md) - Glossary: canonical navigation target for this shelf.
18. [GTM-PLAN.md](GTM-PLAN.md) - Gtm Plan: canonical navigation target for this shelf.
19. [HIRING-CAPACITY-PLAN.md](HIRING-CAPACITY-PLAN.md) - Hiring Capacity Plan: canonical navigation target for this shelf.
20. [INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md) - Incident Management: canonical navigation target for this shelf.
21. [INTERNATIONALIZATION.md](INTERNATIONALIZATION.md) - Internationalization: canonical navigation target for this shelf.
22. [LEGAL-IP-LEDGER.md](LEGAL-IP-LEDGER.md) - Legal Ip Ledger: canonical navigation target for this shelf.
23. [MASTERPLAN.md](MASTERPLAN.md) - Masterplan: canonical navigation target for this shelf.
24. [MISTAKES-LEDGER.md](MISTAKES-LEDGER.md) - Mistakes Ledger: canonical navigation target for this shelf.
25. [PRD-OYATIE-FROM-SCRATCH-CANONICAL.md](PRD-OYATIE-FROM-SCRATCH-CANONICAL.md) - Oyatie From-Scratch Canonical PRD: canonical navigation target for this shelf.
26. [PRD.md](PRD.md) - PRD: canonical navigation target for this shelf.
27. [PRIVACY-PROGRAM.md](PRIVACY-PROGRAM.md) - Privacy Program: canonical navigation target for this shelf.
28. [QA-TEST-STRATEGY.md](QA-TEST-STRATEGY.md) - Qa Test Strategy: canonical navigation target for this shelf.
29. [RACI-OWNERSHIP.md](RACI-OWNERSHIP.md) - Raci Ownership: canonical navigation target for this shelf.
30. [README.md](README.md) - Docs: canonical navigation target for this shelf.
31. [RELEASE-MANAGEMENT.md](RELEASE-MANAGEMENT.md) - Release Management: canonical navigation target for this shelf.
32. [RISK-REGISTER.md](RISK-REGISTER.md) - Risk Register: canonical navigation target for this shelf.
33. [ROADMAP.md](ROADMAP.md) - Roadmap: canonical navigation target for this shelf.
34. [RUNBOOKS-INDEX.md](RUNBOOKS-INDEX.md) - Runbooks Index: canonical navigation target for this shelf.
35. [security-program/security-program.json](security-program/security-program.json) - Security Program: canonical navigation target for this shelf.
36. [SLO-CATALOG.md](SLO-CATALOG.md) - SLO Catalog: canonical navigation target for this shelf.
37. [SPEC.md](SPEC.md) - Spec: canonical navigation target for this shelf.
38. [STANDARDS-AND-TEMPLATES.md](STANDARDS-AND-TEMPLATES.md) - Standards And Templates: canonical navigation target for this shelf.
39. [TOOLCHAIN.md](TOOLCHAIN.md) - Toolchain: canonical navigation target for this shelf.
40. [VENDOR-PARTNER-LEDGER.md](VENDOR-PARTNER-LEDGER.md) - Vendor Partner Ledger: canonical navigation target for this shelf.
41. [bootstrap.md](bootstrap.md) - Bootstrap: canonical navigation target for this shelf.

### Machine-Readable Root Specs Shelf

1. [active-machine-readable-artifact-contract.json](../specs/active-machine-readable-artifact-contract.json) - Active Machine Readable Artifact Contract: machine-readable authority or registry contract consumed by gates and agents.
2. [agent-durable-goal.json](../specs/agent-durable-goal.json) - Agent Durable Goal: machine-readable authority or registry contract consumed by gates and agents.
3. [agentic-slo-gated-promotion.json](../specs/agentic-slo-gated-promotion.json) - Agentic SLO Gated Promotion: machine-readable authority or registry contract consumed by gates and agents.
4. [api-surface-separation.json](../specs/api-surface-separation.json) - API Surface Separation: machine-readable authority or registry contract consumed by gates and agents.
5. [artifact-profile-defaults.json](../specs/artifact-profile-defaults.json) - Artifact Profile Defaults: machine-readable authority or registry contract consumed by gates and agents.
6. [brownout-degradation-signal.json](../specs/brownout-degradation-signal.json) - Brownout Degradation Signal: machine-readable authority or registry contract consumed by gates and agents.
7. [capability-tier-schema.json](../specs/capability-tier-schema.json) - Capability Tier Schema: machine-readable authority or registry contract consumed by gates and agents.
8. [cedar-fragment-schema.json](../specs/cedar-fragment-schema.json) - Cedar Fragment Schema: machine-readable authority or registry contract consumed by gates and agents.
9. [chaos-engineering-substrate-canonical.json](../specs/chaos-engineering-substrate-canonical.json) - Chaos Engineering Substrate Canonical: machine-readable authority or registry contract consumed by gates and agents.
10. [ci-fix-loop-context-bundle.json](../specs/ci-fix-loop-context-bundle.json) - Ci Fix Loop Context Bundle: machine-readable authority or registry contract consumed by gates and agents.
11. [codeview-read-surface.json](../specs/codeview-read-surface.json) - Codeview Read Surface: machine-readable authority or registry contract consumed by gates and agents.
12. [compliance-pack-schema.json](../specs/compliance-pack-schema.json) - Compliance Pack Schema: machine-readable authority or registry contract consumed by gates and agents.
13. [crate-naming-audit.json](../specs/crate-naming-audit.json) - Crate Naming Audit: machine-readable authority or registry contract consumed by gates and agents.
14. [csi-storage-class-canonical.json](../specs/csi-storage-class-canonical.json) - Csi Storage Class Canonical: machine-readable authority or registry contract consumed by gates and agents.
15. [decision-principles.json](../specs/decision-principles.json) - Decision Principles: machine-readable authority or registry contract consumed by gates and agents.
16. [decision-rights.json](../specs/decision-rights.json) - Decision Rights: machine-readable authority or registry contract consumed by gates and agents.
17. [deployment-ops-contract.json](../specs/deployment-ops-contract.json) - Deployment Ops Contract: machine-readable authority or registry contract consumed by gates and agents.
18. [design-spec-maturity-claims.json](../specs/design-spec-maturity-claims.json) - Design Spec Maturity Claims: machine-readable authority or registry contract consumed by gates and agents.
19. [dr-business-continuity.json](../specs/dr-business-continuity.json) - Dr Business Continuity: machine-readable authority or registry contract consumed by gates and agents.
20. [evidence-taxonomy.json](../specs/evidence-taxonomy.json) - Evidence Taxonomy: machine-readable authority or registry contract consumed by gates and agents.
21. [feature-flag-substrate-canonical.json](../specs/feature-flag-substrate-canonical.json) - Feature Flag Substrate Canonical: machine-readable authority or registry contract consumed by gates and agents.
22. [final-report-schema.json](../specs/final-report-schema.json) - Final Report Schema: machine-readable authority or registry contract consumed by gates and agents.
23. [finops-cost-attribution.json](../specs/finops-cost-attribution.json) - Finops Cost Attribution: machine-readable authority or registry contract consumed by gates and agents.
24. [forbidden-operations.json](../specs/forbidden-operations.json) - Forbidden Operations: machine-readable authority or registry contract consumed by gates and agents.
25. [gitops-vcs-replacement.json](../specs/gitops-vcs-replacement.json) - Gitops Vcs Replacement: machine-readable authority or registry contract consumed by gates and agents.
26. [governance-amendment.json](../specs/governance-amendment.json) - Governance Amendment: machine-readable authority or registry contract consumed by gates and agents.
27. [hyperscaler-architecture-invariants.json](../specs/hyperscaler-architecture-invariants.json) - Hyperscaler Architecture Invariants: machine-readable authority or registry contract consumed by gates and agents.
28. [hyperscaler-gates.json](../specs/hyperscaler-gates.json) - Hyperscaler Gates: machine-readable authority or registry contract consumed by gates and agents.
29. [industry-best-practice-conformance.json](../specs/industry-best-practice-conformance.json) - Industry Best Practice Conformance: machine-readable authority or registry contract consumed by gates and agents.
30. [iterative-fix-loop.json](../specs/iterative-fix-loop.json) - Iterative Fix Loop: machine-readable authority or registry contract consumed by gates and agents.
31. [knowledge-graph-schema.json](../specs/knowledge-graph-schema.json) - Knowledge Graph Schema: machine-readable authority or registry contract consumed by gates and agents.
32. [markdown-retirement-policy.json](../specs/markdown-retirement-policy.json) - Markdown Retirement Policy: machine-readable authority or registry contract consumed by gates and agents.
33. [master-plan-sequencing.json](../specs/master-plan-sequencing.json) - Master Plan Sequencing: machine-readable authority or registry contract consumed by gates and agents.
34. [masterplan.json](../specs/masterplan.json) - Masterplan: machine-readable authority or registry contract consumed by gates and agents.
35. [merge-queue-parked-pr.json](../specs/merge-queue-parked-pr.json) - Merge Queue Parked Pr: machine-readable authority or registry contract consumed by gates and agents.
36. [microservice-migration-tooling.json](../specs/microservice-migration-tooling.json) - Microservice Migration Tooling: machine-readable authority or registry contract consumed by gates and agents.
37. [multi-region-disposition-canonical.json](../specs/multi-region-disposition-canonical.json) - Multi Region Disposition Canonical: machine-readable authority or registry contract consumed by gates and agents.
38. `multispectrum-review.json` — Retired with the external coordination / bespoke-admission adapter path; preserve multi-lens review through independent reviewer-agent passes, PR Code Review evidence, typed quality-gate artifacts, and cloud-ci/oya-ci gate packets.
39. [oyatie-doctrine.json](../specs/oyatie-doctrine.json) - Oyatie Doctrine: machine-readable authority or registry contract consumed by gates and agents.
40. [per-microservice-flat-layout.json](../specs/per-microservice-flat-layout.json) - Per Microservice Flat Layout: machine-readable authority or registry contract consumed by gates and agents.
41. [per-tenant-audit-log-slicing-canonical.json](../specs/per-tenant-audit-log-slicing-canonical.json) - Per Tenant Audit Log Slicing Canonical: machine-readable authority or registry contract consumed by gates and agents.
42. [plan-schema.json](../specs/plan-schema.json) - Plan Schema: machine-readable authority or registry contract consumed by gates and agents.
43. [planning-closure-contract.json](../specs/planning-closure-contract.json) - Planning Closure Contract: machine-readable authority or registry contract consumed by gates and agents.
44. [planning-closure-status-closure-ledger.json](../specs/planning-closure-status-closure-ledger.json) - Planning Closure Status Closure Ledger: machine-readable authority or registry contract consumed by gates and agents.
45. [platform-architecture.json](../specs/platform-architecture.json) - Platform Architecture: machine-readable authority or registry contract consumed by gates and agents.
46. [root-hub-pointers.json](../specs/root-hub-pointers.json) - Root Hub Pointers: machine-readable authority or registry contract consumed by gates and agents.
47. [saga-shape.json](../specs/saga-shape.json) - Saga Shape: machine-readable authority or registry contract consumed by gates and agents.
48. [schema-registry-canonical.json](../specs/schema-registry-canonical.json) - Schema Registry Canonical: machine-readable authority or registry contract consumed by gates and agents.
49. [score-cards.json](../specs/score-cards.json) - Score Cards: machine-readable authority or registry contract consumed by gates and agents.
50. [sovereign-cloud-air-gapped-canonical.json](../specs/sovereign-cloud-air-gapped-canonical.json) - Sovereign Cloud Air Gapped Canonical: machine-readable authority or registry contract consumed by gates and agents.
51. [sovereign-cloud-overlays.json](../specs/sovereign-cloud-overlays.json) - Sovereign Cloud Overlays: machine-readable authority or registry contract consumed by gates and agents.
52. [stop-conditions.json](../specs/stop-conditions.json) - Stop Conditions: machine-readable authority or registry contract consumed by gates and agents.
53. [tenant-environment-tiers-canonical.json](../specs/tenant-environment-tiers-canonical.json) - Tenant Environment Tiers Canonical: machine-readable authority or registry contract consumed by gates and agents.
54. [tenant-lifecycle.json](../specs/tenant-lifecycle.json) - Tenant Lifecycle: machine-readable authority or registry contract consumed by gates and agents.
55. [tenant-model.json](../specs/tenant-model.json) - Tenant Model: machine-readable authority or registry contract consumed by gates and agents.
56. [test-standard.json](../specs/test-standard.json) - Test Standard: machine-readable authority or registry contract consumed by gates and agents.
57. [throttling-tiers.json](../specs/throttling-tiers.json) - Throttling Tiers: machine-readable authority or registry contract consumed by gates and agents.
58. [workspace-hygiene.json](../specs/workspace-hygiene.json) - Workspace Hygiene: machine-readable authority or registry contract consumed by gates and agents.

<a id="section-1-architecture"></a>
## §1 Architecture

### Section Purpose

Architecture owns the unified ecosystem thesis, the cross-axis shape, the audit and coverage evidence, and the diagrams that make system behavior navigable. New agents use this section to understand what Oyatie is becoming before they inspect product PRDs or implementation plans.

### Canonical Docs (Top 15)

1. [unified ecosystem thesis](architecture/unified-ecosystem-thesis-2026-05-21.md) - load-bearing thesis for Oyatie as one ecosystem rather than separate products.
2. [day in the life](architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md) - day-in-the-life walkthrough showing cross-surface coherence.
3. [training cost doctrine](architecture/training-cost-doctrine-2026-05-21.md) - doctrine for reducing training burden across unified surfaces.
4. [enterprise coverage matrix](architecture/enterprise-software-coverage-matrix-2026-05-21.md) - coverage map for enterprise software domains and gaps.
5. [keystone bundle synthesis](architecture/keystone-bundle-2026-05-20-synthesis.md) - synthesis for the keystone ADR bundle and its implementation force.
6. [keystone lessons learned](architecture/keystone-bundle-2026-05-20-lessons-learned.md) - lessons from the keystone bundle pass.
7. [wave 3-g synthesis adjudication](architecture/wave-3-g-synthesis-adjudication-2026-05-21.md) - adjudication of Wave 3-G synthesis outputs.
8. [cross-coverage matrix](architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md) - cross-coverage matrix connecting personas, journeys, and microservices.
9. [executive briefing](architecture/wave-3-g-executive-briefing-2026-05-21.md) - executive-level readout for Wave 3-G architecture work.
10. [post-wave-3-g rigor audit](architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md) - latest architecture corpus rigor audit snapshot.
11. [six-hops audit](architecture/six-hops-reachability-audit-2026-05-20.md) - reachability audit proving hub navigation obligations.
12. [ADR corpus line audit](architecture/adr-corpus-line-audit-2026-05-21.md) - line-count and substance audit of ADR corpus depth.
13. [IP corpus line audit](architecture/ip-corpus-line-audit-2026-05-21.md) - line-count and substance audit of implementation-plan corpus depth.
14. [inter-microservice call graph](architecture/diagrams/inter-microservice-call-graph.md) - diagram entry for inter-microservice call flow.
15. [tenant lifecycle diagram](architecture/diagrams/tenant-lifecycle-state-machine.md) - diagram entry for tenant lifecycle states.

### Related Sections

Related sections: [§2 Decisions](#section-2-decisions), [§3 Products](#section-3-products), [§4 User Journeys](#section-4-user-journeys), [§6 Standards](#section-6-standards), [§13 Wave Sequence](#section-13-wave-sequence).

### When To Read

- Read this when a question uses words like ecosystem, architecture, coverage, hyperscaler, thesis, or six hops.
- Read this before changing cross-axis contracts, product boundaries, or user-journey coverage.
- Read this when an ADR or PRD feels isolated and you need the whole-system context.
- Read the diagrams before explaining flows to a stakeholder or implementing a cross-service integration.

### Architecture Extended Library

1. [adr-corpus-line-audit-2026-05-21.md](architecture/adr-corpus-line-audit-2026-05-21.md) - ADR Corpus Line Audit 2026 05 21: canonical navigation target for this shelf.
2. [audit-event-coverage-sweep-2026-05-20.md](architecture/audit-event-coverage-sweep-2026-05-20.md) - Audit Event Coverage Sweep 2026 05 20: canonical navigation target for this shelf.
3. [corpus-rigor-audit-2026-05-20-mid-remediation-snapshot.md](architecture/corpus-rigor-audit-2026-05-20-mid-remediation-snapshot.md) - Corpus Rigor Audit 2026 05 20 Mid Remediation Snapshot: canonical navigation target for this shelf.
4. [corpus-rigor-audit-2026-05-20.md](architecture/corpus-rigor-audit-2026-05-20.md) - Corpus Rigor Audit 2026 05 20: canonical navigation target for this shelf.
5. [corpus-rigor-audit-2026-05-21-post-wave-3-g.md](architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md) - Corpus Rigor Audit 2026 05 21 Post Wave 3 G: canonical navigation target for this shelf.
6. [day-in-the-life-coherent-ecosystem-2026-05-21.md](architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md) - Day In The Life Coherent Ecosystem 2026 05 21: canonical navigation target for this shelf.
7. [diagrams/ai-substrate-two-layer-architecture.md](architecture/diagrams/ai-substrate-two-layer-architecture.md) - AI Substrate Two Layer Architecture: canonical navigation target for this shelf.
8. [diagrams/audit-chain-emission-pipeline.md](architecture/diagrams/audit-chain-emission-pipeline.md) - Audit Chain Emission Pipeline: canonical navigation target for this shelf.
9. [diagrams/capability-tier-projection-flow.md](architecture/diagrams/capability-tier-projection-flow.md) - Capability Tier Projection Flow: canonical navigation target for this shelf.
10. [diagrams/cedar-policy-evaluation-flow.md](architecture/diagrams/cedar-policy-evaluation-flow.md) - Cedar Policy Evaluation Flow: canonical navigation target for this shelf.
11. [diagrams/cell-routing-shuffle-sharding.md](architecture/diagrams/cell-routing-shuffle-sharding.md) - Cell Routing Shuffle Sharding: canonical navigation target for this shelf.
12. [diagrams/compliance-pack-overlay-precedence.md](architecture/diagrams/compliance-pack-overlay-precedence.md) - Compliance Pack Overlay Precedence: canonical navigation target for this shelf.
13. [diagrams/dual-tenant-identity-boundary.md](architecture/diagrams/dual-tenant-identity-boundary.md) - Dual Tenant Identity Boundary: canonical navigation target for this shelf.
14. [diagrams/inter-microservice-call-graph.md](architecture/diagrams/inter-microservice-call-graph.md) - Inter Microservice Call Graph: canonical navigation target for this shelf.
15. [diagrams/marketplace-deal-settlement-flow.md](architecture/diagrams/marketplace-deal-settlement-flow.md) - Marketplace Deal Settlement Flow: canonical navigation target for this shelf.
16. [diagrams/tenant-lifecycle-state-machine.md](architecture/diagrams/tenant-lifecycle-state-machine.md) - Tenant Lifecycle State Machine: canonical navigation target for this shelf.
17. [enterprise-software-coverage-matrix-2026-05-21.md](architecture/enterprise-software-coverage-matrix-2026-05-21.md) - Enterprise Software Coverage Matrix 2026 05 21: canonical navigation target for this shelf.
19. [hyperscaler-pattern-attribution.md](architecture/hyperscaler-pattern-attribution.md) - Hyperscaler Pattern Attribution: canonical navigation target for this shelf.
20. [ip-corpus-line-audit-2026-05-21.md](architecture/ip-corpus-line-audit-2026-05-21.md) - Ip Corpus Line Audit 2026 05 21: canonical navigation target for this shelf.
21. [ip-cross-reference-sweep-2026-05-20.md](architecture/ip-cross-reference-sweep-2026-05-20.md) - Ip Cross Reference Sweep 2026 05 20: canonical navigation target for this shelf.
22. [keystone-bundle-2026-05-20-lessons-learned.md](architecture/keystone-bundle-2026-05-20-lessons-learned.md) - Keystone Bundle 2026 05 20 Lessons Learned: canonical navigation target for this shelf.
23. [keystone-bundle-2026-05-20-synthesis.md](architecture/keystone-bundle-2026-05-20-synthesis.md) - Keystone Bundle 2026 05 20 Synthesis: canonical navigation target for this shelf.
24. [keystone-bundle-audit-report.md](architecture/keystone-bundle-audit-report.md) - Keystone Bundle Audit Report: canonical navigation target for this shelf.
25. [keystone-bundle-idea-refine-deep-dive.md](architecture/keystone-bundle-idea-refine-deep-dive.md) - Keystone Bundle Idea Refine Deep Dive: canonical navigation target for this shelf.
26. [keystone-bundle-intern-walkthrough.md](architecture/keystone-bundle-intern-walkthrough.md) - Keystone Bundle Intern Walkthrough: canonical navigation target for this shelf.
27. [keystone-bundle-reading-order.md](architecture/keystone-bundle-reading-order.md) - Keystone Bundle Reading Order: canonical navigation target for this shelf.
28. [memory-spec-runbook-audit-2026-05-21.md](architecture/memory-spec-runbook-audit-2026-05-21.md) - Memory Spec Runbook Audit 2026 05 21: canonical navigation target for this shelf.
29. [microservices-corpus-line-audit-2026-05-21.md](architecture/microservices-corpus-line-audit-2026-05-21.md) - Microservices Corpus Line Audit 2026 05 21: canonical navigation target for this shelf.
30. [persona-journey-microservice-cross-coverage-matrix-2026-05-21.md](architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md) - Persona Journey Microservice Cross Coverage Matrix 2026 05 21: canonical navigation target for this shelf.
31. [product-graph.html](architecture/product-graph.html) - Product Graph: canonical navigation target for this shelf.
32. [product-graph.md](architecture/product-graph.md) - Product Graph: canonical navigation target for this shelf.
33. [six-hops-reachability-audit-2026-05-20.md](architecture/six-hops-reachability-audit-2026-05-20.md) - Six Hops Reachability Audit 2026 05 20: canonical navigation target for this shelf.
34. [standards-corpus-line-audit-2026-05-21.md](architecture/standards-corpus-line-audit-2026-05-21.md) - Standards Corpus Line Audit 2026 05 21: canonical navigation target for this shelf.
35. [training-cost-doctrine-2026-05-21.md](architecture/training-cost-doctrine-2026-05-21.md) - Training Cost Doctrine 2026 05 21: canonical navigation target for this shelf.
36. [transition-classification-2026-05-21.json](architecture/transition-classification-2026-05-21.json) - Transition Classification 2026 05 21: canonical navigation target for this shelf.
37. [unified-ecosystem-thesis-2026-05-21.md](architecture/unified-ecosystem-thesis-2026-05-21.md) - Unified Ecosystem Thesis 2026 05 21: canonical navigation target for this shelf.
38. [wave-3-g-executive-briefing-2026-05-21.md](architecture/wave-3-g-executive-briefing-2026-05-21.md) - Wave 3 G Executive Briefing 2026 05 21: canonical navigation target for this shelf.
39. [wave-3-g-synthesis-adjudication-2026-05-21.md](architecture/wave-3-g-synthesis-adjudication-2026-05-21.md) - Wave 3 G Synthesis Adjudication 2026 05 21: canonical navigation target for this shelf.

<a id="section-2-decisions"></a>
## §2 Decisions

### Section Purpose

Decisions own the why. ADRs explain the invariants behind tenant scoping, audit emission, capability tiers, role projection, B2B coverage, governance pipeline control, and the keystone bundle. Use this section before changing any primitive that has a policy, architecture, or compatibility consequence.

### Canonical Docs (Top 15)

1. [ADR index](ADR-INDEX.md) - generated ADR index and decision freshness source.
2. [decisions README](decisions/README.md) - decision directory entry point and local ADR shelf.
3. [ADR-0105](decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md) - 13-layer enum and check-family pattern foundation.
4. [ADR-0242](decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) - keystone tenant doctrine anchor.
5. [ADR-0244](decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) - tenant scoping doctrine for all primitives.
6. [ADR-0245](decisions/ADR-0245-substrate-vs-product-layering.md) - separates substrate mechanics from product presentation.
7. [ADR-0248](decisions/ADR-0248-amazon-shape-cellular-architecture.md) - cellular architecture precedent and shape.
8. [ADR-0250](decisions/ADR-0250-build-ahead-of-certification-doctrine.md) - build-ahead-of-certification doctrine.
9. [ADR-0251](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) - cell certification levels by compliance pack.
10. [ADR-0258](decisions/ADR-0258-api-versioning-model.md) - API versioning and deprecation doctrine.
11. [ADR-0263](decisions/ADR-0263-observability-emission-contract.md) - audit and observability emission contract.
12. [ADR-0316](decisions/ADR-0316-capability-tier-over-product-fragmentation.md) - capability tiers as the organizing abstraction.
13. [ADR-0317](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) - role projection model for unified UX.
14. [ADR-0321](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) - B2B SaaS industry leader coverage decision.
15. [ADR-0221](decisions/ADR-0221-agentic-development-pipeline-hardening.md) - governance hardening for agentic development pipeline.

### Related Sections

Related sections: [§1 Architecture](#section-1-architecture), [§6 Standards](#section-6-standards), [§8 Governance Pipeline](#section-8-governance-pipeline), [§10 Capability Tiers](#section-10-capability-tiers), [§13 Wave Sequence](#section-13-wave-sequence).

### When To Read

- Read this when code or docs mention an ADR number.
- Read this before changing tenant scope, audit emission, API versioning, capability tiers, or role projection.
- Read this when two docs disagree; the ADR plus machine-readable spec usually explains the intended invariant.
- Read this when preparing a new ADR so numbering, status, and supersession stay coherent.

### Keystone Bundle ADR-0242 Through ADR-0258

1. [ADR-0242-oyatie-is-a-tenant-doctrine.md](decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md) - ADR 0242 Oyatie Is A Tenant Doctrine: decision record; read for why this primitive or policy exists before changing dependent specs.
2. [ADR-0243-cedar-as-universal-gate.md](decisions/ADR-0243-cedar-as-universal-gate.md) - ADR 0243 Cedar As Universal Gate: decision record; read for why this primitive or policy exists before changing dependent specs.
3. [ADR-0244-tenant-as-universal-scoping-primitive.md](decisions/ADR-0244-tenant-as-universal-scoping-primitive.md) - ADR 0244 Tenant As Universal Scoping Primitive: decision record; read for why this primitive or policy exists before changing dependent specs.
4. [ADR-0245-substrate-vs-product-layering.md](decisions/ADR-0245-substrate-vs-product-layering.md) - ADR 0245 Substrate Vs Product Layering: decision record; read for why this primitive or policy exists before changing dependent specs.
5. [ADR-0353-amendment-library-first-network-opt-in-clarification.md](decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md) - ADR 0246 Amendment Library First Network Opt In Clarification: decision record; read for why this primitive or policy exists before changing dependent specs.
6. [ADR-0246-policy-engine-substrate-promotion.md](decisions/ADR-0246-policy-engine-substrate-promotion.md) - ADR 0246 Policy Engine Substrate Promotion: decision record; read for why this primitive or policy exists before changing dependent specs.
7. [ADR-0247-self-hosting-self-modification-doctrine.md](decisions/ADR-0247-self-hosting-self-modification-doctrine.md) - ADR 0247 Self Hosting Self Modification Doctrine: decision record; read for why this primitive or policy exists before changing dependent specs.
8. [ADR-0248-amazon-shape-cellular-architecture.md](decisions/ADR-0248-amazon-shape-cellular-architecture.md) - ADR 0248 Amazon Shape Cellular Architecture: decision record; read for why this primitive or policy exists before changing dependent specs.
9. [ADR-0249-multi-category-marketplace-doctrine.md](decisions/ADR-0249-multi-category-marketplace-doctrine.md) - ADR 0249 Multi Category Marketplace Doctrine: decision record; read for why this primitive or policy exists before changing dependent specs.
10. [ADR-0250-build-ahead-of-certification-doctrine.md](decisions/ADR-0250-build-ahead-of-certification-doctrine.md) - ADR 0250 Build Ahead Of Certification Doctrine: decision record; read for why this primitive or policy exists before changing dependent specs.
11. [ADR-0251-compliance-pack-cell-certification-levels.md](decisions/ADR-0251-compliance-pack-cell-certification-levels.md) - ADR 0251 Compliance Pack Cell Certification Levels: decision record; read for why this primitive or policy exists before changing dependent specs.
12. [ADR-0252-time-coordination-distributed-consistency.md](decisions/ADR-0252-time-coordination-distributed-consistency.md) - ADR 0252 Time Coordination Distributed Consistency: decision record; read for why this primitive or policy exists before changing dependent specs.
13. [ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md](decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md) - ADR 0253 Amendment Http3 Fallback Strict Tls Ech Pqc: decision record; read for why this primitive or policy exists before changing dependent specs.
14. [ADR-0253-network-topology-edge-service-mesh.md](decisions/ADR-0253-network-topology-edge-service-mesh.md) - ADR 0253 Network Topology Edge Service Mesh: decision record; read for why this primitive or policy exists before changing dependent specs.
15. [ADR-0254-deployment-model-spectrum.md](decisions/ADR-0254-deployment-model-spectrum.md) - ADR 0254 Deployment Model Spectrum: decision record; read for why this primitive or policy exists before changing dependent specs.
16. [ADR-0355-amendment-library-first-network-opt-in-clarification.md](decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md) - ADR 0255 Amendment Library First Network Opt In Clarification: decision record; read for why this primitive or policy exists before changing dependent specs.
17. [ADR-0255-intelligence-as-two-layer-ai-substrate.md](decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md) - ADR 0255 Intelligence As Two Layer AI Substrate: decision record; read for why this primitive or policy exists before changing dependent specs.
18. [ADR-0356-amendment-library-first-ontology-read-path.md](decisions/ADR-0356-amendment-library-first-ontology-read-path.md) - ADR 0257 Amendment Library First Ontology Read Path: decision record; read for why this primitive or policy exists before changing dependent specs.
19. [ADR-0257-ontology-object-type-versioning-deprecation-handshake.md](decisions/ADR-0257-ontology-object-type-versioning-deprecation-handshake.md) - ADR 0257 Ontology Object Type Versioning Deprecation Handshake: decision record; read for why this primitive or policy exists before changing dependent specs.
20. [ADR-0258-api-versioning-model.md](decisions/ADR-0258-api-versioning-model.md) - ADR 0258 API Versioning Model: decision record; read for why this primitive or policy exists before changing dependent specs.

### Thirty-ADR Coverage Cluster ADR-0297 Through ADR-0321

1. [ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md](decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md) - ADR 0297 Abuse Defence Baseline Anti Bot Spoof Scrape: decision record; read for why this primitive or policy exists before changing dependent specs.
2. [ADR-0298-emergency-services-bypass-life-safety.md](decisions/ADR-0298-emergency-services-bypass-life-safety.md) - ADR 0298 Emergency Services Bypass Life Safety: decision record; read for why this primitive or policy exists before changing dependent specs.
3. [ADR-0299-account-recovery-resilience.md](decisions/ADR-0299-account-recovery-resilience.md) - ADR 0299 Account Recovery Resilience: decision record; read for why this primitive or policy exists before changing dependent specs.
4. [ADR-0300-whistleblower-press-freedom-anonymity.md](decisions/ADR-0300-whistleblower-press-freedom-anonymity.md) - ADR 0300 Whistleblower Press Freedom Anonymity: decision record; read for why this primitive or policy exists before changing dependent specs.
5. [ADR-0301-survivor-safety-domestic-abuse-mode.md](decisions/ADR-0301-survivor-safety-domestic-abuse-mode.md) - ADR 0301 Survivor Safety Domestic Abuse Mode: decision record; read for why this primitive or policy exists before changing dependent specs.
6. [ADR-0302-deceased-user-inheritance-doctrine.md](decisions/ADR-0302-deceased-user-inheritance-doctrine.md) - ADR 0302 Deceased User Inheritance Doctrine: decision record; read for why this primitive or policy exists before changing dependent specs.
7. [ADR-0303-cognitive-impairment-decision-resilience.md](decisions/ADR-0303-cognitive-impairment-decision-resilience.md) - ADR 0303 Cognitive Impairment Decision Resilience: decision record; read for why this primitive or policy exists before changing dependent specs.
8. [ADR-0304-cross-jurisdiction-conflict-resolution.md](decisions/ADR-0304-cross-jurisdiction-conflict-resolution.md) - ADR 0304 Cross Jurisdiction Conflict Resolution: decision record; read for why this primitive or policy exists before changing dependent specs.
9. [ADR-0305-delegated-agent-authority-chain.md](decisions/ADR-0305-delegated-agent-authority-chain.md) - ADR 0305 Delegated Agent Authority Chain: decision record; read for why this primitive or policy exists before changing dependent specs.
10. [ADR-0306-disaster-mode-cell-resilience.md](decisions/ADR-0306-disaster-mode-cell-resilience.md) - ADR 0306 Disaster Mode Cell Resilience: decision record; read for why this primitive or policy exists before changing dependent specs.
11. [ADR-0307-detection-substrate-streaming-batch.md](decisions/ADR-0307-detection-substrate-streaming-batch.md) - ADR 0307 Detection Substrate Streaming Batch: decision record; read for why this primitive or policy exists before changing dependent specs.
12. [ADR-0308-ml-model-lifecycle-ai-act-compliance.md](decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md) - ADR 0308 Ml Model Lifecycle AI Act Compliance: decision record; read for why this primitive or policy exists before changing dependent specs.
13. [ADR-0309-detection-fairness-audit-civil-rights.md](decisions/ADR-0309-detection-fairness-audit-civil-rights.md) - ADR 0309 Detection Fairness Audit Civil Rights: decision record; read for why this primitive or policy exists before changing dependent specs.
14. [ADR-0310-investigation-case-management.md](decisions/ADR-0310-investigation-case-management.md) - ADR 0310 Investigation Case Management: decision record; read for why this primitive or policy exists before changing dependent specs.
15. [ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md](decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md) - ADR 0311 Dual Tenant Identity Personal Vs Work Boundary: decision record; read for why this primitive or policy exists before changing dependent specs.
16. [ADR-0312-court-warrant-scoped-piercing.md](decisions/ADR-0312-court-warrant-scoped-piercing.md) - ADR 0312 Court Warrant Scoped Piercing: decision record; read for why this primitive or policy exists before changing dependent specs.
17. [ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md](decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md) - ADR 0313 Conglomerate Tenant Hierarchy Sovereign Children: decision record; read for why this primitive or policy exists before changing dependent specs.
18. [ADR-0314-marketplace-as-universal-deal-settlement.md](decisions/ADR-0314-marketplace-as-universal-deal-settlement.md) - ADR 0314 Marketplace As Universal Deal Settlement: decision record; read for why this primitive or policy exists before changing dependent specs.
19. [ADR-0315-erp-coverage-doctrine-sap-parity.md](decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md) - ADR 0315 Erp Coverage Doctrine Sap Parity: decision record; read for why this primitive or policy exists before changing dependent specs.
20. [ADR-0316-capability-tier-over-product-fragmentation.md](decisions/ADR-0316-capability-tier-over-product-fragmentation.md) - ADR 0316 Capability Tier Over Product Fragmentation: decision record; read for why this primitive or policy exists before changing dependent specs.
21. [ADR-0317-role-based-projection-unified-ux-shell.md](decisions/ADR-0317-role-based-projection-unified-ux-shell.md) - ADR 0317 Role Based Projection Unified Ux Shell: decision record; read for why this primitive or policy exists before changing dependent specs.
22. [ADR-0318-collar-color-workspace-universality.md](decisions/ADR-0318-collar-color-workspace-universality.md) - ADR 0318 Collar Color Workspace Universality: decision record; read for why this primitive or policy exists before changing dependent specs.
23. [ADR-0319-front-middle-back-office-information-barrier.md](decisions/ADR-0319-front-middle-back-office-information-barrier.md) - ADR 0319 Front Middle Back Office Information Barrier: decision record; read for why this primitive or policy exists before changing dependent specs.
24. [ADR-0320-apprentice-intern-resident-fellow-transient-identity.md](decisions/ADR-0320-apprentice-intern-resident-fellow-transient-identity.md) - ADR 0320 Apprentice Intern Resident Fellow Transient Identity: decision record; read for why this primitive or policy exists before changing dependent specs.
25. [ADR-0321-b2b-saas-industry-leader-coverage.md](decisions/ADR-0321-b2b-saas-industry-leader-coverage.md) - ADR 0321 B2B Saas Industry Leader Coverage: decision record; read for why this primitive or policy exists before changing dependent specs.

### Governance Pipeline ADR-0110 Through ADR-0116 (historical Foundry brand)

1. [ADR-0110-changeset-state-machine.md](decisions/ADR-0110-changeset-state-machine.md) - ADR 0110 Changeset State Machine: decision record; read for why this primitive or policy exists before changing dependent specs.
2. [ADR-0111-merge-queue-projected-state-fix-at-any-stage.md](decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md) - ADR 0111 Merge Queue Projected State Fix At Any Stage: decision record; read for why this primitive or policy exists before changing dependent specs.
3. [ADR-0112-webhook-driven-intelligence-agent-invocation.md](decisions/ADR-0112-webhook-driven-intelligence-agent-invocation.md) - ADR 0112 Webhook Driven Foundry Agent Invocation: decision record; read for why this primitive or policy exists before changing dependent specs.
4. [ADR-0113-vcs-orchestrator-end-to-end.md](decisions/ADR-0113-vcs-orchestrator-end-to-end.md) - ADR 0113 Vcs Orchestrator End To End: decision record; read for why this primitive or policy exists before changing dependent specs.
5. [ADR-0114-canary-observability-rollback.md](decisions/ADR-0114-canary-observability-rollback.md) - ADR 0114 Canary Observability Rollback: decision record; read for why this primitive or policy exists before changing dependent specs.
6. [ADR-0115-registry-consolidation-flat-singular.md](decisions/ADR-0115-registry-consolidation-flat-singular.md) - ADR 0115 Registry Consolidation Flat Singular: decision record; read for why this primitive or policy exists before changing dependent specs.
7. [ADR-0116-retire-external-agent-coordination-tooling.md](decisions/ADR-0116-retire-external-agent-coordination-tooling.md) - ADR 0116 Retire External Agent Coordination Tooling: decision record; read for why this primitive or policy exists before changing dependent specs.

<a id="section-3-products"></a>
## §3 Products

### Section Purpose

Products translate the ecosystem thesis into concrete PRDs. This shelf separates product-facing PRDs under `docs/products/` from microservice PRDs under `docs/prds/`; the latter currently contains the accepted implementation-driving PRDs for tenancy, ontology, workflow, application, HR, payroll, accounting, communications, and governance pipeline services.

### Canonical Docs (Top 15)

1. [products README](products/README.md) - product PRD index for axis and vertical products; treat absent linked drafts as planned until files exist.
2. [PRD index](prds/INDEX.md) - microservice PRD index for accepted implementation-driving PRDs.
3. [workflow PRD](prds/workflow.md) - Workflow PRD and first hero product substrate.
4. [ontology PRD](prds/ontology.md) - Ontology PRD and graph substrate.
5. [tenancy PRD](prds/tenancy.md) - Tenancy PRD and tenant membership substrate.
6. [application PRD](prds/application.md) - B2B application shell PRD.
7. [HR PRD](prds/hr.md) - HR PRD for enterprise M03 scope.
8. [payroll PRD](prds/payroll.md) - Payroll PRD for enterprise M03 scope.
9. [accounting PRD](prds/accounting.md) - Accounting PRD for enterprise M03 scope.
10. [communications PRD](prds/communications.md) - PRD for Messenger, Mail, and Community communication surfaces.
11. [cloud product PRD](products/cloud/PRD.md) - Cloud product PRD surface.
14. [ERP coverage PRD](products/erp-coverage/PRD.md) - ERP coverage PRD surface.
15. [workplace integration PRD](products/workplace-integration/PRD.md) - Workplace integration PRD surface.

### Related Sections

Related sections: [§1 Architecture](#section-1-architecture), [§4 User Journeys](#section-4-user-journeys), [§5 Personas](#section-5-personas), [§8 Governance Pipeline](#section-8-governance-pipeline), [§13 Wave Sequence](#section-13-wave-sequence).

### When To Read

- Read this when you need the customer-facing or microservice PRD for a product slice.
- Read this before authoring phase specs or implementation plans for a microservice.
- Read this when mapping hero-product claims to actual accepted PRD files.
- Read this when a product README points to planned drafts; use only resolving files from this hub for current navigation.

### Product PRD Files That Resolve Today

1. [products/README.md](products/README.md) - Products: product or microservice requirements authority for scope, users, boundaries, and success metrics.
2. [products/_TEMPLATE.md](products/_TEMPLATE.md) - Template: product or microservice requirements authority for scope, users, boundaries, and success metrics.
3. [products/cloud/PRD.md](products/cloud/PRD.md) - PRD: product or microservice requirements authority for scope, users, boundaries, and success metrics.
4. [products/erp-coverage/PRD.md](products/erp-coverage/PRD.md) - PRD: product or microservice requirements authority for scope, users, boundaries, and success metrics.
33. [products/product-docs-w1-2026-05-20-checkpoint.md](products/product-docs-w1-2026-05-20-checkpoint.md) - Product Docs W1 2026 05 20 Checkpoint: product or microservice requirements authority for scope, users, boundaries, and success metrics.
34. [products/workplace-integration/PRD.md](products/workplace-integration/PRD.md) - PRD: product or microservice requirements authority for scope, users, boundaries, and success metrics.

### Microservice PRD Files

1. [prds/INDEX.md](prds/INDEX.md) - Index: product or microservice requirements authority for scope, users, boundaries, and success metrics.
2. [prds/accounting.md](prds/accounting.md) - Accounting: product or microservice requirements authority for scope, users, boundaries, and success metrics.
3. [prds/application.md](prds/application.md) - Application: product or microservice requirements authority for scope, users, boundaries, and success metrics.
4. [prds/communications.md](prds/communications.md) - Communications: microservice requirements authority for Messenger, Mail, and Community scope, users, boundaries, and success metrics.
6. [prds/hr.md](prds/hr.md) - Hr: product or microservice requirements authority for scope, users, boundaries, and success metrics.
7. [prds/ontology.md](prds/ontology.md) - Ontology: product or microservice requirements authority for scope, users, boundaries, and success metrics.
8. [prds/payroll.md](prds/payroll.md) - Payroll: product or microservice requirements authority for scope, users, boundaries, and success metrics.
9. [prds/tenancy.md](prds/tenancy.md) - Tenancy: product or microservice requirements authority for scope, users, boundaries, and success metrics.
10. [prds/workflow.md](prds/workflow.md) - Workflow: product or microservice requirements authority for scope, users, boundaries, and success metrics.

<a id="section-4-user-journeys"></a>
## §4 User Journeys

### Section Purpose

User journeys prove whether the architecture and product PRDs cover real workflows. Each journey directory is a compact dossier with README, story, UX flow, handshake, and integration-test plan; reports and catalogs summarize bands of journeys by domain, locale, and ecosystem role.

### Canonical Docs (Top 15)

1. [CATALOG j126-j150](user-journeys/CATALOG-j126-j150-ecosystem.md) - ecosystem journey catalog for j126-j150.
2. [REPORT j36-j50](user-journeys/REPORT-j36-j50-hero.md) - hero journey report for early B2B and healthcare surfaces.
3. [REPORT j51-j75](user-journeys/REPORT-j51-j75-crossproduct.md) - cross-product journey report for shared flows.
4. [REPORT j76-j90](user-journeys/REPORT-j76-j90-locale-packs.md) - locale-pack journey report.
5. [REPORT j91-j100](user-journeys/REPORT-j91-j100-locale-pack-final.md) - final locale-pack journey report.
6. [REPORT j116-j150](user-journeys/REPORT-j116-j150-remainder.md) - remainder report for ecosystem journeys.
7. [j01 emergency dispatch](user-journeys/j01-emergency-911-dispatch/README.md) - life-safety emergency dispatch journey.
8. [j33 B2B SSO](user-journeys/j33-b2b-sso-saml-onboarding/README.md) - B2B SSO onboarding journey.
9. [j41 developer platform](user-journeys/j41-b2b-developer-builds-on-platform/README.md) - developer builds on platform journey.
10. [j76 GDPR cascade](user-journeys/j76-eu-gdpr-dsar-full-cascade/README.md) - GDPR DSR cascade journey.
11. [j100 pack rollout](user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md) - pack rollout from onboarding to first action.
12. [j126 3PAO audit](user-journeys/j126-government-auditor-3pao-conducts-fedramp-audit/README.md) - government auditor 3PAO FedRAMP journey.
13. [j151 typhoon evacuation](user-journeys/j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow/README.md) - field/co-op emergency and cash-flow journey.
14. [j167 version cutover](user-journeys/j167-cto-diego-vargas-platform-major-version-cutover/README.md) - CTO major-version cutover journey.
15. [persona-journey matrix](architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md) - matrix connecting journeys to personas and microservices.

### Related Sections

Related sections: [§1 Architecture](#section-1-architecture), [§3 Products](#section-3-products), [§5 Personas](#section-5-personas), [§9 Compliance Packs](#section-9-compliance-packs), [§10 Capability Tiers](#section-10-capability-tiers).

### When To Read

- Read this when validating whether a product or microservice serves a real end-to-end workflow.
- Read this before adding persona coverage or microservice IPs tied to journey IDs.
- Read this when a compliance pack asks for user-visible evidence of regulated behavior.
- Read this when a stakeholder asks what the platform actually does for a user.

### Journey Reports And Catalogs

1. [CATALOG-j126-j150-ecosystem.md](user-journeys/CATALOG-j126-j150-ecosystem.md) - Catalog J126 J150 Ecosystem: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
2. [REPORT-j116-j150-remainder.md](user-journeys/REPORT-j116-j150-remainder.md) - Report J116 J150 Remainder: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
3. [REPORT-j36-j50-hero.md](user-journeys/REPORT-j36-j50-hero.md) - Report J36 J50 Hero: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
4. [REPORT-j51-j75-crossproduct.md](user-journeys/REPORT-j51-j75-crossproduct.md) - Report J51 J75 Crossproduct: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
5. [REPORT-j76-j90-locale-packs.md](user-journeys/REPORT-j76-j90-locale-packs.md) - Report J76 J90 Locale Packs: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
6. [REPORT-j91-j100-locale-pack-final.md](user-journeys/REPORT-j91-j100-locale-pack-final.md) - Report J91 J100 Locale Pack Final: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
7. [j01-j20-lifesafety-deliverable-report.json](user-journeys/j01-j20-lifesafety-deliverable-report.json) - J01 J20 Lifesafety Deliverable Report: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.

### J01 Through J175 Journey Index

1. [j01-emergency-911-dispatch](user-journeys/j01-emergency-911-dispatch/README.md) - J01 Emergency 911 Dispatch: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
2. [j02-healthcare-code-blue-ehr-break-glass](user-journeys/j02-healthcare-code-blue-ehr-break-glass/README.md) - J02 Healthcare Code Blue Ehr Break Glass: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
3. [j03-988-crisis-line-minor-self-report](user-journeys/j03-988-crisis-line-minor-self-report/README.md) - J03 988 Crisis Line Minor Self Report: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
4. [j04-dv-survivor-shelter-mode](user-journeys/j04-dv-survivor-shelter-mode/README.md) - J04 Dv Survivor Shelter Mode: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
5. [j05-whistleblower-anonymous-ethics-report](user-journeys/j05-whistleblower-anonymous-ethics-report/README.md) - J05 Whistleblower Anonymous Ethics Report: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
6. [j06-press-source-securedrop-class](user-journeys/j06-press-source-securedrop-class/README.md) - J06 Press Source Securedrop Class: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
7. [j07-deceased-user-inheritance-handoff](user-journeys/j07-deceased-user-inheritance-handoff/README.md) - J07 Deceased User Inheritance Handoff: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
8. [j08-elder-financial-abuse-detection](user-journeys/j08-elder-financial-abuse-detection/README.md) - J08 Elder Financial Abuse Detection: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
9. [j09-account-recovery-phishing-resistant](user-journeys/j09-account-recovery-phishing-resistant/README.md) - J09 Account Recovery Phishing Resistant: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
10. [j10-account-takeover-SIM-swap-detected](user-journeys/j10-account-takeover-SIM-swap-detected/README.md) - J10 Account Takeover Sim Swap Detected: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
11. [j11-disaster-zone-offline-first-sync](user-journeys/j11-disaster-zone-offline-first-sync/README.md) - J11 Disaster Zone Offline First Sync: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
12. [j12-mass-casualty-incident-10x-traffic](user-journeys/j12-mass-casualty-incident-10x-traffic/README.md) - J12 Mass Casualty Incident 10X Traffic: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
13. [j13-cross-jurisdiction-eu-cloud-act-conflict](user-journeys/j13-cross-jurisdiction-eu-cloud-act-conflict/README.md) - J13 Cross Jurisdiction EU Cloud Act Conflict: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
14. [j14-delegated-llm-agent-acting-for-yejin](user-journeys/j14-delegated-llm-agent-acting-for-yejin/README.md) - J14 Delegated Llm Agent Acting For Yejin: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
15. [j15-bug-bounty-researcher-submission](user-journeys/j15-bug-bounty-researcher-submission/README.md) - J15 Bug Bounty Researcher Submission: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
16. [j16-disability-accommodation-voice-only-signup](user-journeys/j16-disability-accommodation-voice-only-signup/README.md) - J16 Disability Accommodation Voice Only Signup: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
17. [j17-activist-dissident-high-risk-mode](user-journeys/j17-activist-dissident-high-risk-mode/README.md) - J17 Activist Dissident High Risk Mode: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
18. [j18-child-safety-mandatory-reporter](user-journeys/j18-child-safety-mandatory-reporter/README.md) - J18 Child Safety Mandatory Reporter: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
19. [j19-tenant-break-glass-locked-out-tenant-admin](user-journeys/j19-tenant-break-glass-locked-out-tenant-admin/README.md) - J19 Tenant Break Glass Locked Out Tenant Admin: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
20. [j20-data-residency-violation-detection](user-journeys/j20-data-residency-violation-detection/README.md) - J20 Data Residency Violation Detection: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
21. [j21-personal-signup-passkey-first-dm](user-journeys/j21-personal-signup-passkey-first-dm/README.md) - J21 Personal Signup Passkey First Dm: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
22. [j22-personal-mail-inbox-first-week](user-journeys/j22-personal-mail-inbox-first-week/README.md) - J22 Personal Mail Inbox First Week: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
23. [j23-marketplace-listing-and-first-sale](user-journeys/j23-marketplace-listing-and-first-sale/README.md) - J23 Marketplace Listing And First Sale: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
24. [j24-marketplace-purchase-as-buyer](user-journeys/j24-marketplace-purchase-as-buyer/README.md) - J24 Marketplace Purchase As Buyer: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
25. [j25-personal-notes-daily-journaling-with-e2e](user-journeys/j25-personal-notes-daily-journaling-with-e2e/README.md) - J25 Personal Notes Daily Journaling With E2E: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
26. [j26-drive-family-photo-backup](user-journeys/j26-drive-family-photo-backup/README.md) - J26 Drive Family Photo Backup: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
27. [j27-calendar-cross-context-family-and-work](user-journeys/j27-calendar-cross-context-family-and-work/README.md) - J27 Calendar Cross Context Family And Work: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
28. [j28-meet-family-video-call](user-journeys/j28-meet-family-video-call/README.md) - J28 Meet Family Video Call: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
29. [j29-workflow-studio-personal-automation](user-journeys/j29-workflow-studio-personal-automation/README.md) - J29 Workflow Studio Personal Automation: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
30. [j30-shorts-creator-first-post](user-journeys/j30-shorts-creator-first-post/README.md) - J30 Shorts Creator First Post: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
31. [j31-social-broadcast-vs-DM](user-journeys/j31-social-broadcast-vs-DM/README.md) - J31 Social Broadcast Vs Dm: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
32. [j32-community-teamblind-employer-anonymous](user-journeys/j32-community-teamblind-employer-anonymous/README.md) - J32 Community Teamblind Employer Anonymous: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
33. [j33-b2b-sso-saml-onboarding](user-journeys/j33-b2b-sso-saml-onboarding/README.md) - J33 B2B Sso Saml Onboarding: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
34. [j34-b2b-team-channel-with-files](user-journeys/j34-b2b-team-channel-with-files/README.md) - J34 B2B Team Channel With Files: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
35. [j35-b2b-workplace-mail-and-calendar](user-journeys/j35-b2b-workplace-mail-and-calendar/README.md) - J35 B2B Workplace Mail And Calendar: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
36. [j36-b2b-workflow-engine-approval-cascade](user-journeys/j36-b2b-workflow-engine-approval-cascade/README.md) - J36 B2B Workflow Engine Approval Cascade: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
37. [j37-b2b-clocking-and-attendance](user-journeys/j37-b2b-clocking-and-attendance/README.md) - J37 B2B Clocking And Attendance: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
38. [j38-b2b-e-signing-contract](user-journeys/j38-b2b-e-signing-contract/README.md) - J38 B2B E Signing Contract: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
39. [j39-b2b-meeting-with-transcription](user-journeys/j39-b2b-meeting-with-transcription/README.md) - J39 B2B Meeting With Transcription: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
40. [j40-b2b-marketplace-vendor-billing](user-journeys/j40-b2b-marketplace-vendor-billing/README.md) - J40 B2B Marketplace Vendor Billing: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
41. [j41-b2b-developer-builds-on-platform](user-journeys/j41-b2b-developer-builds-on-platform/README.md) - J41 B2B Developer Builds On Platform: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
42. [j42-b2b-finops-portal-spend-attribution](user-journeys/j42-b2b-finops-portal-spend-attribution/README.md) - J42 B2B Finops Portal Spend Attribution: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
43. [j43-healthcare-nurse-patient-handoff](user-journeys/j43-healthcare-nurse-patient-handoff/README.md) - J43 Healthcare Nurse Patient Handoff: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
44. [j44-healthcare-telemedicine-consultation](user-journeys/j44-healthcare-telemedicine-consultation/README.md) - J44 Healthcare Telemedicine Consultation: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
45. [j45-healthcare-patient-portal-records](user-journeys/j45-healthcare-patient-portal-records/README.md) - J45 Healthcare Patient Portal Records: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
46. [j46-healthcare-prescription-renewal-workflow](user-journeys/j46-healthcare-prescription-renewal-workflow/README.md) - J46 Healthcare Prescription Renewal Workflow: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
47. [j47-healthcare-billing-and-insurance](user-journeys/j47-healthcare-billing-and-insurance/README.md) - J47 Healthcare Billing And Insurance: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
48. [j48-sidebusiness-stripe-tax-and-invoicing](user-journeys/j48-sidebusiness-stripe-tax-and-invoicing/README.md) - J48 Sidebusiness Stripe Tax And Invoicing: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
49. [j49-sidebusiness-customer-support-omnichannel](user-journeys/j49-sidebusiness-customer-support-omnichannel/README.md) - J49 Sidebusiness Customer Support Omnichannel: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
50. [j50-sidebusiness-employee-hires-first-helper](user-journeys/j50-sidebusiness-employee-hires-first-helper/README.md) - J50 Sidebusiness Employee Hires First Helper: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
51. [j51-procure-to-pay-po-extraction-and-approval](user-journeys/j51-procure-to-pay-po-extraction-and-approval/README.md) - J51 Procure To Pay Po Extraction And Approval: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
52. [j52-order-to-cash-marketplace-to-fulfillment](user-journeys/j52-order-to-cash-marketplace-to-fulfillment/README.md) - J52 Order To Cash Marketplace To Fulfillment: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
53. [j53-invoice-to-cash-recurring-subscription](user-journeys/j53-invoice-to-cash-recurring-subscription/README.md) - J53 Invoice To Cash Recurring Subscription: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
54. [j54-quote-to-contract-to-payment-saas](user-journeys/j54-quote-to-contract-to-payment-saas/README.md) - J54 Quote To Contract To Payment Saas: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
55. [j55-refund-and-dispute-resolution-cascade](user-journeys/j55-refund-and-dispute-resolution-cascade/README.md) - J55 Refund And Dispute Resolution Cascade: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
56. [j56-job-application-to-offer](user-journeys/j56-job-application-to-offer/README.md) - J56 Job Application To Offer: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
57. [j57-employee-onboarding-day-one-to-week-one](user-journeys/j57-employee-onboarding-day-one-to-week-one/README.md) - J57 Employee Onboarding Day One To Week One: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
58. [j58-quarterly-performance-review-cycle](user-journeys/j58-quarterly-performance-review-cycle/README.md) - J58 Quarterly Performance Review Cycle: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
59. [j59-offboarding-and-knowledge-transfer](user-journeys/j59-offboarding-and-knowledge-transfer/README.md) - J59 Offboarding And Knowledge Transfer: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
60. [j60-internal-mobility-promotion-cascade](user-journeys/j60-internal-mobility-promotion-cascade/README.md) - J60 Internal Mobility Promotion Cascade: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
61. [j61-patient-intake-to-followup](user-journeys/j61-patient-intake-to-followup/README.md) - J61 Patient Intake To Followup: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
62. [j62-prescription-to-pharmacy-to-payment](user-journeys/j62-prescription-to-pharmacy-to-payment/README.md) - J62 Prescription To Pharmacy To Payment: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
63. [j63-clinical-trial-recruitment-to-consent](user-journeys/j63-clinical-trial-recruitment-to-consent/README.md) - J63 Clinical Trial Recruitment To Consent: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
64. [j64-hospital-network-cross-tenant-referral](user-journeys/j64-hospital-network-cross-tenant-referral/README.md) - J64 Hospital Network Cross Tenant Referral: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
65. [j65-gdpr-dsar-cascade-across-all-services](user-journeys/j65-gdpr-dsar-cascade-across-all-services/README.md) - J65 Gdpr DSAR Cascade Across All Services: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
66. [j66-tax-quarterly-filing-multi-jurisdiction](user-journeys/j66-tax-quarterly-filing-multi-jurisdiction/README.md) - J66 Tax Quarterly Filing Multi Jurisdiction: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
67. [j67-law-enforcement-warrant-response](user-journeys/j67-law-enforcement-warrant-response/README.md) - J67 Law Enforcement Warrant Response: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
68. [j68-regulator-audit-pull-hippa-soc2-pci](user-journeys/j68-regulator-audit-pull-hippa-soc2-pci/README.md) - J68 Regulator Audit Pull Hippa SOC2 PCI: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
69. [j69-llm-agent-managing-yejins-week](user-journeys/j69-llm-agent-managing-yejins-week/README.md) - J69 Llm Agent Managing Yejins Week: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
70. [j70-ai-drafted-contract-human-finalized](user-journeys/j70-ai-drafted-contract-human-finalized/README.md) - J70 AI Drafted Contract Human Finalized: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
71. [j71-ai-detected-fraud-pattern-response](user-journeys/j71-ai-detected-fraud-pattern-response/README.md) - J71 AI Detected Fraud Pattern Response: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
72. [j72-ai-translation-cross-locale-business](user-journeys/j72-ai-translation-cross-locale-business/README.md) - J72 AI Translation Cross Locale Business: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
73. [j73-third-party-developer-publishes-plugin](user-journeys/j73-third-party-developer-publishes-plugin/README.md) - J73 Third Party Developer Publishes Plugin: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
74. [j74-tenant-installs-plugin-and-it-spans-services](user-journeys/j74-tenant-installs-plugin-and-it-spans-services/README.md) - J74 Tenant Installs Plugin And It Spans Services: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
75. [j75-plugin-revoked-during-incident-response](user-journeys/j75-plugin-revoked-during-incident-response/README.md) - J75 Plugin Revoked During Incident Response: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
76. [j76-eu-gdpr-dsar-full-cascade](user-journeys/j76-eu-gdpr-dsar-full-cascade/README.md) - J76 EU Gdpr DSAR Full Cascade: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
77. [j77-eu-ai-act-high-risk-credit-decision](user-journeys/j77-eu-ai-act-high-risk-credit-decision/README.md) - J77 EU AI Act High Risk Credit Decision: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
78. [j78-eu-nis2-breach-three-stage-cadence](user-journeys/j78-eu-nis2-breach-three-stage-cadence/README.md) - J78 EU Nis2 Breach Three Stage Cadence: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
79. [j79-eu-dsa-transparency-semi-annual-report](user-journeys/j79-eu-dsa-transparency-semi-annual-report/README.md) - J79 EU Dsa Transparency Semi Annual Report: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
80. [j80-kr-pipa-personal-info-cross-border-transfer](user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/README.md) - J80 KR Pipa Personal Info Cross Border Transfer: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
81. [j81-kr-csap-sovereign-cell-audit-pull](user-journeys/j81-kr-csap-sovereign-cell-audit-pull/README.md) - J81 KR CSAP Sovereign Cell Audit Pull: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
82. [j82-kr-fss-financial-fraud-24h-freeze](user-journeys/j82-kr-fss-financial-fraud-24h-freeze/README.md) - J82 KR Fss Financial Fraud 24H Freeze: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
83. [j83-cn-pipl-data-localization-and-cac-assessment](user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/README.md) - J83 Cn Pipl Data Localization And Cac Assessment: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
84. [j84-jp-appi-elder-user-consent](user-journeys/j84-jp-appi-elder-user-consent/README.md) - J84 Jp Appi Elder User Consent: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
85. [j85-hipaa-end-to-end-phi-workflow](user-journeys/j85-hipaa-end-to-end-phi-workflow/README.md) - J85 HIPAA End To End Phi Workflow: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
86. [j86-pci-dss-l1-tokenized-payment-flow](user-journeys/j86-pci-dss-l1-tokenized-payment-flow/README.md) - J86 PCI Dss L1 Tokenized Payment Flow: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
87. [j87-fedramp-high-il5-air-gap-deployment](user-journeys/j87-fedramp-high-il5-air-gap-deployment/README.md) - J87 Fedramp High Il5 Air Gap Deployment: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
88. [j88-au-irap-protected-tenant](user-journeys/j88-au-irap-protected-tenant/README.md) - J88 Au Irap Protected Tenant: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
89. [j89-uk-aadc-minor-ux-adaptation](user-journeys/j89-uk-aadc-minor-ux-adaptation/README.md) - J89 Uk Aadc Minor Ux Adaptation: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
90. [j90-us-ccpa-cpra-do-not-sell-opt-out](user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/README.md) - J90 US Ccpa Cpra Do Not Sell Opt Out: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
91. [j91-us-state-money-transmitter-licensing](user-journeys/j91-us-state-money-transmitter-licensing/README.md) - J91 US State Money Transmitter Licensing: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
92. [j92-br-lgpd-dsar-with-us-parent](user-journeys/j92-br-lgpd-dsar-with-us-parent/README.md) - J92 Br Lgpd DSAR With US Parent: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
93. [j93-in-dpdpa-rbi-financial-overlay](user-journeys/j93-in-dpdpa-rbi-financial-overlay/README.md) - J93 In Dpdpa Rbi Financial Overlay: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
94. [j94-sox-404-public-company-controls](user-journeys/j94-sox-404-public-company-controls/README.md) - J94 Sox 404 Public Company Controls: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
95. [j95-iso-27001-soc-2-annual-audit](user-journeys/j95-iso-27001-soc-2-annual-audit/README.md) - J95 Iso 27001 Soc 2 Annual Audit: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
96. [j96-ksa-uae-mena-tenant-onboarding](user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md) - J96 Ksa Uae Mena Tenant Onboarding: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
97. [j97-sg-pdpa-mas-singapore-tenant](user-journeys/j97-sg-pdpa-mas-singapore-tenant/README.md) - J97 Sg Pdpa Mas Singapore Tenant: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
98. [j98-au-privacy-apra-cps-234-tenant](user-journeys/j98-au-privacy-apra-cps-234-tenant/README.md) - J98 Au Privacy Apra Cps 234 Tenant: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
99. [j99-cross-jurisdiction-multi-pack-conflict-resolution](user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/README.md) - J99 Cross Jurisdiction Multi Pack Conflict Resolution: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
100. [j100-pack-rollout-from-tenant-onboarding-to-first-action](user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md) - J100 Pack Rollout From Tenant Onboarding To First Action: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
101. [j101-multi-tier-supply-chain-formation](user-journeys/j101-multi-tier-supply-chain-formation/README.md) - J101 Multi Tier Supply Chain Formation: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
102. [j102-raw-material-purchase-with-quality-attestation](user-journeys/j102-raw-material-purchase-with-quality-attestation/README.md) - J102 Raw Material Purchase With Quality Attestation: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
103. [j103-just-in-time-procurement-automation](user-journeys/j103-just-in-time-procurement-automation/README.md) - J103 Just In Time Procurement Automation: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
104. [j104-supplier-vendor-onboarding-kyb-cascade](user-journeys/j104-supplier-vendor-onboarding-kyb-cascade/README.md) - J104 Supplier Vendor Onboarding KYB Cascade: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
105. [j105-dispute-cross-tenant-arbitration](user-journeys/j105-dispute-cross-tenant-arbitration/README.md) - J105 Dispute Cross Tenant Arbitration: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
106. [j106-multi-currency-cross-border-payment](user-journeys/j106-multi-currency-cross-border-payment/README.md) - J106 Multi Currency Cross Border Payment: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
107. [j107-supply-chain-disruption-and-failover](user-journeys/j107-supply-chain-disruption-and-failover/README.md) - J107 Supply Chain Disruption And Failover: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
108. [j108-supplier-rating-and-marketplace-discovery](user-journeys/j108-supplier-rating-and-marketplace-discovery/README.md) - J108 Supplier Rating And Marketplace Discovery: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
109. [j109-construction-co-hires-freelance-specialist](user-journeys/j109-construction-co-hires-freelance-specialist/README.md) - J109 Construction Co Hires Freelance Specialist: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
110. [j110-traveling-nurse-multi-employer-roster](user-journeys/j110-traveling-nurse-multi-employer-roster/README.md) - J110 Traveling Nurse Multi Employer Roster: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
111. [j111-staffing-agency-as-tenant-facilitator](user-journeys/j111-staffing-agency-as-tenant-facilitator/README.md) - J111 Staffing Agency As Tenant Facilitator: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
112. [j112-tenant-to-tenant-rfq-and-bid](user-journeys/j112-tenant-to-tenant-rfq-and-bid/README.md) - J112 Tenant To Tenant Rfq And Bid: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
113. [j113-cross-tenant-internship-from-handshake](user-journeys/j113-cross-tenant-internship-from-handshake/README.md) - J113 Cross Tenant Internship From Handshake: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
114. [j114-employee-secondment-cross-tenant](user-journeys/j114-employee-secondment-cross-tenant/README.md) - J114 Employee Secondment Cross Tenant: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
115. [j115-saas-vendor-sells-api-to-multiple-tenant-customers](user-journeys/j115-saas-vendor-sells-api-to-multiple-tenant-customers/README.md) - J115 Saas Vendor Sells API To Multiple Tenant Customers: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
116. [j116-plugin-marketplace-developer-publishes-and-monetizes](user-journeys/j116-plugin-marketplace-developer-publishes-and-monetizes/README.md) - J116 Plugin Marketplace Developer Publishes And Monetizes: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
117. [j117-api-customer-tenant-incident-response](user-journeys/j117-api-customer-tenant-incident-response/README.md) - J117 API Customer Tenant Incident Response: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
118. [j118-tenant-to-tenant-data-sharing-via-ontology-projection](user-journeys/j118-tenant-to-tenant-data-sharing-via-ontology-projection/README.md) - J118 Tenant To Tenant Data Sharing Via Ontology Projection: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
119. [j119-invoice-financing-marketplace](user-journeys/j119-invoice-financing-marketplace/README.md) - J119 Invoice Financing Marketplace: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
120. [j120-tenant-treasury-multi-currency-fx-hedge](user-journeys/j120-tenant-treasury-multi-currency-fx-hedge/README.md) - J120 Tenant Treasury Multi Currency Fx Hedge: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
121. [j121-business-loan-application-from-bank-tenant](user-journeys/j121-business-loan-application-from-bank-tenant/README.md) - J121 Business Loan Application From Bank Tenant: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
122. [j122-vendor-payment-batch-with-tax-withholding](user-journeys/j122-vendor-payment-batch-with-tax-withholding/README.md) - J122 Vendor Payment Batch With Tax Withholding: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
123. [j123-multi-tenant-coordinated-product-launch](user-journeys/j123-multi-tenant-coordinated-product-launch/README.md) - J123 Multi Tenant Coordinated Product Launch: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
124. [j124-supply-chain-disruption-emergency-coordination](user-journeys/j124-supply-chain-disruption-emergency-coordination/README.md) - J124 Supply Chain Disruption Emergency Coordination: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
125. [j125-marketplace-acquires-supplier-tenant-merger](user-journeys/j125-marketplace-acquires-supplier-tenant-merger/README.md) - J125 Marketplace Acquires Supplier Tenant Merger: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
126. [j126-government-auditor-3pao-conducts-fedramp-audit](user-journeys/j126-government-auditor-3pao-conducts-fedramp-audit/README.md) - J126 Government Auditor 3Pao Conducts Fedramp Audit: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
127. [j127-dual-tenant-identity-employee-resigns-and-keeps-personal](user-journeys/j127-dual-tenant-identity-employee-resigns-and-keeps-personal/README.md) - J127 Dual Tenant Identity Employee Resigns And Keeps Personal: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
128. [j128-auditor-personal-side-uses-workflow-studio-for-family-taxes](user-journeys/j128-auditor-personal-side-uses-workflow-studio-for-family-taxes/README.md) - J128 Auditor Personal Side Uses Workflow Studio For Family Taxes: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
129. [j129-court-warrant-pierces-personal-tenant-with-judicial-oversight](user-journeys/j129-court-warrant-pierces-personal-tenant-with-judicial-oversight/README.md) - J129 Court Warrant Pierces Personal Tenant With Judicial Oversight: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
130. [j130-auditor-receives-bribery-attempt-via-personal-messenger](user-journeys/j130-auditor-receives-bribery-attempt-via-personal-messenger/README.md) - J130 Auditor Receives Bribery Attempt Via Personal Messenger: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
131. [j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy](user-journeys/j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy/README.md) - J131 Cross Jurisdiction Audit EU Vs KR Discrepancy: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
132. [j132-hr-mass-hiring-event-100-roles](user-journeys/j132-hr-mass-hiring-event-100-roles/README.md) - J132 Hr Mass Hiring Event 100 Roles: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
133. [j133-hr-conducts-layoff-with-dignity-and-compliance](user-journeys/j133-hr-conducts-layoff-with-dignity-and-compliance/README.md) - J133 Hr Conducts Layoff With Dignity And Compliance: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
134. [j134-hr-cross-tenant-recruitment-via-staffing-agency](user-journeys/j134-hr-cross-tenant-recruitment-via-staffing-agency/README.md) - J134 Hr Cross Tenant Recruitment Via Staffing Agency: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
135. [j135-hr-handles-harassment-complaint-with-dual-tenant-boundary](user-journeys/j135-hr-handles-harassment-complaint-with-dual-tenant-boundary/README.md) - J135 Hr Handles Harassment Complaint With Dual Tenant Boundary: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
136. [j136-hr-administers-benefits-open-enrollment](user-journeys/j136-hr-administers-benefits-open-enrollment/README.md) - J136 Hr Administers Benefits Open Enrollment: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
137. [j137-corporate-internal-audit-sox-controls-test](user-journeys/j137-corporate-internal-audit-sox-controls-test/README.md) - J137 Corporate Internal Audit Sox Controls Test: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
138. [j138-corporate-audit-fraud-investigation-via-pattern-detection](user-journeys/j138-corporate-audit-fraud-investigation-via-pattern-detection/README.md) - J138 Corporate Audit Fraud Investigation Via Pattern Detection: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
139. [j139-internal-audit-policy-violation-cedar-permit-misuse](user-journeys/j139-internal-audit-policy-violation-cedar-permit-misuse/README.md) - J139 Internal Audit Policy Violation Cedar Permit Misuse: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
140. [j140-internal-audit-data-loss-prevention-egress-trip](user-journeys/j140-internal-audit-data-loss-prevention-egress-trip/README.md) - J140 Internal Audit Data Loss Prevention Egress Trip: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
141. [j141-internal-audit-respects-employee-personal-tenant-boundary](user-journeys/j141-internal-audit-respects-employee-personal-tenant-boundary/README.md) - J141 Internal Audit Respects Employee Personal Tenant Boundary: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
142. [j142-layoff-day-zero-from-employees-side](user-journeys/j142-layoff-day-zero-from-employees-side/README.md) - J142 Layoff Day Zero From Employees Side: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
143. [j143-laid-off-imports-work-portfolio-into-personal-tenant](user-journeys/j143-laid-off-imports-work-portfolio-into-personal-tenant/README.md) - J143 Laid Off Imports Work Portfolio Into Personal Tenant: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
144. [j144-laid-off-builds-job-search-pipeline-in-workflow-studio](user-journeys/j144-laid-off-builds-job-search-pipeline-in-workflow-studio/README.md) - J144 Laid Off Builds Job Search Pipeline In Workflow Studio: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
145. [j145-laid-off-applies-via-community-handshake-linkedin-mode](user-journeys/j145-laid-off-applies-via-community-handshake-linkedin-mode/README.md) - J145 Laid Off Applies Via Community Handshake Linkedin Mode: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
146. [j146-laid-off-uses-marketplace-as-temporary-income](user-journeys/j146-laid-off-uses-marketplace-as-temporary-income/README.md) - J146 Laid Off Uses Marketplace As Temporary Income: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
147. [j147-laid-off-cohort-mutual-aid-community-channel](user-journeys/j147-laid-off-cohort-mutual-aid-community-channel/README.md) - J147 Laid Off Cohort Mutual Aid Community Channel: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
148. [j148-supply-chain-circular-economy-electronics-recycling](user-journeys/j148-supply-chain-circular-economy-electronics-recycling/README.md) - J148 Supply Chain Circular Economy Electronics Recycling: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
149. [j149-gig-economy-multi-platform-worker](user-journeys/j149-gig-economy-multi-platform-worker/README.md) - J149 Gig Economy Multi Platform Worker: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
150. [j150-creator-economy-shorts-creator-monetization-stack](user-journeys/j150-creator-economy-shorts-creator-monetization-stack/README.md) - J150 Creator Economy Shorts Creator Monetization Stack: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
151. [j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow](user-journeys/j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow/README.md) - J151 Captain Olufemi Typhoon Evacuation And Co Op Cash Flow: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
152. [j152-ahmad-hassan-construction-site-incident-bilingual](user-journeys/j152-ahmad-hassan-construction-site-incident-bilingual/README.md) - J152 Ahmad Hassan Construction Site Incident Bilingual: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
153. [j153-devon-williams-hvac-side-business-tax-end-of-year](user-journeys/j153-devon-williams-hvac-side-business-tax-end-of-year/README.md) - J153 Devon Williams Hvac Side Business Tax End Of Year: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
154. [j154-tomas-pieter-channel-partner-co-marketing-launch](user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch/README.md) - J154 Tomas Pieter Channel Partner Co Marketing Launch: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
155. [j155-stefan-kovacs-college-night-shift-and-finals-week](user-journeys/j155-stefan-kovacs-college-night-shift-and-finals-week/README.md) - J155 Stefan Kovacs College Night Shift And Finals Week: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
156. [j156-carlos-reyes-ii-maintenance-emergency-after-hours](user-journeys/j156-carlos-reyes-ii-maintenance-emergency-after-hours/README.md) - J156 Carlos Reyes Ii Maintenance Emergency After Hours: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
157. [j157-diana-lazar-print-operator-batch-defect-and-quality-recall](user-journeys/j157-diana-lazar-print-operator-batch-defect-and-quality-recall/README.md) - J157 Diana Lazar Print Operator Batch Defect And Quality Recall: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
158. [j158-print-shop-cell-rebalance-shorts-creator-spike](user-journeys/j158-print-shop-cell-rebalance-shorts-creator-spike/README.md) - J158 Print Shop Cell Rebalance Shorts Creator Spike: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
159. [j159-saanvi-mehta-mba-application-spans-personal-and-work](user-journeys/j159-saanvi-mehta-mba-application-spans-personal-and-work/README.md) - J159 Saanvi Mehta Mba Application Spans Personal And Work: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
160. [j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard](user-journeys/j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard/README.md) - J160 Cleaning Co Tomas Horak Bid Cross Tenant And Onboard: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
161. [j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination](user-journeys/j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination/README.md) - J161 Cafeteria Soyeon Kim Allergen Recall And School Coordination: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
162. [j162-print-operator-diana-lazar-night-shift-onboarding](user-journeys/j162-print-operator-diana-lazar-night-shift-onboarding/README.md) - J162 Print Operator Diana Lazar Night Shift Onboarding: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
163. [j163-av-coordinator-jordan-park-board-meeting-cross-time-zone](user-journeys/j163-av-coordinator-jordan-park-board-meeting-cross-time-zone/README.md) - J163 Av Coordinator Jordan Park Board Meeting Cross Time Zone: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
164. [j164-retired-hiroshi-tanaka-yearly-tax-and-pension](user-journeys/j164-retired-hiroshi-tanaka-yearly-tax-and-pension/README.md) - J164 Retired Hiroshi Tanaka Yearly Tax And Pension: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
165. [j165-cco-naveen-iyer-board-quarterly-compliance-report](user-journeys/j165-cco-naveen-iyer-board-quarterly-compliance-report/README.md) - J165 Cco Naveen Iyer Board Quarterly Compliance Report: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
166. [j166-cso-mira-goldberg-strategic-acquisition-go-no-go](user-journeys/j166-cso-mira-goldberg-strategic-acquisition-go-no-go/README.md) - J166 Cso Mira Goldberg Strategic Acquisition Go No Go: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
167. [j167-cto-diego-vargas-platform-major-version-cutover](user-journeys/j167-cto-diego-vargas-platform-major-version-cutover/README.md) - J167 Cto Diego Vargas Platform Major Version Cutover: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
168. [j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief](user-journeys/j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief/README.md) - J168 Coo Akira Watanabe Quarterly Ops Review And Incident Debrief: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
169. [j169-cmo-felix-ng-multi-country-launch-with-locale-pack](user-journeys/j169-cmo-felix-ng-multi-country-launch-with-locale-pack/README.md) - J169 Cmo Felix Ng Multi Country Launch With Locale Pack: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
170. [j170-aiko-brown-sustainability-report-and-scope-3-supply-chain](user-journeys/j170-aiko-brown-sustainability-report-and-scope-3-supply-chain/README.md) - J170 Aiko Brown Sustainability Report And Scope 3 Supply Chain: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.
171. [j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege](user-journeys/j171-felix-tan-ombudsperson-cross-tenant-mediation-with-privilege/README.md) - J171 Felix Tan Ombudsperson Cross Tenant Mediation With Privilege: journey dossier entry point with story, UX flow, handshake, and integration-test plan beneath the same directory.

<a id="section-5-personas"></a>
## §5 Personas

### Section Purpose

Personas define who the platform serves across tenant, role, locale, device, skill tier, and workspace contexts. The master roster is the authority; individual dossiers are projections used by journeys, PRDs, and role-based UX decisions.

### Canonical Docs (Top 15)

1. [persona master roster](personas/MASTER-ROSTER-2026-05-21.md) - canonical persona master roster and projection doctrine.
2. [Yejin Park](personas/yejin-park.md) - cross-context consumer/worker/family persona anchor.
3. [Marcus Chen](personas/marcus-chen.md) - executive/family cross-context persona anchor.
4. [Aiyana Singh](personas/aiyana-singh.md) - senior ML engineer and family persona anchor.
5. [Carlos Martinez](personas/carlos-martinez-forklift.md) - blue-collar field worker persona anchor.
6. [Diana Reyes](personas/diana-reyes.md) - auditor persona anchor.
7. [CFO Helena Brandt](personas/cfo-helena-brandt.md) - finance executive persona anchor.
8. [CISO Yuki Park](personas/ciso-yuki-park.md) - security executive persona anchor.
9. [CCO Naveen Iyer](personas/cco-naveen-iyer.md) - compliance/legal executive persona anchor.
10. [CTO Diego Vargas](personas/cto-diego-vargas.md) - technology executive persona anchor.
11. [Carlos Reyes II](personas/maintenance-tech-carlos-reyes-ii.md) - maintenance emergency field persona anchor.
12. [Diana Lazar](personas/print-operator-diana-lazar.md) - production worker persona anchor.
13. [Aiko Brown](personas/sustainability-officer-aiko-brown.md) - sustainability and supply-chain persona anchor.
14. [Sven Eriksson](personas/treasury-ops-sven-eriksson.md) - treasury operations persona anchor.
15. [Aanya Kapoor](personas/investor-lp-aanya-kapoor.md) - investor/LP persona anchor.

### Related Sections

Related sections: [§1 Architecture](#section-1-architecture), [§3 Products](#section-3-products), [§4 User Journeys](#section-4-user-journeys), [§12 Glossary](#section-12-glossary).

### When To Read

- Read this before writing or reviewing a user journey.
- Read this when a feature assumes a role, authority level, device, locale, or tenant context.
- Read this when checking whether the unified-ecosystem thesis covers non-office, field, clinical, executive, and consumer contexts.
- Read the master roster first; individual dossiers answer local questions after that.

### Persona Roster Files

1. [MASTER-ROSTER-2026-05-21.md](personas/MASTER-ROSTER-2026-05-21.md) - Master Roster 2026 05 21: persona dossier for role, context, locale, tenant membership, and journey coverage.
2. [accountant-ravi-iyer.md](personas/accountant-ravi-iyer.md) - Accountant Ravi Iyer: persona dossier for role, context, locale, tenant membership, and journey coverage.
3. [ahmad-hassan.md](personas/ahmad-hassan.md) - Ahmad Hassan: persona dossier for role, context, locale, tenant membership, and journey coverage.
4. [aiyana-singh.md](personas/aiyana-singh.md) - Aiyana Singh: persona dossier for role, context, locale, tenant membership, and journey coverage.
5. [anya-mironova.md](personas/anya-mironova.md) - Anya Mironova: persona dossier for role, context, locale, tenant membership, and journey coverage.
6. [apprentice-jakob-bauer.md](personas/apprentice-jakob-bauer.md) - Apprentice Jakob Bauer: persona dossier for role, context, locale, tenant membership, and journey coverage.
7. [auditor-it-specialist-jakub-nowak.md](personas/auditor-it-specialist-jakub-nowak.md) - Auditor It Specialist Jakub Nowak: persona dossier for role, context, locale, tenant membership, and journey coverage.
8. [av-coordinator-jordan-park.md](personas/av-coordinator-jordan-park.md) - Av Coordinator Jordan Park: persona dossier for role, context, locale, tenant membership, and journey coverage.
9. [bank-compliance-officer-rishi-bhattacharya.md](personas/bank-compliance-officer-rishi-bhattacharya.md) - Bank Compliance Officer Rishi Bhattacharya: persona dossier for role, context, locale, tenant membership, and journey coverage.
10. [bank-ops-officer-olamide-adebanjo.md](personas/bank-ops-officer-olamide-adebanjo.md) - Bank Ops Officer Olamide Adebanjo: persona dossier for role, context, locale, tenant membership, and journey coverage.
11. [bank-risk-manager-anders-pedersen.md](personas/bank-risk-manager-anders-pedersen.md) - Bank Risk Manager Anders Pedersen: persona dossier for role, context, locale, tenant membership, and journey coverage.
12. [banker-external-hideki-watanabe.md](personas/banker-external-hideki-watanabe.md) - Banker External Hideki Watanabe: persona dossier for role, context, locale, tenant membership, and journey coverage.
13. [benefits-specialist-aoife-murphy.md](personas/benefits-specialist-aoife-murphy.md) - Benefits Specialist Aoife Murphy: persona dossier for role, context, locale, tenant membership, and journey coverage.
14. [board-director-patrick-oreilly.md](personas/board-director-patrick-oreilly.md) - Board Director Patrick Oreilly: persona dossier for role, context, locale, tenant membership, and journey coverage.
15. [board-secretary-florence-akinsanya.md](personas/board-secretary-florence-akinsanya.md) - Board Secretary Florence Akinsanya: persona dossier for role, context, locale, tenant membership, and journey coverage.
16. [business-analyst-aditya-verma.md](personas/business-analyst-aditya-verma.md) - Business Analyst Aditya Verma: persona dossier for role, context, locale, tenant membership, and journey coverage.
17. [cafeteria-manager-soyeon-kim.md](personas/cafeteria-manager-soyeon-kim.md) - Cafeteria Manager Soyeon Kim: persona dossier for role, context, locale, tenant membership, and journey coverage.
18. [captain-chen-pilot.md](personas/captain-chen-pilot.md) - Captain Chen Pilot: persona dossier for role, context, locale, tenant membership, and journey coverage.
19. [captain-olufemi.md](personas/captain-olufemi.md) - Captain Olufemi: persona dossier for role, context, locale, tenant membership, and journey coverage.
20. [carlos-martinez-forklift.md](personas/carlos-martinez-forklift.md) - Carlos Martinez Forklift: persona dossier for role, context, locale, tenant membership, and journey coverage.
21. [cco-naveen-iyer.md](personas/cco-naveen-iyer.md) - Cco Naveen Iyer: persona dossier for role, context, locale, tenant membership, and journey coverage.
22. [ceo-aoki-tanaka.md](personas/ceo-aoki-tanaka.md) - Ceo Aoki Tanaka: persona dossier for role, context, locale, tenant membership, and journey coverage.
23. [cfo-helena-brandt.md](personas/cfo-helena-brandt.md) - Cfo Helena Brandt: persona dossier for role, context, locale, tenant membership, and journey coverage.
24. [channel-partner-tomas-pieter.md](personas/channel-partner-tomas-pieter.md) - Channel Partner Tomas Pieter: persona dossier for role, context, locale, tenant membership, and journey coverage.
25. [chris-volkov.md](personas/chris-volkov.md) - Chris Volkov: persona dossier for role, context, locale, tenant membership, and journey coverage.
26. [chro-linda-foster.md](personas/chro-linda-foster.md) - Chro Linda Foster: persona dossier for role, context, locale, tenant membership, and journey coverage.
27. [ciso-yuki-park.md](personas/ciso-yuki-park.md) - Ciso Yuki Park: persona dossier for role, context, locale, tenant membership, and journey coverage.
28. [cleaning-supervisor-tomas-horak.md](personas/cleaning-supervisor-tomas-horak.md) - Cleaning Supervisor Tomas Horak: persona dossier for role, context, locale, tenant membership, and journey coverage.
29. [cmo-felix-ng.md](personas/cmo-felix-ng.md) - Cmo Felix Ng: persona dossier for role, context, locale, tenant membership, and journey coverage.
30. [co-op-student-liam-murphy.md](personas/co-op-student-liam-murphy.md) - Co Op Student Liam Murphy: persona dossier for role, context, locale, tenant membership, and journey coverage.
31. [coach-park.md](personas/coach-park.md) - Coach Park: persona dossier for role, context, locale, tenant membership, and journey coverage.
32. [commercial-banker-frederik-hartmann.md](personas/commercial-banker-frederik-hartmann.md) - Commercial Banker Frederik Hartmann: persona dossier for role, context, locale, tenant membership, and journey coverage.
33. [communications-specialist-charlotte-dubois.md](personas/communications-specialist-charlotte-dubois.md) - Communications Specialist Charlotte Dubois: persona dossier for role, context, locale, tenant membership, and journey coverage.
34. [compliance-analyst-yui-hayashi.md](personas/compliance-analyst-yui-hayashi.md) - Compliance Analyst Yui Hayashi: persona dossier for role, context, locale, tenant membership, and journey coverage.
35. [compliance-officer-tunde-bello.md](personas/compliance-officer-tunde-bello.md) - Compliance Officer Tunde Bello: persona dossier for role, context, locale, tenant membership, and journey coverage.
36. [consultant-adekunle-adebayo.md](personas/consultant-adekunle-adebayo.md) - Consultant Adekunle Adebayo: persona dossier for role, context, locale, tenant membership, and journey coverage.
37. [coo-akira-watanabe.md](personas/coo-akira-watanabe.md) - Coo Akira Watanabe: persona dossier for role, context, locale, tenant membership, and journey coverage.
38. [corp-dev-senior-analyst-saanvi-mehta.md](personas/corp-dev-senior-analyst-saanvi-mehta.md) - Corp Dev Senior Analyst Saanvi Mehta: persona dossier for role, context, locale, tenant membership, and journey coverage.
39. [corporate-relations-director-soo-yeon-han.md](personas/corporate-relations-director-soo-yeon-han.md) - Corporate Relations Director Soo Yeon Han: persona dossier for role, context, locale, tenant membership, and journey coverage.
40. [credit-analyst-hina-mori.md](personas/credit-analyst-hina-mori.md) - Credit Analyst Hina Mori: persona dossier for role, context, locale, tenant membership, and journey coverage.
41. [cs-ic-lin-chen.md](personas/cs-ic-lin-chen.md) - Cs Ic Lin Chen: persona dossier for role, context, locale, tenant membership, and journey coverage.
42. [cso-mira-goldberg.md](personas/cso-mira-goldberg.md) - Cso Mira Goldberg: persona dossier for role, context, locale, tenant membership, and journey coverage.
43. [cto-diego-vargas.md](personas/cto-diego-vargas.md) - Cto Diego Vargas: persona dossier for role, context, locale, tenant membership, and journey coverage.
44. [customer-champion-akemi-sato.md](personas/customer-champion-akemi-sato.md) - Customer Champion Akemi Sato: persona dossier for role, context, locale, tenant membership, and journey coverage.
45. [customer-success-manager-sofia-rezende.md](personas/customer-success-manager-sofia-rezende.md) - Customer Success Manager Sofia Rezende: persona dossier for role, context, locale, tenant membership, and journey coverage.
46. [d-and-i-director-maya-okoroafor.md](personas/d-and-i-director-maya-okoroafor.md) - D And I Director Maya Okoroafor: persona dossier for role, context, locale, tenant membership, and journey coverage.
47. [data-analyst-felipe-andrade.md](personas/data-analyst-felipe-andrade.md) - Data Analyst Felipe Andrade: persona dossier for role, context, locale, tenant membership, and journey coverage.
48. [data-scientist-yu-chen.md](personas/data-scientist-yu-chen.md) - Data Scientist Yu Chen: persona dossier for role, context, locale, tenant membership, and journey coverage.
49. [devon-williams.md](personas/devon-williams.md) - Devon Williams: persona dossier for role, context, locale, tenant membership, and journey coverage.
50. [devops-engineer-olukayode-adejumo.md](personas/devops-engineer-olukayode-adejumo.md) - Devops Engineer Olukayode Adejumo: persona dossier for role, context, locale, tenant membership, and journey coverage.
51. [devops-manager-pavel-korsak.md](personas/devops-manager-pavel-korsak.md) - Devops Manager Pavel Korsak: persona dossier for role, context, locale, tenant membership, and journey coverage.
52. [diana-reyes.md](personas/diana-reyes.md) - Diana Reyes: persona dossier for role, context, locale, tenant membership, and journey coverage.
53. [dr-tanaka-surgeon.md](personas/dr-tanaka-surgeon.md) - Dr Tanaka Surgeon: persona dossier for role, context, locale, tenant membership, and journey coverage.
54. [engineering-manager-aisha-ali.md](personas/engineering-manager-aisha-ali.md) - Engineering Manager Aisha Ali: persona dossier for role, context, locale, tenant membership, and journey coverage.
55. [executive-assistant-olivia-reyes.md](personas/executive-assistant-olivia-reyes.md) - Executive Assistant Olivia Reyes: persona dossier for role, context, locale, tenant membership, and journey coverage.
56. [external-auditor-dimitri-volkov.md](personas/external-auditor-dimitri-volkov.md) - External Auditor Dimitri Volkov: persona dossier for role, context, locale, tenant membership, and journey coverage.
57. [external-auditor-hyo-jin-lee.md](personas/external-auditor-hyo-jin-lee.md) - External Auditor Hyo Jin Lee: persona dossier for role, context, locale, tenant membership, and journey coverage.
58. [father-lopez-priest.md](personas/father-lopez-priest.md) - Father Lopez Priest: persona dossier for role, context, locale, tenant membership, and journey coverage.
59. [fellow-dr-tobias-klein.md](personas/fellow-dr-tobias-klein.md) - Fellow Dr Tobias Klein: persona dossier for role, context, locale, tenant membership, and journey coverage.
60. [finance-director-mei-ling-wu.md](personas/finance-director-mei-ling-wu.md) - Finance Director Mei Ling Wu: persona dossier for role, context, locale, tenant membership, and journey coverage.
61. [financial-analyst-wendy-lee.md](personas/financial-analyst-wendy-lee.md) - Financial Analyst Wendy Lee: persona dossier for role, context, locale, tenant membership, and journey coverage.
62. [hiroshi-tanaka.md](personas/hiroshi-tanaka.md) - Hiroshi Tanaka: persona dossier for role, context, locale, tenant membership, and journey coverage.
63. [hr-specialist-aoife-murphy.md](personas/hr-specialist-aoife-murphy.md) - Hr Specialist Aoife Murphy: persona dossier for role, context, locale, tenant membership, and journey coverage.
64. [hrbp-jamal-carter.md](personas/hrbp-jamal-carter.md) - Hrbp Jamal Carter: persona dossier for role, context, locale, tenant membership, and journey coverage.
65. [intern-manager-felicia-adamou.md](personas/intern-manager-felicia-adamou.md) - Intern Manager Felicia Adamou: persona dossier for role, context, locale, tenant membership, and journey coverage.
66. [internal-comms-lead-ji-ho-yoon.md](personas/internal-comms-lead-ji-ho-yoon.md) - Internal Comms Lead Ji Ho Yoon: persona dossier for role, context, locale, tenant membership, and journey coverage.
67. [investment-banker-yuna-ahn.md](personas/investment-banker-yuna-ahn.md) - Investment Banker Yuna Ahn: persona dossier for role, context, locale, tenant membership, and journey coverage.
68. [investor-lp-aanya-kapoor.md](personas/investor-lp-aanya-kapoor.md) - Investor Lp Aanya Kapoor: persona dossier for role, context, locale, tenant membership, and journey coverage.
69. [ir-manager-lev-kahn.md](personas/ir-manager-lev-kahn.md) - Ir Manager Lev Kahn: persona dossier for role, context, locale, tenant membership, and journey coverage.
70. [ir-specialist-unnamed.md](personas/ir-specialist-unnamed.md) - Ir Specialist Unnamed: persona dossier for role, context, locale, tenant membership, and journey coverage.
71. [it-manager-jamie-o-connor.md](personas/it-manager-jamie-o-connor.md) - It Manager Jamie O Connor: persona dossier for role, context, locale, tenant membership, and journey coverage.
72. [jordan-lee.md](personas/jordan-lee.md) - Jordan Lee: persona dossier for role, context, locale, tenant membership, and journey coverage.
73. [leave-specialist-margarethe-reinhart.md](personas/leave-specialist-margarethe-reinhart.md) - Leave Specialist Margarethe Reinhart: persona dossier for role, context, locale, tenant membership, and journey coverage.
74. [legal-counsel-anika-mehta.md](personas/legal-counsel-anika-mehta.md) - Legal Counsel Anika Mehta: persona dossier for role, context, locale, tenant membership, and journey coverage.
75. [legal-operations-stephen-park.md](personas/legal-operations-stephen-park.md) - Legal Operations Stephen Park: persona dossier for role, context, locale, tenant membership, and journey coverage.
76. [mailroom-hae-won-kim.md](personas/mailroom-hae-won-kim.md) - Mailroom Hae Won Kim: persona dossier for role, context, locale, tenant membership, and journey coverage.
77. [maintenance-tech-carlos-reyes-ii.md](personas/maintenance-tech-carlos-reyes-ii.md) - Maintenance Tech Carlos Reyes Ii: persona dossier for role, context, locale, tenant membership, and journey coverage.
78. [marcus-chen.md](personas/marcus-chen.md) - Marcus Chen: persona dossier for role, context, locale, tenant membership, and journey coverage.
79. [maria-santos.md](personas/maria-santos.md) - Maria Santos: persona dossier for role, context, locale, tenant membership, and journey coverage.
80. [marketing-manager-olu-adeyemi.md](personas/marketing-manager-olu-adeyemi.md) - Marketing Manager Olu Adeyemi: persona dossier for role, context, locale, tenant membership, and journey coverage.
81. [marketing-specialist-riya-sharma.md](personas/marketing-specialist-riya-sharma.md) - Marketing Specialist Riya Sharma: persona dossier for role, context, locale, tenant membership, and journey coverage.
82. [medical-resident-dr-sun-mi-kim.md](personas/medical-resident-dr-sun-mi-kim.md) - Medical Resident Dr Sun Mi Kim: persona dossier for role, context, locale, tenant membership, and journey coverage.
83. [ms-patel-teacher.md](personas/ms-patel-teacher.md) - Ms Patel Teacher: persona dossier for role, context, locale, tenant membership, and journey coverage.
84. [office-coordinator-phoebe-lin.md](personas/office-coordinator-phoebe-lin.md) - Office Coordinator Phoebe Lin: persona dossier for role, context, locale, tenant membership, and journey coverage.
85. [office-manager-priya-ramanathan.md](personas/office-manager-priya-ramanathan.md) - Office Manager Priya Ramanathan: persona dossier for role, context, locale, tenant membership, and journey coverage.
86. [officer-rodriguez-police.md](personas/officer-rodriguez-police.md) - Officer Rodriguez Police: persona dossier for role, context, locale, tenant membership, and journey coverage.
87. [ombudsperson-felix-tan.md](personas/ombudsperson-felix-tan.md) - Ombudsperson Felix Tan: persona dossier for role, context, locale, tenant membership, and journey coverage.
88. [outside-counsel-wei-yi-chen.md](personas/outside-counsel-wei-yi-chen.md) - Outside Counsel Wei Yi Chen: persona dossier for role, context, locale, tenant membership, and journey coverage.
89. [paralegal-tomas-novak.md](personas/paralegal-tomas-novak.md) - Paralegal Tomas Novak: persona dossier for role, context, locale, tenant membership, and journey coverage.
90. [pr-firm-beatriz-fernandez.md](personas/pr-firm-beatriz-fernandez.md) - Pr Firm Beatriz Fernandez: persona dossier for role, context, locale, tenant membership, and journey coverage.
91. [pr-manager-helena-sato.md](personas/pr-manager-helena-sato.md) - Pr Manager Helena Sato: persona dossier for role, context, locale, tenant membership, and journey coverage.
92. [print-operator-diana-lazar.md](personas/print-operator-diana-lazar.md) - Print Operator Diana Lazar: persona dossier for role, context, locale, tenant membership, and journey coverage.
93. [priya-krishnan.md](personas/priya-krishnan.md) - Priya Krishnan: persona dossier for role, context, locale, tenant membership, and journey coverage.
94. [procurement-manager-wei-liu.md](personas/procurement-manager-wei-liu.md) - Procurement Manager Wei Liu: persona dossier for role, context, locale, tenant membership, and journey coverage.
95. [procurement-specialist-beata-kowalski.md](personas/procurement-specialist-beata-kowalski.md) - Procurement Specialist Beata Kowalski: persona dossier for role, context, locale, tenant membership, and journey coverage.
96. [product-designer-akihiro-sato.md](personas/product-designer-akihiro-sato.md) - Product Designer Akihiro Sato: persona dossier for role, context, locale, tenant membership, and journey coverage.
97. [product-manager-lily-chang.md](personas/product-manager-lily-chang.md) - Product Manager Lily Chang: persona dossier for role, context, locale, tenant membership, and journey coverage.
98. [project-manager-soo-jin-park.md](personas/project-manager-soo-jin-park.md) - Project Manager Soo Jin Park: persona dossier for role, context, locale, tenant membership, and journey coverage.
99. [public-affairs-director-carlos-mendez.md](personas/public-affairs-director-carlos-mendez.md) - Public Affairs Director Carlos Mendez: persona dossier for role, context, locale, tenant membership, and journey coverage.
100. [receptionist-daria-volkova.md](personas/receptionist-daria-volkova.md) - Receptionist Daria Volkova: persona dossier for role, context, locale, tenant membership, and journey coverage.
101. [recruiter-marcus-iv.md](personas/recruiter-marcus-iv.md) - Recruiter Marcus Iv: persona dossier for role, context, locale, tenant membership, and journey coverage.
102. [recruiting-manager-hina-suzuki.md](personas/recruiting-manager-hina-suzuki.md) - Recruiting Manager Hina Suzuki: persona dossier for role, context, locale, tenant membership, and journey coverage.
103. [regulator-inspector-sergei-petrov.md](personas/regulator-inspector-sergei-petrov.md) - Regulator Inspector Sergei Petrov: persona dossier for role, context, locale, tenant membership, and journey coverage.
104. [retail-banker-sebastian-vega.md](personas/retail-banker-sebastian-vega.md) - Retail Banker Sebastian Vega: persona dossier for role, context, locale, tenant membership, and journey coverage.
105. [retirement-plan-admin-bryce-williams.md](personas/retirement-plan-admin-bryce-williams.md) - Retirement Plan Admin Bryce Williams: persona dossier for role, context, locale, tenant membership, and journey coverage.
106. [returning-intern-jia-han.md](personas/returning-intern-jia-han.md) - Returning Intern Jia Han: persona dossier for role, context, locale, tenant membership, and journey coverage.
107. [sales-ae-maya-lindqvist.md](personas/sales-ae-maya-lindqvist.md) - Sales Ae Maya Lindqvist: persona dossier for role, context, locale, tenant membership, and journey coverage.
108. [sales-manager-anthony-costa.md](personas/sales-manager-anthony-costa.md) - Sales Manager Anthony Costa: persona dossier for role, context, locale, tenant membership, and journey coverage.
109. [sam-okafor.md](personas/sam-okafor.md) - Sam Okafor: persona dossier for role, context, locale, tenant membership, and journey coverage.
110. [sarah-kim-delivery.md](personas/sarah-kim-delivery.md) - Sarah Kim Delivery: persona dossier for role, context, locale, tenant membership, and journey coverage.
111. [sdr-kofi-asante.md](personas/sdr-kofi-asante.md) - Sdr Kofi Asante: persona dossier for role, context, locale, tenant membership, and journey coverage.
112. [security-analyst-anna-petrova.md](personas/security-analyst-anna-petrova.md) - Security Analyst Anna Petrova: persona dossier for role, context, locale, tenant membership, and journey coverage.
113. [security-guard-stefan-kovacs.md](personas/security-guard-stefan-kovacs.md) - Security Guard Stefan Kovacs: persona dossier for role, context, locale, tenant membership, and journey coverage.
114. [software-engineer-hugo-tanaka.md](personas/software-engineer-hugo-tanaka.md) - Software Engineer Hugo Tanaka: persona dossier for role, context, locale, tenant membership, and journey coverage.
115. [strategic-advisor-rita-almeida.md](personas/strategic-advisor-rita-almeida.md) - Strategic Advisor Rita Almeida: persona dossier for role, context, locale, tenant membership, and journey coverage.
116. [summer-intern-priscilla-sharma.md](personas/summer-intern-priscilla-sharma.md) - Summer Intern Priscilla Sharma: persona dossier for role, context, locale, tenant membership, and journey coverage.
117. [support-rep-nadia-hassani.md](personas/support-rep-nadia-hassani.md) - Support Rep Nadia Hassani: persona dossier for role, context, locale, tenant membership, and journey coverage.
118. [sustainability-officer-aiko-brown.md](personas/sustainability-officer-aiko-brown.md) - Sustainability Officer Aiko Brown: persona dossier for role, context, locale, tenant membership, and journey coverage.
119. [tax-analyst-ji-sung-park.md](personas/tax-analyst-ji-sung-park.md) - Tax Analyst Ji Sung Park: persona dossier for role, context, locale, tenant membership, and journey coverage.
120. [tomas-garcia-jr-farmer.md](personas/tomas-garcia-jr-farmer.md) - Tomas Garcia Jr Farmer: persona dossier for role, context, locale, tenant membership, and journey coverage.
121. [tomas-garcia.md](personas/tomas-garcia.md) - Tomas Garcia: persona dossier for role, context, locale, tenant membership, and journey coverage.
122. [total-rewards-manager-nilufer-demir.md](personas/total-rewards-manager-nilufer-demir.md) - Total Rewards Manager Nilufer Demir: persona dossier for role, context, locale, tenant membership, and journey coverage.
123. [trader-mei-lin.md](personas/trader-mei-lin.md) - Trader Mei Lin: persona dossier for role, context, locale, tenant membership, and journey coverage.
124. [training-specialist-mehmet-yilmaz.md](personas/training-specialist-mehmet-yilmaz.md) - Training Specialist Mehmet Yilmaz: persona dossier for role, context, locale, tenant membership, and journey coverage.
125. [treasury-ops-sven-eriksson.md](personas/treasury-ops-sven-eriksson.md) - Treasury Ops Sven Eriksson: persona dossier for role, context, locale, tenant membership, and journey coverage.
126. [ux-researcher-adaeze-nwosu.md](personas/ux-researcher-adaeze-nwosu.md) - Ux Researcher Adaeze Nwosu: persona dossier for role, context, locale, tenant membership, and journey coverage.
127. [venture-partner-lucas-muller.md](personas/venture-partner-lucas-muller.md) - Venture Partner Lucas Muller: persona dossier for role, context, locale, tenant membership, and journey coverage.
128. [wealth-manager-aamir-khan.md](personas/wealth-manager-aamir-khan.md) - Wealth Manager Aamir Khan: persona dossier for role, context, locale, tenant membership, and journey coverage.
129. [wellness-program-manager-akira-sato.md](personas/wellness-program-manager-akira-sato.md) - Wellness Program Manager Akira Sato: persona dossier for role, context, locale, tenant membership, and journey coverage.
130. [yejin-park.md](personas/yejin-park.md) - Yejin Park: persona dossier for role, context, locale, tenant membership, and journey coverage.

<a id="section-6-standards"></a>
## §6 Standards

### Section Purpose

Standards are the cross-cutting rules for how docs, APIs, code, policy, tests, operations, names, localization, and review evidence are authored. They are normative and should be read before editing the corresponding surface, even when the change is documentation-only.

### Canonical Docs (Top 15)

1. [documentation rigor](standards/documentation-rigor.md) - intern-buildability and hyperscaler-grade depth bar.
2. [naming convention](standards/naming-convention-bnf-v4.md) - canonical naming convention BNF.
3. [i18n canonical](standards/i18n-canonical.md) - canonical base localization and i18n behavior.
4. [layer enum](standards/layer-enum-adr-0105.md) - layer enum companion to ADR-0105.
5. [doc style](standards/doc-style.md) - Diataxis, RFC-2119, and doc-class authoring style.
6. [clean architecture](standards/clean-architecture.md) - clean architecture standards for code layout.
7. [cloud-native infrastructure automation](standards/cloud-native-infrastructure-automation.md) - API-shaped Rust/config/controller/gate standard for core infrastructure automation.
8. [API design](standards/api-design.md) - API design discipline and interface expectations.
9. [OpenAPI authoring](standards/openapi-3-2-authoring.md) - OpenAPI 3.2 authoring standard.
10. [AsyncAPI authoring](standards/asyncapi-3-1-authoring.md) - AsyncAPI 3.1 authoring standard.
11. [proto3 authoring](standards/proto3-authoring.md) - proto3 authoring standard.
12. [Cedar policy discipline](standards/cedar-policy-discipline.md) - Cedar policy discipline and gate posture.
13. [observability](standards/observability.md) - OpenTelemetry, metrics, logs, traces, and audit event standards.
14. [testing](standards/testing.md) - test evidence and coverage bar.
15. [security review](standards/security-review.md) - security review standard.

### Related Sections

Related sections: [§0 Operating Map](#section-0-operating-map), [§2 Decisions](#section-2-decisions), [§7 Onboarding](#section-7-onboarding), [§11 Governance Crates](#section-11-governance-crates), [§12 Glossary](#section-12-glossary).

### When To Read

- Read this before changing any doc class, naming rule, API surface, event schema, Cedar policy, localization pack, or test standard.
- Read documentation-rigor before judging whether a hub, ADR, PRD, standard, runbook, or onboarding guide is substantive enough.
- Read naming and layer enum standards before adding crates, catalog records, or microservice docs.

### Standards Complete Shelf

1. [INDEX.md](standards/INDEX.md) - Index: normative rule shelf for implementation, docs, operations, or governance discipline.
2. [a11y-canonical.md](standards/a11y-canonical.md) - A11Y Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
3. [agent-instructions-discipline.md](standards/agent-instructions-discipline.md) - Agent Instructions Discipline: normative rule shelf for implementation, docs, operations, or governance discipline.
4. [agentic-dev-team-optimization.md](standards/agentic-dev-team-optimization.md) - Agentic Dev Team Optimization: normative rule shelf for implementation, docs, operations, or governance discipline.
5. [api-design.md](standards/api-design.md) - API Design: normative rule shelf for implementation, docs, operations, or governance discipline.
6. [api-surface-separation.md](standards/api-surface-separation.md) - API Surface Separation: normative rule shelf for implementation, docs, operations, or governance discipline.
7. [asyncapi-3-1-authoring.md](standards/asyncapi-3-1-authoring.md) - Asyncapi 3 1 Authoring: normative rule shelf for implementation, docs, operations, or governance discipline.
8. [authz-tier-boundaries.md](standards/authz-tier-boundaries.md) - Authz Tier Boundaries: normative rule shelf for implementation, docs, operations, or governance discipline.
9. [autonomy-ceiling.md](standards/autonomy-ceiling.md) - Autonomy Ceiling: normative rule shelf for implementation, docs, operations, or governance discipline.
10. [backup-canonical.md](standards/backup-canonical.md) - Backup Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
11. [brand-voice.md](standards/brand-voice.md) - Brand Voice: normative rule shelf for implementation, docs, operations, or governance discipline.
12. [brownout-degradation-signal.md](standards/brownout-degradation-signal.md) - Brownout Degradation Signal: normative rule shelf for implementation, docs, operations, or governance discipline.
13. [capability-authoring.md](standards/capability-authoring.md) - Capability Authoring: normative rule shelf for implementation, docs, operations, or governance discipline.
14. [capability-tier-matrix.md](standards/capability-tier-matrix.md) - Capability Tier Matrix: normative rule shelf for implementation, docs, operations, or governance discipline.
15. [cedar-policy-authoring.md](standards/cedar-policy-authoring.md) - Cedar Policy Authoring: normative rule shelf for implementation, docs, operations, or governance discipline.
16. [cedar-policy-discipline.md](standards/cedar-policy-discipline.md) - Cedar Policy Discipline: normative rule shelf for implementation, docs, operations, or governance discipline.
17. [ci-lanes.md](standards/ci-lanes.md) - Ci Lanes: normative rule shelf for implementation, docs, operations, or governance discipline.
18. [claude-code-harness.md](standards/claude-code-harness.md) - **RETIRED tombstone** (ADR-0619 / RR-HARNESS-0619): former Claude Code / external-harness brand standard; live authority is AGENTS.md + ADR-0515 + optional `.grok/` mm-delivery kit (not merge authority).
19. [clean-architecture.md](standards/clean-architecture.md) - Clean Architecture: normative rule shelf for implementation, docs, operations, or governance discipline.
20. [cloud-native-infrastructure-automation.md](standards/cloud-native-infrastructure-automation.md) - Cloud Native Infrastructure Automation: normative rule shelf for implementation, docs, operations, or governance discipline.
21. [code-review.md](standards/code-review.md) - Code Review: normative rule shelf for implementation, docs, operations, or governance discipline.
22. [code-style-rust.md](standards/code-style-rust.md) - Code Style Rust: normative rule shelf for implementation, docs, operations, or governance discipline.
23. [code-style.md](standards/code-style.md) - Code Style: normative rule shelf for implementation, docs, operations, or governance discipline.
24. [commit-message.md](standards/commit-message.md) - Commit Message: normative rule shelf for implementation, docs, operations, or governance discipline.
25. [compliance-evidence-automation.md](standards/compliance-evidence-automation.md) - Compliance Evidence Automation: normative rule shelf for implementation, docs, operations, or governance discipline.
26. [container-image-convention.md](standards/container-image-convention.md) - Container Image Convention: normative rule shelf for implementation, docs, operations, or governance discipline.
27. [crate-naming-convention.md](standards/crate-naming-convention.md) - Crate Naming Convention: normative rule shelf for implementation, docs, operations, or governance discipline.
28. [cross-microservice-latency-budget.md](standards/cross-microservice-latency-budget.md) - Cross Microservice Latency Budget: normative rule shelf for implementation, docs, operations, or governance discipline.
29. [cursor-pagination-canonical.md](standards/cursor-pagination-canonical.md) - Cursor Pagination Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
30. [data-class.md](standards/data-class.md) - Data Class: normative rule shelf for implementation, docs, operations, or governance discipline.
31. [dependency-policy.md](standards/dependency-policy.md) - Dependency Policy: normative rule shelf for implementation, docs, operations, or governance discipline.
32. [design-doc-template.md](standards/design-doc-template.md) - Design Doc Template: normative rule shelf for implementation, docs, operations, or governance discipline.
33. [doc-style.md](standards/doc-style.md) - Doc Style: normative rule shelf for implementation, docs, operations, or governance discipline.
34. [documentation-rigor.md](standards/documentation-rigor.md) - Documentation Rigor: normative rule shelf for implementation, docs, operations, or governance discipline.
35. [dr-business-continuity.md](standards/dr-business-continuity.md) - Dr Business Continuity: normative rule shelf for implementation, docs, operations, or governance discipline.
36. [emoji-sticker-reaction-system.md](standards/emoji-sticker-reaction-system.md) - Emoji Sticker Reaction System: normative rule shelf for implementation, docs, operations, or governance discipline.
37. [error-handling.md](standards/error-handling.md) - Error Handling: normative rule shelf for implementation, docs, operations, or governance discipline.
38. [event-schema-versioning-canonical.md](standards/event-schema-versioning-canonical.md) - Event Schema Versioning Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
39. [finops-cost-attribution-canonical.md](standards/finops-cost-attribution-canonical.md) - Finops Cost Attribution Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
40. [finops-cost-attribution.md](standards/finops-cost-attribution.md) - Finops Cost Attribution: normative rule shelf for implementation, docs, operations, or governance discipline.
41. [fintech-compliance.md](standards/fintech-compliance.md) - Fintech Compliance: normative rule shelf for implementation, docs, operations, or governance discipline.
42. [fips-hsm-substrate-root-signing.md](standards/fips-hsm-substrate-root-signing.md) - Fips Hsm Substrate Root Signing: normative rule shelf for implementation, docs, operations, or governance discipline.
43. [git-workflow.md](standards/git-workflow.md) - Git Workflow: normative rule shelf for implementation, docs, operations, or governance discipline.
44. [gitops-iac-cluster-tier-boundaries.md](standards/gitops-iac-cluster-tier-boundaries.md) - Gitops Iac Cluster Tier Boundaries: normative rule shelf for implementation, docs, operations, or governance discipline.
45. [graceful-shutdown-canonical.md](standards/graceful-shutdown-canonical.md) - Graceful Shutdown Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
46. [helm-chart-convention.md](standards/helm-chart-convention.md) - Helm Chart Convention: normative rule shelf for implementation, docs, operations, or governance discipline.
47. [hyperscaler-best-practices.md](standards/hyperscaler-best-practices.md) - Hyperscaler Best Practices: normative rule shelf for implementation, docs, operations, or governance discipline.
48. [hyperscaler-invariant-conformance.md](standards/hyperscaler-invariant-conformance.md) - Hyperscaler Invariant Conformance: normative rule shelf for implementation, docs, operations, or governance discipline.
49. [i18n-canonical.md](standards/i18n-canonical.md) - I18N Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
50. [idempotency-keys-canonical.md](standards/idempotency-keys-canonical.md) - Idempotency Keys Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
51. [identity-vendor-isolation.md](standards/identity-vendor-isolation.md) - Identity Vendor Isolation: normative rule shelf for implementation, docs, operations, or governance discipline.
52. [image-discipline.md](standards/image-discipline.md) - Image Discipline: normative rule shelf for implementation, docs, operations, or governance discipline.
53. [image-signing-canonical.md](standards/image-signing-canonical.md) - Image Signing Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
54. [incident-severity.md](standards/incident-severity.md) - Incident Severity: normative rule shelf for implementation, docs, operations, or governance discipline.
55. [layer-enum-adr-0105.md](standards/layer-enum-adr-0105.md) - Layer Enum ADR 0105: normative rule shelf for implementation, docs, operations, or governance discipline.
56. [locale-routing.md](standards/locale-routing.md) - Locale Routing: normative rule shelf for implementation, docs, operations, or governance discipline.
57. [logging-tracing.md](standards/logging-tracing.md) - Logging Tracing: normative rule shelf for implementation, docs, operations, or governance discipline.
58. [lts-versions-verified.md](standards/lts-versions-verified.md) - Lts Versions Verified: normative rule shelf for implementation, docs, operations, or governance discipline.
59. [m02-exit-gate-validators.md](standards/m02-exit-gate-validators.md) - M02 Exit Gate Validators: normative rule shelf for implementation, docs, operations, or governance discipline.
60. [messenger-e2e-encryption-mls.md](standards/messenger-e2e-encryption-mls.md) - Messenger E2E Encryption Mls: normative rule shelf for implementation, docs, operations, or governance discipline.
61. [migration-playbook.md](standards/migration-playbook.md) - Migration Playbook: normative rule shelf for implementation, docs, operations, or governance discipline.
62. [mls-rfc-9420-conformance.md](standards/mls-rfc-9420-conformance.md) - Mls Rfc 9420 Conformance: normative rule shelf for implementation, docs, operations, or governance discipline.
63. [multi-agent-tool-map.md](standards/multi-agent-tool-map.md) - Multi Agent Tool Map: normative rule shelf for implementation, docs, operations, or governance discipline.
64. [naming-convention-bnf-v4.md](standards/naming-convention-bnf-v4.md) - Naming Convention Bnf V4: normative rule shelf for implementation, docs, operations, or governance discipline.
65. [observability-slo.md](standards/observability-slo.md) - Observability SLO: normative rule shelf for implementation, docs, operations, or governance discipline.
66. [observability.md](standards/observability.md) - Observability: normative rule shelf for implementation, docs, operations, or governance discipline.
67. [on-call.md](standards/on-call.md) - On Call: normative rule shelf for implementation, docs, operations, or governance discipline.
68. [ontology-projection-substrate.md](standards/ontology-projection-substrate.md) - Ontology Projection Substrate: normative rule shelf for implementation, docs, operations, or governance discipline.
69. [openapi-3-2-authoring.md](standards/openapi-3-2-authoring.md) - Openapi 3 2 Authoring: normative rule shelf for implementation, docs, operations, or governance discipline.
70. [openslo-authoring.md](standards/openslo-authoring.md) - Openslo Authoring: normative rule shelf for implementation, docs, operations, or governance discipline.
71. [outbox-pattern-canonical.md](standards/outbox-pattern-canonical.md) - Outbox Pattern Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
72. [per-tenant-resource-quotas-canonical.md](standards/per-tenant-resource-quotas-canonical.md) - Per Tenant Resource Quotas Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
73. [plugin-authoring.md](standards/plugin-authoring.md) - Plugin Authoring: normative rule shelf for implementation, docs, operations, or governance discipline.
74. [postmortem-template.md](standards/postmortem-template.md) - Postmortem Template: normative rule shelf for implementation, docs, operations, or governance discipline.
75. [prevention-doctrine.md](standards/prevention-doctrine.md) - Prevention Doctrine: normative rule shelf for implementation, docs, operations, or governance discipline.
76. [prfaq-template.md](standards/prfaq-template.md) - Prfaq Template: normative rule shelf for implementation, docs, operations, or governance discipline.
77. [privacy-review.md](standards/privacy-review.md) - Privacy Review: normative rule shelf for implementation, docs, operations, or governance discipline.
78. [proto3-authoring.md](standards/proto3-authoring.md) - Proto3 Authoring: normative rule shelf for implementation, docs, operations, or governance discipline.
79. [realtime-transport-tier.md](standards/realtime-transport-tier.md) - Realtime Transport Tier: normative rule shelf for implementation, docs, operations, or governance discipline.
80. [regulatory-pack-authzpolicy-overlays.md](standards/regulatory-pack-authzpolicy-overlays.md) - Regulatory Pack Authzpolicy Overlays: normative rule shelf for implementation, docs, operations, or governance discipline.
81. [release-management.md](standards/release-management.md) - Release Management: normative rule shelf for implementation, docs, operations, or governance discipline.
82. [release.md](standards/release.md) - Release: normative rule shelf for implementation, docs, operations, or governance discipline.
83. [request-id-canonical.md](standards/request-id-canonical.md) - Request Id Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
84. [rtl-rendering.md](standards/rtl-rendering.md) - Rtl Rendering: normative rule shelf for implementation, docs, operations, or governance discipline.
85. [saga-compensation-policy.md](standards/saga-compensation-policy.md) - Saga Compensation Policy: normative rule shelf for implementation, docs, operations, or governance discipline.
86. [schema-migration.md](standards/schema-migration.md) - Schema Migration: normative rule shelf for implementation, docs, operations, or governance discipline.
87. [security-review.md](standards/security-review.md) - Security Review: normative rule shelf for implementation, docs, operations, or governance discipline.
88. [sovereign-cloud-overlay.md](standards/sovereign-cloud-overlay.md) - Sovereign Cloud Overlay: normative rule shelf for implementation, docs, operations, or governance discipline.
89. [step-up-auth-classes.md](standards/step-up-auth-classes.md) - Step Up Auth Classes: normative rule shelf for implementation, docs, operations, or governance discipline.
90. [stream-processing-rubric.md](standards/stream-processing-rubric.md) - Stream Processing Rubric: normative rule shelf for implementation, docs, operations, or governance discipline.
91. [tenant-lifecycle.md](standards/tenant-lifecycle.md) - Tenant Lifecycle: normative rule shelf for implementation, docs, operations, or governance discipline.
92. [testing.md](standards/testing.md) - Testing: normative rule shelf for implementation, docs, operations, or governance discipline.
93. [throttling-tiers.md](standards/throttling-tiers.md) - Throttling Tiers: normative rule shelf for implementation, docs, operations, or governance discipline.
94. [timescaledb-adoption.md](standards/timescaledb-adoption.md) - Timescaledb Adoption: normative rule shelf for implementation, docs, operations, or governance discipline.
95. [trace-sampling-tier.md](standards/trace-sampling-tier.md) - Trace Sampling Tier: normative rule shelf for implementation, docs, operations, or governance discipline.
96. [twelve-factor-adoption.md](standards/twelve-factor-adoption.md) - Twelve Factor Adoption: normative rule shelf for implementation, docs, operations, or governance discipline.
97. [ux-best-practices.md](standards/ux-best-practices.md) - Ux Best Practices: normative rule shelf for implementation, docs, operations, or governance discipline.
98. [voice-video-call-architecture.md](standards/voice-video-call-architecture.md) - Voice Video Call Architecture: normative rule shelf for implementation, docs, operations, or governance discipline.
99. [wasm-runtime-canonical.md](standards/wasm-runtime-canonical.md) - Wasm Runtime Canonical: normative rule shelf for implementation, docs, operations, or governance discipline.
100. [wcag-2-2-aa-checklist.md](standards/wcag-2-2-aa-checklist.md) - Wcag 2 2 Aa Checklist: normative rule shelf for implementation, docs, operations, or governance discipline.
101. [workflow-substrate-engine.md](standards/workflow-substrate-engine.md) - Workflow Substrate Engine: normative rule shelf for implementation, docs, operations, or governance discipline.
102. [workflow-vs-direct-grpc-rubric.md](standards/workflow-vs-direct-grpc-rubric.md) - Workflow Vs Direct Grpc Rubric: normative rule shelf for implementation, docs, operations, or governance discipline.

<a id="section-7-onboarding"></a>
## §7 Onboarding

### Section Purpose

Onboarding guides turn the standards and corpus into role-specific first-day, first-week, first-month, or first-quarter paths. Use this section to make the corpus executable for interns, engineers, product managers, SREs, security, compliance, customer success, and AI platform contributors.

### Canonical Docs (Top 12)

1. [intern day one](onboarding/intern-day-one.md) - Day 1 intern path and first verifiable artifact.
2. [intern week one](onboarding/intern-week-one.md) - Week 1 intern buildability path.
3. [intern month one](onboarding/intern-month-one.md) - Month 1 intern ownership path.
4. [doctrine bootcamp](onboarding/doctrine-bootcamp-2026-05-21.md) - doctrine bootcamp for project vocabulary and constraints.
5. [frontend engineer week one](onboarding/frontend-engineer-week-one.md) - frontend engineer week-one guide.
6. [platform SWE month one](onboarding/swe-platform-engineer-month-one.md) - platform software engineer month-one guide.
7. [AI platform month one](onboarding/ai-platform-engineer-month-one.md) - AI platform engineer month-one guide.
8. [security month one](onboarding/security-engineer-month-one.md) - security engineer month-one guide.
9. [SRE on-call week one](onboarding/sre-on-call-week-one.md) - SRE on-call week-one guide.
10. [compliance quarter one](onboarding/compliance-officer-quarter-one.md) - compliance officer quarter-one guide.
11. [CS quarter one](onboarding/customer-success-quarter-one.md) - customer success quarter-one guide.
12. [PM month one](onboarding/product-manager-month-one.md) - product manager month-one guide.

### Related Sections

Related sections: [§0 Operating Map](#section-0-operating-map), [§5 Personas](#section-5-personas), [§6 Standards](#section-6-standards), [§8 Governance Pipeline](#section-8-governance-pipeline), [§13 Wave Sequence](#section-13-wave-sequence).

### When To Read

- Read this when someone joins a role and needs a verified artifact path, not a general orientation.
- Read intern guides when testing whether docs meet the cold-start buildability bar.
- Read doctrine bootcamp when terminology or authority rules are causing confusion.
- Read role guides before assigning first slices, on-call shadows, or compliance evidence work.

### Onboarding Library

1. [ai-platform-engineer-month-one.md](onboarding/ai-platform-engineer-month-one.md) - AI Platform Engineer Month One: role-specific onboarding path with verifiable first artifacts.
2. [compliance-officer-quarter-one.md](onboarding/compliance-officer-quarter-one.md) - Compliance Officer Quarter One: role-specific onboarding path with verifiable first artifacts.
3. [customer-success-quarter-one.md](onboarding/customer-success-quarter-one.md) - Customer Success Quarter One: role-specific onboarding path with verifiable first artifacts.
4. [doctrine-bootcamp-2026-05-21.md](onboarding/doctrine-bootcamp-2026-05-21.md) - Doctrine Bootcamp 2026 05 21: role-specific onboarding path with verifiable first artifacts.
5. [frontend-engineer-week-one.md](onboarding/frontend-engineer-week-one.md) - Frontend Engineer Week One: role-specific onboarding path with verifiable first artifacts.
6. [intern-day-one.md](onboarding/intern-day-one.md) - Intern Day One: role-specific onboarding path with verifiable first artifacts.
7. [intern-month-one.md](onboarding/intern-month-one.md) - Intern Month One: role-specific onboarding path with verifiable first artifacts.
8. [intern-week-one.md](onboarding/intern-week-one.md) - Intern Week One: role-specific onboarding path with verifiable first artifacts.
9. [product-manager-month-one.md](onboarding/product-manager-month-one.md) - Product Manager Month One: role-specific onboarding path with verifiable first artifacts.
10. [security-engineer-month-one.md](onboarding/security-engineer-month-one.md) - Security Engineer Month One: role-specific onboarding path with verifiable first artifacts.
11. [sre-on-call-week-one.md](onboarding/sre-on-call-week-one.md) - Sre On Call Week One: role-specific onboarding path with verifiable first artifacts.
12. [swe-platform-engineer-month-one.md](onboarding/swe-platform-engineer-month-one.md) - Swe Platform Engineer Month One: role-specific onboarding path with verifiable first artifacts.

<a id="section-8-governance-pipeline"></a>
## §8 Governance Pipeline

### Section Purpose

Governance pipeline docs explain how agentic work is claimed, verified, reviewed, promoted, supervised, and evidenced. This section ties the ADR-0110..0116 sequence to the **intelligence** capability surfaces and `specs/microservices/foundry.json` retirement tombstone (Foundry product shelf deleted; not live authority). `microservices/intelligence/spec/` remains absent in the current checkout.

### Canonical Docs (Top 15)

1. [ADR-0110](decisions/ADR-0110-changeset-state-machine.md) - ChangeSet state machine for pipeline control.
2. [ADR-0111](decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md) - merge queue projected-state and fix-at-any-stage doctrine.
3. [ADR-0112](decisions/ADR-0112-webhook-driven-intelligence-agent-invocation.md) - webhook-driven Foundry agent invocation.
4. [ADR-0113](decisions/ADR-0113-vcs-orchestrator-end-to-end.md) - end-to-end VCS orchestrator.
5. [ADR-0116](decisions/ADR-0116-retire-external-agent-coordination-tooling.md) - retirement of external agent coordination tooling.
6. [microservices intelligence PRD](../microservices/intelligence/PRD.md) - microservice PRD (intelligence; Foundry brand retired).
7. [microservices intelligence architecture](../microservices/intelligence/ARCHITECTURE.md) - microservice architecture for intelligence.
8. [intelligence manifest](../microservices/intelligence/manifest.json) - intelligence microservice manifest.
9. [claim done verify promote tutorial](../microservices/intelligence/tutorials/claim-work-done-verify-promote-cycle.md) - tutorial for claim / done / verify / promote cycle.
10. [intelligence service ADR](../microservices/intelligence/decisions/ADR-FND-001-agentic-claim-isolation-vs-shared-lock-cedar-gate.md) - service ADR for claim isolation and Cedar gate.

### Related Sections

Related sections: [§2 Decisions](#section-2-decisions), [§3 Products](#section-3-products), [§6 Standards](#section-6-standards), [§10 Capability Tiers](#section-10-capability-tiers), [§11 Governance Crates](#section-11-governance-crates).

### When To Read

- Read this before changing Oya VCS, claim/verify/done/promote semantics, merge queue behavior, webhook invocation, or agent supervision.
- Read this before implementing intelligence capability, evidence, guardrails, provider-router, runtime, or supervisor slices.
- Read this when a task mentions pipeline, ChangeSet, admission, promotion, or agentic evidence (Foundry brand retired).
- Read the intelligence PRD, architecture, manifest, and service ADRs instead of linking to the absent `microservices/intelligence/spec/` directory; do not cite deleted Foundry product shelf paths.

### Foundry Docs And Microservice Entry Points

> `docs/foundry/` deleted permanently (Amendment B) — brand residue; live authority is intelligence ADRs / microservice docs below.

1. [microservices/intelligence/PRD.md](../microservices/intelligence/PRD.md) - PRD: Foundry implementation, operations, policy, catalog, or rollout artifact.
2. [microservices/intelligence/ARCHITECTURE.md](../microservices/intelligence/ARCHITECTURE.md) - Architecture: Foundry implementation, operations, policy, catalog, or rollout artifact.
3. [microservices/intelligence/manifest.json](../microservices/intelligence/manifest.json) - Manifest: Foundry implementation, operations, policy, catalog, or rollout artifact.
4. [microservices/intelligence/PHASE-01-FOUNDRY-FOUNDATION.md](../microservices/intelligence/PHASE-01-FOUNDRY-FOUNDATION.md) - Phase 01 Foundry Foundation: Foundry implementation, operations, policy, catalog, or rollout artifact.
5. [microservices/intelligence/PHASE-02-FOUNDRY-DATA-SUBSTRATE-ADDENDUM.md](../microservices/intelligence/PHASE-02-FOUNDRY-DATA-SUBSTRATE-ADDENDUM.md) - Phase 02 Foundry Data Substrate Addendum: Foundry implementation, operations, policy, catalog, or rollout artifact.
6. [microservices/intelligence/capability-tiers/tier-matrix.md](../microservices/intelligence/capability-tiers/tier-matrix.md) - Tier Matrix: Foundry implementation, operations, policy, catalog, or rollout artifact.
7. [microservices/intelligence/tutorials/claim-work-done-verify-promote-cycle.md](../microservices/intelligence/tutorials/claim-work-done-verify-promote-cycle.md) - Claim Work Done Verify Promote Cycle: Foundry implementation, operations, policy, catalog, or rollout artifact.
8. [microservices/intelligence/reference-implementations/claim-protocol-rust-sdk.md](../microservices/intelligence/reference-implementations/claim-protocol-rust-sdk.md) - Claim Protocol Rust Sdk: Foundry implementation, operations, policy, catalog, or rollout artifact.
9. [microservices/intelligence/benchmarks/foundry-vs-github-merge-queue-vs-bors-vs-spr-vs-shipit.md](../microservices/intelligence/benchmarks/foundry-vs-github-merge-queue-vs-bors-vs-spr-vs-shipit.md) - Foundry Vs Github Merge Queue Vs Bors Vs Spr Vs Shipit: Foundry implementation, operations, policy, catalog, or rollout artifact.
10. [microservices/intelligence/decisions/ADR-FND-001-agentic-claim-isolation-vs-shared-lock-cedar-gate.md](../microservices/intelligence/decisions/ADR-FND-001-agentic-claim-isolation-vs-shared-lock-cedar-gate.md) - ADR Fnd 001 Agentic Claim Isolation Vs Shared Lock Cedar Gate: Foundry implementation, operations, policy, catalog, or rollout artifact.
11. [microservices/intelligence/decisions/SVC-ADR-WASM-001-wasmtime-canonical-foundry.md](../microservices/intelligence/decisions/SVC-ADR-WASM-001-wasmtime-canonical-foundry.md) - Svc ADR Wasm 001 Wasmtime Canonical Foundry: Foundry implementation, operations, policy, catalog, or rollout artifact.

### Foundry Implementation Plan Index

1. [IP-001-runtime-runtime-cluster-iac.md](../microservices/intelligence/IP-001-runtime-runtime-cluster-iac.md) - Ip 001 Runtime Runtime Cluster Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
2. [IP-002-runtime-redis-and-postgres-baseline.md](../microservices/intelligence/IP-002-runtime-redis-and-postgres-baseline.md) - Ip 002 Runtime Redis And Postgres Baseline: Foundry implementation, operations, policy, catalog, or rollout artifact.
3. [IP-003-runtime-capability-executor-kernel.md](../microservices/intelligence/IP-003-runtime-capability-executor-kernel.md) - Ip 003 Runtime Capability Executor Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
4. [IP-004-runtime-capability-executor-domain-and-usecase.md](../microservices/intelligence/IP-004-runtime-capability-executor-domain-and-usecase.md) - Ip 004 Runtime Capability Executor Domain And Usecase: Foundry implementation, operations, policy, catalog, or rollout artifact.
5. [IP-005-runtime-capability-registry-cache-stack.md](../microservices/intelligence/IP-005-runtime-capability-registry-cache-stack.md) - Ip 005 Runtime Capability Registry Cache Stack: Foundry implementation, operations, policy, catalog, or rollout artifact.
6. [IP-006-runtime-session-state-stack.md](../microservices/intelligence/IP-006-runtime-session-state-stack.md) - Ip 006 Runtime Session State Stack: Foundry implementation, operations, policy, catalog, or rollout artifact.
7. [IP-007-runtime-invocation-orchestrator-stack.md](../microservices/intelligence/IP-007-runtime-invocation-orchestrator-stack.md) - Ip 007 Runtime Invocation Orchestrator Stack: Foundry implementation, operations, policy, catalog, or rollout artifact.
8. [IP-008-runtime-runtime-pool-stack.md](../microservices/intelligence/IP-008-runtime-runtime-pool-stack.md) - Ip 008 Runtime Runtime Pool Stack: Foundry implementation, operations, policy, catalog, or rollout artifact.
9. [IP-009-runtime-capability-executor-api-and-rest.md](../microservices/intelligence/IP-009-runtime-capability-executor-api-and-rest.md) - Ip 009 Runtime Capability Executor API And Rest: Foundry implementation, operations, policy, catalog, or rollout artifact.
10. [IP-010-runtime-capability-executor-sdk.md](../microservices/intelligence/IP-010-runtime-capability-executor-sdk.md) - Ip 010 Runtime Capability Executor Sdk: Foundry implementation, operations, policy, catalog, or rollout artifact.
11. [IP-011-runtime-capability-executor-app.md](../microservices/intelligence/IP-011-runtime-capability-executor-app.md) - Ip 011 Runtime Capability Executor App: Foundry implementation, operations, policy, catalog, or rollout artifact.
12. [IP-012-runtime-autonomy-tier-gate.md](../microservices/intelligence/IP-012-runtime-autonomy-tier-gate.md) - Ip 012 Runtime Autonomy Tier Gate: Foundry implementation, operations, policy, catalog, or rollout artifact.
13. [IP-013-runtime-dsr-cascade-session-handler.md](../microservices/intelligence/IP-013-runtime-dsr-cascade-session-handler.md) - Ip 013 Runtime DSR Cascade Session Handler: Foundry implementation, operations, policy, catalog, or rollout artifact.
14. [IP-014-runtime-runtime-self-slo-manifests.md](../microservices/intelligence/IP-014-runtime-runtime-self-slo-manifests.md) - Ip 014 Runtime Runtime Self SLO Manifests: Foundry implementation, operations, policy, catalog, or rollout artifact.
15. [IP-015-runtime-hg-fr-hyperscaler-gate-registration.md](../microservices/intelligence/IP-015-runtime-hg-fr-hyperscaler-gate-registration.md) - Ip 015 Runtime Hg Fr Hyperscaler Gate Registration: Foundry implementation, operations, policy, catalog, or rollout artifact.
16. [IP-016-supervisor-postgres-layer-a-iac.md](../microservices/intelligence/IP-016-supervisor-postgres-layer-a-iac.md) - Ip 016 Supervisor Postgres Layer A Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
17. [IP-017-supervisor-redis-layer-a-iac.md](../microservices/intelligence/IP-017-supervisor-redis-layer-a-iac.md) - Ip 017 Supervisor Redis Layer A Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
18. [IP-018-supervisor-k8s-operator-iac.md](../microservices/intelligence/IP-018-supervisor-k8s-operator-iac.md) - Ip 018 Supervisor K8S Operator Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
19. [IP-019-supervisor-agent-fleet-lifecycle-kernel.md](../microservices/intelligence/IP-019-supervisor-agent-fleet-lifecycle-kernel.md) - Ip 019 Supervisor Agent Fleet Lifecycle Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
20. [IP-020-supervisor-autonomy-policy-enforcement.md](../microservices/intelligence/IP-020-supervisor-autonomy-policy-enforcement.md) - Ip 020 Supervisor Autonomy Policy Enforcement: Foundry implementation, operations, policy, catalog, or rollout artifact.
21. [IP-021-supervisor-capability-deployment.md](../microservices/intelligence/IP-021-supervisor-capability-deployment.md) - Ip 021 Supervisor Capability Deployment: Foundry implementation, operations, policy, catalog, or rollout artifact.
22. [IP-022-supervisor-supervision-event-bus.md](../microservices/intelligence/IP-022-supervisor-supervision-event-bus.md) - Ip 022 Supervisor Supervision Event Bus: Foundry implementation, operations, policy, catalog, or rollout artifact.
23. [IP-023-supervisor-kill-switch-engage-state.md](../microservices/intelligence/IP-023-supervisor-kill-switch-engage-state.md) - Ip 023 Supervisor Kill Switch Engage State: Foundry implementation, operations, policy, catalog, or rollout artifact.
24. [IP-024-supervisor-kill-switch-propagation.md](../microservices/intelligence/IP-024-supervisor-kill-switch-propagation.md) - Ip 024 Supervisor Kill Switch Propagation: Foundry implementation, operations, policy, catalog, or rollout artifact.
25. [IP-025-supervisor-fleet-state-postgres-adapter.md](../microservices/intelligence/IP-025-supervisor-fleet-state-postgres-adapter.md) - Ip 025 Supervisor Fleet State Postgres Adapter: Foundry implementation, operations, policy, catalog, or rollout artifact.
26. [IP-026-supervisor-rest-api.md](../microservices/intelligence/IP-026-supervisor-rest-api.md) - Ip 026 Supervisor Rest API: Foundry implementation, operations, policy, catalog, or rollout artifact.
27. [IP-027-supervisor-supervisor-self-slos.md](../microservices/intelligence/IP-027-supervisor-supervisor-self-slos.md) - Ip 027 Supervisor Supervisor Self SLOs: Foundry implementation, operations, policy, catalog, or rollout artifact.
28. [IP-028-supervisor-sdk-rust-and-ts.md](../microservices/intelligence/IP-028-supervisor-sdk-rust-and-ts.md) - Ip 028 Supervisor Sdk Rust And Ts: Foundry implementation, operations, policy, catalog, or rollout artifact.
29. [IP-029-supervisor-app-composition-root.md](../microservices/intelligence/IP-029-supervisor-app-composition-root.md) - Ip 029 Supervisor App Composition Root: Foundry implementation, operations, policy, catalog, or rollout artifact.
30. [IP-030-supervisor-e2e-drills-and-dashboards.md](../microservices/intelligence/IP-030-supervisor-e2e-drills-and-dashboards.md) - Ip 030 Supervisor E2E Drills And Dashboards: Foundry implementation, operations, policy, catalog, or rollout artifact.
31. [IP-031-eval-layer-a-gpu-runner-pool-iac.md](../microservices/intelligence/IP-031-eval-layer-a-gpu-runner-pool-iac.md) - Ip 031 Eval Layer A Gpu Runner Pool Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
32. [IP-032-eval-layer-a-postgres-clickhouse-golden-store-iac.md](../microservices/intelligence/IP-032-eval-layer-a-postgres-clickhouse-golden-store-iac.md) - Ip 032 Eval Layer A Postgres Clickhouse Golden Store Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
33. [IP-033-eval-eval-runner-kernel.md](../microservices/intelligence/IP-033-eval-eval-runner-kernel.md) - Ip 033 Eval Eval Runner Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
34. [IP-034-eval-eval-runner-domain.md](../microservices/intelligence/IP-034-eval-eval-runner-domain.md) - Ip 034 Eval Eval Runner Domain: Foundry implementation, operations, policy, catalog, or rollout artifact.
35. [IP-035-eval-eval-runner-usecase.md](../microservices/intelligence/IP-035-eval-eval-runner-usecase.md) - Ip 035 Eval Eval Runner Usecase: Foundry implementation, operations, policy, catalog, or rollout artifact.
36. [IP-036-eval-eval-runner-api.md](../microservices/intelligence/IP-036-eval-eval-runner-api.md) - Ip 036 Eval Eval Runner API: Foundry implementation, operations, policy, catalog, or rollout artifact.
37. [IP-037-eval-eval-runner-adapter.md](../microservices/intelligence/IP-037-eval-eval-runner-adapter.md) - Ip 037 Eval Eval Runner Adapter: Foundry implementation, operations, policy, catalog, or rollout artifact.
38. [IP-038-eval-eval-runner-adapter-s3.md](../microservices/intelligence/IP-038-eval-eval-runner-adapter-s3.md) - Ip 038 Eval Eval Runner Adapter S3: Foundry implementation, operations, policy, catalog, or rollout artifact.
39. [IP-039-eval-eval-runner-adapter-gpu.md](../microservices/intelligence/IP-039-eval-eval-runner-adapter-gpu.md) - Ip 039 Eval Eval Runner Adapter Gpu: Foundry implementation, operations, policy, catalog, or rollout artifact.
40. [IP-040-eval-eval-runner-rest.md](../microservices/intelligence/IP-040-eval-eval-runner-rest.md) - Ip 040 Eval Eval Runner Rest: Foundry implementation, operations, policy, catalog, or rollout artifact.
41. [IP-041-eval-eval-runner-worker.md](../microservices/intelligence/IP-041-eval-eval-runner-worker.md) - Ip 041 Eval Eval Runner Worker: Foundry implementation, operations, policy, catalog, or rollout artifact.
42. [IP-042-eval-eval-runner-sdk.md](../microservices/intelligence/IP-042-eval-eval-runner-sdk.md) - Ip 042 Eval Eval Runner Sdk: Foundry implementation, operations, policy, catalog, or rollout artifact.
43. [IP-043-eval-eval-runner-app.md](../microservices/intelligence/IP-043-eval-eval-runner-app.md) - Ip 043 Eval Eval Runner App: Foundry implementation, operations, policy, catalog, or rollout artifact.
44. [IP-044-eval-parity-analyzer-bootstrap.md](../microservices/intelligence/IP-044-eval-parity-analyzer-bootstrap.md) - Ip 044 Eval Parity Analyzer Bootstrap: Foundry implementation, operations, policy, catalog, or rollout artifact.
45. [IP-045-eval-replay-engine-bootstrap.md](../microservices/intelligence/IP-045-eval-replay-engine-bootstrap.md) - Ip 045 Eval Replay Engine Bootstrap: Foundry implementation, operations, policy, catalog, or rollout artifact.
46. [IP-046-evidence-storage-backend-iac.md](../microservices/intelligence/IP-046-evidence-storage-backend-iac.md) - Ip 046 Evidence Storage Backend Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
47. [IP-047-evidence-self-slo-manifest.md](../microservices/intelligence/IP-047-evidence-self-slo-manifest.md) - Ip 047 Evidence Self SLO Manifest: Foundry implementation, operations, policy, catalog, or rollout artifact.
48. [IP-048-evidence-capability-invocation-recorder-kernel.md](../microservices/intelligence/IP-048-evidence-capability-invocation-recorder-kernel.md) - Ip 048 Evidence Capability Invocation Recorder Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
49. [IP-049-evidence-evidence-pack-builder-kernel.md](../microservices/intelligence/IP-049-evidence-evidence-pack-builder-kernel.md) - Ip 049 Evidence Evidence Pack Builder Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
50. [IP-050-evidence-evidence-pack-builder-domain.md](../microservices/intelligence/IP-050-evidence-evidence-pack-builder-domain.md) - Ip 050 Evidence Evidence Pack Builder Domain: Foundry implementation, operations, policy, catalog, or rollout artifact.
51. [IP-051-evidence-evidence-pack-builder-usecase-and-adapters.md](../microservices/intelligence/IP-051-evidence-evidence-pack-builder-usecase-and-adapters.md) - Ip 051 Evidence Evidence Pack Builder Usecase And Adapters: Foundry implementation, operations, policy, catalog, or rollout artifact.
52. [IP-052-evidence-capability-invocation-recorder-stack.md](../microservices/intelligence/IP-052-evidence-capability-invocation-recorder-stack.md) - Ip 052 Evidence Capability Invocation Recorder Stack: Foundry implementation, operations, policy, catalog, or rollout artifact.
53. [IP-053-evidence-eval-evidence-aggregator.md](../microservices/intelligence/IP-053-evidence-eval-evidence-aggregator.md) - Ip 053 Evidence Eval Evidence Aggregator: Foundry implementation, operations, policy, catalog, or rollout artifact.
54. [IP-054-evidence-evidence-query-stack.md](../microservices/intelligence/IP-054-evidence-evidence-query-stack.md) - Ip 054 Evidence Evidence Query Stack: Foundry implementation, operations, policy, catalog, or rollout artifact.
55. [IP-055-evidence-regulator-export-stack.md](../microservices/intelligence/IP-055-evidence-regulator-export-stack.md) - Ip 055 Evidence Regulator Export Stack: Foundry implementation, operations, policy, catalog, or rollout artifact.
56. [IP-056-evidence-audit-chain-bridge.md](../microservices/intelligence/IP-056-evidence-audit-chain-bridge.md) - Ip 056 Evidence Audit Chain Bridge: Foundry implementation, operations, policy, catalog, or rollout artifact.
57. [IP-057-evidence-sdk-cross-microservice.md](../microservices/intelligence/IP-057-evidence-sdk-cross-microservice.md) - Ip 057 Evidence Sdk Cross Microservice: Foundry implementation, operations, policy, catalog, or rollout artifact.
58. [IP-058-evidence-regulator-export-framework-profiles.md](../microservices/intelligence/IP-058-evidence-regulator-export-framework-profiles.md) - Ip 058 Evidence Regulator Export Framework Profiles: Foundry implementation, operations, policy, catalog, or rollout artifact.
59. [IP-059-evidence-evidence-archive-cascade.md](../microservices/intelligence/IP-059-evidence-evidence-archive-cascade.md) - Ip 059 Evidence Evidence Archive Cascade: Foundry implementation, operations, policy, catalog, or rollout artifact.
60. [IP-060-evidence-self-observability-slo-wiring.md](../microservices/intelligence/IP-060-evidence-self-observability-slo-wiring.md) - Ip 060 Evidence Self Observability SLO Wiring: Foundry implementation, operations, policy, catalog, or rollout artifact.
61. [IP-061-guardrails-cedar-policy-engine-iac.md](../microservices/intelligence/IP-061-guardrails-cedar-policy-engine-iac.md) - Ip 061 Guardrails Cedar Policy Engine Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
62. [IP-062-guardrails-classifier-model-serving-iac.md](../microservices/intelligence/IP-062-guardrails-classifier-model-serving-iac.md) - Ip 062 Guardrails Classifier Model Serving Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
63. [IP-063-guardrails-rule-store-postgres-iac.md](../microservices/intelligence/IP-063-guardrails-rule-store-postgres-iac.md) - Ip 063 Guardrails Rule Store Postgres Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
64. [IP-064-guardrails-prompt-classifier-kernel.md](../microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md) - Ip 064 Guardrails Prompt Classifier Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
65. [IP-065-guardrails-output-validator-kernel.md](../microservices/intelligence/IP-065-guardrails-output-validator-kernel.md) - Ip 065 Guardrails Output Validator Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
66. [IP-066-guardrails-autonomy-tier-gate-kernel-and-cedar-adapter.md](../microservices/intelligence/IP-066-guardrails-autonomy-tier-gate-kernel-and-cedar-adapter.md) - Ip 066 Guardrails Autonomy Tier Gate Kernel And Cedar Adapter: Foundry implementation, operations, policy, catalog, or rollout artifact.
67. [IP-067-guardrails-content-safety-rule-engine-kernel-and-postgres-adapter.md](../microservices/intelligence/IP-067-guardrails-content-safety-rule-engine-kernel-and-postgres-adapter.md) - Ip 067 Guardrails Content Safety Rule Engine Kernel And Postgres Adapter: Foundry implementation, operations, policy, catalog, or rollout artifact.
68. [IP-068-guardrails-jailbreak-detector-ensemble.md](../microservices/intelligence/IP-068-guardrails-jailbreak-detector-ensemble.md) - Ip 068 Guardrails Jailbreak Detector Ensemble: Foundry implementation, operations, policy, catalog, or rollout artifact.
69. [IP-069-guardrails-ai-slop-detector.md](../microservices/intelligence/IP-069-guardrails-ai-slop-detector.md) - Ip 069 Guardrails AI SLOp Detector: Foundry implementation, operations, policy, catalog, or rollout artifact.
70. [IP-070-guardrails-classifier-model-adapter-onnx.md](../microservices/intelligence/IP-070-guardrails-classifier-model-adapter-onnx.md) - Ip 070 Guardrails Classifier Model Adapter Onnx: Foundry implementation, operations, policy, catalog, or rollout artifact.
71. [IP-071-guardrails-rest-and-grpc-surface.md](../microservices/intelligence/IP-071-guardrails-rest-and-grpc-surface.md) - Ip 071 Guardrails Rest And Grpc Surface: Foundry implementation, operations, policy, catalog, or rollout artifact.
72. [IP-072-guardrails-worker-and-app-composition.md](../microservices/intelligence/IP-072-guardrails-worker-and-app-composition.md) - Ip 072 Guardrails Worker And App Composition: Foundry implementation, operations, policy, catalog, or rollout artifact.
73. [IP-073-guardrails-runtime-guardrails-coupling-lane.md](../microservices/intelligence/IP-073-guardrails-runtime-guardrails-coupling-lane.md) - Ip 073 Guardrails Runtime Guardrails Coupling Lane: Foundry implementation, operations, policy, catalog, or rollout artifact.
74. [IP-074-guardrails-shadow-mode-rollout-and-false-positive-budget.md](../microservices/intelligence/IP-074-guardrails-shadow-mode-rollout-and-false-positive-budget.md) - Ip 074 Guardrails Shadow Mode Rollout And False Positive Budget: Foundry implementation, operations, policy, catalog, or rollout artifact.
75. [IP-075-guardrails-sdk-rust-and-typescript.md](../microservices/intelligence/IP-075-guardrails-sdk-rust-and-typescript.md) - Ip 075 Guardrails Sdk Rust And Typescript: Foundry implementation, operations, policy, catalog, or rollout artifact.
76. [IP-076-providers-router-kernel.md](../microservices/intelligence/IP-076-providers-router-kernel.md) - Ip 076 Providers Router Kernel: Foundry implementation, operations, policy, catalog, or rollout artifact.
77. [IP-077-providers-router-domain.md](../microservices/intelligence/IP-077-providers-router-domain.md) - Ip 077 Providers Router Domain: Foundry implementation, operations, policy, catalog, or rollout artifact.
78. [IP-078-providers-router-usecase.md](../microservices/intelligence/IP-078-providers-router-usecase.md) - Ip 078 Providers Router Usecase: Foundry implementation, operations, policy, catalog, or rollout artifact.
79. [IP-079-providers-router-api.md](../microservices/intelligence/IP-079-providers-router-api.md) - Ip 079 Providers Router API: Foundry implementation, operations, policy, catalog, or rollout artifact.
80. [IP-080-providers-router-adapter.md](../microservices/intelligence/IP-080-providers-router-adapter.md) - Ip 080 Providers Router Adapter: Foundry implementation, operations, policy, catalog, or rollout artifact.
81. [IP-081-providers-adapter-anthropic-api.md](../microservices/intelligence/IP-081-providers-adapter-anthropic-api.md) - Ip 081 Providers Adapter Anthropic API: Foundry implementation, operations, policy, catalog, or rollout artifact.
82. [IP-082-providers-adapter-anthropic-subscription.md](../microservices/intelligence/IP-082-providers-adapter-anthropic-subscription.md) - Ip 082 Providers Adapter Anthropic Subscription: Foundry implementation, operations, policy, catalog, or rollout artifact.
83. [IP-083-providers-adapter-openai-api.md](../microservices/intelligence/IP-083-providers-adapter-openai-api.md) - Ip 083 Providers Adapter Openai API: Foundry implementation, operations, policy, catalog, or rollout artifact.
84. [IP-084-providers-adapter-openai-subscription.md](../microservices/intelligence/IP-084-providers-adapter-openai-subscription.md) - Ip 084 Providers Adapter Openai Subscription: Foundry implementation, operations, policy, catalog, or rollout artifact.
85. [IP-085-providers-adapter-gemini-api.md](../microservices/intelligence/IP-085-providers-adapter-gemini-api.md) - Ip 085 Providers Adapter Gemini API: Foundry implementation, operations, policy, catalog, or rollout artifact.
86. [IP-086-providers-adapter-gemini-subscription.md](../microservices/intelligence/IP-086-providers-adapter-gemini-subscription.md) - Ip 086 Providers Adapter Gemini Subscription: Foundry implementation, operations, policy, catalog, or rollout artifact.
87. [IP-087-providers-adapter-in-house.md](../microservices/intelligence/IP-087-providers-adapter-in-house.md) - Ip 087 Providers Adapter In House: Foundry implementation, operations, policy, catalog, or rollout artifact.
88. [IP-088-providers-adapter-openbao.md](../microservices/intelligence/IP-088-providers-adapter-openbao.md) - Ip 088 Providers Adapter Openbao: Foundry implementation, operations, policy, catalog, or rollout artifact.
89. [IP-089-providers-router-rest-worker-app.md](../microservices/intelligence/IP-089-providers-router-rest-worker-app.md) - Ip 089 Providers Router Rest Worker App: Foundry implementation, operations, policy, catalog, or rollout artifact.
90. [IP-090-providers-router-sdk.md](../microservices/intelligence/IP-090-providers-router-sdk.md) - Ip 090 Providers Router Sdk: Foundry implementation, operations, policy, catalog, or rollout artifact.
91. [IP-091-milvus-cluster-iac.md](../microservices/intelligence/IP-091-milvus-cluster-iac.md) - Ip 091 Milvus Cluster Iac: Foundry implementation, operations, policy, catalog, or rollout artifact.
92. [IP-092-vector-collection-bootstrap.md](../microservices/intelligence/IP-092-vector-collection-bootstrap.md) - Ip 092 Vector Collection Bootstrap: Foundry implementation, operations, policy, catalog, or rollout artifact.
93. [IP-093-embedding-ingest-pipeline.md](../microservices/intelligence/IP-093-embedding-ingest-pipeline.md) - Ip 093 Embedding Ingest Pipeline: Foundry implementation, operations, policy, catalog, or rollout artifact.
94. [IP-094-hnsw-tuning-and-adapter.md](../microservices/intelligence/IP-094-hnsw-tuning-and-adapter.md) - Ip 094 Hnsw Tuning And Adapter: Foundry implementation, operations, policy, catalog, or rollout artifact.
95. [IP-095-gpu-acceleration-optional.md](../microservices/intelligence/IP-095-gpu-acceleration-optional.md) - Ip 095 Gpu Acceleration Optional: Foundry implementation, operations, policy, catalog, or rollout artifact.
96. [IP-096-milvus-backup-restore.md](../microservices/intelligence/IP-096-milvus-backup-restore.md) - Ip 096 Milvus Backup Restore: Foundry implementation, operations, policy, catalog, or rollout artifact.
97. [IP-097-milvus-cross-region-replication.md](../microservices/intelligence/IP-097-milvus-cross-region-replication.md) - Ip 097 Milvus Cross Region Replication: Foundry implementation, operations, policy, catalog, or rollout artifact.
98. [IP-WASMTIME-001-tool-sandbox-runtime-integration.md](../microservices/intelligence/IP-WASMTIME-001-tool-sandbox-runtime-integration.md) - Ip Wasmtime 001 Tool Sandbox Runtime Integration: Foundry implementation, operations, policy, catalog, or rollout artifact.
99. [IP-WASMTIME-002-capability-token-binding.md](../microservices/intelligence/IP-WASMTIME-002-capability-token-binding.md) - Ip Wasmtime 002 Capability Token Binding: Foundry implementation, operations, policy, catalog, or rollout artifact.
100. [IP-WASMTIME-003-fuel-and-memory-accounting.md](../microservices/intelligence/IP-WASMTIME-003-fuel-and-memory-accounting.md) - Ip Wasmtime 003 Fuel And Memory Accounting: Foundry implementation, operations, policy, catalog, or rollout artifact.
101. [IP-WASMTIME-004-component-model-onboarding.md](../microservices/intelligence/IP-WASMTIME-004-component-model-onboarding.md) - Ip Wasmtime 004 Component Model Onboarding: Foundry implementation, operations, policy, catalog, or rollout artifact.
102. [IP-journey-j100-pack-rollout-first-action.md](../microservices/intelligence/IP-journey-j100-pack-rollout-first-action.md) - Ip Journey J100 Pack Rollout First Action: Foundry implementation, operations, policy, catalog, or rollout artifact.
103. [IP-journey-j116-capability-vetting-attestation.md](../microservices/intelligence/IP-journey-j116-capability-vetting-attestation.md) - Ip Journey J116 Capability Vetting Attestation: Foundry implementation, operations, policy, catalog, or rollout artifact.
104. [IP-journey-j41-prod-rollout-gate.md](../microservices/intelligence/IP-journey-j41-prod-rollout-gate.md) - Ip Journey J41 Prod Rollout Gate: Foundry implementation, operations, policy, catalog, or rollout artifact.
105. [IP-journey-j73-supply-chain-checks.md](../microservices/intelligence/IP-journey-j73-supply-chain-checks.md) - Ip Journey J73 Supply Chain Checks: Foundry implementation, operations, policy, catalog, or rollout artifact.
106. [IP-journey-j75-sca-cve-detection.md](../microservices/intelligence/IP-journey-j75-sca-cve-detection.md) - Ip Journey J75 Sca Cve Detection: Foundry implementation, operations, policy, catalog, or rollout artifact.
107. [IP-journey-j91-us-msb-mtl-overlay.md](../microservices/intelligence/IP-journey-j91-us-msb-mtl-overlay.md) - Ip Journey J91 US Msb Mtl Overlay: Foundry implementation, operations, policy, catalog, or rollout artifact.
108. [IP-journey-j92-br-lgpd-us-parent-dsar.md](../microservices/intelligence/IP-journey-j92-br-lgpd-us-parent-dsar.md) - Ip Journey J92 Br Lgpd US Parent DSAR: Foundry implementation, operations, policy, catalog, or rollout artifact.
109. [IP-journey-j93-in-dpdpa-rbi-overlay.md](../microservices/intelligence/IP-journey-j93-in-dpdpa-rbi-overlay.md) - Ip Journey J93 In Dpdpa Rbi Overlay: Foundry implementation, operations, policy, catalog, or rollout artifact.
110. [IP-journey-j94-sox404-public-company-controls.md](../microservices/intelligence/IP-journey-j94-sox404-public-company-controls.md) - Ip Journey J94 Sox404 Public Company Controls: Foundry implementation, operations, policy, catalog, or rollout artifact.
111. [IP-journey-j95-iso27001-soc2-annual-audit.md](../microservices/intelligence/IP-journey-j95-iso27001-soc2-annual-audit.md) - Ip Journey J95 Iso27001 SOC2 Annual Audit: Foundry implementation, operations, policy, catalog, or rollout artifact.
112. [IP-journey-j96-ksa-uae-mena-onboarding.md](../microservices/intelligence/IP-journey-j96-ksa-uae-mena-onboarding.md) - Ip Journey J96 Ksa Uae Mena Onboarding: Foundry implementation, operations, policy, catalog, or rollout artifact.
113. [IP-journey-j97-sg-pdpa-mas-tenant.md](../microservices/intelligence/IP-journey-j97-sg-pdpa-mas-tenant.md) - Ip Journey J97 Sg Pdpa Mas Tenant: Foundry implementation, operations, policy, catalog, or rollout artifact.
114. [IP-journey-j98-au-privacy-apra-cps234.md](../microservices/intelligence/IP-journey-j98-au-privacy-apra-cps234.md) - Ip Journey J98 Au Privacy Apra Cps234: Foundry implementation, operations, policy, catalog, or rollout artifact.
115. [IP-journey-j99-multi-pack-conflict-resolution.md](../microservices/intelligence/IP-journey-j99-multi-pack-conflict-resolution.md) - Ip Journey J99 Multi Pack Conflict Resolution: Foundry implementation, operations, policy, catalog, or rollout artifact.

### Foundry Runbook Shelf

1. [eval-clickhouse-rebalance.md](../microservices/intelligence/runbooks/eval-clickhouse-rebalance.md) - Eval Clickhouse Rebalance: Foundry implementation, operations, policy, catalog, or rollout artifact.
2. [eval-eval-set-rollback.md](../microservices/intelligence/runbooks/eval-eval-set-rollback.md) - Eval Eval Set Rollback: Foundry implementation, operations, policy, catalog, or rollout artifact.
3. [eval-golden-output-restore.md](../microservices/intelligence/runbooks/eval-golden-output-restore.md) - Eval Golden Output Restore: Foundry implementation, operations, policy, catalog, or rollout artifact.
4. [eval-gpu-pool-rebalance.md](../microservices/intelligence/runbooks/eval-gpu-pool-rebalance.md) - Eval Gpu Pool Rebalance: Foundry implementation, operations, policy, catalog, or rollout artifact.
5. [eval-parity-regression-triage.md](../microservices/intelligence/runbooks/eval-parity-regression-triage.md) - Eval Parity Regression Triage: Foundry implementation, operations, policy, catalog, or rollout artifact.
6. [eval-replay-divergence-investigation.md](../microservices/intelligence/runbooks/eval-replay-divergence-investigation.md) - Eval Replay Divergence Investigation: Foundry implementation, operations, policy, catalog, or rollout artifact.
7. [evidence-audit-chain-backlog.md](../microservices/intelligence/runbooks/evidence-audit-chain-backlog.md) - Evidence Audit Chain Backlog: Foundry implementation, operations, policy, catalog, or rollout artifact.
8. [evidence-blob-storage-restore.md](../microservices/intelligence/runbooks/evidence-blob-storage-restore.md) - Evidence Blob Storage Restore: Foundry implementation, operations, policy, catalog, or rollout artifact.
9. [evidence-evidence-archive-migration.md](../microservices/intelligence/runbooks/evidence-evidence-archive-migration.md) - Evidence Evidence Archive Migration: Foundry implementation, operations, policy, catalog, or rollout artifact.
10. [evidence-evidence-pack-rebuild.md](../microservices/intelligence/runbooks/evidence-evidence-pack-rebuild.md) - Evidence Evidence Pack Rebuild: Foundry implementation, operations, policy, catalog, or rollout artifact.
11. [evidence-pack-assembly-fail.md](../microservices/intelligence/runbooks/evidence-pack-assembly-fail.md) - Evidence Pack Assembly Fail: Foundry implementation, operations, policy, catalog, or rollout artifact.
12. [evidence-regulator-export-reissue.md](../microservices/intelligence/runbooks/evidence-regulator-export-reissue.md) - Evidence Regulator Export Reissue: Foundry implementation, operations, policy, catalog, or rollout artifact.
13. [guardrails-cedar-engine-restart.md](../microservices/intelligence/runbooks/guardrails-cedar-engine-restart.md) - Guardrails Cedar Engine Restart: Foundry implementation, operations, policy, catalog, or rollout artifact.
14. [guardrails-classifier-model-rollback.md](../microservices/intelligence/runbooks/guardrails-classifier-model-rollback.md) - Guardrails Classifier Model Rollback: Foundry implementation, operations, policy, catalog, or rollout artifact.
15. [guardrails-false-positive-tenant-relief.md](../microservices/intelligence/runbooks/guardrails-false-positive-tenant-relief.md) - Guardrails False Positive Tenant Relief: Foundry implementation, operations, policy, catalog, or rollout artifact.
16. [guardrails-jailbreak-escalation.md](../microservices/intelligence/runbooks/guardrails-jailbreak-escalation.md) - Guardrails Jailbreak Escalation: Foundry implementation, operations, policy, catalog, or rollout artifact.
17. [guardrails-policy-rule-rollback.md](../microservices/intelligence/runbooks/guardrails-policy-rule-rollback.md) - Guardrails Policy Rule Rollback: Foundry implementation, operations, policy, catalog, or rollout artifact.
18. [guardrails-rule-store-restore.md](../microservices/intelligence/runbooks/guardrails-rule-store-restore.md) - Guardrails Rule Store Restore: Foundry implementation, operations, policy, catalog, or rollout artifact.
19. [milvus-restore.md](../microservices/intelligence/runbooks/milvus-restore.md) - Milvus Restore: Foundry implementation, operations, policy, catalog, or rollout artifact.
20. [milvus-tenant-quota.md](../microservices/intelligence/runbooks/milvus-tenant-quota.md) - Milvus Tenant Quota: Foundry implementation, operations, policy, catalog, or rollout artifact.
21. [milvus.md](../microservices/intelligence/runbooks/milvus.md) - Milvus: Foundry implementation, operations, policy, catalog, or rollout artifact.
22. [providers-adapter-version-pin.md](../microservices/intelligence/runbooks/providers-adapter-version-pin.md) - Providers Adapter Version Pin: Foundry implementation, operations, policy, catalog, or rollout artifact.
23. [providers-credential-rotation.md](../microservices/intelligence/runbooks/providers-credential-rotation.md) - Providers Credential Rotation: Foundry implementation, operations, policy, catalog, or rollout artifact.
24. [providers-in-house-model-rollback.md](../microservices/intelligence/runbooks/providers-in-house-model-rollback.md) - Providers In House Model Rollback: Foundry implementation, operations, policy, catalog, or rollout artifact.
25. [providers-provider-credentials-revoke.md](../microservices/intelligence/runbooks/providers-provider-credentials-revoke.md) - Providers Provider Credentials Revoke: Foundry implementation, operations, policy, catalog, or rollout artifact.
26. [providers-provider-outage-failover.md](../microservices/intelligence/runbooks/providers-provider-outage-failover.md) - Providers Provider Outage Failover: Foundry implementation, operations, policy, catalog, or rollout artifact.
27. [providers-rate-limit-cascade-recovery.md](../microservices/intelligence/runbooks/providers-rate-limit-cascade-recovery.md) - Providers Rate Limit Cascade Recovery: Foundry implementation, operations, policy, catalog, or rollout artifact.
28. [runtime-autonomy-violation-quarantine.md](../microservices/intelligence/runbooks/runtime-autonomy-violation-quarantine.md) - Runtime Autonomy Violation Quarantine: Foundry implementation, operations, policy, catalog, or rollout artifact.
29. [runtime-capability-registry-resync.md](../microservices/intelligence/runbooks/runtime-capability-registry-resync.md) - Runtime Capability Registry Resync: Foundry implementation, operations, policy, catalog, or rollout artifact.
30. [runtime-emergency-runtime-drain.md](../microservices/intelligence/runbooks/runtime-emergency-runtime-drain.md) - Runtime Emergency Runtime Drain: Foundry implementation, operations, policy, catalog, or rollout artifact.
31. [runtime-redis-failover.md](../microservices/intelligence/runbooks/runtime-redis-failover.md) - Runtime Redis Failover: Foundry implementation, operations, policy, catalog, or rollout artifact.
32. [runtime-runtime-pod-crash.md](../microservices/intelligence/runbooks/runtime-runtime-pod-crash.md) - Runtime Runtime Pod Crash: Foundry implementation, operations, policy, catalog, or rollout artifact.
33. [runtime-session-state-recovery.md](../microservices/intelligence/runbooks/runtime-session-state-recovery.md) - Runtime Session State Recovery: Foundry implementation, operations, policy, catalog, or rollout artifact.
34. [supervisor-autonomy-violation.md](../microservices/intelligence/runbooks/supervisor-autonomy-violation.md) - Supervisor Autonomy Violation: Foundry implementation, operations, policy, catalog, or rollout artifact.
35. [supervisor-deployment-rollback.md](../microservices/intelligence/runbooks/supervisor-deployment-rollback.md) - Supervisor Deployment Rollback: Foundry implementation, operations, policy, catalog, or rollout artifact.
36. [supervisor-fleet-state-recovery.md](../microservices/intelligence/runbooks/supervisor-fleet-state-recovery.md) - Supervisor Fleet State Recovery: Foundry implementation, operations, policy, catalog, or rollout artifact.
37. [supervisor-kill-switch-engage.md](../microservices/intelligence/runbooks/supervisor-kill-switch-engage.md) - Supervisor Kill Switch Engage: Foundry implementation, operations, policy, catalog, or rollout artifact.
38. [supervisor-kubernetes-operator-restart.md](../microservices/intelligence/runbooks/supervisor-kubernetes-operator-restart.md) - Supervisor Kubernetes Operator Restart: Foundry implementation, operations, policy, catalog, or rollout artifact.
39. [supervisor-supervision-bus-replay.md](../microservices/intelligence/runbooks/supervisor-supervision-bus-replay.md) - Supervisor Supervision Bus Replay: Foundry implementation, operations, policy, catalog, or rollout artifact.
40. [wasm-runtime-failover.md](../microservices/intelligence/runbooks/wasm-runtime-failover.md) - Wasm Runtime Failover: Foundry implementation, operations, policy, catalog, or rollout artifact.
41. [wasm-tool-quarantine.md](../microservices/intelligence/runbooks/wasm-tool-quarantine.md) - Wasm Tool Quarantine: Foundry implementation, operations, policy, catalog, or rollout artifact.

<a id="section-9-compliance-packs"></a>
## §9 Compliance Packs

### Section Purpose

Compliance packs bind regulatory obligations to pack manifests, localization packs, privacy/security programs, and journey-level evidence. Use this section to answer which legal/regulatory overlay applies and where the machine-readable manifest lives.

### Canonical Docs (Top 15)

1. [CSAP pack](../registry/compliance-packs/CSAP.yaml) - KR CSAP compliance pack manifest.
2. [KR-PIPA pack](../registry/compliance-packs/KR-PIPA.yaml) - KR PIPA compliance pack manifest.
3. [GDPR pack](../registry/compliance-packs/GDPR.yaml) - GDPR compliance pack manifest.
4. [EU AI Act pack](../registry/compliance-packs/EU-AI-Act.yaml) - EU AI Act compliance pack manifest.
5. [HIPAA pack](../registry/compliance-packs/HIPAA.yaml) - HIPAA compliance pack manifest.
6. [PCI DSS pack](../registry/compliance-packs/PCI-DSS-v4.yaml) - PCI DSS v4 compliance pack manifest.
7. [SOC2 pack](../registry/compliance-packs/SOC2-Type-II.yaml) - SOC2 Type II compliance pack manifest.
8. [EU CSRD pack](../registry/compliance-packs/EU-CSRD.yaml) - EU CSRD compliance pack manifest.
9. [compliance pack schema](../specs/compliance-pack-schema.json) - schema for compliance pack manifests.
10. [compliance matrix](COMPLIANCE-MATRIX.md) - human-readable compliance matrix.
11. [privacy program](PRIVACY-PROGRAM.md) - privacy program authority.
12. [security program](security-program/security-program.json) - security program authority.
13. [localization index](localization-packs/INDEX.md) - localization pack index.
14. [regional packs README](regional-packs/README.md) - regional pack documentation entry.
15. [KR regional pack](regional-packs/oya-pack-kr/PACK.md) - KR regional pack documentation.

### Related Sections

Related sections: [§4 User Journeys](#section-4-user-journeys), [§6 Standards](#section-6-standards), [§10 Capability Tiers](#section-10-capability-tiers), [§12 Glossary](#section-12-glossary), [§13 Wave Sequence](#section-13-wave-sequence).

### When To Read

- Read this before changing pack manifests, regional overlays, compliance evidence, regulator-facing docs, privacy, or security controls.
- Read this when a journey mentions GDPR, CSAP, KR-PIPA, HIPAA, PCI, SOC2, EU AI Act, or EU CSRD.
- Read the schema before adding or changing a pack manifest.
- Read regional and localization packs when compliance interacts with language, tax, identity, residency, or payment rails.

### Compliance Pack Manifests

1. [CSAP.yaml](../registry/compliance-packs/CSAP.yaml) - CSAP: compliance, localization, residency, identity, or regulatory-pack authority.
2. [EU-AI-Act.yaml](../registry/compliance-packs/EU-AI-Act.yaml) - EU AI Act: compliance, localization, residency, identity, or regulatory-pack authority.
3. [EU-CSRD.yaml](../registry/compliance-packs/EU-CSRD.yaml) - EU Csrd: compliance, localization, residency, identity, or regulatory-pack authority.
4. [GDPR.yaml](../registry/compliance-packs/GDPR.yaml) - Gdpr: compliance, localization, residency, identity, or regulatory-pack authority.
5. [HIPAA.yaml](../registry/compliance-packs/HIPAA.yaml) - HIPAA: compliance, localization, residency, identity, or regulatory-pack authority.
6. [KR-PIPA.yaml](../registry/compliance-packs/KR-PIPA.yaml) - KR Pipa: compliance, localization, residency, identity, or regulatory-pack authority.
7. [PCI-DSS-v4.yaml](../registry/compliance-packs/PCI-DSS-v4.yaml) - PCI Dss V4: compliance, localization, residency, identity, or regulatory-pack authority.
8. [SOC2-Type-II.yaml](../registry/compliance-packs/SOC2-Type-II.yaml) - SOC2 Type Ii: compliance, localization, residency, identity, or regulatory-pack authority.

### Regional And Localization Pack Docs

1. [regional-packs/README.md](regional-packs/README.md) - Regional Packs: compliance, localization, residency, identity, or regulatory-pack authority.
2. [regional-packs/_TEMPLATE.md](regional-packs/_TEMPLATE.md) - Template: compliance, localization, residency, identity, or regulatory-pack authority.
3. [regional-packs/oya-pack-kr/PACK.md](regional-packs/oya-pack-kr/PACK.md) - Pack: compliance, localization, residency, identity, or regulatory-pack authority.
4. [localization-packs/INDEX.md](localization-packs/INDEX.md) - Index: compliance, localization, residency, identity, or regulatory-pack authority.
5. [localization-packs/kr/pack.yaml](localization-packs/kr/pack.yaml) - Pack: compliance, localization, residency, identity, or regulatory-pack authority.
6. [localization-packs/kr.md](localization-packs/kr.md) - Kr: compliance, localization, residency, identity, or regulatory-pack authority.

<a id="section-10-capability-tiers"></a>
## §10 Capability Tiers

### Section Purpose

Capability tiers define capability maturity and autonomy boundaries across products, microservices, vendors, and compliance contexts. This section makes ADR-0316 navigable from the live tier registry and standards that govern authoring and uplift.

### Canonical Docs (Top 12)

1. [capability tier index](../registry/capability-tiers/index.json) - capability tier registry index.
6. [checkpoint tier](../registry/capability-tiers/checkpoint.json) - checkpoint tier behavior.
7. [microservice tier mapping](../registry/capability-tiers/microservice-tier-mapping.yaml) - microservice to capability tier mapping.
8. [vendor tier mapping](../registry/capability-tiers/vendor-tier-mapping.yaml) - vendor to capability tier mapping.
9. [ADR-0316](decisions/ADR-0316-capability-tier-over-product-fragmentation.md) - decision making capability tiers primary.
10. [capability tier matrix](standards/capability-tier-matrix.md) - standard matrix for capability tiers.
11. [capability authoring](standards/capability-authoring.md) - capability authoring standard.
12. [autonomy ceiling](standards/autonomy-ceiling.md) - autonomy ceiling standard tied to tier uplift.

### Related Sections

Related sections: [§2 Decisions](#section-2-decisions), [§6 Standards](#section-6-standards), [§8 Governance Pipeline](#section-8-governance-pipeline), [§9 Compliance Packs](#section-9-compliance-packs), [§11 Governance Crates](#section-11-governance-crates).

### When To Read

- Read this before publishing or promoting a capability.
- Read this when tier names appear in manifests, catalog records, provider decisions, or autonomy policies.
- Read the mapping files before claiming microservice or vendor tier coverage.
- Read ADR-0316 before deciding whether something is a product boundary or capability tier.

### Capability Tier Registry Files

2. [checkpoint.json](../registry/capability-tiers/checkpoint.json) - Checkpoint: capability-tier registry or tier-governance authority.
4. [index.json](../registry/capability-tiers/index.json) - Index: capability-tier registry or tier-governance authority.
5. [microservice-tier-mapping.yaml](../registry/capability-tiers/microservice-tier-mapping.yaml) - Microservice Tier Mapping: capability-tier registry or tier-governance authority.
8. [vendor-tier-mapping.yaml](../registry/capability-tiers/vendor-tier-mapping.yaml) - Vendor Tier Mapping: capability-tier registry or tier-governance authority.

<a id="section-11-governance-crates"></a>
## §11 Governance Crates

### Section Purpose

Governance crates are executable policy lanes. They turn standards and ADRs into checks for audit event emission, BYOK disambiguation, capability tier coverage, Cedar coverage, naming justifications, template-stamping prevention, pack overlay completeness, and substance bar enforcement.

### Canonical Docs (Top 11)

1. [audit-event-emission crate](../crates/oya-governance-audit-event-emission) - audit event emission governance lane crate.
2. [BYOK disambiguation crate](../crates/oya-governance-byok-disambiguation) - BYOK disambiguation governance lane crate.
3. [capability-tier-coverage crate](../crates/oya-governance-capability-tier-coverage) - capability tier coverage governance lane crate.
4. [Cedar coverage crate](../crates/oya-governance-cedar-coverage) - Cedar coverage governance lane crate.
5. [naming justifications crate](../crates/oya-governance-naming-justifications) - naming justification governance lane crate.
6. [no-template-stamping crate](../crates/oya-governance-no-template-stamping) - template-stamping prevention governance lane crate.
7. [pack overlay completeness crate](../crates/oya-governance-pack-overlay-completeness) - pack overlay completeness governance lane crate.
8. [substance bar crate](../crates/oya-governance-substance-bar) - substance bar governance lane crate.
9. [ADR-0221](decisions/ADR-0221-agentic-development-pipeline-hardening.md) - ADR for agentic development pipeline hardening.
10. [documentation rigor](standards/documentation-rigor.md) - substance bar standard paired with governance lanes.

### Related Sections

Related sections: [§2 Decisions](#section-2-decisions), [§6 Standards](#section-6-standards), [§8 Governance Pipeline](#section-8-governance-pipeline), [§10 Capability Tiers](#section-10-capability-tiers), [§13 Wave Sequence](#section-13-wave-sequence).

### When To Read

- Read this when a governance lane fails or a standard says enforcement is handled by `oya-governance-*`.
- Read this before changing policy gates or claiming a doc/code change meets a governance standard.
- Read ADR-0221 before altering agentic pipeline hardening or review evidence requirements.
- Read the crate source after the standard when you need exact acceptance behavior.

### Governance Crate Shelf

1. [oya-governance-audit-event-emission](../crates/oya-governance-audit-event-emission) - Oya Governance Audit Event Emission: Rust governance lane implementation; inspect before changing the corresponding policy gate.
2. [oya-governance-byok-disambiguation](../crates/oya-governance-byok-disambiguation) - Oya Governance Byok Disambiguation: Rust governance lane implementation; inspect before changing the corresponding policy gate.
3. [oya-governance-capability-tier-coverage](../crates/oya-governance-capability-tier-coverage) - Oya Governance Capability Tier Coverage: Rust governance lane implementation; inspect before changing the corresponding policy gate.
4. [oya-governance-cedar-coverage](../crates/oya-governance-cedar-coverage) - Oya Governance Cedar Coverage: Rust governance lane implementation; inspect before changing the corresponding policy gate.
5. [oya-governance-naming-justifications](../crates/oya-governance-naming-justifications) - Oya Governance Naming Justifications: Rust governance lane implementation; inspect before changing the corresponding policy gate.
6. [oya-governance-no-template-stamping](../crates/oya-governance-no-template-stamping) - Oya Governance No Template Stamping: Rust governance lane implementation; inspect before changing the corresponding policy gate.
7. [oya-governance-pack-overlay-completeness](../crates/oya-governance-pack-overlay-completeness) - Oya Governance Pack Overlay Completeness: Rust governance lane implementation; inspect before changing the corresponding policy gate.
8. [oya-governance-substance-bar](../crates/oya-governance-substance-bar) - Oya Governance Substance Bar: Rust governance lane implementation; inspect before changing the corresponding policy gate.

<a id="section-12-glossary"></a>
## §12 Glossary

### Section Purpose

The glossary owns vocabulary consistency. It prevents drift across ADRs, standards, PRDs, journeys, personas, localization packs, and code comments by naming canonical terms, retired terms, Korean-English parity, and industry analogs.

### Canonical Docs (Top 10)

1. [glossary](GLOSSARY.md) - canonical vocabulary and retired-term guardrail.
2. [glossary mirror](machine-readable/glossary.json) - machine-readable glossary mirror.
3. [naming BNF](standards/naming-convention-bnf-v4.md) - naming convention BNF that protects vocabulary consistency.
4. [doc style](standards/doc-style.md) - doc-style rules for vocabulary and tone.
5. [documentation rigor](standards/documentation-rigor.md) - requires every repeated term to resolve to the glossary.
6. [ADR-0018](decisions/ADR-0018-glossary-and-terminology-canon.md) - decision record establishing glossary canon.
7. [ADR-0105](decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md) - layer enum vocabulary anchor.
8. [layer enum standard](standards/layer-enum-adr-0105.md) - standards companion to the layer enum.
9. [i18n canonical](standards/i18n-canonical.md) - localization vocabulary and canonical base guidance.
10. [KR localization pack](localization-packs/kr.md) - KR localization vocabulary pack.

### Related Sections

Related sections: [§0 Operating Map](#section-0-operating-map), [§2 Decisions](#section-2-decisions), [§5 Personas](#section-5-personas), [§6 Standards](#section-6-standards), [§9 Compliance Packs](#section-9-compliance-packs).

### When To Read

- Read this when a term appears with multiple spellings or unclear scope.
- Read this before inventing a new Oyatie-specific term.
- Read this before touching naming, layer enums, localization packs, or compliance terminology.
- Read this when a review flags retired vocabulary such as legacy product-group or object-graph terminology.

### Glossary-Adjacent Vocabulary Controls

1. [GLOSSARY.md](GLOSSARY.md) - human-readable vocabulary authority.
2. [machine-readable glossary.json](machine-readable/glossary.json) - machine-readable vocabulary mirror.
3. [naming-convention-bnf-v4.md](standards/naming-convention-bnf-v4.md) - BNF-level naming control.
4. [layer-enum-adr-0105.md](standards/layer-enum-adr-0105.md) - layer vocabulary companion.
5. [localization-packs/kr.md](localization-packs/kr.md) - Korean localization vocabulary and pack terms.

<a id="section-13-wave-sequence"></a>
## §13 Wave Sequence

### Section Purpose

Wave sequence answers what happens now, what recently changed, and what comes next. It is the staging bridge between masterplan authority, roadmap projection, architecture audit evidence, readiness checklists, and release/runbook operations.

### Canonical Docs (Top 15)

1. [ROADMAP.md](ROADMAP.md) - human-readable wave sequence and current gate map.
2. [MASTERPLAN.md](MASTERPLAN.md) - human-readable master plan projection.
3. [masterplan.json](../specs/masterplan.json) - machine-readable master plan authority.
4. [master-plan-sequencing.json](../specs/master-plan-sequencing.json) - machine-readable sequence and forbidden primitive contract.
5. [planning-closure-contract.json](../specs/planning-closure-contract.json) - planning closure contract for milestone gates.
6. [planning closure ledger](../specs/planning-closure-status-closure-ledger.json) - planning closure status ledger.
7. [agent durable goal](../specs/agent-durable-goal.json) - durable agent goal prompt projection.
8. [wave 3-g briefing](architecture/wave-3-g-executive-briefing-2026-05-21.md) - Wave 3-G executive briefing.
9. [wave 3-g adjudication](architecture/wave-3-g-synthesis-adjudication-2026-05-21.md) - Wave 3-G synthesis adjudication.
10. [post-wave rigor audit](architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md) - post-Wave-3-G corpus rigor audit.
11. [six-hops audit](architecture/six-hops-reachability-audit-2026-05-20.md) - six-hops audit that makes this README load-bearing.
12. [wave gate checklist](../templates/checklists/wave-gate.md) - wave-gate checklist.
13. [wave gate evaluation runbook](runbooks/wave-gate-evaluation.md) - wave-gate evaluation runbook.
14. [wave gate readiness runbook](runbooks/wave-gate-readiness-check.md) - wave-gate readiness runbook.
15. [release management](RELEASE-MANAGEMENT.md) - release-management doc that consumes wave state.

### Current State And Recent Waves

1. Current canonical wave vocabulary is gate-based: W-Foundation, W-Foundry-Preview, W-Cloud-Preview, W-SaaS-Preview, W-Search-Preview, W-Vertical-Pilot, W-Vertical-Fan-Out, W-Cloud-Stable, W-Search-Stable, W-Ads-Preview, W-Ads-Stable, and W-Region-Fan-Out.
2. Current state from the checked-in roadmap is still Foundation-oriented with Foundry preview as the next major dependency gate.
3. Recent architecture waves include Wave 3-G synthesis adjudication, executive briefing, corpus rigor audits, and six-hops reachability audit.
4. Next-wave reading should start from machine-readable masterplan and planning-closure contracts, then cross-check ROADMAP for human-facing wave language.
5. Checkpoint for this gardening pass: docs-hub-gardening-w1-2026-05-20 deepens this hub only; canonical ADRs, products, journeys, personas, standards, architecture docs, microservices, and registry sources are intentionally not modified.

### Related Sections

Related sections: [§0 Operating Map](#section-0-operating-map), [§1 Architecture](#section-1-architecture), [§2 Decisions](#section-2-decisions), [§8 Governance Pipeline](#section-8-governance-pipeline), [§9 Compliance Packs](#section-9-compliance-packs).

### When To Read

- Read this before answering milestone or wave transition questions.
- Read this when a user asks whether the repo is ready to move to the next milestone or wave.
- Read this before planning parallel work so prerequisites, blockers, and gate evidence stay ordered.
- Read this before release, promotion, or readiness claims.

### Runbook Shelf For Wave, Release, Incident, And Operations Questions

1. [runbooks/ad-auction-latency-incident.md](runbooks/ad-auction-latency-incident.md) - Ad Auction Latency Incident: operational procedure for incident, rollback, audit, release, or service recovery.
2. [runbooks/adr-promotion-triage.md](runbooks/adr-promotion-triage.md) - ADR Promotion Triage: operational procedure for incident, rollback, audit, release, or service recovery.
3. [runbooks/adr-supersession-graph-update.md](runbooks/adr-supersession-graph-update.md) - ADR Supersession Graph Update: operational procedure for incident, rollback, audit, release, or service recovery.
4. [runbooks/ads/auction-engine-overload.md](runbooks/ads/auction-engine-overload.md) - Auction Engine Overload: operational procedure for incident, rollback, audit, release, or service recovery.
5. [runbooks/ads/click-fraud-spike.md](runbooks/ads/click-fraud-spike.md) - Click Fraud Spike: operational procedure for incident, rollback, audit, release, or service recovery.
6. [runbooks/ads/data-use-boundary-violation.md](runbooks/ads/data-use-boundary-violation.md) - Data Use Boundary Violation: operational procedure for incident, rollback, audit, release, or service recovery.
7. [runbooks/agent-authoring-evidence-attach.md](runbooks/agent-authoring-evidence-attach.md) - Agent Authoring Evidence Attach: operational procedure for incident, rollback, audit, release, or service recovery.
9. [runbooks/alias-sunset-promotion.md](runbooks/alias-sunset-promotion.md) - Alias Sunset Promotion: operational procedure for incident, rollback, audit, release, or service recovery.
10. [runbooks/aml-alert-escalation.md](runbooks/aml-alert-escalation.md) - Aml Alert Escalation: operational procedure for incident, rollback, audit, release, or service recovery.
11. [runbooks/analytics/dp-budget-exhausted.md](runbooks/analytics/dp-budget-exhausted.md) - Dp Budget Exhausted: operational procedure for incident, rollback, audit, release, or service recovery.
12. [runbooks/analytics-warehouse-reconciliation.md](runbooks/analytics-warehouse-reconciliation.md) - Analytics Warehouse Reconciliation: operational procedure for incident, rollback, audit, release, or service recovery.
13. [runbooks/api-gateway-rate-limit-incident.md](runbooks/api-gateway-rate-limit-incident.md) - API Gateway Rate Limit Incident: operational procedure for incident, rollback, audit, release, or service recovery.
14. [runbooks/attribution-pipeline-lag.md](runbooks/attribution-pipeline-lag.md) - Attribution Pipeline Lag: operational procedure for incident, rollback, audit, release, or service recovery.
15. [runbooks/audit-chain-integrity-check.md](runbooks/audit-chain-integrity-check.md) - Audit Chain Integrity Check: operational procedure for incident, rollback, audit, release, or service recovery.
16. [runbooks/audit-chain-integrity-recovery.md](runbooks/audit-chain-integrity-recovery.md) - Audit Chain Integrity Recovery: operational procedure for incident, rollback, audit, release, or service recovery.
17. [runbooks/autonomy-ceiling-breach-response.md](runbooks/autonomy-ceiling-breach-response.md) - Autonomy Ceiling Breach Response: operational procedure for incident, rollback, audit, release, or service recovery.
18. [runbooks/autonomy-tier-uplift.md](runbooks/autonomy-tier-uplift.md) - Autonomy Tier Uplift: operational procedure for incident, rollback, audit, release, or service recovery.
19. [runbooks/axis-admission-proposal.md](runbooks/axis-admission-proposal.md) - Axis Admission Proposal: operational procedure for incident, rollback, audit, release, or service recovery.
20. [runbooks/axis-retire-consolidate.md](runbooks/axis-retire-consolidate.md) - Axis Retire Consolidate: operational procedure for incident, rollback, audit, release, or service recovery.
21. [runbooks/bootstrap-ci-compromise.md](runbooks/bootstrap-ci-compromise.md) - Bootstrap Ci Compromise: operational procedure for incident, rollback, audit, release, or service recovery.
22. [runbooks/brand-rename-batch-execute.md](runbooks/brand-rename-batch-execute.md) - Brand Rename Batch Execute: operational procedure for incident, rollback, audit, release, or service recovery.
23. [runbooks/brand-rename-rollback.md](runbooks/brand-rename-rollback.md) - Brand Rename Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
24. [runbooks/breach-notification-council-escalation.md](runbooks/breach-notification-council-escalation.md) - Breach Notification Council Escalation: operational procedure for incident, rollback, audit, release, or service recovery.
25. [runbooks/breach-notification.md](runbooks/breach-notification.md) - Breach Notification: operational procedure for incident, rollback, audit, release, or service recovery.
26. [runbooks/break-glass-with-evidence.md](runbooks/break-glass-with-evidence.md) - Break Glass With Evidence: operational procedure for incident, rollback, audit, release, or service recovery.
27. [runbooks/byok-rotation-encryption-tenant-duress.md](runbooks/byok-rotation-encryption-tenant-duress.md) - Byok Rotation Encryption Tenant Duress: operational procedure for incident, rollback, audit, release, or service recovery.
28. [runbooks/byok-rotation-provider-tenant-duress.md](runbooks/byok-rotation-provider-tenant-duress.md) - Byok Rotation Provider Tenant Duress: operational procedure for incident, rollback, audit, release, or service recovery.
29. [runbooks/capability-rollback.md](runbooks/capability-rollback.md) - Capability Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
30. [runbooks/capacity-scaling-emergency.md](runbooks/capacity-scaling-emergency.md) - Capacity Scaling Emergency: operational procedure for incident, rollback, audit, release, or service recovery.
31. [runbooks/cedar-fragment-emergency-rollback.md](runbooks/cedar-fragment-emergency-rollback.md) - Cedar Fragment Emergency Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
32. [runbooks/cedar-policy-breach.md](runbooks/cedar-policy-breach.md) - Cedar Policy Breach: operational procedure for incident, rollback, audit, release, or service recovery.
33. [runbooks/cedar-policy-rollback.md](runbooks/cedar-policy-rollback.md) - Cedar Policy Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
34. [runbooks/cell-evacuation.md](runbooks/cell-evacuation.md) - Cell Evacuation: operational procedure for incident, rollback, audit, release, or service recovery.
35. [runbooks/cell-failover-intra-region.md](runbooks/cell-failover-intra-region.md) - Cell Failover Intra Region: operational procedure for incident, rollback, audit, release, or service recovery.
36. [runbooks/cell-isolation-breach.md](runbooks/cell-isolation-breach.md) - Cell Isolation Breach: operational procedure for incident, rollback, audit, release, or service recovery.
37. [runbooks/cell-isolation-evidence-quarterly.md](runbooks/cell-isolation-evidence-quarterly.md) - Cell Isolation Evidence Quarterly: operational procedure for incident, rollback, audit, release, or service recovery.
38. [runbooks/cell-provision.md](runbooks/cell-provision.md) - Cell Provision: operational procedure for incident, rollback, audit, release, or service recovery.
39. [runbooks/cell-tier-promotion.md](runbooks/cell-tier-promotion.md) - Cell Tier Promotion: operational procedure for incident, rollback, audit, release, or service recovery.
40. [runbooks/claim-ceiling-bypass-expiry.md](runbooks/claim-ceiling-bypass-expiry.md) - Claim Ceiling Bypass Expiry: operational procedure for incident, rollback, audit, release, or service recovery.
41. [runbooks/clinical-audit-replay.md](runbooks/clinical-audit-replay.md) - Clinical Audit Replay: operational procedure for incident, rollback, audit, release, or service recovery.
42. [runbooks/cloud/billing-event-stream-stuck.md](runbooks/cloud/billing-event-stream-stuck.md) - Billing Event Stream Stuck: operational procedure for incident, rollback, audit, release, or service recovery.
43. [runbooks/cloud/cell-isolation-breach.md](runbooks/cloud/cell-isolation-breach.md) - Cell Isolation Breach: operational procedure for incident, rollback, audit, release, or service recovery.
44. [runbooks/cloud/dcops-cooling-failure.md](runbooks/cloud/dcops-cooling-failure.md) - Dcops Cooling Failure: operational procedure for incident, rollback, audit, release, or service recovery.
45. [runbooks/cloud/dcops-power-event.md](runbooks/cloud/dcops-power-event.md) - Dcops Power Event: operational procedure for incident, rollback, audit, release, or service recovery.
46. [runbooks/cloud/iam-key-rotation.md](runbooks/cloud/iam-key-rotation.md) - Iam Key Rotation: operational procedure for incident, rollback, audit, release, or service recovery.
47. [runbooks/cloud/kms-emergency-rotation.md](runbooks/cloud/kms-emergency-rotation.md) - Kms Emergency Rotation: operational procedure for incident, rollback, audit, release, or service recovery.
48. [runbooks/cloud/region-failover.md](runbooks/cloud/region-failover.md) - Region Failover: operational procedure for incident, rollback, audit, release, or service recovery.
49. [runbooks/cold-chain-breach-alert.md](runbooks/cold-chain-breach-alert.md) - Cold Chain Breach Alert: operational procedure for incident, rollback, audit, release, or service recovery.
50. [runbooks/compliance-pack-emergency-suspension.md](runbooks/compliance-pack-emergency-suspension.md) - Compliance Pack Emergency Suspension: operational procedure for incident, rollback, audit, release, or service recovery.
51. [runbooks/compliance-pack-revocation.md](runbooks/compliance-pack-revocation.md) - Compliance Pack Revocation: operational procedure for incident, rollback, audit, release, or service recovery.
52. [runbooks/consent-withdrawal-cascade.md](runbooks/consent-withdrawal-cascade.md) - Consent Withdrawal Cascade: operational procedure for incident, rollback, audit, release, or service recovery.
53. [runbooks/contract-breaking-change.md](runbooks/contract-breaking-change.md) - Contract Breaking Change: operational procedure for incident, rollback, audit, release, or service recovery.
54. [runbooks/contract-introduction.md](runbooks/contract-introduction.md) - Contract Introduction: operational procedure for incident, rollback, audit, release, or service recovery.
55. [runbooks/cost-anomaly-response.md](runbooks/cost-anomaly-response.md) - Cost Anomaly Response: operational procedure for incident, rollback, audit, release, or service recovery.
56. [runbooks/crawler-politeness-incident.md](runbooks/crawler-politeness-incident.md) - Crawler Politeness Incident: operational procedure for incident, rollback, audit, release, or service recovery.
57. [runbooks/cross-axis/audit-chain-integrity-failure.md](runbooks/cross-axis/audit-chain-integrity-failure.md) - Audit Chain Integrity Failure: operational procedure for incident, rollback, audit, release, or service recovery.
58. [runbooks/cross-axis/cohesion-fitness-violation.md](runbooks/cross-axis/cohesion-fitness-violation.md) - Cohesion Fitness Violation: operational procedure for incident, rollback, audit, release, or service recovery.
59. [runbooks/cross-axis/cross-tenant-access-detected.md](runbooks/cross-axis/cross-tenant-access-detected.md) - Cross Tenant Access Detected: operational procedure for incident, rollback, audit, release, or service recovery.
60. [runbooks/cross-axis/data-class-violation-detected.md](runbooks/cross-axis/data-class-violation-detected.md) - Data Class Violation Detected: operational procedure for incident, rollback, audit, release, or service recovery.
61. [runbooks/cross-axis/dsr-cascade-stuck.md](runbooks/cross-axis/dsr-cascade-stuck.md) - DSR Cascade Stuck: operational procedure for incident, rollback, audit, release, or service recovery.
62. [runbooks/cross-axis/foundation-bypass-expired.md](runbooks/cross-axis/foundation-bypass-expired.md) - Foundation Bypass Expired: operational procedure for incident, rollback, audit, release, or service recovery.
63. [runbooks/cross-axis/regional-pack-regulator-update.md](runbooks/cross-axis/regional-pack-regulator-update.md) - Regional Pack Regulator Update: operational procedure for incident, rollback, audit, release, or service recovery.
64. [runbooks/cross-axis-contradiction-audit.md](runbooks/cross-axis-contradiction-audit.md) - Cross Axis Contradiction Audit: operational procedure for incident, rollback, audit, release, or service recovery.
65. [runbooks/cross-doc-impact-analysis.md](runbooks/cross-doc-impact-analysis.md) - Cross Doc Impact Analysis: operational procedure for incident, rollback, audit, release, or service recovery.
66. [runbooks/cross-pack-tenant-residency.md](runbooks/cross-pack-tenant-residency.md) - Cross Pack Tenant Residency: operational procedure for incident, rollback, audit, release, or service recovery.
67. [runbooks/cross-plane-call-introduction.md](runbooks/cross-plane-call-introduction.md) - Cross Plane Call Introduction: operational procedure for incident, rollback, audit, release, or service recovery.
68. [runbooks/cve-critical-patch.md](runbooks/cve-critical-patch.md) - Cve Critical Patch: operational procedure for incident, rollback, audit, release, or service recovery.
69. [runbooks/data-class-transition-approval.md](runbooks/data-class-transition-approval.md) - Data Class Transition Approval: operational procedure for incident, rollback, audit, release, or service recovery.
70. [runbooks/demo-environment-reset.md](runbooks/demo-environment-reset.md) - Demo Environment Reset: operational procedure for incident, rollback, audit, release, or service recovery.
71. [runbooks/dep-replacement-execution.md](runbooks/dep-replacement-execution.md) - Dep Replacement Execution: operational procedure for incident, rollback, audit, release, or service recovery.
72. [runbooks/design-partner-feedback-session.md](runbooks/design-partner-feedback-session.md) - Design Partner Feedback Session: operational procedure for incident, rollback, audit, release, or service recovery.
73. [runbooks/design-partner-onboarding.md](runbooks/design-partner-onboarding.md) - Design Partner Onboarding: operational procedure for incident, rollback, audit, release, or service recovery.
74. [runbooks/doc-update-pr.md](runbooks/doc-update-pr.md) - Doc Update Pr: operational procedure for incident, rollback, audit, release, or service recovery.
75. [runbooks/dr-drill-playbook.md](runbooks/dr-drill-playbook.md) - Dr Drill Playbook: operational procedure for incident, rollback, audit, release, or service recovery.
76. [runbooks/dsr-cascade-orchestration.md](runbooks/dsr-cascade-orchestration.md) - DSR Cascade Orchestration: operational procedure for incident, rollback, audit, release, or service recovery.
77. [runbooks/dsr-cascade-proof-of-erasure.md](runbooks/dsr-cascade-proof-of-erasure.md) - DSR Cascade Proof Of Erasure: operational procedure for incident, rollback, audit, release, or service recovery.
78. [runbooks/dsr-cascade-with-evidence.md](runbooks/dsr-cascade-with-evidence.md) - DSR Cascade With Evidence: operational procedure for incident, rollback, audit, release, or service recovery.
79. [runbooks/dsr-compliance-report.md](runbooks/dsr-compliance-report.md) - DSR Compliance Report: operational procedure for incident, rollback, audit, release, or service recovery.
80. [runbooks/employee-dsr-cascade.md](runbooks/employee-dsr-cascade.md) - Employee DSR Cascade: operational procedure for incident, rollback, audit, release, or service recovery.
81. [runbooks/error-budget-exhaustion.md](runbooks/error-budget-exhaustion.md) - Error Budget Exhaustion: operational procedure for incident, rollback, audit, release, or service recovery.
82. [runbooks/esign-failure.md](runbooks/esign-failure.md) - Esign Failure: operational procedure for incident, rollback, audit, release, or service recovery.
83. [runbooks/evidence-pack-generation.md](runbooks/evidence-pack-generation.md) - Evidence Pack Generation: operational procedure for incident, rollback, audit, release, or service recovery.
84. [runbooks/external-dep-onboarding.md](runbooks/external-dep-onboarding.md) - External Dep Onboarding: operational procedure for incident, rollback, audit, release, or service recovery.
85. [runbooks/fhir-resource-dsr.md](runbooks/fhir-resource-dsr.md) - Fhir Resource DSR: operational procedure for incident, rollback, audit, release, or service recovery.
86. [runbooks/finops-monthly-close.md](runbooks/finops-monthly-close.md) - Finops Monthly Close: operational procedure for incident, rollback, audit, release, or service recovery.
87. [runbooks/fintech-payment-failure.md](runbooks/fintech-payment-failure.md) - Fintech Payment Failure: operational procedure for incident, rollback, audit, release, or service recovery.
88. [runbooks/flat-crates-move-pr.md](runbooks/flat-crates-move-pr.md) - Flat Crates Move Pr: operational procedure for incident, rollback, audit, release, or service recovery.
89. [runbooks/forbidden-license-rollback.md](runbooks/forbidden-license-rollback.md) - Forbidden License Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
114. [runbooks/gl-reconciliation.md](runbooks/gl-reconciliation.md) - Gl Reconciliation: operational procedure for incident, rollback, audit, release, or service recovery.
115. [runbooks/glossary-amendment-pr.md](runbooks/glossary-amendment-pr.md) - Glossary Amendment Pr: operational procedure for incident, rollback, audit, release, or service recovery.
117. [runbooks/healthcare-break-glass.md](runbooks/healthcare-break-glass.md) - Healthcare Break Glass: operational procedure for incident, rollback, audit, release, or service recovery.
118. [runbooks/iam-key-rotation.md](runbooks/iam-key-rotation.md) - Iam Key Rotation: operational procedure for incident, rollback, audit, release, or service recovery.
119. [runbooks/identity-provider-federation.md](runbooks/identity-provider-federation.md) - Identity Provider Federation: operational procedure for incident, rollback, audit, release, or service recovery.
120. [runbooks/in-house-replacement-trigger.md](runbooks/in-house-replacement-trigger.md) - In House Replacement Trigger: operational procedure for incident, rollback, audit, release, or service recovery.
121. [runbooks/industrial-ot-write-emergency-stop.md](runbooks/industrial-ot-write-emergency-stop.md) - Industrial Ot Write Emergency Stop: operational procedure for incident, rollback, audit, release, or service recovery.
122. [runbooks/kafka-topic-provisioning.md](runbooks/kafka-topic-provisioning.md) - Kafka Topic Provisioning: operational procedure for incident, rollback, audit, release, or service recovery.
123. [runbooks/kcmvp-hsm-incident.md](runbooks/kcmvp-hsm-incident.md) - Kcmvp Hsm Incident: operational procedure for incident, rollback, audit, release, or service recovery.
124. [runbooks/kyc-review-queue.md](runbooks/kyc-review-queue.md) - KYC Review Queue: operational procedure for incident, rollback, audit, release, or service recovery.
125. [runbooks/legal-corpus-update.md](runbooks/legal-corpus-update.md) - Legal Corpus Update: operational procedure for incident, rollback, audit, release, or service recovery.
126. [runbooks/license-tier-3-review.md](runbooks/license-tier-3-review.md) - License Tier 3 Review: operational procedure for incident, rollback, audit, release, or service recovery.
127. [runbooks/logistics-edi-failure.md](runbooks/logistics-edi-failure.md) - Logistics Edi Failure: operational procedure for incident, rollback, audit, release, or service recovery.
128. [runbooks/machine-readable-mirror-regenerate.md](runbooks/machine-readable-mirror-regenerate.md) - Machine Readable Mirror Regenerate: operational procedure for incident, rollback, audit, release, or service recovery.
129. [runbooks/marketplace-listing-takedown.md](runbooks/marketplace-listing-takedown.md) - Marketplace Listing Takedown: operational procedure for incident, rollback, audit, release, or service recovery.
130. [runbooks/meta-trust-root-recovery.md](runbooks/meta-trust-root-recovery.md) - Meta Trust Root Recovery: operational procedure for incident, rollback, audit, release, or service recovery.
131. [runbooks/og-ciphertext-key-shred.md](runbooks/og-ciphertext-key-shred.md) - Og Ciphertext Key Shred: operational procedure for incident, rollback, audit, release, or service recovery.
132. [runbooks/og-property-tier-migration.md](runbooks/og-property-tier-migration.md) - Og Property Tier Migration: operational procedure for incident, rollback, audit, release, or service recovery.
133. [runbooks/og-rls-policy-regenerate.md](runbooks/og-rls-policy-regenerate.md) - Og Rls Policy Regenerate: operational procedure for incident, rollback, audit, release, or service recovery.
134. [runbooks/og-schema-rollback.md](runbooks/og-schema-rollback.md) - Og Schema Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
135. [runbooks/on-call-handover.md](runbooks/on-call-handover.md) - On Call Handover: operational procedure for incident, rollback, audit, release, or service recovery.
136. [runbooks/opcua-adapter-disconnect.md](runbooks/opcua-adapter-disconnect.md) - Opcua Adapter Disconnect: operational procedure for incident, rollback, audit, release, or service recovery.
137. [runbooks/ops/dr-drill-runbook.md](runbooks/ops/dr-drill-runbook.md) - Dr Drill Runbook: operational procedure for incident, rollback, audit, release, or service recovery.
138. [runbooks/ops/game-day-procedure.md](runbooks/ops/game-day-procedure.md) - Game Day Procedure: operational procedure for incident, rollback, audit, release, or service recovery.
139. [runbooks/ops/regulator-notification-procedure.md](runbooks/ops/regulator-notification-procedure.md) - Regulator Notification Procedure: operational procedure for incident, rollback, audit, release, or service recovery.
140. [runbooks/ops/sev-1-bridge-procedure.md](runbooks/ops/sev-1-bridge-procedure.md) - Sev 1 Bridge Procedure: operational procedure for incident, rollback, audit, release, or service recovery.
141. [runbooks/ops/trust-portal-publish-procedure.md](runbooks/ops/trust-portal-publish-procedure.md) - Trust Portal Publish Procedure: operational procedure for incident, rollback, audit, release, or service recovery.
142. [runbooks/outbox-poller-recovery.md](runbooks/outbox-poller-recovery.md) - Outbox Poller Recovery: operational procedure for incident, rollback, audit, release, or service recovery.
143. [runbooks/outbox-relay-lag.md](runbooks/outbox-relay-lag.md) - Outbox Relay Lag: operational procedure for incident, rollback, audit, release, or service recovery.
144. [runbooks/pack-onboarding.md](runbooks/pack-onboarding.md) - Pack Onboarding: operational procedure for incident, rollback, audit, release, or service recovery.
145. [runbooks/pack-version-upgrade.md](runbooks/pack-version-upgrade.md) - Pack Version Upgrade: operational procedure for incident, rollback, audit, release, or service recovery.
146. [runbooks/partner-contract-renewal.md](runbooks/partner-contract-renewal.md) - Partner Contract Renewal: operational procedure for incident, rollback, audit, release, or service recovery.
147. [runbooks/payroll-run-failure.md](runbooks/payroll-run-failure.md) - Payroll Run Failure: operational procedure for incident, rollback, audit, release, or service recovery.
148. [runbooks/per-cell-broker-failover.md](runbooks/per-cell-broker-failover.md) - Per Cell Broker Failover: operational procedure for incident, rollback, audit, release, or service recovery.
149. [runbooks/per-cell-hsm-rotation.md](runbooks/per-cell-hsm-rotation.md) - Per Cell Hsm Rotation: operational procedure for incident, rollback, audit, release, or service recovery.
150. [runbooks/per-context-flatten-phase.md](runbooks/per-context-flatten-phase.md) - Per Context Flatten Phase: operational procedure for incident, rollback, audit, release, or service recovery.
151. [runbooks/plane-class-correction.md](runbooks/plane-class-correction.md) - Plane Class Correction: operational procedure for incident, rollback, audit, release, or service recovery.
152. [runbooks/plugin-sandbox-escape.md](runbooks/plugin-sandbox-escape.md) - Plugin Sandbox Escape: operational procedure for incident, rollback, audit, release, or service recovery.
153. [runbooks/preview-to-stable-promotion.md](runbooks/preview-to-stable-promotion.md) - Preview To Stable Promotion: operational procedure for incident, rollback, audit, release, or service recovery.
154. [runbooks/privacy-council-data-class-review.md](runbooks/privacy-council-data-class-review.md) - Privacy Council Data Class Review: operational procedure for incident, rollback, audit, release, or service recovery.
155. [runbooks/provider-credential-leak-response.md](runbooks/provider-credential-leak-response.md) - Provider Credential Leak Response: operational procedure for incident, rollback, audit, release, or service recovery.
156. [runbooks/region-failover.md](runbooks/region-failover.md) - Region Failover: operational procedure for incident, rollback, audit, release, or service recovery.
157. [runbooks/regulator-evidence-pack-regen.md](runbooks/regulator-evidence-pack-regen.md) - Regulator Evidence Pack Regen: operational procedure for incident, rollback, audit, release, or service recovery.
158. [runbooks/regulator-publication-feed-health.md](runbooks/regulator-publication-feed-health.md) - Regulator Publication Feed Health: operational procedure for incident, rollback, audit, release, or service recovery.
159. [runbooks/regulatory-change-response.md](runbooks/regulatory-change-response.md) - Regulatory Change Response: operational procedure for incident, rollback, audit, release, or service recovery.
160. [runbooks/regulatory-relationship-escalation.md](runbooks/regulatory-relationship-escalation.md) - Regulatory Relationship Escalation: operational procedure for incident, rollback, audit, release, or service recovery.
161. [runbooks/regulatory-replay.md](runbooks/regulatory-replay.md) - Regulatory Replay: operational procedure for incident, rollback, audit, release, or service recovery.
162. [runbooks/release-rollback.md](runbooks/release-rollback.md) - Release Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
163. [runbooks/saas/marketplace-listing-takedown.md](runbooks/saas/marketplace-listing-takedown.md) - Marketplace Listing Takedown: operational procedure for incident, rollback, audit, release, or service recovery.
164. [runbooks/saas/plugin-runtime-sandbox-escape.md](runbooks/saas/plugin-runtime-sandbox-escape.md) - Plugin Runtime Sandbox Escape: operational procedure for incident, rollback, audit, release, or service recovery.
165. [runbooks/saas/workflow-engine-deadlock.md](runbooks/saas/workflow-engine-deadlock.md) - Workflow Engine Deadlock: operational procedure for incident, rollback, audit, release, or service recovery.
166. [runbooks/sanctioned-primitives/preflight.md](runbooks/sanctioned-primitives/preflight.md) - Preflight: operational procedure for incident, rollback, audit, release, or service recovery.
167. [runbooks/sbom-regenerate.md](runbooks/sbom-regenerate.md) - Sbom Regenerate: operational procedure for incident, rollback, audit, release, or service recovery.
168. [runbooks/sdk-regen-failure.md](runbooks/sdk-regen-failure.md) - Sdk Regen Failure: operational procedure for incident, rollback, audit, release, or service recovery.
169. [runbooks/sdk-release.md](runbooks/sdk-release.md) - Sdk Release: operational procedure for incident, rollback, audit, release, or service recovery.
170. [runbooks/search/crawler-blocked-by-host.md](runbooks/search/crawler-blocked-by-host.md) - Crawler Blocked By Host: operational procedure for incident, rollback, audit, release, or service recovery.
171. [runbooks/search/index-corruption.md](runbooks/search/index-corruption.md) - Index Corruption: operational procedure for incident, rollback, audit, release, or service recovery.
172. [runbooks/search/rtbf-cascade.md](runbooks/search/rtbf-cascade.md) - Rtbf Cascade: operational procedure for incident, rollback, audit, release, or service recovery.
173. [runbooks/search/serp-quality-regression.md](runbooks/search/serp-quality-regression.md) - Serp Quality Regression: operational procedure for incident, rollback, audit, release, or service recovery.
174. [runbooks/search-index-dsr-cascade.md](runbooks/search-index-dsr-cascade.md) - Search Index DSR Cascade: operational procedure for incident, rollback, audit, release, or service recovery.
175. [runbooks/security-incident-response.md](runbooks/security-incident-response.md) - Security Incident Response: operational procedure for incident, rollback, audit, release, or service recovery.
176. [runbooks/self-modification-rollback.md](runbooks/self-modification-rollback.md) - Self Modification Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
177. [runbooks/serp-sponsored-slot-failure.md](runbooks/serp-sponsored-slot-failure.md) - Serp Sponsored SLOt Failure: operational procedure for incident, rollback, audit, release, or service recovery.
178. [runbooks/sev1-incident-response.md](runbooks/sev1-incident-response.md) - Sev1 Incident Response: operational procedure for incident, rollback, audit, release, or service recovery.
179. [runbooks/shamir-share-loss-or-coercion.md](runbooks/shamir-share-loss-or-coercion.md) - Shamir Share Loss Or Coercion: operational procedure for incident, rollback, audit, release, or service recovery.
180. [runbooks/sub-axis-promotion.md](runbooks/sub-axis-promotion.md) - Sub Axis Promotion: operational procedure for incident, rollback, audit, release, or service recovery.
181. [runbooks/supply-chain-compromise.md](runbooks/supply-chain-compromise.md) - Supply Chain Compromise: operational procedure for incident, rollback, audit, release, or service recovery.
182. [runbooks/supply-chain-trivy-alert.md](runbooks/supply-chain-trivy-alert.md) - Supply Chain Trivy Alert: operational procedure for incident, rollback, audit, release, or service recovery.
183. [runbooks/tenant-data-residency-violation.md](runbooks/tenant-data-residency-violation.md) - Tenant Data Residency Violation: operational procedure for incident, rollback, audit, release, or service recovery.
184. [runbooks/tenant-escalation-management.md](runbooks/tenant-escalation-management.md) - Tenant Escalation Management: operational procedure for incident, rollback, audit, release, or service recovery.
185. [runbooks/tenant-onboarding.md](runbooks/tenant-onboarding.md) - Tenant Onboarding: operational procedure for incident, rollback, audit, release, or service recovery.
186. [runbooks/term-deprecation-protocol.md](runbooks/term-deprecation-protocol.md) - Term Deprecation Protocol: operational procedure for incident, rollback, audit, release, or service recovery.
187. [runbooks/topic-schema-rollback.md](runbooks/topic-schema-rollback.md) - Topic Schema Rollback: operational procedure for incident, rollback, audit, release, or service recovery.
188. [runbooks/vertical-fintech/aml-rule-fired.md](runbooks/vertical-fintech/aml-rule-fired.md) - Aml Rule Fired: operational procedure for incident, rollback, audit, release, or service recovery.
189. [runbooks/vertical-fintech/cde-isolation-breach.md](runbooks/vertical-fintech/cde-isolation-breach.md) - Cde Isolation Breach: operational procedure for incident, rollback, audit, release, or service recovery.
190. [runbooks/vertical-fintech/pci-incident-suspected.md](runbooks/vertical-fintech/pci-incident-suspected.md) - PCI Incident Suspected: operational procedure for incident, rollback, audit, release, or service recovery.
191. [runbooks/vertical-healthcare/clinical-safety-anomaly.md](runbooks/vertical-healthcare/clinical-safety-anomaly.md) - Clinical Safety Anomaly: operational procedure for incident, rollback, audit, release, or service recovery.
192. [runbooks/vertical-healthcare/phi-leak-suspected.md](runbooks/vertical-healthcare/phi-leak-suspected.md) - Phi Leak Suspected: operational procedure for incident, rollback, audit, release, or service recovery.
193. [runbooks/vertical-industrial/ot-safety-anomaly.md](runbooks/vertical-industrial/ot-safety-anomaly.md) - Ot Safety Anomaly: operational procedure for incident, rollback, audit, release, or service recovery.
194. [runbooks/vertical-logistics/edi-counterparty-down.md](runbooks/vertical-logistics/edi-counterparty-down.md) - Edi Counterparty Down: operational procedure for incident, rollback, audit, release, or service recovery.
195. [runbooks/vertical-pilot-wave-gate-readiness.md](runbooks/vertical-pilot-wave-gate-readiness.md) - Vertical Pilot Wave Gate Readiness: operational procedure for incident, rollback, audit, release, or service recovery.
196. [runbooks/wave-gate-evaluation.md](runbooks/wave-gate-evaluation.md) - Wave Gate Evaluation: operational procedure for incident, rollback, audit, release, or service recovery.
197. [runbooks/wave-gate-readiness-check.md](runbooks/wave-gate-readiness-check.md) - Wave Gate Readiness Check: operational procedure for incident, rollback, audit, release, or service recovery.
198. [runbooks/webhook-delivery-failure.md](runbooks/webhook-delivery-failure.md) - Webhook Delivery Failure: operational procedure for incident, rollback, audit, release, or service recovery.
199. [runbooks/workflow-engine-restart.md](runbooks/workflow-engine-restart.md) - Workflow Engine Restart: operational procedure for incident, rollback, audit, release, or service recovery.
200. [runbooks/workspace/doc-crdt-divergence.md](runbooks/workspace/doc-crdt-divergence.md) - Doc Crdt Divergence: operational procedure for incident, rollback, audit, release, or service recovery.
201. [runbooks/workspace/drive-permission-escalation.md](runbooks/workspace/drive-permission-escalation.md) - Drive Permission Escalation: operational procedure for incident, rollback, audit, release, or service recovery.
202. [runbooks/workspace/mail-deliverability-collapse.md](runbooks/workspace/mail-deliverability-collapse.md) - Mail Deliverability Collapse: operational procedure for incident, rollback, audit, release, or service recovery.
203. [runbooks/workspace/meet-sfu-failover.md](runbooks/workspace/meet-sfu-failover.md) - Meet Sfu Failover: operational procedure for incident, rollback, audit, release, or service recovery.
204. [runbooks/workspace/recording-archiver-stuck.md](runbooks/workspace/recording-archiver-stuck.md) - Recording Archiver Stuck: operational procedure for incident, rollback, audit, release, or service recovery.
205. [runbooks/workspace-members-merge-queue.md](runbooks/workspace-members-merge-queue.md) - Workspace Members Merge Queue: operational procedure for incident, rollback, audit, release, or service recovery.

---

Hub checkpoint: this file was gardened on 2026-05-20 under Oya VCS agent `codex-docs-hub-gardening`. Stop condition for this pass is a substantive `docs/README.md` at or above 1500 lines, with every Markdown link in this file resolving to an existing file, directory, or local section.

Known non-edit note: `microservices/intelligence/spec/` was requested as a link target but is absent in this checkout. This hub does not create or edit microservice files; it routes Foundry readers to existing `microservices/intelligence/PRD.md`, `ARCHITECTURE.md`, `manifest.json`, service ADRs, catalogs, tutorials, and implementation plans instead.
