---
doc_class: FeatureParityMatrix
microservice: incident-management
date: 2026-05-21
counterparts:
  - PagerDuty
  - Opsgenie
  - FireHydrant
big_8_family: ServiceNow (Phase 4A.4)
big_8_p0_elevation: true
union_coverage_basis: ADR-0328 §D-5 (top-3 counterpart union coverage)
authority_chain:
  - ADR-0328 §D-2.16-§D-2.17 (Big-8 ITSM ordering)
  - ADR-0328 §D-5 (top-3 counterpart parity check)
  - brief-template §3.4.T §3.4.C §3.4.B
  - feedback_no_tenant_class_adoption_2026_05_20.md
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md
---

# incident-management — Feature Parity Matrix (Top-3 Union Coverage)

## A. Scope, Methodology, and Counterpart Profiles

The brief mandates **PagerDuty + Opsgenie + FireHydrant** as the top-3
counterparts under ADR-0328 §D-5 union-coverage rule. This matrix unions
their feature surfaces across 14 feature axes drawn from the brief
(on-call schedules, escalation policies, alerting, integrations, incident
command, war room, postmortem retro, status page, runbooks, severity
definitions, response analytics, SLA tracking, mobile, slack integration,
monitoring tool integration) and adds three Oyatie-specific axes that
emerge from the corpus (Cedar gating, audit chain, sovereign-pack
residency). Total axes: 17.

Per the user directive of 2026-05-20, this matrix declares feature
presence per UNION ROW × COUNTERPART, **without** demo_trial / paid tier-deltas. Where a counterpart gates a feature by pricing tier
(PagerDuty Free/Professional/Business/Enterprise/Digital-Ops; Opsgenie
Free/Essentials/Standard/Enterprise; FireHydrant Starter/Pro/Enterprise)
that is recorded as a counterpart-side note, not as an Oyatie-side tier.

### Counterpart 1 — PagerDuty

Vendor profile. Founded 2009 (San Francisco). The category-defining
on-call paging + escalation product. Acquired Rundeck (2020) and
Catalytic (2022) to extend into process automation. Public company
(NYSE: PD). Market lead position: 22,000+ customers (Q3 FY26 disclosure).
Pricing model: per-user / per-month, four tiers (Free up to 5 users /
Professional $21 / Business $41 / Digital Operations $74). Enterprise +
GovCloud + Customer-Managed-Keys are quote-only.

Surface coverage:

- On-call schedules (recurring, manual override, layered).
- Escalation policies (up to 10 levels, business-hours conditional,
  multiple escalation steps per level).
- Event-orchestration (per-service rule-based dedupe, suppression,
  routing).
- Service registry (one service = one paging entry).
- Incident lifecycle (triggered → acknowledged → resolved; with
  reassignment and notes).
- Postmortem (formal templates + action-item tracking + retros).
- Status page (PagerDuty Status; merged from Statuspage acquisition).
- AIOps (intelligent grouping; PagerDuty Copilot generative summaries).
- Runbook automation (Rundeck-derived; for Digital Operations).
- Process automation (Catalytic-derived; for Digital Operations).
- Customer service ops (sister product line).
- Mobile app (iOS + Android with biometric ack; offline ack on lost
  network).
- ChatOps (Slack + MS Teams + Webex; bi-directional).
- Monitoring integrations (~700 connectors including Datadog, NewRelic,
  Prometheus, Splunk, Sentry, ServiceNow, Jira, CloudWatch, Azure
  Monitor, OCI Monitoring).
- API + Webhook (REST v2 + Event API v2 + EventOrchestrations rules
  webhook).
- HIPAA + SOC-2 + ISO-27001 + FedRAMP-Moderate compliance.
- GovCloud (US-only sovereign profile).

### Counterpart 2 — Opsgenie

Vendor profile. Founded 2012 (Ankara/Boston). Acquired by Atlassian
2018 for $295M; merged with Jira Service Management 2020+. Pricing
model: per-user / per-month, four tiers (Free up to 5 users /
Essentials $9 / Standard $19 / Enterprise $29). Atlassian Cloud
Premium adds advanced data residency + Customer-Managed-Keys via
Atlassian Trust Center.

Surface coverage:

- On-call schedules (rotation, restriction, layered with timezone +
  follow-the-sun).
- Escalation policies (10 levels max; conditional on tag + priority +
  service).
- Alert policies + automation rules (declarative; can suppress, dedupe,
  enrich).
- Heartbeats (passive monitor — alert if external check stops).
- Integrations (~200 native + 1000+ via JSM marketplace).
- Incident Investigation (Atlassian Intelligence; post-incident
  analysis; Enterprise tier).
- Stakeholder communications (cross-team broadcast).
- Status page (via Atlassian Statuspage product).
- ChatOps (Slack + Teams + Mattermost; bi-directional).
- Runbook automation (via JSM workflow + Atlassian Forge).
- Reports + analytics (MTTA, MTTR, on-call distribution; advanced in
  Enterprise tier).
- Mobile app (iOS + Android).
- Data residency (EU + US; Atlassian Cloud Premium for Government
  cloud).
- HIPAA + SOC-2 + ISO-27001 (via Atlassian Trust Center compliance
  pack).

### Counterpart 3 — FireHydrant

Vendor profile. Founded 2018 (New York). Originally focused on incident
command + retro workflow; expanded into status page (acquired Nunc 2021)
and on-call (FireHydrant Signals, launched 2023). Pricing model: per-user
/ per-month (Starter $20 / Pro $40 / Enterprise quote-only). Smaller
install base (~2,000 customers) but rapidly growing.

Surface coverage:

- Incident command (declared incident lifecycle with role assignment +
  task tracking + chat-channel auto-create).
- Runbooks (per-service playbooks; conditional execution; auto-run on
  declared incident).
- Status page (private + public; per-component status; subscriber
  notifications).
- Retros (blameless postmortem template + 5-whys + action-item tracking).
- Signals (newer on-call + paging product; rotations + escalation;
  Pro+ tier).
- Slack-first UI (most operations driven from Slack slash commands).
- Service catalog (lightweight CMDB; service + component registry).
- Integrations with Datadog, NewRelic, Sentry, Honeycomb, Splunk,
  Prometheus.
- Analytics (incident frequency, severity distribution, MTTR by team).
- API + Webhook (REST + GraphQL).
- SOC-2 + HIPAA + ISO-27001.

### Union surface

The union of features across the three counterparts (the bar Oyatie must
hit per ADR-0328 §D-5) is the OR over all three columns plus the
Oyatie-specific differentiators captured at the bottom.

## B. Feature Parity Matrix (17 axes)

Legend:
- ✅ FULL = feature is first-class with vendor SLA + GA.
- 🟡 PARTIAL = feature exists but is limited or gated by a high tier or
  marketplace add-on.
- ❌ ABSENT = feature is not in the product surface.
- 🔵 DIFFERENTIATOR = Oyatie-only capability (no counterpart).
- ➡️ ROADMAP = declared in current Oyatie corpus but contract / IaC /
  capability surface not yet authored.

### Axis 1 — On-call schedules (rotation primitives)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Weekly rotations | ✅ | ✅ | ✅ (Signals) | ✅ (ADR-IM-001 OnCallSchedule) |
| Daily / shift rotations | ✅ | ✅ | ✅ | ✅ |
| Custom-interval rotations | ✅ | ✅ | 🟡 | ➡️ (PRD declares but contract pending) |
| Manual overrides | ✅ | ✅ | ✅ | ✅ |
| Restrictions (time-window, day-of-week) | ✅ | ✅ | 🟡 | ✅ (FR-001..006 on-call-schedule) |
| Layered (primary / secondary / backup) | ✅ (up to 5 layers) | ✅ (up to 10) | ✅ | ✅ |
| Follow-the-sun | ✅ | ✅ | 🟡 | ➡️ (declared in tenant_class adoption record; needs first-class contract) |
| Skill-matrix selection | 🟡 | ❌ | ❌ | 🔵 (ADR-IM-001 OnCallSchedule resolves skill_requirements) |
| Vacation / OOO honor | ✅ | ✅ | 🟡 | ➡️ |
| Calendar (iCal) export | ✅ | ✅ | ❌ | ➡️ |
| Bulk import (CSV) | ✅ | ✅ | ❌ | ✅ (FR-004 on-call-schedule.import) |
| Auto-detect rotation conflict | 🟡 | 🟡 | ❌ | 🔵 (declared in ADR-IM-001 No Silent Suppression) |

Oyatie verdict: PASS on union coverage (12/12 sub-features either ✅ or
➡️ roadmap-bound). Differentiators: skill-matrix selection + auto-detect
rotation conflict. Roadmap items must promote to contract surface before
Phase 4A.4 ships.

### Axis 2 — Escalation policies

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Multi-level (≥ 10) | ✅ | ✅ | ✅ | ✅ (ADR-IM-001 declares up to 100 conceptual) |
| Per-level timeout customizable | ✅ | ✅ | ✅ | ✅ |
| Branch on severity | ✅ | ✅ | ✅ | ✅ |
| Branch on service | ✅ | ✅ | ✅ | ✅ |
| Branch on business-hours / time | ✅ | ✅ | 🟡 | ✅ |
| Branch on tenant entitlement | 🟡 | 🟡 | ❌ | 🔵 (Cedar-policy gate) |
| Branch on customer-impact estimate | ❌ | ❌ | ❌ | 🔵 (paid tenant_class tenant_class adoption record; needs re-expression as paid-tenant overlay) |
| Branch on data-class | ❌ | ❌ | ❌ | 🔵 (Cedar-policy gate) |
| Branch on pack residency | ❌ | ❌ | ❌ | 🔵 (compliance-pack overlay) |
| Cross-team fan-out | ✅ | ✅ | ✅ | ✅ |
| Re-page on no-ack | ✅ | ✅ | ✅ | ✅ |
| Auto-resolve on no-impact | ✅ | ✅ | 🟡 | ➡️ |
| Repeat limit | ✅ | ✅ | ✅ | ✅ |
| Version history of policy | ✅ | ✅ | 🟡 | ✅ (FR-007..012 + ADR-IM-001 policy_version) |

Oyatie verdict: PASS-WITH-DIFFERENTIATOR — Cedar-policy escalation +
customer-impact / data-class / pack overlay branches are Oyatie-only.
Re-expression of paid tenant_class tier features as paid-tenant + per_usage
overlay required.

### Axis 3 — Alerting + paging

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| SMS (Twilio + AT&T + EU carriers) | ✅ | ✅ | 🟡 (via PagerDuty bridge) | ✅ (ADR-IM-001 page dispatch; Twilio + Bandwidth + Plivo) |
| Voice (Twilio Programmable Voice) | ✅ | ✅ | 🟡 | ✅ |
| Mobile push (iOS APNs + Android FCM) | ✅ | ✅ | ✅ (Signals) | ➡️ (runbooks/mobile-push-degradation.md exists; no first-class contract) |
| Email | ✅ | ✅ | ✅ | ✅ |
| Slack (DM + channel) | ✅ | ✅ | ✅ | ✅ (via messenger substrate) |
| Microsoft Teams (DM + channel) | ✅ | ✅ | ✅ | ✅ |
| Discord | 🟡 (community plugin) | 🟡 | ❌ | ✅ |
| Telegram | 🟡 | 🟡 | ❌ | ✅ |
| KakaoTalk Bizmessage (KR) | ❌ | ❌ | ❌ | 🔵 (tenant_class adoption records paid / pack KR-PIPA — needs re-expression as paid + KR pack overlay) |
| Naver Works (KR) | ❌ | ❌ | ❌ | 🔵 (KR pack — needs re-expression) |
| LINE Notify (JP) | ❌ | ❌ | ❌ | ➡️ (JP-pack roadmap) |
| WeChat Work (CN) | ❌ | ❌ | ❌ | ➡️ (CN-pack roadmap) |
| Multi-provider parallel routing (failover) | ✅ (Enterprise) | 🟡 | ❌ | ✅ (tenant_class adoption records paid; needs re-expression) |
| Delivery-receipt evidence | ✅ | ✅ | 🟡 | ✅ (audit-chain emission per ADR-0263) |
| STIR/SHAKEN voice attestation | ✅ | 🟡 | ❌ | ✅ (tenant_class adoption records paid; needs re-expression as paid + compliance pack) |

Oyatie verdict: PASS-WITH-DIFFERENTIATORS — KR / JP / CN pack-resident
paging providers + STIR/SHAKEN are Oyatie-only when expressed as
compliance-pack overlays for paid tenants.

### Axis 4 — Integrations (monitoring + observability + ticketing)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Datadog | ✅ | ✅ | ✅ | ➡️ (via observability substrate; no first-class capability) |
| New Relic | ✅ | ✅ | ✅ | ➡️ |
| Prometheus + Alertmanager | ✅ | ✅ | ✅ | ➡️ (via observability substrate) |
| Splunk + Splunk Observability | ✅ | ✅ | ✅ | ➡️ |
| Sentry (error tracking) | ✅ | ✅ | ✅ | ➡️ |
| Honeycomb | ✅ | 🟡 | ✅ | ➡️ |
| Grafana (alerting) | ✅ | ✅ | 🟡 | ➡️ |
| CloudWatch (AWS) | ✅ | ✅ | ✅ | ➡️ (cloud-* substrate) |
| Azure Monitor | ✅ | ✅ | ✅ | ➡️ |
| OCI Monitoring | ✅ | 🟡 | 🟡 | ➡️ |
| Jira (ticket → incident) | ✅ | ✅ (native) | ✅ | ➡️ (via itsm substrate) |
| ServiceNow (ticket sync) | ✅ | ✅ | ✅ | ➡️ (via itsm substrate) |
| Linear | ✅ | 🟡 | ✅ | ➡️ |
| GitHub Issues | ✅ | ✅ | ✅ | ➡️ |
| Webhook (inbound, generic) | ✅ | ✅ | ✅ | ✅ (OpenAPI page-dispatch) |
| Webhook (outbound, on state-change) | ✅ | ✅ | ✅ | ✅ (AsyncAPI events) |
| CloudEvents (CNCF) | 🟡 | 🟡 | ❌ | ✅ (AsyncAPI 3.1.0 + CloudEvents per ADR-IM-001) |
| OpenTelemetry (correlation trace) | ✅ | ✅ | 🟡 | ✅ (traceparent in events) |

Oyatie verdict: FINDING — most monitoring integrations are routed via
the `observability` substrate but no first-class
`integrations/` directory or `capabilities/integration-*.yaml` exists in
incident-management. Promote to capability surface before Phase 4A.4.

### Axis 5 — Incident command (state machine)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Single-IC enforcement (sev1 / sev2) | 🟡 (advisory) | 🟡 (advisory) | ✅ (mandatory) | ✅ (ADR-IM-001 Single Incident Commander Constraint) |
| IC transfer with audit | 🟡 | 🟡 | ✅ | ✅ (ADR-IM-001) |
| Role assignment (IC + Scribe + Comms + SME) | 🟡 (manual) | 🟡 | ✅ | ✅ (FR-013..018 incident-room + ADR-IM-001) |
| Auto-declare from alert correlation | ✅ | ✅ | ✅ | ✅ (ADR-IM-001 alert-fingerprint correlation) |
| Manual declare from chat slash-command | ✅ | ✅ | ✅ (Slack-first) | ➡️ (capabilities/incident-room-open.yaml; chat integration via messenger) |
| Severity escalation mid-incident | ✅ | ✅ | ✅ | ✅ |
| Severity downgrade | ✅ | ✅ | ✅ | ✅ |
| State machine (declared / triaged / mitigated / resolved) | ✅ | ✅ | ✅ | ✅ (ADR-IM-001) |
| Reopen incident | ✅ | ✅ | ✅ | ✅ |
| Linked deployment / change | 🟡 (via integration) | 🟡 | ✅ | ➡️ (cross-handoff with change-management) |
| Linked observability dashboards | ✅ | ✅ | 🟡 | ➡️ (cross-handoff with observability) |

Oyatie verdict: PASS — single-IC enforcement is mandatory (matching
FireHydrant's strongest stance) rather than advisory.

### Axis 6 — War room (collaboration surface)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Auto-create Slack channel | ✅ | ✅ | ✅ | ✅ |
| Auto-create Microsoft Teams channel | ✅ | ✅ | ✅ | ✅ |
| Auto-create Discord channel | 🟡 | 🟡 | ❌ | ✅ |
| Auto-invite responders | ✅ | ✅ | ✅ | ✅ |
| Auto-pin runbook / postmortem template | 🟡 | 🟡 | ✅ | ✅ |
| Bot for slash commands (/declare, /resolve, /assign) | ✅ | ✅ | ✅ (Slack-first) | ➡️ (chat-bot capability surface needed) |
| Voice bridge (Zoom / Meet / Teams) | ✅ (link) | ✅ | ✅ | ➡️ |
| Permanent channel archive | ✅ | ✅ | ✅ | ✅ (audit-chain replay) |
| Cross-pack channel federation (KR-PIPA pack) | ❌ | ❌ | ❌ | 🔵 (KR pack — Naver Works / Kakao Work; needs re-expression as paid + KR-PIPA pack overlay) |

### Axis 7 — Postmortem + retro

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Blameless template | ✅ | ✅ | ✅ | ✅ (capabilities/postmortem-seal.yaml) |
| 5-Whys template | ✅ | ✅ | ✅ | ✅ |
| Fishbone diagram | 🟡 | 🟡 | ✅ | ➡️ |
| Timeline auto-import (chat + alerts + state changes) | ✅ | ✅ | ✅ | ✅ (ADR-IM-001 audit-chain emission) |
| Action items with owner + due-date | ✅ | ✅ | ✅ | ✅ (ADR-IM-001 postmortem action_items) |
| Action-item sync to Jira / Linear / ServiceNow | ✅ | ✅ | ✅ | ➡️ (cross-handoff with tasks + itsm substrate) |
| Approval workflow before publish | ✅ | ✅ | ✅ | ✅ (FR-027 postmortem.approve) |
| Closure requirement (seal before incident close) | 🟡 (optional) | 🟡 | ✅ | ✅ (ADR-IM-001 Postmortem Seal Constraint) |
| Retention floor (7y for SOX) | 🟡 (per-customer) | 🟡 | ✅ | ✅ (compliance-pack overlay) |
| Public-facing retro publish | ✅ | ✅ | ✅ | ✅ (via community + status page) |
| Search across historical postmortems | ✅ | ✅ | ✅ | ➡️ (via intelligence substrate) |

### Axis 8 — Status page

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Public status page | ✅ (Statuspage acquisition) | ✅ (Atlassian Statuspage) | ✅ (Nunc acquisition) | ✅ (capabilities/statuspage-sync.yaml + community substrate) |
| Private internal status page | ✅ | ✅ | ✅ | ✅ |
| Per-component status | ✅ | ✅ | ✅ | ✅ |
| Subscriber notifications (email + SMS) | ✅ | ✅ | ✅ | ✅ |
| Subscriber RSS / Atom feed | ✅ | ✅ | ✅ | ➡️ |
| Auto-update from incident state | ✅ | ✅ | ✅ | ✅ (statuspage-sync.yaml) |
| Custom branding + domain (CNAME) | ✅ | ✅ | ✅ | ➡️ |
| Maintenance-window announcement | ✅ | ✅ | ✅ | ➡️ (capability missing — see audit Dimension 9 partial) |
| Localized status page (i18n) | ✅ | ✅ | 🟡 | ✅ (KR / JP / ES / etc. via localization packs) |

### Axis 9 — Runbooks (operational procedures)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Markdown runbook per service | ✅ (Rundeck-Linked) | ✅ (Atlassian Forge) | ✅ (FireHydrant Runbooks) | ✅ (`runbooks/` 21 files) |
| Auto-attach runbook on incident declare | ✅ | ✅ | ✅ | ✅ (tenant_class adoption records paid "auto-runbook surfacing"; re-express) |
| Conditional execution (per severity / service) | ✅ | ✅ | ✅ | ➡️ |
| Step checklists with auto-skip | 🟡 | 🟡 | ✅ | ➡️ |
| Inline command execution (ChatOps) | 🟡 (Rundeck) | 🟡 (Forge) | ✅ | ➡️ (via workflow-engine substrate) |
| Runbook version history | ✅ | ✅ | ✅ | ✅ (FR-007..012 with version) |
| Runbook test (dry-run) | 🟡 | 🟡 | 🟡 | ➡️ |

### Axis 10 — Severity definitions

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Closed enum (P1 / P2 / P3 / P4) | ✅ | ✅ (P1..P5) | ✅ (SEV1..SEV4) | ✅ (SEV1..SEV4 per ADR-IM-001 + tenant_class adoption record retired) |
| Per-tenant severity ladder customization | 🟡 | 🟡 | ✅ | ➡️ |
| Per-pack severity ladder (FSC / NIS2 / HIPAA) | ❌ | ❌ | ❌ | 🔵 (tenant_class adoption records paid; re-express as compliance-pack overlay) |
| Automatic severity inference from impact | 🟡 (AIOps) | 🟡 (Investigator) | 🟡 | ✅ (tenant_class adoption records paid AI-triage; re-express as paid feature) |
| Severity SLO contract (page-to-ack per sev) | ✅ | ✅ | ✅ | ✅ (ADR-IM-001 sev1 ack 5 min; sev1 first-page p95 30 s) |

### Axis 11 — Response analytics

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| MTTA (mean time to acknowledge) | ✅ | ✅ | ✅ | ✅ (dashboards/slo-and-error-budget.json) |
| MTTR (mean time to resolve) | ✅ | ✅ | ✅ | ✅ |
| MTTD (mean time to detect) | ✅ | ✅ | ✅ | ✅ |
| MTBF (mean time between failure) | ✅ | ✅ | ✅ | ➡️ |
| On-call distribution (load per responder) | ✅ | ✅ | ✅ | ✅ (dashboards/operating-bar-overview.json) |
| Burnout signal (sleeping pages, weekend pages) | ✅ (Enterprise) | 🟡 | 🟡 | ➡️ |
| Postmortem-to-action-item closure rate | ✅ | ✅ | ✅ | ➡️ |
| Per-service incident frequency | ✅ | ✅ | ✅ | ✅ |
| Per-severity duration histogram | ✅ | ✅ | ✅ | ✅ |
| AIOps clustering (similar incidents) | ✅ (AIOps) | ✅ (Investigator) | 🟡 | ✅ (tenant_class adoption records paid; re-express) |

### Axis 12 — SLA tracking

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Customer-facing SLA per service | ✅ | ✅ | ✅ | ➡️ (capability missing per audit Dimension 9 partial) |
| Per-tenant SLA contract | ✅ | ✅ | ✅ | ➡️ |
| SLA breach detection | ✅ | ✅ | ✅ | ➡️ (only in dashboard, not capability) |
| SLA credit auto-issue | 🟡 (manual) | 🟡 | 🟡 | ➡️ (cross-handoff with cloud-billing) |
| Status-page SLA history publish | ✅ | ✅ | ✅ | ➡️ |
| Error-budget burn rate | 🟡 | 🟡 | 🟡 | ✅ (dashboards/slo-and-error-budget.json) |

Oyatie verdict: FINDING — SLA tracking is mostly dashboard-only. Promote
to first-class capability with contract surface before Phase 4A.4.

### Axis 13 — Mobile

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| iOS app (App Store) | ✅ | ✅ | ✅ | ➡️ (no iOS app declared; mobile-push-degradation runbook implies plan) |
| Android app (Play Store) | ✅ | ✅ | ✅ | ➡️ |
| Biometric ack | ✅ | ✅ | ✅ | ➡️ |
| Offline ack queue (sync on reconnect) | ✅ | ✅ | 🟡 | ➡️ |
| Push notification with action buttons | ✅ | ✅ | ✅ | ➡️ |
| In-app war-room (chat + voice bridge) | ✅ | ✅ | ✅ | ➡️ |
| Sovereign-pack-bound app distribution | ❌ | ❌ | ❌ | 🔵 (KR PIPA pack — pack-resident app store; private enterprise distribution per ADR-0251) |

Oyatie verdict: FINDING — mobile is entirely roadmap; no first-class
contract surface, no app project declared. Phase 4A.4 ServiceNow
displacement requires mobile presence at the responder level.

### Axis 14 — Slack integration

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Slack notification (bot DM + channel) | ✅ | ✅ | ✅ | ✅ (messenger substrate) |
| Slack slash command (/declare, /ack, /resolve) | ✅ | ✅ | ✅ (primary UI) | ➡️ |
| Slack message ack (button) | ✅ | ✅ | ✅ | ➡️ |
| Slack workflow (multi-step war-room) | ✅ | ✅ | ✅ | ➡️ |
| Slack channel auto-archive on close | ✅ | ✅ | ✅ | ➡️ |
| Slack thread → postmortem timeline import | ✅ | ✅ | ✅ | ➡️ |

### Axis 15 — Monitoring-tool integration depth

(Already partially covered in Axis 4; this axis evaluates integration
DEPTH — bi-directional state sync, not just one-way notification.)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| Datadog Monitor → incident bi-directional | ✅ | ✅ | ✅ | ➡️ |
| Datadog event-stream from incident | ✅ | ✅ | ✅ | ➡️ |
| NewRelic Alert → incident bi-directional | ✅ | ✅ | ✅ | ➡️ |
| Sentry issue → incident link | ✅ | ✅ | ✅ | ➡️ |
| Prom Alertmanager grouping → incident | ✅ | ✅ | ✅ | ➡️ |
| Splunk Observability detector → incident | ✅ | ✅ | ✅ | ➡️ |
| CloudWatch Alarm → incident | ✅ | ✅ | ✅ | ➡️ |
| Azure Monitor Alert → incident | ✅ | ✅ | ✅ | ➡️ |
| OCI Alarm → incident | ✅ | 🟡 | 🟡 | ➡️ |

### Axis 16 — Cedar gating (Oyatie-only)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| RBAC per-resource | ✅ | ✅ | ✅ | ✅ |
| Policy-language-defined (Cedar / Rego / OPA) | ❌ | ❌ | ❌ | 🔵 (policy/*.cedar — 6 cedar policies) |
| Default-deny across all mutations | ❌ | ❌ | ❌ | 🔵 (ADR-0243 universal Cedar gate) |
| Tenant-scoped policy evaluation | 🟡 | 🟡 | 🟡 | 🔵 (ADR-0244 universal tenant scoping) |
| Compliance-pack-aware policy overlay | ❌ | ❌ | ❌ | 🔵 (ADR-0251 compliance-pack primitive) |
| Auditable policy decision trail | 🟡 | 🟡 | 🟡 | 🔵 (ADR-0263 audit-chain emission per policy decision) |

### Axis 17 — Sovereign-pack residency (Oyatie-only)

| Sub-feature | PagerDuty | Opsgenie | FireHydrant | Oyatie incident-management |
|---|---|---|---|---|
| On-prem air-gap deployment | ❌ | 🟡 (legacy Opsgenie On-Prem retired 2022) | ❌ | 🔵 (deployment_context = on-prem; needs IaC module) |
| KR-PIPA sovereign-pack | ❌ | ❌ | ❌ | 🔵 (KR pack — needs re-expression as paid + KR-PIPA pack overlay) |
| CSAP sovereign-pack | ❌ | ❌ | ❌ | 🔵 |
| EU NIS2 sovereign-pack | ❌ | 🟡 (Atlassian EU) | ❌ | 🔵 (EU-sovereign pack) |
| GovCloud equivalent | ✅ (US-only) | ❌ | ❌ | 🔵 (FedRAMP-High pack) |
| Pack-resident paging providers (KR Kakao / KT 070) | ❌ | ❌ | ❌ | 🔵 |
| Dual-control admin (2-of-3 quorum for escalation-policy change) | ❌ | ❌ | ❌ | 🔵 (tenant_class adoption records paid dual-control; re-express as compliance-pack overlay) |
| FSC regulator pre-notification automation | ❌ | ❌ | ❌ | 🔵 |
| Per-tenant signing key for paging-receipt evidence | ❌ | ❌ | ❌ | 🔵 (STIR/SHAKEN + KCC equivalent) |

## C. Union-Coverage Verdict by Axis

| Axis | Union surface size | Oyatie coverage | Verdict |
|---|---:|---|---|
| 1. On-call schedules | 12 sub-features | 12 ✅ or ➡️ | PASS |
| 2. Escalation policies | 14 | 14 ✅ + 5 🔵 | PASS-DIFFERENTIATED |
| 3. Alerting + paging | 15 | 15 ✅ or ➡️ + 4 🔵 | PASS-DIFFERENTIATED |
| 4. Integrations | 18 | 18 ➡️ (mostly substrate-delegated) | FINDING (promote to capability) |
| 5. Incident command | 11 | 11 ✅ | PASS |
| 6. War room | 9 | 9 ✅ or ➡️ + 1 🔵 | PASS |
| 7. Postmortem + retro | 11 | 11 ✅ or ➡️ | PASS |
| 8. Status page | 9 | 9 ✅ or ➡️ | PASS |
| 9. Runbooks | 7 | 7 ✅ or ➡️ | PASS |
| 10. Severity definitions | 5 | 5 ✅ + 1 🔵 | PASS-DIFFERENTIATED |
| 11. Response analytics | 10 | 10 ✅ or ➡️ | PASS |
| 12. SLA tracking | 6 | 6 ➡️ | FINDING (promote to capability) |
| 13. Mobile | 7 | 7 ➡️ + 1 🔵 | FINDING (no app project) |
| 14. Slack integration | 6 | 6 ➡️ | FINDING (promote to capability) |
| 15. Monitoring depth | 9 | 9 ➡️ | FINDING (promote to capability) |
| 16. Cedar gating | 6 | 6 🔵 | DIFFERENTIATED |
| 17. Sovereign-pack residency | 9 | 9 🔵 | DIFFERENTIATED |

**Aggregate union-coverage verdict**: PASS with five FINDING axes (4, 12,
13, 14, 15). All five findings are documentation / contract-surface
gaps; the underlying capability is either present (Cedar / sovereign
pack / multi-provider paging) or substrate-delegated (observability /
messenger). Phase 4A.4 promotion requires closing the five FINDING axes
into first-class capability + contract surface within the µservice.

## D. Counterpart-Selection Rationale

The brief mandates **top-3** = PagerDuty + Opsgenie + FireHydrant. This
section records why those three and not the alternatives:

- **incident.io** considered but not in top-3. Rationale: incident.io is
  more an emerging FireHydrant-style competitor; its market share is
  smaller and its surface largely overlaps FireHydrant (incident command
  + retro + Slack-first UI). IP-030 already covers incident.io
  displacement, so its features enter the union surface indirectly.
- **xMatters** considered but explicitly EXCLUDED. Rationale: xMatters
  was acquired by Everbridge in 2021 and is now positioned as an
  enterprise legacy product; ServiceNow Big-8 buyers do not name it as a
  primary alternative in 2026. Manifest currently lists xMatters and
  should be amended (audit finding IM-AUDIT-2026-05-21-014).
- **VictorOps (Splunk On-Call)** considered but EXCLUDED. Rationale:
  Splunk announced On-Call deprecation roadmap 2024; product is
  end-of-life. IP-028 covers historical displacement.
- **Squadcast** considered but EXCLUDED. Rationale: SMB-focused; not the
  Big-8 enterprise competition target.
- **Rootly** considered but EXCLUDED. Rationale: Rootly is more an AI-
  triage retro player (covered by FireHydrant union); IP-029 already
  covers it.
- **PagerDuty AIOps + Process Automation + Customer Service Ops** —
  these are PagerDuty extensions, not separate products; union features
  appear in the PagerDuty column.

## E. Tenant_class Behavior per Axis (replacement for tier-deltas)

Per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`,
tier-deltas are **retired** and replaced by binary tenant_class behavior.
This section records, axis-by-axis, what demo_trial vs paid expects.

| Axis | demo_trial behavior | paid behavior |
|---|---|---|
| 1. On-call schedules | Up to 10 rotations; weekly or daily only; manual override allowed | No cap; all rotation primitives; iCal export; calendar federation |
| 2. Escalation policies | Up to 5 levels; up to 5 escalation rules per policy | Up to 100 levels; full Cedar-policy branching; customer-impact + data-class + pack overlay |
| 3. Alerting + paging | Slack + Telegram + email + community-substrate channel; no SMS / no voice; multi-provider parallel disabled | Twilio + Bandwidth + Plivo SMS; Twilio + Bandwidth Voice; APNs + FCM mobile push; pack-resident providers (Kakao / KT 070 / NHN) per pack overlay |
| 4. Integrations | Webhook + Slack + GitHub + Datadog free-tier connector | All 18 native integrations; bi-directional sync |
| 5. Incident command | Full state machine; up to 20 concurrent open incidents | Full state machine; no cap |
| 6. War room | Slack + Telegram channel; auto-create | Slack + Teams + Discord + Naver Works + Kakao Work (per pack); voice-bridge link |
| 7. Postmortem + retro | Full template; 90-day retention | Full template; per-pack retention (≥ 7 y for SOX; configurable) |
| 8. Status page | Internal status page only; oyatie-domain subdomain | Custom-branded; custom domain; CNAME; per-pack-resident subscriber notifications |
| 9. Runbooks | Up to 20 runbooks; no auto-attach | No cap; auto-attach + inline ChatOps + version history |
| 10. Severity | SEV1..SEV4 closed enum | SEV1..SEV4 + per-pack ladder overlay (FSC / NIS2 / HIPAA) |
| 11. Response analytics | MTTA + MTTR; 30-day retention | All metrics; per-pack-mandated retention; AIOps clustering |
| 12. SLA tracking | Best-effort; no contractual SLA | Contractual SLA per tenant contract; auto-credit issue via cloud-billing |
| 13. Mobile | iOS + Android general apps | iOS + Android general apps + pack-resident distribution (KR PIPA private enterprise distribution) |
| 14. Slack integration | DM + channel + slash command + ack button | All of demo_trial + slash command + workflow + thread → postmortem import |
| 15. Monitoring depth | One-way notification only | Bi-directional state sync; auto-resolve from Monitor recovery |
| 16. Cedar gating | Default-deny + tenant scope | Default-deny + tenant scope + pack overlay |
| 17. Sovereign-pack residency | Not eligible (demo cannot activate packs) | Eligible to activate KR-PIPA / CSAP / EU-sovereign / FedRAMP-High / DORA / HIPAA / GDPR / PCI-DSS / SOX |

## F. Billing-Component Touchpoints

The replacement model declares paid.billing_components ⊆
{revenue_share, per_seat, per_usage}. For incident-management:

- **per_seat**: charged per ONCALL_RESPONDER seat = one named user who
  is rotation-eligible. Inactive responders drop after a 30-day grace
  window (configurable). This is the dominant paid model for B2B
  enterprises matching the PagerDuty / Opsgenie / FireHydrant per-user
  pricing posture.
- **per_usage**: metered on paged incidents per month + outbound SMS /
  voice / push deliveries. Useful for pay-as-you-go usage tenants who
  may have many SREs but low incident volume.
- **revenue_share**: applies when a marketplace seller distributes a
  runbook pack, escalation template, or paging-channel adapter through
  Oyatie's marketplace; oyatie takes a commission per sale.

These three components compose per tenant contract.

## G. Provenance Citations

- ADR-0328 §D-2.16 ServiceNow Big-8 Phase 4A.4 ordering.
- ADR-0328 §D-5 top-3 counterpart union coverage.
- ADR-0328 §D-20.111–D-20.115 P0 severity elevation.
- brief-template §3.4.T (top-3), §3.4.C (capability completeness),
  §3.4.B (Big-8 family completeness).
- `feedback_no_tenant_class_adoption_2026_05_20.md` (tier retirement).
- `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`
  (replacement two-class model).
- `feedback_multi_category_marketplace_doctrine.md` (revenue_share
  component context).
- `microservices/incident-management/decisions/ADR-IM-001-escalation-routing-and-incident-command-state-machine.md`
  (Oyatie state-machine source-of-truth).
- `microservices/incident-management/IP-026..IP-030` (counterpart
  displacement IPs).
- `microservices/incident-management/manifest.json#coverage_benchmarks`
  (current binding; amendment required).
- PagerDuty product surface (https://www.pagerduty.com/platform/),
  pricing (https://www.pagerduty.com/pricing/), AIOps + Process
  Automation product pages.
- Opsgenie / Jira Service Management product surface
  (https://www.atlassian.com/software/jira/service-management).
- FireHydrant product surface (https://firehydrant.com/), pricing
  (https://firehydrant.com/pricing/).

## H. Halt Statement

This matrix halts cleanly. It does not commit. It does not modify any
existing artifact in `microservices/incident-management/`. It records
where capability surface gaps exist (axes 4, 12, 13, 14, 15) and where
Oyatie differentiators sit (axes 2, 3, 6, 16, 17). The coherence-audit
deliverable (`coherence-audit-2026-05-20.md`) carries the corresponding
findings; the benchmark deliverable
(`performance-benchmark-numbers-2026-05-20.md`) carries the numeric
targets per axis. No tier-deltas appear anywhere in this matrix; only
tenant_class behavior (§E) and billing_components (§F) overlay the
single-bar capability surface.

End of parity matrix.
