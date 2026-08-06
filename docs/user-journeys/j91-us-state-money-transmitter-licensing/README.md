---
doc_class: User-Journey-README
journey_id: j91-us-state-money-transmitter-licensing
slice: locale-pack-overlay-final
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Yejin Park
audience_type: B2C_SIDE_BUSINESS_OPERATOR
microservice_count: 45
pack_overlay_anchor: US-MSB + per-state MTLs
related_adrs:
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0263-observability-emission-contract
  - ADR-0131-per-microservice-flat-layout
  - ADR-0105-thirteen-layer-canonical-enum
---

# j91 - US state money transmitter licensing for Yejin

## At a glance

Yejin runs a vintage-clothing business whose marketplace payments cross five state thresholds. The journey exercises finops-portal + connect, activates US-MSB, US-CA-MTL, US-NY-MTL, US-TX-MTL, US-FL-MTL, US-WA-MTL, and proves the pack overlay without touching j76-j90 or any existing ADR/standard/PRD/ARCHITECTURE file.

## Artifact inventory

| Artifact | Purpose | Minimum line target |
|---|---|---:|
| story.md | Persona narrative and compliance pressure | 800 |
| ux-flow.md | Screen-by-screen UX and accessibility states | 400 |
| handshake.md | Cross-microservice sequence, Cedar permits, ADR-0263 classes | 600 |
| schemas/openapi-overlay-action.json | OpenAPI 3.2.0 ingress contract | n/a |
| schemas/asyncapi-overlay-events.yaml | AsyncAPI 3.1.0 event contract | n/a |
| schemas/journey-messages.proto | proto3 RPC/event message contract | n/a |
| schemas/pack-state-bnf.md | BNF v4.1 grammar and ADR-0105 13-layer mapping | n/a |
| microservices/<svc>/IP-journey-* | 45 flat per-microservice implementation plans | 400 each |
| integration-test-plan.md | End-to-end and adversarial test sets | 400 |
| README.md | Index, matrix, and operating contract | 300 |

## Regulatory anchors

1. 31 CFR 1010.100(ff) money transmitter definition
2. 31 CFR 1022.210 money services business anti-money-laundering program
3. 31 CFR 1022.320 suspicious activity reporting for money services businesses
4. California Financial Code section 2030 license requirement and section 2037 surety/securities obligation
5. New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding
6. Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security
7. Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security
8. Washington RCW 19.230.030 license required and 19.230.050 surety bond

## Pack and cell matrix

| Pack | Cell/certification | Journey effect |
|---|---|---|
| US-MSB | us-general | threshold detection |
| US-CA-MTL | us-financial-services | state license gap analysis |
| US-NY-MTL | state-mtl-evidence-ready | surety bond packet |
| US-TX-MTL | us-general | NMLS evidence upload |
| US-FL-MTL | us-financial-services | Cedar-gated payment throttling |
| US-WA-MTL | state-mtl-evidence-ready | regulator renewal calendar |

## Microservices touched

| Microservice | Journey responsibility |
|---|---|
| analytics | risk scoring, cohort metrics, and transparency-report aggregates |
| api-gateway | pack-aware ingress, route admission, and OpenAPI 3.2.0 response shaping |
| application | user-facing shell state, locale copy routing, and session affordances |
| audit-chain | ADR-0263 event class sealing, Merkle anchoring, and regulator evidence proofs |
| calendar | deadline scheduling, regulator meeting slots, and evidence review reminders |
| cell | cell certification pinning, sovereign placement, and migration guardrails |
| cloud-iac | regional infrastructure overlays, control manifests, and promotion evidence |
| cloud-k8s | namespace isolation, admission labels, and policy workload placement |
| cloud-secrets | OpenBao-backed key handles, per-pack signing keys, and TTL rotation |
| comms-email | transactional notices, regulator acknowledgements, and signed delivery receipts |
| community | public and counterparty-facing portal flows plus ecosystem communication surfaces |
| compliance | pack activation, regulator article mapping, and auditor portal evidence inventory |
| connector | cross-tenant connector handshakes, parent/subsidiary bridges, and partner attestations |
| consent-graph | purpose consent, withdrawal propagation, and data-subject rights state |
| developer-sdk | SDK contracts, examples, and generated client tests for the journey API |
| docs | tenant documentation portal, policy packet publishing, and regulator-readable knowledge base |
| drive | evidence bundle storage, export packaging, and controlled document retention |
| feature-flags | safe rollout gates, kill switches, and jurisdiction-scoped enablement |
| finops-portal | licensing cost, bond threshold, audit cost, and regulator fee operations |
| forms | intake forms, attestation questionnaires, and reviewed submission packets |
| foundry | agentic build plan execution, artifact provenance, and pack-rule verification runs |
| governance | signed pack registry, Cedar bundle publication, and control-plane approvals |
| identity | principal resolution, WebAuthn step-up, role binding, and cross-tenant subject identity |
| intelligence | classification assistance, policy summarization, and human-reviewed inference surfaces |
| mail | user mailbox notices, DSAR delivery packets, and external regulator correspondence |
| meet | auditor calls, review sessions, and evidence interview room records |
| messenger | tenant/user messaging, secure support channel, and escalation transcript handling |
| network | egress policy, cross-border route controls, and secure connectivity posture |
| notes | operator notes, legal rationale capture, and review memo retention |
| observability | metrics, traces, dashboards, logs, and audit-event telemetry correlation |
| ontology | typed entity graph, obligation relationships, and policy decision explainability |
| ops-dashboard-control-center | operator console, pack health view, and incident/review workbench |
| payments | fees, refunds, remittance/payment flow gating, and settlement evidence |
| plugin-app-store | pack-safe extension admission and third-party capability boundaries |
| recordings | meeting transcripts, consented recordings, and audit interview retention |
| sheets | control matrices, evidence spreadsheets, and reconciliation worksheets |
| shorts | consumer media surfaces affected by data-rights or minor-protection overlays |
| sites | tenant notices, regulator disclosure pages, and public transparency pages |
| slides | board/audit committee decks and regulator presentation packets |
| social | social notification, public transparency context, and abuse-signal backstops |
| tasks | operator task queues, approval checklists, and remediation follow-ups |
| tenancy | tenant scope, pack activation state, and audience-type boundaries |
| translate | locale-safe rendering, Arabic/Portuguese/Hindi/Singapore English support, and legal glossary |
| workflow-engine | durable orchestration, compensation, timers, and pack activation cascades |
| workflow-studio | no-code workflow authoring and visual policy preview for tenant admins |

## Load-bearing rules

- Conflict rule: Payment flows fail closed once any state threshold is reached without an active MTL or permitted agent-of-payee exemption.
- Cedar permits are signed pack fragments under ADR-0243 and published by governance before any mutating call.
- Every irreversible action emits ADR-0263 audit-event classes with subject, tenant, pack, cell, decision, and regulator-article dimensions.
- Contracts use OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, and BNF v4.1 as listed in schemas/.
- Flat per-microservice implementation plans live in microservices/<svc>/ per ADR-0131.
- The community surface is microservices/community/ and no retired service alias is used.

## Cross-references

- Documentation rigor: docs/standards/documentation-rigor.md.
- Pack doctrine: docs/decisions/ADR-0708-platform-foundations-live-apex.md.
- Exemplar pack roster: packs/cn-pipl/.
- Ecosystem catalog style anchor: docs/user-journeys/CATALOG-j126-j150-ecosystem.md.

## Acceptance summary table

| Check | Required result |
|---|---|
| README-AC-001 | analytics | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-002 | api-gateway | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-003 | application | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-004 | audit-chain | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-005 | calendar | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-006 | cell | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-007 | cloud-iac | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-008 | cloud-k8s | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-009 | cloud-secrets | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-010 | comms-email | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-011 | community | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-012 | compliance | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-013 | connector | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-014 | consent-graph | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-015 | developer-sdk | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-016 | docs | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-017 | drive | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-018 | feature-flags | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-019 | finops-portal | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-020 | forms | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-021 | foundry | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-022 | governance | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-023 | identity | Cedar-gated payment throttling | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-024 | intelligence | regulator renewal calendar | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-025 | mail | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-026 | meet | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-027 | messenger | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-028 | network | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-029 | notes | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-030 | observability | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-031 | ontology | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-032 | ops-dashboard-control-center | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-033 | payments | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-034 | plugin-app-store | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-035 | recordings | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-036 | sheets | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-037 | shorts | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-038 | sites | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-039 | slides | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-040 | social | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-041 | tasks | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-042 | tenancy | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-043 | translate | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-044 | workflow-engine | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-045 | workflow-studio | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-046 | analytics | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-047 | api-gateway | Cedar-gated payment throttling | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-048 | application | regulator renewal calendar | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-049 | audit-chain | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-050 | calendar | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-051 | cell | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-052 | cloud-iac | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-053 | cloud-k8s | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-054 | cloud-secrets | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-055 | comms-email | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-056 | community | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-057 | compliance | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-058 | connector | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-059 | consent-graph | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-060 | developer-sdk | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-061 | docs | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-062 | drive | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-063 | feature-flags | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-064 | finops-portal | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-065 | forms | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-066 | foundry | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-067 | governance | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-068 | identity | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-069 | intelligence | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-070 | mail | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-071 | meet | Cedar-gated payment throttling | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-072 | messenger | regulator renewal calendar | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-073 | network | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-074 | notes | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-075 | observability | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-076 | ontology | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-077 | ops-dashboard-control-center | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-078 | payments | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-079 | plugin-app-store | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-080 | recordings | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-081 | sheets | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-082 | shorts | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-083 | sites | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-084 | slides | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-085 | social | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-086 | tasks | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-087 | tenancy | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-088 | translate | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-089 | workflow-engine | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-090 | workflow-studio | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-091 | analytics | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-092 | api-gateway | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-093 | application | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-094 | audit-chain | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-095 | calendar | Cedar-gated payment throttling | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-096 | cell | regulator renewal calendar | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-097 | cloud-iac | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-098 | cloud-k8s | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-099 | cloud-secrets | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-100 | comms-email | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-101 | community | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-102 | compliance | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-103 | connector | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-104 | consent-graph | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-105 | developer-sdk | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-106 | docs | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-107 | drive | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-108 | feature-flags | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-109 | finops-portal | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-110 | forms | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-111 | foundry | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-112 | governance | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-113 | identity | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-114 | intelligence | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-115 | mail | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-116 | meet | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-117 | messenger | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-118 | network | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-119 | notes | Cedar-gated payment throttling | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-120 | observability | regulator renewal calendar | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-121 | ontology | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-122 | ops-dashboard-control-center | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-123 | payments | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-124 | plugin-app-store | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-125 | recordings | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-126 | sheets | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-127 | shorts | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-128 | sites | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-129 | slides | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-130 | social | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-131 | tasks | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-132 | tenancy | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-133 | translate | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-134 | workflow-engine | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-135 | workflow-studio | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-136 | analytics | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-137 | api-gateway | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-138 | application | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-139 | audit-chain | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-140 | calendar | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-141 | cell | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-142 | cloud-iac | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-143 | cloud-k8s | Cedar-gated payment throttling | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-144 | cloud-secrets | regulator renewal calendar | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-145 | comms-email | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-146 | community | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-147 | compliance | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-148 | connector | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-149 | consent-graph | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-150 | developer-sdk | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-151 | docs | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-152 | drive | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-153 | feature-flags | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-154 | finops-portal | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-155 | forms | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-156 | foundry | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-157 | governance | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-158 | identity | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-159 | intelligence | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-160 | mail | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-161 | meet | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-162 | messenger | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-163 | network | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-164 | notes | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-165 | observability | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-166 | ontology | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-167 | ops-dashboard-control-center | Cedar-gated payment throttling | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-168 | payments | regulator renewal calendar | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-169 | plugin-app-store | threshold detection | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-170 | recordings | state license gap analysis | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-171 | sheets | surety bond packet | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-172 | shorts | NMLS evidence upload | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-173 | sites | Cedar-gated payment throttling | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-174 | slides | regulator renewal calendar | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-175 | social | threshold detection | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-176 | tasks | state license gap analysis | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-177 | tenancy | surety bond packet | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-178 | translate | NMLS evidence upload | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-179 | workflow-engine | Cedar-gated payment throttling | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-180 | workflow-studio | regulator renewal calendar | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-181 | analytics | threshold detection | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-182 | api-gateway | state license gap analysis | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-183 | application | surety bond packet | Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-184 | audit-chain | NMLS evidence upload | Washington RCW 19.230.030 license required and 19.230.050 surety bond | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-185 | calendar | Cedar-gated payment throttling | 31 CFR 1010.100(ff) money transmitter definition | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-186 | cell | regulator renewal calendar | 31 CFR 1022.210 money services business anti-money-laundering program | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-187 | cloud-iac | threshold detection | 31 CFR 1022.320 suspicious activity reporting for money services businesses | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-188 | cloud-k8s | state license gap analysis | California Financial Code section 2030 license requirement and section 2037 surety/securities obligation | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-189 | cloud-secrets | surety bond packet | New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding | Cedar deny-wins; ADR-0263 event sealed |
| README-AC-190 | comms-email | NMLS evidence upload | Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security | Cedar deny-wins; ADR-0263 event sealed |
