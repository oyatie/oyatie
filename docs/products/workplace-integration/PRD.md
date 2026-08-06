---
doc_class: ProductRequirements
template_id: TPL-PRD
prd_id: PRD-workplace-integration
product: workplace-integration
status: Draft
date: 2026-05-20
owner: axis-product + axis-workflow + axis-application-shell + axis-identity + axis-tenancy + axis-compliance
sales_segment: cross-cutting-product-layer
tier: product-layer-cross-cutting
milestone_first_ship: M04-workplace-integration-foundation
related_oyatie_adrs:
  - ADR-0009
  - ADR-0131
  - ADR-0132
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0251
  - ADR-0252
  - ADR-0255
  - ADR-0263
  - ADR-0316
related_microservices:
  - workflow-engine
  - workflow-studio
  - calendar
  - meet
  - mail
  - messenger
  - drive
  - intelligence
  - policy-engine
  - audit-chain
  - tenancy
  - identity
  - ontology
  - plugin-app-store
tenant_class: ["evaluation_limited", "paid"]
related_adrs:
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0028-cloud-microservice-architecture
  - ADR-0099-data-class-registry
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0106-application-to-usecase-rename
  - ADR-0117-data-residency-jurisdiction-code
  - ADR-0128-hyperscaler-architecture-invariants
  - ADR-0131-per-microservice-flat-layout
  - ADR-0132-product-platform-and-bundle-dissolution
  - ADR-0139-agentic-slo-gated-promotion
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0148-service-mesh-cilium
  - ADR-0150-cedar-policy-engine
  - ADR-0174-sustainability-tag
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission
  - ADR-0211-in-house-tech-stack-preference
  - ADR-0218-tenant-granular-control-surface
  - ADR-0240-sovereign-cloud-per-regional-pack
  - ADR-0241-dr-business-continuity-portfolio-policy
  - ADR-0242
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0246-policy-engine-substrate-promotion
  - ADR-0247-self-hosting-self-modification-doctrine
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0252-workflow-engine-per-step-idempotency
related_specs:
  - /specs/products/workplace-integration.json
  - /specs/microservices/workflow-engine.json
  - /specs/microservices/workflow-studio.json
  - /specs/microservices/calendar.json
  - /specs/microservices/meet.json
  - /specs/microservices/mail.json
  - /specs/microservices/messenger.json
  - /specs/microservices/drive.json
  - /specs/microservices/intelligence.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/identity.json
  - /specs/microservices/ontology.json
  - /specs/microservices/plugin-app-store.json
  - /specs/per-microservice-flat-layout.json
  - /specs/agentic-slo-gated-promotion.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_substrate_vs_product_layering
  - feedback_workflow_studio_scope
  - feedback_workflow_is_shared
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_autonomous_implementation_artifacts
  - feedback_canonical_base_localization
  - feedback_doc_coverage_enforced
  - feedback_no_silent_regression
owner_team: axis-product + axis-workflow + axis-application-shell + axis-identity + axis-tenancy + axis-compliance
doc_status: draft_target_non_claim
---

# PRD: Workplace Integration — Cross-Cutting Product Layer

> **Status:** published; wave-gated promotion targets remain in §2.4
> **Owning team:** founder-governed product authority (with axis-workflow as primary executor)
> **Owning axis:** cross-cutting product layer (NOT a single µservice)
> **Catalog reference:** `oya/workplace-integration/catalog/oya-workplace-integration-application.yaml` plus layer catalog entries under `oya/workplace-integration/catalog/`
> **Last updated:** 2026-05-20 by founder-governed product authority
> **Path convention:** repo-local service artifacts use `oya/<service>/...`; machine-readable service specs use `specs/microservices/*.json`; legacy `microservices/...` paths are not authoritative in this checkout.

---

## 1. Purpose

Workplace Integration is the **connective tissue** that turns oyatie's suite of independent first-party B2B µservices — Mail, Messenger, Calendar, Meet, Drive, Notes, Tasks, Forms, Workflow Studio, Workflow Engine, HR, Payroll, Compensation (promotion-gated), Plugin App Store, Sites, Recordings, Sheets, Slides, Docs, Comms-Email, Audit-Chain, Tenancy, Identity, Ontology, Intelligence — into a **single coherent workplace platform** that competes head-to-head with the combined offerings of:

- Microsoft 365 (Outlook + Teams + SharePoint + OneDrive + Power Automate + Viva)
- Google Workspace (Gmail + Chat + Meet + Calendar + Drive + Docs + Apps Script)
- Notion (docs + teamspaces + projects + databases + automations)
- Slack (chat + workflow builder + canvas + huddles)
- ServiceNow (ITSM + employee workflows + approvals + custom apps)
- Workday (HR + payroll + benefits + talent + analytics)
- Concur (expense + travel)
- DocuSign / Adobe Sign (e-signing)
- BambooHR / Rippling / ChartHop (HRIS)
- Expensify / Brex / Ramp (expense management)
- Greenhouse / Lever / Lattice (onboarding + performance)
- Calendly / x.ai (meeting scheduling)

Workplace Integration is **not a single µservice**. Per ADR-0245 substrate-vs-product layering, it is a **cross-cutting product layer** — a coherent end-user product experience composed of orchestrated capabilities from many µservices. It owns:

1. **The end-user workplace flows** (clocking in, approvals, e-signing, meetings, expenses, leave, performance reviews, onboarding, offboarding, travel, procurement, announcements, project tasks, compliance training).
2. **The durable saga definitions** that orchestrate each flow across µservices, registered with `oya/workflow-engine/`.
3. **The cross-µservice integration contracts** (event schemas, ontology object types, Cedar policy fragments, audit emission contracts) that bind these flows together.
4. **The cross-µservice UX surfaces** (rich Messenger cards, mail templates, mobile mini-apps, voice-trigger handlers, Workflow Studio templates) that expose each flow to end users.
5. **The competitor-parity feature matrix** that benchmarks oyatie's workplace experience against Microsoft 365 / Google Workspace / Notion / Slack / ServiceNow / Workday / Concur / etc.

Per ADR-0242 (`oyatie`-is-a-tenant doctrine), Workplace Integration's flows are first dogfooded inside the `oyatie` tenant — every oyatie engineer clocks in via this layer, every oyatie vacation request flows through it, every internal ADR e-signature uses it — before being offered to customer tenants. This forces every flow to production-grade quality before external GA.

### 1.1 Why "workplace integration" is necessary as a distinct product layer

The standalone µservices solve their own problems: Mail delivers mail, Calendar schedules events, HR records employment data. **But a vacation request is not a "Calendar event" — it is a multi-step workflow that touches HR (balance check, record creation), Calendar (out-of-office block + team coverage check), Messenger (approve/deny card), Mail (notification), Audit-Chain (compliance), Tenancy (jurisdiction-aware policy), Workflow Engine (durable orchestration), and Workflow Studio (visual edit by HR power user)**. No single µservice owns the flow; the flow exists in the seams *between* µservices.

Microsoft, Google, Workday, ServiceNow each succeed not because any individual product (Outlook, Gmail, Workday Recruiting, ServiceNow Incident) is best-in-class on its own dimensions, but because the platform-level integration is coherent enough that an employee can complete a vacation request without context-switching between four apps and three approval portals. **That platform-level coherence is what Workplace Integration owns.**

### 1.2 Why cross-cutting and not "one big workplace µservice"

Per ADR-0131 (per-microservice flat layout) + ADR-0132 (no-grouping forward-policy): a "workplace" µservice that bundles HR + payroll + e-sign + expense + scheduling would violate the no-grouping rule. The flat-µservice rule requires every concern to live in its own µservice that does one thing well. Workplace Integration therefore manifests as:

- **Per-µservice ownership** of each concern (HR owns HR-records, Calendar owns Calendar-events, Workflow-Engine owns saga orchestration, etc.).
- **A workplace-integration product layer** that authors saga specs (workflow_spec.v1.json artefacts), event contracts, and competitor-parity templates, registers them with the relevant µservices, and packages the UX surfaces.
- **The Application Shell** (per ADR-0131 IP-M01-MIGR-008; `oya/application/`) hosts the consolidated end-user surfaces as embedded views.

The product layer is documented HERE (this PRD) and IMPLEMENTED across the relevant µservices via per-flow IPs (Implementation Plans). Each saga's workflow_spec.v1.json + ontology object types + Cedar policy fragments + Workflow Studio templates live under `oya/workflow-engine/specs/workplace-integration/<flow-id>/` and `oya/workflow-studio/templates/workplace-integration/<flow-id>/`.

### 1.3 Competitive thesis

oyatie's workplace integration wins by:

- **Suite coherence on a sovereign substrate** — Microsoft 365 + Google Workspace are non-sovereign for KR / EU / regulated industries; oyatie is sovereign-per-pack per ADR-0240.
- **Workflow Studio as the cross-product authoring substrate** — Notion's automations + Slack workflow builder + Microsoft Power Automate are each siloed within their parent suite; Workflow Studio (per `feedback_workflow_studio_scope`) authors workflows across every oyatie µservice including HR / Payroll / Compensation / Mail / Drive / Calendar / Meet / Messenger / Plugin-App-Store.
- **First-class Plugin App Store** — every tenant can extend any workplace flow via marketplace plugins, gated by Cedar (per ADR-0243) and signed by the Plugin App Store substrate.
- **AI-native by default** — Intelligence substrate (per ADR-0255 + the `substrate-ai` tier) is consumed by every workplace flow for receipt OCR, leave-balance reasoning, contract field detection, meeting summarisation, and policy-violation explanation.
- **Compliance-by-construction** — every workplace flow emits to audit-chain (ADR-0028) under per-pack retention floors (ADR-0251) and per-jurisdiction Cedar gates (ADR-0243 + ADR-0244).

---

## 2. Scope

### 2.1 In-scope workplace flows

Workplace Integration owns thirteen first-class flows (A–N) plus a long tail of derived flows. Each flow is a durable saga registered with `oya/workflow-engine/`. The thirteen first-class flows:

| Flow ID | Name | Primary persona | Saga ID | Wave |
|---|---|---|---|---|
| **A** | Clocking In / Out | Employee | `ClockingInSaga`, `ClockingOutSaga` | M04 preview |
| **B** | Vacation / Leave Approval | Employee + Manager | `LeaveRequestSaga` | M04 preview |
| **C** | E-Signing (Contracts, NDAs, Offers, HR Docs) | HR + Signer(s) | `ESignSaga` | M04 preview |
| **D** | Meeting Scheduling | Employee + Attendees | `MeetingScheduleSaga` | M04 preview |
| **E** | Expense Report + Reimbursement | Employee + Manager + Finance | `ExpenseSaga` | M04 stable |
| **F** | Onboarding (Employee) | HR + IT + Manager + New Hire | `OnboardingSaga` | M04 stable |
| **G** | Offboarding (Employee) | HR + IT + Manager + Departing Employee | `OffboardingSaga` | M04 stable |
| **H** | Performance Review | Manager + Employee + HR | `PerformanceReviewSaga` | M04 stable |
| **I** | Travel Request | Employee + Manager + Travel-Desk | `TravelRequestSaga` | M05 |
| **J** | Procurement / Purchase Order | Requester + Approver(s) + Finance + Vendor | `ProcurementSaga` | M05 |
| **K** | Internal Announcement | Comms + Audience | `AnnouncementSaga` | M04 preview |
| **L** | Project / Task Management | Project lead + Contributors | `ProjectTaskSaga` | M04 stable |
| **M** | Compliance Training Assignment + Completion | HR + Compliance + Employee | `ComplianceTrainingSaga` | M05 |
| **N** | Document Collaboration (Review + Sign-Off) | Author + Reviewer(s) | `DocumentCollaborationSaga` | M04 stable |

### 2.2 Derived flows (long-tail, M05+)

- Time-off-in-lieu accrual + redemption
- Overtime approval (jurisdictional — KR labor act 1.5x cap; US FLSA exempt vs non-exempt)
- Sick leave (separate balance pool; doctor's-note evidence flow)
- Parental leave (long-running saga; payroll integration)
- Bereavement leave (compassionate fast-path; bypass manager approval)
- Sabbatical request (multi-month durable saga)
- Remote-work request (location change; tax-jurisdiction implications)
- Equipment-issue request (procurement sub-flow)
- Stock-option exercise (compensation µservice integration)
- 1:1 meeting scheduling (calendar sub-flow with auto-agenda)
- All-hands scheduling (announcement + meeting sub-flow)
- Visitor pre-registration (facility + identity integration)
- Conference-room booking with catering (calendar + procurement sub-flow)
- Expense pre-approval (above per-tenant threshold; expense sub-flow)
- Reimbursement payout (payroll integration; expense sub-flow)
- Birthday + work-anniversary auto-announcement
- New-hire welcome announcement (onboarding sub-flow)
- Departure announcement (offboarding sub-flow)
- Org-chart-change announcement (HR sub-flow)
- Goal-setting (OKR / SMART) + check-in (performance sub-flow)
- 360-degree feedback collection (performance sub-flow)
- Calibration session (performance sub-flow; manager-cohort)
- Promotion request (compensation sub-flow)
- Compensation review (compensation sub-flow)
- Bonus distribution (compensation sub-flow)
- Time-sheet submission + approval (timekeeping sub-flow; consulting tenants)
- Project timesheet → invoice (timekeeping + finance integration)
- Procurement vendor onboarding (procurement sub-flow; KYC + tax-ID)
- Vendor invoice approval (procurement sub-flow)
- Contract renewal reminder (e-sign + procurement sub-flow)
- Document retention sunset notification (compliance + drive sub-flow)
- Access-review (identity + compliance sub-flow; SOX, ISO 27001)
- DSAR-response coordination (consent-graph + workflow sub-flow)
- Whistleblower / ethics report (anonymous sub-flow; EU Whistleblowing Directive)
- Workplace safety incident report (compliance sub-flow)

### 2.3 Out-of-scope (anti-scope)

The following are explicitly out-of-scope for the Workplace Integration product layer:

1. **End-user payment processing** — handled by the reserved `payments` µservice (ADR-0245 D-3.D); workplace flows that need payment routing (expense reimbursement, vendor payment) emit events to the payments substrate post-certification.
2. **Tax filing engines** — handled by the reserved `tax-engine` µservice.
3. **Identity verification (KYC) for external counterparties** — handled by the reserved `identity-verification` µservice.
4. **Brand-surface UI authoring** — owned by `oya/application/` (Application Shell); workplace-integration provides the embedded views + state, not the shell.
5. **Per-µservice substrate concerns** — Mail's IMAP/JMAP protocol, Calendar's RFC 5545 parser, Meet's LiveKit SFU, Workflow Engine's state machine — all owned by the respective µservice; workplace-integration consumes their stable contracts.
6. **Country-by-country labor law interpretation** — owned by the `oya/compliance/` substrate (per-pack overlay); workplace-integration consumes the resolved policy fragment at Cedar evaluation time.
7. **HR record-of-truth schema authoring** — owned by `oya/hr/` (reserved-then-promoted; see §5.4) and ontology object types `Employee`, `LeaveBalance`, `EmploymentRecord`.
8. **General-purpose document storage** — owned by `oya/drive/`; workplace-integration documents are stored in drive with retention policy refs.

Promotion of any anti-scope item to in-scope requires a founder-/governance-recorded decision in this PRD's §12 decision log.

### 2.4 Wave gating

Per ADR-0139 agentic SLO-gated promotion:

- **Preview** (M04 preview): Flows A, B, C, D, K (clocking, leave, e-sign, meeting, announcement) ship to `oyatie` tenant only. SLO target: 99.5% saga completion rate per (tenant, flow) per 30d window. Used by oyatie engineers for real internal flows.
- **Stable** (M04 stable): Flows E, F, G, H, L, N (expense, onboarding, offboarding, performance, project-task, document-collab) added. Flows A–D + K hardened to 99.9% saga completion + competitor-parity matrix complete. Selected design-partner customer tenants admitted.
- **GA** (M05): Flows I, J, M (travel, procurement, compliance-training) added. Full long-tail enabled. All flows at 99.95% saga completion. Customer tenants admitted by default.

---

## 3. Architectural model

### 3.1 Layering

Workplace Integration is a **cross-cutting product layer** on top of substrates and product µservices. The composition:

```
+--------------------------------------------------------------+
|         Workplace Integration product layer (THIS PRD)       |
|  - flow specs (workflow_spec.v1.json) for sagas A-N          |
|  - ontology object types (LeaveRequest, ExpenseReport, ...)  |
|  - Cedar policy fragments (per-flow + per-jurisdiction)      |
|  - Workflow Studio templates (per-flow visual starters)      |
|  - mail/messenger card templates                             |
|  - mobile mini-app screens                                   |
|  - voice-trigger handlers (Siri / Google Assistant)          |
|  - competitor-parity feature matrix                          |
+--------------------------------------------------------------+
                              |
                              v
+--------------------------------------------------------------+
|             Product µservices (tier=product)                 |
|  application (shell)   workflow-studio   plugin-app-store    |
|  mail        messenger calendar  meet    drive               |
|  docs        sheets    slides    notes   tasks    forms      |
|  recordings  hr*       payroll*  compensation*               |
|  finops-portal         feature-flags     analytics           |
+--------------------------------------------------------------+
                              |
                              v
+--------------------------------------------------------------+
|             Substrate µservices (tier=substrate)             |
|  workflow-engine      intelligence       ontology            |
|  policy-engine        audit-chain        consent-graph       |
|  identity             tenancy            compliance          |
|  governance           cell               comms-email         |
|  cloud-secrets        cloud-iac          cloud-k8s           |
|  network              observability      api-gateway         |
+--------------------------------------------------------------+
```

*Asterisks (\*) denote currently-reserved µservices to be promoted out of `reserved` per the HR/Payroll/Compensation promotion ADRs (see §5.4).*

### 3.2 The orchestration triangle

Every workplace flow is structured as a triangle of three substrate concerns:

1. **Workflow Engine (durable orchestrator)** — owns the saga lifecycle. Receives the trigger event, advances through steps, persists state, retries idempotently per ADR-0252, handles compensation on failure, and seals the run to audit-chain.
2. **Ontology (canonical data)** — owns the entity types that the flow reads + writes. `LeaveRequest`, `ExpenseReport`, `EmploymentRecord`, `Signature`, `MeetingScheduleProposal`, `OnboardingChecklist`, etc.
3. **Policy Engine (Cedar gate)** — owns the per-step authorisation, per-tenant policy overlay, per-jurisdiction labor-law overlay, per-step `data_class` enforcement.

Plus three delivery surfaces:

4. **Mail + Messenger + Calendar + Meet + Drive + Application Shell + Mobile** — the UX delivery channels.
5. **Audit-Chain** — every state-change emits a Merkle-sealed record per ADR-0028.
6. **Plugin App Store** — third-party extensions to any flow (e.g., a tenant installs "Concur-style expense pre-approval" plugin to extend the ExpenseSaga).

### 3.3 Saga authoring convention

Every flow's saga is authored as a `workflow_spec.v1.json` document (per `workflow-engine` PRD + Bominal ADR-0164). The workplace-owned source bundle in this checkout is the concrete doc/contract set under `oya/workplace-integration/` plus this PRD; downstream Workflow Engine and Workflow Studio service lanes consume that bundle through the acceptance contract below before any flow can be promoted.

Required artifact contract per promoted flow:

```
workplace-owned source bundle
├── docs/products/workplace-integration/PRD.md         # product-layer requirements and flow acceptance
├── oya/workplace-integration/contracts/               # OpenAPI, AsyncAPI, proto surfaces
├── oya/workplace-integration/policies/                # Cedar policy fragments owned by this layer
├── oya/workplace-integration/runbooks/                # operator recovery evidence
├── oya/workplace-integration/IP-journey-<id>-*.md     # implementation IPs when present
└── docs/user-journeys/j<id>-*/                        # story, UX, handshake, and test-plan context

service-owned import targets required for promotion
├── oya/workflow-engine/.../saga.workflow_spec.v1.json # durable saga definition
├── oya/workflow-engine/.../ontology-bindings.json     # object-type reads + writes
├── oya/workflow-engine/.../events-*.asyncapi.yaml     # workflow events emitted/consumed
├── oya/workflow-engine/.../audit-emission-contract.*  # audit-chain contract
└── oya/workflow-studio/.../template.workflow_spec.v1.json # editable Studio starter
```

A flow is not promotion-eligible until both the workplace-owned source bundle and the service-owned import targets resolve in the repository and pass their owning lanes' contract checks. This PRD therefore treats absent service-owned import targets as promotion blockers, not as completed artifacts.

### 3.4 Inter-µservice communication

Per ADR-0145 inter-µservice communication reform:

- Direct gRPC + Cedar gate + audit + tracing — for per-call reads (e.g., HR balance check before LeaveRequestSaga decides).
- Workflow event emission via Workflow Engine event-bus — for fan-out (e.g., `LeaveApproved` event consumed by Calendar to block OOO, by Mail to notify, by audit-chain to seal).
- Ontology projection — for cross-µservice entity reads where strong-consistency isn't required.

The workplace integration layer **does not** use point-to-point hardcoded couplings. Every flow is event-driven; every step is durable; every transition is policy-gated.

### 3.5 Tenancy + sub-scope handling

Per ADR-0242 + ADR-0244:

- Every flow carries a `tenant_id` + optional `sub_scope` (e.g., `tenant-acme.engineering.team-platform`).
- Sub-scopes inherit parent policy unless explicit override.
- The `oyatie` tenant uses sub-scopes like `oyatie.engineer.<id>` for engineer principals; their workplace flows (vacation, e-sign, expense) work identically to a customer tenant's `tenant-acme.employee.<id>`.
- Per-jurisdiction labor-law overlay is selected by the tenant's `jurisdiction_code` per ADR-0117 + ADR-0240.

### 3.6 Workflow Studio as the visual authoring surface

Per `feedback_workflow_studio_scope` (Workflow Studio is the n8n-class first hero product covering multi-domain workflows including business/HR/healthcare/supply-chain/delivery):

- Every promoted saga A–N must resolve a corresponding Workflow Studio template in the Workflow Studio service lane before promotion; absent templates are promotion blockers, not completed artifacts.
- Tenant power users (HR, finance, ops) can open Studio, load the template, customise it (e.g., add a second approval level, change the policy threshold, append a Slack-equivalent notification), and save as their tenant-specific workflow.
- The customised workflow is registered with workflow-engine's spec-store; subsequent triggers run the tenant's variant rather than the default template.
- Round-trip byte-equality between Studio canvas and workflow_spec.v1.json is the load-bearing invariant (per workflow-studio PRD AC-02).

### 3.7 Plugin App Store extension points

Per `oya/plugin-app-store/`:

- Every flow declares **extension points** in its saga spec (e.g., `LeaveRequestSaga` declares extension points `pre-approval`, `post-approval`, `policy-evaluator`, `notification-recipient`).
- Plugins from the Plugin App Store register handlers at extension points (e.g., a "Concur integration" plugin registers as `expense.post-approval.export-to-concur`).
- Plugin handlers are gated by Cedar (the plugin's per-tenant install must grant the corresponding action permit), executed in Wasmtime sandbox per ADR-0037, and time-bounded (default 30s; configurable to 5min for long-running plugin ops).

---

## 4. Flow specifications

This section specifies every flow A–N in full step-by-step detail, including trigger conditions, saga steps, edge cases, UX, and per-jurisdiction compliance.

### Flow A — Clocking In / Out (TimeTracking)

#### A.1 Purpose

Records employee arrival + departure times to the HR µservice for: timesheet generation, overtime calculation, payroll feed, attendance audit (KR labor act 근로기준법 Article 50 — managerial attestation), and labor-law compliance (US FLSA, EU Working Time Directive, KR Labor Standards Act, JP Labor Standards Act).

#### A.2 Trigger surfaces (any of)

- **Mobile app one-tap clock-in button** on the home screen (primary, ≥ 70% of clock-ins).
- **`/in` slash command in #attendance Messenger channel** (secondary; works from desktop chat).
- **Reply "in" or "clock in" to morning attendance mail** sent at start-of-shift T-15min.
- **Tap NFC time clock** at office entrance (per-tenant physical hardware; optional).
- **Voice trigger via Siri ("Hey Siri, clock me in")** or Google Assistant ("Hey Google, clock me in at oyatie").
- **Passive geofence trigger** when employee's mobile device enters the office geofence (consent-gated per tenant policy; disabled by default; KR PIPA + EU GDPR explicit-consent required).
- **Browser extension click** for desk-bound workers using web-only flow.

#### A.3 Saga: `ClockingInSaga`

Saga source for the current workplace-owned slice is `oya/workplace-integration/IP-journey-j37-clock-in-geofence.md`; its Workflow Engine import target must resolve before promotion. Steps:

1. **`receive_trigger`** — accept signed trigger event from one of A.2 sources; validate signature; extract `(tenant_id, employee_id, trigger_source, trigger_timestamp)`.
2. **`resolve_employee`** — gRPC call to `hr` µservice `GetEmploymentRecord(employee_id)`; verify employment status is `active` (or `on_probation`); fetch `assigned_work_schedule`. Cedar gate: `action == "ClockIn", principal == employee_id, resource == EmploymentRecord`.
3. **`capture_geolocation`** — if tenant policy permits + employee consented, capture device GPS coords (lat, lng, accuracy_m); attach to the saga state. Skip if consent withheld.
4. **`capture_device_fingerprint`** — hash device-id + IP + user-agent into a stable fingerprint; attach to saga state. Used for anti-spoofing (anti-buddy-punching).
5. **`validate_within_work_hours`** — compute the difference between trigger_timestamp and assigned shift start. Within ±15min of shift start → on-time. Earlier than -15min → early; later than +15min → late. Outside ±60min → suspicious; route to `step_anomaly_review`.
6. **`validate_geofence`** — if tenant defines work-area geofences (GPS-bounded), check the captured geolocation lies inside the geofence. Outside → route to `step_outside_geofence_handler`.
7. **`record_timesheet_entry`** — gRPC call to `hr` µservice `CreateTimesheetEntry({employee_id, timestamp, kind: "clock_in", source, geolocation, device_fingerprint, lateness_status})`. Cedar gate: `action == "WriteTimesheet"`. Idempotency key: SHA-256 of `(employee_id, date, kind)` (per ADR-0252; one clock-in per employee per day per kind).
8. **`emit_audit_chain`** — emit `AttendanceClockIn` event to `audit-chain` µservice with full state; Merkle-sealed within 1s per audit-chain SLO.
9. **`escalate_if_late`** — if `lateness_status == late` and tenant policy escalates lateness → publish `LatenessEscalation` event consumed by employee's manager via Messenger card.
10. **`send_confirmation`** — Messenger card to employee: "Clocked in at 09:14 KST. Have a great day! [Adjust] [Report Issue]". Mobile push notification. Optional voice confirmation if voice-triggered.
11. **`schedule_clock_out_reminder`** — schedule a timer for shift-end - 5min; on fire, send `ClockOutReminder` to employee.

Idempotency: every step keyed per (saga_run_id, step_id, idempotency_key). Retries safe per ADR-0252.

#### A.4 Edge cases

- **Late arrival (>15min after shift start)** → mark `late`; emit `LatenessEvent` to manager. Per-tenant policy may auto-deduct from PTO balance (KR-jurisdiction-pack: requires explicit employee consent per 근로기준법 Art 43 — wages can't be deducted without written agreement).
- **Early arrival (>15min before shift start)** → allow but flag for overtime computation if tenant pays for early work.
- **Outside work geofence** → if tenant policy is strict-fence → refuse clock-in with explanation ("You appear to be 2.3km outside your assigned work area. Contact your manager if remote-work approved."). If tenant policy is soft-fence → record clock-in with `outside_geofence: true` flag for manager review.
- **Missing clock-out (employee forgot to clock out)** → at end-of-shift + 60min, auto-clock-out with `auto_clocked_out: true` flag; emit `AutoClockOut` event to employee + manager Messenger; allow employee to correct via Messenger card ("Your shift was auto-closed at 18:60. Tap to adjust if you worked late.").
- **Duplicate clock-in (same day, same kind)** → idempotency key catches; second attempt returns `AlreadyClockedIn` with the existing timestamp.
- **Clock-in from non-work device (corp policy violation)** → if tenant requires corp-managed devices for attendance, refuse non-managed-device clock-in; emit `PolicyViolation` event to security.
- **Buddy-punching attempt detected** (same device fingerprint clocking in multiple employees) → escalate to security review; block subsequent clock-ins from that device pending investigation.
- **Holiday clock-in** (employee clocked in on a holiday) → emit `HolidayWorkRecord` event with overtime multiplier per jurisdiction (KR labor act: 50% premium for holiday work; EU varies by member state; US FLSA only premium if non-exempt + >40h/week).
- **Shift-swap pending** (employee on covering shift) → resolve via `hr` µservice's `ShiftSwapResolver` before recording.

#### A.5 UX details

- **Mobile**: large tap target (≥ 88pt × 88pt per WCAG 2.5.5 Target Size); single tap if location-permitted; biometric confirmation (FaceID/TouchID) optional for high-security tenants; visual confirmation animation (≤ 800ms); haptic feedback on success.
- **Messenger card**: structured action button with employee-name + timestamp + adjust + report-issue actions; per Slack Block Kit / Microsoft Adaptive Card spec.
- **Voice trigger**: Apple App Intents (iOS 18+) + Android App Actions; intent `oyatie.attendance.clock_in` registered with on-device intent dispatcher; works hands-free during commute.
- **Passive geofence**: opt-in only; UX shows clear indicator when active ("Auto-clock-in is on") + tap-to-disable; battery-friendly using significant-location-change API not continuous-GPS.
- **Confirmation**: < 500ms p99 from tap to confirmation visible; if longer, optimistic UI (show "Clocked in" immediately, retry in background, surface error only on hard fail).

#### A.6 Compliance (per-jurisdiction)

| Jurisdiction | Statute / regulation | Applied rule |
|---|---|---|
| KR | 근로기준법 (Labor Standards Act) Article 50 — work-hour definition + Article 53 — overtime cap | record exact clock-in/out; 40h/week regular + 12h/week overtime cap; refuse clock-in beyond cap with warning |
| KR | 근로기준법 Article 56 — overtime premium | 1.5x regular wage for >40h/week or >8h/day; passed to payroll |
| KR | PIPA Article 15 — consent for biometric/location | passive geofence + biometric confirmation requires explicit consent recorded in consent-graph |
| US (federal) | FLSA 29 USC §207 — overtime | non-exempt employees: 1.5x for >40h/week; exempt: no overtime; tenant employment-classification metadata determines |
| US (state) | California Labor Code §510, §512 — daily overtime + meal breaks | 1.5x for >8h/day + meal break required at >5h; passed to payroll |
| EU | Working Time Directive 2003/88/EC | max 48h/week (4-month reference period); min 11h daily rest; flag violation events |
| EU | GDPR Article 5(1)(c) — data minimisation | geolocation collected only if essential + consented |
| JP | Labor Standards Act Article 32, 36 — work hours + 36協定 | 40h/week regular + 36-agreement overtime cap |
| CN | Labor Contract Law Article 41 — overtime | 36h/month overtime cap |
| BR | CLT Article 59 — overtime | banco de horas (hour bank) or 1.5x premium |
| SG | Employment Act §38 — overtime | 72h/month overtime cap; 1.5x premium |
| IN | Factories Act §54 — work hours | 9h/day max; 48h/week; weekly off mandatory |

Each rule is encoded as a Cedar fragment in `oya/policy-engine/fragments/workplace-integration/clocking/<jurisdiction>.cedar` and applied per tenant's `jurisdiction_code`.

---

### Flow B — Vacation / Leave Approval

#### B.1 Purpose

Records and approves employee leave requests (vacation, sick, parental, bereavement, sabbatical) against per-employee balances; coordinates with calendar for team-coverage visibility; routes through manager chain for approval; emits audit-chain record for SOX-equivalent record-keeping.

#### B.2 Trigger surfaces

- **HR portal** (Application Shell embedded view) — full-form request.
- **Mail to `leave@<tenant-domain>`** — natural-language request parsed by Intelligence substrate ("I'd like to take vacation from June 5 to June 10").
- **`/leave` slash command in Messenger** — opens leave-request card with date-picker + reason field + leave-type picker.
- **Mobile mini-app screen** — vacation calendar visual picker.
- **Voice trigger** — "Hey Siri, request vacation for next Monday and Tuesday" → confirmation dialog → submit.

#### B.3 Saga: `LeaveRequestSaga`

Steps:

1. **`receive_request`** — accept request with `(employee_id, leave_type, start_date, end_date, partial_day_hours?, reason?, evidence_attachments?)`.
2. **`resolve_employee_and_policy`** — fetch employment record + applicable leave policy (per-tenant + per-jurisdiction overlay).
3. **`validate_balance`** — fetch current leave balance from `hr` µservice; compute days requested (excluding weekends + tenant holidays per `oya/calendar/` calendar-of-record). If balance < requested → route to `step_balance_insufficient_handler`.
4. **`check_eligibility`** — Cedar gate: probation employees may have limited leave; employee under 90d may not be eligible for some leave types (KR labor act); per-tenant blackout-period policy (e.g., no vacation during quarter-end finance close).
5. **`check_team_coverage`** — gRPC to `calendar` µservice `QueryTeamAvailability(team_id, [start_date, end_date])`; if >50% of team OOO during requested window → flag `team_coverage_risk: high` for manager attention.
6. **`detect_policy_violations`** — Cedar gate against per-jurisdiction overlay: KR labor act Article 60 requires employer to grant annual leave on requested dates "unless business operations are significantly hindered"; auto-deny is uncommon; flag-for-manager-review is the norm. EU varies. US: at-will employer-discretion (subject to FMLA for medical).
7. **`build_approval_chain`** — resolve manager hierarchy from HR org chart; build approval chain (employee → direct manager → [skip-level if direct manager OOO or skip-level required by tenant policy]).
8. **`notify_first_approver`** — Messenger card + Mail to first approver: "Leave request from [employee] for [dates]. Balance OK. Team coverage [%]. [Approve] [Deny] [Defer] [Adjust]". Card shows employee's leave balance, team's OOO calendar overlay, and reason if provided.
9. **`await_approval_decision`** — durable wait (saga can sleep for days/weeks); reminders sent at T+24h, T+72h, T+7d if no decision; on T+14d, auto-escalate to skip-level.
10. **`handle_decision`** — on approve: proceed to `step_update_hr`. On deny: emit `LeaveRequestDenied` to employee with reason; saga ends. On defer: schedule re-prompt at deferred time; loop back to `step_notify_first_approver`. On adjust: present adjusted dates to employee for re-confirmation.
11. **`update_hr_record`** — gRPC to `hr` µservice `CommitLeaveBalance({employee_id, leave_type, dates, approved: true, approver_id, decision_timestamp})`; idempotent per ADR-0252.
12. **`push_to_calendar`** — emit `OutOfOffice` event consumed by `calendar` µservice to create OOO block on employee's calendar + (per tenant policy) on team's shared calendar.
13. **`notify_employee`** — Messenger + Mail + Mobile push: "Your leave request for [dates] is approved by [approver]. Have a wonderful time!" with one-click adjust + cancel.
14. **`notify_backup_assignee`** — if HR record specifies backup-assignee, notify them of the coverage requirement.
15. **`emit_audit_chain`** — `LeaveApproved` event to audit-chain; Merkle-sealed; retained per pack retention floor (KR 5y per 근로기준법 Article 42; US payroll record 4y per FLSA; EU 5y typical).
16. **`schedule_pre_leave_reminder`** — T-3d before leave start, send "Your leave starts in 3 days — set OOO autoresponder?" prompt.
17. **`schedule_return_reminder`** — T+0 of return date, send "Welcome back! Update OOO autoresponder?" + auto-clear OOO calendar block.

#### B.4 Edge cases

- **Insufficient balance** → offer (a) reduce request to available balance, (b) request unpaid leave (separate sub-saga with different approval chain), (c) cancel request.
- **Conflicting team requests** (>50% of team OOO) → flag manager card "Team coverage at 60% during requested dates. Approve anyway?" with adjacent-team-member-on-leave names + their return dates.
- **Manager OOO during approval window** → after T+72h reminder unanswered, auto-escalate to skip-level (or to HR if no skip-level).
- **Policy violation (blackout period)** → auto-flag with explanation; manager can override if tenant policy permits.
- **Leave during notice period** (employee resigned) → coordinate with offboarding saga; per-jurisdiction rules on accrued-but-unused leave payout.
- **Partial-day leave** (half-day in morning) → handled separately; payroll integration knows partial-day prorating.
- **Multi-day across weekend** → exclude weekend days from balance deduction; show employee the net days deducted.
- **Last-minute leave** (request for tomorrow) → fast-path: skip 72h reminder cycle, manager gets immediate push notification with prominent "URGENT" marker.
- **Recurring leave** (every Friday afternoon, ongoing) → treat as employee-policy change, not as a single saga; route to HR for formal flexible-work-arrangement record.
- **Leave reversal after approval** (manager changed mind) → manager can revoke within T+24h via card; after T+24h, requires HR override + audit-trail.
- **Cross-jurisdiction employee** (remote worker in different tax-residency than employer) → apply employee's residence-country labor law overlay, not employer-country (per EU posted-workers directive + KR equivalent).
- **Bereavement / compassionate leave** → bypass manager-approval; auto-approve up to per-tenant maximum (e.g., 5 days) on submission; emit notification only.
- **Sick leave** → separate balance pool; may require doctor's-note attachment beyond 3 days (jurisdiction-dependent); doctor's-note goes through `drive` µservice with restricted access (PHI per HIPAA pack-us-healthcare or sensitive-health-data per EU GDPR Article 9).
- **Parental leave** → durable saga lasting weeks/months; coordinates with payroll for parental-pay (jurisdiction-dependent: KR 출산휴가 90 days / 육아휴직 up to 1y; US FMLA 12 weeks unpaid + state-specific paid leave; EU varies); may include keep-in-touch days.

#### B.5 UX details

- **Rich Messenger card** with action buttons (Approve / Deny / Defer / Adjust) and inline context (employee leave balance, team calendar overlay, manager's previous approval pattern from Intelligence-summarised history).
- **One-click approve** — tap "Approve" → confirmation dialog ("Approve [employee]'s leave for [dates]?") → tap confirm → saga advances. Total interaction time < 5 seconds.
- **Calendar block visibility** — once approved, employee's OOO appears on team calendar immediately; team members see "Vacation: [name] returns [date]" tooltip on hover.
- **Deep link from manager card to context** — tap "View details" on card → opens employee's leave history + team-coverage view in Application Shell; manager has full context without searching.
- **Mobile-first** — vacation request from mobile in 3 taps: open app → tap leave → select dates → submit.
- **Voice flow** — natural-language understanding via Intelligence: "vacation next Monday Tuesday" → parsed → confirmation dialog read aloud → "Yes" → submitted.

#### B.6 Compliance (per-jurisdiction)

| Jurisdiction | Statute | Rule |
|---|---|---|
| KR | 근로기준법 Article 60 — annual leave | min 15 days/year after 1y service; pro-rata for <1y; employer must grant on requested dates unless significant business impact |
| KR | 근로기준법 Article 73 — sick leave (no statutory min) | tenant policy; if industrial accident → 근로기준법 Article 78 (separate) |
| KR | 남녀고용평등법 Article 18 (parental leave) | up to 1y per child; financial support from 고용보험 |
| US | FMLA 29 USC §2612 | 12 weeks unpaid for FMLA-qualifying medical/family events |
| US (CA) | CFRA + PFL | additional state-paid leave; per-state varies |
| EU | EU Working Time Directive 2003/88/EC Article 7 | min 4 weeks paid annual leave |
| EU | EU Work-Life Balance Directive 2019/1158 | min 4 months parental leave per parent |
| UK | Working Time Regulations 1998 | 28 days incl. bank holidays |
| JP | Labor Standards Act §39 — paid leave | min 10 days after 6mo (year 1) up to 20 (year 6+) |
| DE | Bundesurlaubsgesetz (BUrlG) | min 20 days |
| FR | Code du travail L3141-3 | 30 days |
| ES | Estatuto de los Trabajadores | 30 days |
| AU | National Employment Standards | 4 weeks annual + 10 days personal/carer's |
| BR | CLT Article 130 — férias | 30 days |
| CA | Canada Labour Code §183 | 2 weeks then 3 weeks at 5y |
| IN | Factories Act §79 | 1 day per 20 worked |

---

### Flow C — E-Signing (Contracts, NDAs, Offers, HR Docs)

#### C.1 Purpose

Legally-binding electronic signature collection for contracts, NDAs, offer letters, HR policy acknowledgements, vendor agreements, consultant SOWs, and any document requiring multi-party attestation. Conformance to US ESIGN Act + EU eIDAS QES + KR 전자서명법 + JP e-signature law + UK Electronic Communications Act 2000 + India IT Act 2000 §3 + Singapore Electronic Transactions Act + Australia Electronic Transactions Act 1999.

#### C.2 Trigger surfaces

- **HR uploads PDF to onboarding mail** sent to new hire (HR-initiated).
- **HR Portal e-sign upload screen** in Application Shell.
- **Mail with attached PDF tagged `e-sign-request`** sent to signer.
- **Workflow Studio template** — tenant HR designs custom onboarding doc bundle.
- **API call** from external system (e.g., HRIS imports offer letter via Plugin App Store integration).
- **Drive folder convention** — drop PDF into `/Workplace/e-sign-queue/` → auto-trigger.

#### C.3 Saga: `ESignSaga` (long-running, may span days)

Steps:

1. **`receive_doc`** — accept PDF + signer-list + signing-order (sequential or parallel) + expiration_at + signing-jurisdiction + legal-binding-tier (`simple`, `advanced`, `qualified` per eIDAS levels).
2. **`store_master_pdf`** — store the canonical PDF in `drive` µservice with content-hash; data_class = `LEGAL_DOC_PENDING`; retention floor: 7 years (US IRS / SOX / SEC 17a-4); legal-hold-compatible.
3. **`extract_signature_fields`** — Intelligence substrate (per ADR-0255) OCRs the PDF + uses layout-detection (vision LLM) to locate signature blocks, date blocks, initial blocks, checkbox fields. Result: a JSON of field positions + types + assigned-to-which-signer.
4. **`review_extracted_fields`** — HR (originator) sees the extraction in Application Shell; can drag fields to adjust positions; can add manual fields (initials, date, checkbox, free-text). Signed UI gesture stored.
5. **`build_signer_order`** — accept signer order (e.g., "employee first, then CEO, then HR"); parallel mode allowed (all sign at once); mixed mode allowed (e.g., legal reviews first, then parallel sign).
6. **`send_to_first_signer`** — Mail + Messenger + Mobile push to first signer with one-click "Open document" deep link. Mail includes preview thumbnail + summary ("This is your offer letter for the role of Senior Engineer. Click to review and sign.").
7. **`signer_authentication`** — signer opens link; identity verified per legal-binding tier:
   - `simple` (basic ESIGN) — email link with one-time code; sufficient for non-binding internal docs.
   - `advanced` (eIDAS AdES) — passkey/WebAuthn or government-ID-linked OIDC; sufficient for employment contracts.
   - `qualified` (eIDAS QES) — qualified certificate from a trust service provider (e.g., D-Trust, Bundesdruckerei for EU; KICA for KR); required for some legal docs in EU and KR.
8. **`signer_review`** — signer sees the PDF in-browser (or in mobile app native viewer); navigates pages; sees highlighted signature blocks; can decline ("I don't agree with section 5 — request revision") with reason.
9. **`signer_signs`** — signer signs each assigned field. Signature input options:
   - **Drawn signature** — finger or stylus on touch device; vectorised + saved to signer profile for next time.
   - **Typed signature** — typed name rendered in handwriting font; cryptographically bound to signer-identity.
   - **Pre-saved signature** — signer's prior signature reused with explicit confirmation per signing-session.
   - **Initials** — separate initial-stroke field for paragraph-level initialing.
   - **Date** — auto-filled with current date in signer's timezone.
   - **Checkbox** — for "I agree to terms" style fields.
10. **`validate_signature_integrity`** — verify all required fields are signed; verify signature blob format is valid; cryptographically bind signature to PDF content-hash via cosign attestation (per ADR-0211 in-house tech-stack preference + cosign-based signing).
11. **`apply_signature_to_pdf`** — render the signature into the PDF at the field positions, producing a new versioned PDF; embed PAdES (PDF Advanced Electronic Signatures, ETSI EN 319 142) signature metadata for eIDAS-compliant docs.
12. **`route_to_next_signer`** — if sequential mode, send to next signer (loop back to step 6). If parallel, wait for all signers (durable wait).
13. **`all_signed_or_expired`** — when all signers complete OR expiration_at reached → branch.
14. **`finalize_signed_pdf`** — apply the final long-term-validation (LTV) timestamp from a Qualified Trust Service Provider (QTSP for EU); embed certificate chain; produce the final PAdES-LTV PDF.
15. **`store_signed_pdf`** — store final PDF in `drive` µservice with content-hash, data_class = `LEGAL_DOC_SIGNED`, retention 7+ years; legal-hold compatible; restricted access list (originator + signers + tenant compliance officer).
16. **`emit_audit_chain`** — `ESignCompleted` event with signer identities, signature timestamps, IP + device fingerprint per signer, certificate chain hash, content-hash; Merkle-sealed; retention indefinite for legally-binding docs.
17. **`notify_all_parties`** — Mail to each signer with copy of final PDF (or link); Messenger confirmation; mobile push.
18. **`route_to_hr_system`** — if doc is HR-bound (offer letter, employment contract), emit `OnboardingDocSigned` event consumed by `hr` µservice + onboarding saga (Flow F).

#### C.4 Edge cases

- **Signer rejects (declines to sign)** → originator notified with reason; saga branches to (a) revise + restart, (b) terminate. Audit-chain records the decline.
- **Expiration reached without all signers** → saga ends in `expired` state; originator notified; partial signatures invalidated (no PDF produced) OR retained as evidence of partial agreement per tenant policy.
- **Signer requests revision** — saga branches to revision-sub-saga: originator amends, hash changes, all prior signatures invalidated, all signers re-sign with new doc. Audit-chain links the revision chain.
- **Legal hold during signing** — if doc enters legal-hold mid-saga, signing continues but the saga record is locked from deletion; legal-hold flag propagates to the final PDF metadata.
- **Signer identity changed** (signer left company between send + sign) — flag for HR re-review; option to re-assign or cancel.
- **Cross-jurisdiction signers** (US-based originator, EU signer) → apply highest-tier requirement (qualified) unless tenant policy explicitly downgrades; conformance to both ESIGN + eIDAS for cross-border docs.
- **Document with embedded form fields** (e.g., I-9 form) → preserve form-field interactivity; signer fills + signs; capture both form data + signature.
- **Mobile-only signer** — full sign experience works in mobile app + browser on mobile; signature drawn with finger; no jump-to-desktop required.
- **Multi-language signers** — Intelligence substrate translates the doc summary for the signer's preferred language; underlying PDF remains in original language with the binding signature.
- **Notary requirement** (some US states require notary for certain docs) → Plugin App Store integration with Notarize / OneNotary for online-notarisation; saga step `notary_attestation` for those tenants.
- **Apostille / consular legalisation** required for cross-border use → out-of-scope for workplace-integration; tenants use Plugin App Store provider integrations.
- **Hardware-token signing** (e.g., KR 공인인증서, EU qualified-certificate USB tokens) → supported via WebAuthn + per-jurisdiction extensions.
- **Revoked certificate during signing** — check certificate revocation status (OCSP / CRL) at signature application; reject if revoked; signer must re-obtain valid cert.

#### C.5 UX details

- **Adobe Sign / DocuSign parity** for signing experience; finger or stylus drawing; saved-signatures gallery; one-click sign for routine docs; mobile-first responsive design.
- **Inline progress bar** — "Step 2 of 4: Sign initials on page 3" — signer always knows where they are.
- **Field-by-field guidance** — auto-scroll to next required field; visual highlight; "Tap here to sign" inline call-to-action.
- **Pre-signing summary** — "You're about to sign: Employment Contract for Senior Engineer role. Review key terms below before signing." Intelligence-extracted summary of compensation, start date, key obligations.
- **Post-signing receipt** — Mail with signed PDF attached + link + audit-trail summary ("Signed by you on 2026-05-20 at 14:32 KST from device [type], IP [region]").
- **Signature gallery** — signer's prior signatures saved; one-tap reuse with explicit "Apply this signature?" confirmation per session.
- **Accessibility** — signing flow keyboard-navigable; screen-reader announces "Signature field, signer required, your turn to sign"; sufficient color contrast on all UI; alt-text on signature pad.

#### C.6 Compliance

| Jurisdiction | Statute | Binding tier(s) supported |
|---|---|---|
| US | ESIGN Act 15 USC §7001 + UETA (state) | simple + advanced |
| EU | eIDAS Regulation 910/2014 | simple + advanced + qualified (with QTSP integration) |
| KR | 전자서명법 (Electronic Signature Act, 2020 revision) | simple + qualified (공인전자서명) |
| JP | 電子署名及び認証業務に関する法律 (Act on Electronic Signatures and Certification Business) 2001 | simple + qualified |
| UK | Electronic Communications Act 2000 | simple + advanced |
| IN | Information Technology Act 2000 §3 | simple (with Aadhaar eSign) |
| SG | Electronic Transactions Act | simple + secure |
| AU | Electronic Transactions Act 1999 | simple |
| CA | PIPEDA + provincial Electronic Commerce Act | simple |
| BR | MP 2.200-2 / Lei 14.063/2020 | simple + advanced + qualified (ICP-Brasil) |
| CN | 电子签名法 (Electronic Signature Law) 2004 | simple + reliable (依法成立的认证机构) |

Legally-binding tier per doc declared in tenant policy; saga refuses lower-tier sign for docs requiring higher tier.

---

### Flow D — Meeting Scheduling

#### D.1 Purpose

End-to-end scheduling of meetings across attendees (internal + external; same-tenant + cross-tenant); creates calendar events; provisions Meet video link; sends RFC 5545 ITIP invitations; handles RSVP + reschedule + cancel + recurring patterns; coordinates resources (rooms, equipment). Competes with Calendly / x.ai / Google Calendar / Outlook / Fantastical.

#### D.2 Trigger surfaces

- **Calendar app** in Application Shell — full UI.
- **`/schedule` slash command in Messenger** — open scheduling card with attendee picker + time slots.
- **Mail with proposed times** — reply to a mail thread proposing meeting; Intelligence parses dates and offers RSVP.
- **Voice trigger** — "Hey Siri, schedule a meeting with Jane tomorrow at 2".
- **Workflow Studio template** — tenant authors custom scheduling flow (e.g., "interview-scheduling" template with custom approval flow).
- **Public Calendly-style link** — tenant publishes `oyatie.com/meet/<tenant>/<user>` with available slots; external users self-schedule.
- **Mobile mini-app** — quick-schedule from contacts.

#### D.3 Saga: `MeetingScheduleSaga`

Steps:

1. **`receive_request`** — accept `(organizer, attendees, duration_minutes, preferred_time_window, meeting_type, agenda?, attached_docs?, room_required?, equipment_required?)`.
2. **`resolve_attendees`** — for each attendee, identify: internal (same tenant), external-same-platform (different oyatie tenant), or external-other-platform (need iMIP RFC 6047 invitation via Mail). Cedar gate on cross-tenant scheduling per ADR-0244.
3. **`gather_availability`** — gRPC to `calendar` µservice `QueryFreeBusy({attendees, window})`; for cross-tenant attendees, use cross-tenant availability resolver (per calendar PRD FR-10) which returns free/busy projection without leaking event details.
4. **`suggest_times`** — Intelligence substrate ranks candidate slots by: attendee availability, meeting-preference patterns (does attendee prefer morning?), timezone fairness (don't always schedule outside 9-5 for one timezone), focus-time avoidance (don't fragment heads-down blocks); returns top 3 slots.
5. **`reserve_room_if_required`** — gRPC to `calendar` µservice `ReserveRoom({preferred_rooms, slots, equipment})`; tentative reservation with 24h hold pending confirmation.
6. **`present_slots_to_organizer`** — Messenger card / Application Shell / Mail with 3 candidate slots + room assignment; organizer picks one.
7. **`create_calendar_event`** — gRPC to `calendar` µservice `CreateEvent({...})`; calendar emits `EventCreated` event (per calendar PRD); Cedar gate on creation; idempotency key: SHA-256 of `(organizer, attendees, start_time)`.
8. **`provision_meet_link_if_video`** — if meeting type is video, gRPC to `meet` µservice `CreateRoom({tenant_id, scheduled_for, expected_attendees, lobby_policy, recording_policy, transcription_policy})`; bind meet room URL to calendar event.
9. **`provision_collaborative_doc_if_agenda`** — if agenda provided, create `docs` µservice doc; bind to event; auto-share with attendees; pre-meeting agenda + post-meeting notes flow.
10. **`send_invitations`** — `calendar` µservice sends RFC 5545 ITIP invitation via `comms-email` substrate to all attendees (internal + external); attendees receive native calendar invite (Outlook, Google Calendar, Apple Calendar all compatible); meeting link + agenda doc + dial-in info embedded.
11. **`await_rsvp`** — durable wait; track RSVP state per attendee (accepted / declined / tentative / no-response).
12. **`check_quorum`** — at T-24h before meeting, check if required attendees confirmed; if quorum not met, branch to `step_handle_quorum_failure`.
13. **`handle_quorum_failure`** — Messenger to organizer with options: (a) reschedule (loop to step 3 with adjusted window), (b) proceed anyway, (c) cancel; organizer decides.
14. **`send_reminder`** — T-15min before meeting, Messenger + mobile push to all confirmed attendees with one-click join link.
15. **`meeting_starts`** — `meet` µservice emits `MeetingStarted`; saga records start.
16. **`meeting_ends`** — `meet` µservice emits `MeetingEnded`; if recording enabled, recording uploaded to `recordings` µservice; if transcription enabled, transcript posted to `drive` linked to calendar event.
17. **`post_meeting_summary`** — Intelligence substrate generates summary from transcript; posts to agenda doc; sends Mail summary to attendees with action items extracted; updates `tasks` µservice with action-item tasks assigned to specific attendees.
18. **`emit_audit_chain`** — `MeetingCompleted` event with attendee list + duration + recording-status; Merkle-sealed.

#### D.4 Edge cases

- **Attendee in different timezone** — display times in each attendee's local timezone in invitations; primary organizer's timezone in event metadata; auto-convert RRULE recurrence per RFC 5545.
- **Attendee declines** — Intelligence offers alternative slots; organizer can re-invite without restarting from scratch.
- **Recurring meeting (weekly, monthly)** — RFC 5545 RRULE/EXDATE; bounded expansion (per calendar PRD); option to modify single occurrence vs entire series.
- **External attendee (non-oyatie tenant)** — federated calendar via iMIP RFC 6047; external attendee receives standards-compliant .ics invitation; their calendar app handles RSVP back via mail; oyatie's saga parses the mail response and updates calendar event.
- **Calendly-style public link** — external user picks slot from organizer's available slots; auto-creates event; sends confirmation; per-tenant rate limits to prevent abuse; per-tenant Cedar policy on which users can have public links.
- **Cross-tenant scheduling** — two oyatie tenants opt-in via invitation grant (per calendar PRD); free/busy projection only; no event-detail leak.
- **Room double-booking conflict** — calendar's room-booking BC detects + resolves at booking time; saga gets back conflict; suggests alternate room.
- **Equipment conflict** (limited resource) — same as room.
- **Last-minute change** (organizer reschedules 1h before) — emit `EventUpdated` to all attendees with explicit "Time changed!" Messenger card + mail.
- **Meeting overruns** — `meet` µservice detects; offers organizer one-click extend (creates follow-up event); attendees notified.
- **No-show attendee** — `meet` µservice records ParticipantJoined events; saga can detect no-show and (per tenant policy) notify or auto-reschedule.
- **All-day events** (no time-of-day) — handled as full-day RRULE block; no `meet` link provisioned.
- **Travel time consideration** — Intelligence checks attendee's prior + next meetings for location; warns of unrealistic back-to-back across locations.
- **Heads-down time protection** — per attendee preference, certain calendar blocks marked "focus" (no meetings); scheduler refuses to suggest those times unless organizer overrides with reason.
- **Vacation/OOO attendees** — calendar shows OOO; scheduler suggests post-return slots or asks if attendee can be skipped.

#### D.5 UX details

- **Calendly / x.ai parity** — best-in-class suggestion UX; show 3 slots not 30; one-tap confirmation; saved-slot-templates ("30-min intro", "1h deep-dive").
- **Pre-meeting agenda doc** — auto-created in `docs` µservice; organizer pre-populates; attendees can edit; visible to all confirmed attendees.
- **One-click join** — calendar event has prominent "Join" button; opens meet app native (mobile) or browser (desktop) with auto-camera/mic-permission flow.
- **In-Messenger scheduling card** — full flow without leaving chat; date picker, attendee picker, slot picker all inline.
- **Recurring-meeting management** — clear UI for "edit this occurrence" vs "edit this and future" vs "edit all"; standard Google-Calendar / Outlook parity.
- **Smart suggestions** — "You have 5 meetings tomorrow already — book this one for next week instead?" — Intelligence-driven nudges.
- **Conflict resolution UI** — when attendees have conflicts, side-by-side timeline view of all attendees showing the conflicts; organizer drag-resizes to find common slot.

#### D.6 Compliance + Standards

- **RFC 5545 (iCalendar)** — event format.
- **RFC 5546 (iCalendar Transport-Independent Interoperability Protocol — iTIP)** — invitation/reply protocol.
- **RFC 6047 (iCalendar Message-Based Interoperability Protocol — iMIP)** — mail-based delivery.
- **RFC 4791 (CalDAV)** — calendar over WebDAV.
- **KR PIPA Article 15** — consent for cross-tenant scheduling.
- **GDPR Article 6 lawful basis** — for cross-tenant attendee data sharing.

---

### Flow E — Expense Report + Reimbursement

#### E.1 Purpose

Capture employee out-of-pocket expenses + corporate-card spend; categorise per tenant chart-of-accounts; enforce per-tenant + per-category policy; route through approval chain; transfer approved expenses to payroll/finance for reimbursement. Competes with Expensify / SAP Concur / Brex / Ramp / Pleo / Spendesk.

#### E.2 Trigger surfaces

- **Mobile app receipt-photo button** (primary) — snap photo of receipt; OCR extracts merchant, amount, date, category.
- **Corporate-card transaction detection** — bank/card feed (Plaid / native bank API / corporate-card issuer webhook) emits txn; auto-creates expense pre-populated.
- **Mail forward** — forward digital receipt mail to `expense@<tenant-domain>`; Intelligence parses.
- **Expense Portal in Application Shell** — manual entry for cash + edge cases.
- **`/expense` slash command in Messenger** — quick add.
- **Mileage tracker** — mobile app GPS-tracks travel; converts to mileage expense per IRS / per-country rate.

#### E.3 Saga: `ExpenseSaga`

Steps:

1. **`receive_expense`** — accept expense record `(employee_id, amount, currency, merchant, date, category?, receipt_attachment, project_id?, business_purpose?)`.
2. **`ocr_receipt`** — if receipt-photo, Intelligence substrate OCRs + extracts: merchant name, total, currency, tax, date, line items; saves structured data.
3. **`categorise`** — Intelligence categorises against tenant chart-of-accounts (Meals, Travel-Airfare, Travel-Lodging, Travel-Ground, Office-Supplies, Software, Training, etc.); confidence score; employee can override.
4. **`fx_conversion`** — if foreign currency, fetch exchange rate (from FX provider per Plugin App Store integration; default exchange rate at transaction date); convert to tenant's home currency; record both.
5. **`policy_check`** — Cedar gate per-tenant + per-category policy: max amount per meal, max nights per hotel, requires-receipt-above-N, requires-business-purpose, requires-project-id, etc.; record violations.
6. **`tax_categorisation`** — per-jurisdiction tax category (US: 1099-NEC, W-2 reportable; KR: 부가세 separate; EU: VAT recoverable); for VAT/GST, attempt VAT-line-item extraction from receipt.
7. **`route_to_employee_review`** — Messenger card / Application Shell to employee: OCR'd fields + categorisation + any policy flags; employee reviews + submits.
8. **`build_approval_chain`** — based on amount + category + tenant policy: small amounts auto-approve; mid amounts to direct manager; large amounts to manager + finance.
9. **`notify_approver`** — Messenger + Mail card to manager with receipt thumbnail + amount + business-purpose + policy-violation summary (if any) + employee's historical patterns (Intelligence-summarised).
10. **`approver_decision`** — approve / deny / request-info / partial-approve.
11. **`handle_partial_approval`** — if approver approves $200 of a $250 expense, employee notified with reason; can accept or appeal.
12. **`route_to_finance`** — on approval, emit `ExpenseApproved` event consumed by `payroll` µservice (or `finance` integration via Plugin App Store).
13. **`payroll_inclusion`** — reimbursement appears in employee's next paycheck OR processed as a separate ACH/wire (per tenant policy); employee notified of expected date.
14. **`emit_audit_chain`** — `ExpenseReimbursed` event; Merkle-sealed; retention 7y for tax records (US IRS / KR 국세청 / EU VAT records).
15. **`accounting_export`** — emit `ExpenseAccountingRecord` event consumed by tenant's accounting system via Plugin App Store (QuickBooks, NetSuite, SAP, Xero, etc.).

#### E.4 Edge cases

- **Missing receipt** — for expenses below tenant-policy threshold (e.g., $25), allow no-receipt with attestation; above threshold, require receipt photo or reason.
- **Policy violation** (over per-meal cap, etc.) — manager can override with reason; over-policy expenses flagged for compliance review.
- **Foreign currency without txn-day rate** — use best-available rate (interbank rate from FX provider) + record source.
- **Personal expense on corp card** — employee flags as personal; auto-deduct from payroll OR employee pays back; either way, accounting recordable.
- **Duplicate expense detection** — Intelligence detects if same receipt photo or same merchant/amount/date combo already submitted; flag for employee review.
- **In-flight expense (corp card txn live)** — txn streamed in real-time from card issuer; saga creates expense in `pending_receipt` state; nudges employee for receipt within 7 days.
- **Mileage expense** — GPS-tracked OR manual entry of start/end addresses; auto-calculate mileage; apply jurisdiction-specific rate (US 2026: $0.67/mi business; KR varies; EU varies per country).
- **Multi-leg travel** — bundle related expenses (flight + hotel + ground transport + meals) into single trip-report submission; common ‘‘trip’’ approval.
- **Tip / gratuity** — separate line on US receipts; auto-extract.
- **Tax-categorisation** — auto-flag potential 1099 expenses; auto-flag VAT-recoverable items for jurisdictions that support recovery.
- **Corp-card statement reconciliation** — month-end, reconcile all card txns vs submitted expenses; flag unreconciled.
- **Expense after termination** — must be submitted within tenant policy window (typically 30 days post-termination); after window, denied.
- **Pre-approval required** (per tenant policy, expenses >$X need pre-approval) — separate pre-approval sub-saga before incurring spend.
- **Expense in dispute** — if employee disagrees with manager denial, escalate to skip-level + HR; audit-chain records dispute resolution.
- **Reimbursement to non-employee** (contractor, candidate-travel-reimbursement) — separate flow with W-9 / 1099 collection if US-based.

#### E.5 UX details

- **Expensify / Brex / Ramp parity** — magical receipt OCR; auto-categorise; in-flight txn detection; one-tap submit.
- **Receipt-photo with auto-crop** — mobile camera with edge detection; auto-rotate; auto-enhance contrast.
- **Auto-categorise with high confidence** — when Intelligence confidence > 90%, no employee action needed; otherwise show top 3 picks.
- **In-flight expense banner** — "Your corp card was charged $34.50 at Coffee Shop. Add receipt?" appears in Messenger 30s after txn.
- **One-tap approval for managers** — Messenger card with thumbnail + amount + employee historical patterns + one-tap "Approve" with biometric confirmation.
- **Bulk-approval** — manager sees multiple pending expenses; can select-all + approve-all if all within policy + small.
- **Trip-report view** — when employee has a trip, bundle all expenses into a single trip view for manager.
- **VAT-recovery tooltip** — show finance team the VAT-recoverable amounts for jurisdictions that allow recovery.

#### E.6 Compliance

| Jurisdiction | Statute | Rule |
|---|---|---|
| US (federal) | IRS Pub. 463 — Travel, Gift, and Car Expenses | accountable plan requires: (a) business purpose, (b) substantiation, (c) return of excess; receipts required for >$75 lodging; auto-mileage rate published annually |
| US (state) | varies | per-state mileage / per-diem may differ |
| EU | VAT Directive 2006/112/EC | VAT recovery on business expenses; per-country invoice format requirements |
| KR | 부가가치세법 (VAT Act) | 부가세 separation; T-invoice (전자세금계산서) for B2B |
| KR | 소득세법 (Income Tax Act) Article 121 | business expense substantiation |
| JP | 法人税法 + 消費税法 | business expense + consumption tax substantiation |
| UK | HMRC ITEPA 2003 | benefits-in-kind reporting; P11D |
| Mileage rates | annually published by tax authority | US IRS 2026 $0.67/mi; UK HMRC 45p/mi first 10k; KR 비과세 한도 |

Retention: 7 years (US IRS audit window) / 5 years (KR 국세기본법) / per-jurisdiction.

---

### Flow F — Onboarding (Employee)

#### F.1 Purpose

Coordinate every cross-µservice action needed when a new hire joins: HR record creation, identity provisioning, device + access provisioning, policy acknowledgements (e-sign), buddy assignment, first-week schedule, training, equipment, payroll setup. Competes with Workday HR + BambooHR + Rippling + Greenhouse + Sapling.

#### F.2 Trigger

- **HR creates new-hire record** in `hr` µservice (or imports from ATS like Greenhouse via Plugin App Store).
- **Start date reached** (saga starts T-14d before start date).

#### F.3 Saga: `OnboardingSaga` (long-running, weeks-long)

Steps:

1. **`receive_new_hire`** — accept `(employee_id, start_date, role, manager_id, department, location, employment_class, compensation_summary)`.
2. **`pre_start_t14d`** — T-14d: send welcome mail to candidate's personal email; collect bank-account info for payroll (encrypted via tenant-DEK); collect personal info needed for HR record; collect emergency contact.
3. **`pre_start_t7d`** — T-7d: provision identity in `oya/identity/` (OIDC service principal under `tenant.<dept>.employee.<id>`); generate corp credentials.
4. **`pre_start_t7d_devices`** — order laptop / monitor / equipment per role; ship to home address pre-start; emit `EquipmentOrdered` event consumed by `procurement` µservice (Flow J).
5. **`pre_start_t3d`** — schedule first-week calendar: orientation, manager 1:1s, team intros, training sessions; book conference rooms.
6. **`pre_start_t1d`** — send Day-1 welcome mail with login info, agenda, building access info, buddy intro.
7. **`day_1`** — employee arrives; clocks in (Flow A); receives device; first-day orientation; signs Day-1 docs via e-sign saga (Flow C): employee handbook, NDA, IP-assignment, harassment policy, code of conduct, IT acceptable use; HR I-9 (US) or equivalent.
8. **`day_1_access`** — provision access to all tenant apps: Mail, Drive, Calendar, Messenger, Meet, Workflow Studio, role-specific tools; based on role-based access policy fragment per Cedar.
9. **`week_1`** — daily check-ins via Messenger ("How's your first week going? Any blockers?"); assigned reading; assigned training modules; buddy lunch; manager 1:1.
10. **`week_2`** — first project / team contribution; goal-setting kickoff (sets 30/60/90-day goals into `performance` µservice).
11. **`30_day_check_in`** — manager + employee + HR review; satisfaction survey via `forms` µservice; address concerns.
12. **`60_day_check_in`** — similar.
13. **`90_day_check_in`** — final probation review; either: confirm full-time, extend probation, or terminate (route to offboarding saga); emit `EmploymentConfirmed` event consumed by payroll for any post-probation comp changes.
14. **`emit_audit_chain`** — events at each milestone; full onboarding audit trail.

#### F.4 Edge cases

- **Delayed start date** — adjust saga timer; if delayed >30d, revalidate offer.
- **Withdrawn offer** — saga branches to cancellation; revoke provisioned identity; clear personal data per consent-graph.
- **Pre-start info incomplete** — escalate to HR; gate Day-1 if blocked.
- **Equipment shipment delayed** — auto-arrange loaner OR delay start.
- **Background-check failure** — saga pauses; HR reviews; potentially withdraw offer.
- **Cross-border hire** — coordinate visa, work-permit, tax-form variants; per-jurisdiction onboarding checklist.
- **Remote hire** — no physical office access; remote-onboarding variant.
- **Contractor vs employee** — different doc bundle; different access scope; different payroll path.
- **Re-hire** — reuse prior employee record; refresh policy acknowledgements.

#### F.5 UX details

- **Single onboarding dashboard** for new hire — sees all pending tasks, completed checkmarks, upcoming sessions.
- **Manager dashboard** — sees their new hires' progress; nudges for incomplete checklist items.
- **HR dashboard** — sees all in-flight onboardings.
- **No-paper Day-1** — every form digital; e-sign for everything; mobile-friendly.

---

### Flow G — Offboarding (Employee)

#### G.1 Purpose

Coordinate every cross-µservice action needed when an employee leaves: knowledge transfer, access revocation, equipment return, final paycheck + accrued-leave payout, exit interview, retention of records per regulatory requirements, alumni-network enrolment (optional).

#### G.2 Trigger

- **Resignation** — employee submits via HR portal.
- **Termination** — HR / manager initiates.
- **End of fixed-term contract** — auto-triggered at contract end.
- **Retirement** — separate sub-flow.

#### G.3 Saga: `OffboardingSaga`

Steps:

1. **`receive_separation`** — accept `(employee_id, separation_type, last_day, reason?)`.
2. **`legal_hold_check`** — if employee under legal hold (litigation, investigation), preserve all data per Sedona Principles; flag.
3. **`notify_stakeholders`** — manager, HR, IT, finance, security all notified.
4. **`build_kt_plan`** — manager + employee jointly build knowledge-transfer plan: docs to write, projects to hand off, accounts to transfer ownership; tracks completion via `tasks` µservice.
5. **`schedule_exit_interview`** — HR books 30-min exit interview within last week.
6. **`access_revocation_plan`** — list of accesses to revoke: identity, mail, drive, messenger, meet, repos, third-party SaaS (via Plugin App Store integrations), VPN, badge. Revocation scheduled for end-of-last-day.
7. **`equipment_return_plan`** — laptop, monitor, badge, phone, etc.; if remote, ship return-label; tracked.
8. **`final_paycheck_calculation`** — payroll computes final paycheck including accrued unused vacation payout (per jurisdiction: CA requires immediate; KR within 14 days; EU varies), bonus pro-rata (per tenant policy), stock-option vesting cutoff.
9. **`last_day_actions`** — clock-out triggers; team announcement (if employee consents); farewell.
10. **`post_last_day_revoke`** — at end of last day, revoke all access; preserve audit-chain records (per retention); preserve commits + PRs as `authored_by_former_employee` with pseudonymisation option per consent-graph (DSAR + GDPR Article 17 compatible).
11. **`alumni_enrolment`** — optional: employee opts into alumni network (separate tenant role).
12. **`hr_record_archive`** — HR record moves to `terminated` state; retained per jurisdiction (US 4-7y per FLSA + SOX; KR 3y per 근로기준법; EU varies).
13. **`emit_audit_chain`** — full offboarding sealed.

#### G.4 Edge cases

- **Same-day termination** — accelerated saga; revoke immediately; security escort if needed; final paycheck within jurisdiction-required window.
- **Garden leave** — employee on payroll but no work; access limited.
- **Dispute / litigation** — legal-hold prevents data deletion; all records preserved.
- **Death of employee** — sensitive sub-flow; coordinate with family; bereavement support to team.
- **Re-hire eligibility flag** — HR marks; controls future application visibility.

#### G.5 UX details

- **Departing employee dashboard** — checklist of return-tasks + KT-tasks; clear visibility of what's expected.
- **Manager dashboard** — KT progress.
- **HR dashboard** — full offboarding overview.

---

### Flow H — Performance Review

#### H.1 Purpose

Periodic structured review of employee performance against goals, with 360-degree feedback, calibration, and downstream compensation actions. Competes with Lattice / 15Five / Culture Amp / Workday Performance.

#### H.2 Trigger

- **Cycle start** (quarterly, semi-annually, annually per tenant policy).
- **Off-cycle review** (e.g., for performance improvement plan).

#### H.3 Saga: `PerformanceReviewSaga`

Steps:

1. **`receive_cycle_start`** — accept cycle config from HR.
2. **`goal_check`** — verify each employee has goals in performance system; if missing, prompt manager.
3. **`self_assessment`** — employee fills self-assessment via `forms` µservice.
4. **`peer_feedback_collection`** — solicit 360 from selected peers; anonymous option (per tenant policy); confidential to manager+HR.
5. **`manager_assessment`** — manager fills assessment.
6. **`upward_feedback`** — direct reports fill feedback on manager (anonymous, aggregated).
7. **`calibration_session`** — manager-cohort meeting to normalise ratings; reduce bias; HR facilitates.
8. **`review_meeting`** — manager + employee 1:1 to discuss; calendar saga (Flow D) integration; agenda doc + summary via Intelligence.
9. **`final_rating_and_actions`** — final rating; compensation actions (raises, bonuses) initiated as separate `CompensationReviewSaga`; promotion considerations.
10. **`acknowledge_review`** — e-sign saga (Flow C): employee acknowledges review.
11. **`development_plan`** — manager + employee author dev plan; tracks in `tasks` µservice.
12. **`emit_audit_chain`** — review record sealed; 7y retention for legal compliance (US: EEOC complaint window).

#### H.4 Edge cases

- **Performance Improvement Plan (PIP)** — sub-saga with structured milestones, weekly check-ins, formal termination-path if unmet.
- **Calibration outlier** — flagged for HR follow-up.
- **Bias detection** — Intelligence flags potential bias patterns (e.g., consistent lower ratings for protected class); HR review.
- **Discrimination claim** — escalate to HR + legal; legal-hold; preserve all artifacts.

#### H.5 UX details

- **Streamlined review forms** — short, focused; not 50-question monstrosities.
- **In-line evidence collection** — link to specific PRs, customer feedback, OKR completions throughout the cycle, not retroactively.
- **Calibration view** — manager sees their team's ratings + adjacent team distribution; nudges for outliers.

---

### Flow I — Travel Request

#### I.1 Purpose

Pre-approve business travel; book flights/hotels/transport; manage in-trip changes; reconcile with expense reports. Competes with SAP Concur / TripActions / Egencia.

#### I.2 Trigger

- **Employee submits via Travel Portal** or `/travel` Messenger command.

#### I.3 Saga: `TravelRequestSaga`

Steps:

1. **`receive_request`** — accept `(employee_id, destination, dates, purpose, estimated_cost, preferred_options)`.
2. **`policy_check`** — Cedar gate per-tenant travel policy: pre-approval threshold, preferred carriers, hotel-tier limits, advance-booking discount targets.
3. **`approval_chain`** — manager approval; finance approval if above threshold.
4. **`book_flight`** — integration via Plugin App Store (Amadeus, Sabre, Travelport, direct airline APIs); price-comparison; carbon-emissions display per ADR-0174 sustainability tag.
5. **`book_hotel`** — similar.
6. **`book_ground_transport`** — rideshare, train, rental car.
7. **`send_itinerary`** — Mail + Mobile push + Calendar event sequence.
8. **`pre_trip_alerts`** — flight delays, weather, advisories (State Dept / GOV.UK / KR 외교부 advisories integration).
9. **`in_trip_support`** — 24/7 support via Plugin App Store provider; emergency contact.
10. **`post_trip_reconciliation`** — auto-create expense report (Flow E) from itinerary.
11. **`emit_audit_chain`** — trip record sealed.

#### I.4 Edge cases

- **Trip cancellation** — automatic re-booking or refund per tenant policy.
- **Schedule change** — auto-update itinerary + calendar.
- **Visa requirements** — per-destination check; remind employee.
- **Health requirements** — vaccinations, COVID protocols per destination.
- **Risk-level travel** — high-risk destinations require additional approval + security briefing.

---

### Flow J — Procurement / Purchase Order

#### J.1 Purpose

Request, approve, issue, fulfill, and reconcile purchase orders for goods + services. Competes with Coupa / SAP Ariba / Oracle Procurement.

#### J.2 Trigger

- **Employee submits PR (Purchase Requisition)** via Procurement Portal or `/procure` Messenger command.

#### J.3 Saga: `ProcurementSaga`

Steps:

1. **`receive_pr`** — `(requester, items, vendor?, estimated_amount, business_justification, project_id?)`.
2. **`vendor_check`** — if new vendor, route through vendor-onboarding sub-saga (KYC, tax-ID collection, certificate validation).
3. **`approval_chain`** — per amount + category; finance approval; legal approval for contracts.
4. **`issue_po`** — generate PO number; send to vendor; legally-binding (e-sign saga Flow C if needed).
5. **`receive_goods_services`** — receiving record; 3-way match (PO + receipt + invoice).
6. **`invoice_processing`** — vendor invoice received; matched; routed for payment approval.
7. **`payment_authorization`** — finance approves payment; coordinate with payments substrate (reserved; via Plugin App Store integration today).
8. **`emit_audit_chain`** — SOX-compliant audit trail.

#### J.4 Edge cases

- **Emergency procurement** — fast-path; post-hoc approval.
- **Vendor non-performance** — dispute resolution sub-saga.
- **Budget exceeded** — block; require budget-amendment approval.
- **Compliance flag** (sanctions, export-control) — block; legal review.

---

### Flow K — Internal Announcement

#### K.1 Purpose

Broadcast information from leadership / HR / comms team to defined audience segments with tracking + acknowledgement. Competes with Workplace by Meta / Yammer / Slack announcements / SnapComms.

#### K.2 Trigger

- **Comms team drafts announcement** in Application Shell announcement composer.

#### K.3 Saga: `AnnouncementSaga`

Steps:

1. **`compose`** — author drafts content; Intelligence-assisted writing; tone/inclusivity check.
2. **`select_audience`** — per Cedar audience expression (entire tenant / specific dept / locale / role).
3. **`select_channels`** — Mail + Messenger + Mobile push + Application Shell home banner; pick subset.
4. **`schedule`** — immediate or future send.
5. **`require_acknowledgement?`** — for policy updates / mandatory reads, require explicit ack.
6. **`send`** — fan out via comms-email + messenger; track delivery + read receipts.
7. **`track_engagement`** — open rate, click rate, ack rate.
8. **`follow_up_unread`** — reminder to non-readers if acknowledgement required.
9. **`emit_audit_chain`** — for mandatory acknowledgements, record per-employee ack (compliance: training certification, code-of-conduct, harassment-policy).

#### K.4 Edge cases

- **Time-sensitive emergency announcement** (security incident, building closure) — skip approval; immediate fan-out; SMS fallback for critical.
- **Multi-language audience** — Intelligence translates; per-locale rendering.
- **CEO-tier announcement** — special template; video-message integration.

---

### Flow L — Project / Task Management

#### L.1 Purpose

Plan, track, and complete projects + tasks across team. Competes with Asana / Linear / Jira / Monday.com / ClickUp.

#### L.2 Trigger

- **Project created** in `tasks` µservice; **Task assigned** to team member.

#### L.3 Saga: `ProjectTaskSaga`

Steps:

1. **`project_setup`** — create project, milestones, initial tasks; team membership; Workflow Studio template for project-specific automation.
2. **`task_assignment`** — assign tasks; notify assignees via Messenger + Mail.
3. **`progress_tracking`** — daily standups in Messenger; weekly check-ins; milestone reviews via Meet (Flow D).
4. **`task_completion`** — assignee marks done; reviewer verifies; emit `TaskCompleted` event.
5. **`risk_detection`** — Intelligence flags overdue tasks, blocked dependencies, resource conflicts.
6. **`milestone_review`** — meeting saga; review recordings posted; lessons-learned doc.
7. **`project_close`** — final retro; deliverables archived in drive; lessons-learned in community/knowledge-base.

#### L.4 Edge cases

- **Cross-team dependencies** — explicit dependency declaration; auto-notification on blockers.
- **Scope change** — explicit scope-change request; approval chain.
- **Team-member departure** — task reassignment via offboarding saga (Flow G).

---

### Flow M — Compliance Training

#### M.1 Purpose

Assign mandatory training (security, harassment, ethics, code-of-conduct, jurisdiction-specific) to employees; track completion; certify; renew on cycle. Competes with KnowBe4 / Workday Learning / SAP SuccessFactors Learning.

#### M.2 Trigger

- **New hire onboarding** assigns Day-1 training (Flow F integration).
- **Annual cycle** assigns refreshers.
- **Role change** assigns new role-required training.
- **Incident-driven** assigns remedial training.

#### M.3 Saga: `ComplianceTrainingSaga`

Steps:

1. **`assignment`** — assign training module(s) to employee; deadline.
2. **`notification`** — Mail + Messenger + Application Shell home banner.
3. **`take_training`** — employee completes (videos, reading, quiz); content from training-content Plugin App Store providers or in-house authoring.
4. **`pass_check`** — quiz passing required; re-attempts allowed.
5. **`certification`** — generate certificate; e-sign acknowledgement (Flow C); retain per jurisdiction (KR PIPA training records 3y; US harassment training records varies by state — CA 2y, NY 1y).
6. **`overdue_escalation`** — reminders; manager + HR notified.
7. **`emit_audit_chain`** — completion record; regulator-queryable.

#### M.4 Edge cases

- **Failed quiz** — retake; if repeated fail, manager + HR involved.
- **Mandatory deadline missed** — automatic compliance flag; may affect compensation eligibility (per tenant policy).
- **Jurisdictional training** — different per locale (CA harassment training; KR 직장 내 괴롭힘 prevention training; EU GDPR training).

---

### Flow N — Document Collaboration (Review + Sign-Off)

#### N.1 Purpose

Author + collaborate + review + approve documents (proposals, specs, plans, contracts-pending-signing); integrates with drive + docs/sheets/slides for the editing surface + workflow for the approval chain. Competes with Google Docs comments + Sharepoint approvals + Notion review.

#### N.2 Trigger

- **Author creates doc** in `docs`/`sheets`/`slides`/`drive`; requests review/approval.

#### N.3 Saga: `DocumentCollaborationSaga`

Steps:

1. **`doc_created`** — doc exists; author requests review.
2. **`reviewer_selection`** — author picks reviewers + approvers; sequence (sequential / parallel) and required-vs-optional.
3. **`notify_reviewers`** — Mail + Messenger with deep link; show review-deadline.
4. **`reviewer_comments`** — inline comments in doc; threaded; author resolves.
5. **`approver_decision`** — approve / request-changes / reject.
6. **`revision_loop`** — if changes requested, author revises; re-route to reviewers as needed.
7. **`final_approval`** — all approvers signed-off.
8. **`distribution`** — publish doc; notify final audience; for legally-binding, route to e-sign saga (Flow C).
9. **`retention_policy`** — apply per-doc-class retention.
10. **`emit_audit_chain`** — version history sealed; for compliance, signed PDF version.

#### N.4 Edge cases

- **Reviewer unresponsive** — escalate after deadline.
- **Conflicting reviewer feedback** — author mediation; escalation to skip-level if needed.
- **External reviewer** (non-tenant) — Cedar-gated external-share with time-bounded access.
- **Sensitive doc** (data class restricted) — review limited to authorised personnel; audit-chain on every view.

---

## 5. Cross-µservice integration points

### 5.1 Per-flow integration matrix

For each flow A-N, the orchestrator + UX surfaces + system-of-record + Cedar gates + audit-chain emissions:

| Flow | Orchestrator | UX surfaces | System-of-record (writes) | Reads from | Cedar gates required | Audit-chain emissions |
|---|---|---|---|---|---|---|
| **A — Clock In/Out** | workflow-engine | messenger card, mail, mobile mini-app, voice intent, NFC, geofence | hr (timesheet) | hr (employment), tenancy (policy), calendar (holidays) | ClockIn, WriteTimesheet, ReadGeolocation | AttendanceClockIn, AttendanceClockOut, LatenessEscalation, AutoClockOut |
| **B — Leave Approval** | workflow-engine | messenger card, mail, mobile, voice, HR portal | hr (leave balance + record), calendar (OOO) | hr (balance, manager hierarchy), calendar (team availability), policy-engine (jurisdiction) | LeaveRequest, ApproveLeave, WriteOOO | LeaveRequestSubmitted, LeaveApproved, LeaveDenied, LeaveCancelled |
| **C — E-Sign** | workflow-engine | mail, messenger card, mobile, sign-pad | drive (signed PDF), audit-chain (cert chain) | drive (master PDF), identity (signer cert), intelligence (field extraction) | UploadDoc, SignDoc, ApplyCert, RetrieveSigned | ESignInitiated, ESignSignerSigned, ESignCompleted, ESignDeclined, ESignExpired |
| **D — Meeting Schedule** | workflow-engine | calendar app, messenger card, mail, voice, Calendly-link, mobile | calendar (event), meet (room), docs (agenda), tasks (action-items) | calendar (availability), tenancy (cross-tenant grant), intelligence (slot ranking) | CreateEvent, ReserveRoom, ProvisionMeet, ExternalAttendee | EventCreated, MeetingStarted, MeetingEnded, RecordingProduced |
| **E — Expense Report** | workflow-engine | mobile camera, mail, expense portal, messenger | hr/payroll (reimbursement), audit-chain | drive (receipt), intelligence (OCR), tenancy (policy), fx-rate provider | SubmitExpense, OcrReceipt, ApproveExpense, RouteToPayroll | ExpenseSubmitted, ExpenseApproved, ExpenseReimbursed, ExpensePolicyViolation |
| **F — Onboarding** | workflow-engine | application-shell, mail, messenger, mobile, e-sign | hr (record), identity (principal), drive (signed docs), calendar (events), procurement (equipment) | hr, identity, policy-engine | CreateEmployee, ProvisionIdentity, AssignRole, OrderEquipment | OnboardingStarted, OnboardingDocsSigned, OnboardingDay1, OnboardingProbationEnd |
| **G — Offboarding** | workflow-engine | application-shell, mail, messenger | hr (terminated), identity (revoked), drive (legal-hold), payroll (final paycheck) | hr, identity, audit-chain (legal-hold check) | RevokeAccess, TerminateEmployee, ComputeFinalPay | OffboardingStarted, AccessRevoked, FinalPaycheckIssued, OffboardingComplete |
| **H — Performance Review** | workflow-engine | application-shell, forms, calendar (review meeting), e-sign (ack) | hr (review record), tasks (dev plan) | hr, tenancy (cycle config), intelligence (bias detection) | StartReview, SubmitAssessment, FinalizeRating, AcknowledgeReview | ReviewStarted, AssessmentSubmitted, ReviewFinalized, ReviewAcknowledged |
| **I — Travel Request** | workflow-engine | messenger, mail, travel portal, mobile | travel-provider (booking), calendar (itinerary), payroll (advance) | tenancy (policy), policy-engine (jurisdiction), risk-advisory | RequestTravel, BookFlight, BookHotel, AdvanceTravelPay | TravelRequested, TravelBooked, TravelStarted, TravelCompleted |
| **J — Procurement** | workflow-engine | procurement portal, messenger, mail | procurement-system (PO), drive (PO doc), audit-chain | tenancy (budget), policy-engine (vendor approval) | SubmitPR, IssuePO, ReceiveGoods, AuthorizePayment | PrSubmitted, PoIssued, GoodsReceived, InvoicePaid |
| **K — Announcement** | workflow-engine | mail, messenger, mobile push, application-shell banner | audit-chain (mandatory ack records) | tenancy (audience), policy-engine | ComposeAnnouncement, SendAnnouncement, RequireAck | AnnouncementSent, AnnouncementAcked, AnnouncementOverdue |
| **L — Project Task** | workflow-engine | tasks, messenger, application-shell, meet | tasks (project + tasks) | hr (team), calendar (capacity), intelligence (risk) | CreateProject, AssignTask, CompleteTask | ProjectCreated, TaskAssigned, TaskCompleted, ProjectClosed |
| **M — Compliance Training** | workflow-engine | application-shell, mail, messenger | training-system (completion), audit-chain | tenancy (assignments), policy-engine (jurisdiction-required) | AssignTraining, TakeTraining, CertifyCompletion | TrainingAssigned, TrainingCompleted, TrainingOverdue, TrainingCertified |
| **N — Doc Collaboration** | workflow-engine | docs/sheets/slides editors, messenger, mail | drive (versioned docs), audit-chain | docs, tenancy (sharing policy), policy-engine | RequestReview, CommentDoc, ApproveDoc | ReviewRequested, ReviewCompleted, DocApproved, DocPublished |

### 5.2 Workflow event topics (cross-flow)

Workplace integration sagas publish to and consume from canonical event topics on the workflow-engine event-bus:

- `workplace.attendance.v1` — clock-in/out + lateness
- `workplace.leave.v1` — leave requests + decisions
- `workplace.esign.v1` — e-sign lifecycle
- `workplace.meeting.v1` — meeting lifecycle (forwarded from calendar.event.lifecycle.v1)
- `workplace.expense.v1` — expense lifecycle
- `workplace.onboarding.v1` — onboarding milestones
- `workplace.offboarding.v1` — offboarding milestones
- `workplace.performance.v1` — performance-review lifecycle
- `workplace.travel.v1` — travel lifecycle
- `workplace.procurement.v1` — procurement lifecycle
- `workplace.announcement.v1` — announcement delivery + ack
- `workplace.project.v1` — project + task lifecycle
- `workplace.training.v1` — compliance-training lifecycle
- `workplace.doc-collab.v1` — doc-review lifecycle

### 5.3 Ontology object types introduced

New object types in `oya/ontology/` for workplace integration:

- `TimesheetEntry`, `LatenessRecord`, `WorkSchedule`, `WorkAreaGeofence`
- `LeaveRequest`, `LeaveBalance`, `LeavePolicy`, `OOOEntry`
- `ESignDocument`, `Signature`, `SignerInvitation`, `SignatureCertificateChain`
- `MeetingScheduleProposal`, `RoomReservation` (extends `calendar`)
- `ExpenseReport`, `ExpenseLineItem`, `Receipt`, `ReimbursementRequest`, `MileageTrip`
- `OnboardingChecklist`, `OnboardingMilestone`, `OnboardingTask`
- `OffboardingChecklist`, `KnowledgeTransferItem`, `AccessRevocationRecord`
- `PerformanceCycle`, `PerformanceAssessment`, `PerformanceRating`, `DevelopmentPlan`, `PIP`
- `TravelRequest`, `Itinerary`, `TravelBookingLeg`
- `PurchaseRequisition`, `PurchaseOrder`, `Vendor`, `VendorInvoice`
- `Announcement`, `AnnouncementAudience`, `AnnouncementAcknowledgement`
- `Project`, `Milestone`, `TaskAssignment` (extends `tasks`)
- `TrainingModule`, `TrainingAssignment`, `TrainingCompletionCertificate`
- `DocReviewSession`, `DocReviewComment`, `DocApprovalDecision`

Each object type carries `data_class` annotation per ADR-0099; per-tenant + per-jurisdiction retention; legal-hold compatible; audit-chain integrated.

### 5.4 Promotion-gated workforce µservices

For Workplace Integration to reach M04 stable, workforce dependencies must be explicit repo-local service anchors or remain blocked behind promotion gates per ADR-0245 D-6:

- **`oya/hr/`** — repo-local HR service anchor for records of truth (employment, leave-balance, org-chart, compensation-summary). Promotion remains certification-gated: PRD, threat-model, DPIA, manifest tier=product tier_subtype=product-consumer-hr, IaC, SOC 2 Type II employment-data controls, and per-jurisdiction labor-law overlay packs (KR pack-kr-labor, EU pack-eu-working-time, US pack-us-flsa).
- **`oya/payroll/`** — repo-local payroll service anchor for payroll calculation + paycheck issuance; coordinates with banking via reserved `payments` µservice + Plugin App Store integrations (ADP, Gusto, Justworks, Rippling). Promotion remains gated on per-jurisdiction tax authority registrations (US-IRS, KR-NTS, EU-VAT-MOSS, etc.) and labor-law packs.
- **`oya/compensation/`** — not present as a repo-local service anchor in this checkout; compensation remains promotion-blocked until a founder-/governance-approved service anchor, PRD, threat model, DPIA, manifest, IaC, 409A provider integration, and ASC 718 controls exist.

Until every promotion gate lands, Workplace Integration sagas that need HR/payroll/compensation data use the existing service anchors where present and otherwise integrate via Plugin App Store providers (BambooHR, Rippling, Gusto, ADP) with their respective APIs; they must not silently assume an unavailable record-of-truth service.

### 5.5 Cedar policy fragments

Cedar fragments live in `oya/policy-engine/fragments/workplace-integration/`:

```
workplace-integration/
├── flow-a-clocking/
│   ├── base.cedar
│   ├── pack-kr-labor.cedar
│   ├── pack-eu-working-time.cedar
│   ├── pack-us-flsa.cedar
│   ├── pack-us-ca-overtime.cedar
│   ├── pack-jp-labor.cedar
│   └── geolocation-consent.cedar
├── flow-b-leave/
│   ├── base.cedar
│   ├── pack-kr-annual-leave.cedar
│   ├── pack-eu-wlb-directive.cedar
│   ├── pack-us-fmla.cedar
│   ├── pack-us-ca-cfra-pfl.cedar
│   └── manager-approval-chain.cedar
├── flow-c-esign/
│   ├── base.cedar
│   ├── tier-simple.cedar
│   ├── tier-advanced-eidas-ades.cedar
│   ├── tier-qualified-eidas-qes.cedar
│   ├── pack-kr-electronic-signature-act.cedar
│   ├── pack-jp-e-signature-law.cedar
│   └── revocation-check.cedar
└── ... (flow-d through flow-n)
```

Each fragment is signed by org root key (per ADR-0242 bootstrap step 5) and refreshed on per-pack-overlay updates.

### 5.6 Audit-chain emission contract

Every workplace flow emits to `oya/audit-chain/` per ADR-0028 envelope:

```json
{
  "tenant_id": "tenant-acme.engineering",
  "principal": "tenant-acme.engineering.employee.7421",
  "action": "LeaveRequest.Approved",
  "resource": "LeaveRequest:lr-abc123",
  "saga_run_id": "wfr-xyz-789",
  "step_id": "handle_decision",
  "timestamp": "2026-05-20T14:32:18.341Z",
  "data_class": "EMPLOYMENT_LEAVE",
  "decision": "approve",
  "approver_principal": "tenant-acme.engineering.manager.114",
  "policy_fragment_versions": ["base@v3", "pack-kr-annual-leave@v2"],
  "evidence_hashes": ["sha256:..."],
  "retention_floor": "5y",
  "legal_hold_compatible": true
}
```

Merkle-sealed per Bominal ADR-0028; retention floor per pack (KR 5y / US 4-7y / EU 5y typical).

---

## 6. User stories

Each user story has a stable AC-ID. Per `agent-durable-goal.json#spec_contract.acceptance_criteria_rule`, ACs are append-only and back-linked from tests.

### Flow A — Clocking In/Out

**AC-WI-01 — Mobile one-tap clock-in (employee)**
- **Persona**: hourly employee
- **Precondition**: employee has active employment record + consented to attendance tracking
- **Given**: employee opens mobile app at 09:00 KST start-of-shift
- **When**: employee taps "Clock In" button on home screen
- **Then**: ClockingInSaga triggers; geolocation captured (if consented); timesheet entry created in HR; confirmation shown within 500ms p99; audit-chain event sealed within 1s
- **Edge**: late arrival → manager notified; outside geofence → soft-flag or refuse per tenant policy

**AC-WI-02 — Messenger `/in` slash command (employee)**
- **Persona**: desk-bound employee
- **Precondition**: employee in #attendance channel
- **Given**: employee types `/in` in Messenger
- **When**: slash command processed
- **Then**: ClockingInSaga triggers; confirmation card replies "Clocked in at HH:MM. [Adjust]"; saga state visible to manager

**AC-WI-03 — Voice clock-in via Siri (employee)**
- **Persona**: commuting employee with hands-busy
- **Given**: employee says "Hey Siri, clock me in at oyatie"
- **When**: App Intent dispatches to oyatie mobile app
- **Then**: ClockingInSaga triggers; voice confirmation "You're clocked in at 09:14"; haptic confirmation

**AC-WI-04 — Auto-clock-out for missing clock-out (employee)**
- **Persona**: forgetful employee
- **Precondition**: employee clocked in at 09:00 but did not clock out by 19:00 (1h after 18:00 shift end)
- **Given**: 60min post-shift-end timer fires
- **When**: auto-clock-out triggers
- **Then**: timesheet entry auto-created with `auto_clocked_out: true`; employee notified via Messenger with one-tap adjust; manager notified for visibility

**AC-WI-05 — Buddy-punching detection (security)**
- **Persona**: tenant security officer
- **Precondition**: same device fingerprint attempts clock-in for two different employees
- **Given**: device-fingerprint repeats across employees
- **When**: ClockingInSaga step `capture_device_fingerprint` detects collision
- **Then**: second attempt flagged; security review event emitted; subsequent clock-ins from device blocked pending review

### Flow B — Vacation/Leave Approval

**AC-WI-06 — Mail-based natural-language leave request (employee)**
- **Persona**: employee
- **Given**: employee sends "I'd like to take vacation from June 5 to June 10" to leave@<tenant>
- **When**: Mail received; Intelligence parses dates + type
- **Then**: LeaveRequestSaga starts; employee receives Messenger confirmation card with parsed request to confirm + edit; on confirm, saga advances

**AC-WI-07 — Manager one-click approve (manager)**
- **Persona**: direct manager
- **Precondition**: employee leave request with balance OK + team coverage OK
- **Given**: manager receives Messenger card with full context (balance, team OOO overlay, employee history)
- **When**: manager taps "Approve"
- **Then**: LeaveRequestSaga step `handle_decision` advances; HR record updated; OOO calendar block created; employee notified; total interaction time < 5s

**AC-WI-08 — Insufficient balance handling (employee)**
- **Persona**: employee requesting more days than balance
- **Given**: employee requests 10 days; balance is 7
- **When**: LeaveRequestSaga step `validate_balance` detects shortfall
- **Then**: employee shown options (reduce to 7, request 3 unpaid, cancel); choice recorded; saga branches

**AC-WI-09 — Manager-OOO escalation (employee)**
- **Persona**: employee whose direct manager is OOO during approval window
- **Given**: 72h after submission, no decision from direct manager
- **When**: saga timer fires
- **Then**: auto-escalate to skip-level manager; original manager notified for visibility

**AC-WI-10 — Cross-jurisdiction leave (remote employee)**
- **Persona**: employee residing in Berlin working for US-incorporated tenant
- **Given**: employee submits 4-week leave request
- **When**: LeaveRequestSaga step `policy_check` runs
- **Then**: EU Working Time Directive overlay applied (not US FMLA); 4-week min annual leave honoured; approved without question

### Flow C — E-Signing

**AC-WI-11 — HR uploads contract; signer e-signs (HR + employee)**
- **Persona**: HR + new-hire signer
- **Given**: HR uploads offer-letter PDF to onboarding queue
- **When**: ESignSaga triggers; Intelligence extracts signature fields
- **Then**: signer receives Mail link; opens; reviews; signs; PDF stored; audit-chain sealed; legally-binding per US ESIGN

**AC-WI-12 — Mobile finger-signature (signer)**
- **Persona**: signer on mobile
- **Given**: signer opens e-sign link on iPhone
- **When**: signer draws signature with finger on touch screen
- **Then**: signature vectorised + saved to profile; applied to PDF; signature replays cleanly on subsequent signings

**AC-WI-13 — Signer declines with revision request (signer)**
- **Persona**: signer disagrees with section
- **Given**: signer sees contract with disagreeable section
- **When**: signer taps "Request revision" + provides reason
- **Then**: ESignSaga branches to revision-sub-saga; originator notified; revised doc invalidates prior signatures; re-route to signers

**AC-WI-14 — Qualified signature for EU (eIDAS QES)**
- **Persona**: EU employee signing legally-binding doc
- **Given**: tenant policy requires `qualified` tier for employment contracts in EU jurisdiction
- **When**: ESignSaga step `signer_authentication` runs
- **Then**: signer prompted for qualified-certificate (D-Trust / Bundesdruckerei); PAdES-LTV signature applied; eIDAS-compliant final PDF

**AC-WI-15 — Expiration without all signers (originator)**
- **Persona**: HR sending doc with 30-day expiration
- **Given**: 30 days pass; one signer has not signed
- **When**: ESignSaga step `all_signed_or_expired` reaches expiration
- **Then**: saga ends in `expired` state; originator notified; partial signatures invalidated (no final PDF); audit-chain records expired

### Flow D — Meeting Scheduling

**AC-WI-16 — Slash-command schedule with 3-attendee meeting (organizer)**
- **Persona**: project lead
- **Given**: organizer types `/schedule Jane Marcus tomorrow 30min`
- **When**: MeetingScheduleSaga triggers
- **Then**: 3 candidate slots shown ranked by Intelligence; one-tap pick; calendar event + meet link + agenda doc created; invitations sent via iMIP

**AC-WI-17 — Cross-tenant meeting with external attendee (organizer)**
- **Persona**: sales rep meeting customer
- **Given**: customer is at different oyatie tenant with opted-in cross-tenant grant
- **When**: MeetingScheduleSaga step `resolve_attendees` runs
- **Then**: customer's free/busy projection fetched (no event leak); slot suggested; iMIP invite sent; customer's calendar shows event in their tenant

**AC-WI-18 — Calendly-style public link self-scheduling (external user)**
- **Persona**: external candidate scheduling interview
- **Given**: candidate visits `oyatie.com/meet/tenant-acme/recruiter-1`
- **When**: candidate picks slot
- **Then**: event auto-created; recruiter calendar updated; candidate confirmation sent; rate-limited per tenant policy

**AC-WI-19 — Recurring meeting with single-occurrence edit (organizer)**
- **Persona**: organizer with weekly team standup
- **Given**: weekly recurring meeting exists; one week falls on holiday
- **When**: organizer edits that single occurrence
- **Then**: RFC 5545 EXDATE applied for that occurrence; remaining occurrences unchanged; attendees notified of single change only

**AC-WI-20 — Post-meeting AI summary + action items (attendee)**
- **Persona**: meeting attendee
- **Precondition**: meeting recorded + transcribed
- **Given**: meeting ends
- **When**: post-meeting summarisation runs
- **Then**: Intelligence summarises transcript; action items extracted with assignees; tasks created in `tasks` µservice; mail summary sent to attendees

### Flow E — Expense Report

**AC-WI-21 — Receipt photo to expense in under 30 seconds (employee)**
- **Persona**: employee at lunch
- **Given**: employee snaps photo of receipt
- **When**: ExpenseSaga triggers; OCR runs
- **Then**: amount, merchant, date, category populated within 5s p99; employee reviews + submits in single screen; total flow < 30s

**AC-WI-22 — Auto-detected corp card txn (employee)**
- **Persona**: employee using corp card
- **Given**: employee swipes corp card at coffee shop
- **When**: card-issuer webhook fires; ExpenseSaga pending-receipt state
- **Then**: Messenger card appears within 30s "Card charged $4.50 at Blue Bottle. Add receipt?"; employee taps to attach; saga advances

**AC-WI-23 — Foreign currency with FX (employee)**
- **Persona**: employee on business trip in Europe
- **Given**: receipt in EUR; tenant home currency USD
- **When**: ExpenseSaga step `fx_conversion` runs
- **Then**: txn-day exchange rate fetched; both EUR and USD recorded; receipt categorised for VAT recovery

**AC-WI-24 — Policy violation flagged (employee + manager)**
- **Persona**: employee dining over per-meal cap
- **Given**: meal $80; tenant cap $50
- **When**: ExpenseSaga step `policy_check` runs
- **Then**: violation flagged; employee can add justification; manager card shows violation prominently; approver can override with reason

### Flow F — Onboarding

**AC-WI-25 — Pre-Day-1 setup automation (new hire + HR)**
- **Persona**: new hire 14 days before start
- **Given**: HR creates new-hire record
- **When**: OnboardingSaga starts
- **Then**: T-14d welcome mail sent; T-7d identity provisioned; T-7d equipment ordered; T-3d calendar populated; T-1d welcome packet; Day-1 ready

**AC-WI-26 — Day-1 e-signed policy bundle (new hire)**
- **Persona**: new hire on Day-1
- **Given**: bundle of 6 docs (handbook, NDA, IP-assignment, harassment, COC, IT-AUP)
- **When**: ESignSaga (Flow C) routed by OnboardingSaga
- **Then**: all 6 docs signed in one flow with progress bar "Doc 3 of 6"; each acknowledgement audit-chain sealed

### Flow G — Offboarding

**AC-WI-27 — Same-day termination access revocation (security)**
- **Persona**: security officer
- **Precondition**: employee terminated mid-day
- **Given**: HR initiates offboarding with `last_day = today`
- **When**: OffboardingSaga step `post_last_day_revoke` accelerated
- **Then**: all access revoked within 60s; audit-chain sealed; final paycheck computation initiated; equipment-return label sent

**AC-WI-28 — Legal-hold preservation during offboarding (compliance)**
- **Persona**: tenant compliance officer
- **Precondition**: departing employee under litigation legal-hold
- **Given**: OffboardingSaga starts
- **When**: step `legal_hold_check` runs
- **Then**: all data flagged for retention; audit-chain records hold; pseudonymisation deferred until hold released

### Flow H — Performance Review

**AC-WI-29 — Cycle kickoff for entire org (HR)**
- **Persona**: HR head
- **Given**: cycle config submitted
- **When**: PerformanceReviewSaga cycle starts
- **Then**: all eligible employees notified; self-assessments solicited; peer-feedback nominations open; calibration sessions scheduled

**AC-WI-30 — Bias detection alert (HR)**
- **Persona**: HR head
- **Given**: manager has consistently lower ratings for protected class
- **When**: PerformanceReviewSaga step `final_rating_and_actions` runs Intelligence bias-check
- **Then**: pattern flagged; HR notified; calibration session re-opened

### Flow I — Travel

**AC-WI-31 — Travel request with manager approval (employee)**
- **Persona**: employee planning business trip
- **Given**: employee submits travel request for $2,500
- **When**: TravelRequestSaga triggers; above $1,000 threshold needs finance approval
- **Then**: manager + finance approval cards; on approval, booking integrations (via Plugin App Store) issue tickets; calendar populated with itinerary

### Flow J — Procurement

**AC-WI-32 — Software purchase with 3-way match (requester + finance)**
- **Persona**: engineer needing software license
- **Given**: requester submits PR for $5,000 SaaS subscription
- **When**: ProcurementSaga advances through approvals
- **Then**: PO issued; vendor invoice received; receipt confirmed; 3-way match passes; payment authorized

### Flow K — Announcement

**AC-WI-33 — CEO all-hands announcement with mandatory ack (CEO)**
- **Persona**: CEO sending policy update
- **Given**: CEO drafts announcement with `require_ack = true`
- **When**: AnnouncementSaga sends
- **Then**: every employee receives via Messenger + Mail; ack tracked; non-readers nudged at 24h + 72h; audit-chain records per-employee ack for compliance

### Flow L — Project Task

**AC-WI-34 — Project setup with milestone tracking (project lead)**
- **Persona**: project lead launching new initiative
- **Given**: lead creates project with 4 milestones
- **When**: ProjectTaskSaga initialises
- **Then**: project + milestones + initial tasks created; team notified; weekly check-ins scheduled; risk-detection running

### Flow M — Compliance Training

**AC-WI-35 — Annual harassment-prevention training (CA employee)**
- **Persona**: California employee
- **Given**: annual cycle; CA requires harassment training every 2 years
- **When**: ComplianceTrainingSaga assigns module
- **Then**: employee notified; takes module; quiz passed; certificate generated; CA-compliant record retained 2y

### Flow N — Doc Collaboration

**AC-WI-36 — Doc review with parallel approvers (author)**
- **Persona**: author seeking 3 approver sign-offs
- **Given**: author selects 3 approvers in parallel mode
- **When**: DocumentCollaborationSaga routes
- **Then**: all 3 approvers receive simultaneously; doc finalises when all approve; revision-loop triggers if any request changes

### Cross-flow

**AC-WI-37 — Workflow Studio template customisation (HR power user)**
- **Persona**: HR head customising leave-approval flow
- **Given**: tenant wants 2-level approval (direct + skip)
- **When**: HR opens Workflow Studio; loads LeaveRequestSaga template; adds skip-level approval node; saves
- **Then**: tenant variant registered with workflow-engine; subsequent requests use 2-level flow; round-trip byte-equality maintained

**AC-WI-38 — Plugin App Store extension (tenant admin)**
- **Persona**: tenant admin installing Concur-style pre-approval plugin
- **Given**: plugin registered handler for `expense.pre-approval`
- **When**: tenant installs from Plugin App Store
- **Then**: subsequent ExpenseSaga runs invoke plugin at pre-approval step; plugin sandboxed in Wasmtime; Cedar-gated; audit-chain records plugin invocation

**AC-WI-39 — `oyatie` tenant dogfooding (oyatie engineer)**
- **Persona**: oyatie engineer
- **Given**: engineer is principal `oyatie.engineer.jasonlee`
- **When**: engineer requests vacation, signs ADR, books meeting, submits expense
- **Then**: all flows work identically to customer tenant; audit-chain segregated to `oyatie.*` stream; DSAR-compatible

**AC-WI-40 — Per-jurisdiction policy overlay correctness (compliance auditor)**
- **Persona**: auditor verifying KR labor compliance
- **Given**: KR-jurisdiction tenant
- **When**: auditor queries Workplace Integration evidence pack
- **Then**: every leave / clocking / overtime decision evidences applied policy fragment + applied jurisdiction overlay; immutable per audit-chain

### Additional cross-flow user stories

**AC-WI-41 — Mobile offline clock-in (field worker)**
- **Persona**: field worker without network
- **Given**: worker has no connectivity
- **When**: worker taps clock-in; mobile queues locally
- **Then**: on reconnect, queued event submitted; saga reconciles with idempotency key; if duplicate (worker clocked in via another channel meanwhile), idempotent no-op

**AC-WI-42 — Voice-triggered leave from car (employee commuting)**
- **Persona**: employee driving
- **Given**: employee says "Hey Siri, request sick day for tomorrow"
- **When**: voice intent processed
- **Then**: confirmation read aloud; on confirm, LeaveRequestSaga starts; status updated when manager approves

**AC-WI-43 — Cross-locale document with mixed-language signers**
- **Persona**: cross-border employment contract signer
- **Given**: contract in English; signer speaks Korean
- **When**: signer opens document
- **Then**: Intelligence-generated Korean summary shown alongside original English; signature still binds original English text; audit-chain notes language assistance

**AC-WI-44 — Manager bulk-approve daily expenses (manager)**
- **Persona**: manager with 12 pending small expenses
- **Given**: all within policy + below auto-approve threshold
- **When**: manager opens bulk-approval view
- **Then**: all 12 visible; select-all; approve-all with single biometric; audit-chain records bulk decision with each individual record

**AC-WI-45 — Cell-failover during long-running saga (saga reliability)**
- **Persona**: tenant under cell-failover event
- **Given**: ESignSaga in-progress when home cell degraded
- **When**: cell failover to paired cell
- **Then**: saga resumes from last persisted step per ADR-0252; signers' partial signatures preserved; no duplicate sign emails; durability invariant honoured

---

## 7. UX strive / avoid

### 7.1 Strive

1. **1-tap actions for repetitive flows.** Clock-in, expense-receipt-snap, leave-day-pick, meeting-RSVP: every recurring action accomplishable in a single tap on mobile + single click on desktop. Measured: median taps-to-success per flow ≤ 2.
2. **Rich Messenger + Mail cards with structured action buttons.** Per Slack Block Kit + Microsoft Adaptive Card + Apple/Google rich-notification specs. No "click here to open the portal" dead-end links — actions are inline, in-context.
3. **Voice-trigger compatibility.** Apple App Intents (iOS 18+) and Android App Actions registered for every primary workplace flow. Hands-free clock-in, leave request, meeting schedule, expense voice-memo.
4. **Passive mode where consented.** Geofence-based clock-in; ambient-recording-based meeting capture; passive-time-tracking. Always opt-in, transparent indicators when active, one-tap disable. Default OFF (KR PIPA + EU GDPR explicit-consent compliance).
5. **Inline progress on multi-step flows.** E-sign progress bar ("Doc 3 of 6"); onboarding checklist ("Day 1 of 90"); travel itinerary ("3 of 5 legs booked"); approval-chain visibility ("Awaiting manager → finance").
6. **Deep links to relevant context.** Manager-card "Approve leave?" deep-links to employee's balance + team calendar + their prior leave patterns. Reviewer-card "Approve expense?" deep-links to expense + receipt + employee's expense history. Never make the user search for context.
7. **Mobile-first.** Every flow fully usable on mobile (≥ 60% of expected usage is mobile). No "open on desktop to complete" dead-ends. Touch targets ≥ 88pt × 88pt. Single-thumb-reachable critical actions.
8. **Accessibility-by-construction.** WCAG 2.2 AA minimum; AAA for critical safety/compliance UI (harassment-reporting, anonymous whistleblower). Full keyboard nav; screen-reader compatible; high-contrast mode; configurable text size; reduced-motion options.
9. **Intelligence-augmented authoring.** Compose-assist for announcements (tone, inclusivity, clarity); auto-summary of long approvals; auto-categorise expenses; auto-extract signature fields; natural-language leave requests parsed.
10. **Workflow Studio as the customisation surface.** Tenant HR / ops authors customisations visually; round-trip-stable spec; never need to drop to code for routine workplace flows.
11. **Multi-language.** Every flow rendered in ≥ 20 languages; per-user preference; Intelligence translates inline content (announcement bodies, meeting summaries, doc summaries).
12. **Optimistic UI with clear retry semantics.** Tap-to-clock-in shows confirmation immediately, retries network call in background; surfaces error only on hard fail with clear retry path.
13. **Offline-tolerant.** Critical flows queue locally on disconnect; reconcile on reconnect; idempotent per ADR-0252. Mobile-app stores last 30 days of attendance + leave-balance for offline view.
14. **Plugin extensibility transparent.** When a tenant has installed a plugin that extends a flow (e.g., a procurement-tool plugin), the plugin's actions are clearly attributed in the UI ("Approved via Concur Plugin v2.3.1") — no opaque "magic" behaviour.

### 7.2 Avoid

1. **5-page-form anti-pattern.** Workday's "fill out this entire form before submission" pattern is forbidden. Decompose into single-question messages (one decision per Messenger card; one field per mobile screen step). User progresses through micro-steps with clear progress.
2. **Separate-app context-switching.** ServiceNow's "open this approval portal, then go to that approval portal" pattern is forbidden. Every approval is inline in Messenger/Mail/Mobile.
3. **Paper-replicas (8.5x11-style form PDFs).** Concur's PDF-style expense forms are forbidden. Mobile-first, screen-native UI. PDFs only when legally required (e-sign output, regulator filing).
4. **Synchronous wait-on-approver.** No flow blocks user UX waiting for approver's decision. Submit → async wait → notification on decision. User can do other work meanwhile.
5. **Tech-speak in user-visible text.** No "Saga execution failed at step `validate_balance` with error ECONNREFUSED". User-visible: "We couldn't check your leave balance right now. Try again in a minute or contact HR."
6. **Hidden state.** Always show user where they are in a multi-step flow. Show what's pending, what's done, what's blocked, who's the blocker.
7. **Re-prompt fatigue.** If we've asked the user a question, don't ask again 5 minutes later. Coalesce + de-duplicate prompts.
8. **Modal-trap surveys.** Don't block user from doing their job to fill out a satisfaction survey. Side-bar nudge, not modal block.
9. **Manager-only visibility.** Don't hide leave-balance, expense-policy, performance-criteria from employees. Transparency by default.
10. **One-size-fits-all approval chains.** Don't hardcode "manager → director → VP" approval for every flow. Tenant policy + Workflow Studio customisation = each tenant's reality.
11. **Lossy escalations.** When escalating from manager-OOO to skip-level, carry full context (employee message, history, related approvals). Don't restart the conversation.
12. **English-only error messages.** Per ADR-0244 + i18n requirements, every user-visible string is localised; error messages especially.

### 7.3 Competitor comparison

| Competitor | What they do well | Where we strive to beat | Measurable target (M04 GA) |
|---|---|---|---|
| **ServiceNow ITSM + HR Service Delivery** | Enterprise approval routing depth; per-tenant customisation | Less form-heavy; mobile-first; messenger-native; faster median time-to-approval | Median leave-approval time < 4h (ServiceNow benchmark ~12h); mobile usage > 60% (ServiceNow ~30%); customer NPS > 50 (ServiceNow ~30) |
| **Workday HCM** | Comprehensive HR record; payroll integration; analytics depth | Workflow Studio for tenant customisation (vs. Workday's hardcoded flows); modern mobile UX; messenger integration; faster onboarding to first-value | Time-to-first-leave-request < 2 minutes from app open (Workday ~7 min); customisation without consultant (Workday requires Studio consultant per tenant) |
| **SAP Concur** | Travel + expense globally; receipt OCR | Mobile-first; messenger-native; faster receipt-to-submit; foreign-currency simpler; integrated with workplace messenger | Receipt-to-submit < 30s (Concur ~3 min); auto-OCR confidence ≥ 95% (Concur ~85%) |
| **BambooHR** | Mid-market simplicity; clean UI | Multi-jurisdiction labor-law overlay; Workflow Studio customisation; Plugin App Store ecosystem | Number of jurisdiction overlays at GA: ≥ 30 (BambooHR ~5); plugin marketplace size at GA: ≥ 100 plugins |
| **Microsoft 365 + Power Automate** | Tight Outlook integration; broad enterprise reach | Sovereign-per-pack; Workflow Studio more powerful than Power Automate for cross-product flows; better mobile messenger | Sovereign-cloud for KR / EU / regulated US tenants (M365 has gaps); workflow editor open-source-DSL (Power Automate is proprietary) |
| **Google Workspace + Apps Script** | Native collaborative editing; web-first | Workflow Studio + Workflow Engine substrate; HR-grade audit-chain (Apps Script has limited audit); first-class workplace flows (Workspace doesn't ship them) | Apps Script feature parity in Workflow Studio + 50% more workflow features; audit-chain Merkle-sealed per ADR-0028 (Workspace audit is logs-only) |
| **Notion teamspaces** | Modern UX; flexible doc + database | Durable saga execution (Notion automations are best-effort); compliance-grade audit; multi-jurisdiction overlay | Saga durability per ADR-0252 (Notion has no durability story); compliance posture HIPAA / SOC 2 / KR PIPA pack-certified |
| **Slack workflow builder** | Conversational interface; quick-build | Workflow Studio's spec-DSL + visual canvas + cross-µservice; durable execution; audit-chain | Cross-product workflows (Slack workflow stays in Slack); durable per ADR-0252; multi-jurisdiction overlay |
| **DocuSign** | E-sign market leader; broad integrations | Lower per-doc cost (in-house signing engine); deeper workplace integration (signing flows are natively part of onboarding/offboarding/leave/etc); sovereign-per-pack | Per-doc marginal cost zero (Concur ~$3/doc); eIDAS QES via in-house QTSP integration |
| **Adobe Sign** | Adobe ecosystem integration | Same as DocuSign | Same |
| **Calendly / x.ai** | Magical scheduling UX | Native to calendar µservice + workflow; cross-tenant federation; Plugin App Store extensions | Public-link self-scheduling at parity; cross-tenant grants supported (Calendly has limited cross-org); free for all tenants (Calendly per-seat) |
| **Expensify / Brex / Ramp** | OCR + auto-categorise; corp-card integration | Workflow Studio customisation; multi-jurisdiction; deeper HR integration; messenger-native approvals | OCR confidence ≥ 95%; receipt-to-submit < 30s; jurisdiction overlays ≥ 30 |
| **Lattice / 15Five** | Performance management UX | Cross-flow integration (performance ↔ compensation ↔ onboarding); messenger-native; Workflow Studio customisation | Cycle setup-time < 1 day (Lattice ~3-5 days); customisation no-code |
| **Greenhouse / Lever** | ATS + onboarding | Onboarding saga integrates with HR + identity + drive + calendar + messenger natively; ATS via Plugin App Store integration | Time-to-Day-1-ready < 7 days from offer-accept (Greenhouse ~10-14 days) |
| **Rippling** | Unified employee record + payroll | Workflow Studio (more flexible than Rippling's hardcoded flows); Plugin App Store (broader ecosystem); sovereign-per-pack | Number of tenant-customisable flows ≥ 50 (Rippling ~10); plugin marketplace size |
| **ChartHop** | Org-chart visualisation; HR analytics | Integrated with workplace flows (not separate analytics product); modern UX | Org-chart-to-action time < 30s (ChartHop ~2 min) |

---

## 8. Compliance

Workplace Integration touches employment data, which is among the most heavily regulated data classes globally. Per ADR-0240 sovereign-cloud + ADR-0251 compliance-pack certification levels + ADR-0117 jurisdiction-code inheritance:

### 8.1 Labor / Employment law

| Jurisdiction | Statute(s) | Workplace-integration application |
|---|---|---|
| **US (federal)** | FLSA 29 USC §§201-219 (Fair Labor Standards Act) | Overtime calc (Flow A clocking-out → payroll); minimum wage; non-exempt vs exempt classification |
| **US (federal)** | FMLA 29 USC §§2601-2654 (Family and Medical Leave Act) | Flow B leave: 12-week unpaid eligible leave |
| **US (federal)** | EEOC Title VII, ADA, ADEA | Flow H performance: bias detection; reasonable-accommodation tracking |
| **US (federal)** | I-9 / E-Verify | Flow F onboarding: I-9 e-sign + E-Verify integration via Plugin App Store |
| **US (state) CA** | Cal. Labor Code §§510, 512, 1198.5 | Daily overtime + meal break + payroll-record access |
| **US (state) NY** | NY Labor Law §§650-665 | State minimum wage; sexual harassment training (annual) |
| **EU** | Working Time Directive 2003/88/EC | 48h/week cap; min 11h daily rest; 4-week min annual leave |
| **EU** | Work-Life Balance Directive 2019/1158 | 4-month parental leave; carer's leave |
| **EU** | Posted Workers Directive 96/71/EC + 2018/957 | Cross-border workers apply host-country labor law |
| **EU** | Whistleblowing Directive 2019/1937 | Anonymous reporting channel (Flow N variant) |
| **UK** | Working Time Regulations 1998 | 28 days annual leave incl. bank holidays |
| **DE** | BUrlG (Bundesurlaubsgesetz) | 20 days min annual leave |
| **FR** | Code du travail L3141-3 | 30 days annual leave |
| **KR** | 근로기준법 (Labor Standards Act) | 40h/week regular + 12h overtime cap; 15+ days annual leave; 1.5x overtime premium; 출산휴가 90 days |
| **KR** | 남녀고용평등법 (Equal Employment Act) | Parental leave; equal pay |
| **KR** | 산업안전보건법 (Industrial Safety + Health Act) | Workplace safety reporting (Flow K/N) |
| **JP** | 労働基準法 (Labor Standards Act) | 40h/week + 36協定 overtime cap; 10-20 days annual leave |
| **JP** | 育児・介護休業法 | Childcare + family-care leave |
| **CN** | 劳动合同法 (Labor Contract Law) | Overtime caps; severance |
| **BR** | Consolidação das Leis do Trabalho (CLT) | 30 days férias; banco de horas |
| **AU** | Fair Work Act 2009 | National Employment Standards |
| **SG** | Employment Act | Overtime caps |
| **IN** | Factories Act 1948 + Industrial Disputes Act | Work hours; severance |

### 8.2 Privacy + data protection

| Regulation | Workplace-integration application |
|---|---|
| **GDPR (EU)** | Employee data is PII; consent for biometric/geolocation; DSAR cascade per ADR-0244 + ADR-0242; Article 17 erasure for ex-employees |
| **GDPR Article 9** | Special-category data (health, sick-leave) requires explicit consent + heightened security |
| **CCPA / CPRA (US-CA)** | California employee data rights; opt-out of sale; deletion rights |
| **KR PIPA (Personal Information Protection Act)** | Employee data consent; cross-border transfer SCC-gated; sensitive data heightened protection |
| **JP APPI (Act on the Protection of Personal Information)** | Similar to GDPR-lite |
| **HIPAA (US)** | Sick-leave + health-related expense may invoke PHI; pack-us-healthcare overlay required |
| **PIPEDA (CA)** | Canadian employee data; consent + access rights |

### 8.3 Financial + tax

| Regulation | Application |
|---|---|
| **US IRS Pub. 463 (expenses)** | Flow E expense substantiation; accountable plan requirements |
| **US SOX (Sarbanes-Oxley)** | Flow J procurement controls; Flow N doc-approval segregation-of-duties; 7y retention for financial records |
| **US SEC Rule 17a-4** | Broker-dealer record retention 3-7y if pack-us-financial |
| **EU MiFID II** | Investment-firm communications retention 5-7y |
| **KR 부가가치세법** | VAT separation on expense receipts; T-invoice |
| **KR 소득세법 / 법인세법** | Income tax + corporate tax substantiation |
| **Per-jurisdiction tax** | Payroll tax (covered by reserved `payroll` µservice integration); per-jurisdiction filing |

### 8.4 E-signature

| Regulation | Application |
|---|---|
| **US ESIGN Act 15 USC §7001** | Legal-binding of e-signatures; consumer protection on UETA opt-in |
| **EU eIDAS Regulation 910/2014** | Simple + Advanced + Qualified electronic signatures; cross-border recognition |
| **KR 전자서명법 (Electronic Signature Act, 2020 revision)** | Simple + 공인전자서명 (qualified); regulator-recognised |
| **JP 電子署名及び認証業務に関する法律** | Reliable e-signature + certification authority |
| **PAdES (ETSI EN 319 142)** | PDF Advanced Electronic Signature standard; required for eIDAS conformance |

### 8.5 Records retention

Per `oya/compliance/` per-pack overlay:

| Record class | Retention floor | Jurisdiction sources |
|---|---|---|
| Timesheet | 3-7y | US FLSA 4y; KR 3y; EU 5y; SOX 7y |
| Payroll record | 4-7y | US FLSA 4y; SOX 7y; KR 3y; EU 5y; tax 7y |
| Leave record | 3-5y | US ADA / FMLA 3y; KR 5y per labor act; EU 5y |
| E-signed contract | 7y+ | US IRS 7y; eIDAS LTV indefinite; KR 5y minimum |
| Performance review | 3-7y | US EEOC complaint window varies; KR 5y |
| Expense + receipt | 7y | US IRS 7y; KR 5y; EU VAT records 5y |
| Onboarding doc | 7y after termination | I-9 retention rules; SOX |
| Termination record | 3-7y after termination | varies; legal-hold extends indefinitely |
| Compliance training cert | per cycle + 2-3y | CA harassment 2y; KR 3y |
| Meeting recording | per pack | none default; pack-us-financial 3-7y per FINRA 4511 / SEC 17a-4 |
| Audit-chain emission | indefinite or per-jurisdiction | KR 5y; US SOX 7y; EU 5y typical |

### 8.6 Cross-border data transfer

- **EU GDPR** — Standard Contractual Clauses (SCCs) for transfer outside EU; pack-eu enforces residency.
- **KR PIPA Article 28** — cross-border transfer consent + SCC equivalent.
- **CN PIPL** — security assessment for outbound transfer.
- **Schrems II considerations** — supplementary measures.

### 8.7 Accessibility

- **WCAG 2.2 AA** — all flows.
- **ADA Title III (US)** — public-facing surfaces compliant.
- **EU Web Accessibility Directive 2016/2102** — public-sector + per-tenant.
- **KR 장애인차별금지법** — accessibility for disabled.

---

## 9. Non-functional

### 9.1 Performance (per saga; p99)

| Saga | Trigger-to-first-action | End-to-end happy path | Notes |
|---|---|---|---|
| ClockingInSaga | ≤ 500ms | ≤ 2s | mobile + voice; geolocation capture inside budget |
| LeaveRequestSaga | ≤ 1s | ≤ 24-72h (human approval) | saga overhead < 2s; rest is human time |
| ESignSaga | ≤ 1s (per signer step) | hours-days (long-running) | per-step durable; multi-day expiration |
| MeetingScheduleSaga | ≤ 1s (trigger) | ≤ 5s (slots presented) | Intelligence ranking + availability fetch |
| ExpenseSaga | ≤ 500ms (OCR) | ≤ 24h (approval) | OCR p95 ≤ 5s end-to-end |
| OnboardingSaga | weeks (long-running) | weeks | T-14d to Day-90; per-step durable |
| OffboardingSaga | minutes-hours | days | accelerated for same-day termination |
| PerformanceReviewSaga | cycle-bound | cycle-bound | quarterly/annual cycle |
| TravelRequestSaga | ≤ 2s | minutes-hours (booking) | depends on provider integration |
| ProcurementSaga | ≤ 2s | days (3-way match) | depends on vendor + invoice |
| AnnouncementSaga | ≤ 1s | seconds (fan-out) | broadcast within 30s p99 |
| ProjectTaskSaga | ≤ 500ms | continuous (project duration) | per-action; persistent |
| ComplianceTrainingSaga | ≤ 1s | per-assignment deadline | training duration is content-dependent |
| DocumentCollaborationSaga | ≤ 500ms | hours-days (review cycles) | per-action; durable |

Per ADR-0252 every saga step idempotent; replay-safe.

### 9.2 Durability + reliability

- **Workflow Engine durable execution** per ADR-0252 — every saga step persists state before advancing; per-step idempotency; cross-region replay on cell failover.
- **Saga completion rate ≥ 99.95%** at GA (per-tenant, per-flow, monthly). Failed sagas reach human-resolution path via compliance dashboard.
- **No data loss** — audit-chain emission Merkle-sealed within 1s of every state-change.
- **Per-tenant fair-share** — workflow-engine enforces per-tenant budget per ADR-0145 (no noisy-neighbour starvation).

### 9.3 Scalability

- **Concurrent active sagas per cell**: 100,000+ (per workflow-engine PRD ceiling).
- **Per-tenant cap**: configurable; default 10,000 active sagas per tenant.
- **Horizontal sharding** by tenant_id + saga_id; Citus partitioning per workflow-engine PRD.

### 9.4 Cost attribution

Per ADR-0242 + FinOps portal:

- Every saga run attributes cost to deepest tenant sub-scope.
- Cost components: workflow-engine compute + storage; intelligence inference (OCR, parsing, summary); calendar / mail / messenger / meet / drive per-µservice metering; audit-chain emission cost; cedar evaluations.
- Workplace-integration aggregate dashboard shows per-tenant per-flow cost (e.g., "Flow E expense reports cost tenant-acme $0.34 per submission on average").
- Per ADR-0174 sustainability tag — carbon-cost attribution per flow.

### 9.5 Retention (per-flow)

Per §8.5 above. Encoded in per-flow saga spec's `retention_floor` field; compliance-pack overlay applies highest floor across applicable packs.

### 9.6 SLO targets

Per ADR-0245 product-tier SLO bar (99.9% baseline; cross-cutting product layer aggregates substrate + product SLOs):

- **Per-flow saga completion**: 99.95% monthly.
- **Per-flow user-action-to-confirmation latency**: ≤ 1s p99.
- **Audit-chain emission timeliness**: ≤ 1s from state-change.
- **Cross-µservice availability composition**: workplace-integration SLO is bounded by min of (workflow-engine 99.95%, ontology 99.99%, policy-engine 99.99%, audit-chain 99.99%, per-product µservice availability).

### 9.7 DR + Business Continuity

Per ADR-0241 per-µservice `dr_tier`:

- Workplace-integration sagas: T2 (RTO ≤ 1h; some throughput degradation tolerable).
- Audit-chain emissions: T1 (RTO ≤ 5min; zero data loss).
- Critical flows (clocking, e-sign, expense) at T1 — payroll dependency.

### 9.8 Security

- Cedar gate on every action per ADR-0243.
- Tenant + sub-scope scoping per ADR-0244.
- mTLS service mesh per ADR-0148.
- Per-tenant DEK encryption for sensitive fields (signed contracts, performance reviews, sick-leave records).
- Plugin sandboxing in Wasmtime per ADR-0037.
- Bot-detection on public-facing surfaces (Calendly-style public links).

### 9.9 Observability

- Every saga emits start, step-transition, end events to `observability` substrate.
- Per-tenant dashboards in Application Shell: "Active workplace flows", "Pending approvals", "Overdue tasks", "Compliance training status".
- Per-flow funnel metrics: drop-off at each step; identify UX friction.
- Per-jurisdiction policy-fragment hit-rate (audit verification).

---

## 10. References

### 10.1 Product references

- **Workday HCM product docs (2024-Q4)** — workday.com/en-us/products/human-capital-management.html — enterprise HCM benchmark.
- **ServiceNow ITSM + Employee Service Center docs (2024)** — servicenow.com/products — workflow orchestration benchmark.
- **SAP Concur Travel + Expense (2024)** — concur.com — expense + travel benchmark.
- **Notion teamspaces + automations docs (2024-Q4)** — notion.so/help/teamspaces — modern workplace doc benchmark.
- **Slack Workflow Builder (2024)** — slack.com/help/categories/360001877268 — chat-native workflow benchmark.
- **Microsoft Power Automate docs (2024)** — learn.microsoft.com/en-us/power-automate — enterprise low-code workflow.
- **DocuSign Developer Center (2024-Q4)** — developers.docusign.com — e-signature API reference.
- **DocuSign 2024 State of Agreements report** — docusign.com/resources — market analysis.
- **Adobe Sign API (2024-Q4)** — developer.adobe.com/document-services/apis/pdf-services/electronic-signatures — alternate e-sign reference.
- **HelloSign / Dropbox Sign API (2024-Q4)** — developers.hellosign.com — alternate e-sign reference.
- **Calendly engineering blog (2023-2024)** — engineering.calendly.com — scheduling UX patterns.
- **BambooHR product features (2024-Q4)** — bamboohr.com/features — mid-market HRIS benchmark.
- **Expensify product (2024)** — use.expensify.com — expense management benchmark.
- **Brex platform docs (2024)** — brex.com/product — corp-card + expense benchmark.
- **Ramp engineering blog (2024)** — ramp.com/blog/engineering — expense + procurement benchmark.
- **Rippling integration patterns (2024)** — rippling.com/employee-management/integrations — unified-platform benchmark.
- **ChartHop product (2024)** — charthop.com/product — org-chart + comp.
- **Lattice product (2024)** — lattice.com/products/performance — performance benchmark.
- **15Five product (2024)** — 15five.com/platform — continuous performance benchmark.
- **Greenhouse onboarding (2024)** — greenhouse.io/onboarding — onboarding benchmark.
- **Lever ATS (2024)** — lever.co — ATS reference.
- **Coupa Procurement (2024)** — coupa.com — procurement benchmark.
- **SAP Ariba Procurement (2024)** — ariba.com — procurement benchmark.
- **TripActions / Navan (2024)** — navan.com — travel management benchmark.
- **Workable ATS (2024)** — workable.com — ATS reference.
- **Gusto / Justworks payroll (2024)** — gusto.com / justworks.com — payroll providers (Plugin App Store integration targets).
- **ADP RUN (2024)** — adp.com/what-we-offer/payroll/run-payroll-software.aspx — enterprise payroll benchmark.
- **Sapling HR (2024)** — saplinghr.com — modern onboarding.
- **KnowBe4 compliance training (2024)** — knowbe4.com — training delivery.
- **Workday Learning (2024)** — workday.com/en-us/products/talent-management/learning.html — learning benchmark.
- **Asana (2024)** — asana.com — project management.
- **Linear (2024)** — linear.app — engineering issue tracking.
- **Jira product (2024)** — atlassian.com/software/jira — issue tracking benchmark.
- **Monday.com (2024)** — monday.com — work management.
- **ClickUp (2024)** — clickup.com — work management.
- **Workplace by Meta (2024)** — workplace.com — internal communications benchmark.
- **Microsoft Viva (2024)** — microsoft.com/en-us/microsoft-viva — employee experience benchmark.

### 10.2 Regulatory references

- **ESIGN Act (US Public Law 106-229), 2000** — 15 USC §§7001-7031.
- **eIDAS Regulation (EU) 910/2014** — Article 25 (general legal effect); Article 26 (advanced); Articles 28-29 (qualified).
- **eIDAS 2.0 Regulation (EU) 2024/1183** — wallet-based identity + signature.
- **KR 전자서명법 (Electronic Signature Act, 2020 revision; 법률 제17354호)** — qualified signature definitions.
- **JP 電子署名及び認証業務に関する法律 (Law No. 102, 2000)** — e-signature law.
- **UK Electronic Communications Act 2000 c.7** — e-signature provisions.
- **India IT Act 2000 §3 (Digital Signature) + §3A (Electronic Signature, 2008 amendment)** — Aadhaar eSign provisions.
- **PAdES — ETSI EN 319 142** — PDF Advanced Electronic Signatures (5 parts).
- **CAdES — ETSI EN 319 122** — CMS Advanced Electronic Signatures.
- **XAdES — ETSI EN 319 132** — XML Advanced Electronic Signatures.
- **Sedona Conference Working Group 1 — "The Sedona Principles" 3rd ed.** — eDiscovery + legal hold.
- **FRCP 37(e) — Failure to Preserve Electronically Stored Information** — legal hold authority.
- **FLSA — 29 USC §§201-219** — Fair Labor Standards Act.
- **FMLA — 29 USC §§2601-2654** — Family and Medical Leave Act.
- **GDPR — Regulation (EU) 2016/679** — particularly Articles 5, 6, 9, 17, 22.
- **EU Working Time Directive 2003/88/EC**.
- **EU Work-Life Balance Directive 2019/1158**.
- **EU Whistleblowing Directive 2019/1937**.
- **KR PIPA (Personal Information Protection Act) — 개인정보 보호법** — particularly Articles 15, 17, 22, 36, 39-4.
- **KR 근로기준법 (Labor Standards Act)** — particularly Articles 50, 53, 56, 60, 73.
- **KR 남녀고용평등법 (Equal Employment Opportunity Act) Article 18 (parental leave)**.
- **HIPAA Privacy + Security Rules — 45 CFR 160, 164** — particularly Subpart C, E.
- **SOX (Sarbanes-Oxley Act) of 2002, Section 404** — internal controls over financial reporting.
- **SEC Rule 17a-4 — 17 CFR 240.17a-4** — broker-dealer record retention.
- **FINRA Rule 4511** — books and records.
- **EU MiFID II — Directive 2014/65/EU + MiFIR Regulation 600/2014** — investment firm communications.
- **IRS Pub. 463** — Travel, Gift, and Car Expenses.
- **CCPA / CPRA — California Civil Code §1798.100 et seq.**.
- **PIPEDA (Canada) — S.C. 2000, c. 5**.
- **PIPL (China) — 个人信息保护法 2021**.
- **APPI (Japan) — 個人情報の保護に関する法律**.
- **LGPD (Brazil) — Lei 13.709/2018**.
- **POPIA (South Africa) — Act 4 of 2013**.

### 10.3 Standards + technical references

- **RFC 5545 — iCalendar** (Internet Calendaring and Scheduling Core Object Specification).
- **RFC 5546 — iTIP** (iCalendar Transport-Independent Interoperability Protocol).
- **RFC 6047 — iMIP** (iCalendar Message-Based Interoperability Protocol).
- **RFC 4791 — CalDAV** (Calendaring Extensions to WebDAV).
- **RFC 9420 — MLS** (Messaging Layer Security).
- **WCAG 2.2 — W3C Web Content Accessibility Guidelines** (Oct 2023).
- **Apple App Intents (iOS 18+)** — developer.apple.com/documentation/appintents.
- **Google App Actions** — developers.google.com/assistant/app/overview.
- **Slack Block Kit** — api.slack.com/block-kit.
- **Microsoft Adaptive Cards** — adaptivecards.io.
- **Apple Wallet Passes** — developer.apple.com/documentation/walletpasses.
- **Cedar Policy Language v4.2** — docs.cedarpolicy.com.
- **OpenSLO v1** — openslo.com.
- **AsyncAPI v3** — asyncapi.com.
- **OpenAPI v3.1** — openapis.org.
- **W3C WebAuthn Level 3** — w3.org/TR/webauthn-3.

### 10.4 Internal oyatie references

- **ADR-0028 — Cloud microservice architecture + audit chain** — Merkle-sealed emission contract.
- **ADR-0099 — Data class registry** — per-field data_class annotation.
- **ADR-0105 — Thirteen-layer canonical enum** — layer rules.
- **ADR-0117 — Data residency jurisdiction code** — per-tenant overlay.
- **ADR-0128 — Hyperscaler architecture invariants** — quality bar.
- **ADR-0131 — Per-microservice flat layout** — folder shape.
- **ADR-0132 — Product-platform + bundle dissolution** — no-grouping-forward-policy.
- **ADR-0139 — Agentic SLO-gated promotion** — wave promotion.
- **ADR-0145 — Inter-microservice communication reform** — three invariants.
- **ADR-0148 — Service mesh Cilium** — mTLS.
- **ADR-0150 — Cedar policy engine**.
- **ADR-0174 — Sustainability tag** — carbon attribution.
- **ADR-0211 — In-house tech-stack preference**.
- **ADR-0240 — Sovereign cloud per regional pack**.
- **ADR-0241 — DR + business-continuity portfolio policy**.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine**.
- **ADR-0243 — Cedar as universal gate**.
- **ADR-0244 — Tenant as universal scoping primitive**.
- **ADR-0245 — Substrate vs Product layering**.
- **ADR-0246 — Policy-engine substrate promotion**.
- **ADR-0247 — Self-hosting / self-modification doctrine**.
- **ADR-0248 — Amazon-shape cellular architecture**.
- **ADR-0251 — Compliance pack cell certification levels**.
- **ADR-0252 — Workflow-engine per-step idempotency**.
- **`oya/workflow-engine/PRD.md`** — durable execution substrate.
- **`oya/workflow-studio/PRD.md`** — visual authoring product.
- **`oya/calendar/PRD.md`** — calendar substrate.
- **`oya/meet/PRD.md`** — video meeting.
- **`oya/mail/PRD.md`** — mail product.
- **`oya/messenger/PRD.md`** — chat product.
- **`oya/drive/PRD.md`** — file storage.
- **`oya/plugin-app-store/PRD.md`** — plugin marketplace.
- **`oya/intelligence/PRD.md`** — AI substrate.
- **`oya/audit-chain/PRD.md`** — Merkle-sealed audit.
- **`oya/policy-engine/PRD.md`** — Cedar substrate.
- **`oya/tenancy/PRD.md`** — tenant + sub-scope.
- **`oya/identity/PRD.md`** — OIDC + WebAuthn.
- **`oya/ontology/PRD.md`** — canonical entity types.
- **`oya/consent-graph/PRD.md`** — DSAR cascade.
- **`oya/compliance/PRD.md`** — per-pack overlay.
- **`oya/governance/PRD.md`** — fitness gates.

### 10.5 Auto-memory feedback referenced

- `feedback_oyatie_is_a_tenant_doctrine` (2026-05-20)
- `feedback_substrate_vs_product_layering` (2026-05-20)
- `feedback_workflow_studio_scope` — Workflow Studio = n8n-class first hero product
- `feedback_workflow_is_shared` — workflow-engine is shared substrate
- `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)` — cross-µservice via Workflow + Ontology pattern (historical context only; ADR-0145 supersedes; direct gRPC + 3 invariants now)
- `feedback_quality_performance_scalability_bar` — hyperscaler-grade
- `feedback_clean_architecture_requirements` — 12-layer enum + inward-only
- `feedback_autonomous_implementation_artifacts` — full templates / docs / specs
- `feedback_canonical_base_localization` — per-jurisdiction overlay
- `feedback_doc_coverage_enforced` — per-µservice + per-pack doc bundle
- `feedback_no_silent_regression` — public-contract protection
- `feedback_flat_product_catalog` — flat catalog; everything shared

---

## 11. Implementation Plans (high-level; full IPs to follow)

The workplace-integration product layer ships in IPs (Implementation Plans) under `docs/products/workplace-integration/IP-*.md`. Initial IPs:

| IP | Title | Wave |
|---|---|---|
| IP-001 | Saga spec authoring + workflow-engine registration | M04 preview |
| IP-002 | Ontology object-type introduction (TimesheetEntry, LeaveRequest, ESignDocument, ExpenseReport) | M04 preview |
| IP-003 | Cedar policy fragments + per-jurisdiction overlay packs | M04 preview |
| IP-004 | Flow A — ClockingInSaga + ClockingOutSaga + UX | M04 preview |
| IP-005 | Flow B — LeaveRequestSaga + UX + manager card | M04 preview |
| IP-006 | Flow C — ESignSaga + signature engine + PAdES integration | M04 preview |
| IP-007 | Flow D — MeetingScheduleSaga + Calendly-link sub-feature | M04 preview |
| IP-008 | Flow K — AnnouncementSaga + audience targeting | M04 preview |
| IP-009 | HR µservice promotion (out of `reserved`) | M04 preview ← dependency |
| IP-010 | Flow E — ExpenseSaga + OCR pipeline | M04 stable |
| IP-011 | Flow F — OnboardingSaga + Day-1 doc bundle | M04 stable |
| IP-012 | Flow G — OffboardingSaga + KT plan | M04 stable |
| IP-013 | Flow H — PerformanceReviewSaga + bias detection | M04 stable |
| IP-014 | Payroll µservice promotion (out of `reserved`) | M04 stable ← dependency |
| IP-015 | Flow L — ProjectTaskSaga + dependency tracking | M04 stable |
| IP-016 | Flow N — DocumentCollaborationSaga + version-history | M04 stable |
| IP-017 | Compensation µservice promotion (out of `reserved`) | M04 stable |
| IP-018 | Workflow Studio templates for all flows | M04 stable |
| IP-019 | Plugin App Store extension-point specifications | M04 stable |
| IP-020 | Application Shell embedded views | M04 stable |
| IP-021 | Mobile app workplace mini-apps | M04 stable |
| IP-022 | Voice intent handlers (Siri / Google Assistant) | M04 stable |
| IP-023 | Flow I — TravelRequestSaga + provider integrations | M05 |
| IP-024 | Flow J — ProcurementSaga + 3-way match | M05 |
| IP-025 | Flow M — ComplianceTrainingSaga + content integration | M05 |
| IP-026 | Long-tail derived flows (overtime, sick, parental, etc.) | M05 |
| IP-027 | Multi-jurisdiction overlay completeness (≥ 30 jurisdictions) | M05 |
| IP-028 | Competitor-parity feature matrix completion + benchmarking | M05 |

---

## 12. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-20 | Author workplace-integration PRD as cross-cutting product layer, not single µservice | Per ADR-0131 + ADR-0132 + ADR-0245; flat-µservice doctrine prohibits monolith; flows belong in workflow-engine specs + workflow-studio templates |
| 2026-05-20 | HR / Payroll / Compensation as currently-reserved µservices; promotion ADRs to follow | Per ADR-0245 D-6 reserved-tier rules; certification gates (SOC 2 + per-jurisdiction labor-law + tax) required before promotion |
| 2026-05-20 | First-flow set (A, B, C, D, K) for M04 preview to dogfood inside `oyatie` tenant | Per ADR-0242 Oyatie-as-tenant doctrine; flows must pass internal use before customer-tenant exposure |
| 2026-05-20 | Workflow Studio is the canonical authoring surface for tenant flow customisations | Per feedback_workflow_studio_scope; tenants edit visually; round-trip-stable spec |
| 2026-05-20 | Plugin App Store provides extension points for every flow | Per Plugin App Store PRD + ADR-0037; sandboxed Wasmtime; Cedar-gated |
| 2026-05-20 | Per-flow saga specs live in workflow-engine specs/workplace-integration/ | Centralised in workflow-engine substrate; one source of truth |
| 2026-05-20 | Per-jurisdiction Cedar overlays live in policy-engine fragments/workplace-integration/ | Per ADR-0246 policy-engine substrate; per-pack-overlay |

---

## 13. Sources scanned

- `oya/calendar/PRD.md` (workplace flow integration)
- `oya/meet/PRD.md` (meeting/video flow integration)
- `oya/workflow-engine/PRD.md` (durable saga substrate)
- `oya/workflow-studio/PRD.md` (visual authoring product)
- `oya/mail/`, `oya/messenger/`, `oya/drive/` (delivery surfaces)
- `oya/audit-chain/`, `oya/policy-engine/`, `oya/ontology/`, `oya/tenancy/`, `oya/identity/`, `oya/intelligence/` (substrate dependencies)
- `oya/plugin-app-store/PRD.md` (extension model)
- `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- `docs/decisions/ADR-0705-product-protocol-live-apex.md`
- `ADR-0242`
- `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`
- `docs/adr-archive/ADR-0252-time-coordination-distributed-consistency.md`
- `docs/products/_TEMPLATE.md` (PRD template)
- Memory ledger: `feedback_oyatie_is_a_tenant_doctrine`, `feedback_substrate_vs_product_layering`, `feedback_workflow_studio_scope`, `feedback_quality_performance_scalability_bar`, `feedback_clean_architecture_requirements`, `feedback_autonomous_implementation_artifacts`, `feedback_canonical_base_localization`, `feedback_doc_coverage_enforced`, `feedback_no_silent_regression`

---

## 14. Open questions

1. **HR/Payroll/Compensation promotion ADRs** — each requires its own ADR with full certification-gate articulation. Target authoring date: M04 preview minus 30 days.
2. **Plugin App Store provider list for M04 preview** — which initial set of Plugin App Store integrations ship Day-1? Candidates: BambooHR, Greenhouse, Gusto, ADP, Stripe (for payments), QuickBooks, NetSuite, Concur (read-only), Notarize. Open question for axis-product.
3. **Per-jurisdiction overlay completeness target** — at M04 preview, which ≤ 5 jurisdictions (KR, US-federal + CA + NY, EU-core, JP) are mandatory? At GA, target ≥ 30. Open question for axis-compliance.
4. **Anonymous whistleblower flow** — Flow N variant; EU Whistleblowing Directive 2019/1937 requires anonymity; design needed for true-anonymity vs pseudonymity tradeoff. Open question for axis-privacy + axis-security.
5. **Equity-grant integration** — currently routed via compensation µservice; 409A valuation + ASC 718 accounting; cap-table integration. Out-of-scope for M04 preview; tracked for M05.
6. **Time-zone fairness in meeting scheduling** — Intelligence-driven heuristic ("don't always schedule outside business hours for one timezone") needs explicit policy definition. Open question for axis-product.
7. **Cross-jurisdiction tax withholding** — for remote-worker leave + expense + travel; coordination with reserved `tax-engine` µservice. Tracked for M05+.
8. **Voice-trigger latency for accessibility** — Apple/Google App Intents have varying latency budgets; verify they meet WCAG 2.2 timing requirements for assistive-tech users.
9. **Plugin certification process** — for marketplace plugins extending workplace flows, what is the security review + functional testing requirement before listing? Open question for axis-security + plugin-app-store team.
10. **Localisation completeness** — at M04 preview, which languages are fully localised vs Intelligence-translated? Open question for axis-product + axis-design-system.

---

*End of Workplace Integration PRD.*

---

## Hero Surface Substance Bar Addendum - Workplace Integration

This addendum deepens Workplace Integration as a hero product surface. It does not convert the product into one microservice. It documents the cross-product product layer with named personas, jobs, stories, surfaces, data, Cedar guards, workflow nodes, ADR-0255 intelligence, ADR-0251 packs, ADR-0263 audit events, vendor migration paths, ADR-0316 tier deltas, dependencies, and recovery behavior.

## Vision

Workplace Integration exists so a person can complete real work without thinking about which microservice owns the underlying primitive. The product is for employees, managers, HR leaders, finance operators, field workers, executives, auditors, and tenant admins who need the coherence of Microsoft 365, Google Workspace, Slack, ServiceNow, Workday, and Concur while keeping oyatie's tenant, Cedar, audit-chain, workflow, ontology, and regional-pack guarantees. The timing matters because the individual workplace services now exist as independently governed products, and the missing hero value is the connective product layer that turns them into daily work.

## Personas

- Primary: Priya Krishnan, HR Director; MASTER-ROSTER row 8.
- Primary: Engineering Manager Aisha Ali; MASTER-ROSTER row 35.
- Primary: Carlos Martinez, warehouse field worker; MASTER-ROSTER row 11.
- Primary: CFO Helena Brandt; MASTER-ROSTER row 26.
- Primary: CISO Yuki Park; MASTER-ROSTER row 32.
- Secondary: Yejin Park, nurse and parent; MASTER-ROSTER row 1.
- Secondary: CEO Aoki Tanaka; MASTER-ROSTER row 25.
- Secondary: Sam Okafor, internal audit director; MASTER-ROSTER row 9.
- Secondary: Board director Patrick O'Reilly; MASTER-ROSTER row 34.
- Secondary: Jordan Lee, kiosk user and minor worker; MASTER-ROSTER row 16.

## Jobs-to-be-Done

### Job-to-be-done-WI-01 - Request leave without context switching
- Situation: An employee asks for leave from mobile.
- Acceptance: LeaveRequestSaga touches HR balance, Calendar OOO, Messenger approval, Mail notice, AuditChain seal, and policy-engine decision.
- Acceptance: denial explains the policy and cites the pack rule.

### Job-to-be-done-WI-02 - Approve spend inside daily communication
- Situation: A manager receives an expense card in Messenger.
- Acceptance: ExpenseSaga shows amount, receipt, budget, policy, and approver scope.
- Acceptance: approval creates audit evidence and never bypasses finance review threshold.

### Job-to-be-done-WI-03 - Onboard a new employee with IT, HR, payroll, calendar, and drive ready
- Situation: Priya starts a new hire.
- Acceptance: OnboardingSaga creates account, group, drive folder, payroll task, security training, equipment request, and manager checklist.
- Acceptance: each node has idempotency and owner fallback.

### Job-to-be-done-WI-04 - Schedule meetings fairly across time zones
- Situation: Aisha schedules a team meeting spanning KR, US, and EU.
- Acceptance: MeetingScheduleSaga proposes slots, explains fairness, and respects local work-hour pack overlays.
- Acceptance: repeated unfair time-zone burden triggers a fairness alert.

### Job-to-be-done-WI-05 - Capture field work on rugged or kiosk devices
- Situation: Carlos submits safety incident and time clock from a rugged device.
- Acceptance: offline queue, device identity, photo evidence, and kiosk timeout are enforced.
- Acceptance: sync conflict opens a supervisor review task.

### Job-to-be-done-WI-06 - Make workplace flows audit-ready
- Situation: Sam audits access review, leave approval, expense reimbursement, and e-sign.
- Acceptance: every flow exports workflow run, Cedar decision, signer, event id, data class, and pack redaction evidence.
- Acceptance: export completeness is measurable without screenshots.

### Job-to-be-done-WI-07 - Extend a workflow with a plugin safely
- Situation: A tenant installs a travel provider plugin.
- Acceptance: PluginAppStore extension runs in signed, scoped, Cedar-gated mode.
- Acceptance: extension cannot read or mutate outside declared flow resources.

### Job-to-be-done-WI-08 - Explain work state with intelligence
- Situation: A user asks why a workflow is blocked.
- Acceptance: intelligence retrieves tenant-private workflow, policy, and document context with citations.
- Acceptance: explanation cannot approve, deny, or mutate.

## User Stories

### Story WI-HS-001 - Leave Request Card
As an employee, I want a leave card in Messenger so that I can request time off without opening HR.
Pass: card shows balance, dates, policy, coverage conflict, and submit action.
Pass: submission emits EVT-WI-LEAVE-REQUESTED.

### Story WI-HS-002 - Manager Leave Approval
As a manager, I want a single approve or deny card so that I can act without losing policy context.
Pass: card includes team coverage, local labor rule, and conflict summary.
Pass: deny requires reason and emits policy-cited event.

### Story WI-HS-003 - Calendar OOO Propagation
As an employee, I want approved leave to create out-of-office calendar state so that coworkers see availability.
Pass: Calendar event links leave_request_id and retention class.
Pass: cancellation removes OOO with evidence.

### Story WI-HS-004 - Expense Receipt OCR
As an employee, I want receipt capture to parse vendor, tax, date, and amount so that expense entry is fast.
Pass: OCR suggestion remains editable and cited.
Pass: high-risk amount routes to finance.

### Story WI-HS-005 - Expense Reimbursement
As finance operator, I want approved expenses to route to payroll or payments so that reimbursement is timely.
Pass: reimbursement command includes policy_decision_id and budget_ref.
Pass: payment release cannot occur from workplace layer directly.

### Story WI-HS-006 - E-Sign Packet
As HR admin, I want offer and NDA e-sign in one packet so that new hires finish required documents before start date.
Pass: every signature has signer identity, timestamp, document hash, and consent event.
Pass: unsigned required doc blocks onboarding completion.

### Story WI-HS-007 - Meeting Poll
As Aisha, I want schedule proposals across attendees so that the meeting lands on the fairest slot.
Pass: proposal includes time-zone burden score.
Pass: manual override emits reason.

### Story WI-HS-008 - Onboarding Checklist
As Priya, I want one onboarding checklist so that HR, IT, payroll, and manager tasks are visible.
Pass: tasks show owner, due date, dependency, and evidence status.
Pass: overdue critical task alerts manager and HR.

### Story WI-HS-009 - Offboarding Access Review
As CISO Yuki, I want offboarding to revoke access and export evidence so that leavers do not retain permissions.
Pass: revocation covers identity, groups, drive, mail delegate, app tokens, and plugins.
Pass: failure keeps offboarding red until resolved.

### Story WI-HS-010 - Performance Review Packet
As manager, I want review forms, goals, feedback, and calibration in one packet so that performance cycles are consistent.
Pass: packet supports employee comments and HR lock.
Pass: calibration changes emit audit event.

### Story WI-HS-011 - Project Task From Chat
As project lead, I want to create task from Messenger message so that commitments are captured.
Pass: task links source message, assignee, due date, project, and visibility.
Pass: private chat content is redacted in public project views.

### Story WI-HS-012 - Announcement Targeting
As comms lead, I want targeted announcements by tenant, region, role, and language so that people receive relevant notices.
Pass: audience preview shows count and pack restrictions.
Pass: urgent announcement requires second approver.

### Story WI-HS-013 - Compliance Training
As compliance officer, I want required training assigned by role and jurisdiction so that obligations are tracked.
Pass: training assignment lists due date, pack basis, and escalation.
Pass: completion emits evidence event.

### Story WI-HS-014 - Procurement Request
As employee, I want purchase request from chat or mobile so that equipment requests do not require ERP knowledge.
Pass: request captures item, reason, cost center, vendor, and approval path.
Pass: high-risk vendor routes to ERP supplier onboarding.

### Story WI-HS-015 - Travel Request
As employee, I want travel request to check budget, policy, visa, and risk so that approval is complete.
Pass: request includes itinerary, cost estimate, risk flags, and manager approval.
Pass: destination risk change reopens approval.

### Story WI-HS-016 - Document Collaboration Sign-Off
As document owner, I want review and sign-off state in Drive and Messenger so that approval is visible.
Pass: sign-off includes document hash and reviewer role.
Pass: updated document invalidates stale approval.

### Story WI-HS-017 - Kiosk Clock In
As kiosk worker Jordan, I want fast clock-in with privacy-safe session timeout so that shared devices are safe.
Pass: kiosk clears user state after configured idle timeout.
Pass: minor-worker rules apply by persona context.

### Story WI-HS-018 - Field Safety Incident
As Carlos, I want rugged-device safety incident capture so that field incidents are reported immediately.
Pass: incident supports offline photo, location, supervisor, and severity.
Pass: sync emits queued evidence with original device timestamp.

### Story WI-HS-019 - Audit Export
As Sam, I want a flow evidence export so that audit testing is not screenshot-based.
Pass: export includes workflow run ids, policy decisions, audit events, signer list, and redaction reason.
Pass: missing evidence marks export incomplete.

### Story WI-HS-020 - Plugin Extension Guard
As tenant admin, I want to install a plugin extension to expense flow so that our travel provider can prefill data.
Pass: plugin declares resources, actions, data classes, and network egress.
Pass: Cedar denies undeclared resource access.

## Surface Map

### Surface WI-SURF-01 - Work Hub
```
+ Today + Approvals + Tasks + Meetings + Docs + Alerts +
| Leave pending | Expense needs receipt | Meeting fairness warning |
+--------------------------------------------------------+
```

### Surface WI-SURF-02 - Manager Approval Queue
```
+ Item + Policy + Risk + SLA + Action +
| Leave B-88 | KR labor ok | low | 4h | Approve / Deny |
+--------------------------------------------------------+
```

### Surface WI-SURF-03 - Mobile Mini-App
```
+ Flow + Quick action + Evidence +
| Clock in | Safety incident | Receipt scan | Leave request |
+--------------------------------------------------------+
```

### Surface WI-SURF-04 - Workflow Studio Template Browser
```
+ Flow + Pack + Tier + Editable + Last version +
+--------------------------------------------------------+
```

### Surface WI-SURF-05 - Audit Evidence Viewer
```
+ Flow + Run + Cedar + Event + Export +
| ExpenseSaga | RUN-88 | DEC-31 | EVT-WI-EXPENSE-APPROVED | ready |
+--------------------------------------------------------+
```

### Surface WI-SURF-06 - Plugin Extension Panel
```
+ Plugin + Flow + Actions + Data class + Certification +
| TravelMate | TravelRequestSaga | read itinerary | confidential | signed |
+--------------------------------------------------------+
```

## Data Model

### Entity WI-ENT-01 - WorkplaceFlow
- Fields: flow_id, name, saga_id, owner_microservices, tier_minimum, pack_overlays, status.
- Relationships: owns WorkflowTemplate, SurfaceBinding, PolicyFragment, AuditEventContract.
- Invariant: flow status cannot be GA until audit completeness is 100%.

### Entity WI-ENT-02 - WorkplaceRun
- Fields: run_id, flow_id, tenant_id, actor_id, state, current_node, started_at, completed_at.
- Relationships: references workflow-engine run and audit-chain events.
- Invariant: every state transition has a policy decision or system reason.

### Entity WI-ENT-03 - ApprovalCard
- Fields: card_id, flow_id, approver_id, subject_ref, policy_summary, actions, expiry.
- Relationships: delivered through Messenger, Mail, Mobile, and Application Shell.
- Invariant: action token expires and is single-use.

### Entity WI-ENT-04 - FlowDocument
- Fields: document_id, flow_id, drive_ref, hash, retention_policy, signer_refs, data_class.
- Relationships: belongs to e-sign, onboarding, review, or audit flows.
- Invariant: signed hash must match active document version.

### Entity WI-ENT-05 - PluginExtensionBinding
- Fields: binding_id, plugin_id, flow_id, declared_actions, declared_resources, certification_state.
- Relationships: belongs to Plugin App Store listing and Cedar policy bundle.
- Invariant: unsigned plugin cannot bind to regulated flow.

### Entity WI-ENT-06 - FlowEvidenceBundle
- Fields: bundle_id, flow_id, run_id, audit_event_ids, policy_decision_ids, redaction_pack, export_state.
- Relationships: exported to auditors and regulators.
- Invariant: export_state ready requires no missing required event.

## Cedar Policy Model

- Principal workplace::Employee can start self-service flows for own tenant membership.
- Principal workplace::Manager can approve subordinate flows only within reporting scope.
- Principal workplace::HRAdmin can start onboarding, offboarding, leave-balance correction, and e-sign packets.
- Principal workplace::FinanceOperator can review expense and procurement finance nodes.
- Principal workplace::Auditor can read FlowEvidenceBundle but cannot mutate WorkplaceRun.
- Principal workplace::TenantAdmin can install PluginExtensionBinding only after certification green.
- Action workplace::approve requires approver not equal to requester unless explicit self-approval pack rule permits.
- Action workplace::install_plugin requires signed artifact, declared egress, and allowed data class.
- Action workplace::export_evidence requires audit scope and redaction pack.
- Resource workplace::WorkplaceRun includes tenant_id, flow_id, actor_id, data_class, pack_set.
- Resource workplace::ApprovalCard includes approver_scope, expiry, and action_token_hash.

## Workflow Engine Integration

- Node WI-WF-01 TriggerNormalize accepts chat, mail, mobile, shell, API, or plugin trigger.
- Node WI-WF-02 PersonaResolve loads identity, tenant, role, device, locale, and audience type.
- Node WI-WF-03 PolicyPrecheck runs Cedar before any side effect.
- Node WI-WF-04 SurfaceRender emits ApprovalCard or mobile form.
- Node WI-WF-05 DomainRead reads HR, calendar, drive, finance, or task state.
- Node WI-WF-06 DecisionRoute chooses approve, deny, request-info, or escalate.
- Node WI-WF-07 DomainWrite writes through owning microservice only.
- Node WI-WF-08 AuditSeal emits ADR-0263 event.
- Node WI-WF-09 NotifyParticipants sends Messenger, Mail, Calendar, or mobile update.
- Node WI-WF-10 Compensation handles downstream rollback or reversal.
- Node WI-WF-11 EvidenceBundleBuild prepares export package.
- Node WI-WF-12 PluginHookInvoke calls signed plugin extension after Cedar permit.
- Branch B1: policy denial routes to explanation, not mutation.
- Branch B2: offline mobile queues with timestamp and replay guard.
- Branch B3: plugin failure isolates plugin and keeps core flow alive where safe.

## AI / Intelligence Integration

- ADR-0220 layer: classify receipts, policy questions, and workflow blockers.
- ADR-0255 layer 1: tenant-private retrieval cites workflow run, document hash, policy decision, and user-visible state.
- ADR-0255 layer 2: aggregate flow friction learns common bottlenecks without tenant data leakage.
- Capability workplace.leave.explain-denial summarizes denied leave with cited policy.
- Capability workplace.expense.ocr parses receipt but cannot submit without user confirmation.
- Capability workplace.meeting.fair-slot ranks candidate times and explains fairness.
- Capability workplace.onboarding.next-risk detects delayed onboarding tasks.
- Capability workplace.audit.export-summary summarizes evidence completeness.
- Prohibited: intelligence cannot approve, deny, sign, install plugin, revoke access, release payment, or alter evidence.

## Pack Overlays

- KR labor pack activates working-hour, overtime, leave, personal information, and kiosk guard rules.
- EU working-time pack activates time-zone fairness, retention minimization, and worker-representation evidence.
- US FLSA pack activates exempt/non-exempt overtime and state leave overlays.
- JP labor pack activates work-style reform constraints and local document retention.
- Healthcare pack activates HIPAA redaction and clinical persona access constraints.
- Finance pack activates SOX expense, access-review, and evidence export controls.
- Public-sector pack activates conflict-of-interest and procurement transparency gates.

## SLO Targets

- Approval card render p95 <= 500 ms.
- LeaveRequestSaga completion p95 <= 30 s excluding human wait.
- Expense OCR suggestion p95 <= 3 s for one receipt.
- Meeting slot generation p95 <= 2 s for 50 attendees.
- Onboarding checklist create p95 <= 5 s.
- Offboarding critical revocation command dispatch p95 <= 60 s.
- Audit evidence export p95 <= 2 min for one flow period.
- Mobile offline replay conflict detection p95 <= 5 s after reconnect.
- Plugin hook timeout default <= 2 s with isolated failure.
- Audit event emission completeness = 100% for regulated flows.

## Telemetry

- EVT-WI-FLOW-STARTED emits flow_id, actor_id, tenant_id, channel, and pack_set.
- EVT-WI-PERSONA-RESOLVED emits run_id, audience_type, device_profile, and locale.
- EVT-WI-POLICY-PRECHECK-DENIED emits policy_decision_id, flow_id, action, and reason_code.
- EVT-WI-APPROVAL-CARD-RENDERED emits card_id, flow_id, approver_id, and expiry.
- EVT-WI-APPROVAL-SUBMITTED emits card_id, action, approver_id, and policy_decision_id.
- EVT-WI-LEAVE-REQUESTED emits leave_request_id, dates, balance_snapshot, and run_id.
- EVT-WI-EXPENSE-SCANNED emits expense_id, ocr_confidence, amount, and currency.
- EVT-WI-MEETING-FAIRNESS-SCORED emits meeting_id, fairness_score, and candidate_count.
- EVT-WI-ONBOARDING-TASK-CREATED emits task_id, owner, dependency, and due_at.
- EVT-WI-OFFBOARDING-REVOCATION-DISPATCHED emits identity_id, system_ref, and status.
- EVT-WI-PLUGIN-HOOK-INVOKED emits plugin_id, flow_id, action, and data_class.
- EVT-WI-PLUGIN-HOOK-DENIED emits plugin_id, action, resource, and Cedar reason.
- EVT-WI-EVIDENCE-BUNDLE-GENERATED emits bundle_id, flow_id, event_count, and redaction_pack.

## Migration Playbook Index

- Microsoft 365: Outlook, Teams, SharePoint, OneDrive, Planner, Power Automate migration.
- Google Workspace: Gmail, Chat, Meet, Calendar, Drive, Docs, Apps Script migration.
- Slack: channel workflows, huddles, canvas, app shortcuts, workflow builder migration.
- Notion: databases, teamspaces, projects, docs, automations migration.
- ServiceNow: employee workflow, approval, ITSM, HR service delivery migration.
- Workday: leave, onboarding, performance, HR task, and approval migration.
- Concur: expense, travel, receipt, reimbursement migration.
- DocuSign and Adobe Sign: envelope, signer, document hash, consent migration.
- BambooHR and Rippling: HR profile, onboarding, offboarding, payroll handoff migration.
- Greenhouse and Lever: candidate, offer, onboarding packet migration.

## Capability Tier Deltas


## Competitive Positioning

- Microsoft 365: oyatie wins on Cedar policy, audit-chain evidence, sovereign packs, and cross-service workflow ownership.
- Google Workspace: oyatie wins on tenant-scoped automation and regulated evidence.
- Slack: oyatie wins by connecting chat workflow to HR, finance, calendar, audit, and ontology primitives.
- ServiceNow: oyatie wins by making employee workflows native to daily workplace channels.
- Workday: oyatie wins by linking HR flows to mail, messenger, docs, drive, identity, and audit.
- Concur: oyatie wins by connecting expense to workflow, policy, finance, and payments under one tenant.
- DocuSign: oyatie wins by binding signature state to workflows, drive documents, and audit chain.
- Notion: oyatie wins by replacing flexible docs-only workflow with typed workflow and policy enforcement.

## Roadmap

- Wave M04-preview: LeaveRequestSaga, ClockingSaga, ESignSaga, MeetingScheduleSaga, AnnouncementSaga.
- Wave M04-stable: ExpenseSaga, OnboardingSaga, OffboardingSaga, PerformanceReviewSaga, ProjectTaskSaga, DocumentCollaborationSaga.
- Wave M05: TravelRequestSaga, ProcurementSaga, ComplianceTrainingSaga, long-tail derived flows.
- Wave M06: regulator export, sovereign-cell posture, enterprise plugin certification.
- Milestone 1: internal oyatie tenant dogfood.
- Milestone 2: design partner tenants.
- Milestone 3: GA by pack and capability tier.

## Cross-Product Dependencies

- workflow-engine owns durable saga execution, retries, idempotency, and compensation.
- workflow-studio owns visual template authoring and tenant edits.
- calendar owns event state, availability, OOO, meeting rooms, and time zones.
- meet owns meeting links, recordings, and transcript handoff.
- mail owns notification templates and email delivery.
- messenger owns cards, approvals, reminders, and chat-origin tasks.
- drive owns documents, hashes, retention, and folders.
- intelligence owns OCR, explanations, summarization, and fairness suggestions.
- policy-engine owns Cedar decisions and pack overlays.
- audit-chain owns immutable evidence.
- tenancy owns tenant membership, region, and pack set.
- identity owns passkey, role, group, and revocation.
- ontology owns LeaveRequest, ExpenseReport, Signature, MeetingProposal, and EmploymentRecord types.
- plugin-app-store owns extension certification and signed artifact trust.

## Failure Modes + Recovery

- Failure: workflow-engine run stuck. Recovery: retry idempotent node, show owner, and emit stuck-run event.
- Failure: Messenger card action token expired. Recovery: regenerate card after policy recheck.
- Failure: Calendar write succeeds but HR write fails. Recovery: compensation removes OOO or marks exception.
- Failure: plugin hook times out. Recovery: isolate plugin, continue core flow when non-critical, and notify tenant admin.
- Failure: mobile offline replay conflict. Recovery: supervisor review task compares original timestamp and current state.
- Failure: e-sign document hash mismatch. Recovery: invalidate signature packet and require re-sign.
- Failure: offboarding revocation fails. Recovery: keep critical incident open and retry by system connector.
- Failure: expense OCR low confidence. Recovery: require user confirmation before submit.
- Failure: policy pack conflict. Recovery: choose stricter rule and open compliance review.
- Failure: audit export missing event. Recovery: reseal from audit-chain or mark incomplete.
- Failure: time-zone fairness regression. Recovery: disable auto-pick and require manual scheduling reason.
- Failure: tenant plugin tries undeclared data. Recovery: Cedar denial, plugin suspension candidate, and audit event.

## AI substrate + Cellular automation

This product consumes current SSOT doctrine for the intelligence substrate, cellular automation, and cloud-native delivery:

- D-CICD-AUTHORITY binds this lane to the branch-protected `oya-ci-required` cloud-ci/oya-ci gate as live merge authority; local command output is transition evidence only. Historical ADR-0346 verifier wording is retained only where it does not conflict with `registry/stores/design-store.json` current truth.
- D-GOVERNANCE-CENTRAL: central PaC/CaC/PDP/evidence pipelines own governance authority; do not scatter authority across local CLI lanes.
- ADR-0348 binds workplace tenant placement, workflow execution locality, and plugin blast-radius control to cellular topology. Enforcement evidence flows through central governance and the branch-protected `oya-ci-required` gate, not scattered local lanes.
- D-CICD-AUTHORITY keeps one canonical CI authority now (`oya-ci-required`) and the owned oya-ci cutover later; self-hostable delivery references are subordinate to the current SSOT and are not parallel merge authorities. Historical ADR-0349 substrate wording is retained only as non-authoritative context until reconciled with the current stores.

## References

- docs/standards/documentation-rigor.md
- docs/personas/MASTER-ROSTER-2026-05-21.md
- docs/decisions/ADR-0242
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- docs/decisions/ADR-0708-platform-foundations-live-apex.md
- docs/adr-archive/ADR-0252-time-coordination-distributed-consistency.md
- docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md
- docs/adr-archive/ADR-0263-observability-emission-contract.md
- docs/adr-archive/ADR-0316-capability-tier-over-product-fragmentation.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- `registry/stores/instructions-store.json` D-CICD-AUTHORITY / D-CLOUD-NATIVE current CI authority
- specs/products/workplace-integration.json
- specs/oya/workflow-engine.json
- specs/oya/workflow-studio.json
- specs/oya/intelligence.json

## 2a. Acceptance criteria traceability (required)

This section is a planning-maturity contract only. It does **not** claim runtime, product-ready, or hyperscaler-ready status; promotion still requires fresh CI, SLO, security, SBOM, rollback/DR, owner/RACI, and product-pain evidence.

| AC-ID | Given | When | Then | Test ID | Test path |
|---|---|---|---|---|---|
| WORKPLACE-PRD-AC-001 | The Workplace Integration PRD is used as a planning contract and cross-service HR, payroll, calendar, messenger, and workflow saga readiness is evaluated | The planned-maturity gate scans product PRDs | workplace saga acceptance is linked to test and evidence paths instead of generic prose | WORKPLACE-PRD-GATE-001 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |
| WORKPLACE-PRD-AC-002 | a workplace-flow promotion packet references this PRD | Readiness evidence is evaluated | fresh saga, HR/payroll/calendar/messenger integration, audit, and user-pain evidence is required outside this PRD | WORKPLACE-PRD-GATE-002 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |

## 9b. Verification commands (required) — one runnable check per metric

| Metric | Verification command | Pass criterion | CI lane |
|---|---|---|---|
| Workplace saga/workflow integration planning maturity | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | At least one Workplace row names saga, workflow, HR/payroll, calendar/messenger, and audit obligations | `oya-ci-required` |
| Workplace product-ready non-claim boundary | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | A workplace promotion packet cannot treat this PRD as product-ready evidence without fresh CI and product-pain proof | `oya-ci-required` |
