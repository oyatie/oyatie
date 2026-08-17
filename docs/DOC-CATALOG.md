---
purpose: "Legacy projection of the pre-PHASE-5 doc catalog and update protocol"
doc_status: legacy_projection
---

# Doc Catalog & Update Protocol

## Historical doctrinal inputs — [decision-principles.json](../specs/decision-principles.json) + [forbidden-operations.json](../specs/forbidden-operations.json)


> **Status:** Legacy, non-authoritative projection — 2026-07-13. This file preserves the
> pre-PHASE-5 catalog design for provenance and migration input. It is not a live lifecycle protocol,
> gate inventory, or sequencing authority. Current routing comes from
> [`/specs/root-hub-pointers.json`](../specs/root-hub-pointers.json); Markdown lifecycle comes from
> [`/specs/markdown-retirement-policy.json`](../specs/markdown-retirement-policy.json); execution
> sequencing comes only from [`/specs/masterplan.json#masterplan_v2`](../specs/masterplan.json).
> Promotion requires the PHASE-5 schema, producer, consumers, and cross-artifact enforcement to land
> atomically. Historical imperative language below is nonbinding until that promotion evidence exists.

---

## 0. Reading guide

Each doc has a row in §2 with these columns:

| Column | Meaning |
|---|---|
| `id` | Stable doc identifier (used by JSON catalog and agent tooling) |
| `path` | File path |
| `owner_team` | Team-charter ID owning the doc |
| `update_trigger` | The event(s) that obligate an update |
| `update_cadence` | The latest schedule for a refresh even absent triggers |
| `dependent_docs` | Other docs that MUST be re-read or re-authored when this doc changes |
| `validation_check` | The CI / agent / reviewer check that must pass after an update |
| `agent_authoring_allowed` | Whether agents may author updates without human review (rare) |

The checked-in [`machine-readable/catalog.json`](machine-readable/catalog.json) is a historical
mirror of this legacy projection. It is not the promised live `/registry/doc-catalog.json` producer
or lifecycle authority.

---

## 1. Update-triggering events (the "when")

Each event below maps to specific docs. The §2 rows enumerate the docs per event.

| Event ID | Description |
|---|---|
| `EVT-AXIS-SCOPE-CHANGE` | Any of the 7 axes changes scope (in-scope item moves to out-of-scope, or vice versa). |
| `EVT-AXIS-CONTRACT-CHANGE` | Any inter-axis contract row in DESIGN §10 is added, modified, or removed. |
| `EVT-ADR-PROMOTED` | An ADR moves from Proposed → Accepted (or any other status transition). |
| `EVT-ADR-AUTHORED` | A new ADR is drafted (Proposed). |
| `EVT-CAPABILITY-AUTHORED` | A new capability is registered (catalog/registry/capabilities/). |
| `EVT-CONTRACT-AUTHORED` | A new contract (OpenAPI / proto / event-schema) is added under `contracts/`. |
| `EVT-FLAT-CRATE-MOVED` | An ADR-0015 / Issue #1458 phase PR lands. |
| `EVT-AUDIT-FINDING` | A new audit recommendation is published (audits/, security review). |
| `EVT-INCIDENT-CLOSED` | A Sev-1/2 incident is closed; postmortem published. |
| `EVT-REGULATORY-CHANGE` | A regulator (KISA, MFDS, FSC, KCC, NIS, foreign equiv) issues a new control or revision. |
| `EVT-VERTICAL-ADDED` | A new vertical product is approved by Architecture Council. |
| `EVT-TENANT-CLASS-ADDED` | A new tenant class override is added (Privacy Council). |
| `EVT-WAVE-GATE-PASSED` | A wave gate per PRD §3.1 (W-Foundation, W-Foundry-Preview, W-Foundry-Preview, W-Cloud-Preview, W-SaaS-Preview, W-Search-Preview, W-Vertical-Pilot, W-Vertical-Fan-Out, W-Cloud-Stable, W-Search-Stable, W-Ads-Preview, W-Ads-Stable, W-Region-Expansion) passes its readiness pack. |
| `EVT-FOUNDRY-CAPABILITY-PROMOTED` | A capability promotes from preview → stable in Foundry. |
| `EVT-RENAME-PHASE-PASSED` | A brand-rename phase (per ADR-0017 PG-0a) completes. |
| `EVT-RISK-MATERIALIZED` | A row in RISK-REGISTER changes severity or owner. |
| `EVT-DSR-CASCADE-RUN` | A DSR cascade completes; proof-of-erasure published. |
| `EVT-PRICING-CHANGE` | Any product pricing or packaging changes. |
| `EVT-HIRE-NEW-TEAM-LEAD` | A new team-lead hire — charter handoff required. |

---

## 2. The catalog

> Historical path conventions: rooted at `docs/`. Owner team IDs were intended to match
> `teams/<team-id>/CHARTER.md`. Validation names in this projection are design inventory only; their
> presence does not prove a live gate or producer.

### 2.1 Tier 1 — Strategy / Architecture / Compliance

| id | path | owner_team | update_trigger | update_cadence | dependent_docs | validation_check | agent_authoring_allowed |
|---|---|---|---|---|---|---|---|
| `doc.masterplan` | `MASTERPLAN.md` | `council-architecture` | master-plan authority or sequencing change | per change + quarterly | PRD.md, DESIGN.md, ROADMAP.md, RACI-OWNERSHIP.md, RISK-REGISTER.md | `master-plan-completion`, `doc-catalog-self-coverage` | NO |
| `doc.foundry_supervisor_readme` | `docs/foundry/supervisor/README.md` | `axis-foundry` | architecture change | quarterly | RACI-OWNERSHIP.md | `doc-catalog-self-coverage` | YES |
| `doc.foundry_supervisor_arch` | `docs/foundry/supervisor/architecture.md` | `axis-foundry` | 4-crate boundary change | quarterly | DESIGN.md | `doc-catalog-self-coverage` | NO |
| `doc.foundry_supervisor_ops` | `docs/foundry/supervisor/operations.md` | `axis-foundry` | signal/lifecycle change | quarterly | - | `doc-catalog-self-coverage` | YES |
| `doc.foundry_supervisor_security` | `docs/foundry/supervisor/security.md` | `axis-foundry` | secret-ref or tier change | quarterly | SECURITY-PLAN.md | `doc-catalog-self-coverage` | NO |
| `doc.foundry_supervisor_samples` | `docs/foundry/supervisor/sample-payloads.md` | `axis-foundry` | contract/schema change | quarterly | contracts/ | `doc-catalog-self-coverage` | YES |
| `doc.decision_principles` | `/specs/decision-principles.json` | `council-architecture` | doctrinal authority change | quarterly | AGENTS.md, DESIGN.md, DOC-CATALOG.md | `authority-cohesion` | NO |
| `doc.forbidden_operations` | `/specs/forbidden-operations.json` | `council-architecture` | doctrinal prohibition change | quarterly | AGENTS.md, DESIGN.md, DOC-CATALOG.md | `authority-cohesion` | NO |
| `doc.spec_agent_durable_goal` | `/specs/agent-durable-goal.json` | `council-architecture + axis-foundry` | autonomous-Foundry contract change (review tiering, score cards, source/doubt-driven, autonomy ceiling) | quarterly | AGENTS.md, DESIGN.md, DOC-CATALOG.md | `authority-cohesion`, `spec-contract-mirror` | NO |
| `doc.spec_decision_rights` | `/specs/decision-rights.json` | `council-architecture` | decision-class authority change | quarterly | AGENTS.md, DESIGN.md, RACI-OWNERSHIP.md | `authority-cohesion` | NO |
| `doc.spec_governance_amendment` | `/specs/governance-amendment.json` | `council-architecture` | amendment procedure change | quarterly | decision-principles.json, forbidden-operations.json | `authority-cohesion` | NO |
| `doc.spec_oyatie_doctrine` | `/specs/oyatie-doctrine.json` | `council-architecture` | repository_layout / BNF / 12-layer enum change | quarterly | DESIGN.md, ADR-INDEX.md | `authority-cohesion`, `spec-contract-mirror` | NO |
| `doc.spec_masterplan` | `/specs/masterplan.json` | `council-architecture` | milestone/phase/IP topology change | per event | ROADMAP.md, MASTERPLAN.md (compatibility projection only); historical `.omc/plans/**` is provenance not dependent authority | `masterplan-coherence`, `spec-contract-mirror` | NO |
| `doc.spec_master_plan_sequencing` | `/specs/master-plan-sequencing.json` | `council-architecture` | historical sequencing-sidecar change | provenance only | masterplan.json, gitops-vcs-replacement.json | historical validation inventory only | NO |
| `doc.spec_root_hub_pointers` | `/specs/root-hub-pointers.json` | `council-architecture` | canonical entry-point change | per event | README.md, CLAUDE.md, AGENTS.md | `authority-cohesion` | NO |
| `doc.spec_active_machine_readable_artifact_contract` | `/specs/active-machine-readable-artifact-contract.json` | `council-architecture` | active-artifact capability contract change (ADR-0069) | per event | artifact-profile-defaults.json | `spec-contract-mirror` | NO |
| `doc.spec_artifact_profile_defaults` | `/specs/artifact-profile-defaults.json` | `council-architecture` | per-profile defaults change | per event | active-machine-readable-artifact-contract.json | `spec-contract-mirror` | NO |
| `doc.spec_plan_schema` | `/specs/plan-schema.json` | `council-architecture` | ralplan consensus-plan schema change | per event | masterplan.json | `spec-contract-mirror` | NO |
| `doc.spec_evidence_taxonomy` | `/specs/evidence-taxonomy.json` | `council-architecture` | evidence class / minimum_completion_set change | per event | audit-chain emitters | `spec-contract-mirror` | NO |
| `doc.spec_hyperscaler_gates` | `/specs/hyperscaler-gates.json` | `council-architecture` | HG-* gate change | per event | masterplan.json, oya-governance-* lanes | `spec-contract-mirror` | NO |
| `doc.spec_stop_conditions` | `/specs/stop-conditions.json` | `council-architecture` | SC-* condition change | per event | autonomous_foundry.first_of_kind_protocol | `spec-contract-mirror` | NO |
| `doc.spec_final_report_schema` | `/specs/final-report-schema.json` | `council-architecture` | MPR final-report schema change | per event | autonomous master-plan loop | `spec-contract-mirror` | NO |
| `doc.spec_test_standard` | `/specs/test-standard.json` | `council-architecture + axis-foundry` | test class / coverage rule change | per event | agent-durable-goal.json tdd_contract | `spec-contract-mirror` | NO |
| `doc.spec_iterative_fix_loop` | `/specs/iterative-fix-loop.json` | `axis-foundry-vcs` | iterative-fix-loop semantics change | per event | gitops-vcs-replacement.json, ci-fix-loop-context-bundle.json | `spec-contract-mirror` | NO |
| `doc.spec_ci_fix_loop_context_bundle` | `/specs/ci-fix-loop-context-bundle.json` | `axis-foundry-vcs` | CI-fix context-bundle shape change | per event | iterative-fix-loop.json | `spec-contract-mirror` | NO |
| `doc.spec_merge_queue_parked_pr` | `/specs/merge-queue-parked-pr.json` | `axis-foundry-vcs` | merge-queue parked-PR semantics change | per event | gitops-vcs-replacement.json | `spec-contract-mirror` | NO |
| `doc.spec_gitops_vcs_replacement` | `/specs/gitops-vcs-replacement.json` | `council-architecture + axis-foundry-vcs` | Foundry VCS / pipeline doctrine change | per event | masterplan.json, ADR-0110/0111/0112/0113 | `spec-contract-mirror`, `authority-cohesion` | NO |
| `doc.spec_markdown_retirement_policy` | `/specs/markdown-retirement-policy.json` | `council-architecture` | markdown-retirement-phase change | per event | ADR-0054, ADR-0119 | `spec-contract-mirror` | NO |
| `doc.spec_crate_naming_audit` | `/specs/crate-naming-audit.json` | `council-architecture` | naming audit / retired_package_notes change | per event | docs/standards/crate-naming-convention.md, ADR-0056 | `spec-contract-mirror` | NO |
| `doc.spec_knowledge_graph_schema` | `/specs/knowledge-graph-schema.json` | `council-architecture` | knowledge-graph schema delta | per event | registry/knowledge-graph-*.json | `spec-contract-mirror` | NO |
| `doc.spec_codeview_read_surface` | `/specs/codeview-read-surface.json` | `council-architecture` | code-view read-surface change | per event | oya gate validate codeview-read-surface | `spec-contract-mirror` | NO |
| `doc.agents` | `AGENTS.md` | `axis-foundry` + `council-architecture` | agent operating-contract change | quarterly | decision-principles.json, forbidden-operations.json, DESIGN.md, DOC-CATALOG.md | `authority-cohesion` | NO |
| `doc.agents_operating_contract_refs` | `AGENTS-OPERATING-CONTRACT.md` | `axis-foundry` + `council-architecture` | Wave 15-ZF operating-contract doctrine propagation | per ADR-0346..ADR-0349 doctrine update | AGENTS.md | `doc-catalog-self-coverage`, `authority-cohesion` | NO |
| `doc.prd_oyatie_from_scratch_canonical` | `PRD-OYATIE-FROM-SCRATCH-CANONICAL.md` | `founder` + `product-council` + `architecture-council` | from-scratch handoff PRD change | per handoff revision | PRD.md, DESIGN.md | `doc-catalog-self-coverage`, `readme-doc-coverage` | NO |
| `doc.prd` | `PRD.md` | `council-architecture` | EVT-AXIS-SCOPE-CHANGE, EVT-PRICING-CHANGE, EVT-VERTICAL-ADDED | quarterly | DESIGN.md, ROADMAP.md, GTM-PLAN.md, products/*/PRD.md | `prd-internal-consistency`, `prd-axis-coverage`, `prd-glossary-alignment` | NO — council-only |
| `doc.design` | `DESIGN.md` | `council-architecture` | EVT-AXIS-CONTRACT-CHANGE, EVT-ADR-PROMOTED (cross-cutting axis), EVT-FLAT-CRATE-MOVED (target-shape change only) | monthly | SPEC.md, ROADMAP.md, ADR-INDEX.md, contracts.json | `design-contracts-mirror`, `design-vs-adr-cite-coverage` | NO |
| `doc.spec` | `SPEC.md` | `platform-api-sdk` | EVT-CONTRACT-AUTHORED, EVT-CAPABILITY-AUTHORED, EVT-AXIS-CONTRACT-CHANGE | weekly | DESIGN.md, products/*/PRD.md, machine-readable/contracts.json | `spec-contract-mirror`, `spec-capability-coverage` | YES — agent may auto-PR for additions only; deletions need human review |
| `doc.roadmap` | `ROADMAP.md` | `tactical-m3-launch` (until [wave name per PRD §3.1]); thereafter rolling | EVT-WAVE-GATE-PASSED, EVT-FOUNDRY-CAPABILITY-PROMOTED, EVT-AUDIT-FINDING (P0/P1) | bi-weekly | PRD.md, batches.json, RISK-REGISTER.md | `roadmap-band-totals`, `roadmap-foundry-batch-shape` | YES — agent may rebalance bands; band-promotion requires human |
| `doc.adr_index` | `ADR-INDEX.md` | `crew-adr-promotion` | EVT-ADR-AUTHORED, EVT-ADR-PROMOTED | per event | DESIGN.md, machine-readable/decisions.json | `adr-index-completeness`, `adr-supersession-graph` | YES — agent re-emits index from `decisions/` directory |
| `doc.adr_0096` | `decisions/ADR-0096-supervisor-language-rust-not-node.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0097` | `decisions/ADR-0097-intelligence-account-adapter-rename-target-slot-last.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0098` | `decisions/ADR-0098-supervisor-dep-policy-Y-zero-deps-best-effort-durability.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0099` | `decisions/ADR-0099-cedar-policy-extend-supervisor-capabilities.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0100` | `decisions/ADR-0100-supervisor-public-contract-lean-a10.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0101` | `decisions/ADR-0101-supervisor-mountpoint-direct-hyper.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0102` | `decisions/ADR-0102-intelligence-settings-template-canonical-rendering.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0104` | `decisions/ADR-0104-ecosystem-expansion-toolchain-and-adapters.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0105` | `decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0106` | `decisions/ADR-0106-rename-application-to-usecase.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0107` | `decisions/ADR-0107-tools-implicit-app-convention.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0108` | `decisions/ADR-0108-sunset-lifecycle-automation.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0109` | `decisions/ADR-0109-lifecycle-automation-framework.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0110` | `decisions/ADR-0110-changeset-state-machine.md` | `council-foundry-vcs` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0111` | `decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md` | `council-foundry-vcs` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0112` | `decisions/ADR-0112-webhook-driven-intelligence-agent-invocation.md` | `council-foundry-vcs` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0113` | `decisions/ADR-0113-vcs-orchestrator-end-to-end.md` | `council-foundry-vcs` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0114` | `decisions/ADR-0114-canary-observability-rollback.md` | `council-foundry-vcs` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0115` | `decisions/ADR-0115-registry-consolidation-flat-singular.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0116` | `decisions/ADR-0116-retire-external-agent-coordination-tooling.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0117` | `decisions/ADR-0117-repo-hygiene-gitignore-audit-config-and-kyverno-consolidation.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0118` | `decisions/ADR-0118-retire-archive-orphan-fitness-lane.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0119` | `decisions/ADR-0119-specs-flat-root-topology.md` | `council-architecture + council-foundry-vcs` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0120` | `decisions/ADR-0120-rust-first-onprem-tooling-with-paired-uninstall.md` | `council-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0121` | `decisions/ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy.md` | `axis-cloud + axis-foundry` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.adr_0122` | `decisions/ADR-0122-ontology-crate-rename-from-object-graph.md` | `council-architecture` | EVT-ADR-AUTHORED | per event | ADR-INDEX.md | `adr-index-completeness` | NO |
| `doc.prd_workflow` | `/specs/microservices/workflow.json` | `council-architecture + axis-foundry` | workflow PRD change (engine half of ecosystem-backbone pair) | per event | workflow-studio PRD, ontology PRD, contracts/workflow_spec.v1.json | `spec-contract-mirror`, `authority-cohesion` | NO |
| `doc.prd_workflow_studio` | `/specs/microservices/workflow-studio.json` | `council-design-system + workflow-team` | workflow-studio PRD change (visual editor + DSL half) | per event | workflow PRD, design-system specs | `spec-contract-mirror`, `authority-cohesion` | NO |
| `doc.prd_ontology` | `/specs/microservices/ontology.json` | `council-architecture + ontology-team` | ontology+KG PRD change (typed-entity backbone + 3-layer runtime) | per event | knowledge-graph specs, Bominal-ADR-0106/0107/0133 | `spec-contract-mirror`, `authority-cohesion` | NO |
| `doc.audit_kg_robustness` | `/registry/milestone-audit/index.json` | `council-architecture + ontology-team` | historical KG audit planning row change; no standalone KG audit registry is promoted until backed by schema/owner/gate | quarterly | knowledge-graph specs, ontology PRD | `spec-contract-mirror` | NO |
| `doc.adr_consolidation_plan` | `ADR-CONSOLIDATION-PLAN.md` | `crew-adr-promotion` | ADR consolidation strategy change | per event | ADR-INDEX.md, DESIGN.md | `adr-index-completeness` | NO |
| `doc.adr_legacy_regression_mapping` | `ADR-LEGACY-REGRESSION-MAPPING.md` | `crew-adr-promotion` | legacy ADR regression discovered or retired | per event | ADR-INDEX.md, DESIGN.md | `adr-index-completeness` | NO |
| `doc.risk_register` | `RISK-REGISTER.md` | `council-architecture` | EVT-RISK-MATERIALIZED, EVT-INCIDENT-CLOSED, EVT-AUDIT-FINDING | weekly | ROADMAP.md, machine-readable/risks.json | `risk-register-coverage` | YES for low/med; NO for catastrophic |
| `doc.contradiction_ledger` | `CONTRADICTION-LEDGER.md` | `council-architecture` | contradiction opened, resolved, or escalated | weekly | DESIGN.md, RISK-REGISTER.md | `risk-register-coverage` | NO |
| `doc.compliance_matrix` | `COMPLIANCE-MATRIX.md` | `ops-compliance` | EVT-REGULATORY-CHANGE, EVT-AUDIT-FINDING | monthly | security-program/security-program.json, PRIVACY-PROGRAM.md, machine-readable/compliance.json | `compliance-matrix-coverage`, `compliance-evidence-recency` | NO |
| `doc.security_program` | `security-program/security-program.json` | `ops-security` | EVT-REGULATORY-CHANGE, EVT-AUDIT-FINDING, EVT-INCIDENT-CLOSED (security-class) | quarterly | COMPLIANCE-MATRIX.md, INCIDENT-MANAGEMENT.md | `security-controls-coverage` | NO |
| `doc.privacy_program` | `PRIVACY-PROGRAM.md` | `council-privacy` | EVT-DSR-CASCADE-RUN, EVT-REGULATORY-CHANGE, EVT-TENANT-CLASS-ADDED | monthly | DESIGN.md, COMPLIANCE-MATRIX.md, ADR-0008 (Data Use Boundary) | `privacy-class-taxonomy-coverage`, `privacy-consent-flow-completeness` | NO |
| `doc.gtm_plan` | `GTM-PLAN.md` | `gtm-sales-se` | EVT-PRICING-CHANGE, EVT-AXIS-SCOPE-CHANGE, EVT-WAVE-GATE-PASSED | monthly | PRD.md, products/*/PRD.md, ROADMAP.md | `gtm-pricing-coverage` | NO |
| `doc.competitive_gap_analysis` | `COMPETITIVE-GAP-ANALYSIS.md` | `council-architecture` | competitive gap or scope constraint changes | quarterly | PRD.md, ROADMAP.md | `prd-axis-coverage` | NO |

### 2.2 Tier 2 — Operations / Delivery

| id | path | owner_team | update_trigger | update_cadence | dependent_docs | validation_check | agent_authoring_allowed |
|---|---|---|---|---|---|---|---|
| `doc.runbooks_index` | `RUNBOOKS-INDEX.md` | `ops-sre-reliability` | new runbook authored, runbook deprecation | weekly | SLO-CATALOG.md, INCIDENT-MANAGEMENT.md | `runbook-discoverability`, `runbook-orphan-check` | YES |
| `doc.slo_catalog` | `SLO-CATALOG.md` | `ops-sre-reliability` | new surface, SLO drift, error-budget exhaustion | weekly | RELEASE-MANAGEMENT.md (burn-rate gate), DESIGN.md plane § | `slo-surface-coverage` | YES |
| `doc.release_management` | `RELEASE-MANAGEMENT.md` | `ops-sre-reliability` + `axis-foundry` | new CI lane, gate ratchet, rollout strategy change | monthly | SLO-CATALOG.md, QA-TEST-STRATEGY.md, ADR-0050/0188 | `release-lane-coverage` | NO |
| `doc.qa_test_strategy` | `QA-TEST-STRATEGY.md` | `axis-foundry` | new test class, fixture-discipline change | quarterly | RELEASE-MANAGEMENT.md | `qa-coverage-by-class` | NO |
| `doc.raci_ownership` | `RACI-OWNERSHIP.md` | `council-architecture` | new team, surface owner change, decision-rights matrix update | quarterly | teams/*/CHARTER.md, CODEOWNERS | `raci-team-coverage`, `codeowners-mirror` | YES — agent may sync from CODEOWNERS |
| `doc.incident_management` | `INCIDENT-MANAGEMENT.md` | `ops-sre-reliability` | EVT-INCIDENT-CLOSED, severity taxonomy change | per Sev-1/2 + quarterly | RUNBOOKS-INDEX.md, security-program/security-program.json | `incident-template-completeness` | NO |

### 2.3 Tier 3 — Business / Resourcing

| id | path | owner_team | update_trigger | update_cadence | dependent_docs | validation_check | agent_authoring_allowed |
|---|---|---|---|---|---|---|---|
| `doc.hiring_capacity_plan` | `HIRING-CAPACITY-PLAN.md` | `council-architecture` (until founder hires CFO/COO) | EVT-HIRE-NEW-TEAM-LEAD, quarterly capacity review | quarterly | RACI-OWNERSHIP.md, FINOPS-PLAN.md | `hiring-axis-coverage` | NO |
| `doc.finops_plan` | `FINOPS-PLAN.md` | `ops-finops` | EVT-PRICING-CHANGE, capacity-cost ≥ 10% drift | monthly | SLO-CATALOG.md, ROADMAP.md, machine-readable/products.json | `finops-margin-coverage` | YES (re-pull cost numbers) |
| `doc.vendor_partner_ledger` | `VENDOR-PARTNER-LEDGER.md` | `gtm-partnerships` + `ops-security` | new vendor onboarded, contract expiring < 90 days | quarterly | RISK-REGISTER.md, COMPLIANCE-MATRIX.md | `vendor-contract-recency` | NO |
| `doc.legal_ip_ledger` | `LEGAL-IP-LEDGER.md` | `gtm-partnerships` + Founder | new patent / trademark / contract template | quarterly | PRD.md (anti-scope), GTM-PLAN.md | `legal-ip-recency` | NO |
| `doc.internationalization` | `INTERNATIONALIZATION.md` | `council-architecture` + `gtm-marketing` | new locale, regulator-by-region update | quarterly | COMPLIANCE-MATRIX.md, GTM-PLAN.md | `i18n-locale-coverage` | YES (locale data only) |

### 2.4 Cross-cutting

| id | path | owner_team | update_trigger | update_cadence | dependent_docs | validation_check | agent_authoring_allowed |
|---|---|---|---|---|---|---|---|
| `doc.changelog` | `CHANGELOG.md` | (system-emitted) | every consolidated-doc commit | per commit | (none) | `changelog-completeness` | YES — automated emission |
| `doc.glossary` | `GLOSSARY.md` | `council-architecture` | new domain term, taxonomy resolution per ADR-0017 | monthly | PRD.md, DESIGN.md, SPEC.md, all per-product PRDs | `glossary-cross-doc-coverage`, `glossary-vocabulary` | YES — agent extracts new terms; humans rename |
| `doc.doc_catalog` | `DOC-CATALOG.md` (this doc) | `council-architecture` | canonical doc added/removed | per change + monthly | (all canonical docs) | `doc-catalog-self-coverage` | NO |
| `doc.doc_update_protocol` | `DOC-UPDATE-PROTOCOL.md` | `council-architecture` | protocol change | quarterly | (all canonical docs) | `doc-catalog-self-coverage` | NO |
| `doc.documentation` | `DOCUMENTATION.md` | `council-architecture` | documentation-system contract change | quarterly | DOC-CATALOG.md, README.md | `doc-catalog-self-coverage`, `documentation-system` | NO |
| `doc.doc_coverage` | `DOC-COVERAGE.md` | `axis-foundry` + `council-architecture` | documentation-set coverage snapshot regeneration | per change | DOCUMENTATION.md, DOC-CATALOG.md | `documentation-system`, `doc-catalog-self-coverage` | YES — auto-emitted by documentation coverage tooling |
| `doc.agent_instruction_sources` | `AGENT-INSTRUCTION-SOURCES.md` | `axis-foundry` + `council-architecture` | agent-instruction source inventory changes | per change | AGENTS.md, DOC-CATALOG.md | `banned-primitives`, `doc-catalog-self-coverage` | YES — agent may re-emit inventory after source audit |
| `doc.standards_and_templates` | `STANDARDS-AND-TEMPLATES.md` | `axis-foundry` + `council-architecture` | standard or template change | quarterly | DOC-CATALOG.md, TOOLCHAIN.md | `doc-catalog-self-coverage` | NO |
| `doc.toolchain` | `TOOLCHAIN.md` | `axis-foundry` | toolchain or CI contract change | quarterly | RELEASE-MANAGEMENT.md, STANDARDS-AND-TEMPLATES.md | `release-lane-coverage` | NO |
| `doc.bootstrap` | `bootstrap.md` | `axis-foundry` | contributor bootstrap or hook install contract change | per change | TOOLCHAIN.md, AGENT-INSTRUCTION-SOURCES.md | `doc-catalog-self-coverage`, `readme-doc-coverage` | YES |
| `doc.mistakes_ledger` | `MISTAKES-LEDGER.md` | `council-architecture` | mistake discovered, remediated, or escalated | monthly | CONTRADICTION-LEDGER.md, RUNBOOKS-INDEX.md | `runbook-orphan-check` | NO |
| `doc.readme` | `README.md` | `council-architecture` | new file added in `docs/` | per change | (all canonical docs) | `readme-doc-coverage` | YES |

### 2.5 Per-product PRDs (Layer 2)

Each `products/<product-id>/PRD.md` follows the same pattern with the per-product team owning it.

| product-id | owner_team | update_trigger | update_cadence | depends_on |
|---|---|---|---|---|
| `saas-platform` | `axis-saas` | scope, contract, capability | monthly | `doc.prd`, `doc.design`, `doc.spec` |
| `foundry` | `axis-foundry` | capability, autonomy-ceiling, model, provider adapter, gate / scorecard / fitness-fn (Foundry consolidates agent runtime + engineering platform per ADR-0025) | bi-weekly | `doc.design`, `doc.privacy_program`, `doc.release_management`, ADR-0020/0021/0022/0024/0025/0050 |
| `workspace` | `axis-workspace` | mail / docs / sheets / slides / drive / calendar / meet / chat / forms / sites / tasks / notes / translate / recordings | bi-weekly | `doc.design`, `doc.privacy_program`, ADR-0017 |
| `cloud` | `axis-cloud` | resource type, region, KCMVP/CSAP gate | monthly | `doc.design`, `doc.compliance_matrix` |
| `search` | `axis-search` | index lifecycle, ranker | monthly | `doc.privacy_program`, ADR-0047 |
| `ads-analytics` | `axis-ads-analytics` | data-class taxonomy, KR adtech change | monthly | `doc.privacy_program` (the ADR is gating) |
| `vertical-corporate` | `vertical-corporate` | KR statutory change, ADR-0050 wave plan | weekly during [wave name per PRD §3.1] push, monthly otherwise | `doc.compliance_matrix` (KR), ADR-0033/0126/0127 |
| `vertical-healthcare` | `vertical-healthcare` | MFDS / 의료법 / clinical-AI ADR | monthly | `doc.compliance_matrix` (MFDS), ADR-0016/0137 |
| `vertical-industrial` | `vertical-industrial` | ISA-95, OPC UA, OT safety ADR | monthly | ADR-0033 |
| `vertical-logistics` | `vertical-logistics` | EDI standard, customs change | monthly | `doc.compliance_matrix` (logistics) |
| `vertical-fintech` | `vertical-fintech` | FSC / KYC standard / NACHA / RTP | monthly | `doc.compliance_matrix` (FSC + PCI), ADR-0027 |
| `vertical-legal` | `vertical-legal` | corpus update, contract template | quarterly | ADR-0033 |
| (others) | per-team | scope/regulatory | quarterly | per-team |

### 2.5b Per-product technical documentation (Supervisor Lane)

Foundry supervisor lane documentation: 26 files (5 crates × 5 docs each + 1 overview).

| id | path | owner_team | update_trigger | update_cadence | dependent_docs | validation_check | agent_authoring_allowed |
|---|---|---|---|---|---|---|---|
| `doc.supervisor_overview` | `products/foundry/supervisor/README.md` | `axis-foundry` | supervisor architecture change, new crate, new adapter | monthly | all supervisor-* docs | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_kernel_readme` | `products/foundry/supervisor/supervisor-kernel/README.md` | `axis-foundry` | kernel API change, port trait signature change | monthly | ARCHITECTURE.md, OPERATIONS.md, SECURITY.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_kernel_architecture` | `products/foundry/supervisor/supervisor-kernel/ARCHITECTURE.md` | `axis-foundry` | 12-layer placement change, adapter composition change | monthly | README.md, kernel source | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_kernel_operations` | `products/foundry/supervisor/supervisor-kernel/OPERATIONS.md` | `axis-foundry` | port trait API change, debugging guidance update | quarterly | README.md, ARCHITECTURE.md | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_kernel_security` | `products/foundry/supervisor/supervisor-kernel/SECURITY.md` | `axis-foundry` | secret handling policy change, Cedar enforcement update | quarterly | README.md, security-program/security-program.json | `doc-catalog-self-coverage` | NO |
| `doc.supervisor_kernel_benchmarks` | `products/foundry/supervisor/supervisor-kernel/BENCHMARKS.md` | `axis-foundry` | perf budget change, benchmark harness update | monthly | README.md, performance regression | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_app_readme` | `products/foundry/supervisor/supervisor-app/README.md` | `axis-foundry` | daemon API change, config schema change | monthly | ARCHITECTURE.md, OPERATIONS.md, SECURITY.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_app_architecture` | `products/foundry/supervisor/supervisor-app/ARCHITECTURE.md` | `axis-foundry` | call chain change, signal handling change, composition change | monthly | README.md, app source | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_app_operations` | `products/foundry/supervisor/supervisor-app/OPERATIONS.md` | `axis-foundry` | daemon startup procedure change, watchdog tuning guidance update | monthly | README.md, ARCHITECTURE.md | `doc-catalog-self-coverage` | YES |
| `doc.supervisor_app_security` | `products/foundry/supervisor/supervisor-app/SECURITY.md` | `axis-foundry` | signal safety change, audit conformance change | quarterly | README.md, security-program/security-program.json, ADR-0003 | `doc-catalog-self-coverage` | NO |
| `doc.supervisor_app_benchmarks` | `products/foundry/supervisor/supervisor-app/BENCHMARKS.md` | `axis-foundry` | perf budget change, heartbeat harness update | monthly | README.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.jsonl_adapter_readme` | `products/foundry/supervisor/jsonl-supervisor-adapter/README.md` | `axis-foundry` | adapter API change, file layout change | monthly | ARCHITECTURE.md, OPERATIONS.md, SECURITY.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.jsonl_adapter_architecture` | `products/foundry/supervisor/jsonl-supervisor-adapter/ARCHITECTURE.md` | `axis-foundry` | atomicity model change, fsync placement change | monthly | README.md, adapter source | `doc-catalog-self-coverage` | YES |
| `doc.jsonl_adapter_operations` | `products/foundry/supervisor/jsonl-supervisor-adapter/OPERATIONS.md` | `axis-foundry` | cleanup procedure change, recovery workflow change | quarterly | README.md, ARCHITECTURE.md | `doc-catalog-self-coverage` | YES |
| `doc.jsonl_adapter_security` | `products/foundry/supervisor/jsonl-supervisor-adapter/SECURITY.md` | `axis-foundry` | file permissions policy change, race condition mitigation | quarterly | README.md, security-program/security-program.json | `doc-catalog-self-coverage` | NO |
| `doc.jsonl_adapter_benchmarks` | `products/foundry/supervisor/jsonl-supervisor-adapter/BENCHMARKS.md` | `axis-foundry` | I/O latency budget change, fsync cost update | monthly | README.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_kernel_readme` | `products/foundry/supervisor/settings-template-kernel/README.md` | `axis-foundry` | SettingsTemplate API change, RendererMode change | monthly | ARCHITECTURE.md, OPERATIONS.md, SECURITY.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_kernel_architecture` | `products/foundry/supervisor/settings-template-kernel/ARCHITECTURE.md` | `axis-foundry` | 12-layer placement change, adapter composition change | monthly | README.md, kernel source | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_kernel_operations` | `products/foundry/supervisor/settings-template-kernel/OPERATIONS.md` | `axis-foundry` | template validation change, drift detection workflow change | quarterly | README.md, ARCHITECTURE.md | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_kernel_security` | `products/foundry/supervisor/settings-template-kernel/SECURITY.md` | `axis-foundry` | sref:// secret handling policy change, data class change | quarterly | README.md, security-program/security-program.json, ADR-0008 | `doc-catalog-self-coverage` | NO |
| `doc.settings_template_kernel_benchmarks` | `products/foundry/supervisor/settings-template-kernel/BENCHMARKS.md` | `axis-foundry` | template serialization budget change, memoization update | monthly | README.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_adapter_readme` | `products/foundry/supervisor/settings-template-adapter/README.md` | `axis-foundry` | renderer API change, atomic write pattern change | monthly | ARCHITECTURE.md, OPERATIONS.md, SECURITY.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_adapter_architecture` | `products/foundry/supervisor/settings-template-adapter/ARCHITECTURE.md` | `axis-foundry` | per-provider renderer change, format dialect change, HookEvent mapping | monthly | README.md, adapter source | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_adapter_operations` | `products/foundry/supervisor/settings-template-adapter/OPERATIONS.md` | `axis-foundry` | render/verify workflow change, reconciliation procedure change | monthly | README.md, ARCHITECTURE.md | `doc-catalog-self-coverage` | YES |
| `doc.settings_template_adapter_security` | `products/foundry/supervisor/settings-template-adapter/SECURITY.md` | `axis-foundry` | symlink defense change, file permissions change | quarterly | README.md, security-program/security-program.json | `doc-catalog-self-coverage` | NO |
| `doc.settings_template_adapter_benchmarks` | `products/foundry/supervisor/settings-template-adapter/BENCHMARKS.md` | `axis-foundry` | render latency budget change, verify latency budget change | monthly | README.md, BENCHMARKS.md | `doc-catalog-self-coverage` | YES |

### 2.6 Per-team charters (Layer 4)

Every team charter is owned by the team itself; meta-supervised by `council-architecture`.

| Trigger | What updates |
|---|---|
| New team formed | `teams/<id>/CHARTER.md` authored, README updated, RACI updated, FINOPS updated |
| Team scope shift | charter, RACI, dependent contracts re-checked |
| Team disbanded | charter archived, work re-assigned, RACI updated |

---

## 3. Historical update protocol (nonbinding migration input)

The following checklist records the pre-PHASE-5 design. It does not override the root hub, the
Markdown-retirement policy, masterplan v2, the protected-PR admission contract, or live reviewer and
`oya-ci-required` evidence.

### 3.1 Pre-flight checklist

1. ☐ Identify the trigger (which `EVT-*` from §1).
2. ☐ Read the doc you intend to change AND every doc in its `dependent_docs` column AND every upstream doc that points to it.
3. ☐ Read the team charter of `owner_team`. If you are not on that team, request a co-author from that team.
4. ☐ If the trigger is regulatory (`EVT-REGULATORY-CHANGE`), read the relevant section of `COMPLIANCE-MATRIX.md` for the regulator + the relevant ADR.
5. ☐ Open the issue tracker (`gh issue view`) for any referenced GitHub issue.
6. ☐ Confirm `agent_authoring_allowed` for the doc — if NO and you are an agent, hand off to a human reviewer.

### 3.2 Authoring

7. ☐ Author the change.
8. ☐ Add a row to `CHANGELOG.md` with `<doc.id> <iso-date> <author> <one-line-summary>`.
9. ☐ Update the doc's "Sources scanned" footer with current timestamps.
10. ☐ If the change adds/removes/renames a doc, update `README.md` AND `DOC-CATALOG.md` (this file) AND `machine-readable/catalog.json`.
11. ☐ If the change touches an axis contract (DESIGN §10), update `machine-readable/contracts.json`.
12. ☐ If the change is a new ADR, run the ADR-INDEX regeneration validator.
13. ☐ If the change touches a Foundry batch shape, regenerate `machine-readable/batches.json`.

### 3.3 Validation

14. ☐ Run the `validation_check` listed in §2 row.
15. ☐ Run the dependent-docs cross-link check (`oya-governance-doc-catalog`).
16. ☐ Run `oya-governance-glossary` to sync any new terms into `GLOSSARY.md`.
17. ☐ For agent authoring: emit an evidence record to the audit chain (per ADR-0003) tagged with the doc id, trigger event, and validator hash.

### 3.4 Review

18. ☐ Open PR with `## Verification` section listing every check from §3.3 and its outcome.
19. ☐ One author-distinct reviewer agent reviews and approves the exact PR head.
20. ☐ For Tier 1 docs, that reviewer applies the council-architecture lens; no human approval or reviewer quorum is required.
21. ☐ Merge through the protected PR only after review threads resolve, `oya-ci-required` is green, no conflict exists, and branch protection is satisfied.
22. ☐ Post-merge: emit `EVT-DOC-UPDATED` audit-chain record.

### 3.5 Publish

23. ☐ If the doc is regulator-relevant, the trust portal mirror is regenerated (see `RUNBOOKS-INDEX.md` "trust portal publish").
24. ☐ If the change is a contract change, the cross-axis announcement goes to all consumer teams' charter inboxes.
25. ☐ If the doc is `doc.glossary` and a term changed, run the `glossary-rename-cascade` agent.

---

## 4. Historical validation-check inventory

These names are historical design inventory. They are not proof of a wired binary CI gate and do
not block or authorize a merge unless the current `oya-ci-required` producer and change-class gate
mapping include them.

| Check | Does what |
|---|---|
| `prd-internal-consistency` | PRD §1-§9 cross-references resolve; success metrics and constraints are reflexive (no constraint violates a metric). |
| `prd-axis-coverage` | All 7 axes appear in PRD §3 (in-scope) or §3.2 (out-of-scope); no axis is absent. |
| `prd-glossary-alignment` | Every new domain term in PRD has a row in GLOSSARY.md. |
| `design-contracts-mirror` | Every row in DESIGN §10 has a row in `machine-readable/contracts.json`. |
| `design-vs-adr-cite-coverage` | DESIGN sections that reference an ADR include the ADR's current status. |
| `spec-contract-mirror` | Every source contract in `contracts/openapi/**/*.yaml` is in SPEC.md, `machine-readable/contracts.json`, and the OpenAPI runtime/schema binding registries, with typed explicit response-status parity, request/response schema shape and scalar type parity, and vice versa. |
| `spec-capability-coverage` | Every capability in `registry/catalog/capabilities/` has a section in SPEC.md. |
| `roadmap-band-totals` | Sum of leaves per band matches `machine-readable/batches.json`. |
| `roadmap-foundry-batch-shape` | Every batch declares fanout=N + SHARED-WRITES. |
| `adr-index-completeness` | Every file in `decisions/` has a row in ADR-INDEX.md (or is explicitly excluded). |
| `adr-supersession-graph` | Every Superseded ADR has a `superseded_by`; every superseder has a `supersedes` back-link. |
| `risk-register-coverage` | DESIGN §11 contradiction risks all appear in RISK-REGISTER.md. |
| `compliance-matrix-coverage` | Every regulator referenced in PRD / DESIGN / per-product PRDs has a row. |
| `compliance-evidence-recency` | No control evidence older than its declared cadence. |
| `security-controls-coverage` | Every CIS/ISO 27001/SOC2 control class has a row. |
| `privacy-class-taxonomy-coverage` | Every data class in §2.2.1 of PRIVACY-PROGRAM is referenced by every cross-axis flow in DESIGN. |
| `privacy-consent-flow-completeness` | Every consent tier (PRIVACY §2.2.2) has a UI surface, a backend gate, and an audit-emission point. |
| `gtm-pricing-coverage` | Every product PRD has a pricing model reference in GTM-PLAN. |
| `runbook-discoverability` | Every runbook in `docs/runbooks/` is in RUNBOOKS-INDEX. |
| `runbook-orphan-check` | No runbook references a deleted SLO or capability. |
| `slo-surface-coverage` | Every public surface from SPEC.md has an SLO entry. |
| `release-lane-coverage` | Every CI lane in ADR-0042 / `docs/standards/ci-lanes.md` is in RELEASE-MANAGEMENT.md. |
| `qa-coverage-by-class` | Test pyramid covers every kernel/domain/app/adapter role per ADR-0015. |
| `raci-team-coverage` | Every team in `teams/*/CHARTER.md` has a row in RACI-OWNERSHIP. |
| `codeowners-mirror` | RACI-OWNERSHIP per-surface owner matches `CODEOWNERS`. |
| `incident-template-completeness` | Severity taxonomy + comms templates + postmortem template present. |
| `hiring-axis-coverage` | Every team has a per-axis headcount row. |
| `finops-margin-coverage` | Every product PRD has a per-tenant unit-economic estimate. |
| `vendor-contract-recency` | No vendor contract within 90 days of expiry without a renewal task. |
| `legal-ip-recency` | Trademark / patent dates updated; OSS license inventory current. |
| `i18n-locale-coverage` | Every supported locale has a regulator + currency row. |
| `changelog-completeness` | Every consolidated-doc commit has a CHANGELOG row. |
| `glossary-cross-doc-coverage` | Every term in GLOSSARY appears in ≥1 consolidated doc. |
| `glossary-vocabulary` | Retired vocabulary hard-fails outside forensic docs; casing/acronym drift ratchets against `registry/glossary-vocabulary/warning-baseline.tsv` per ADR-0018. |
| `placeholder-debt` | `TODO` / `TBD` markers are tracked in `registry/placeholder-debt/registry.tsv`; new, stale, or count-drifted placeholders fail CI instead of hiding in glossary warnings. |
| `quality-lanes` | `registry/quality/lanes.yaml`, `docs/standards/ci-lanes.md`, owner-team charters, runtime budgets, and active `oya gate run-all` commands stay mirrored. |
| `cargo-prefix` | Every Cargo workspace member path and package name keeps the ADR-0017 `oya-` prefix, and the member path matches the package name. |
| `adr-citation` | Active docs cite only existing new-pack ADRs; legacy ADR numbers are confined to the explicit forensic consolidation surfaces. |
| `brand-residue` | Product-brand usage stays canonical while sed-style tautological rebrand / rename residues fail CI. |
| `api-semver` | Public contract artifacts under `contracts/` must carry ADR-0037 tier, owner, semver, sunset, and ADR metadata before becoming tenant-facing commitments. |
| `supply-chain` | Catalog supply-chain claims stay source-only unless ADR-0039 scan, signing, and SBOM evidence is wired; RustSec and deny checks remain in the per-PR script. |
| `release-supply-chain` | Every digest-pinned release artifact has Trivy 4-layer, dual-SBOM, Cosign/Rekor, provenance, audit-event, and zero HIGH/CRITICAL evidence before release. |
| `runbook-freshness` | Every runbook carries a parsable `Last verified` date and stays within the RUNBOOKS-INDEX freshness SLA by severity; unscoped deferred stubs use the Sev-4 / 365-day freshness ceiling. |
| `audit-chain-replay` | Checked-in audit shard fixtures replay through the ADR-0003 hash-chain verifier; malformed, empty, or tampered shards fail the chain-replay drill. |
| `foundry-eval` | Published capability records under `registry/capability-templates/` must point at signed eval-set and latest-run artifacts that pass ADR-0024 adversarial, linguistic, threshold, and publish-readiness checks. |
| `cross-tenant-access-fuzz` | Deterministic tenant/cell isolation probes prove cross-tenant MCP discovery, tool invocation, capability grants, and cell rebinding fail closed while same-tenant control access still succeeds. |
| `doc-catalog-self-coverage` | Every canonical doc has a row in this catalog (this is what saves us from drift). |
| `documentation-system` | `docs/DOCUMENTATION.md`, `registry/docs/pipeline.tsv`, and `docs/wiki/quickref/README.md` stay mutually grounded. |
| `readme-doc-coverage` | Every cataloged root doc in `docs/` has a link in README. |

No live `oya-governance-doc-catalog` producer or all-checks fan-out is claimed. PHASE-5 remains
blocked until the machine catalog, producer, consumers, and cross-artifact enforcement land together.

---

## 5. Roles and escalation

| Role | Owner team | Authority |
|---|---|---|
| Doc Catalog Curator | `council-architecture` | Adds/removes catalog rows; enforces protocol. |
| Doc Author (per doc) | `owner_team` (per row) | Drafts updates within the doc's update trigger. |
| Doc Reviewer (Tier 1) | `council-architecture` second member | Approves Tier 1 updates. |
| Doc Reviewer (Tier 2/3) | `owner_team` peer | Approves Tier 2/3 updates. |
| Glossary Editor | `council-architecture` | Final say on term naming. |
| ADR Index Curator | `crew-adr-promotion` | Owns ADR-INDEX freshness + supersession graph. |

Escalation: a stuck doc update goes to `council-architecture`. A blocked council goes to the Founder.

---

## 6. Anti-patterns

1. **Editing a Tier 1 doc without reading its dependents.** Almost guaranteed to introduce drift.
2. **Letting an agent author a Tier 1 doc end-to-end.** Agents may *propose* updates; humans approve.
3. **Skipping the CHANGELOG entry.** Erases audit history.
4. **Renaming a glossary term without running the cascade.** Silent drift across docs.
5. **Multiple-doc batch PR.** Each doc update is one PR. Bundling > 2 docs at once is anti-pattern unless the change is a coordinated rename (rare).

---

## 7. Sources scanned

- `README.md` (this directory)
- All consolidated docs §0 ("Status" lines)
- ADR-0015 (repo structure), ADR-0037 (deprecation governance), ADR-0050 (governance umbrella)
- `CLAUDE.md` (project memory)
- `.github/CODEOWNERS`
- `registry/quality/claude-integration.json`

*Footer regenerated whenever this doc is edited.*
