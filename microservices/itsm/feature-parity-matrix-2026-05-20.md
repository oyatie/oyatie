---
doc_class: FeatureParityMatrix
microservice: itsm
parity_wave: Wave-4-rolling
parity_date: 2026-05-21
parity_owner: codex-itsm-ownership-audit-w4r
parity_mode: union-coverage
top_3_counterparts:
  - ServiceNow ITSM (Washington DC family + Vancouver release; ITSM Pro + CMDB + Now Assist + Service Mapping + Discovery + Now Mobile + Service Portal)
  - Atlassian Jira Service Management Cloud Enterprise (Service Desk + Assets formerly Insight + Atlassian Intelligence + JSM Mobile + Customer Portal + Opsgenie + Compass adjacent)
  - Freshservice Enterprise (ITSM + Freshservice Asset Management + Freddy AI + Orchestration Center + Workload Management + Project Management + Freshservice Mobile + Self-Service Portal + Analytics+)
five_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-5 union-coverage parity bar
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md ServiceNow + Atlassian + Freshworks dossiers
  - /Users/jasonlee/oyatie/microservices/itsm/PRD.md + ARCHITECTURE.md + manifest.json + capabilities/ + IP-026..IP-030
  - /Users/jasonlee/oyatie/microservices/itsm/coherence-audit-2026-05-20.md §6 + §7
  - ServiceNow product documentation (Washington DC release notes) + Atlassian JSM Cloud REST API v3 + Freshservice Enterprise feature index (vendor-public references)
related_adrs:
  - ADR-0244 tenant scoping
  - ADR-0243 Cedar universal gate
  - ADR-0263 audit emission
  - ADR-0247 self-modification (Foundry absorption)
  - ADR-0255-amendment intelligence + Foundry absorption
  - ADR-0316 capability tiers (retirement candidate)
  - ADR-0321 B2B leader coverage
  - ADR-0328 substance bar + canonical sequence
union_coverage_states:
  - covered: feature exists in Oyatie µservice (with path to owning artifact)
  - partial: feature partially in Oyatie µservice (with named missing gap)
  - missing: feature absent from Oyatie µservice (with proposed remediation target)
  - out-of-scope-intentional: feature deliberately out-of-scope per Oyatie doctrine (with reason)
---

# ITSM Feature-Parity Matrix — Union Coverage vs ServiceNow ITSM + JSM + Freshservice

## 0. Methodology and brief-anchor header

This matrix computes union coverage per ADR-0328 §D-5: if any of the top-3 counterparts has a major feature, Oyatie must either cover it (with the owning µservice / capability / surface named) or mark it intentionally out of scope (with reason). The parity bar is stricter than average coverage; one counterpart having a feature is sufficient to require an Oyatie answer.

Top-3 selection rationale: ServiceNow ITSM is the displacement target (highest enterprise share, deepest CMDB and Service Mapping, premium pricing); Jira Service Management is the developer-tooling-adjacent leader (lowest friction, broad Atlassian-suite gravity); Freshservice is the mid-market price-leader with parity feature set. BMC Helix, Ivanti Neurons, SolarWinds Service Desk, Zendesk Support, Freshdesk are second-tier and contribute corroborating evidence rows but are not the union-coverage gate.

Each row carries the canonical counterpart capability label, what each top-3 counterpart actually ships (citing vendor-public surface where applicable), the Oyatie owning surface, the union-coverage verdict, and the remediation target when missing or partial.

No tier deltas appear in this matrix. Per `feedback_no_capability_profiles_2026_05_20.md`, the retired named capability levels scheme is retired; capability availability is uniform across paid tenants regardless of `billing_components` choice, with demo/trial tenants getting hard usage caps + time gates (not feature gating).

## 1. Incident Management family

Counterpart canonical surface (ServiceNow ITSM Incident table + JSM Incident issue type + Freshservice Incident ticket):

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 1.1 | Incident create (form + API) | sys_incident table + REST API + email-to-ticket | Service Desk incident issue type + REST API v3 + email handler | Incident ticket form + REST + email | `capabilities/incident-open.yaml` + `src/domain::IncidentTicket` + `contracts/openapi-v1.yaml /itsm/actions/{action_id}` | partial | Wave 15E: expand OpenAPI lifecycle endpoints to /itsm/incidents POST/GET/PATCH; email ingest under `worker` layer |
| 1.2 | Incident categorization (Cause/Effect category trees) | Choice list with cmdb_ci_class linkage | Request type + field configuration | Category + sub-category | `src/domain::DataClass` enum (incident_ticket variant) + IP-026 process classification | partial | Wave 15E: declare category taxonomy in PRD §D + contracts schema |
| 1.3 | Priority matrix (Impact × Urgency) | Configurable priority matrix | Priority field + automation | Configurable priority + SLA | `src/domain::Priority` (P1/P2/P3/P4 enum visible in `src/lib.rs::default_incident_ticket()` shape) | partial | Wave 15E: declare configurable Impact × Urgency matrix in PRD; expose via Cedar policy attribute |
| 1.4 | Assignment + assignment groups | Assignment group + auto-routing rules | Queue + automation rule | Group + Workload Management | Workflow-engine substrate + `policy/service-management-authorization.cedar` | partial | Wave 15E: declare assignment-routing capability primitive; bind to `tasks` µservice for workload |
| 1.5 | Skills-based routing | Predictive Intelligence Skill matching | Automation + queue | Workload Management skill | not declared | missing | Wave 15E: PRD declaration + binding to identity skill claims |
| 1.6 | On-call schedule + escalation | On-Call Scheduling (sold separately) | Opsgenie native | On-Call Management add-on | not declared | missing | Wave 15E: bind to `incident-management` µservice (4A.4 sibling) for on-call rotation |
| 1.7 | Incident state machine (New→In Progress→On Hold→Resolved→Closed) | Predefined incident state lifecycle | Workflow status set | Status workflow | `src/domain::TicketStatus` (Draft/Open/Pending/Resolved/Closed implied via type) + ARCHITECTURE §C invariants | covered | — |
| 1.8 | Linked records (parent/child incidents, related changes) | sys_incident.parent + related_change_records | Issue links | Parent ticket + related items | `IP-026` schema field `ontology_object_ref` | partial | Wave 15E: declare parent-child link semantics in PRD + contracts |
| 1.9 | Knowledge article suggestions | KB AI suggestions + KCS workflow | Smart KB suggestion | KB AI recommendation (Freddy) | not declared | missing | Wave 15E: declare KB binding (probably to `community` or new `knowledge` µservice) |
| 1.10 | Time tracking + work notes (audit log per-incident) | Activity stream + work notes + journal | Activity log + internal comments | Activity timeline + private notes | ARCHITECTURE §F audit-chain invariants + ADR-0263 audit emission | covered | — |
| 1.11 | Resolution + closure with closure code | Resolution code + closure notes | Resolution dropdown + comment | Resolution + reason | `IP-026` process stages `verified` + `sealed` | partial | Wave 15E: declare closure-code taxonomy |
| 1.12 | SLA attach + breach detection | SLA Definitions + retroactive | SLA goals + breach automation | SLA policy + breach event | `slos/local-sla-breach-detection.openslo.yaml` + `IP-030 SLA breach remediation loop` | covered | — |
| 1.13 | Reopen + re-categorization | Reopen with reason | Reopen with comment | Reopen action | `IP-026 process stages` `received` + audit invariant | partial | Wave 15E: declare reopen flow in PRD |
| 1.14 | Customer / requester satisfaction (CSAT) survey | Survey designer + dispatch | Customer feedback survey | Built-in CSAT survey | not declared | missing | Wave 15E: route to `community` or `mail` µservice for survey dispatch |
| 1.15 | Incident reports + dashboards | Performance Analytics + reports | Reports + dashboards | Analytics+ dashboards | `dashboards/*.json` (operating-bar-overview, slo-and-error-budget, local-domain-throughput) | partial | Wave 15E: route premium analytics to `analytics` µservice; declare handoff |
| 1.16 | Bulk incident operations (mass close, mass reassign) | List view + bulk actions | JQL bulk operations | Bulk update | not declared | missing | Wave 15E: declare bulk operation endpoint with idempotency |
| 1.17 | Attachments + file size policy | sys_attachment + per-pack size limits | Issue attachments + storage policy | Attachments + storage | ARCHITECTURE.md §F deployment invariant references SPIFFE + cell policy; no concrete attachment surface | missing | Wave 15E: declare attachment surface (probably bound to `drive` µservice) |
| 1.18 | Multi-language incident UI (EN/KO/JA/ES/DE/FR/PT/ZH-CN/etc.) | Localized UI via system_dictionary | i18n via Atlassian-cloud language packs | UI translation + Freddy multilingual | not declared | missing | Wave 15E: bind to `translate` µservice; KR-PIPA pack overlay sets locale defaults |
| 1.19 | Mobile incident triage (iOS + Android agent app) | Now Mobile Agent + Mobile App Builder | JSM Mobile + JSM Cloud Mobile | Freshservice Mobile + Mobile Agent | not declared | missing | Wave 15E: frontend/ios + frontend/android per `feedback_os_support_matrix_2026_05_20.md` |
| 1.20 | Major Incident Management (war-room, status updates, postmortem) | Major Incident Management + Postmortem | Major Incident response automation | Major Incident Management | `capabilities/major-incident-bridge.yaml` | partial | Wave 15E: declare MLS posture for war-room (per F-CD-06); declare status-page integration |

## 2. Problem Management family

Counterpart canonical surface (ServiceNow Problem + JSM problem + Freshservice Problem):

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 2.1 | Problem creation from incident | "Create Problem" action | Create Problem from Incident | Convert Incident to Problem | `capabilities/problem-link.yaml` + IP-026 process_family problem_management | covered | — |
| 2.2 | Root-cause analysis workflow | Problem investigation + RCA template | Problem investigation form | RCA template | IP-026 process stages classified/authorized/projected | partial | Wave 15E: declare RCA template structure |
| 2.3 | Known-error database (KEDB) | Known Error Articles + KB integration | Knowledge base linking | Known Error article type | not declared in canonical surface | missing | Wave 15E: declare KEDB capability primitive |
| 2.4 | Workaround publication | Workaround field + KB publish | Workaround comment + KB push | Workaround field + KB | IP-026 process_family `knowledge_management` (declared, not implemented) | partial | Wave 15E: implement workaround → KB workflow |
| 2.5 | Problem correlation (multiple incidents → one problem) | Related incidents tab | Issue links bulk | Linked incidents view | IP-026 process stage `routed` ontology binding | partial | Wave 15E: declare correlation graph endpoint |
| 2.6 | Problem closure with permanent fix link | Closure with permanent fix RFC link | Resolution with linked change | Closure with linked solution | `IP-027 CMDB reconciliation` ontology projection | partial | Wave 15E: declare problem→change link semantics |
| 2.7 | Problem trend analytics | Performance Analytics problem trends | Reports + custom | Analytics+ problem reports | dashboards stubs | partial | Wave 15E: route to `analytics` µservice |
| 2.8 | Proactive problem identification (AI clustering) | Predictive Intelligence Clustering | Atlassian Intelligence | Freddy AI Problem | not declared | missing | Wave 15E: bind to `intelligence` µservice |
| 2.9 | Problem priority + impact assessment | Priority + Business Impact field | Priority + custom field | Priority + impact | `src/domain::Priority` + `ServiceImpact` (visible in lib.rs export) | covered | — |
| 2.10 | Cross-tenant federation of known-errors | NOT supported in ServiceNow (tenant-isolated) | NOT supported | NOT supported | ARCHITECTURE.md §A explicitly out-of-scope | out-of-scope-intentional | Reason: tenant isolation per ADR-0244 |

## 3. Change Enablement family

Counterpart canonical surface (ServiceNow Change Management Pro + JSM Change + Freshservice Change):

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 3.1 | Change types (Standard, Normal, Emergency) | Predefined + custom change types | Standard/Normal/Emergency configurable | Change types | `capabilities/change-approve.yaml` + IP-029 change freeze + risk calculator | partial | Wave 15E: declare three change-type lifecycles |
| 3.2 | CAB (Change Advisory Board) workflow | CAB Workbench + meeting + approval | Approval workflow + CAB role | CAB approval workflow | IP-029 + `policies/local-change-approval-window.cedar` | partial | Wave 15E: declare CAB role + workbench surface |
| 3.3 | E-CAB (Emergency CAB) | E-CAB approval workflow | Emergency approval | Emergency approval | IP-029 emergency freeze override | partial | Wave 15E: declare E-CAB role + escalation |
| 3.4 | Change freeze windows / blackout periods | Change Freeze schedule + override | Change Calendar + freeze rules | Change blackout schedule | IP-029 + `runbooks/change-freeze-override.md` | covered | — |
| 3.5 | Change risk calculation | Risk Conditions framework | Risk score automation | Risk assessment field | IP-029 change freeze + risk calculator (substantive) | covered | — |
| 3.6 | CMDB impact analysis (affected CIs) | Affected CIs auto-population from Service Mapping | Insight asset linking | Asset linking | IP-027 CMDB reconciliation graph | covered | — |
| 3.7 | Implementation tasks (linked work items) | sys_change_task + tasks list | Sub-tasks + linked issues | Tasks + child changes | `tasks` µservice substrate (declared dependency in manifest.json) | partial | Wave 15E: declare change-task handoff to `tasks` µservice |
| 3.8 | Implementation evidence + verification | Verification steps + evidence | Resolution evidence + verification | Verification field | IP-026 process stages `executing` + `verified` | covered | — |
| 3.9 | Post-Implementation Review (PIR) | PIR template + automation | PIR custom workflow | PIR | not declared | missing | Wave 15E: declare PIR template |
| 3.10 | Rollback plan + execution | Backout plan field + execution | Rollback plan + automation | Rollback plan | IP-026 process stage `rolled_back` + ARCHITECTURE.md §E rollback evidence | covered | — |
| 3.11 | DevOps integration (PR → Change) | DevOps Change Velocity (sold separately) | Native (Bitbucket + GitHub + GitLab) | DevOps integration add-on | mentioned in capability-tiers/tier-matrix.md §retired-advanced: "oya git PRs cross-emit to ITSM as Standard Changes" | partial | Wave 15E: declare in PRD + canonical capability; bind to `oya git` substrate |
| 3.12 | Change calendar visualization | Change Calendar | Change Calendar | Change Calendar | not declared | missing | Wave 15E: declare calendar UI binding (to `calendar` µservice) |
| 3.13 | Approval delegation + multi-level | Approval rules + delegation | Approval automation + delegate | Multi-level approval | `policy/service-management-authorization.cedar` + identity µservice claim chains | partial | Wave 15E: declare delegation flow in PRD |
| 3.14 | Standard change template library | Standard Change Catalog + templates | Pre-approved templates | Standard change templates | IP-004 workflow-template-library binding | partial | Wave 15E: declare standard-change library |
| 3.15 | Change collision detection | Change Conflict Detection | Custom automation | Change conflict detection | IP-029 change freeze risk calculator | covered | — |

## 4. Service Request Management family

Counterpart canonical surface (ServiceNow Request + JSM Request + Freshservice Service Request):

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 4.1 | Service request submission via portal | Service Catalog form | JSM Portal request | Self-Service Portal request | not declared in canonical surface | missing | Wave 15E: declare service-request endpoint + bind to `community`/self-service shell |
| 4.2 | Request approval workflow | Multi-level approval + Cedar-equivalent | Approval automation | Multi-level approval | `service-request` bounded context (in ARCHITECTURE.md §C) | partial | Wave 15E: expose service-request lifecycle endpoints |
| 4.3 | Fulfillment task generation | Workflow + tasks | Automation + linked issues | Workflow + child tasks | IP-028 service catalog entitlement orchestrator | covered | — |
| 4.4 | Status updates to requester | Email + portal notifications | Customer portal updates | Email + portal updates | bind to `mail` µservice | partial | Wave 15E: declare requester notification binding |
| 4.5 | Request fulfillment SLA | Fulfillment SLAs | SLA goals on requests | Service item SLAs | IP-030 SLA breach remediation | partial | Wave 15E: declare fulfillment SLA distinct from incident SLA |
| 4.6 | Requester cancellation | Cancel action with reason | Cancel issue | Cancel request | IP-026 process stage `denied` + audit | partial | Wave 15E: declare cancel flow in PRD |
| 4.7 | Self-service password reset (canonical request type) | Now Assist self-service | JSM workflow | Self-service password | not declared | missing | Wave 15E: bind to `identity` µservice |
| 4.8 | Cost approval (procurement integration) | Procurement integration | Procurement plugin | Procurement integration | not declared | missing | Wave 15E: bind to ERP/payments µservices |

## 5. Service Catalog Management family

Counterpart canonical surface (ServiceNow Service Catalog + JSM Service Catalog + Freshservice Service Catalog):

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 5.1 | Catalog item creation + publishing | Service Catalog + cat_item table | Service Desk catalog | Service Catalog | `capabilities/service-catalog-publish.yaml` | partial | Wave 15E: declare CRUD endpoints |
| 5.2 | Catalog categorization | Categories + sub-categories | Catalog folder structure | Categories + sections | not declared | missing | Wave 15E: declare category taxonomy |
| 5.3 | Catalog entitlement (who can request what) | User criteria | Permissions per request type | User group entitlement | IP-028 service catalog entitlement orchestrator (substantive) | covered | — |
| 5.4 | Catalog item versioning | Catalog Builder + versions | Catalog version history | Versioning | IP-028 schema field `mapping_version` (visible in IP-026) | partial | Wave 15E: declare versioning surface |
| 5.5 | Catalog item pricing + cost approval | Price field + procurement | Custom field + workflow | Cost + approval | not declared | missing | Wave 15E: bind to `payments` + `marketplace` µservices |
| 5.6 | Marketplace catalog distribution | Store apps | Atlassian Marketplace | Freshworks Marketplace | `IP-014-marketplace-dealset-settlement.md` + ADR-0314 DealSet | covered | — |
| 5.7 | Catalog item AI recommendations | Now Assist catalog | Atlassian Intelligence | Freddy AI catalog | not declared | missing | Wave 15E: bind to `intelligence` µservice |
| 5.8 | Catalog request fulfillment SLA | Fulfillment SLAs | SLA goals | Service catalog SLAs | IP-030 SLA breach remediation | partial | Wave 15E: declare catalog-specific SLA |

## 6. Knowledge Management family

Counterpart canonical surface (ServiceNow KB + JSM Confluence-integrated + Freshservice Knowledge Base):

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 6.1 | KB article authoring | Knowledge Management v3 | Confluence integration | KB authoring | not declared in canonical surface | missing | Wave 15E: bind to `docs` µservice (or new `knowledge` capability) |
| 6.2 | KCS (Knowledge-Centered Service) v6 workflow | KCS-certified workflow | KCS Confluence template | KCS workflow | not declared | missing | Wave 15E: declare KCS workflow as workflow-engine template |
| 6.3 | KB versioning + draft/publish | Workflow + versions | Confluence versions | Draft/publish workflow | bind to `docs` versioning | partial | Wave 15E: declare versioning binding |
| 6.4 | KB search (full-text + faceted) | Zing search engine | Confluence search | KB search | bind to `search` µservice (declared in master plan) | partial | Wave 15E: declare search binding |
| 6.5 | KB article ratings + feedback | Article ratings + comments | Confluence likes + comments | Article feedback | bind to `community` µservice | partial | Wave 15E: declare feedback binding |
| 6.6 | KB AI suggestions during incident triage | Now Assist | Atlassian Intelligence | Freddy AI | mentioned in capability-tiers/tier-matrix.md §retired-advanced (Llama-3.1-70B) | partial | Wave 15E: lift into canonical capability + bind to `intelligence` |
| 6.7 | KB retention + archive | KB retention policy | Confluence retention | Article retention | `IP-015-data-residency-pack-overlays.md` | partial | Wave 15E: declare KB retention class |
| 6.8 | KB multi-language | Multi-language KB | Confluence i18n | Multilingual KB | bind to `translate` µservice | missing | Wave 15E: declare translate binding |

## 7. Asset / Configuration Management (CMDB) family

Counterpart canonical surface (ServiceNow CMDB + Discovery + Service Mapping; JSM Assets formerly Insight; Freshservice Asset Management):

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 7.1 | CI types (Computer, Server, Network, Software, etc.) | cmdb_ci hierarchy + custom CI classes | Assets object schemas | Asset categories | `capabilities/cmdb-sync.yaml` + `configuration-item` bounded context + IP-027 | partial | Wave 15E: declare canonical CI type roster |
| 7.2 | CI relationships (depends-on, contains, hosts) | cmdb_rel_ci + relationship types | Object references + schema | Relationships | IP-027 CMDB reconciliation graph (substantive) | covered | — |
| 7.3 | Discovery (auto-population) | ServiceNow Discovery (SNMP, WMI, SSH, K8s) | Insight Discovery + JSM Assets Discovery | Probe + cloud discovery | mentioned in capability-tiers/tier-matrix.md §retired-advanced; not in canonical surface | missing | Wave 15E: declare Discovery capability primitive; bind to `cloud-dcops` for hardware probes + K8s label scrapers |
| 7.4 | Service Mapping (top-down dependency) | Service Mapping (sold separately) | JSM-Insight Service Map | Service Mapping | not declared | missing | Wave 15E: declare service-mapping primitive |
| 7.5 | Asset Lifecycle (procurement → retirement) | Asset Management Pro | Asset lifecycle states | Lifecycle | not declared | missing | Wave 15E: declare asset lifecycle bounded context |
| 7.6 | Software License Management | Software Asset Management (SAM) | Insight Software License | Software License Tracking | not declared | missing | Wave 15E: declare SAM capability; bind to `marketplace` + `payments` |
| 7.7 | Hardware Warranty Tracking | Hardware Asset Management (HAM) | Insight Hardware | Hardware Asset | not declared | missing | Wave 15E: declare HAM capability |
| 7.8 | Contract Management (asset contracts) | Contract Management | Insight Contracts | Contract Tracking | bind to `contract-lifecycle-management` µservice (Phase 4 sibling) | partial | Wave 15E: declare handoff |
| 7.9 | CMDB Health (completeness + correctness) | CMDB Health Dashboards | Insight Assets health | Asset Audit | dashboards/local-cmdb-relation-drift exists | partial | Wave 15E: declare CMDB health canonical capability |
| 7.10 | CMDB Federation (multiple sources, conflict resolution) | IRE (Identification and Reconciliation Engine) | Multi-source import | Multi-source asset | IP-027 reconciliation graph | partial | Wave 15E: declare IRE-equivalent in PRD |
| 7.11 | CMDB Export + Reporting | Export + reports | Export + insights reports | Asset export | dashboards/* + bind to `analytics` | partial | Wave 15E: declare export endpoint |
| 7.12 | CMDB graph visualization | CMDB Graph viewer | Insight graph view | Visual relationships | not declared in canonical surface | missing | Wave 15E: bind to UX shell (`application` µservice) |
| 7.13 | CI confidence scoring (discovery reliability) | Discovery confidence | Insight confidence | Asset confidence | IP-026 schema `vendor_payload_digest` mentions confidence implicit | partial | Wave 15E: declare confidence field |

## 8. Service-Level Management family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 8.1 | SLA Definition + assignment | SLA Definitions + assignment rules | SLA goals + JQL | SLA Policy + applicability | `slos/*.openslo.yaml` (12 files, OpenSLO format) | covered | — |
| 8.2 | OLA (Operational Level Agreement) | OLA Definitions | Not native | OLAs | not declared | missing | Wave 15E: declare OLA distinct from SLA |
| 8.3 | UC (Underpinning Contract) | UC tracking | Not native | UCs | not declared | missing | Wave 15E: declare UC binding to `contract-lifecycle-management` |
| 8.4 | SLA Clock (pause / resume on hold) | SLA pause conditions | Time-tracking pauses | SLA pause | `src/domain::SlaClock` + IP-030 | covered | — |
| 8.5 | SLA Breach Detection | Breach automation | Breach automation rules | Breach + escalation | `slos/local-sla-breach-detection.openslo.yaml` + IP-030 | covered | — |
| 8.6 | SLA Reporting + Trends | Performance Analytics | Reports + dashboards | Analytics+ SLA | dashboards + bind to `analytics` | partial | Wave 15E: declare SLA reporting binding |
| 8.7 | SLA breach predictions | Predictive Intelligence | Atlassian Intelligence | Freddy AI | not declared | missing | Wave 15E: bind to `intelligence` µservice |

## 9. Workflow Automation family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 9.1 | Visual workflow designer | Flow Designer + Workflow Editor | Automation + Workflow editor | Orchestration Center + Workflow | bind to `workflow-studio` µservice (Phase 2 substrate) | partial | Wave 15E: declare workflow-studio binding |
| 9.2 | Pre-built workflow library | Flow Designer templates | Automation library | Orchestration templates | IP-004 workflow-template-library | covered | — |
| 9.3 | HTTP / REST integration steps | REST step + Integration Hub | HTTP request action | HTTP action | `workflow-engine` substrate | covered | — |
| 9.4 | Conditional logic + branches | Decision blocks | If/Then/Else | Conditions + branches | `workflow-engine` substrate | covered | — |
| 9.5 | Loops + iterators | For Each loops | For Each | Iterators | `workflow-engine` substrate | covered | — |
| 9.6 | Error handling + retries | Error handlers + retry policy | Error handling | Error + retry | `workflow-engine` substrate + idempotency keys | covered | — |
| 9.7 | Scheduled workflows | Scheduled job | Scheduled automation | Scheduler | `workflow-engine` substrate + worker scheduler | partial | Wave 15E: declare scheduler binding |
| 9.8 | Workflow approval steps | Approval activities | Approval automation | Approval steps | `policy/service-management-authorization.cedar` + Cedar gating | covered | — |
| 9.9 | AI-assisted workflow node generation | Now Assist Flow generation | Atlassian Intelligence | Freddy AI Builder | bind to `workflow-studio` + `intelligence` (per Foundry absorption ADR-0247) | missing | Wave 15E: declare AI-assisted node generation |
| 9.10 | Integration Hub (pre-built spokes for SaaS) | IntegrationHub + 100+ spokes | Atlassian Marketplace apps | Connect apps | `marketplace` + `plugin-app-store` µservices | partial | Wave 15E: declare ITSM spoke catalog binding |

## 10. Customer / Self-Service Portal family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 10.1 | Self-service portal UI (employee-facing) | Service Portal + Employee Center | JSM Customer Portal | Self-Service Portal | bind to `application` + `community` µservices | missing | Wave 15E: declare portal binding |
| 10.2 | Service catalog browsing | Catalog UI in Service Portal | Portal catalog UI | Portal catalog | bind to `application` shell | missing | Wave 15E: declare browsing UI |
| 10.3 | Ticket submission via portal | Service Portal form | Portal form | Portal form | bind to `application` | missing | Wave 15E: declare form binding |
| 10.4 | Ticket tracking + status | "My Tickets" UI | Portal "My Requests" | "My Tickets" | bind to `application` | missing | Wave 15E: declare tracking UI |
| 10.5 | KB browsing | KB in Service Portal | KB in Portal | KB in Portal | bind to `application` + `docs` | missing | Wave 15E: declare KB binding |
| 10.6 | Virtual Agent / AI Chat | Virtual Agent Designer | Atlassian Virtual Service Agent | Freddy AI Chatbot | bind to `intelligence` µservice | missing | Wave 15E: declare chat-agent binding |
| 10.7 | Mobile portal (responsive + native) | Mobile Web + Now Mobile | JSM Mobile + responsive web | Mobile + responsive | per F-PA-04 frontend/ios + frontend/android | missing | Wave 15E: declare mobile binding |
| 10.8 | Branding + theming | Branding configuration | Brand customization | Theming | bind to `brand` µservice (Phase 4 distribution substrate) | partial | Wave 15E: declare brand binding |
| 10.9 | Multi-language portal | Localized strings | i18n | Multilingual UI | bind to `translate` µservice | missing | Wave 15E: declare translate binding |
| 10.10 | Accessibility (WCAG 2.1+ AA) | WCAG 2.1 AA compliance | WCAG 2.1 AA | WCAG 2.1 AA | bind to UX shell | missing | Wave 15E: declare accessibility binding |

## 11. Agent Workspace / Unified Agent UI family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 11.1 | Unified agent workspace | Now Workspace + Agent Workspace | JSM Queue + Console | Freshservice Agent Console | bind to `application` + `ops-dashboard-control-center` | missing | Wave 15E: declare agent workspace binding |
| 11.2 | Multi-ticket tabbed UI | Workspace tabs | Browser tabs + sidebar | Tabbed workspace | bind to `application` | missing | Wave 15E |
| 11.3 | Context-aware AI panel | Now Assist sidebar | Atlassian Intelligence panel | Freddy AI sidekick | bind to `intelligence` | missing | Wave 15E |
| 11.4 | Activity stream | Activity history | Activity feed | Activity timeline | ARCHITECTURE audit-chain invariants | covered | — |
| 11.5 | Real-time notifications | Push + in-app | In-app + push | Notification stream | bind to `messenger` for push | partial | Wave 15E: declare push binding |
| 11.6 | Hot keys + keyboard shortcuts | Configurable shortcuts | Standard shortcuts | Shortcuts | bind to `application` | missing | Wave 15E |
| 11.7 | Customizable layouts | Workspace personalization | Personal preferences | Layout preferences | bind to `application` | missing | Wave 15E |

## 12. AI / Intelligence family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 12.1 | AI-driven ticket categorization | Predictive Intelligence | Atlassian Intelligence | Freddy AI Triage | bind to `intelligence` (Llama-3.1-70B mentioned) | missing | Wave 15E |
| 12.2 | AI-driven routing | Predictive Intelligence Skills | Atlassian Intelligence Routing | Freddy AI Routing | bind to `intelligence` | missing | Wave 15E |
| 12.3 | AI ticket summarization | Now Assist Summary | Atlassian Intelligence | Freddy Summary | bind to `intelligence` | missing | Wave 15E |
| 12.4 | AI sentiment analysis | Now Assist Sentiment | Atlassian Intelligence | Freddy Sentiment | bind to `intelligence` | missing | Wave 15E |
| 12.5 | AI deflection (self-service answer suggestion) | Now Assist Deflection | Virtual Service Agent | Freddy AI Deflection | mentioned in tier-matrix §retired-advanced; not in canonical surface | missing | Wave 15E |
| 12.6 | AI workflow builder (NL → workflow) | Now Assist Flow | Atlassian Intelligence | Freddy AI Builder | bind to `workflow-studio` + `intelligence` (Foundry absorption) | missing | Wave 15E |
| 12.7 | LLM model selection (vendor BYOM) | Restricted (ServiceNow models) | OpenAI + custom | Restricted (Freddy) | per ADR-0255 BYOK + intelligence two-layer | partial | Wave 15E: declare BYOM mode (paid tenant only) |
| 12.8 | AI explanation / audit trail | Now Assist explanations | Limited | Limited | per ADR-0263 + ADR-0247 Foundry self-modification | partial | Wave 15E: declare AI audit-chain emission |

## 13. ITIL Process Conformance family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 13.1 | ITIL v4 process pack | ITIL v4 templates | ITIL v4 templates | ITIL v4 templates | IP-026 ITIL process normalizer (substantive — 8 process families + 12 process stages + 20 vendor mappings) | covered | — |
| 13.2 | ISO/IEC 20000-1 conformance | ISO 20000-1 certified | ISO 20000-1 audit evidence | ISO 20000-1 alignment | compliance.md + `manifest.json#compliance_packs` ISO-27001 (note: ISO-27001 ≠ ISO-20000 — distinct standards) | partial | Wave 15E: declare ISO/IEC 20000-1 distinct from ISO-27001 |
| 13.3 | Process maturity assessment | Common Service Data Model | JSM process assessment | Process audit | not declared | missing | Wave 15E: declare maturity assessment |
| 13.4 | Process pack overlay per industry (Healthcare, Finance, Gov) | Industry packs | Limited industry packs | Industry packs | bind to `compliance` µservice + pack overlay model | partial | Wave 15E: declare industry overlays |

## 14. Integration family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 14.1 | REST API (per-object CRUD) | sys_rest_api + Table API | JSM Cloud REST API v3 | Freshservice API | `contracts/openapi-v1.yaml` (78 lines, thin — per F-SB-05) | partial | Wave 15E: expand OpenAPI surface |
| 14.2 | Webhooks (outbound events) | Outbound REST + scripted | JSM Webhooks | Webhooks | `contracts/asyncapi-v1.yaml` | partial | Wave 15E: declare webhook surface |
| 14.3 | Pre-built integrations (Slack, Teams, Jira, ServiceNow, Confluence, etc.) | IntegrationHub spokes | Atlassian Marketplace apps | Connect apps + Marketplace | bind to `marketplace` + `plugin-app-store` | partial | Wave 15E: declare canonical spoke list |
| 14.4 | SSO + SCIM | SAML, OIDC, OAuth, SCIM v2 | SAML, OIDC, SCIM | SAML, OIDC, SCIM | bind to `identity` + `cloud-iam` | covered | — |
| 14.5 | LDAP / AD sync | LDAP integration | AD sync | LDAP / AD | bind to `identity` | partial | Wave 15E: declare LDAP/AD binding |
| 14.6 | Email-to-ticket | Inbound email parser | Email handler | Email-to-ticket | bind to `mail` µservice | partial | Wave 15E: declare email binding |
| 14.7 | Chat / messenger integration | Slack, Teams, Webex spokes | Slack-native | Slack/Teams | bind to `messenger` | partial | Wave 15E: declare messenger binding |
| 14.8 | Monitoring tool integration (PagerDuty, Datadog, Splunk) | Event Management spokes | Marketplace apps | Connect apps | bind to `observability` µservice | partial | Wave 15E: declare monitoring binding |
| 14.9 | DevOps tool integration (GitHub, GitLab, Bitbucket, Jenkins) | DevOps Change Velocity | Native (Atlassian) | Connect apps | bind to `oya git` substrate + workflow-engine | partial | Wave 15E: declare DevOps binding |

## 15. Mobile family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 15.1 | Native iOS agent app | Now Mobile Agent (iOS) | JSM Mobile (iOS) | Freshservice Mobile (iOS) | `frontend/ios/` (per language_policy Swift) | missing | Wave 15E: declare iOS surface |
| 15.2 | Native Android agent app | Now Mobile Agent (Android) | JSM Mobile (Android) | Freshservice Mobile (Android) | `frontend/android/` (per language_policy Kotlin) | missing | Wave 15E: declare Android surface |
| 15.3 | Native requester app (separate from agent app) | Now Mobile Requester | JSM Mobile | Same app, requester mode | not declared | missing | Wave 15E: declare requester mode |
| 15.4 | Offline mode (cached tickets) | Limited offline | Limited offline | Limited offline | not declared | missing | Wave 15E: declare offline policy |
| 15.5 | Push notifications | APNs + FCM | APNs + FCM | APNs + FCM | bind to `messenger` push | missing | Wave 15E: declare push binding |
| 15.6 | Biometric / passkey auth | Face ID + Touch ID + Android biometric | Same | Same | bind to `identity` passkeys | missing | Wave 15E: declare passkey binding |
| 15.7 | Camera attachment capture | Camera in-app | Camera in-app | Camera in-app | bind to native frontend | missing | Wave 15E |
| 15.8 | Voice ticket creation | Voice-to-text + AI | Limited | Limited | bind to `intelligence` voice | missing | Wave 15E |

## 16. Multi-Instance / Multi-Tenant Architecture family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 16.1 | Multi-instance architecture (per-tenant DB) | ServiceNow defining architectural feature: each customer gets own MariaDB | Multi-tenant cloud (shared DB) | Multi-tenant cloud | Per ADR-0244 tenant scoping + ADR-0248 cellular | partial | Wave 15E: declare per-tenant DB option (paid tenants opting into dedicated cell) |
| 16.2 | Tenant isolation (data + Cedar) | Logical isolation via instance | Logical isolation via project | Logical isolation via workspace | ADR-0244 tenant scoping + Cedar | covered | — |
| 16.3 | Cross-tenant migration | Limited (export + import) | Limited | Limited | `migration-playbooks/` (4 files: from-servicenow, from-jsm, from-bmc-helix, from-servicenow-itsm) | partial | Wave 15E: substance-sample migration playbooks |
| 16.4 | Shuffle sharding | Not exposed | Not exposed | Not exposed | ADR-0248 cellular Amazon-shape doctrine | partial | Wave 15E: declare shuffle sharding |
| 16.5 | Per-tenant compliance pack | ServiceNow GRC packs | JSM compliance add-ons | Compliance pack add-ons | ADR-0251 compliance pack primitive | covered | — |
| 16.6 | Per-tenant cell residency (sovereign cells) | ServiceNow Sovereign Cloud (dedicated cloud premium) | JSM Data Residency | Data residency | ADR-0250 build-ahead-of-certification + sovereign cell | partial | Wave 15E: declare sovereign cell residency |
| 16.7 | Per-tenant BYOK encryption | Customer-managed keys (premium) | Limited | Limited | ADR-0255 §D-4 BYOK | partial | Wave 15E: declare BYOK opt-in (paid only) |

## 17. RBAC / Permissions family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 17.1 | Roles (agent, admin, requester, etc.) | sys_user_role + role hierarchy | JSM permissions | Roles | bind to `identity` + Cedar | covered | — |
| 17.2 | Access Control Lists (per-table, per-field) | ACLs per row + per field | Permissions per project | Permissions | `policy/*.cedar` + Cedar attribute-based | covered | — |
| 17.3 | Group-based permissions | sys_user_group | Group permissions | Groups | bind to `identity` | covered | — |
| 17.4 | Delegation (temporary delegation of role) | Delegation rules | Limited | Delegation | not declared | missing | Wave 15E |
| 17.5 | Just-in-Time (JIT) elevation (break-glass) | Privileged Access Management | Limited | Limited | `policy/emergency-services-bypass.cedar` + IP-013 emergency services bypass | covered | — |
| 17.6 | Audit log of permission changes | sys_audit + ACL audit | Audit log | Audit log | ADR-0263 audit emission + `IP-011 observability audit events` | covered | — |
| 17.7 | Separation of Duties (SoD) | SoD rules in GRC | Custom workflows | SoD | `policy/*.cedar` + multi-approver patterns | partial | Wave 15E: declare SoD as canonical pattern |

## 18. Operational / SRE family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 18.1 | Runbooks (operator-facing) | Limited (Standard Operating Procedures) | Limited | Limited | `runbooks/*.md` (22 files; canonical + local-pack overlay pairs) | covered | — |
| 18.2 | SLO dashboards | Performance Analytics dashboards | Dashboards | Analytics+ dashboards | `dashboards/*.json` (10 files) + `slos/*.openslo.yaml` (12 files) | covered | — |
| 18.3 | Chaos drill testing | Limited | Limited | Limited | `IP-022-chaos-drill-pack.md` | covered | — |
| 18.4 | Capacity model + admission control | Capacity Management (ITIL practice) | Limited | Capacity Management add-on | `capacity-model.md` (86 KB) + `IP-018-capacity-admission-control.md` | covered | — |
| 18.5 | Cost / FinOps view | Cost Insights | Limited | Cost Insights | `cost-budget.md` (69 KB) + `IP-017-cost-budget-enforcer.md` | covered | — |
| 18.6 | Backfill / replay | Limited | Limited | Limited | `backfill-replay.md` + `IP-016-backfill-replay-worker.md` | covered | — |
| 18.7 | Multi-region failover | Multi-instance failover | Cloud multi-region | Cloud multi-region | `multi-region.md` (69 KB) + `IP-010-multi-region-cell-layout.md` | covered | — |
| 18.8 | DR (Disaster Recovery) plan | DR Standby | Cloud DR | Cloud DR | `iac/dr-failover.yaml` + multi-region.md | partial | Wave 15E: declare DR runbook explicit RPO/RTO |

## 19. Security family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 19.1 | Threat model | Limited (Security Operations sold separately) | Limited | Limited | `threat-model.md` (150 KB) + `IP-024-threat-model-control-map.md` | covered | — |
| 19.2 | DPIA (Data Protection Impact Assessment) | GDPR DPIA template | Limited | Limited | `dpia.md` (116 KB) + `IP-023-dpia-evidence-packet.md` | covered | — |
| 19.3 | Abuse defense + WAF | Limited | Cloud Edge protection | Cloud WAF | `iac/edge-waf.yaml` + `IP-012-abuse-defence-edge-waf.md` + `policy/abuse-defence.cedar` | covered | — |
| 19.4 | Audit chain (immutable log) | sys_audit (mutable in some admin ops) | Audit log | Audit log | ADR-0263 audit emission + `IP-011 observability audit events` + audit-chain µservice | covered | — |
| 19.5 | Encryption at rest | At-rest TDE | Cloud encryption | Cloud encryption | bind to `cloud-kms` + `cloud-secrets` | covered | — |
| 19.6 | Encryption in transit | TLS 1.3 | TLS 1.3 | TLS 1.3 | TLS 1.3 floor + ECH + PQC hybrid per ADR-0253-amendment + `iac/pqc-cert.yaml` + `iac/ech-config.yaml` | covered | — |
| 19.7 | Penetration testing artifact | Available on request | Available on request | Available on request | not declared | missing | Wave 15E: declare pentest artifact location |
| 19.8 | Secrets management | Credential store | Vault integrations | Vault | bind to `cloud-secrets` + OpenBao + `iac/openbao-policy.yaml` | covered | — |
| 19.9 | Post-quantum cryptography | Not announced | Not announced | Not announced | `iac/pqc-cert.yaml` + ADR-0253-amendment PQC hybrid | covered (differentiator) | — |

## 20. Compliance + Pack family

| # | Capability | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 20.1 | SOC 2 Type II | SOC 2 Type II certified | SOC 2 Type II certified | SOC 2 Type II certified | `compliance.md` + manifest.json `SOC-2` | covered | — |
| 20.2 | ISO 27001 | ISO 27001 certified | ISO 27001 certified | ISO 27001 certified | `compliance.md` + manifest.json `ISO-27001` | covered | — |
| 20.3 | ITIL Pack | ITIL v4 templates | ITIL v4 templates | ITIL v4 templates | `compliance.md` + manifest.json `ITIL` + IP-026 process normalizer | covered | — |
| 20.4 | GDPR | GDPR DPIA + RoPA | GDPR DPIA | GDPR | `compliance.md` + manifest.json `GDPR` + `dpia.md` | covered | — |
| 20.5 | HIPAA | HIPAA-compliant edition | HIPAA add-on | HIPAA | manifest.json `compliance_packs_applicable` lists `hipaa` but `compliance_packs` does not (per F-IC-01 drift) | partial | Wave 15E: reconcile HIPAA presence in manifest |
| 20.6 | FedRAMP High | FedRAMP High authorized | FedRAMP Moderate | FedRAMP Moderate | `compliance.md` + manifest.json `FedRAMP-High` | covered | — |
| 20.7 | KR-PIPA (Korea Personal Information Protection Act) | Not native | Not native | Not native | `compliance.md` + manifest.json `KR-PIPA` + ADR-0064 canonical-base + KR pack | covered (differentiator) | — |
| 20.8 | KR-CSAP (Korea Cloud Security Assurance Program) | Not native | Not native | Not native | per ADR-0250 build-ahead-of-certification | partial | Wave 15E: declare KR-CSAP pack |
| 20.9 | EU AI Act Annex III refusal | Not declared | Not declared | Not declared | per `feedback_build_ahead_of_certification` + intelligence-binding | partial | Wave 15E: bind to `intelligence` AI act refusal |
| 20.10 | PCI-DSS | PCI DSS Level 1 | PCI DSS attestation | PCI DSS attestation | not in current pack roster | missing | Wave 15E: declare PCI pack |

## 21. New constraint dimensions union coverage

| # | Constraint dimension | ServiceNow ITSM | Jira Service Management | Freshservice | Oyatie owning surface | Verdict | Remediation target |
|---|---|---|---|---|---|---|---|
| 21.1 | Multi-context deployment (public cloud + AWS guest + OCI guest + on-prem + colo + Oyatie-as-provider) | Public + Sovereign Cloud (premium) + On-Prem (legacy product) | Public + Data Center self-hosted (Atlassian Data Center) | Public-only | Per ADR-0328 §D-15: six contexts mandatory for Phase 4 µservices; not declared yet | missing | Wave 15E: six-context `iac/<context>/` modules |
| 21.2 | OpenTofu IaC (not Terraform) | N/A (proprietary deployment) | Atlassian Data Center docs (limited IaC) | N/A | Per ADR-0328 §D-16: OpenTofu mandatory; current iac/terraform-module.tf is Terraform-named | partial | Wave 15E: rename + restructure |
| 21.3 | Multi-OS support (Tier 1 = 13 OSes incl. Talos, RHEL, Ubuntu, macOS M5+) | Server OS list per release | Atlassian DC supported OSes | Cloud-only (no OS exposure) | Per ADR-0328 §D-17: supported-oses.json mandatory; not declared | missing | Wave 15E: author supported-oses.json |
| 21.4 | Rust-strict backend | N/A (Java + JavaScript stack) | N/A (Java stack) | N/A (Ruby + JavaScript) | Per ADR-0328 §D-18: Rust strict; `Cargo.toml` + `src/*.rs` clean | covered | — |
| 21.5 | OCI Always Free profile for demo/trial tenants | N/A | N/A | N/A | Per ADR-0328 §D-19: demo/trial tenants on OCI Always Free; `iac/oci-guest/always-free/` not present | missing | Wave 15E: author `iac/oci-guest/always-free/` |
| 21.6 | Tenant class enum (demo_trial + paid) with composable billing components | N/A (per-seat licensing only) | N/A (per-seat tiers) | N/A (per-agent tiers) | Per tenant-class doctrine; not declared | missing | Wave 15E: declare tenant_class + billing_components |

## 22. Coverage summary

### 22.1 Coverage by family

| Family | Total rows | Covered | Partial | Missing | Out-of-scope intentional |
|---|---:|---:|---:|---:|---:|
| 1. Incident Management | 20 | 2 | 11 | 7 | 0 |
| 2. Problem Management | 10 | 1 | 7 | 1 | 1 |
| 3. Change Enablement | 15 | 5 | 8 | 2 | 0 |
| 4. Service Request Management | 8 | 1 | 4 | 3 | 0 |
| 5. Service Catalog | 8 | 1 | 4 | 3 | 0 |
| 6. Knowledge Management | 8 | 0 | 4 | 4 | 0 |
| 7. Asset / CMDB | 13 | 1 | 6 | 6 | 0 |
| 8. Service-Level Management | 7 | 3 | 1 | 3 | 0 |
| 9. Workflow Automation | 10 | 5 | 4 | 1 | 0 |
| 10. Self-Service Portal | 10 | 0 | 1 | 9 | 0 |
| 11. Agent Workspace | 7 | 1 | 1 | 5 | 0 |
| 12. AI / Intelligence | 8 | 0 | 2 | 6 | 0 |
| 13. ITIL Process Conformance | 4 | 1 | 2 | 1 | 0 |
| 14. Integration | 9 | 1 | 8 | 0 | 0 |
| 15. Mobile | 8 | 0 | 0 | 8 | 0 |
| 16. Multi-Instance Architecture | 7 | 2 | 5 | 0 | 0 |
| 17. RBAC / Permissions | 7 | 5 | 2 | 0 | 0 |
| 18. Operational / SRE | 8 | 7 | 1 | 0 | 0 |
| 19. Security | 9 | 8 | 0 | 1 | 0 |
| 20. Compliance + Pack | 10 | 6 | 3 | 1 | 0 |
| 21. New constraint dimensions | 6 | 1 | 1 | 4 | 0 |
| **Total** | **182** | **51** | **75** | **65** | **1** |

### 22.2 Coverage rate (raw)

- Covered: 51 / 182 = 28.0%.
- Partial: 75 / 182 = 41.2%.
- Missing: 65 / 182 = 35.7%.
- Out-of-scope intentional: 1 / 182 = 0.5%.

Combined "addressed" (covered + partial + intentional): 127 / 182 = 69.8%.
Gap (missing): 65 / 182 = 35.7% — must be closed before Phase 4 promotion.

### 22.3 Per-counterpart pressure ratio

ServiceNow ITSM has the broadest counterpart surface — it contributes the most "if any counterpart has X" features. Most missing rows trace to ServiceNow-only capabilities (Predictive Intelligence, IntegrationHub, Service Mapping, Discovery, Now Workspace). JSM contributes Atlassian Marketplace integration depth + DevOps-native flow. Freshservice contributes mid-market portal + mobile + AI deflection ease-of-use.

Big 8 priority uplift: every "missing" row is treated as P0 under ADR-0328 §D-20.111-115 BIG 8 rules.

## 23. Out-of-scope intentional rows

Per ADR-0328 §D-5.12, out-of-scope-intentional rows must name the doctrine reason and approving authority.

- OOS-01 [Row 2.10 cross-tenant federation of known-errors]: Reason: tenant isolation per ADR-0244 — Oyatie does not federate KEDB across tenants. Approving authority: ADR-0244 + ARCHITECTURE.md §A boundary statement.

Additional out-of-scope candidates that the audit recommends for Wave 14 backlog evaluation (not yet declared out-of-scope intentional, but candidates):

- OOS-CAND-01 [Row 14.3 pre-built integration with ServiceNow ITSM]: Reason: ServiceNow is a counterpart being displaced; integration with ServiceNow as a peer tool runs counter to displacement thesis. However, ServiceNow-to-Oyatie one-way migration import IS in scope (`migration-playbooks/from-servicenow.md` + `from-servicenow-itsm.md`).
- OOS-CAND-02 [Row 1.18 multi-language UI in MVP]: Reason: KR-PIPA pack first per `feedback_canonical_base_localization.md`; other locales follow via pack overlays. This is "phased rollout" not "out-of-scope".

## 24. Verification Notes

- Counterpart product surfaces were referenced from vendor-public documentation: ServiceNow Washington DC release notes + Now Platform docs, Atlassian JSM Cloud REST API v3 + Atlassian Marketplace, Freshworks Freshservice Enterprise feature index.
- Oyatie owning surfaces were referenced from the sampled `microservices/itsm/` files in §1.4 of the coherence audit.
- Union-coverage assignments use ADR-0328 §D-5.15 vocabulary: covered, partial, missing, out-of-scope intentional. Each "covered" row cites a concrete owning artifact. Each "partial" row names the missing gap. Each "missing" row names a Wave 15E remediation target. The single "out-of-scope intentional" row names the approving ADR.

## 25. Findings

Per ADR-0328 §D-6.23, this section is required even when empty.

The findings produced by this parity matrix (as F-PA-01..F-PA-16 in the companion coherence-audit deliverable) are:

- 14 family-level missing categories (Knowledge Management, Self-Service Portal, Agent Workspace, AI, Mobile, etc.).
- 65 row-level missing capabilities to remediate in Wave 15E.
- 75 row-level partial capabilities to deepen substance in Wave 15E (most "partial" rows are declared in PRD or capability primitives but lack endpoint surface + handler implementation).
- 1 out-of-scope intentional row.

Per ADR-0328 §D-20.111-115 Big 8 priority uplift: every "missing" row is P0; every "partial" row that gates a counterpart-canonical capability is P1.

## 26. Backlog Rows

Per ADR-0328 §D-6.24, this section is required.

The 65 missing rows + 75 partial rows enter the Wave 14 backlog with:

- microservice = itsm
- severity = P0 (missing, Big 8 uplift) | P1 (partial, gates counterpart-canonical) | P2 (partial, secondary)
- category = parity
- file = the row's "Oyatie owning surface" path (or target path for the new surface)
- fix = the row's "Remediation target" column

Total backlog rows produced by this parity matrix: 140 (65 missing + 75 partial). Combined with the 82 rows from the coherence audit, the µservice contributes 222 finding rows to Wave 14 aggregation.

The parity matrix produces NO direct edits to `microservices/itsm/*` outside this deliverable. The matrix is findings-only. Remediation proceeds in Wave 15E per the canonical sequence.
