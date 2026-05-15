---
doc_class: Index
shape: ~
length_cap: 120
authority_tier: 0
excludes:
  - path: docs/AGENTS.md
    reason: Agent operating contract — the contract for any agent or human about to make a change.
  - path: docs/CONSTITUTION.md
    reason: Constitutional frame — what overrides everything else.
  - path: docs/DOC-CATALOG.md
    reason: Per-doc lifecycle protocol.
  - path: docs/products/
    reason: Per-product authority.
  - path: docs/teams/
    reason: Per-team authority.
  - path: docs/regional-packs/
    reason: Per-region authority.
authority_chain_declaration: |
  system / developer / user instructions
    > /specs/cross-cutting/root-hub-pointers.json
    > docs/AGENTS.md (until /specs/cross-cutting/agent-operating-contract.json PHASE-5 promotion)
    > machine-readable specs and registries under .omc/
    > docs/ authority files during markdown-retirement compatibility
    > repo-root Redirect-class files (non-authoritative; lane-thin)
    > working drafts (never authoritative)
---

# Oyatie — Canonical Engineering Documentation

This is the canonical engineering documentation tree for **Oyatie**, one cohesive ecosystem-as-a-service across seven axes (SaaS · Workspace · Vertical · Foundry · Cloud · Search · Ads + Analytics). Every doc in this directory is authoritative; everything outside is non-authoritative discovery, working drafts, or implementation artifacts.

## 30-second routing — start here

| If you are a... | Read first | Then |
|---|---|---|
| Founder / Council member | [`CONSTITUTION.md`](CONSTITUTION.md) | [`PRD.md`](PRD.md) <!-- forward-reference: wave-1 --> |
| Engineer / contributor | This file | [`AGENTS.md`](AGENTS.md) → the canonical doc map there |
| Coding agent (Claude / Codex / Gemini / OMC) | [`AGENTS.md`](AGENTS.md) | per-agent appendix in `## Per-agent appendices` |
| Auditor / regulator | [`COMPLIANCE-MATRIX.md`](COMPLIANCE-MATRIX.md) <!-- forward-reference: wave-1 --> | [`PRIVACY-PROGRAM.md`](PRIVACY-PROGRAM.md) <!-- forward-reference: wave-1 -->, [`SECURITY-PROGRAM.md`](SECURITY-PROGRAM.md) <!-- forward-reference: wave-1 --> |
| External contributor (OSS / ISV / partner) | [`PRD.md`](PRD.md) <!-- forward-reference: wave-1 --> | [`DESIGN.md`](DESIGN.md) <!-- forward-reference: wave-1 --> §"7 axes" |

## Tier-1 documents

- [`MASTERPLAN.md`](MASTERPLAN.md) — **canonical Master Plan anchor**. All milestone INDEXes / phase INDEXes / Implementation Plans under `docs/plans/milestones/M*/` derive their authority chain from this document and ultimately from `docs/CONSTITUTION.md`. Foundation ADRs: ADR-0052, ADR-0053, ADR-0054.
- [`CONSTITUTION.md`](CONSTITUTION.md) — mission, decision rights, prohibitions, amendments.
- [`AGENTS.md`](AGENTS.md) — single agent operating contract for every agent and every human.
- [`DESIGN.md`](DESIGN.md) <!-- forward-reference: wave-1 --> — architecture, planes, cross-axis contracts.
- [`PRD.md`](PRD.md) <!-- forward-reference: wave-1 --> — north star, axes, scope, success metrics.
- [`SPEC.md`](SPEC.md) <!-- forward-reference: wave-1 --> — surface enumeration.
- [`ROADMAP.md`](ROADMAP.md) <!-- forward-reference: wave-1 --> — wave sequence, gate criteria.
- [`DOC-CATALOG.md`](DOC-CATALOG.md) — per-doc lifecycle, update protocol, validators.
- [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 --> — failure modes + mechanical preventions.
- [`standards/doc-style.md`](standards/doc-style.md) <!-- forward-reference: wave-1 --> — house style, doc-class taxonomy.

## Layer redirects

- Per-product PRDs → [`products/`](products/) <!-- forward-reference: wave-1 -->
- Per-team charters → [`teams/`](teams/) <!-- forward-reference: wave-1 -->
- Per-region packs → [`regional-packs/`](regional-packs/) <!-- forward-reference: wave-1 -->
- Architectural decisions → [`ADR-INDEX.md`](ADR-INDEX.md) <!-- forward-reference: wave-1 -->
- Runbooks (incident, DR, on-call, per-service) → [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md) <!-- forward-reference: wave-1 -->
- Standards (cross-cutting authoring norms) → [`standards/`](standards/) <!-- forward-reference: wave-1 -->
- Templates (PR, ADR, capability, runbook, …) → [`templates/`](templates/) <!-- forward-reference: wave-1 -->
- Checklists (pre-push, wave-gate, DSR, …) → [`checklists/`](checklists/) <!-- forward-reference: wave-1 -->
- Machine-readable mirrors → [`machine-readable/`](machine-readable/) <!-- forward-reference: wave-1 -->
- Glossary → [`GLOSSARY.md`](GLOSSARY.md) <!-- forward-reference: wave-1 -->

## Root document index

This root-doc index is release-checked by `oya gate validate readme-doc-coverage`; every `docs/*.md` file MUST appear here and in `machine-readable/catalog.json`.

| id | doc | tier | owner_team |
|---|---|---|---|
| `doc.readme` | [`README.md`](README.md) | `cross-cutting` | `council-architecture` |
| `doc.masterplan` | [`MASTERPLAN.md`](MASTERPLAN.md) | `0` | `council-architecture` |
| `doc.constitution` | [`CONSTITUTION.md`](CONSTITUTION.md) | `1` | `council-architecture` |
| `doc.agents` | [`AGENTS.md`](AGENTS.md) | `1` | `axis-foundry, council-architecture` |
| `doc.agent_instruction_sources` | [`AGENT-INSTRUCTION-SOURCES.md`](AGENT-INSTRUCTION-SOURCES.md) | `cross-cutting` | `axis-foundry, council-architecture` |
| `doc.prd` | [`PRD.md`](PRD.md) | `1` | `council-architecture` |
| `doc.design` | [`DESIGN.md`](DESIGN.md) | `1` | `council-architecture` |
| `doc.spec` | [`SPEC.md`](SPEC.md) | `1` | `platform-api-sdk` |
| `doc.roadmap` | [`ROADMAP.md`](ROADMAP.md) | `1` | `tactical-first-vertical-pilot` |
| `doc.adr_index` | [`ADR-INDEX.md`](ADR-INDEX.md) | `1` | `crew-adr-promotion` |
| `doc.adr_consolidation_plan` | [`ADR-CONSOLIDATION-PLAN.md`](ADR-CONSOLIDATION-PLAN.md) | `1` | `crew-adr-promotion` |
| `doc.adr_legacy_regression_mapping` | [`ADR-LEGACY-REGRESSION-MAPPING.md`](ADR-LEGACY-REGRESSION-MAPPING.md) | `1` | `crew-adr-promotion` |
| `doc.risk_register` | [`RISK-REGISTER.md`](RISK-REGISTER.md) | `1` | `council-architecture` |
| `doc.contradiction_ledger` | [`CONTRADICTION-LEDGER.md`](CONTRADICTION-LEDGER.md) | `1` | `council-architecture` |
| `doc.compliance_matrix` | [`COMPLIANCE-MATRIX.md`](COMPLIANCE-MATRIX.md) | `1` | `ops-compliance` |
| `doc.security_program` | [`SECURITY-PROGRAM.md`](SECURITY-PROGRAM.md) | `1` | `ops-security` |
| `doc.privacy_program` | [`PRIVACY-PROGRAM.md`](PRIVACY-PROGRAM.md) | `1` | `council-privacy` |
| `doc.gtm_plan` | [`GTM-PLAN.md`](GTM-PLAN.md) | `1` | `gtm-sales-se` |
| `doc.competitive_gap_analysis` | [`COMPETITIVE-GAP-ANALYSIS.md`](COMPETITIVE-GAP-ANALYSIS.md) | `1` | `council-architecture` |
| `doc.runbooks_index` | [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md) | `2` | `ops-sre-reliability` |
| `doc.slo_catalog` | [`SLO-CATALOG.md`](SLO-CATALOG.md) | `2` | `ops-sre-reliability` |
| `doc.release_management` | [`RELEASE-MANAGEMENT.md`](RELEASE-MANAGEMENT.md) | `2` | `ops-sre-reliability, axis-foundry` |
| `doc.qa_test_strategy` | [`QA-TEST-STRATEGY.md`](QA-TEST-STRATEGY.md) | `2` | `axis-foundry` |
| `doc.raci_ownership` | [`RACI-OWNERSHIP.md`](RACI-OWNERSHIP.md) | `2` | `council-architecture` |
| `doc.incident_management` | [`INCIDENT-MANAGEMENT.md`](INCIDENT-MANAGEMENT.md) | `2` | `ops-sre-reliability` |
| `doc.hiring_capacity_plan` | [`HIRING-CAPACITY-PLAN.md`](HIRING-CAPACITY-PLAN.md) | `3` | `council-architecture` |
| `doc.finops_plan` | [`FINOPS-PLAN.md`](FINOPS-PLAN.md) | `3` | `ops-finops` |
| `doc.vendor_partner_ledger` | [`VENDOR-PARTNER-LEDGER.md`](VENDOR-PARTNER-LEDGER.md) | `3` | `gtm-partnerships, ops-security` |
| `doc.legal_ip_ledger` | [`LEGAL-IP-LEDGER.md`](LEGAL-IP-LEDGER.md) | `3` | `gtm-partnerships, founder` |
| `doc.internationalization` | [`INTERNATIONALIZATION.md`](INTERNATIONALIZATION.md) | `3` | `council-architecture, gtm-marketing` |
| `doc.changelog` | [`CHANGELOG.md`](CHANGELOG.md) | `cross-cutting` | `system-emitted` |
| `doc.glossary` | [`GLOSSARY.md`](GLOSSARY.md) | `cross-cutting` | `council-architecture` |
| `doc.doc_catalog` | [`DOC-CATALOG.md`](DOC-CATALOG.md) | `cross-cutting` | `council-architecture` |
| `doc.doc_update_protocol` | [`DOC-UPDATE-PROTOCOL.md`](DOC-UPDATE-PROTOCOL.md) | `cross-cutting` | `council-architecture` |
| `doc.documentation` | [`DOCUMENTATION.md`](DOCUMENTATION.md) | `cross-cutting` | `council-architecture` |
| `doc.doc_coverage` | [`DOC-COVERAGE.md`](DOC-COVERAGE.md) | `cross-cutting` | `axis-foundry, council-architecture` |
| `doc.standards_and_templates` | [`STANDARDS-AND-TEMPLATES.md`](STANDARDS-AND-TEMPLATES.md) | `cross-cutting` | `axis-foundry, council-architecture` |
| `doc.toolchain` | [`TOOLCHAIN.md`](TOOLCHAIN.md) | `cross-cutting` | `axis-foundry` |
| `doc.mistakes_ledger` | [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) | `cross-cutting` | `council-architecture` |

## Reading order for a new contributor (≤90 minutes)

1. This file (10 min).
2. [`CONSTITUTION.md`](CONSTITUTION.md) (10 min).
3. [`PRD.md`](PRD.md) <!-- forward-reference: wave-1 --> §1–§3 — north star + axes + scope (15 min).
4. [`DESIGN.md`](DESIGN.md) <!-- forward-reference: wave-1 --> §1–§4 — cohesion thesis + planes + axes-as-bounded-contexts (30 min).
5. [`AGENTS.md`](AGENTS.md) (15 min) — the operating contract you'll honor on every change.
6. [`ROADMAP.md`](ROADMAP.md) <!-- forward-reference: wave-1 --> active wave (10 min).

After step 6 you have the orientation needed to read any tier-2 or tier-3 doc on demand.

## Authority precedence

```
docs/CONSTITUTION.md
  > rest of docs/
  > catalog records (registry/catalog/, contracts/, machine-readable/)
  > repo-root Redirect-class files (non-authoritative; lane-thin)
  > working drafts (never authoritative)
```

This chain appears verbatim in [`CONSTITUTION.md`](CONSTITUTION.md), in [`AGENTS.md`](AGENTS.md), and in this file. The `oya-foundry-fitness-authority-cohesion` lane validates the three declarations are character-identical.

## Anti-overlap

This index does not cover:

- The agent operating contract — see [`AGENTS.md`](AGENTS.md).
- The constitutional frame — see [`CONSTITUTION.md`](CONSTITUTION.md).
- The per-doc lifecycle protocol — see [`DOC-CATALOG.md`](DOC-CATALOG.md).
- Per-product PRDs — see [`products/`](products/) <!-- forward-reference: wave-1 -->.
- Per-team charters — see [`teams/`](teams/) <!-- forward-reference: wave-1 -->.
- Per-region packs — see [`regional-packs/`](regional-packs/) <!-- forward-reference: wave-1 -->.

The full machine-readable list is in this file's front-matter `excludes:` block.

## Sources scanned

- 2026-05-10 — initial draft authored from agentic-workflow best practice + RFC-2119 + RFC-8174 + Diátaxis + openai/symphony benchmark.
