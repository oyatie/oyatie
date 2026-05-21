---
doc_class: User-Stories-Compendium
doc_id: USC-B2B-WORK-SURFACES
title: B2B Work Surfaces — User Stories Compendium
status: Draft
date: 2026-05-20
owners:
  - council-product
  - council-design-system
  - axis-messenger
  - axis-mail
  - axis-calendar
  - axis-meet
  - axis-drive
  - axis-docs
  - axis-sheets
  - axis-slides
  - axis-tasks
  - axis-forms
  - axis-community
  - axis-workflow
  - axis-ecosystem
  - axis-tenancy
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0131-per-microservice-flat-layout
  - ADR-0132-no-suite-forward-policy
  - ADR-0135-connect-unbundle
  - ADR-0238-connect-dual-context
  - ADR-0218-tenant-granular-control-surface
related_prds:
  - microservices/mail/PRD.md
  - microservices/messenger/PRD.md
  - microservices/community/PRD.md
  - microservices/workflow-studio/PRD.md
  - microservices/calendar/PRD.md
  - microservices/meet/PRD.md
  - microservices/drive/PRD.md
  - microservices/docs/PRD.md
  - microservices/sheets/PRD.md
  - microservices/slides/PRD.md
  - microservices/tasks/PRD.md
  - microservices/forms/PRD.md
  - microservices/plugin-app-store/PRD.md
intern_buildable_bar: true
---

# B2B Work Surfaces — User Stories Compendium

## 1. Purpose

This compendium is the canonical user-stories catalog for every B2B work surface in
oyatie. Its purpose is fourfold:

1. **Intent translation.** Translate the PRDs of the work µservices (mail,
   messenger, community, workflow-studio, calendar, meet, drive, docs, sheets,
   slides, tasks, forms, plugin-app-store, HR/payroll/compensation as composed
   surfaces) into concrete user stories that a junior engineer or product
   intern can read, internalise, and translate into UI flows + acceptance
   tests in a single working week.
2. **Acceptance-criteria seed corpus.** Every story is written so that an
   acceptance criterion (testable assertion) can be derived from it without
   asking the author for clarification. Stories carry priority (Must / Should
   / Could), surface (which µservice owns the entry point), persona, intent,
   and outcome.
3. **Cross-surface coherence anchor.** Many of the most valuable user
   experiences in a B2B platform cross µservice boundaries (a new-hire
   onboarding flow touches HR + mail + messenger + drive + calendar + tasks +
   community in one workflow). This document is the canonical narrative form
   of those flows.
4. **UX strive/avoid corpus.** Every section ends with explicit "strive for"
   and "avoid" guidance so the design system + the engineering team share a
   single vocabulary for quality decisions.

**Intern-buildable bar.** Any story in this document MUST be buildable end-to-
end by a competent intern within one sprint, assuming the underlying µservice
substrate is present per its PRD. If a story cannot be built within that bar,
it is decomposed into smaller stories until it can be. Stories that depend on
features outside the M03 milestone are explicitly flagged with `[m04+]`. The
intern-buildable bar is enforced by:

- One persona per story (no compound personas).
- One outcome per story (no compound outcomes).
- One surface or one well-defined cross-surface bridge per story (no
  "everything everywhere").
- Story body fits in ten lines. Acceptance criteria (where present) fit in
  three lines.
- Every story names the µservice(s) that own the data + the µservice that
  owns the entry point.

**Doctrine anchor.** Per ADR-0242 (oyatie-is-a-tenant), every story assumes
uniform tenant treatment: every action is gated by Cedar (ADR-0243), every
state-changing action emits to audit-chain, every personal-context surface is
isolated from professional-context per ADR-0238. There are no "internal"
versus "external" carve-outs; `oyatie.*` principals exercise the same flows
that `tenant-acme-corp.*` principals do.

---

## 2. Personas

The compendium is anchored to twelve named personas. Each persona carries a
tenant context (the tenant they belong to or assume into), an incumbent-tool
profile (what they use today before adopting oyatie), and a one-line
motivational sketch. Personas are reused across surface sections; the persona
roster is closed (no ad-hoc personas).

### 2.1 Anna — Mid-size Tech Manager

- **Role:** Engineering manager at `tenant-acme-corp`, a 600-person B2B SaaS
  company headquartered in San Francisco with offices in Berlin and Seoul.
  Anna manages a team of 12 engineers across two pods.
- **Incumbent stack today:** Slack (primary chat), Gmail (mail), Google
  Calendar (scheduling), Notion (docs + knowledge base), Linear (tasks),
  Lattice (performance review), Carta (cap table).
- **Motivation:** Anna's calendar is overloaded; her Slack DMs blow past 200
  unread daily; she switches tools 18+ times per hour according to her own
  hand-recorded RescueTime data. She wants a single shell where her
  decision-making context (last messages with a report, their open tasks,
  their recent calendar load) is at her fingertips when she opens a 1:1.
- **Power level:** Power user. Comfortable with keyboard shortcuts; will
  customise filters + saved views; expects Cmd+K everywhere.
- **Sensitive data she touches:** HR review notes, comp data for her reports,
  customer-confidential roadmap material.

### 2.2 Brian — Individual Contributor Engineer

- **Role:** Senior backend engineer at `tenant-acme-corp`, on Anna's team.
  Tenured 3 years; previously at a FAANG. Owns the billing subsystem.
- **Incumbent stack today:** Slack, Gmail, Linear, GitHub, VSCode, iTerm,
  Google Docs (RFC drafts), Notion (team wiki). Maintains a personal
  Obsidian vault for engineering notes.
- **Motivation:** Brian hates context-switching; his single most-felt pain is
  being yanked out of a coding flow by a Slack ping that turns out to be
  non-urgent. He wants: aggressive DND that respects his calendar focus
  blocks, intelligent batching of non-urgent messages, fast keyboard nav
  between PR comments / messenger threads / Linear issues without leaving
  the keyboard.
- **Power level:** Extreme power user. Would happily write Lua scripts to
  customise his workflows.
- **Sensitive data he touches:** Customer-confidential billing data, PII in
  staging snapshots, internal-only architectural decisions.

### 2.3 Catalina — HR Director

- **Role:** Head of People at `tenant-acme-corp`, manages a 4-person HR team
  serving the 600-person org. Reports to the COO.
- **Incumbent stack today:** Workday (HCM), Greenhouse (recruiting), Lattice
  (perf), Carta (equity), Justworks (payroll for US headcount), DocuSign
  (offer letters), Slack, Gmail, Google Drive. Korean payroll lives on a
  KR-specific 더존 system; German payroll lives on DATEV via her PEO.
- **Motivation:** Catalina spends 40% of her week stitching together data
  across 8 vendors. She wants a unified employee record: every employee's
  comp, perf, vacation, training, equity, payroll, mailbox, drive, and
  community-presence visible (with appropriate Cedar gates) in one surface.
- **Power level:** Domain expert; comfortable with formula-driven sheets +
  HRIS-style form builders; less comfortable with arbitrary scripting.
- **Sensitive data she touches:** Effectively all PII in the company,
  including SSNs, bank accounts, medical leave records, performance
  improvement plans, terminations.

### 2.4 Dan — Finance Manager

- **Role:** Senior finance manager at `tenant-acme-corp`, owns the FP&A
  function. Reports to the CFO. Approves all expenses above $500; reviews
  monthly close.
- **Incumbent stack today:** NetSuite (ERP), Ramp (corporate cards +
  expenses), Mercury (banking), Carta (equity), Stripe (revenue), Anaplan
  (planning), Excel (everything else).
- **Motivation:** Dan's biggest pain is approval chasing. Expense reports
  bounce between Ramp, Slack, email, and NetSuite, and he loses track. He
  wants approvals to come to him with full context (who, what, why, prior
  spend, budget remaining) and to act with one keystroke. He also wants
  monthly close to take 3 days instead of 8.
- **Power level:** Spreadsheet power user; not a developer, but will hand-
  write SQL for one-off reports.
- **Sensitive data he touches:** Company-confidential financials, vendor
  contracts, salary data (in aggregate).

### 2.5 Emma — Sales Rep (Account Executive)

- **Role:** Mid-market AE at `tenant-acme-corp`, owns the $50k-$500k ARR
  segment. Quarterly quota $1.2M. Reports to the VP Sales.
- **Incumbent stack today:** Salesforce (CRM), Outreach (sequences), Gong
  (call recording), LinkedIn Sales Navigator (prospecting), Slack, Gmail,
  Google Calendar, Calendly (scheduling), DocuSign (contracts), Zoom
  (meetings).
- **Motivation:** Emma's pain is data entry overhead. After every customer
  call she spends 20 minutes typing up notes into Salesforce. She wants
  meeting recordings to auto-transcribe and auto-populate the CRM with
  structured next-step actions. She also hates that customer-facing
  scheduling lives in Calendly (separate tool) while her internal calendar
  is Google Calendar.
- **Power level:** Heavy template user; not a developer; will build small
  workflows in a no-code tool if guided.
- **Sensitive data she touches:** Customer-confidential pricing, prospect
  contact data (PII), pipeline forecasts.

### 2.6 Faisal — Marketing Manager

- **Role:** Head of demand-gen at `tenant-acme-corp`, manages a team of 4
  marketers + 2 contractors. Reports to the CMO.
- **Incumbent stack today:** HubSpot (marketing automation), Marketo
  (campaigns), Salesforce (lead handoff), Slack, Gmail, Asana (campaign
  tracking), Adobe Creative Cloud, Figma, Notion (campaign briefs),
  WordPress (blog), Twitter/X + LinkedIn + Reddit (community).
- **Motivation:** Faisal runs a 4-channel campaign (email, social, blog,
  community) every two weeks. The cross-channel coordination is brutal —
  copy lives in Notion, design in Figma, email in HubSpot, social in
  Sprout, community in his Slack-Connect with customers. He wants one
  workspace where a campaign is one artifact with channels as
  publication targets.
- **Power level:** Moderate; can write HTML email; not a coder; relies on
  no-code automation tools.
- **Sensitive data he touches:** Marketing-qualified lead PII (large volume),
  campaign-spend data, customer references (NDA).

### 2.7 Gabriela — Executive (COO)

- **Role:** Chief Operating Officer at `tenant-acme-corp`. Direct reports
  include the CFO, CMO, CRO, Head of People (Catalina), Head of Legal, Head
  of IT. Reports to the CEO.
- **Incumbent stack today:** Outlook (mail), Outlook Calendar, Microsoft
  Teams (chat), an executive-assistant-mediated workflow for most things.
  Reads dashboards in Looker + Mode. Approves contracts in DocuSign.
- **Motivation:** Gabriela's days are 80% meetings + 20% reading dashboards
  + email triage. She delegates 60% of approvals to her chief of staff.
  She wants: a high-fidelity weekly dashboard, ability to delegate
  approvals with explicit scope + audit trail, voice-driven dictation for
  email replies, predictable meeting cadence with focus blocks the EA
  cannot violate without explicit override.
- **Power level:** Low-touch; wants the system to do as much as possible
  for her with explicit confirmation.
- **Sensitive data she touches:** All board-level material, all M&A
  conversations, all executive compensation, all litigation correspondence.

### 2.8 Hiroshi — IT Admin

- **Role:** Senior IT administrator at `tenant-acme-corp`'s Tokyo office,
  serving the 80-person APAC headcount. Reports to the global Head of IT.
- **Incumbent stack today:** Okta (SSO), Jamf (Mac management), Intune
  (Windows management), Crowdstrike (EDR), Datadog (observability), Slack
  Enterprise Grid, Microsoft 365 admin center, AWS IAM, KR-specific HRIS
  for KR headcount, JP-specific payroll vendor for JP headcount.
- **Motivation:** Hiroshi is the policy + provisioning + offboarding
  enforcement point for APAC. His pain: every tool has a separate admin
  console with a separate policy schema. Setting "no external sharing of
  files containing PII" requires configuring it in Google Drive, Box, and
  Microsoft 365 separately. He wants: one tenant admin console with one
  policy schema that propagates to every surface.
- **Power level:** Senior IT operator; comfortable with SCIM, SAML, OIDC,
  YAML configs; will write Terraform when needed.
- **Sensitive data he touches:** Tenant-wide configuration, SSO identity
  records, EDR telemetry, audit logs.

### 2.9 Inez — Security Officer (KR-FSS Regulated)

- **Role:** Chief Information Security Officer at `tenant-kr-fintech-co`, a
  Seoul-based financial-services tenant under KR-FSS (금융감독원) supervision.
  Reports to the CEO; has a dotted line to the Board's risk committee.
- **Incumbent stack today:** Splunk (SIEM), Crowdstrike (EDR), Tessian (DLP
  email), Forcepoint (DLP web), AWS Security Hub, GuardDuty, Macie. KR-
  specific tools: 안랩 (AhnLab) endpoint, 더존 (Douzone) ERP audit module.
- **Motivation:** Inez's pain is audit evidence. KR-FSS does annual on-site
  inspections; she has to produce evidence packets for: who accessed which
  customer record on which date, who had which permissions on what date,
  what was sent to whom externally with which DLP verdict. She wants every
  surface (mail, messenger, drive, docs) to emit per-action evidence in a
  uniform schema, with Ed25519-sealed audit records and a 5-year retention
  floor.
- **Power level:** Domain expert; uses regex + SIEM query language fluently;
  not a coder but reads code.
- **Sensitive data she touches:** Customer PII (financial), regulator
  correspondence, incident-response material, M&A due-diligence.

### 2.10 Jin — DevOps Lead

- **Role:** Staff platform engineer at `tenant-acme-corp`, leads a 4-person
  platform team that owns CI/CD, observability, and Kubernetes for the
  product engineering org. Reports to the VP Eng.
- **Incumbent stack today:** GitHub (source + CI), ArgoCD (deployment),
  Datadog (observability), PagerDuty (incident response), Slack (alerts),
  Terraform (IaC), Helm + Kustomize (k8s manifests), Backstage (developer
  portal). Writes Go and Rust; deeply familiar with the Kubernetes object
  model and operator pattern.
- **Motivation:** Jin wants oyatie to integrate with his existing CI/CD
  rather than replace it. Specifically: GitHub Actions posting to a
  messenger channel; PagerDuty incidents auto-creating a war-room channel;
  Datadog dashboards embedding in docs; Backstage TechDocs rendering from
  the docs µservice. He also wants Workflow Studio to be the glue for
  cross-tool incident response automation.
- **Power level:** Extreme power user; will read source; will write
  oyatie-plugin code if the SDK is good.
- **Sensitive data he touches:** Production credentials (via OpenBao),
  customer-data-touching CI logs, SLO dashboards.

### 2.11 Kara — Customer Success Manager

- **Role:** Senior CSM at `tenant-acme-corp`, manages a portfolio of 30
  strategic customers each with $200k-$2M ARR. Reports to the VP CS.
- **Incumbent stack today:** Gainsight (CSM platform), Salesforce (CRM),
  Slack + Slack-Connect (customer communication), Zoom (QBRs), Notion
  (customer playbooks), HubSpot (community + support tickets), Zendesk
  (support).
- **Motivation:** Kara's pain is customer signal aggregation. A customer's
  health spans: support ticket volume, product usage, last QBR sentiment,
  email response latency, NPS, contract milestones. Today she opens 5 tabs
  to triage one account. She wants a per-customer "command center" surface
  that aggregates everything and surfaces churn risk + expansion signal.
- **Power level:** Moderate; spreadsheet user; relies on templated playbooks.
- **Sensitive data she touches:** Customer contract terms, customer NPS +
  CSAT, customer-confidential strategic plans (shared under NDA).

### 2.12 Leo — Consultant Across Multiple Tenants

- **Role:** Independent management consultant; partner at `tenant-leo-
  partners-llp`, a small consulting firm of 6 partners. Engagements
  typically last 8-16 weeks across 3-4 client tenants at a time.
- **Incumbent stack today:** Each client has their own Slack/Teams/Gmail
  Workspace; Leo maintains 4-5 simultaneous logins; juggles 3 phones
  (work, personal, dedicated client device). Uses 1Password to remember
  credentials. Bills hours via Harvest.
- **Motivation:** Leo wants a partner-tenant model: log in once to `tenant-
  leo-partners-llp` and assume role into `tenant-acme-corp` for the duration
  of an engagement, with explicit Cedar-scoped permissions, audit-chain
  attribution showing his actions on behalf of the client, and clean role-
  off at engagement end. He hates the "one inbox per client" pattern.
- **Power level:** Moderate; PowerPoint + Excel power user; not a coder.
- **Sensitive data he touches:** Client-confidential strategy material under
  NDA, sometimes financials, sometimes M&A diligence.

---

## 3. Per-Surface User Stories

Each surface section is structured as: (a) brief surface scope recap; (b)
numbered user stories with persona / story / outcome / surface / priority /
acceptance criterion; (c) cross-references to PRDs and ADRs.

### 3.1 Surface 1: Messenger (work mode)

The messenger µservice (per microservices/messenger/PRD.md) is the team-
channels-plus-DM-plus-threads-plus-presence work surface. In work mode it
provides Slack/Teams-class channels with channel-level RBAC, dual-context
isolation from personal DMs, eDiscovery hold support, mention resolution
against the Ontology, and native Workflow event emission.

#### MSG-001 — Anna creates a campaign channel

- **Persona:** Anna (Manager).
- **Story:** Anna opens messenger, hits `Cmd+K`, types "new channel",
  presses Enter. A modal appears with name, purpose, topic, members,
  privacy (public / private / shared-with-external), retention policy
  override, and emoji icon. Anna names it `#campaign-q3-launch`, sets
  purpose to "Cross-functional coordination for the Q3 product launch",
  selects 8 members from a typeahead that resolves against the Ontology
  `Person` type, marks it private, accepts the default 90-day retention,
  hits Create.
- **Outcome:** Channel exists, members notified via in-app + mail per their
  preference, channel URL is shareable, `ChannelCreated` event sealed in
  audit-chain.
- **Surface:** messenger (`channel-store` BC).
- **Priority:** Must.
- **Acceptance:** Channel-create roundtrip ≤300ms p99 (matches PRD §
  Performance). All 8 invited members appear in the member list within
  500ms.

#### MSG-002 — Brian responds to a manager mention in-thread

- **Persona:** Brian (Engineer).
- **Story:** Brian gets an `@brian` mention from Anna in `#campaign-q3-launch`.
  The mention shows up in his notification tray with channel context, the
  parent message excerpt, and Anna's avatar. He clicks the notification,
  the messenger surface scrolls to the thread, his cursor is in the reply
  composer. He types "I can pick that up tomorrow morning, I'm heads-down
  on the billing migration today" + sends. Anna sees Brian's reply within
  her own thread view in ≤100ms.
- **Outcome:** Reply posted, mention resolution audit-emitted, no full-
  channel re-render (only the thread re-renders).
- **Surface:** messenger (`message-stream` + `thread-tree` + `mention-router` BCs).
- **Priority:** Must.
- **Acceptance:** Reply-send latency ≤100ms p99 within region.

#### MSG-003 — Catalina sends a company-wide announcement

- **Persona:** Catalina (HR Director).
- **Story:** Catalina opens `#all-hands`, a tenant-default broadcast
  channel where only certain principals (Cedar-gated as `announcement-
  publishers`) can post. She drafts a 4-paragraph announcement about the
  upcoming open enrollment window, uses the rich-text formatting toolbar,
  inserts a link to the benefits explainer doc (which embeds inline as a
  rich preview card), and adds a `@here` mention. She hits send; the
  message fans out to 600 members; read receipts come back over 24h.
- **Outcome:** Broadcast delivered, no inadvertent thread reply storm
  (channel is configured `replies-in-thread-only`), Catalina sees a
  delivered + read counter.
- **Surface:** messenger (`message-stream` + `channel-store` BCs); Cedar
  policy `announcement-publishers` gates the post.
- **Priority:** Must.
- **Acceptance:** Fan-out to 600 members completes ≤2s p99 (allows for
  WebSocket gateway sharding).

#### MSG-004 — Dan approves an expense via emoji react on a rich card

- **Persona:** Dan (Finance Manager).
- **Story:** A workflow-engine-emitted rich card lands in `#finance-
  approvals`. The card shows: requester name (Brian), amount ($1,230 for
  conference travel), category (Travel — Conference), prior YTD spend by
  Brian ($4,400), remaining annual T&E budget for Brian's team. The card
  has three reaction-actions: `✅ approve`, `❌ deny`, `❓ need info`.
  Dan reacts with `✅`. The reaction emits a `WorkflowSignalEmitted` event
  consumed by the workflow-engine, which marks the expense approved in
  the source-of-truth store, emails the requester, and updates the card
  in-place to "Approved by Dan at 2026-05-20 14:32".
- **Outcome:** Approval recorded, audit-chain sealed, no separate UI visit
  required.
- **Surface:** messenger (`message-stream` for reactions) ↔ workflow-engine
  (signal handler) ↔ finops-portal (expense source-of-truth).
- **Priority:** Must.
- **Acceptance:** Card update visible within 500ms; reaction-as-signal
  emits with full Cedar evaluation; non-approver reactions are recorded
  but ignored as signals.

#### MSG-005 — Emma uses a slash command to query CRM

- **Persona:** Emma (Sales Rep).
- **Story:** In a DM to her sales manager, Emma types `/lead acme-co` and
  Tab. The slash-command autocompletes against the CRM plugin (installed
  via plugin-app-store). She submits; the plugin queries Salesforce (via
  the tenant's CRM connector), returns a card showing: account name, ARR,
  stage, last touch, open opportunities, and three quick-action buttons
  (open in Salesforce, schedule call, log activity). The card is visible
  only in this DM (per-message scope) and is not persisted in the channel
  history of any other channel.
- **Outcome:** Emma got the info she needed without leaving messenger; the
  plugin action is audit-trailed under her principal.
- **Surface:** messenger (slash command surface) ↔ plugin-app-store (the
  CRM plugin) ↔ external CRM via tenant connector.
- **Priority:** Must.
- **Acceptance:** Slash-command resolution + card render ≤2s p99 (network-
  bounded by external CRM); audit-chain entry includes the principal + the
  declared capability the plugin used.

#### MSG-006 — Faisal hosts an AMA in #ask-marketing

- **Persona:** Faisal (Marketing Manager).
- **Story:** Faisal schedules an AMA: opens `#ask-marketing`, hits the AMA
  affordance in the channel header, sets start + end time + topic ("Q3
  campaign retrospective"). At start, the channel enters AMA mode: every
  message becomes a top-level thread, replies are scoped to thread,
  upvotes show on questions, Faisal can mark questions answered, and a
  "Live Q&A" indicator shows in the channel tile. At end, the AMA
  transcript is auto-archived to a community KB article in the
  `community` µservice with attribution preserved.
- **Outcome:** AMA structured as Q&A, durable artifact written, no chaotic
  scrollback.
- **Surface:** messenger (`channel-store` + `thread-tree`) → community (`kb-
  article-store`).
- **Priority:** Should.
- **Acceptance:** AMA mode toggle ≤300ms; archive write to community
  completes ≤30s after end-time.

#### MSG-007 — Gabriela delegates approvals to her chief of staff

- **Persona:** Gabriela (COO).
- **Story:** Gabriela opens her tenant-admin self-service settings, finds
  "Delegate approvals", selects her chief of staff (a `Person` resolved
  from Ontology), specifies the delegation scope ("approvals where amount
  ≤ $25,000 and category in [travel, vendor renewal, marketing]"), sets
  duration (90 days), confirms. The Cedar policy `approval-delegation` is
  amended with a new fragment scoped to chief-of-staff's principal. Going
  forward, qualifying approval cards route to chief of staff; Gabriela
  sees them in a "Delegated" filter on her own approvals queue.
- **Outcome:** Approvals routed to delegate; Gabriela retains override
  authority; audit-chain shows every delegated approval as `acted on
  behalf of Gabriela by chief-of-staff`.
- **Surface:** tenant admin console (delegation UI) → policy-engine (Cedar
  fragment write) → messenger (approval card routing).
- **Priority:** Must.
- **Acceptance:** Delegation takes effect within 60 seconds of save;
  audit-chain entries for delegated approvals carry both principals.

#### MSG-008 — Hiroshi sets per-team retention policy

- **Persona:** Hiroshi (IT Admin).
- **Story:** Hiroshi opens the messenger admin surface, navigates to
  "Retention", selects the team scope `team:apac-engineering`, sets
  retention to 365 days for messages and 90 days for file attachments,
  with overrides allowed only for legal-hold-engaged channels. He
  saves; the policy is sealed in audit-chain; existing messages older
  than 365 days enter the soft-delete sweep queue (subject to legal
  hold check).
- **Outcome:** Retention enforced per-team; KR PIPA compliance for the
  APAC team's stricter floor (5-year for KR-FSS-equivalent records)
  remains via override.
- **Surface:** messenger (`channel-store` retention policy) + tenancy
  (per-team scope resolution) + retention-policy worker.
- **Priority:** Must.
- **Acceptance:** Retention policy change emits `TenantRetentionPolicyUpdated`
  event; sweep worker honors hold-before-purge invariant.

#### MSG-009 — Inez audits DLP-flagged messages

- **Persona:** Inez (Security Officer, KR-FSS).
- **Story:** Inez opens the compliance console (tenant admin surface),
  selects "DLP flag review", filters to "outbound external-recipient
  messages, last 30 days, severity ≥ medium". She sees 14 flagged
  messages with: principal, recipient, DLP rule that fired, message
  excerpt (redacted PII), action taken (allowed / quarantined / blocked).
  She clicks on one (a financial-data-pattern match), reads the redacted
  excerpt, decides it's a true positive, opens an investigation case
  that creates a private incident channel + a tasks µservice entry
  assigned to her deputy, and triggers a workflow-engine flow to notify
  Legal.
- **Outcome:** Audit complete; incident response engaged; full chain-of-
  custody preserved.
- **Surface:** compliance console → messenger (DLP-flag store) →
  tasks (incident task) → workflow-engine (Legal notification).
- **Priority:** Must.
- **Acceptance:** DLP-flag list query ≤500ms p99 for 30-day window;
  four-eyes approval required to unredact PII in the excerpt (per ADR-
  0215 inheritance).

#### MSG-010 — Jin acknowledges a CI failure via messenger

- **Persona:** Jin (DevOps Lead).
- **Story:** GitHub Actions posts a CI failure to `#ci-alerts` via the
  oyatie messenger webhook (issued by his Workflow Studio flow that
  bridges GitHub webhook → messenger card). The card shows: repo, branch,
  commit, failed job, log tail (30 lines, syntax-highlighted), and three
  buttons: `Acknowledge`, `Re-run`, `Open in GitHub`. Jin clicks
  `Acknowledge`; the card updates to "Acknowledged by Jin"; a Workflow
  Studio signal fires that opens a tasks entry for him to triage.
- **Outcome:** Alert acknowledged from one surface; downstream triage
  workflow auto-spawns.
- **Surface:** messenger ← GitHub webhook → workflow-engine signal →
  tasks.
- **Priority:** Must.
- **Acceptance:** Card update + signal emission ≤500ms p99; idempotent
  on double-click.

#### MSG-011 — Kara escalates a customer issue

- **Persona:** Kara (CSM).
- **Story:** Kara is in a Slack-Connect-equivalent shared channel with one
  of her strategic customers, who reports a P0 outage. She right-clicks
  the customer's message, selects "Escalate to #urgent-customer-success",
  fills in: severity (P0), customer impact (production billing pipeline),
  proposed responder (auto-suggested based on on-call rotation in the
  workflow-engine). A new private incident channel is created with the
  customer message + her annotation pinned, the on-call engineer is paged
  via the PagerDuty plugin, a tasks entry is created with the customer's
  CRM record linked.
- **Outcome:** Single right-click → full incident response engaged;
  audit-chain shows the escalation as one structured event.
- **Surface:** messenger (escalation flow) → workflow-engine (on-call
  rotation lookup) → plugin-app-store (PagerDuty plugin) → tasks.
- **Priority:** Must.
- **Acceptance:** Escalation flow completes ≤3s p99 including external
  PagerDuty page; incident channel members visible immediately.

#### MSG-012 — Leo joins acme-corp via partner-tenant assume-role

- **Persona:** Leo (Consultant).
- **Story:** Leo logs into `tenant-leo-partners-llp`, sees in his sidebar
  a "Client engagements" section listing his three active client tenants.
  He clicks `tenant-acme-corp`; a Cedar-evaluated assume-role flow
  prompts him to confirm the scope ("read + write to channels under
  `engagement-2026-q3-leo-acme`; no access to other channels"). He
  confirms; his shell context switches; the URL bar shows `acme-corp.
  oyatie.app` with a partner-tenant banner. Every action he takes in
  the next 4 hours is audit-emitted under `tenant-acme-corp` with
  `acted_via_assume_role: tenant-leo-partners-llp` attribution.
- **Outcome:** Single-login experience across multiple client tenants;
  full audit attribution; clean role-off at session end.
- **Surface:** tenancy (partner-tenant relationship) + identity (assume-
  role + STS-equivalent token issuance) + Cedar (scope-bound permits).
- **Priority:** Must.
- **Acceptance:** Assume-role completes ≤1s; banner persists in UI;
  audit-chain entries carry both principal identities; session expires
  per Cedar-declared TTL (default 8h, max 24h).

#### MSG-013 — Anna sets focus mode during deep-work block

- **Persona:** Anna (Manager).
- **Story:** Anna has a 90-minute focus block on her calendar
  ("Architecture review prep"). Messenger detects the calendar block via
  the calendar integration and auto-enters DND for those 90 minutes.
  Non-mention messages are batched; @mentions still ring through unless
  the sender is in a configured "always-quiet" list. At the end of the
  block, a digest card surfaces with what she missed grouped by channel +
  by people, with one-click affordances to mark-read or follow-up.
- **Outcome:** Anna kept focus; she didn't lose context.
- **Surface:** messenger (DND state) + calendar (event-as-signal) +
  notification-fanout (digest generator).
- **Priority:** Should.
- **Acceptance:** DND state changes ≤2s after calendar block start; digest
  arrives within 60s of block end.

#### MSG-014 — Brian posts a code snippet with syntax highlighting

- **Persona:** Brian (Engineer).
- **Story:** Brian pastes a 40-line Rust snippet into `#backend-engineering`.
  Messenger detects code (heuristic + leading triple-backtick), prompts
  to confirm language (Rust auto-detected from indentation + keywords),
  renders with syntax highlighting + line numbers + a copy button +
  inline GitHub-style code-folding for blocks >20 lines. Brian's
  colleague reacts with a 👀 (looking) emoji, then replies in-thread.
- **Outcome:** Code is readable in messenger; no need to switch to a
  gist or paste site.
- **Surface:** messenger (`message-stream` rich rendering).
- **Priority:** Should.
- **Acceptance:** Highlight rendering ≤100ms client-side; copy-button
  works on click without network.

#### MSG-015 — Emma uses /scheduled to set a follow-up reminder

- **Persona:** Emma (Sales Rep).
- **Story:** After a discovery call, Emma types in a DM to herself
  `/remindme followup with marko at acme in 3 days "send pricing deck"`.
  Three days later, at 9am Emma's local timezone, the reminder pops as a
  card with the original context + a one-click "snooze 1 day" + "mark
  done" + "open in CRM" affordance.
- **Outcome:** Reminder fires reliably; original context preserved.
- **Surface:** messenger (slash-command + scheduled-message worker).
- **Priority:** Should.
- **Acceptance:** Reminder fires within ±60 seconds of scheduled time;
  survives messenger pod restarts (durable scheduling).

#### MSG-016 — Catalina pins onboarding resources to #new-hires

- **Persona:** Catalina (HR Director).
- **Story:** Catalina creates a `#new-hires` channel; pins 4 resources
  (the onboarding doc in docs µservice, the benefits-explainer slide deck
  in slides µservice, the welcome video in drive, the org-chart in the
  community KB). Pinned items render as rich preview cards in a
  collapsible "Pinned" tray at the top of the channel. New hires
  joining the channel see the pinned tray expanded by default for their
  first 14 days.
- **Outcome:** Onboarding resources are durably accessible; not buried in
  scrollback.
- **Surface:** messenger (`channel-store` pins) + cross-µservice rich-link
  resolvers.
- **Priority:** Should.
- **Acceptance:** Pin tray renders ≤100ms; rich preview cards refresh on
  source-doc change within 60s.

#### MSG-017 — Faisal hosts a poll in #marketing

- **Persona:** Faisal (Marketing Manager).
- **Story:** Faisal types `/poll "Which campaign theme resonates?" "Time
  Saved" "Money Saved" "Trust Earned"` in `#marketing`. A poll card
  appears; team members vote; results show as live-updating bars; Faisal
  closes voting after 24h. The poll archives as an inline card with
  final counts + per-person attribution (if poll is non-anonymous).
- **Outcome:** Quick decision-gathering without external tool.
- **Surface:** messenger (built-in poll BC; if not native, via plugin).
- **Priority:** Could.
- **Acceptance:** Poll card supports ≥1000 voters; vote latency ≤200ms.

#### MSG-018 — Hiroshi audits a former employee's channel memberships

- **Persona:** Hiroshi (IT Admin).
- **Story:** A former employee's offboarding workflow triggers Hiroshi's
  review queue. He opens the audit surface for the principal, sees: every
  channel the principal was a member of (with timestamps of join/leave),
  every message they posted (count + last-30-days excerpts), every DM
  they participated in (count, not content unless legal-hold engaged).
  Hiroshi confirms offboarding complete; the principal is removed from
  remaining memberships; an evidence packet is generated for retention.
- **Outcome:** Offboarding compliance evidence produced; no orphaned
  memberships.
- **Surface:** tenant admin console + messenger (`channel-store` audit
  view) + audit-chain.
- **Priority:** Must.
- **Acceptance:** Audit view query ≤1s for principals with ≤10,000
  channel events; evidence packet generation ≤30s.

#### MSG-019 — Brian uses `/me` for an away status

- **Persona:** Brian (Engineer).
- **Story:** Brian types `/me at the doctor; back at 3pm` in his
  team's main channel. Messenger renders an italic third-person status
  message ("Brian is at the doctor; back at 3pm") and sets his presence
  to "Away — at the doctor" with the 3pm auto-clear. The status syncs
  to calendar (so meeting invites in that window auto-decline with the
  status as the decline reason) and to mail (so mail rules can auto-
  reply with the status).
- **Outcome:** One affordance updates three surfaces.
- **Surface:** messenger (presence) + calendar (availability projection) +
  mail (out-of-office sync).
- **Priority:** Should.
- **Acceptance:** Cross-surface sync ≤5s; auto-clear at declared time.

#### MSG-020 — Kara coordinates with Faisal on a customer reference

- **Persona:** Kara (CSM) and Faisal (Marketing Manager).
- **Story:** Kara DMs Faisal: "Acme is willing to be a public reference;
  can we line up a case study?". Faisal replies in DM; both convert the
  DM to a new shared `#case-study-acme` channel with one click. The
  conversion preserves message history (visible only to original
  participants by default) and adds the relevant teams (sales, marketing,
  legal) as members.
- **Outcome:** DM becomes a project hub; no information loss; no
  re-typing.
- **Surface:** messenger (DM-to-channel promotion flow).
- **Priority:** Could.
- **Acceptance:** Promotion completes ≤2s; original participants see
  history; new members do not see pre-promotion history unless
  explicitly shared.

### 3.2 Surface 2: Mail (work mode)

The mail µservice (per microservices/mail/PRD.md) is a standards-compatible
mail surface with SMTP/IMAP/JMAP/REST, dual-context isolation, eDiscovery
hold, per-tenant retention, and mail-to-workflow handoff.

#### MAIL-001 — Brian creates a VIP-routing mail rule

- **Persona:** Brian (Engineer).
- **Story:** Brian opens mail rules, clicks New. He sets: "If From matches
  any of `vip@enterprise-customer.com`, `cto@bigco.com`, or anyone in
  Ontology group `vip-customers`, THEN: flag as urgent, route to my
  attention folder, post to `#engineering-urgent` messenger channel via
  card, and notify my manager Anna via DM with a summary link". The rule
  evaluates server-side (no client trust) and is sealed in audit-chain
  on save.
- **Outcome:** Brian never misses a VIP mail; downstream surfaces are
  notified.
- **Surface:** mail (rule engine) + messenger (card emission) + ontology
  (group resolution).
- **Priority:** Must.
- **Acceptance:** Rule evaluation runs in ≤50ms; cross-surface fan-out
  ≤2s; audit-chain entry on every rule firing.

#### MAIL-002 — Catalina manages shared support@ inbox

- **Persona:** Catalina (HR Director).
- **Story:** Catalina's HR team operates a shared `hr-support@acme-corp`
  mailbox. She configures it as a "team mailbox": members of the
  `hr-team` Ontology group can read all messages, claim threads, mark
  resolved, and reply on behalf-of the shared address. Conversations
  show claim status (claimed by Catalina; in-progress) and last-update
  time. SLA timer surfaces a yellow indicator at 8h, red at 24h.
- **Outcome:** No mail falls through cracks; SLAs are visible.
- **Surface:** mail (`mailbox-store` shared-inbox semantics).
- **Priority:** Must.
- **Acceptance:** Claim action ≤300ms; SLA timer accurate to ±1 minute;
  per-thread audit-chain entries on claim, reassign, resolve.

#### MAIL-003 — Dan triggers an expense workflow from a receipt email

- **Persona:** Dan (Finance Manager).
- **Story:** A vendor sends an invoice as a PDF attachment to Dan's
  mailbox. Dan clicks "Process as Expense" in the mail toolbar (a
  workflow handoff action). A modal opens with auto-extracted fields
  (vendor name, amount, due date, line items — extracted via the
  intelligence µservice with Dan's explicit consent). Dan reviews +
  confirms; the workflow-engine routes the expense per the routing
  rules (above $500 to Anna for approval; over $5000 to the CFO).
- **Outcome:** Mail-to-workflow handoff is one click; consent is
  explicit; extraction errors are correctable.
- **Surface:** mail (`mailbox-store` handoff button) + workflow-engine
  (routing) + intelligence µservice (PDF parsing under consent).
- **Priority:** Must.
- **Acceptance:** Handoff modal renders ≤500ms; extraction completes ≤5s
  for typical invoice; full audit trail.

#### MAIL-004 — Emma sends a DLP-checked mass email to leads

- **Persona:** Emma (Sales Rep).
- **Story:** Emma drafts a mass email to 200 leads from a campaign list.
  At send-time, the DLP scanner checks: (a) recipient cardinality (200 >
  the per-message warn-threshold 100), (b) attachment content (none in
  this case), (c) body content (matches DLP rule "external bulk
  promotional content" — requires marketing-compliance review). The
  send is held in a "pending review" state; Faisal (marketing-compliance
  reviewer) gets a notification; he approves; the send proceeds.
- **Outcome:** No accidental mass-sends without review; no data leakage.
- **Surface:** mail (`outbound-smtp` + DLP) + workflow-engine (review
  routing).
- **Priority:** Must.
- **Acceptance:** Hold notification ≤2s after send-click; reviewer
  approval triggers release ≤2s; audit-chain on hold + approve + send.

#### MAIL-005 — Faisal schedules a campaign with A/B testing

- **Persona:** Faisal (Marketing Manager).
- **Story:** Faisal composes a campaign email; clicks "A/B test"; provides
  variant B (different subject line); sets split (50/50), audience (a
  segment of 10,000 marketing-qualified-leads from the CRM connector),
  and send-time (next Tuesday 10am in recipient's local timezone). The
  campaign plugin (installed via plugin-app-store) handles segment
  resolution, scheduling, throttling per tenant SMTP reputation budget,
  open + click tracking (with explicit consent footer in the email per
  GDPR), and outcome attribution.
- **Outcome:** Campaign sent; A/B results show after 48h.
- **Surface:** mail + plugin-app-store (campaign plugin) + intelligence
  (subject-line variant generation if used).
- **Priority:** Should.
- **Acceptance:** Send throttling honors per-tenant IP reputation budget;
  audit-chain on each send batch; tracking complies with declared
  consent.

#### MAIL-006 — Gabriela gets a daily digest from direct reports

- **Persona:** Gabriela (COO).
- **Story:** Gabriela has configured a daily 8am digest of unread mail
  from her direct reports + flagged-VIPs. The digest arrives as a single
  email + a messenger card; it lists, per sender, the top 3 unread
  threads with one-sentence auto-summaries, sentiment-tag (positive /
  neutral / concerning), and a "mark all read" + "open" affordance. She
  scans in 3 minutes instead of 30.
- **Outcome:** Executive triage takes a fraction of the time.
- **Surface:** mail (digest worker) + intelligence (summarization +
  sentiment, under tenant policy).
- **Priority:** Should.
- **Acceptance:** Digest assembly ≤30s; sentiment labelling tier-T1 per
  EU AI Act risk tiering; user can opt-out of sentiment.

#### MAIL-007 — Hiroshi enforces enterprise SSO + 2FA

- **Persona:** Hiroshi (IT Admin).
- **Story:** Hiroshi opens tenant admin → identity → mail access policy.
  He sets: "All mail access requires SSO via the tenant's primary OIDC
  IdP; 2FA via WebAuthn passkey or TOTP is mandatory; legacy IMAP basic-
  auth refused; app-passwords allowed only for principals with
  `legacy-imap-required` Cedar entitlement (and audited on every use)".
- **Outcome:** Mail access is gated by enterprise-grade identity; legacy
  paths are accounted for + minimised.
- **Surface:** tenant admin + identity µservice + mail (IMAP/JMAP/REST
  auth-frontend) + Cedar.
- **Priority:** Must.
- **Acceptance:** Policy takes effect within 5 minutes; refused legacy
  auth attempts emit audit-chain entries; legacy app-passwords expire
  after 90d.

#### MAIL-008 — Inez audits external-recipient mail per FSS

- **Persona:** Inez (Security Officer, KR-FSS).
- **Story:** Inez opens compliance console → "External mail audit",
  filters: "Outbound mail to non-acme-domain recipients, containing PII-
  class attachments or DLP-flagged keywords, last quarter". She sees a
  list with sender, recipient domain, DLP verdict, attachment count.
  She exports the list as a sealed evidence packet (Ed25519-signed,
  chain-of-custody bundle per FR-06 of mail PRD).
- **Outcome:** Quarterly FSS audit evidence in 15 minutes instead of 2
  weeks of manual SIEM queries.
- **Surface:** mail (audit + eDiscovery export) + compliance console.
- **Priority:** Must.
- **Acceptance:** Filter result returns ≤5s; evidence export completes
  ≤24h per PRD; bundle verifies third-party.

#### MAIL-009 — Jin sets up a bot account for CI notifications

- **Persona:** Jin (DevOps Lead).
- **Story:** Jin creates a service-principal `ci-bot@acme-corp` under
  the `oyatie.dev.acme.ci` sub-scope. He grants it `mail:send` capability
  scoped to `to:#ci-alerts@acme-corp` (a mail-list alias for the
  channel). He configures his GitHub Actions to send mail via the
  service-principal's SMTP credentials (stored in OpenBao). CI failures
  arrive as mail; mail rules route them to the CI channel via card.
- **Outcome:** Service-principal model for automation; no human creds in
  CI; auditable.
- **Surface:** identity (service-principal) + mail (`outbound-smtp` with
  SP auth) + cloud-secrets (OpenBao).
- **Priority:** Must.
- **Acceptance:** SP creation + grant ≤30s; SP credential rotation 90d;
  audit-chain on every SP mail send.

#### MAIL-010 — Kara uses canned responses for common asks

- **Persona:** Kara (CSM).
- **Story:** Kara has 8 canned response templates ("Pricing inquiry",
  "Outage status", "Renewal kickoff", etc.). When composing a reply, she
  types `/canned` and a typeahead surfaces her templates. She picks one,
  it inserts with merge-fields (customer name auto-resolved from the
  thread's CRM link). She edits as needed, sends.
- **Outcome:** CSM productivity up; consistency improves.
- **Surface:** mail (composer + templates).
- **Priority:** Should.
- **Acceptance:** Canned-response insertion ≤200ms; merge-field
  resolution uses Ontology lookup; templates personal-scope by default,
  shareable team-scope opt-in.

#### MAIL-011 — Leo accesses client mailbox via assumed role

- **Persona:** Leo (Consultant).
- **Story:** Leo's engagement scope at `tenant-acme-corp` includes
  read-access to a project-specific shared mailbox `consulting-engagement-
  2026-q3@acme-corp`. After he assumes-role into acme-corp, he opens
  mail; only the engagement mailbox is visible (no other acme mailboxes).
  Every read or write is audit-chained under his identity with
  `acted_via_assume_role: tenant-leo-partners-llp` attribution.
- **Outcome:** Cross-tenant collaboration with full attribution.
- **Surface:** mail (`mailbox-store` ACL via Cedar) + identity (assume-
  role).
- **Priority:** Must.
- **Acceptance:** Mailbox-list query honors Cedar scope; audit-chain
  attribution complete; role-off revokes access within 60s.

#### MAIL-012 — Anna manages 3 mail aliases

- **Persona:** Anna (Manager).
- **Story:** Anna has 3 mail aliases: `anna@acme-corp` (personal),
  `eng-management@acme-corp` (role alias), `q3-launch@acme-corp` (project
  alias for the campaign she manages). All three deliver to her unified
  inbox; she can send-as any of them; replies preserve the alias used in
  the inbound thread.
- **Outcome:** Multi-context mail without a separate account per context.
- **Surface:** mail (`mailbox-store` aliases).
- **Priority:** Should.
- **Acceptance:** Alias add ≤500ms; send-as resolution server-side;
  DKIM signature per alias.

#### MAIL-013 — Catalina sets vacation auto-reply

- **Persona:** Catalina (HR Director).
- **Story:** Catalina opens settings → vacation. Sets dates (next Mon-
  Fri), auto-reply text ("I'm out of office; for urgent HR matters
  contact my deputy Sara at sara@..."), delegate (Sara) for time-
  sensitive items. During vacation, internal senders see Sara as a
  delegate option in the suggestion bar; external senders get the
  auto-reply once per sender per week (no spam loops).
- **Outcome:** Coverage configured; loops prevented.
- **Surface:** mail (`mailbox-store` auto-reply policy).
- **Priority:** Must.
- **Acceptance:** Auto-reply rate-limit honored; delegate suggestions
  visible internally; audit-chain on activation.

#### MAIL-014 — Dan flags an incoming mail as phishing

- **Persona:** Dan (Finance Manager).
- **Story:** Dan receives an email that looks like a CEO wire-transfer
  request but is from a misspelled domain. He clicks "Report phishing"
  in the toolbar. The mail is quarantined, scored by the abuse
  classifier, the sender is added to the tenant's reputation deny-list
  (if confirmed), Inez (security officer) gets a notification, and an
  audit-chain entry seals the report.
- **Outcome:** Threat intel improves; sender list updated; security
  team in the loop.
- **Surface:** mail (`inbound-smtp` abuse) + security console.
- **Priority:** Must.
- **Acceptance:** Quarantine ≤2s after click; reputation update ≤30s;
  no false-positive on legitimate sender (reversible).

#### MAIL-015 — Faisal uses smart suggestions for subject lines

- **Persona:** Faisal (Marketing Manager).
- **Story:** Faisal is composing a customer-newsletter draft. The
  composer offers an "AI-suggest subject" affordance (tier-T2 per EU
  AI Act, with explicit click-to-invoke). Faisal clicks; 5 candidate
  subject lines appear with predicted open-rate estimates (based on
  prior campaign data). Faisal picks one, edits to taste, sends.
- **Outcome:** Faster drafting with explicit, opt-in AI assistance.
- **Surface:** mail (composer) + intelligence (LLM under tenant policy).
- **Priority:** Could.
- **Acceptance:** Suggestion render ≤2s; consent UI clearly shows AI
  invocation; opt-out persists.


### 3.3 Surface 3: Workflow Studio (work mode)

The workflow-studio µservice (per microservices/workflow-studio/PRD.md) is the
visual workflow authoring product. Studio binds to the canonical workflow_
spec.v1.json format; engine (sibling microservices/workflow-engine) owns
durable execution.

#### WFS-001 — Brian builds a PR-notification workflow

- **Persona:** Brian (Engineer).
- **Story:** Brian opens Workflow Studio, picks the GitHub-trigger node
  (`when PR opened`), connects to a message-card node (`post to
  #engineering`), configures the card body with PR title + author +
  labels (using the Studio's field-picker that resolves against the
  GitHub plugin's schema), saves. The spec emits as a canonical
  workflow_spec.v1.json; engine registers it; live runs flow.
- **Outcome:** Standard team notification flow built in <10 minutes.
- **Surface:** workflow-studio (visual-canvas + dsl-emitter) + workflow-
  engine (run) + plugin-app-store (GitHub plugin) + messenger.
- **Priority:** Must.
- **Acceptance:** Save round-trip ≤200ms p99; first live run ≤30s after
  save; spec round-trip byte-equal.

#### WFS-002 — Anna builds a new-hire onboarding workflow

- **Persona:** Anna (Manager).
- **Story:** Anna composes a multi-stage onboarding flow: trigger (HR
  `EmployeeOnboarded` event), then in parallel: (a) mail welcome from
  Catalina's mailbox, (b) add to `#team-engineering` + `#all-hands`
  channels, (c) create a personal drive folder seeded with onboarding
  docs, (d) schedule a 30-min intro 1:1 with Anna in the new hire's
  first week, (e) assign 5 first-week tasks in tasks µservice. She
  uses the Studio's parallel-branch node and previews the Cedar policy
  impact before saving.
- **Outcome:** Onboarding goes from "5 humans coordinating" to "1
  workflow + 1 human reviewer (Anna for the personal touch)".
- **Surface:** workflow-studio + HR-as-composed-surface (`Employee` Ontology
  object) + mail + messenger + drive + calendar + tasks.
- **Priority:** Must.
- **Acceptance:** Spec validates against schema; Cedar preview shows
  every capability the workflow will exercise; preview accurate to
  live execution per AC-09 of workflow-studio PRD.

#### WFS-003 — Catalina builds an offboarding workflow

- **Persona:** Catalina (HR Director).
- **Story:** Catalina composes: trigger (`EmployeeTerminationFinalized`),
  then sequentially: (a) disable SSO via identity µservice, (b) transfer
  mailbox + drive to manager via mail + drive admin APIs, (c) revoke
  channel memberships + active sessions in messenger, (d) revoke plugin
  access in plugin-app-store, (e) emit a sealed offboarding-evidence
  bundle in audit-chain, (f) notify legal if any active legal-holds
  attach to the principal. Every step requires four-eyes approval if
  the principal had `executive` role.
- **Outcome:** Offboarding hardened against drift; full evidence packet.
- **Surface:** workflow-studio + identity + mail + drive + messenger +
  plugin-app-store + audit-chain.
- **Priority:** Must.
- **Acceptance:** Full flow completes ≤5 minutes for standard cases;
  four-eyes gate for executives; evidence packet sealed.

#### WFS-004 — Dan builds an expense-routing workflow

- **Persona:** Dan (Finance Manager).
- **Story:** Dan composes a router: trigger (`ExpenseSubmitted`), then a
  decision node based on amount: ≤$500 auto-approve + audit, >$500 →
  route to submitter's manager (resolved via Ontology), >$5000 →
  route to CFO with manager pre-approval required, >$50,000 → require
  board-CFO chain. Each branch emits a messenger card to the approver
  with reaction-based one-click approve/deny (per MSG-004 pattern).
- **Outcome:** Expense routing matches policy; nothing is manual.
- **Surface:** workflow-studio + workflow-engine + finops-portal + mail
  + messenger.
- **Priority:** Must.
- **Acceptance:** Decision node evaluation ≤10ms; routing fan-out
  ≤500ms; full audit-chain.

#### WFS-005 — Emma builds a lead-scoring workflow

- **Persona:** Emma (Sales Rep).
- **Story:** Emma composes: trigger (`CRMLeadUpdated` from the CRM
  plugin), then a scoring node (uses the intelligence µservice with
  tier-T1 scoring on declared lead attributes), then a decision: if
  score > 80, in parallel: (a) schedule a follow-up call (calendar),
  (b) post a high-priority card in `#sales-pipeline`, (c) add a task
  for Emma in tasks µservice. If 50 < score ≤ 80, only add to her
  weekly nurture queue.
- **Outcome:** No high-intent lead falls through cracks.
- **Surface:** workflow-studio + plugin-app-store (CRM plugin) +
  intelligence + calendar + messenger + tasks.
- **Priority:** Should.
- **Acceptance:** Score evaluation ≤2s; downstream fan-out ≤1s; AI
  scoring transparently labeled per EU AI Act tier.

#### WFS-006 — Faisal builds a multi-channel campaign workflow

- **Persona:** Faisal (Marketing Manager).
- **Story:** Faisal composes a campaign-launch flow: trigger (manual
  start by Faisal at campaign start time), then in parallel: (a)
  publish a community post in tenant community, (b) send mail-blast
  via campaign plugin, (c) post to social channels via social plugin,
  (d) post to `#marketing-launches`. Each branch checks per-channel
  rate limits + reputation before sending.
- **Outcome:** Campaign launches across channels atomically.
- **Surface:** workflow-studio + community + mail + plugin-app-store
  (social plugin) + messenger.
- **Priority:** Should.
- **Acceptance:** All channels publish within ±5min of trigger; failures
  on one channel do not block others; per-channel audit-chain.

#### WFS-007 — Gabriela governs workflow changes

- **Persona:** Gabriela (COO).
- **Story:** Gabriela receives a weekly digest of "newly published or
  amended workflows" with capability summary + Cedar-permission diff.
  She can drill into any workflow, see its diff against prior version
  via the Studio diff viewer, and require a multi-approver gate on
  high-sensitivity workflows (those touching HR, finance, or legal-
  hold capabilities).
- **Outcome:** Governance scaled without manual review of every flow.
- **Surface:** workflow-studio (diff viewer + governance UI) + tenant
  admin console.
- **Priority:** Should.
- **Acceptance:** Diff render ≤500ms; governance gate definable per
  capability set; bypass attempts emit Sev-1 audit entry.

#### WFS-008 — Hiroshi sets per-team workflow quotas

- **Persona:** Hiroshi (IT Admin).
- **Story:** Hiroshi opens tenant admin → workflow quotas. For
  `team:apac-engineering`, he sets: max 50 active workflows, max
  10,000 runs/day, max LLM-assist tokens 1M/day, max external HTTP
  calls 100,000/day. Excess attempts emit alerts to him + the team
  lead.
- **Outcome:** Tenant resource use is bounded; no surprise bills.
- **Surface:** tenant admin + workflow-engine (quota substrate) +
  finops-portal (alerting).
- **Priority:** Must.
- **Acceptance:** Quota change ≤5s to take effect; budget alarms fire
  at 80 / 95 / 100% per Hiroshi's config.

#### WFS-009 — Inez audits workflow run history

- **Persona:** Inez (Security Officer).
- **Story:** Inez investigates a suspected data exfil event. She opens
  Studio's run-history UI, filters: workflows that touched
  `data_class=PII_IDENTIFYING` AND `egress=external`, last 30 days.
  She sees 24 runs; she inspects each via the replay-debugger, which
  shows step-by-step inputs/outputs (decrypted only after four-eyes
  approval per ADR-0215 inheritance).
- **Outcome:** Investigation evidence in 1 hour vs 2 days.
- **Surface:** workflow-studio (replay-debugger-frontend) + audit-chain.
- **Priority:** Must.
- **Acceptance:** Filter query ≤5s for 30-day window; four-eyes gate
  on decryption; audit-chain on investigator's access.

#### WFS-010 — Jin debugs a failed workflow via replay

- **Persona:** Jin (DevOps Lead).
- **Story:** A scheduled deployment workflow failed overnight. Jin opens
  the failed run in Studio's replay-debugger; sees the failed step
  (a Kubernetes apply that returned 409 conflict); steps through prior
  state; identifies the cause (concurrent deploy from another team).
  He adds a serialisation gate to the workflow spec and re-runs.
- **Outcome:** Root cause found visually; fix shipped in ≤30 minutes.
- **Surface:** workflow-studio + workflow-engine (replay-debugger-backend).
- **Priority:** Must.
- **Acceptance:** Replay loads ≤3s for ≤1000-step runs; step navigation
  ≤100ms per step.

#### WFS-011 — Kara builds a customer-onboarding workflow

- **Persona:** Kara (CSM).
- **Story:** Kara composes: trigger (`CustomerContractSigned` from CRM
  plugin), then: (a) create a customer-dedicated channel in messenger
  + community space, (b) provision tenant users from the customer
  contact list via mail invitations, (c) create a tasks project from
  her CSM playbook template, (d) schedule a kick-off call within 5
  business days, (e) send a welcome packet (drive folder with onboarding
  docs).
- **Outcome:** First-touch customer experience consistent + fast.
- **Surface:** workflow-studio + community + messenger + mail + tasks +
  calendar + drive.
- **Priority:** Should.
- **Acceptance:** End-to-end run ≤10 minutes; idempotent on retry;
  template-driven so other CSMs reuse.

#### WFS-012 — Brian uses LLM-assist to draft a workflow

- **Persona:** Brian (Engineer).
- **Story:** Brian needs a workflow that "scans all open PRs daily,
  finds ones with 0 reviewer activity for >48h, and DMs the author
  with a polite nudge". He types this prose into Studio's LLM-assist;
  a candidate spec is drafted; he reviews the visual canvas
  representation; tweaks the DM message; saves.
- **Outcome:** Prose to working workflow in 5 minutes.
- **Surface:** workflow-studio (LLM-assist-bridge) + intelligence (LLM
  invocation under tenant consent).
- **Priority:** Should (GA).
- **Acceptance:** Draft generation ≤3s p99; spec validates against
  schema; LLM-assist usage audit-trailed.

#### WFS-013 — Anna toggles a workflow's jurisdiction overlay

- **Persona:** Anna (Manager).
- **Story:** Anna's onboarding workflow (WFS-002) targets all employees.
  When a hire is in the EU, she needs the workflow to emit GDPR-compliant
  consent prompts before adding to community. She toggles the EU
  jurisdiction overlay in Studio's overlay-renderer; the canvas shows
  diff (highlighted new nodes for consent + Cedar guard); she accepts;
  spec save includes overlay metadata.
- **Outcome:** Single workflow handles multiple jurisdictions cleanly.
- **Surface:** workflow-studio (jurisdiction-overlay-renderer).
- **Priority:** Should.
- **Acceptance:** Overlay toggle ≤200ms; diff visible; spec round-trip
  byte-equal under overlay.

#### WFS-014 — Faisal collaborates live with a colleague on a workflow

- **Persona:** Faisal + a colleague.
- **Story:** Faisal opens a campaign workflow; his colleague opens the
  same flow simultaneously. Both see each other's cursors + name labels;
  edits from each merge via CRDT in <100ms; conflicting edits surface
  as a structured conflict prompt (no silent loss).
- **Outcome:** Real-time collaboration; never lose work.
- **Surface:** workflow-studio (collab-crdt + WebSocket gateway).
- **Priority:** Must.
- **Acceptance:** CRDT merge ≤100ms p99; conflict UI visible; AC-06 of
  workflow-studio PRD.

#### WFS-015 — Catalina previews Cedar policy before save

- **Persona:** Catalina (HR Director).
- **Story:** Catalina edits the offboarding workflow (WFS-003) to add a
  new "auto-revoke equity Carta access" step. Before save, Studio shows
  the Cedar policy preview: the new capability `carta:revoke` is
  required; Catalina lacks direct authority to grant it; the save will
  request escalation to the legal-tenant-admin team. She accepts.
- **Outcome:** Authorization-by-design; no post-deploy policy surprises.
- **Surface:** workflow-studio (Cedar policy preview) + policy-engine.
- **Priority:** Must.
- **Acceptance:** Preview render ≤500ms; escalation request audit-
  trailed; AC-16 of workflow-studio PRD.

### 3.4 Surface 4: HR / Payroll / Compensation (composed surface)

HR / Payroll / Compensation are not yet standalone µservices in oyatie;
per ADR-0132 (no-suite-forward-policy), they manifest as a composed
surface built on the per-microservice primitives: an `Employee` Ontology
object type owned by a future `microservices/hr/` (out of scope for M03
but planned post-bootstrap per ADR-0242 § sandbox+preview tenants), with
working surfaces built atop tasks (HR tasks), forms (onboarding intake),
drive (employee files), docs (offer letters), sheets (comp tables),
calendar (review cycles), workflow-studio (HR workflows), and a
dedicated tenant-admin HR console.

#### HR-001 — Catalina onboards a new hire end-to-end

- **Persona:** Catalina (HR Director).
- **Story:** A new hire (Mariana) signs an offer letter. Catalina opens
  the HR console, hits "Onboard new hire", fills in the intake form
  (legal name, start date, role, manager Anna, comp band, location
  Berlin). The submit triggers the onboarding workflow (WFS-002): SSO
  provisioned, mailbox created, channels assigned, drive folder seeded
  with welcome docs, 1:1 with Anna scheduled, first-week tasks
  created. Mariana receives a welcome mail with her oyatie login + a
  link to a guided community KB onboarding tour.
- **Outcome:** Onboarding takes 30 minutes of Catalina's time vs the
  previous 4 hours.
- **Surface:** HR console + workflow-studio + every downstream µservice.
- **Priority:** Must.
- **Acceptance:** Full flow completes ≤15 minutes; idempotent on
  retry; audit-chain.

#### HR-002 — Anna submits a vacation request

- **Persona:** Anna (Manager) as employee.
- **Story:** Anna opens her self-service HR portal, navigates to "Time
  Off". She selects vacation dates (June 12-19), enters reason
  (optional), confirms it does not collide with her team's blackout
  dates (which the system surfaces inline). Submits. Her manager (the
  VP Eng) gets a notification with one-click approve/deny. Anna's
  calendar auto-blocks the dates as out-of-office once approved; her
  mail auto-reply activates; her messenger DND engages.
- **Outcome:** Vacation flow is integrated; no separate tool to update.
- **Surface:** HR console + workflow-engine (approval routing) +
  calendar + mail + messenger.
- **Priority:** Must.
- **Acceptance:** Submit + approval ≤2s each; cross-surface sync ≤30s
  after approval.

#### HR-003 — Dan runs monthly payroll

- **Persona:** Dan (Finance Manager) — payroll role.
- **Story:** Dan opens the payroll console at month-end. The console
  shows: total headcount, total gross + net, tax withholdings by
  jurisdiction (US-federal, US-state, KR 원천징수, DE Lohnsteuer, etc.),
  benefit deductions, equity grants vesting this period, any
  exceptions (newly-hired mid-month, terminated mid-month, comp
  adjustments). Dan reviews exceptions, approves, hits "Execute".
  The payroll workflow runs: per-jurisdiction tax forms produced,
  bank transfers initiated, employee pay stubs emailed.
- **Outcome:** Monthly close from 8 days to 3.
- **Surface:** HR/payroll console + workflow-studio + sheets (calculation
  workbooks) + mail (pay stubs) + drive (forms archive).
- **Priority:** Must.
- **Acceptance:** Exception list visible ≤2s; execute step under four-
  eyes approval with CFO; bank transfer initiation via a per-
  jurisdiction integrations layer.

#### HR-004 — Catalina conducts a quarterly performance review

- **Persona:** Catalina (HR Director).
- **Story:** Catalina opens the perf review console at quarter-end.
  She launches the quarterly cycle: every manager gets a review form
  (built in forms µservice from her template) for each report. The
  forms collect: goal attainment scores, narrative feedback, comp
  recommendation. Managers submit; Catalina reviews calibration in
  a sheets pivot; finalizes; the workflow emits comp-adjustment events
  to payroll for next cycle.
- **Outcome:** Review cycle structured + audit-trailed.
- **Surface:** HR console + forms + sheets + workflow-engine + mail
  (notification).
- **Priority:** Must.
- **Acceptance:** Form rollout to ≥100 managers ≤2 minutes; calibration
  sheet computes ≤1s on filter; cycle close locks edits + emits
  evidence packet.

#### HR-005 — Brian views his current comp + history

- **Persona:** Brian (Engineer).
- **Story:** Brian opens his self-service portal → "My Compensation".
  He sees: current base salary, bonus target, equity grants (vested +
  unvested), benefits enrolled, last comp adjustment date, prior
  comp history. Equity is rendered with a vesting schedule chart and
  current FMV (under appropriate disclosure controls).
- **Outcome:** Transparency without HR mediation.
- **Surface:** HR console (employee self-service) + Ontology
  `Compensation` object + Cedar-gated read.
- **Priority:** Should.
- **Acceptance:** View renders ≤500ms; data sourced from HR system of
  record; audit-chain on access.

#### HR-006 — Emma sees commission calculation + disputes a line

- **Persona:** Emma (Sales Rep).
- **Story:** Emma opens "My Commissions". Sees: Q3 quota, attainment %,
  closed deals with per-deal commission lines, accelerators applied,
  total earned, paid YTD. She spots a deal where the commission rate
  looks wrong; she clicks "Dispute"; opens a form pre-populated with
  context; submits; the dispute routes to her manager + finance for
  review.
- **Outcome:** Disputes surface fast; resolution tracked.
- **Surface:** HR/comp console + forms + workflow-engine (dispute
  routing).
- **Priority:** Should.
- **Acceptance:** Disputes acknowledged ≤1 business day; resolution
  SLA tracked.

#### HR-007 — Catalina handles a termination + offboarding

- **Persona:** Catalina.
- **Story:** A manager submits a termination request via the HR
  console. Catalina reviews, ensures performance-improvement-plan
  documentation is attached (per company policy), schedules the
  termination conversation with the manager + HR business partner,
  triggers the offboarding workflow (WFS-003) timed to the last day.
- **Outcome:** Compliance-clean termination; no missed steps.
- **Surface:** HR console + workflow-studio.
- **Priority:** Must.
- **Acceptance:** Termination policy compliance check on submit;
  four-eyes approval at execute; offboarding emits full evidence
  packet.

#### HR-008 — Catalina runs open enrollment

- **Persona:** Catalina.
- **Story:** At benefits open enrollment, Catalina launches the
  campaign: every employee receives a personalised mail + community KB
  announcement, with a deadline. Employees open the enrollment form
  (forms µservice), make selections (medical, dental, 401k, etc.),
  submit. Catalina's dashboard tracks completion. Reminders auto-fire
  at 7d, 3d, 1d before deadline to non-completers.
- **Outcome:** Higher completion rates; less manual chasing.
- **Surface:** HR console + forms + mail + community + workflow-engine
  (reminder cadence).
- **Priority:** Must.
- **Acceptance:** Campaign launch to ≥600 employees ≤5 minutes;
  reminder fan-out at correct times.

#### HR-009 — Dan generates per-jurisdiction tax forms

- **Persona:** Dan (Finance / Payroll).
- **Story:** At year-end, Dan triggers tax-form generation. Per US
  employee: W-2 + 1099 (if contractor). Per KR employee: 원천징수
  영수증. Per DE employee: Lohnsteuerbescheinigung. Per UK
  employee: P60. Forms are generated per-jurisdiction-template, signed
  by the tenant's KMS-held signing key, archived in drive (immutable
  WORM tier), distributed to employees via mail with download links.
- **Outcome:** Tax compliance across 4+ jurisdictions, generated +
  delivered in a single batch.
- **Surface:** HR console + workflow-engine + drive (WORM tier) + mail.
- **Priority:** Must.
- **Acceptance:** Per-employee form generation ≤30s; immutable storage
  per SEC 17a-4 + KR commercial code; audit-chain seal per form.

#### HR-010 — Catalina handles a parental leave request

- **Persona:** Catalina.
- **Story:** An employee submits a parental leave request via the HR
  portal. Catalina reviews; verifies eligibility per the employee's
  jurisdiction (US FMLA, EU minimum directives, KR 육아휴직, etc.);
  approves with leave dates + return-to-work plan. The workflow:
  notifies payroll for benefit continuation, schedules a return-to-
  work check-in, creates handoff tasks for the employee's responsibilities.
- **Outcome:** Compliance-aware leave handling.
- **Surface:** HR console + workflow-engine + jurisdiction-pack overlay
  + tasks + calendar.
- **Priority:** Must.
- **Acceptance:** Jurisdiction-correct policy applied; audit-chain on
  eligibility check.

#### HR-011 — Gabriela views org-wide headcount + cost dashboard

- **Persona:** Gabriela (COO).
- **Story:** Gabriela opens her exec dashboard. She sees: total
  headcount by team + by location, monthly cost trend, hiring plan
  vs actual, attrition rate, time-to-hire, gender + ethnic diversity
  by org level (where lawful to collect + report), comp band
  distribution. She drills into any cell; the dashboard reveals
  underlying detail (under Cedar gating, with PII redacted at
  aggregate level).
- **Outcome:** Executive-grade workforce visibility.
- **Surface:** HR console + sheets (rollups) + intelligence (anomaly
  detection on attrition spikes).
- **Priority:** Should.
- **Acceptance:** Dashboard load ≤2s; drill-down respects Cedar; PII
  redaction enforced.

#### HR-012 — Anna requests a comp adjustment for a report

- **Persona:** Anna (Manager).
- **Story:** During the perf cycle, Anna identifies that Brian's comp
  is below the median for his band + tenure. She opens his record
  in the HR portal, drafts a comp-adjustment request (+8% base, +0.1%
  equity refresh), justifies via narrative + market data. The
  request routes via workflow-engine through her VP, the HR business
  partner, the head of comp, and the CFO. Approvals stack;
  notification flows back; comp record updates next pay period.
- **Outcome:** Compensation governance preserved while flow is fast.
- **Surface:** HR console + workflow-engine + mail (notifications).
- **Priority:** Should.
- **Acceptance:** Approval chain renders + visible ≤500ms; SLAs at
  each step; audit-chain on every approval.

### 3.5 Surface 5: Calendar + Meet (work mode)

The calendar µservice (per microservices/calendar/PRD.md) + meet µservice
(per microservices/meet/PRD.md) together provide scheduling, invitations,
room booking, federated availability, and video conferencing.

#### CAL-001 — Emma schedules a sales call with an external customer

- **Persona:** Emma (Sales Rep).
- **Story:** Emma in a customer DM proposes a call; the messenger
  surface offers a "Suggest times" affordance that opens an inline
  calendar picker showing her availability. She picks 3 slots, sends.
  The customer (on a foreign mail system) gets an iMIP-compliant
  invitation; they pick a slot via the standard reply mechanism;
  Emma's calendar locks the slot; a meet room is auto-bound; the
  customer gets a join link.
- **Outcome:** Cross-organisation scheduling without Calendly.
- **Surface:** calendar (`invitation-flow` + iMIP) + meet (room binding)
  + messenger (inline picker).
- **Priority:** Must.
- **Acceptance:** Iine picker renders ≤500ms; invitation send via iMIP;
  reply auto-locks the slot.

#### CAL-002 — Anna creates a recurring team standup

- **Persona:** Anna (Manager).
- **Story:** Anna creates "Team Standup", Mon-Wed-Fri 9:00-9:15 AM,
  attendees: her team. Sets recurrence RRULE (weekly, MWF, indefinite),
  meet room auto-bound, agenda template attached (a docs µservice
  template with sections for blockers, asks, updates). Attendees see
  it on their calendars; calendar honors timezone localisation per
  attendee.
- **Outcome:** Standard recurring meeting created in 30 seconds.
- **Surface:** calendar (`event-store` + `recurrence-engine`) + meet +
  docs (template binding).
- **Priority:** Must.
- **Acceptance:** RRULE expansion bounded; per-attendee timezone view
  correct; template auto-attaches.

#### CAL-003 — Brian books focused-work blocks

- **Persona:** Brian (Engineer).
- **Story:** Brian opens his calendar, hits "Auto-block focus time";
  picks "3-4 hours/day, mornings preferred, avoid meeting-heavy days".
  The calendar's intelligent scheduler proposes 5 focus blocks this
  week, avoiding existing meetings + lunch + team-standup, leaving
  buffer between meetings + focus blocks. Brian accepts. His messenger
  DND auto-engages during focus blocks (per MSG-013).
- **Outcome:** Deep work protected without manual blocking.
- **Surface:** calendar (focus-block scheduler) + intelligence
  (heuristics) + messenger (DND).
- **Priority:** Should.
- **Acceptance:** Proposal computed ≤1s; conflict-free against existing
  events; user override at any time.

#### CAL-004 — Catalina schedules quarterly 1:1s

- **Persona:** Catalina (HR Director).
- **Story:** Catalina runs "Schedule Q3 1:1s with my team", picks
  duration 45min each, picks the date range (the week of Sep 1), the
  scheduler finds non-conflicting slots across each report's calendar,
  proposes a schedule, Catalina reviews + adjusts, accepts. All
  invitations send in one batch.
- **Outcome:** Batch-scheduling 4 1:1s takes 2 minutes vs 30 minutes
  of manual proposing.
- **Surface:** calendar (batch scheduler + `availability-resolver`).
- **Priority:** Should.
- **Acceptance:** Proposal computed ≤3s for 4 attendees over 1 week;
  batch send ≤5s.

#### CAL-005 — Gabriela's EA manages her calendar

- **Persona:** Gabriela (COO) — delegated to her EA.
- **Story:** Gabriela's EA has been granted "scheduling delegate" via
  the tenant admin (Cedar permit `calendar:write` scoped to
  Gabriela's calendar). The EA accepts/declines invites, blocks
  focus time, schedules meetings on Gabriela's behalf. All actions
  audit-chain under the EA's principal with `acted_on_behalf_of:
  gabriela`. Gabriela sees a daily summary of changes.
- **Outcome:** Delegation works cleanly; Gabriela retains visibility.
- **Surface:** calendar (delegation) + Cedar + audit-chain.
- **Priority:** Must.
- **Acceptance:** Delegation grant ≤5s; audit attribution complete;
  daily summary at 6am EA's local time.

#### CAL-006 — Faisal hosts a 500-attendee webinar

- **Persona:** Faisal (Marketing Manager).
- **Story:** Faisal schedules a customer webinar (date, topic, speakers,
  registration form). Meet hosts in "webinar mode": registration via
  forms µservice, practice-session 30min before, Q&A moderated,
  attendee analytics live, RTMP egress to YouTube live. 487 attendees
  join; the webinar runs; transcripts auto-generate; recording
  archives to drive; analytics surface attendee engagement metrics.
- **Outcome:** Full webinar program without a separate Zoom Webinar
  vendor.
- **Surface:** meet (webinar mode) + calendar + forms + drive
  (recording) + community (post-event recap).
- **Priority:** Must.
- **Acceptance:** Webinar supports ≥1000 interactive + ≥10,000
  broadcast per meet PRD; recording archived ≤5min after end.

#### CAL-007 — Hiroshi configures org working hours + holiday calendar

- **Persona:** Hiroshi (IT Admin).
- **Story:** Hiroshi opens tenant admin → calendar policy. Sets default
  working hours per location (US-Pacific 9-17, KR-Seoul 9-18, DE-
  Berlin 9-17), holiday calendars per location (US federal + state, KR
  공휴일, DE federal + Bundesland). When users schedule across
  locations, the scheduler honors local working hours + holidays.
- **Outcome:** Cross-locale scheduling respects local norms.
- **Surface:** tenant admin + calendar (jurisdiction + holiday data).
- **Priority:** Must.
- **Acceptance:** Holiday data updates yearly; per-locale defaults
  applied to all new users.

#### CAL-008 — Inez audits meeting recordings

- **Persona:** Inez (Security Officer).
- **Story:** Inez investigates a suspected info-leak that allegedly
  occurred during a meeting. She opens compliance console → meeting
  recordings, filters by meeting host + date range + classification
  (recordings tagged `confidential-internal`). She finds 3 candidates,
  invokes the four-eyes approval to view; CTO co-approves; she views
  the transcript + recording; opens an investigation case.
- **Outcome:** Compliance investigation enabled; four-eyes preserved.
- **Surface:** meet (recording vault) + compliance console + audit-chain.
- **Priority:** Must.
- **Acceptance:** Recording-list query ≤2s; four-eyes gate; audit on
  every view.

#### CAL-009 — Jin runs sprint retro with breakout rooms

- **Persona:** Jin (DevOps Lead).
- **Story:** Jin runs a 90-min sprint retro for his 12-person team.
  Schedules meet meeting with breakout-rooms enabled. At minute 30,
  he splits into 3 breakouts (4 people each), each working on a
  retro topic, shared whiteboard per room. At minute 60, he merges
  rooms back, each breakout shares findings, action items captured
  into tasks µservice.
- **Outcome:** Structured retro; outputs immediately actionable.
- **Surface:** meet (breakout-rooms + whiteboard) + tasks (action-
  items).
- **Priority:** Should.
- **Acceptance:** Breakout split ≤5s; whiteboard persists post-meeting;
  action items emitted to tasks.

#### CAL-010 — Kara hosts customer office hours

- **Persona:** Kara (CSM).
- **Story:** Kara hosts weekly customer office hours (every Thursday 4-
  5pm). Customers across her portfolio can drop in via a public meet
  link in her customer-shared community space. She mutes attendees by
  default; lets them raise-hand to ask questions; records sessions
  optionally with consent.
- **Outcome:** Scalable customer touch; consent-respecting recording.
- **Surface:** meet (raise-hand + recording-with-consent) + community.
- **Priority:** Could.
- **Acceptance:** Public link works without login (lobby-gated);
  consent prompt before recording.

#### CAL-011 — Anna imports her existing Google Calendar

- **Persona:** Anna (Manager) at oyatie migration time.
- **Story:** Anna onboards to oyatie. She runs the calendar migration:
  authorise Google Calendar OAuth, import last 2 years + future
  recurring + shared calendars; the import preserves event IDs, RSVP
  state, attachments. Anna's prior schedule appears in oyatie calendar
  identical to Google.
- **Outcome:** Migration is one-click; no manual rebuild.
- **Surface:** calendar (`ics-import-export` + Google adapter via plugin).
- **Priority:** Must.
- **Acceptance:** 2-year import ≤2 minutes; preservation of recurrence
  + attendees + attachments.

#### CAL-012 — Brian books a meeting room

- **Persona:** Brian (Engineer).
- **Story:** Brian schedules a 1:1 with a colleague in person at the
  Berlin office. He picks the time; the room booking module proposes
  available rooms (Berlin → Floor 3 → "Hummel" room, capacity 4, AV-
  equipped); Brian picks; the room is booked + visible on the floor
  display + on his calendar.
- **Outcome:** Hybrid-office scheduling without separate room tool.
- **Surface:** calendar (`room-booking`).
- **Priority:** Must.
- **Acceptance:** Room availability query ≤200ms; booking idempotent
  on retry.

### 3.6 Surface 6: Community (work mode)

The community µservice (per microservices/community/PRD.md) is the org-wide
community surface: announcements, Q&A, KB articles, threaded discussion.

#### COM-001 — Catalina creates the company internal KB

- **Persona:** Catalina (HR Director).
- **Story:** Catalina sets up the company knowledge base in community:
  spaces for HR Policies, IT Help, Engineering Standards, Marketing
  Brand, Sales Playbooks. Per-space moderators are assigned; per-space
  posting permissions configured (announcement vs open Q&A vs KB-only).
- **Outcome:** One canonical KB instead of 5 Confluence + Notion +
  Google Sites blends.
- **Surface:** community + tenant admin.
- **Priority:** Must.
- **Acceptance:** Space creation + permission config ≤1min; per-space
  search index.

#### COM-002 — Brian asks a Q&A and accepts an answer

- **Persona:** Brian (Engineer).
- **Story:** Brian doesn't know how to use the company's deployment
  CLI. He posts in `#engineering-help` Q&A space: "How do I rotate
  the staging deploy token?". Three colleagues answer; one with a
  detailed walkthrough. Brian upvotes + marks accepted. The Q&A
  becomes searchable; next engineer with the same question finds it
  in seconds.
- **Outcome:** Knowledge compounds.
- **Surface:** community (`post-store` + `voting-engine` + `search-
  index`).
- **Priority:** Must.
- **Acceptance:** Post → answer ≤1min for tracked Q&A; search index
  reflects within 5 minutes.

#### COM-003 — Anna creates a cross-team campaign community

- **Persona:** Anna (Manager).
- **Story:** Anna creates `q3-launch` community space spanning sales,
  marketing, engineering, CS. Long-form posts, threaded discussion,
  decision logs accumulate. At launch, the space becomes a public KB
  archive of decisions, runbooks, retrospectives.
- **Outcome:** Cross-team coordination has a durable home.
- **Surface:** community.
- **Priority:** Should.
- **Acceptance:** Space supports ≥100 active contributors; archive +
  search work.

#### COM-004 — Catalina builds a guided onboarding KB tour

- **Persona:** Catalina.
- **Story:** Catalina creates an onboarding KB tour: 10 sequential
  articles ("Day 1 — Welcome", "Day 2 — Your Manager", ...). New hires
  see it pinned in their personal community feed for the first 30 days.
  Completion tracked.
- **Outcome:** Self-service onboarding; HR scales.
- **Surface:** community + HR console (new-hire pinning).
- **Priority:** Should.
- **Acceptance:** Tour completion telemetry; per-hire progress visible
  to Catalina.

#### COM-005 — Faisal hosts a customer advocacy community

- **Persona:** Faisal.
- **Story:** Faisal creates a customer-facing community (external space,
  Cedar-gated to authenticated customer users). Customers post product
  feedback, vote on feature requests, share success stories. Marketing
  highlights top contributors as case studies.
- **Outcome:** Customer community without separate vendor (Khoros /
  Hivebrite / Vanilla).
- **Surface:** community (external mode + Cedar).
- **Priority:** Should.
- **Acceptance:** External-user access gated via tenant SSO or
  external IdP; cross-tenant mention forbidden.

#### COM-006 — Inez moderates per regulator policy

- **Persona:** Inez (Security Officer, KR-FSS).
- **Story:** Inez configures content moderation per FSS rules: posts
  containing customer financial PII auto-flagged for review; posts
  containing material non-public information auto-quarantined. The
  moderation queue surfaces in her console.
- **Outcome:** Regulator-compliant community without manual sweep.
- **Surface:** community (`moderation-queue`) + DLP.
- **Priority:** Must.
- **Acceptance:** Auto-flag ≤2s after post; quarantine ≤5s; appeal
  flow defined.

#### COM-007 — Brian subscribes to a tag for notifications

- **Persona:** Brian.
- **Story:** Brian subscribes to the `#rust` tag in the engineering Q&A
  space. He gets notified (via mail digest, configurable) when new
  Q&A threads tagged `#rust` are posted.
- **Outcome:** Targeted knowledge feed.
- **Surface:** community (subscriptions) + mail (digest).
- **Priority:** Should.
- **Acceptance:** Subscription ≤300ms; digest delivers per cadence.

#### COM-008 — Anna pins decisions for her team

- **Persona:** Anna.
- **Story:** After a tech-design discussion, Anna posts the decision
  ("We're using Tokio task::spawn over async-channel for X reason")
  in the engineering KB. Pins it so it surfaces atop the team's space.
- **Outcome:** Decisions don't get lost in chat.
- **Surface:** community (pin + posts).
- **Priority:** Should.
- **Acceptance:** Pin ≤200ms; visible in space feed.

#### COM-009 — Catalina sends a company-wide policy update

- **Persona:** Catalina.
- **Story:** A new code-of-conduct rev publishes. Catalina posts in
  `#hr-policies` announcement space; the post requires every employee
  to "Acknowledge". Per-employee acknowledgment tracked; reminders to
  non-acknowledgers at 7d, 3d, 1d before deadline.
- **Outcome:** Compliance acknowledgment trackable.
- **Surface:** community (announcement + acknowledgment) + workflow-
  engine.
- **Priority:** Must.
- **Acceptance:** Per-user ack tracked + audit-chained; reminders fire.

#### COM-010 — Kara publishes a customer success playbook

- **Persona:** Kara.
- **Story:** Kara writes a long-form KB article about a customer
  retention playbook. Attaches related decks (slides), checklists
  (tasks), example mails (mail). Other CSMs reference it during their
  account work.
- **Outcome:** Tribal knowledge captured.
- **Surface:** community (KB) + cross-µservice embeds.
- **Priority:** Should.
- **Acceptance:** Embeds render inline; revision history; comment
  threads.

#### COM-011 — Jin documents a postmortem in community

- **Persona:** Jin.
- **Story:** After a production incident, Jin writes a blameless
  postmortem in community KB: timeline, root cause, what went well,
  what didn't, action items (with tasks links). Engineering reviews
  + comments + accepts.
- **Outcome:** Postmortems are durable + searchable.
- **Surface:** community + tasks + audit-chain.
- **Priority:** Must.
- **Acceptance:** Postmortem template available; action-item linkage
  bidirectional.

#### COM-012 — Faisal hosts a quarterly all-hands AMA

- **Persona:** Faisal (and the CEO).
- **Story:** Quarterly, the CEO hosts an all-hands AMA. Employees
  submit questions to community in advance (with upvotes); top 20
  questions get answered live in the meeting (per CAL-006 webinar
  pattern); answers archived back to community.
- **Outcome:** Structured executive engagement.
- **Surface:** community + meet + workflow-engine (Q&A pipeline).
- **Priority:** Could.
- **Acceptance:** Q&A submission + voting up to 1 week prior; top-K
  surface clear.


### 3.7 Surface 7: Drive + Docs + Sheets + Slides

The drive, docs, sheets, slides µservices together provide files, collaborative
documents, spreadsheets, and presentations.

#### DRV-001 — Brian creates a design doc; shares with team

- **Persona:** Brian (Engineer).
- **Story:** Brian creates a new doc in docs µservice from the
  "Engineering RFC" template. Writes a 5-page design rationale.
  Shares with `team:backend` with comment permission, with Anna with
  edit permission, with `team:platform` view-only. He inserts
  embedded sheet (for projected query rates) + slides (for the
  architecture diagram). Colleagues comment + suggest; Brian accepts
  + rejects suggestions; ships v1.
- **Outcome:** Design docs are first-class collaborative artifacts.
- **Surface:** docs + sheets (embed) + slides (embed) + drive
  (bytes-at-rest) + Cedar (ACL).
- **Priority:** Must.
- **Acceptance:** Doc creation ≤500ms; collaborative edit CRDT-merged;
  embedded sheet refresh on source change.

#### DRV-002 — Anna co-authors a campaign plan with Faisal

- **Persona:** Anna + Faisal.
- **Story:** Anna and Faisal co-author a campaign plan. Both have
  cursors visible in the doc; CRDT merges concurrent edits; in-line
  comments resolve discussion threads; revision history captures
  every state.
- **Outcome:** Real-time co-authoring without conflict.
- **Surface:** docs (`collab-crdt`) + comments.
- **Priority:** Must.
- **Acceptance:** Concurrent edits merge ≤100ms; no silent loss; AC-
  03 of docs PRD.

#### DRV-003 — Catalina manages employee files with per-employee access

- **Persona:** Catalina (HR Director).
- **Story:** Catalina's HR drive has a folder per employee containing
  PII-grade files (signed offer letters, I-9, background-check, perf
  reviews). Cedar policy: only HR team + the employee themselves can
  read their own file; managers can read review docs for their direct
  reports only. Hiroshi cannot read any unless under explicit legal
  hold + four-eyes approval.
- **Outcome:** Cedar-enforced privacy.
- **Surface:** drive (`permissions` + Cedar) + HR console.
- **Priority:** Must.
- **Acceptance:** Permission preview before share; audit-chain on
  every read of PII-class file; four-eyes for elevated access.

#### DRV-004 — Dan builds a budget spreadsheet

- **Persona:** Dan (Finance).
- **Story:** Dan opens sheets, creates "FY26 budget", with departmental
  tabs (engineering, sales, marketing, HR, ops). Uses formulas to
  roll up to a summary tab. Cross-sheet references via SHEETS()
  function. Shares with dept heads with comment-only access. Comments
  drive negotiation.
- **Outcome:** Budgeting in sheets without Excel exports.
- **Surface:** sheets (formula engine + collab) + comments.
- **Priority:** Must.
- **Acceptance:** Formula recalc ≤500ms for 10K-cell sheet; comments
  threaded inline; per-tab permission.

#### DRV-005 — Emma presents to a customer using slides

- **Persona:** Emma (Sales).
- **Story:** Emma uses a customer-pitch deck template; customises with
  customer logo + named slides; presents from slides µservice live in
  a meet call. Customer sees presenter view; Emma sees speaker notes
  + audience reactions. Post-meeting, she shares the deck via signed
  link (expiring 7 days).
- **Outcome:** Native presentation in customer-facing meeting.
- **Surface:** slides + meet + drive (share-link).
- **Priority:** Must.
- **Acceptance:** Slide transitions ≤200ms; share-link expiry honored.

#### DRV-006 — Gabriela locks down a board doc

- **Persona:** Gabriela (COO).
- **Story:** Gabriela creates a board-only doc. Permissions: read =
  board members + her, write = her. WORM immutability after the
  board meeting locks the version. Audit-chain shows every view.
- **Outcome:** Highly-sensitive doc protected.
- **Surface:** docs + drive (WORM tier) + Cedar + audit-chain.
- **Priority:** Must.
- **Acceptance:** WORM lock irreversible without four-eyes; every
  read audit-chained.

#### DRV-007 — Jin sets up cross-µservice CI artifacts in drive

- **Persona:** Jin (DevOps).
- **Story:** Jin's CI pipeline uploads build artifacts (Docker images,
  test reports, perf reports) to per-build folders in drive under a
  service-principal. Retention: 90 days for build artifacts; 1 year
  for release artifacts; immutable for compliance-mode reports.
- **Outcome:** Artifact storage governed.
- **Surface:** drive (service-principal upload) + tenant admin (retention
  policy).
- **Priority:** Must.
- **Acceptance:** SP upload throughput ≥1GB/s aggregate; retention
  honored.

#### DRV-008 — Brian uses delta-sync for laptop offline editing

- **Persona:** Brian.
- **Story:** Brian works on a flight with poor wifi. Drive desktop
  client uses FastCDC delta-sync: only changed chunks upload when
  connectivity resumes. He doesn't need to wait minutes on resume.
- **Outcome:** Offline-tolerant authoring.
- **Surface:** drive (`sync` BC).
- **Priority:** Must.
- **Acceptance:** Sync delta time ≤30s for 100 changed files per
  drive PRD §Performance.

#### DRV-009 — Faisal organises campaign assets in drive

- **Persona:** Faisal.
- **Story:** Faisal creates a campaign folder structure: `q3-launch/`
  with subfolders `assets/`, `copy/`, `videos/`, `decks/`. Sets
  team-write + extended-shareable-link permissions on the parent.
  Uploads via drag-drop, multipart resumable for large videos.
- **Outcome:** Campaign assets organised + accessible.
- **Surface:** drive (`folder` + `upload`).
- **Priority:** Must.
- **Acceptance:** Multipart upload of 5GB video ≤90s p99.

#### DRV-010 — Kara templates a customer onboarding deck

- **Persona:** Kara.
- **Story:** Kara saves her customer-onboarding slides as a template.
  Each new customer gets a copy auto-populated with customer name,
  logo, account team. She edits last-mile content; ships.
- **Outcome:** Standardised customer onboarding artifacts.
- **Surface:** slides (template + auto-populate via Ontology lookup).
- **Priority:** Should.
- **Acceptance:** Template-to-instance copy ≤2s; merge-fields render.

#### DRV-011 — Inez sets DLP on drive

- **Persona:** Inez (Security).
- **Story:** Inez configures DLP rules in drive: files containing
  resident-registration-number patterns auto-flagged; external-share
  of such files refused unless explicitly approved by her or her
  deputy.
- **Outcome:** Data egress controlled.
- **Surface:** drive (`dlp` BC) + Cedar.
- **Priority:** Must.
- **Acceptance:** DLP scan ≤2s per file at upload; share refusal
  immediate; appeal path documented.

#### DRV-012 — Leo accesses client deck via assumed role

- **Persona:** Leo (Consultant).
- **Story:** Leo's engagement scope grants read+comment on a specific
  drive folder at the client tenant. After assume-role he opens drive;
  only the engagement folder is visible. Comments are attributed to him
  with `acted_via_assume_role` audit metadata.
- **Outcome:** Cross-tenant document collaboration with attribution.
- **Surface:** drive + identity (assume-role).
- **Priority:** Must.
- **Acceptance:** Folder list honors Cedar; role-off revokes access.

### 3.8 Surface 8: Tasks + Forms

The tasks µservice (per microservices/tasks/PRD.md) is the user-facing work-
item primitive; the forms µservice (per microservices/forms/PRD.md) is
structured intake forms.

#### TSK-001 — Anna creates a project + assigns work

- **Persona:** Anna.
- **Story:** Anna creates project `q3-launch` in tasks. Defines columns
  (Todo / In Progress / Review / Done), custom fields (priority, ETA,
  dependencies). Creates 30 tasks, assigns to her team, sets due
  dates. Renders kanban board.
- **Outcome:** Project planned + visible.
- **Surface:** tasks (`project-list` + `view-engine` + custom fields).
- **Priority:** Must.
- **Acceptance:** Board render ≤200ms p99 for 30 tasks; drag-drop
  ≤50ms perceived.

#### TSK-002 — Brian focuses on his task queue

- **Persona:** Brian.
- **Story:** Brian opens tasks → "My Work". Sees his open tasks across
  all projects, sorted by due date + priority. Hits `j/k` to navigate;
  `e` to edit inline; `space` to mark done; `Cmd+/` to comment.
- **Outcome:** Keyboard-driven task management.
- **Surface:** tasks (cross-project view).
- **Priority:** Must.
- **Acceptance:** Keyboard nav ≤30ms; inline edit instant.

#### TSK-003 — Catalina builds an onboarding intake form

- **Persona:** Catalina.
- **Story:** Catalina builds a forms µservice form: "New Hire Intake",
  fields for personal info, tax docs (file upload), banking info
  (encrypted-at-field), benefits selections. Conditional logic shows
  benefits options only after location is selected. Submissions feed
  the HR onboarding workflow.
- **Outcome:** Structured intake; no email-back-and-forth.
- **Surface:** forms + HR/workflow-engine.
- **Priority:** Must.
- **Acceptance:** Conditional logic engine; field-level encryption for
  PII; submit ≤500ms.

#### TSK-004 — Dan builds an expense submission form

- **Persona:** Dan.
- **Story:** Dan builds an "Expense Submission" form: amount, vendor,
  category (dropdown), date, receipt (file upload, OCR auto-extract
  with explicit consent), narrative. On submit, the expense workflow
  (WFS-004) triggers.
- **Outcome:** Structured expense intake replacing email-receipts.
- **Surface:** forms + workflow-engine.
- **Priority:** Must.
- **Acceptance:** OCR pre-fill consent visible; submit ≤500ms.

#### TSK-005 — Emma uses forms for proposal request intake

- **Persona:** Emma.
- **Story:** Emma's prospects fill a "Request a Proposal" form on her
  account page. Fields capture company size, use case, timeline,
  budget range. Submissions route to Emma + log in CRM via plugin.
- **Outcome:** Structured prospect intake.
- **Surface:** forms (external-facing mode) + plugin-app-store (CRM).
- **Priority:** Should.
- **Acceptance:** External form access via signed link; captcha
  defense; submit ack ≤2s.

#### TSK-006 — Faisal collects survey responses

- **Persona:** Faisal.
- **Story:** Faisal launches a customer NPS survey: forms with NPS
  question + open-ended feedback. Distribute via mail-blast.
  Aggregate results visible in sheets via export. Top detractor
  responses auto-create tasks for Kara.
- **Outcome:** Survey loop closed.
- **Surface:** forms + mail + sheets + tasks.
- **Priority:** Should.
- **Acceptance:** Aggregate ≤2s; auto-task creation idempotent.

#### TSK-007 — Hiroshi runs an IT-asset audit via form

- **Persona:** Hiroshi.
- **Story:** Hiroshi sends "Annual IT-asset confirmation" form to every
  employee. Pre-populated with what IT believes the employee has;
  employees confirm or correct. Non-respondents reminded; aggregates
  feed asset-management.
- **Outcome:** IT asset reconciliation.
- **Surface:** forms + workflow-engine (reminders).
- **Priority:** Must.
- **Acceptance:** Form delivery ≥600 employees ≤5min; aggregate
  visible to Hiroshi.

#### TSK-008 — Anna sets task dependencies

- **Persona:** Anna.
- **Story:** In her project, Anna sets "Backend API ready" task as a
  blocker on "Frontend integration" + "QA". Tasks µservice prevents
  cycles via DAG enforcement. Gantt view renders the dependency chain
  with critical path highlighted.
- **Outcome:** Dependencies + critical path visible.
- **Surface:** tasks (`dependency-graph` + Gantt view).
- **Priority:** Must.
- **Acceptance:** Cycle check ≤50ms; Gantt renders ≤300ms p99.

#### TSK-009 — Brian time-tracks against a task

- **Persona:** Brian.
- **Story:** Brian hits `t` on a task to start a timer. Codes for 90
  minutes. Stops. The 90min is logged. Weekly, his time-by-project
  summary surfaces for capacity planning.
- **Outcome:** Lightweight time-tracking.
- **Surface:** tasks (`time-tracking` BC).
- **Priority:** Should.
- **Acceptance:** Timer survives app crash; summary aggregates.

#### TSK-010 — Kara templates a CSM playbook as a project template

- **Persona:** Kara.
- **Story:** Kara saves her "Customer Onboarding Playbook" as a project
  template (30 tasks with dependencies, custom fields, owners).
  Future customer onboardings instantiate from the template.
- **Outcome:** Repeatable customer onboarding.
- **Surface:** tasks (template-marketplace).
- **Priority:** Should.
- **Acceptance:** Template-to-project ≤2s; merge-fields populate
  (customer name, contract date, etc.).

### 3.9 Surface 9: Plugin App Store (work mode)

The plugin-app-store µservice (per microservices/plugin-app-store/PRD.md) is
the third-party plugin distribution surface.

#### PLG-001 — Hiroshi approves plugins for org-wide use

- **Persona:** Hiroshi (IT Admin).
- **Story:** Hiroshi sets the tenant plugin policy: only plugins with
  `vetting:enterprise` badge installable; categories `analytics`,
  `crm`, `developer-tools` allowed; `ai-generative` requires per-
  install case-by-case approval. Employees attempting install of non-
  conforming plugins are refused with policy-display.
- **Outcome:** Plugin governance.
- **Surface:** plugin-app-store (per-plugin-permissions) + tenant admin.
- **Priority:** Must.
- **Acceptance:** Policy enforcement at install attempt; refusal
  message clear.

#### PLG-002 — Brian installs an approved plugin

- **Persona:** Brian.
- **Story:** Brian opens plugin store, filters approved-for-org,
  finds the GitHub plugin, reviews capabilities, installs. The plugin
  attaches to his account; capability grants are scoped per his role.
  Installation completes in ≤5s.
- **Outcome:** Self-service install within governance.
- **Surface:** plugin-app-store (`plugin-install` + per-plugin-perms).
- **Priority:** Must.
- **Acceptance:** Install ≤5s p95 per plugin-app-store PRD.

#### PLG-003 — Developer publishes a paid plugin

- **Persona:** External developer (not in core persona set; included
  for end-to-end ecosystem).
- **Story:** A developer submits a paid plugin via developer-sdk;
  vetting pipeline runs (Cosign + Trivy + Wasmtime + Cedar + WCAG
  + AI-Act + perf); vetting reviewer approves; plugin lists in catalog
  with price + capabilities. Tenants install with per-seat billing.
- **Outcome:** Marketplace economics.
- **Surface:** plugin-app-store (vetting-pipeline + subscription-
  billing).
- **Priority:** Must (M04).
- **Acceptance:** Vetting ≤4h p95; billing reconciled via finops-
  portal.

#### PLG-004 — Anna installs a calendar plugin

- **Persona:** Anna.
- **Story:** Anna installs the "Time-Off Sync" plugin that syncs HR
  vacation requests to her calendar visibility. Permission grant
  modal lists exactly what the plugin accesses (calendar read/write,
  HR vacation read); she accepts.
- **Outcome:** Plugin extends workflow.
- **Surface:** plugin-app-store (Apple-style consent modal).
- **Priority:** Should.
- **Acceptance:** Consent modal explicit; capabilities granted match
  declared.

#### PLG-005 — Jin develops an internal plugin

- **Persona:** Jin (DevOps).
- **Story:** Jin builds an internal-only plugin for his team that
  surfaces ArgoCD deploys in a messenger card. Submits via developer-
  sdk → publishes to private tenant-scope. Only acme-corp installations
  visible.
- **Outcome:** Tenant-private plugins.
- **Surface:** plugin-app-store + developer-sdk.
- **Priority:** Should.
- **Acceptance:** Private publish scope; per-tenant catalog visibility.

#### PLG-006 — Inez audits per-plugin actions

- **Persona:** Inez.
- **Story:** Inez opens compliance console → plugin actions audit.
  Sees per-plugin: action count, data classes touched, principals
  affected, declared-vs-observed capability deltas. Suspects one
  plugin of over-reach; she suspends.
- **Outcome:** Plugin-as-attack-vector visibility.
- **Surface:** plugin-app-store (audit-stream) + compliance console.
- **Priority:** Must.
- **Acceptance:** Audit query ≤2s; suspension propagates ≤30s.

#### PLG-007 — Kara installs a Salesforce sync plugin

- **Persona:** Kara.
- **Story:** Kara installs the Salesforce-CSM-sync plugin. It syncs
  customer health metrics from Salesforce into her oyatie command
  center. Capability grant: read-only Salesforce account + contact +
  opportunity. Sync runs every 15 minutes.
- **Outcome:** Cross-system data flow controlled.
- **Surface:** plugin-app-store + CSM command center surface.
- **Priority:** Should.
- **Acceptance:** Sync interval respected; rate-limit honored.

#### PLG-008 — Hiroshi sets per-plugin spend cap

- **Persona:** Hiroshi.
- **Story:** Hiroshi caps each plugin's per-tenant spend (LLM tokens,
  HTTP calls, storage) per a budget. Overruns auto-throttle plus
  alert. Per-plugin spend visible in finops-portal.
- **Outcome:** No surprise plugin bills.
- **Surface:** plugin-app-store + finops-portal.
- **Priority:** Should.
- **Acceptance:** Spend cap enforced atomically; alerts fire at 80 /
  95 / 100%.

### 3.10 Surface 10: Tenant Admin Console

The tenant admin console (across tenancy + identity + audit-chain + policy-
engine + finops-portal + observability) is the central tenant operator
surface.

#### ADM-001 — Hiroshi onboards a new tenant

- **Persona:** Hiroshi.
- **Story:** Hiroshi opens "Onboard new tenant" (for a subsidiary acme-
  corp-eu). Configures: SSO via SAML (uploads metadata XML), default
  retention (per-jurisdiction; EU GDPR), DLP policies (default + EU
  overlay), KMS pinning (KR / EU / US cell selection), encryption-key BYOK option (ADR-0251 §D-10)
  off (use tenant-managed KMS).
- **Outcome:** New tenant operational in <30 minutes.
- **Surface:** tenancy + identity + policy-engine + cloud-secrets.
- **Priority:** Must.
- **Acceptance:** Tenant operational ≤30min; SSO test passes; per-
  jurisdiction overlays applied.

#### ADM-002 — Hiroshi adds a new compliance pack

- **Persona:** Hiroshi.
- **Story:** acme-corp gets SOC 2 + KR-PIPA + new KR-FSS requirement.
  Hiroshi selects compliance packs in tenant admin; each pack
  contributes Cedar fragments + retention floors + audit-chain
  requirements + reporting templates.
- **Outcome:** Compliance posture composable.
- **Surface:** policy-engine (pack composition) + tenancy.
- **Priority:** Must.
- **Acceptance:** Pack activation ≤1min; per-pack CI lanes BLOCKER
  status reachable.

#### ADM-003 — Hiroshi audits user activity

- **Persona:** Hiroshi.
- **Story:** Hiroshi queries audit-chain: "All actions by principal X
  in last 30 days, across all surfaces". Returns within seconds with
  per-action breakdown, data classes touched, Cedar evaluation
  outcomes.
- **Outcome:** Per-user audit visibility.
- **Surface:** audit-chain + tenant admin.
- **Priority:** Must.
- **Acceptance:** Query ≤5s for ≤100K events; sealable evidence
  packet exportable.

#### ADM-004 — Hiroshi sets feature flags per team

- **Persona:** Hiroshi.
- **Story:** Hiroshi enables a beta feature (e.g., new AI-assist
  composer in mail) for `team:engineering-leadership` only. Other
  teams unaffected.
- **Outcome:** Feature-flag-driven rollout.
- **Surface:** feature-flags µservice + tenant admin.
- **Priority:** Should.
- **Acceptance:** Flag flip ≤30s; per-team scope respected.

#### ADM-005 — Inez configures legal hold for litigation

- **Persona:** Inez.
- **Story:** A subpoena lands. Inez configures legal hold scope: all
  mail / drive / docs / messenger / community for principals X, Y, Z
  during a date range. Hold-before-purge invariant engages; retention
  expiry blocked.
- **Outcome:** Legal hold uniform across surfaces.
- **Surface:** audit-chain (cross-channel hold coordinator) + mail +
  drive + docs + messenger + community.
- **Priority:** Must.
- **Acceptance:** Hold engages ≤2s; bypass attempts emit Sev-1.

#### ADM-006 — Inez initiates DSAR cascade for departing employee

- **Persona:** Inez.
- **Story:** An EU employee leaves and exercises GDPR Art 17 erasure.
  Inez triggers DSAR; the cascade per ADR-0242 Appendix B enumerates
  every µservice + per-source erasure plan; she reviews + approves;
  execution runs; subject receives confirmation within 30 days.
- **Outcome:** Uniform DSAR per the keystone doctrine.
- **Surface:** governance/dsar-intake + audit-chain + every µservice
  touched.
- **Priority:** Must.
- **Acceptance:** Plan generation ≤5min; execution within 30-day SLA;
  confirmation packet sealed.

#### ADM-007 — Hiroshi configures cell pinning for data residency

- **Persona:** Hiroshi.
- **Story:** acme-corp-eu requires data residency in EU. Hiroshi pins
  the tenant to `cell-eu-frankfurt-1` (primary) + `cell-eu-paris-1`
  (DR). Per ADR-0240 sovereign-cloud-per-regional-pack: only EU-
  region cells eligible; cross-region replication SCC-gated.
- **Outcome:** Residency enforced by construction.
- **Surface:** cell µservice + tenancy + Cedar.
- **Priority:** Must.
- **Acceptance:** Pin enforced at cell-routing layer; egress checks.

#### ADM-008 — Hiroshi configures encryption-key BYOK (ADR-0251 §D-10)

- **Persona:** Hiroshi.
- **Story:** acme-corp-eu requires encryption-key BYOK (their own KMS; ADR-0251 §D-10). Hiroshi adds
  KMS endpoint (their AWS KMS in their AWS account) + service-
  account permission. Tenant DEKs wrapped under their KEK.
- **Outcome:** Customer-managed encryption.
- **Surface:** cloud-secrets (encryption-key BYOK, ADR-0251 §D-10) + tenancy.
- **Priority:** Should.
- **Acceptance:** encryption-key BYOK enabled ≤10min; key rotation supported (ADR-0251 §D-10).

#### ADM-009 — Hiroshi manages partner-tenant relationships

- **Persona:** Hiroshi.
- **Story:** acme-corp engages Leo's consulting firm (`tenant-leo-
  partners-llp`) for Q3. Hiroshi creates a partner-tenant relationship,
  scopes Leo's assume-role to `engagement-2026-q3-leo-acme` channels +
  drive folders, sets TTL 90 days. At TTL end, role-off automatic.
- **Outcome:** Cross-tenant work governed.
- **Surface:** tenancy (partner-tenant) + Cedar + identity.
- **Priority:** Must.
- **Acceptance:** Relationship create ≤2min; auto role-off at TTL;
  audit-chain attribution.

#### ADM-010 — Inez configures DLP rules per pack

- **Persona:** Inez.
- **Story:** Inez configures DLP rules: in KR pack, RRN patterns
  (주민등록번호) auto-flagged; in EU pack, IBAN + national ID
  patterns; in US pack, SSN + credit card patterns. Rules apply
  uniformly across mail, drive, docs.
- **Outcome:** Per-pack DLP without per-surface duplication.
- **Surface:** policy-engine + DLP rules.
- **Priority:** Must.
- **Acceptance:** Per-pack rules apply ≤5min after change.

#### ADM-011 — Gabriela reviews her tenant's compliance posture

- **Persona:** Gabriela (COO).
- **Story:** Gabriela opens the executive compliance dashboard: SOC 2
  control status, KR PIPA evidence packets, GDPR DSAR SLA stats,
  legal-hold inventory, audit-finding remediation. Greens / yellows /
  reds visible.
- **Outcome:** Executive-grade compliance signal.
- **Surface:** tenant admin (compliance dashboard) + audit-chain +
  policy-engine.
- **Priority:** Should.
- **Acceptance:** Dashboard load ≤2s; per-control drill-down.

#### ADM-012 — Hiroshi configures cost-center attribution

- **Persona:** Hiroshi.
- **Story:** Hiroshi maps oyatie sub-tenants / teams to internal cost
  centers (eng / sales / marketing / ops). Per-cost-center spend
  rolls up in finops-portal. Monthly invoice line-items per cost
  center.
- **Outcome:** Internal chargeback / showback.
- **Surface:** finops-portal + tenancy.
- **Priority:** Should.
- **Acceptance:** Cost-center taxonomy editable; per-CC invoice rolls.

---

## 4. Cross-surface Integration Stories

These stories explicitly cross µservice boundaries. They illustrate the
power of the platform when surfaces compose.

### XS-001 — New customer onboarding (end-to-end)

**Personas:** Emma (Sales), Catalina (HR-as-customer-ops), Hiroshi (IT-as-
provisioning), Kara (CSM), Faisal (Marketing-as-welcome-comms).

**Story:** Emma closes a new customer deal. She marks the CRM
opportunity "Closed-Won". This triggers the customer-onboarding
workflow (WFS-011): the CSM rotation assigns Kara as the dedicated
CSM. Hiroshi-as-IT provisions the customer's tenant with declared
configuration. A welcome mail is sent from Faisal's marketing-welcome
mailbox. A customer-shared community space is created. A drive folder
is seeded with onboarding docs. A kick-off calendar event is scheduled
within 5 business days. A tasks project for Kara is instantiated from
her CSM playbook. The customer's success metrics start tracking in her
command center.

**Outcome:** 7 surfaces coordinate in one workflow; first-touch
customer experience consistent.

### XS-002 — Quarterly business review

**Personas:** Anna (Manager), Gabriela (Exec), Dan (Finance).

**Story:** Each quarter, the QBR cycle runs: calendar schedules QBR
meetings across all departments; docs are auto-created from QBR
templates (pre-populated with finops + headcount + key metrics from
sheets pivots); meet hosts the meetings with recording; community
captures action items in a per-team thread; tasks µservice instantiates
follow-ups; mail digests circulate the QBR outcomes to executives.

**Outcome:** QBR cycle structured across 7 surfaces; nothing manual.

### XS-003 — Incident response

**Personas:** Jin (DevOps), Brian (Engineer), Anna (Manager), Kara
(CSM), Gabriela (Exec), Inez (Security).

**Story:** PagerDuty pages on a P0 production outage. The plugin emits
a card to `#oncall`. The workflow auto-creates a war-room incident
channel + invites the on-call rotation + Anna (incident commander) +
Kara (customer comms lead). The war-room has a pinned docs page for
the live timeline + a pinned sheet for impact tracking. Meet bridge
auto-spins up; recording engaged with consent. As resolution flows,
status posts to a customer-facing status-page community space. Post-
incident, the docs page becomes the postmortem (per COM-011); action
items become tasks; Inez reviews the postmortem for security signal.

**Outcome:** Incident structured across 6 surfaces; no signal lost.

### XS-004 — Vendor onboarding

**Personas:** Dan (Finance), Hiroshi (IT), Inez (Security).

**Story:** A new vendor is engaged. The vendor-onboarding workflow
runs: forms intake captures vendor data (legal name, tax forms,
banking, security questionnaire); legal reviews contracts in docs
with e-sign integration; security review by Inez checks DLP / SOC 2
posture; finance sets up in ERP via plugin; IT provisions a vendor-
portal account if relevant. Approvals stack; vendor record sealed in
ontology.

**Outcome:** Vendor onboarded with full audit trail.

### XS-005 — Performance review cycle

**Personas:** Catalina, Anna (as manager), Brian (as employee), Gabriela
(as approver of org-wide calibration).

**Story:** Catalina launches the perf cycle (HR-004). Each manager
gets a docs review template per direct report. 360-feedback collected
via forms. Calibration session held in meet, facilitated via a sheets
calibration grid. Decisions ship; comp adjustments flow to payroll
(HR-012). Brian sees his result in his self-service portal.

**Outcome:** Perf cycle uniform; calibration auditable.

### XS-006 — Marketing campaign launch

**Personas:** Faisal, Emma, Kara.

**Story:** Campaign launch workflow (WFS-006): community announcement,
mail-blast, social-post, internal `#marketing-launches` post. Emma's
sales follow-up tasks instantiate. Kara's customer-touches scripted.
Analytics dashboard tracks attribution. Post-campaign retro in docs.

**Outcome:** Multi-surface launch coordinated.

### XS-007 — Employee onboarding (deep)

**Personas:** Catalina, Anna (hiring manager), Brian (peer mentor),
Hiroshi (IT), new hire Mariana.

**Story:** Per HR-001 + WFS-002: Mariana's offer is signed in docs
with e-sign. HR-001 fires; onboarding workflow runs. Hiroshi-as-IT
auto-provisions Mariana's accounts. Anna receives a calendar invite
for the 1:1; Brian is assigned as peer-mentor with his own task
(walk through codebase). Mariana opens oyatie on day 1; the
onboarding KB tour is pinned (COM-004); her tasks are pre-loaded;
her welcome mail is waiting; her drive folder has the materials.

**Outcome:** Day-1 experience polished; HR cost down.

### XS-008 — Employee offboarding (deep)

**Personas:** Catalina, manager, Hiroshi, Inez.

**Story:** Per HR-007 + WFS-003 + ADM-006: termination submitted +
approved + scheduled. Offboarding workflow fires on last-day. SSO
disabled. Mailbox + drive transferred to manager. Channels revoked.
Plugins uninstalled. Tasks reassigned. Equity treatment routed to
Carta. DSAR option offered to departing employee per their
jurisdiction.

**Outcome:** Offboarding complete in hours, not weeks.

### XS-009 — Sales-to-CS handoff

**Personas:** Emma → Kara.

**Story:** Emma's deal closes. Sales-to-CS handoff workflow fires:
deal context (customer goals, mutual action plan, technical
requirements) packages from Salesforce into a docs handoff brief.
Kara is assigned. Customer-shared community + drive folder + meet
kickoff scheduled. Emma's commission marks (HR-006). Kara's playbook
instantiates (XS-001).

**Outcome:** No "lost in handoff" pain.

### XS-010 — Expense → Reimbursement

**Personas:** Brian (submitter), Anna (approver), Dan (finance).

**Story:** Per TSK-004 + WFS-004 + MSG-004: Brian submits expense via
form. Workflow routes to Anna (under $500: direct). Anna approves via
messenger reaction card. Expense flows to finops; reimbursement
issued next payroll. Brian sees status in his self-service portal.

**Outcome:** Expense lifecycle one workflow.

### XS-011 — Document → Decision → Action

**Personas:** Anna (decision-author), Brian (commenter), team.

**Story:** Anna drafts a tech-architecture proposal in docs; shares
with team for comment. Comments resolve; decision finalised. The
docs surface offers "Convert to community decision-log + create
follow-up tasks". One click: post pinned in `#engineering-decisions`
community space; 5 tasks created in tasks µservice with owners +
due dates.

**Outcome:** Decision → action without manual artifact-shuffling.

### XS-012 — Customer support thread → KB

**Personas:** Kara, support engineer.

**Story:** Customer reports an issue in their support channel. Kara +
engineer resolve it. Post-resolution, Kara hits "Promote to KB"; the
thread distills into a community KB article (auto-drafted, then
human-edited). Future customers searching find the answer.

**Outcome:** Support resolves once, helps many.

### XS-013 — Board meeting prep

**Personas:** Gabriela, CFO, CEO's EA.

**Story:** Quarterly board prep: docs assembles board deck (pulling
financials from sheets, KPIs from finops, headcount from HR). Board
members access via signed drive links (WORM-locked at meeting time).
Calendar schedules the meet. Post-meeting, the minutes archive in
community board-only space.

**Outcome:** Board prep less painful.

### XS-014 — Security incident review

**Personas:** Inez, CISO, Hiroshi, engineering.

**Story:** A DLP-flagged event surfaces (MSG-009). Inez opens a case;
the case-creation workflow runs: private incident channel, evidence
collection workflow (audit-chain query, mail samples, drive access
logs), legal-hold engages, four-eyes approvals on plaintext access,
postmortem in community.

**Outcome:** Repeatable security incident process.

### XS-015 — Quarterly compliance attestation

**Personas:** Inez, Hiroshi, executives.

**Story:** Per ADM-011 + ADM-002: each quarter, every active
compliance pack runs its attestation: SOC 2 controls auto-collected
from observability, audit-chain, Cedar fragments; KR PIPA evidence
emitted; SOC 2 reviewer (external) accesses a read-only audit packet.

**Outcome:** Quarterly compliance evidence assembled mostly-
automatically.

---

## 5. Per-role Day-in-the-Life Narratives

These are chronological hour-by-hour narratives for each major work persona.
Each narrative grounds the per-surface stories in real workday context.

### DIL-001 — A day in the life of a Manager (Anna)

**8:30 AM:** Anna opens oyatie on her laptop while sipping coffee. Her
home shell shows: today's calendar (a 9am team standup, a 10am 1:1
with Brian, an 11:30am architecture review, a 2pm cross-functional sync
with Faisal + Kara, a 4pm focus block), her notification tray (12
items; 3 @mentions in messenger, 5 mail-rule-routed VIPs, 2 approval
requests, 2 task assignments). She processes @mentions in 4 minutes
via keyboard nav; 2 are FYI (mark-read); 2 require thread replies (she
fires off 2-sentence responses); 1 is a question that requires more
thought (snoozes for end of standup).

**9:00 AM:** Team standup. She joins the meet bridge; the recurring
event has the team-standup docs template pre-attached. Standup
proceeds: each team member updates against blockers + asks + updates;
Anna captures action items in tasks µservice with assignees during the
meeting (no post-meeting transcription overhead).

**9:30 AM:** Post-standup, the snoozed @mention surfaces. She reads,
considers, replies with a substantive answer; reply is logged as a
decision in the engineering decision-log community space (COM-008).

**10:00 AM:** 1:1 with Brian. She opens the recurring 1:1 docs page;
agenda items from both of them surface; growth-conversation notes
from last 1:1 are visible. They discuss; Anna captures Brian's career
ask (more system-design exposure); she creates a task on her own list
to propose him for the architecture-review committee.

**10:30 AM:** Anna catches up on the mail digest (MAIL-006-equivalent
for her direct reports). 3 unread VIP threads; she handles each in
≤3min. Inbox-zero at 10:50am.

**11:30 AM:** Architecture review. She presents her team's design via
slides + embedded docs design-doc; meet recording captured (with
consent); decisions logged.

**12:30 PM:** Lunch (focus block; messenger DND auto-engaged).

**1:00 PM:** Anna reviews her tasks + delegates one to a senior
engineer. She uses bulk-edit (TSK-002) to reassign 3 tasks.

**2:00 PM:** Cross-functional sync with Faisal + Kara. They discuss
the Q3 campaign + customer launch sequencing. The campaign channel
(MSG-001) accumulates decisions live.

**3:00 PM:** Anna approves 2 expense reports via messenger reactions
(MSG-004). She approves a comp adjustment for one of her reports via
the HR portal (HR-012).

**4:00 PM:** Focus block — Anna writes a technical strategy doc in
docs µservice. Her DND is on; she's not interrupted.

**5:30 PM:** Wrap-up. Anna's "End of Day" digest summarises what she
shipped today + what needs her tomorrow. She closes the laptop.

### DIL-002 — A day in the life of an Engineer (Brian)

**9:00 AM:** Brian arrives, opens oyatie. He sets his focus-mode
profile to "deep-work"; messenger DND on; mail batched. His tasks
queue shows 4 open tasks; he picks the highest-priority one (the
billing migration). He uses `Cmd+K` to jump to the related design
doc, the related PR, the related Linear-equivalent.

**9:00 AM - 11:30 AM:** Deep work on the billing migration. He
commits + opens a PR. CI runs; status posts to `#backend-engineering`
via card (WFS-001). One CI step fails; he investigates via the replay-
debugger (WFS-010-equivalent).

**11:30 AM - 12:00 PM:** Code review for a teammate's PR. Inline
comments in the PR (via GitHub plugin) surface in messenger; he
responds in-thread (MSG-002).

**12:00 PM:** Lunch. Messenger shows a missed @mention from Anna; he
responds when he gets back.

**1:00 PM:** 1:1 with Anna (per Anna's DIL).

**1:30 PM - 4:00 PM:** Another deep-work block. Brian uses focus mode
(MSG-013).

**4:00 PM:** Sprint retro (CAL-009) with breakout rooms.

**5:00 PM:** End of day. Brian time-tracks his work against the
billing migration task (TSK-009).

**5:30 PM:** Closes laptop.

### DIL-003 — A day in the life of HR (Catalina)

**8:30 AM:** Catalina opens oyatie. Her HR-console dashboard shows:
new-hires-starting-today (2), pending vacation requests (5), pending
comp-adjustments (3), open termination cases (1), enrollment campaign
status (open-enrollment-fall-2026 at 67% completion with 3 days
remaining).

**9:00 AM:** She onboards Mariana (HR-001 + XS-007).

**9:30 AM:** She reviews + approves 4 of 5 vacation requests (HR-002);
1 conflicts with a blackout, she replies with alternative dates.

**10:00 AM:** She reviews 2 comp adjustments (HR-012); approves both;
escalates 1 for the report whose new comp would exceed band.

**11:00 AM:** Open-enrollment monitoring (HR-008). 33% of employees
haven't enrolled with 3 days remaining; she triggers a reminder.

**12:00 PM:** Lunch.

**1:00 PM:** A termination conversation (HR-007). She prepares; runs
the HR-007 protocol with the manager + employee.

**2:30 PM:** Quarterly perf cycle prep (HR-004 + XS-005). She finalises
templates + calibration grid.

**3:30 PM:** Policy update — new code of conduct (COM-009). She
posts.

**4:30 PM:** Drafts a community KB article on parental leave policy
(HR-010 + COM-010).

**5:30 PM:** Wrap-up.

### DIL-004 — A day in the life of a Sales Rep (Emma)

**8:30 AM:** Emma reviews her pipeline dashboard (CRM plugin in her
oyatie shell). Top of mind: 3 deals at the verbal-commit stage.

**9:00 AM:** Cold outreach hour. She drafts a templated mail (MAIL-
010) for a new prospect.

**10:00 AM:** Discovery call with a prospect via meet. Auto-transcribed;
auto-summary populates Salesforce via plugin (EMA-equivalent flow).

**11:00 AM:** Post-call she enters CRM notes (or, more often, the
auto-fill is good enough that she just reviews + accepts).

**12:00 PM:** Lunch.

**1:00 PM:** Internal sales sync (calendar recurring event).

**2:00 PM:** A high-intent lead surfaces (lead-score > 80 per WFS-005);
she schedules a follow-up call (CAL-001).

**3:00 PM:** Customer demo via meet + slides.

**4:00 PM:** Proposal writing in docs; uses a customer-pitch template.

**5:00 PM:** Wrap-up; reviews commission status (HR-006).

### DIL-005 — A day in the life of Finance (Dan)

**8:30 AM:** Dan opens his finance dashboard. Pending: 18 expense
approvals, 3 budget variance alerts, 2 vendor onboarding intake forms.

**9:00 AM - 10:00 AM:** Expense approval triage via messenger reactions
(MSG-004 batch).

**10:00 AM:** Monthly close prep meeting with his team via meet.

**11:00 AM:** Budget variance analysis in sheets (DRV-004).

**12:00 PM:** Lunch.

**1:00 PM:** Vendor onboarding review (XS-004).

**2:00 PM:** Payroll exception review (HR-003 pre-run).

**3:00 PM:** Board prep — financial section (XS-013).

**4:00 PM:** Strategy work in docs.

**5:30 PM:** Wrap-up.

### DIL-006 — A day in the life of Marketing (Faisal)

**9:00 AM:** Faisal opens his marketing dashboard. Yesterday's campaign
launched at 95% deliverability. NPS came back 47 (down 3 from last
quarter; he flags).

**9:30 AM:** Campaign retro in `#marketing` community thread (COM-003).

**10:00 AM:** Hosts a weekly AMA in `#ask-marketing` (MSG-006) for the
first 30 minutes; engages community.

**10:30 AM:** Drafts copy for the next campaign in docs.

**11:30 AM:** Reviews creative drafts from Figma plugin in slides
preview.

**12:00 PM:** Lunch.

**1:00 PM:** Cross-functional sync with Anna + Kara (per Anna's DIL).

**2:00 PM:** Survey rollout (TSK-006).

**3:00 PM:** Campaign launch workflow run (WFS-006) for a small
segment.

**4:00 PM:** Customer advocacy community moderation (COM-005).

**5:30 PM:** Wrap-up.

### DIL-007 — A day in the life of an Executive (Gabriela)

**7:00 AM:** Gabriela checks her dashboard from her phone over
breakfast. Top metrics; she dictates 2 short replies via voice to her
EA.

**8:30 AM:** She arrives. Her EA has triaged her morning mail (CAL-
005). She has 4 1:1s with direct reports today.

**9:00 AM:** 1:1 with CFO.

**10:00 AM:** 1:1 with CMO.

**11:00 AM:** Board call prep (XS-013).

**12:00 PM:** Lunch with a customer at HQ.

**1:30 PM:** 1:1 with CRO.

**2:30 PM:** 1:1 with Catalina (HR).

**3:30 PM:** Compliance dashboard review (ADM-011).

**4:30 PM:** Strategy reading time (focus block; she dictates
annotations into her own private community KB).

**6:00 PM:** Wrap-up.

### DIL-008 — A day in the life of an IT Admin (Hiroshi)

**9:00 AM:** Hiroshi checks tenant admin dashboard. Yesterday: 3
employee onboardings, 1 offboarding (full evidence packet sealed),
12 SSO logins from unusual geos (he reviews + clears).

**9:30 AM:** Plugin policy review (PLG-001); approves 2 new plugins
from the vetting queue.

**10:30 AM:** Onboards a new APAC subsidiary tenant (ADM-001).

**12:00 PM:** Lunch.

**1:00 PM:** Compliance pack review with Inez.

**2:30 PM:** IT-asset audit campaign launch (TSK-007).

**3:30 PM:** Per-team retention policy update (MSG-008).

**5:00 PM:** Wrap-up.

### DIL-009 — A day in the life of DevOps (Jin)

**8:00 AM:** Jin checks PagerDuty + observability dashboards.

**9:00 AM:** Sprint retro (CAL-009).

**10:30 AM:** A flaky CI workflow needs debug (WFS-010).

**12:00 PM:** Lunch.

**1:00 PM:** Builds an internal plugin (PLG-005).

**3:00 PM:** Postmortem from yesterday's near-miss (COM-011).

**4:30 PM:** Capacity planning in sheets.

**5:30 PM:** Wrap-up.

### DIL-010 — A day in the life of a CSM (Kara)

**8:30 AM:** Kara opens her CSM command center. Per-customer health
scores; 2 customers in yellow.

**9:00 AM:** Customer QBR (XS-002).

**11:00 AM:** Triages a P0 escalation from yesterday (MSG-011) — now
resolved; she sends post-resolution mail to customer with
postmortem link.

**12:00 PM:** Lunch.

**1:00 PM:** Office hours (CAL-010).

**2:30 PM:** Account planning for top-3 accounts in docs.

**3:30 PM:** Salesforce sync review (PLG-007).

**4:30 PM:** KB article on customer retention playbook (COM-010).

**5:30 PM:** Wrap-up.


---

## 6. UX Strive / Avoid (Work-Specific)

This section codifies the design system's strive-for + avoid stances for B2B
work surfaces. Each strive + avoid pair is grounded in a competitor reference
that oyatie either matches or exceeds.

### 6.1 Strive for

1. **Keyboard-first power-user shortcuts.** `Cmd+K` opens a universal command
   palette in every surface. Every action discoverable in the palette. j/k
   navigation in lists. Vim-style verb-object grammar where it fits. Linear
   sets this bar; oyatie matches.
2. **Deep linking between surfaces.** A mail thread links to a messenger
   channel which links to a calendar event which links to a docs design doc
   without re-auth, without losing context, with bidirectional refer-back.
   Slack's "Quick Switcher" + Notion's `@` mention are the comparison bar.
3. **Rich context cards.** Hover over a Person + see their role + last
   activity + open tasks + recent comms. Hover over a Channel + see members
   + activity + retention. Hover over a Doc + see authors + recent edits.
   GitHub's hover cards set the bar.
4. **Intelligent defaults.** Onboarding workflow is pre-configured;
   retention floors set per-jurisdiction; common Cedar permits are pre-
   declared. New tenants are functional in under 30 minutes (ADM-001 bar).
5. **Templates everywhere.** Every workflow, doc, sheet, slide, task project,
   form is templatable. Templates are sharable in a per-tenant template
   marketplace. Notion's template gallery sets the bar; we exceed by making
   templates work across surfaces.
6. **Sentiment-aware notifications.** A "from your manager" mention is more
   urgent than a "from a watcher" notification. Notification fan-out
   reweights per relationship + recency. Slack's "important conversations"
   inbox is the comparison.
7. **Focus mode.** DND that respects calendar focus blocks, digest-on-resume
   summaries, mention-throughscale (only VIPs ring through). Slack's "DND"
   sets the bar; we exceed by integrating calendar + meet + mail.
8. **Calendar-aware scheduling.** The system knows what's on your calendar
   when surfacing options. Suggested-times use availability. Reminders
   respect working hours. Time-zone localisation per-attendee is automatic.
9. **Meeting-fatigue detection.** If a user's daily meeting load exceeds
   their declared threshold, the calendar UX suggests declining + offers to
   propose async alternatives. Outlook's MyAnalytics + Microsoft Viva
   Insights set partial comparison; we go further by integrating with
   task + mail to identify the highest-leverage meetings.
10. **One-click cross-surface actions.** A right-click in messenger
    promotes a thread to a tasks item, a docs page, a community KB, a
    calendar event. Without rewriting context. Slack's "Add to ..." is
    weak; oyatie's is the bar.
11. **Predictable round-trip on every save.** Workflow specs round-trip
    byte-equal; doc edits round-trip CRDT-merged; sheet formulas round-trip
    semantically-identical; slide layouts round-trip pixel-stable. No
    silent mutation on save.
12. **Audit visibility at user level.** Every user can see their own audit
    trail ("here is every action you took on platform last 30 days"); not
    just for compliance, also for self-debugging.
13. **Composable workspaces.** Users define their own "workspace" (a
    bookmark of channels + docs + projects + dashboards) and switch between
    them with `Cmd+1`/`Cmd+2`/etc. Notion's workspaces set the bar; we
    exceed by making workspaces cross-µservice.
14. **Plugin governance visible to user.** Users see what plugins they have
    + what each plugin accessed about them. Apple's "App Privacy" report is
    the comparison.

### 6.2 Avoid

1. **5-page onboarding flows.** Decompose into per-step actions, not
   monolithic flows. Anyone abandoning at step 3 should still have a useful
   state.
2. **Over-permissioning.** Default to least-privilege; declare every
   capability; explicit grant at install + role. Apple App Store style.
3. **Legacy paper-form replicas.** A web form that looks like a paper form
   is a bad form. Use conditional logic, inline validation, autosave.
4. **Siloed UX.** Every surface should feel like one product, not 12 tools.
   Common header, common sidebar, common command palette, common
   notifications.
5. **Excessive notifications.** Notification fatigue is a top reason users
   churn. Default to thread-only, mention-only, digest-by-default. Allow
   per-channel + per-conversation overrides.
6. **Meeting overload.** Don't accept invites that exceed the user's daily
   threshold; suggest decline. Don't double-book without explicit override.
7. **Ad-hoc workflows.** A 10-step process that an engineer remembers in
   their head is a bug. Encourage codification in workflow-studio.
8. **Surprise charges.** Per-plugin spend caps; per-tenant overall budget
   alarms; FinOps portal as the source of truth.
9. **Black-box AI.** Every AI-assisted action shows: which model invoked,
   what prompt-class fired, what data classes were touched, EU AI Act tier.
   Opt-out always available.
10. **Tab proliferation.** A user shouldn't need to open 7 tabs to do a
    single task. Right-pane drawer, popovers, inline previews preferred.
11. **Hidden retention policies.** Users should see how long their content
    will live + the per-jurisdiction reason. No surprise deletions.
12. **Persona-blurring.** Personal context must be invisible to org admin
    flows. Professional context audit must not show personal DM content.
    Per ADR-0238 dual-context isolation.

### 6.3 Compare / exceed

| Competitor | Where they set the bar | Where oyatie exceeds |
|---|---|---|
| Microsoft 365 | Enterprise breadth | Per-microservice flat layout (no SKU lock-in); per-tenant sovereign deployment per ADR-0240 |
| Google Workspace | Collab + ML | Local-first via per-pack residency; offline-tolerant sync via FastCDC delta-sync; spec-round-trip invariants |
| Slack | Channels + DMs UX | Dual-context isolation built into the data model; uniform Cedar gating across all surfaces |
| Notion | Block-based docs + database | First-class docs + sheets + slides + community as separate µservices that compose; superior cross-surface deep-linking |
| Salesforce | CRM + Trailhead community | CRM via plugin; Trailhead-equivalent via community + intelligence µservices, with no vendor lock-in |
| Workday | HCM + payroll | HR composes via Ontology Employee + tasks + forms + workflow-studio rather than vertical SKU |
| Asana / Monday / ClickUp | Project + task management | Tasks µservice with deeper integration into messenger + calendar + workflow-studio |
| ServiceNow | ITSM | Workflow Studio as visual canvas + ticket-like tasks; SOC 2 + ITIL workflows codified |
| Lattice / Greenhouse | Perf + recruiting | HR-as-composed-surface with full audit-chain + ML-driven calibration |
| Linear | Opinionated PM UX + keyboard | Tasks µservice's keyboard speed + workflow-studio for state machines |
| Carta | Cap table + equity | Carta-plugin in plugin-app-store; equity events flow through finops + HR Ontology |

---

## 7. Accessibility User Stories (Work-Specific)

Accessibility is a first-class concern. Per WCAG 2.2 AA + KS X 6906 (KR) +
EN 301 549 (EU public-sector accessibility directive).

### A11Y-001 — Jin (color-blind) uses CI dashboard

- **Persona:** Jin (color-blind — deuteranopia variant).
- **Story:** Jin opens the CI dashboard. Status indicators use shape +
  color (✓ green / ✗ red / ⚠ yellow); never color alone. The dashboard's
  high-contrast mode is one toggle; pattern fills on charts (lines /
  dots / dashes); WCAG AA 4.5:1 contrast for text.
- **Outcome:** Jin reads CI status unimpaired.
- **Surface:** observability + design system.
- **Priority:** Must.
- **Acceptance:** WCAG AA + AAA where feasible; no color-only signal.

### A11Y-002 — Gabriela (motor-impaired) uses voice dictation

- **Persona:** Gabriela (motor-fatigue; uses voice as primary input).
- **Story:** Gabriela uses voice to dictate email + messenger replies +
  calendar holds + meeting notes. Voice models tier-T1 (local) for
  privacy. Voice commands ("Send to Catalina: please review the
  attached deck") parse + confirm before send.
- **Outcome:** Voice-first executive workflows.
- **Surface:** mail + messenger + calendar + intelligence (local voice
  model under tenant policy).
- **Priority:** Should.
- **Acceptance:** Voice confirmation before send; local processing
  default for sensitive surfaces.

### A11Y-003 — Cognitive-disability-friendly reading mode

- **Persona:** An employee with dyslexia.
- **Story:** They toggle reading mode in docs + community articles:
  larger line-height, OpenDyslexic font (optional), reduced motion,
  paragraph chunking with summary headlines.
- **Outcome:** Content accessible.
- **Surface:** docs + community + design system.
- **Priority:** Must.
- **Acceptance:** Reading mode toggleable; persists across surfaces.

### A11Y-004 — Brian (hearing-impaired) uses Meet with captions

- **Persona:** Brian (assumes hearing-impaired for this story).
- **Story:** In every meet call, captions auto-enabled. Multi-language
  caption switching. Caption opacity + size adjustable. Transcript
  saved with speaker labels.
- **Outcome:** Hearing-impaired full participation.
- **Surface:** meet (`transcription` BC).
- **Priority:** Must.
- **Acceptance:** Caption WCAG-conformant; transcript downloadable.

### A11Y-005 — Screen-reader-friendly tables in sheets

- **Persona:** An employee using JAWS or NVDA.
- **Story:** Sheets renders as proper ARIA tables with row + column
  headers announced; cell formula descriptions surfaceable; navigation
  via Tab + arrow keys.
- **Outcome:** Sheets usable with screen reader.
- **Surface:** sheets.
- **Priority:** Must.
- **Acceptance:** WCAG AA; tested against JAWS, NVDA, VoiceOver.

### A11Y-006 — Slides supports alt text per element

- **Persona:** Slide author + screen-reader audience.
- **Story:** Slide authors are prompted for alt text on every image +
  chart. Screen-reader audience hears alt text during presentation.
- **Outcome:** Inclusive slide consumption.
- **Surface:** slides.
- **Priority:** Must.
- **Acceptance:** Authoring prompt; screen-reader correct.

### A11Y-007 — Keyboard-only navigation across all surfaces

- **Persona:** Power user without mouse.
- **Story:** Every action achievable via keyboard. `Cmd+K` for command
  palette. Skip-link landmarks. Focus indicators always visible.
- **Outcome:** Mouseless full operation.
- **Surface:** every µservice's frontend.
- **Priority:** Must.
- **Acceptance:** axe-core + Lighthouse a11y 100; CI lane.

### A11Y-008 — Reduced motion respected

- **Persona:** User with vestibular disorder.
- **Story:** `prefers-reduced-motion` media query honored everywhere.
  Slide transitions, workflow-studio canvas animation, messenger
  scroll-into-view all degrade gracefully.
- **Outcome:** Motion sensitivity respected.
- **Surface:** every frontend.
- **Priority:** Must.
- **Acceptance:** CSS media-query honored; CI a11y lane.

### A11Y-009 — High-contrast theme

- **Persona:** Low-vision user.
- **Story:** A high-contrast theme available. WCAG AAA 7:1 contrast
  ratios.
- **Outcome:** Low-vision usability.
- **Surface:** design system.
- **Priority:** Should.
- **Acceptance:** Theme toggleable; persists.

### A11Y-010 — Form field labels + error messages

- **Persona:** Screen-reader user filling a form.
- **Story:** Every form field has a programmatically-associated label;
  errors are announced via aria-live; instructions are within the
  label or directly adjacent.
- **Outcome:** Forms accessible.
- **Surface:** forms.
- **Priority:** Must.
- **Acceptance:** WCAG AA conformant.

---

## 8. Localization (Work-Specific)

Per the per-pack regional architecture (ADR-0010, ADR-0240, ADR-0064
canonical-base + localization overlay).

### L10N-001 — KR enterprise users with KakaoTalk patterns

- **Persona:** A KR enterprise user.
- **Story:** Messenger in KR-pack honors KR UX conventions: in-app
  notifications use the KakaoTalk-style alert tone (optional); message
  composer supports Hangul IME without composition-loss bugs; default
  date format `YYYY-MM-DD`; default currency KRW; 음력 (lunar) calendar
  available for cultural-event reference. Names render Surname-First
  (성-이름) order in directory listings + mentions.
- **Outcome:** KR users feel native.
- **Surface:** messenger + calendar + design system KR overlay.
- **Priority:** Must.
- **Acceptance:** KR pack overlay applies on tenant pin; IME-safe.

### L10N-002 — JP enterprise with Japanese vertical option

- **Persona:** A JP enterprise user.
- **Story:** Docs supports tategaki (縦書き; vertical Japanese text)
  layout where the author chooses. Slides templates include JP business-
  card-style layouts. Default date format `YYYY/MM/DD`. Currency JPY
  with no decimals.
- **Outcome:** JP users have native-feeling authoring.
- **Surface:** docs + slides + design system JP overlay.
- **Priority:** Should.
- **Acceptance:** Vertical layout toggleable; print-correct.

### L10N-003 — EU enterprise with GDPR-default privacy choices

- **Persona:** An EU enterprise user.
- **Story:** On tenant onboarding in EU pack: cookies + analytics opt-
  out default; AI processing opt-in only; data retention defaults to
  GDPR-minimum (90d unless legal-hold); cross-border egress refused
  unless SCC declared.
- **Outcome:** Privacy-by-default for EU.
- **Surface:** tenant admin + design system EU overlay + policy-engine.
- **Priority:** Must.
- **Acceptance:** Defaults verified by EU jurisdiction config.

### L10N-004 — KR-FSS financial-services overlay

- **Persona:** Inez (KR-FSS-regulated tenant).
- **Story:** KR-FSS pack adds: 5y retention floors on all audit + mail
  + drive content; per-action regulator-evidence emission; quarterly
  attestation templates pre-built; KR commercial-code-conformant
  e-signing.
- **Outcome:** Regulatory compliance compoundable.
- **Surface:** policy-engine + audit-chain + KR-FSS pack.
- **Priority:** Must.
- **Acceptance:** Pack activation enforces floors uniformly.

### L10N-005 — Right-to-left languages

- **Persona:** An Arabic-speaking user.
- **Story:** UI mirrors RTL: navigation on the right, text flows right-
  to-left, dates + numbers in locale format. Mixed-script content
  (Arabic + English) handles bidirectional text correctly.
- **Outcome:** First-class RTL support.
- **Surface:** design system + every frontend.
- **Priority:** Should.
- **Acceptance:** BiDi tests pass; per-locale review by native
  speakers.

### L10N-006 — Time-zone-correct everything

- **Persona:** A user in São Paulo collaborating with Seoul.
- **Story:** Calendar shows times in user's local timezone with
  recipient's TZ in tooltip. Messages timestamp in local TZ.
  Notifications respect quiet hours per local TZ.
- **Outcome:** Cross-TZ collaboration painless.
- **Surface:** every µservice with temporal data.
- **Priority:** Must.
- **Acceptance:** IANA tzdata + DST-correct.

### L10N-007 — Currency + tax localization

- **Persona:** Dan (Finance) with multi-currency operations.
- **Story:** Expense submissions accept native currency; conversion
  done at policy-determined rate (real-time FX from a pinned source);
  tax per local jurisdiction (US sales tax, EU VAT, KR 부가가치세,
  JP consumption tax).
- **Outcome:** Multi-currency, multi-tax compliant.
- **Surface:** finops-portal + HR/payroll + workflow-studio.
- **Priority:** Must.
- **Acceptance:** Conversion + tax computed correctly per pack.

---

## 9. References

### 9.1 Vendor product documentation (2024-2026)

- **Slack** — slack.com/help. 2024 product principles: "Single
  conversation per channel; thread-replies for off-topic". 2024
  feature: Slack Lists. 2025 feature: Slack AI Recap. Reference for
  channel UX + threads + reactions + slash-commands.
- **Microsoft 365 / Teams** — learn.microsoft.com/microsoft-365 +
  learn.microsoft.com/microsoftteams. Microsoft 365 Admin Center
  design guidelines; Teams retention + eDiscovery via Purview;
  Channels + chats UX. Microsoft Build 2024 + 2025 keynotes on
  Copilot integration.
- **Google Workspace** — workspace.google.com + support.google.com/a.
  Google Vault legal hold; Drive shared drives; Docs collaborative
  editing; Calendar working hours + working location. Google Cloud
  Next 2024 + 2025 announcements on Workspace AI.
- **Notion** — notion.so/help + notion.com/product/database. Block-
  based document model; database views (table, board, calendar,
  gallery, timeline); embed semantics; AI synthesis.
- **Salesforce** — trailhead.salesforce.com + developer.salesforce.com.
  Salesforce design guidelines (Lightning Design System); Customer
  Community Cloud; AppExchange marketplace model.
- **Workday** — workday.com/en-us/products/human-capital-management.
  Workday HCM design guidelines; absence + comp + perf + onboarding
  modules.
- **Asana** — asana.com/guide + developers.asana.com. Asana product
  principles 2024 ("Clarity, accountability, focus"); My Tasks; Goals.
- **Monday.com** — monday.com/p/work-management + developer.monday.com.
  Boards + workdocs; custom automations.
- **ClickUp** — help.clickup.com. ClickUp hierarchy (workspace, space,
  folder, list, task); custom statuses + fields.
- **ServiceNow** — docs.servicenow.com. ITSM workflows; CMDB; agent
  workspace. 2024 Now-Assist Gen AI announcements.
- **SAP SuccessFactors** — help.sap.com/docs/SAP_SUCCESSFACTORS_FOUNDATION.
  Talent management + comp + perf modules.
- **Greenhouse** — greenhouse.io/customers. Recruiting workflows +
  candidate experience.
- **Lattice** — help.lattice.com. Performance management + 1:1 +
  goals + comp module (2024 launch).
- **Carta** — carta.com/resources. Equity management + cap-table +
  vesting + 409A.
- **Discord** — discord.com/developers/docs. Channels + threads at
  scale; community moderation.
- **Linear** — linear.app/docs. Linear method (opinionated workflow);
  keyboard-first UX; cycles + initiatives.

### 9.2 Standards + protocols

- **RFC 5321** — Simple Mail Transfer Protocol.
- **RFC 5322** — Internet Message Format.
- **RFC 8314** — TLS over IMAP / POP / SMTP.
- **RFC 6376** — DomainKeys Identified Mail (DKIM).
- **RFC 7489** — Domain-based Message Authentication, Reporting, and Conformance (DMARC).
- **RFC 5545** — iCalendar (calendar event format).
- **RFC 5546** — iTIP (calendar interoperability transport).
- **RFC 6047** — iMIP (calendar invitations via mail).
- **RFC 4791** — CalDAV.
- **RFC 4918** — WebDAV.
- **RFC 9420** — Messaging Layer Security (MLS) for E2E meetings.
- **WCAG 2.2** — W3C Web Content Accessibility Guidelines.
- **EN 301 549** — EU public-sector accessibility directive.
- **KS X 6906** — KR accessibility standard.
- **JMAP Core** — RFC 8620; JMAP Mail — RFC 8621.
- **Matrix Client-Server r0.6.1** — federated messaging spec.

### 9.3 Regulatory frameworks

- **GDPR** — Regulation (EU) 2016/679. Articles 12, 17, 20 cited in
  multiple stories.
- **KR PIPA** — 개인정보 보호법 (Personal Information Protection Act).
  Article 36 erasure equivalent.
- **KR-FSS regulatory framework** — 금융감독원 supervision; 전자금융거래법
  (Electronic Financial Transactions Act).
- **HIPAA** — 45 CFR Part 160 + 164. Privacy + Security Rule.
- **SOC 2 Type II** — AICPA Trust Service Criteria.
- **ISO 22301** — Business continuity management systems.
- **ISO 27001** — Information security management.
- **EU AI Act** — Regulation (EU) 2024/1689. Risk tiers cited in
  several stories.
- **SEC 17a-4(f)** — broker-dealer record retention.
- **FINRA 4511** — books and records.
- **FRCP 37(e)** — failure to preserve ESI; legal hold authority.

### 9.4 Internal oyatie sources

- **ADR-0242** — `oyatie`-is-a-tenant doctrine. Foundational.
- **ADR-0243** — Cedar as universal gate.
- **ADR-0244** — Tenant as universal scoping primitive.
- **ADR-0218** — Tenant granular control surface.
- **ADR-0131** — Per-microservice flat layout.
- **ADR-0132** — No-suite forward policy.
- **ADR-0135** — Connect unbundle (parallel session).
- **ADR-0238** — Connect dual-context (parallel session).
- **ADR-0064** — Canonical-base + localization overlay.
- **ADR-0240** — Sovereign cloud per regional pack.
- **PRD-mail** — microservices/mail/PRD.md.
- **PRD-messenger** — microservices/messenger/PRD.md.
- **PRD-community** — microservices/community/PRD.md.
- **PRD-workflow-studio** — microservices/workflow-studio/PRD.md.
- **PRD-calendar** — microservices/calendar/PRD.md.
- **PRD-meet** — microservices/meet/PRD.md.
- **PRD-drive** — microservices/drive/PRD.md.
- **PRD-docs** — microservices/docs/PRD.md.
- **PRD-sheets** — microservices/sheets/PRD.md.
- **PRD-slides** — microservices/slides/PRD.md.
- **PRD-tasks** — microservices/tasks/PRD.md.
- **PRD-forms** — microservices/forms/PRD.md.
- **PRD-plugin-app-store** — microservices/plugin-app-store/PRD.md.

---

## 10. Uncertainties + Open Questions

1. **HR / Payroll / Compensation as a µservice vs composed surface.** The
   compendium treats HR/Payroll/Comp as a composed surface (built atop
   tasks, forms, drive, docs, sheets, workflow-studio, calendar). This
   matches ADR-0132 (no-suite-forward-policy). The open question is
   whether to promote `microservices/hr/` to a first-class µservice once
   the Ontology `Employee` object type is concrete (likely post-M03).
2. **Workplace-integration µservice.** `docs/products/workplace-integration/`
   exists as a directory but its PRD is not yet published. Stories that
   reference cross-surface workplace integration assume the per-surface
   µservices interoperate via Workflow events + Ontology reads as the
   integration substrate. If workplace-integration crystallises as its own
   µservice, this compendium will be revised.
3. **Per-customer plugin governance vs per-org plugin governance.** Stories
   PLG-001 + ADM-001 assume Hiroshi (IT admin) governs at the org level.
   The Cedar policy model supports per-team + per-principal overrides; the
   compendium presumes a sensible default of "org-level baseline; per-
   team explicit-grant for higher-risk plugins". This may be tightened.
4. **Voice + dictation surface ownership.** A11Y-002 (Gabriela voice
   dictation) crosses mail + messenger + calendar. The local voice model
   sits in intelligence µservice; the surfacing UX should be a unified
   "compose with voice" affordance. The owning team for that UX is open.
5. **External-collaboration model vs partner-tenant assume-role.** MSG-012
   + DRV-012 + MAIL-011 use partner-tenant assume-role. There is a parallel
   "guest user" model (single-tenant external invitee per channel). The
   compendium prefers assume-role for repeat external collaboration; guest
   user for one-off. The boundary needs explicit ADR.
6. **HR module → payroll integration depth.** HR-003 + HR-009 + HR-012
   assume a payroll execution layer (bank transfer initiation, per-
   jurisdiction tax form generation). The compendium treats this as a
   per-jurisdiction integrations layer; whether oyatie ships first-party
   payroll engines or integrates with vendors (Justworks, Workday, 더존,
   DATEV) per jurisdiction is open.
7. **Customer-shared spaces semantics.** COM-005 + XS-001 + XS-009 imply
   customer-shared community + drive spaces. The exact data-isolation
   pattern (one tenant with external invitees vs federated cross-tenant)
   needs follow-up ADR.
8. **EU AI Act tier alignment.** Multiple stories invoke EU AI Act tier
   T0/T1/T2 (e.g., WFS-005, MAIL-015). Per-feature tier assignment per
   ADR-0144 / intelligence µservice catalog is partially complete; final
   tier alignment per feature is in progress.
9. **Cross-pack assume-role.** Leo's assume-role (MSG-012) is presented as
   straightforward; cross-pack assume-role (Leo in US, acme-corp tenant
   in EU pack) has residency + Cedar + identity nuances per ADR-0240.
   The compendium presumes same-pack assume-role unless explicit cross-
   pack permission is granted.

---

*End of B2B Work Surfaces — User Stories Compendium.*
