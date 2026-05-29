# ITSM Competitor Parity Matrix

Service: itsm
Business capability: service-management
Date: 2026-05-21
Doc class: Competitor Parity Matrix
Top-3 counterparts: ServiceNow ITSM (Washington DC + Vancouver release; ITSM Pro + CMDB + Now Assist) / Atlassian Jira Service Management (Cloud Enterprise; Assets formerly Insight; Atlassian Intelligence) / Freshservice (Enterprise tier; Freddy AI; Orchestration Center)
Second-tier counterparts: BMC Helix ITSM; Ivanti Neurons for ITSM; SolarWinds Service Desk; Zendesk Support; PagerDuty; Opsgenie; FireHydrant
Binding authorities: docs/standards/documentation-rigor.md §§1.1, 1.2, 2, 3.2.1, 3.2.3, 3.2.5; ADR-0064; ADR-0105; ADR-0131; ADR-0132; ADR-0243; ADR-0244; ADR-0245; ADR-0246; ADR-0247; ADR-0251; ADR-0252; ADR-0253; ADR-0263; ADR-0314; ADR-0321; ADR-0328; ADR-0329; ADR-0330; ADR-0331.

## How to read this matrix

This matrix computes **union coverage** per ADR-0328 §D-5. For every major feature in at least one of the top-3 counterparts (ServiceNow ITSM, JSM, Freshservice), the matrix reports Oyatie ITSM's stance: COVERED, PARTIAL, MISSING, or OUT-OF-SCOPE-INTENTIONAL (with handoff target). MISSING entries enter the backlog. OUT-OF-SCOPE-INTENTIONAL entries are explicitly rejected with a routing rationale.

The matrix replaces the prior shape-only file (audit finding F-SB-03 / F-PA-01) which cycled identical evidence rows through 14 section headers.

## Union-coverage table

| Feature | ServiceNow ITSM | JSM | Freshservice | Oyatie ITSM | Verdict | Cite |
|---|---|---|---|---|---|---|
| Incident Management — full lifecycle | Yes (sys_incident table + Flow) | Yes (incident issue type) | Yes (tickets) | Yes — `capabilities/incident-open.yaml` + IP-026 + `src/domain` IncidentTicket aggregate; lifecycle endpoints in OpenAPI v1 + proto3 | COVERED | IP-026 |
| Problem Management — root-cause + known-error library | Yes (sys_problem + Now Assist RCA) | Yes (problem issue type + linked incidents) | Yes (problem module) | Yes — `capabilities/problem-link.yaml` + IP-026; known-error library projected via knowledge-base IP-034 | COVERED | IP-026 / IP-034 |
| Change Enablement — Standard / Normal / Emergency / CAB | Yes (CAB workbench + risk score) | Yes (change calendar + auto-approve) | Yes (CAB + change risk) | Yes — `capabilities/change-approve.yaml` + IP-029 change-freeze + risk calculator; CAB workflow templates via IP-044 | COVERED | IP-029 / IP-044 |
| Service Request Management — fulfillment workflows | Yes (sc_request + sc_task) | Yes (request type + automation) | Yes (service request) | Yes — `service-request` legacy bounded context + IP-028 entitlement orchestrator | COVERED | IP-028 |
| Service Catalog — catalog item lifecycle | Yes (sc_cat_item + entitlement) | Yes (request catalog) | Yes (catalog module) | Yes — `capabilities/service-catalog-publish.yaml` + IP-028 + marketplace listing publication via ADR-0314 DealSet | COVERED | IP-028 |
| Knowledge Management (KCS v6) | Yes (kb_knowledge + Knowledge Centric Service) | Yes (Confluence + JSM linking) | Yes (Solutions module) | Yes — `capabilities/knowledge-base.yaml` + IP-034; RAG via intelligence µservice | COVERED (Wave 15A) | IP-034 |
| Asset Management (HW / SW licenses / contracts) | Yes (SAM / HAM Pro) | Yes (Assets, formerly Insight) | Yes (Asset Management) | Partial — substrate via cmdb µservice + IP-027 reconciliation graph; HW/SW license tracking declared by cmdb µservice | PARTIAL — handoff to cmdb µservice | IP-027 |
| Configuration Management Database (CMDB) | Yes (cmdb_ci tables) | Yes (Assets schema) | Yes (Assets) | Yes — `capabilities/cmdb-sync.yaml` + IP-027; data store owned by cmdb µservice; ITSM owns operational integration | COVERED via substrate | IP-027 |
| Discovery (auto-CMDB population) | Yes (Discovery — separate product) | Yes (Assets discovery probes) | Yes (Probe + cloud connectors) | Yes — `capabilities/discovery.yaml` + IP-036; SNMP/WMI/SSH/K8s/AWS/OCI agent kinds | COVERED (Wave 15A) | IP-036 |
| Service Mapping (top-down dependency) | Yes (Service Mapping) | Yes (Insight Service Graph) | Partial (relationship views) | Yes — `capabilities/service-mapping.yaml` + IP-037; 3-hop p99 ≤ 380ms; 3.7× ServiceNow | COVERED (Wave 15A) | IP-037 |
| Service-Level Management (SLAs / OLAs / UCs) | Yes (SLA Engine) | Yes (SLA goals + breach trends) | Yes (SLA Policies) | Yes — `capabilities/sla-engine.yaml` + IP-030 + IP-041; event-driven detection 8× ServiceNow | COVERED (Wave 15A) | IP-030 / IP-041 |
| Major Incident Management (war-room) | Yes (Major Incident Management) | Partial (linked-incidents + Opsgenie integration) | Partial (escalations) | Yes — `capabilities/major-incident-bridge.yaml` + `incident-room` bounded-context crate; MLS RFC 9420 per ADR-0246 | COVERED | crates/incident-room |
| Workflow Automation (Flow Designer-equivalent) | Yes (Flow Designer + IntegrationHub) | Yes (Automation rules) | Yes (Orchestration Center) | Yes — `capabilities/workflow-designer.yaml` + IP-044; substrate via workflow-engine µservice; 7× ServiceNow throughput | COVERED (Wave 15A) | IP-044 |
| AI / Virtual Agent | Yes (Now Assist Virtual Agent) | Yes (Atlassian Intelligence) | Yes (Freddy AI) | Yes — `capabilities/ai-virtual-agent.yaml` + IP-035; intelligence µservice substrate; tenant-isolated | COVERED (Wave 15A) | IP-035 |
| Performance Analytics | Yes (Performance Analytics) | Yes (Reports for JSM) | Yes (Analytics Plus) | Yes — `capabilities/performance-analytics.yaml` + IP-043; KPI catalog owned by ITSM, data routed to analytics µservice | COVERED (Wave 15A) | IP-043 |
| Predictive Intelligence | Yes (Predictive Intelligence) | Yes (Atlassian Intelligence predictive) | Yes (Freddy Predict) | Yes — `capabilities/predictive-intelligence.yaml` + IP-038; per-tenant fine-tune via intelligence µservice | COVERED (Wave 15A) | IP-038 |
| ITSM Mobile (agent + requester) | Yes (Now Mobile + Now Mobile Agent) | Yes (JSM Mobile) | Yes (Freshservice Mobile) | Yes — `capabilities/mobile-itsm.yaml` + IP-032; native iOS (Swift) + Android (Kotlin) | COVERED (Wave 15A) | IP-032 |
| Customer / Self-Service Portal | Yes (Service Portal) | Yes (Customer Portal) | Yes (Self-Service Portal) | Yes — `capabilities/self-service-portal.yaml` + IP-031; deflection target 35% | COVERED (Wave 15A) | IP-031 |
| Survey + CSAT | Yes (Survey module) | Yes (CSAT module) | Yes (CSAT module) | Yes — `capabilities/csat-survey.yaml` + IP-039; multi-channel delivery | COVERED (Wave 15A) | IP-039 |
| ITIL Process Pack | Yes (ITIL alignment) | Yes (ITIL alignment) | Yes (ITIL alignment) | Yes — IP-026 ITIL process normalizer; canonical vendor mapping | COVERED | IP-026 |
| ITSM-to-DevOps integration (PR as Change) | Yes (DevOps Change Velocity) | Yes (DevOps integration native) | Yes (DevOps integration via marketplace) | Routed — `oya git` µservice cross-emits PRs as Standard Changes; auto-link via change-management µservice | OUT-OF-SCOPE-INTENTIONAL; routed | crates/incident-room + change-management µservice |
| Workspace / Agent Workspace | Yes (Agent Workspace + Configurable Workspace) | Yes (Queues) | Yes (Agent Console) | Yes — `capabilities/agent-workspace.yaml` + IP-033; multi-pane | COVERED (Wave 15A) | IP-033 |
| Role-Based Access Control (RBAC + ACLs) | Yes (Roles + ACLs) | Yes (Permissions schemes) | Yes (Roles) | Yes — substrate via identity µservice + Cedar policies per ADR-0243 | COVERED via substrate | policy/*.cedar |
| Multi-Tenant Architecture | No (per-tenant instance — ServiceNow's "defining feature") | Yes (cloud-native multi-tenant) | Yes (cloud-native multi-tenant) | Yes — cloud-cell µservice multi-tenant cell topology; not per-tenant DB | COVERED — Oyatie rejects multi-instance per-tenant DB as wasteful; documented out-of-scope | tenancy + cloud-cell |
| Multi-Instance (per-tenant DB) | Yes (defining feature) | No | No | OUT-OF-SCOPE-INTENTIONAL — ADR-0248 cellular sharding + ADR-0244 tenant scoping provide stronger isolation without per-tenant DB | OUT-OF-SCOPE-INTENTIONAL; ADR cite | ADR-0244 / ADR-0248 |
| Integration Hub (IntegrationHub / Spokes) | Yes (IntegrationHub + ITOM spokes) | Yes (Atlassian Marketplace apps) | Yes (Marketplace apps) | Yes — marketplace µservice + workflow-engine µservice; action packs published per ADR-0314 | COVERED via marketplace | ADR-0249 + ADR-0314 |
| Migration tools (from competitor) | Limited (DR / out-only) | Yes (Project Importer) | Yes (Importers) | Yes — `migration-playbooks/` per top-3 counterpart | COVERED | migration-playbooks/ |
| Walk-Up Experience (kiosk) | Yes (Walk-Up Experience module) | No | Yes (Walk-Up) | Yes — `capabilities/walk-up-experience.yaml` + IP-040 | COVERED (Wave 15A) | IP-040 |
| Visual Task Boards (Kanban) | Yes (Visual Task Boards) | Yes (Boards) | Yes (Kanban) | Yes — `capabilities/visual-task-boards.yaml` + IP-042 | COVERED (Wave 15A) | IP-042 |
| On-Call Schedule (PagerDuty-class) | Partial (via integration) | Yes (Opsgenie acquisition) | Partial (via integration) | Yes — `crates/on-call-schedule` bounded context (own primitive); PagerDuty / Opsgenie / FireHydrant displacement | COVERED (Wave 15A) | crates/on-call-schedule |
| Escalation Policy (PagerDuty-class) | Partial | Yes (Opsgenie) | Partial | Yes — `crates/escalation-policy` bounded context | COVERED (Wave 15A) | crates/escalation-policy |
| Statuspage (incident communications) | Partial (System Status) | Partial (Atlassian Statuspage cross-product) | Partial | Yes — `crates/status-update` bounded context | COVERED (Wave 15A) | crates/status-update |
| Blameless Postmortem | No (separate Now Assist add-on) | No | No | Yes — `crates/postmortem` bounded context; FireHydrant Retro + Jeli + PagerDuty Postmortem displacement | COVERED (Wave 15A; Oyatie OUTPERFORMS) | crates/postmortem |

## Bounded-context plurality (Wave 15A audit fix for F-IC-12 / PRD §C)

Per ADR-0131 flat layout the ITSM µservice composes five separately-buildable crates under `microservices/itsm/crates/`. Each crate has its own Cargo.toml, src/lib.rs, and unit-test set. The umbrella µservice re-exports each crate as a module (`oya_itsm::on_call_schedule`, etc.) and the umbrella `validate_scaffold()` asserts `BOUNDED_CONTEXTS.len() == 5`.

The five crates:

1. `on-call-schedule` — PagerDuty / Opsgenie / FireHydrant Schedules displacement.
2. `escalation-policy` — PagerDuty / Opsgenie / FireHydrant Escalation Policies displacement.
3. `incident-room` — MLS-encrypted (RFC 9420 per ADR-0246) war-rooms.
4. `status-update` — Statuspage-class incident communications.
5. `postmortem` — Blameless retros with change/problem-linked action items.

## Counterpart selection correction (audit fix for F-IC-02 / F-PA-02)

The prior audit found three artifacts naming three different top-3 sets. The Wave 15A remediation reconciles the canonical roster:

- **Top-3 (binding):** ServiceNow ITSM, Jira Service Management, Freshservice.
- **Second-tier (named separately, never substituted for top-3):** BMC Helix ITSM (NOT BMC Remedy — Remedy is the retired predecessor), Ivanti Neurons for ITSM, SolarWinds Service Desk, Zendesk Support, PagerDuty, Opsgenie, FireHydrant.
- **Dropped from manifest:** xMatters (legacy Everbridge predecessor; not part of the canonical roster).
- **Distinction (audit fix):** Freshservice is the ITSM product; Freshdesk is the customer-support product owned by Freshworks. The manifest now distinguishes the two.

## Performance leadership (preserved per audit §13.2)

| Metric | Oyatie ITSM | ServiceNow ITSM | Advantage |
|---|---|---|---|
| SLA breach detection p99 | 15 s | 120 s | 8× |
| Sustained workflow throughput | 800 workflows/sec | 120 workflows/sec | 7× |
| CMDB 3-hop traversal p99 | 380 ms | 1,400 ms | 3.7× |

Source: `microservices/itsm/performance-benchmark-numbers-2026-05-20.md` §3 + §8. The Wave 15A remediation PRESERVES these claims as canonical differentiators (audit §13.2). They were previously documented only in the (retirement-target) `capability-tiers/tier-matrix.md`; this matrix lifts them into the canonical parity surface.

## Tenant-class behavior (per ADR-0331)

Every feature above is delivered with uniform industry-leader quality across `demo_trial` and `paid` tenant classes. Demo/trial tenants experience usage caps + time-gating (60 days) + no compliance pack activation; paid tenants experience per-seat + per-usage + revenue-share billing components. There are no retired named capability levels capability tiers (retired per Wave 15J + audit T-RET-01..T-RET-06).

## Out-of-scope intentionally

| Item | Counterpart | Routing rationale |
|---|---|---|
| Multi-instance per-tenant DB | ServiceNow (defining feature) | ADR-0248 cellular shuffle-sharding + ADR-0244 tenant scoping deliver stronger isolation without the cost of per-tenant DB |
| Public marketing KB | (none of the top-3) | brand-surface µservices own marketing KB; ITSM owns operational KB only |
| External chat-ops bridges as native modules | ServiceNow Chat | Personal Messenger substrate (MLS RFC 9420 per ADR-0246) is the canonical chat layer; bridges live as marketplace listings |
| Per-tenant on-call mobile pagers (hardware) | PagerDuty hardware ack | Out of scope; mobile push via APNs/FCM is the canonical pager channel |

## Backlog deltas absorbed by Wave 15A

Audit findings F-PA-01..F-PA-16 (14 missing or under-declared surfaces) are absorbed by IP-031..IP-044 + the five bounded-context crates. See `REMEDIATION-NOTES-2026-05-21.md` for the per-finding closeout.
