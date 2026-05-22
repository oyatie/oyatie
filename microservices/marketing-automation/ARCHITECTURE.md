---
doc_class: Architecture
microservice: marketing-automation
status: wave-15a-big8-remediation
date: 2026-05-21
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0248
  - ADR-0251
  - ADR-0253-amendment
  - ADR-0263
  - ADR-0297
  - ADR-0314
  - ADR-0321
  - ADR-0328
  - ADR-0331
retired_adr_bindings:
  - ADR-0315
  - ADR-0316
companion_docs:
  - microservices/marketing-automation/PRD.md
  - microservices/marketing-automation/compliance.md
  - microservices/marketing-automation/manifest.json
  - microservices/marketing-automation/coherence-audit-2026-05-20.md
  - microservices/marketing-automation/REMEDIATION-NOTES-2026-05-21.md
---

# Architecture: Marketing Automation

## A. Boundary

Marketing Automation owns 25 bounded contexts that compose the customer-engagement spine across HubSpot Marketing Hub, Adobe Marketo Engage, and Mailchimp coverage. It owns engagement-side state (campaign envelope, journey orchestration, lifecycle progression, suppression, attribution, deliverability, frequency capping, marketing assets, lead scoring, ABM, A/B testing, send-time optimization, customer-facing analytics, behavioral profiling, webhook subscriptions, marketing email composition, landing page, form, workflow visual canvas) plus seam declarations to social, sites, contact-center, messenger, calendar, and crm.

The microservice does not own tenant identity (iam), Cedar policy engine internals (policy-engine), workflow runtime internals (workflow-engine), ontology storage (ontology), payments rails (commerce + cloud-billing), marketplace settlement state (marketplace), sales-record state (crm.contact-master + crm.account-master + crm.opportunity + crm.lead), email delivery execution (mail), or messaging delivery execution (messenger). It also does not recreate the Salesforce Marketing Cloud or HubSpot Hub-suite boundaries — product labels remain UX/marketing concerns, not microservice boundaries.

## B. Layer Map

| ADR-0105 layer | Planned responsibility |
|---|---|
| api | Public command/query DTOs and OpenAPI 3.2.0 contract binding |
| rest | HTTP/3-first transport, idempotency enforcement, request validation |
| application | Usecase orchestration and transaction boundaries |
| usecase | Command handlers, read models, migration dry-runs, replay flows |
| domain | Aggregate invariants and state transitions |
| kernel | Pure value objects, policy-port traits, deterministic calculations |
| adapter | Source-system, storage, queue, evidence adapters |
| worker | Async import, replay, reconciliation, notification, materialization workers |
| governance | Policy, compliance, scorecards, evidence gates |
| cli | Operator and CI-driven CLI surface |
| sdk | Generated client SDK manifest (Rust + TypeScript + Swift + Kotlin) |
| test | Per-IP integration test fixtures |

## C. Bounded Context Architecture

### segment (dynamic membership)

Aggregate root `marketing_segment`. Invariants: tenant scope required; predicate tree validates against ontology trait/event registry before persistence; freshness floor declared per segment (default 750 ms); event-cursor monotonic; member-count delta auditable. Commands: define, amend, materialize, freeze, archive, replay. Events: defined, amended, materialized, delta-applied, frozen, archived. Counterpart map: HubSpot Active List + Marketo Smart List + Mailchimp Segment.

### static-list

Aggregate root `marketing_static_list`. Invariants: tenant scope required; membership rows have explicit source provenance; admin-only mutation paths. Commands: create, add-member, remove-member, archive, import. Events: created, member-added, member-removed, archived, import-completed. Counterpart map: HubSpot Static List + Marketo Static List + Mailchimp Tag.

### campaign

Aggregate root `marketing_campaign`. Invariants: engagement-side campaign envelope only — revenue lineage delegated to crm.opportunity; goal + channel + asset references; budget metadata; UTM parameter scheme. Commands: create, amend, attach-asset, detach-asset, archive, publish, retire. Events: created, amended, asset-attached, asset-detached, archived, published, retired. Counterpart map: HubSpot Marketing Campaign + Marketo Program + Mailchimp Campaign.

### journey (runtime)

Aggregate root `marketing_journey_run`. Invariants: tenant scope; runtime executes a workflow-canvas snapshot; idempotent step replay; HLC-stamped per step. Commands: trigger, advance-step, defer, complete, cancel, replay. Events: triggered, step-advanced, deferred, completed, cancelled, replayed. Counterpart map: HubSpot Workflow runtime + Marketo Smart Campaign runtime + Mailchimp Customer Journey runtime.

### workflow-canvas (authoring)

Aggregate root `marketing_workflow_canvas`. Invariants: tenant scope; graph DAG with no cycles (DAG-validated); step types from canonical step-registry; version monotonic; published versions immutable; only one published version at a time per canvas name. Commands: create, amend, validate, publish, retire, duplicate. Events: created, amended, validated, published, retired, duplicated. Counterpart map: HubSpot Workflow Editor + Marketo Smart Campaign Designer + Mailchimp Customer Journey Builder.

### email (composition)

Aggregate root `marketing_email`. Invariants: tenant scope; subject + content + dynamic tokens + smart content rules + A/B variant set; preview-text, from-name, from-email, replyTo immutable for sent envelopes; accessibility-checker pass before publish. Commands: create, amend, validate, publish, A/B-variant-add, schedule-send, cancel-send. Events: created, amended, validated, published, variant-added, send-scheduled, send-cancelled. Counterpart map: HubSpot Marketing Email + Marketo Email + Mailchimp Regular Campaign / Automated Email / Plain-text / RSS-driven / Postcard.

### landing-page

Aggregate root `marketing_landing_page`. Invariants: tenant scope; template + form attachment + conversion goal + A/B variant + SEO metadata + custom CSS/HTML override + password protection + language localisation. Commands: create, amend, publish, retire, A/B-variant-add. Events: created, amended, published, retired, variant-added. Counterpart map: HubSpot Landing Page + Marketo Landing Page + Mailchimp Landing Page.

### form

Aggregate root `marketing_form`. Invariants: tenant scope; field configuration + validation rules + post-submit redirect + GDPR consent block + progressive profiling + conditional logic; submission idempotency; CAPTCHA invariant per pack overlay. Commands: create, amend, publish, retire, submit. Events: created, amended, published, retired, submission-received. Counterpart map: HubSpot Form (regular + pop-up + collected + chatflow-form) + Marketo Form + Mailchimp Signup Form.

### lead-scoring

Aggregate root `marketing_lead_score`. Invariants: per-subject score with formula version + decay-half-life + manual adjustments + behavioral trigger rules. Commands: define-model, score-subject, decay, manual-adjust, recalculate. Events: model-defined, subject-scored, decayed, manually-adjusted, recalculated. Counterpart map: HubSpot Lead Scoring + Marketo Lead Scoring + Mailchimp Premium Predicted Demographics.

### lifecycle-stage

Aggregate root `marketing_lifecycle_progression`. Invariants: monotonic progression Subscriber → Lead → MQL → SQL → Opportunity → Customer → Evangelist; transitions are auditable events; downgrade paths require principal authorisation. Commands: progress, downgrade, freeze, resume. Events: progressed, downgraded, frozen, resumed. Counterpart map: HubSpot Lifecycle Stage + Marketo Engagement Score Buckets + Mailchimp CLV bands.

### subscription-type

Aggregate root `marketing_subscription_type`. Invariants: per-tenant publication-channel category; per-pack disclosure requirement; opt-in evidence chain; unsubscribe link mandatory per CAN-SPAM/CASL. Commands: create, amend, activate, deactivate, subject-subscribe, subject-unsubscribe. Events: created, amended, activated, deactivated, subject-subscribed, subject-unsubscribed. Counterpart map: HubSpot Subscription Type + Marketo Communication Limits + Mailchimp Groups.

### consent-audience

Aggregate root `marketing_consent_audience`. Invariants: append-only HLC-stamped ledger; per-subject × per-channel × per-purpose row; immutable once written; supports right-of-erasure via tombstone projection. Commands: append-consent, append-revocation, check-suppression, generate-dsr-report. Events: consent-appended, revocation-appended, suppression-applied, dsr-report-generated. Counterpart map: HubSpot Subscription Preferences + Marketo Unsubscribe Tracking + Mailchimp Unsubscribed.

### attribution

Aggregate root `marketing_attribution_run`. Invariants: tenant scope; model parameters versioned; touches loaded from event-store; revenue events sourced from crm with cryptographic seal; credit allocation deterministic given (touches, revenue-events, model-version). Commands: configure-model, reconcile, freeze, replay. Events: model-configured, reconciliation-completed, frozen, replayed. Counterpart map: HubSpot Campaign Attribution + Marketo Revenue Cycle Modeler + Mailchimp Premium Revenue Report.

### deliverability

Aggregate root `marketing_deliverability_warmup`. Invariants: per-domain warmup state machine {warming, healthy, paused, blocked}; DMARC-failure fail-closed; tenant admin override requires Cedar step-up + audit. Commands: configure-warmup, admit-send, pause, resume, block. Events: warmup-configured, send-admitted, paused, resumed, blocked. Counterpart map: HubSpot Email Health + Marketo Deliverability Program + Mailchimp Premium Delivery Optimization.

### frequency-cap

Aggregate root `marketing_frequency_window`. Invariants: per-subject × per-purpose × per-channel window; max-touches enforced atomically (CAS); legal-notice purpose bypass via Cedar; counters survive Valkey miss via Postgres source-of-truth. Commands: reserve-touch, release-touch, expire-window, override. Events: touch-reserved, touch-denied, window-expired, override-applied. Counterpart map: HubSpot Frequency Safeguard + Marketo Communication Limit + Mailchimp Premium Contact Rating.

### abm (account-based marketing)

Aggregate root `marketing_abm_target_account`. Invariants: tied to crm.account-master via account_id; account-score formula + decay; account-level workflow trigger references; intent-data ingest source provenance. Commands: target-account, score-account, attach-workflow, detach-workflow. Events: account-targeted, account-scored, workflow-attached, workflow-detached. Counterpart map: HubSpot ABM + Marketo Account-Based Marketing + Demandbase/Terminus intent.

### a-b-test

Aggregate root `marketing_a_b_test`. Invariants: variant set with allocation percentages summing to 100; statistical significance threshold declared; winner-selection rule declared; test-stop semantics auditable. Commands: configure-test, allocate-traffic, conclude, declare-winner. Events: test-configured, traffic-allocated, concluded, winner-declared. Counterpart map: HubSpot A/B Test + Marketo A/B Test + Mailchimp A/B Test.

### send-time-optimization

Aggregate root `marketing_sto_profile`. Invariants: per-subject optimal-send-time prediction sourced from intelligence µservice; fallback default window; respects frequency-cap and deliverability admit decisions. Commands: predict-window, override-window, reset-profile. Events: window-predicted, window-overridden, profile-reset. Counterpart map: HubSpot Send Time Optimization + Marketo Optimal Send Time + Mailchimp Send Time Optimization.

### email-tracking

Aggregate root `marketing_email_tracking_event`. Invariants: open + click + reply + bounce telemetry with link-parameter scheme; subject-hash binding; respect GPC/DNT signals where applicable. Commands: record-open, record-click, record-reply, record-bounce. Events: open-recorded, click-recorded, reply-recorded, bounce-recorded. Counterpart map: HubSpot Email Tracking + Marketo Email Insights + Mailchimp Click/Open Reports.

### webhook-subscription

Aggregate root `marketing_webhook_subscription`. Invariants: subscriber URL + event-filter + signing secret (in OpenBao); retry policy with exponential backoff; signed-payload contract HMAC-SHA-256; replay-attack defence via signed timestamp. Commands: subscribe, amend, unsubscribe, deliver, rotate-secret. Events: subscribed, amended, unsubscribed, delivery-attempted, delivery-succeeded, delivery-failed, secret-rotated. Counterpart map: HubSpot Webhooks + Marketo Webhooks + Mailchimp Webhooks.

### marketing-calendar

Aggregate root `marketing_calendar_entry`. Invariants: temporal view across email + landing-page + workflow + social + ad assets; tenant scope; conflict detection per channel × audience overlap. Commands: schedule, reschedule, cancel, attach-asset. Events: scheduled, rescheduled, cancelled, asset-attached. Counterpart map: HubSpot Marketing Calendar + Marketo Calendar + Mailchimp Content Calendar.

### behavioral-profile

Aggregate root `marketing_behavioral_profile`. Invariants: per-contact behavior aggregation distinct from segment predicate state; event-counters with HLC; trait derivation rules declared. Commands: ingest-event, derive-trait, recompute, prune. Events: event-ingested, trait-derived, recomputed, pruned. Counterpart map: HubSpot Behavioral Event + Marketo Activity Log + Mailchimp Audience Insights.

### marketing-asset

Aggregate root `marketing_asset`. Invariants: templates + files + design blocks + snippets + brand kit; per-asset version monotonic; locale variants per pack. Commands: create, amend, publish, retire, duplicate. Events: created, amended, published, retired, duplicated. Counterpart map: HubSpot Design Manager + Marketo Design Studio + Mailchimp Content Studio.

### customer-analytics

Aggregate root `marketing_customer_analytics_report`. Invariants: tenant-visible report (distinct from operator dashboards in `dashboards/`); per-report scope binding; export contracts honor data-class boundaries. Commands: define-report, run-report, schedule-report, export-report. Events: report-defined, report-ran, report-scheduled, report-exported. Counterpart map: HubSpot Marketing Analytics + Marketo Performance Insights + Mailchimp Reports.

### chatflow

Aggregate root `marketing_chatflow`. Invariants: bot decision tree + handoff rules to messenger / contact-center; per-tenant routing; PII redaction at handoff boundary. Commands: define, publish, retire, handoff. Events: defined, published, retired, handoff-triggered. Counterpart map: HubSpot Chatflows + Conversational Bots.

## D. Integration Topology

Inbound:

- iam supplies tenant principal claim with `tenant_class ∈ {demo_trial, paid}`; the microservice never trusts a client-supplied tenant_class.
- policy-engine evaluates Cedar policies under `microservices/marketing-automation/policy/` + `microservices/marketing-automation/policies/`.
- ontology supplies trait and event registry; predicate trees compile against this registry.

Outbound (lateral, contract-named):

- `microservices/workflow-engine/contracts/` — workflow-canvas snapshots execute on the workflow-engine runtime.
- `microservices/mail/contracts/` — marketing email composition hands off to mail send execution per IP-006 contract.
- `microservices/messenger/contracts/` — SMS + push + in-app delivery per IP-006 contract.
- `microservices/consent-graph/contracts/` — lawful-basis source-of-truth; the µservice's consent-audience ledger projects from consent-graph.
- `microservices/analytics/contracts/` — behavioral events feed analytics; customer-facing reports compose analytics views.
- `microservices/intelligence/contracts/` — send-time-optimization, lead-scoring, content-recommendation, subject-line-optimization, smart-segmentation, predicted-customer-lifetime-value, churn-risk predictions.
- `microservices/audit-chain/contracts/` — every state transition emits a sealed audit event.
- `microservices/data-boundary/contracts/` — subject-hash + data-class labeling.
- `microservices/finops/contracts/` — per_usage meter dimensions per billing component.
- `microservices/marketplace/contracts/` — DealSet audience-license settlement per ADR-0314.

Lateral seams (B2B-leader-adjacent):

- `microservices/crm/contracts/` — campaign envelope handoff (engagement-side here; revenue-side in crm); attribution revenue-events sourced from crm.opportunity; ABM target-accounts reference crm.account-master; lifecycle-stage progression handoff with crm.lead.
- `microservices/sites/contracts/` — landing-page boundary; marketing-attached pages here; tenant website root in sites.
- `microservices/social/contracts/` — social-publishing seam; ad-network seam.
- `microservices/contact-center/contracts/` — chatflow handoff for live agent escalation.
- `microservices/calendar/contracts/` — marketing-calendar seam; meeting-scheduler primitive.
- `microservices/forms/contracts/` — marketing-automation.form composes on forms substrate (per ADR-MS-MA-003).

All cross-microservice calls carry tenant_id + principal_id + tenant_class + purpose + trace context + idempotency key + Cedar decision id + audit-chain reference. HTTP/3 + QUIC is default per ADR-0253-amendment. gRPC runs over HTTP/3 internally.

## E. Failure Modes

- **Source-system import drift**: dry-run evidence identifies row, field, transform, data class, and rejection reason. Recovery: import staging table holds rejected rows; manual reconciliation tool emits replay-eligible bundle.
- **Cross-tenant reference attempt**: Cedar denies before domain command execution; emits refusal evidence (EVT-MARKETING-CEDAR-DENIED). Recovery: surface tenant-mapping error to operator UI with corrective action.
- **Duplicate command submission**: idempotency key returns the previous result and increments duplicate metric (`marketing_automation_duplicate_command_total`). Recovery: no recovery needed (deduplication is correct behavior).
- **Regional outage**: writes queue in the tenant home cell; reads expose stale-region metadata via `x-stale-region` header. Recovery: per-cell DR runbook (`runbooks/provider-migration-rollback.md`).
- **Audit-chain outage**: critical state transitions pause; non-critical reads continue with degraded banner. Recovery: audit-chain catch-up worker replays buffered evidence.
- **Pack conflict**: pack resolver blocks activation and opens a workflow-engine remediation task. Recovery: compliance operator reviews via pack-conflict runbook.
- **Demo-trial cap breach**: Cedar denies with policy decision id; tenant gets 429 with `x-demo-trial-cap-hit: <cap_name>` header. Recovery: tenant upgrades to paid; recovery is the conversion flow.
- **Per-usage meter dropped**: cloud-billing emits a meter-replay request; the µservice replays the meter window from event-store. Recovery: replay-worker reconstructs from `EVT-MARKETING-PER-USAGE-*` events.
- **Deliverability fail-closed**: DMARC failure pauses all marketing mail; tenant admin override requires Cedar step-up. Recovery: domain-health investigation runbook.
- **Frequency-cap contention**: CAS retry up to 3x then defer. Recovery: deferred touches re-evaluate on next window expiry.

## F. Per-aggregate runtime traces

Five bespoke runtime traces, one per primary aggregate family, replacing the prior 7×30=210 stamped-bullet template that the Wave-4 audit flagged (I-D1 P0 BIG-8). Each trace names the actual command path, decision branches, audit events, and cross-microservice handoffs without anchor-name token substitution.

### F.1 segment.materialize runtime trace

A marketing-ops principal in tenant `acme.io` posts `POST /v1/marketing-automation/segments/{segment_id}:materialize` with a predicate tree `{all: [{trait: 'account.arr', gte: 50000}, {event: 'trial.invited', within_days: 14}]}`. Layer flow: `rest` validates HTTP/3 transport, idempotency-key header, and tenant principal claim; `usecase::MaterializeSegment` calls `kernel::compile_predicate_tree` which calls ontology over gRPC-over-HTTP/3 to validate each trait/event exists in the tenant's ontology projection. If validation fails the request returns 422 with the offending trait/event id. On success, `usecase` calls policy-engine via Cedar with action `marketingAutomation::MaterializeSegment` and context `{tenant_id, predicate_fields, freshness_floor_ms, tenant_class}`. Cedar denies when (a) the tenant is demo_trial and the active_segments count is already at the cap of 5; (b) the predicate references a restricted trait (e.g., `subject.health.condition` for non-HIPAA tenants); (c) the principal is not in role `marketing.ops`. On allow, `worker::SegmentMaterializer` builds initial snapshot, subscribes to event-cursor, emits `EVT-MARKETING-SEGMENT-MATERIALIZED` with `{tenant_id, segment_id, member_count, freshness_floor_ms, tenant_class, cedar_decision_id}` to audit-chain. As event deltas arrive, the worker emits `EVT-MARKETING-SEGMENT-DELTA-APPLIED` with `{segment_id, event_cursor, added, removed}`. Per-usage meter `segment_materialization` increments via finops contract for paid.per_usage tenants. Cross-microservice handoffs: ontology (validation), policy-engine (Cedar), audit-chain (sealing), data-boundary (subject-hash labeling), finops (per_usage meter), workflow-engine (consume freshness for journey trigger).

### F.2 journey.trigger runtime trace

An event arrives on the eventing substrate for tenant `acme.io`: subject `s_abc123` submitted a form attached to a workflow-canvas trigger. Layer flow: `worker::EventConsumer` reads the event, calls `kernel::resolve_canvas_published_version` to bind the canvas snapshot, then `usecase::TriggerJourney` calls policy-engine with action `marketingAutomation::TriggerJourney` and context `{tenant_id, canvas_id, subject_hash, tenant_class}`. Cedar denies when (a) the tenant is demo_trial and active_journeys count is at cap of 2; (b) the subject has an active suppression in `marketing_consent_audience` for the journey's primary channel × purpose; (c) the subject is in a frequency-cap denial window for the journey's primary channel. On allow, `worker::JourneyRunner` creates a `marketing_journey_run` row with HLC stamp, emits `EVT-MARKETING-JOURNEY-TRIGGERED`, and advances to step 1. Each step advance evaluates step-type-specific logic (send email → mail; send sms → messenger; reserve frequency → frequency-cap aggregate; conditional branch → predicate eval; wait timer → defer with HLC deadline). At each step the runner emits `EVT-MARKETING-JOURNEY-STEP-ADVANCED` with `{journey_run_id, step_index, step_type, decision_branch}`. Per-usage meter `journey_executions` increments at trigger; `email_sends` increments per send step; `frequency_reservations` increments per reserve step. Cross-microservice handoffs: workflow-engine (canvas snapshot), policy-engine (Cedar), mail (send execution), messenger (send execution), audit-chain (sealing), finops (per_usage meters), consent-audience (suppression check), frequency-cap (touch reservation).

### F.3 consent-audience.append-revocation runtime trace

A subject submits an unsubscribe link click for tenant `acme.io` subscription_type `product_updates_newsletter`. Layer flow: `rest` accepts the unsubscribe endpoint (signed-link verification at `kernel::verify_unsubscribe_token`), then `usecase::AppendRevocation` calls policy-engine with action `marketingAutomation::AppendRevocation` and context `{tenant_id, subject_hash, subscription_type, purpose, channel, source: 'subject_initiated'}`. Cedar always allows subject-initiated revocations (CAN-SPAM/CASL/GDPR Article 21 right to object) — denial only triggers if the subject_hash is malformed or the subscription_type does not exist. On allow, `domain::MarketingConsentAudience::append_revocation` writes an append-only HLC-stamped row to `marketing_consent_audience`. The aggregate emits `EVT-MARKETING-SUPPRESSION-APPENDED` with `{tenant_id, suppression_id, subject_hash, purpose, channel, source_vendor: 'oyatie-internal', effective_at_hlc, cedar_decision_id, tenant_class}`. A downstream worker projects the revocation into all active journeys (via consent-audience.check-suppression query) — already-triggered journeys with the revoked channel × purpose either pause the affected step (if step is send) or skip it. Cross-microservice handoffs: consent-graph (lawful-basis source-of-truth update), audit-chain (sealing), workflow-engine (in-flight journey notification), mail+messenger (send-time suppression check), data-boundary (subject-hash projection).

### F.4 attribution.reconcile runtime trace

A RevOps manager in tenant `acme.io` posts `POST /v1/marketing-automation/attribution:reconcile` with `{model: 'position_based_40_20_40', window_start: '2026-04-01', window_end: '2026-04-30'}`. Layer flow: `rest` validates principal role (`revops.manager` or higher), then `usecase::ReconcileAttribution` calls policy-engine with action `marketingAutomation::ReconcileAttribution` and context `{tenant_id, model_version, window_duration_days, tenant_class}`. Cedar denies when (a) demo_trial tenant has attribution_models cap = 1 and is requesting a second model; (b) the principal is not authorised for attribution-reconcile; (c) the revenue-event source (crm) is unavailable. On allow, `worker::AttributionReconciler` runs LoadTouches → LoadRevenueEvent → DeduplicateVendorEvents → AllocateCredit → SealReconciliation. The reconciler reads touches from `marketing_attribution_touch` (segment events, email opens/clicks, form submissions, journey step advances, landing-page conversions) and revenue events from crm.opportunity via the crm contract. Credit allocation is deterministic given (touches, revenue-events, model-version) — same inputs yield same outputs (replay-safe). Emits `EVT-MARKETING-ATTRIBUTION-RECONCILED` with `{tenant_id, run_id, model_version, window_start, window_end, total_revenue, allocated_credit, tenant_class, cedar_decision_id}` and per-touch `EVT-MARKETING-ATTRIBUTION-CREDIT-ALLOCATED`. Per-usage meter `attribution_runs` increments. Cross-microservice handoffs: crm (revenue events), audit-chain (sealing), ontology (touch ↔ revenue lineage), analytics (reconciliation result), finops (per_usage meter), data-boundary (PII_QUASI guarding on subject hashes).

### F.5 deliverability.admit-send runtime trace

A scheduled email send from journey step or campaign asks the deliverability governor for an admit decision. Layer flow: `worker::SendDispatcher` calls `usecase::AdmitSend` with `{tenant_id, domain_ref, intended_count, send_window: 'PT1H'}`. The usecase calls policy-engine with action `marketingAutomation::AdmitSend` and context `{tenant_id, domain_ref, intended_count, current_warmup_state, complaint_rate_ppm, bounce_rate_ppm, dmarc_status, tenant_class}`. Cedar denies when (a) DMARC has failed (fail-closed); (b) complaint_rate_ppm exceeds the pack-overlay threshold; (c) tenant_class is demo_trial and monthly_email_sends cap is hit. On allow, the deliverability aggregate emits `EVT-MARKETING-DELIVERABILITY-ADMITTED` with `{tenant_id, domain_ref, admitted_count, deferred_count, pause_reason: null, warmup_state, cedar_decision_id, tenant_class}`. On deny (DMARC failure), the aggregate transitions warmup state to `blocked` and emits `EVT-MARKETING-DELIVERABILITY-PAUSED` with explicit `pause_reason`. Admitted counts feed mail execution; deferred counts re-evaluate at next window. Per-usage meter `deliverability_admit_decisions` increments. Cross-microservice handoffs: mail (DKIM/SPF/DMARC source-of-truth), policy-engine (Cedar with pack-aware thresholds), audit-chain (sealing), finops (per_usage meter), workflow-engine (notify pending steps of admit budget), abuse-defence (consume complaint signals).

## G. Contracts

- REST: `contracts/openapi-v1.yaml` — OpenAPI 3.2.0 with HTTP/3 `x-transport-default` extension annotated per ADR-0253-amendment; `x-tenant-class-claim` extension documents that tenant_class is gateway-stamped (never client-supplied).
- Events: `contracts/asyncapi-v1.yaml` — AsyncAPI 3.1.0 with `x-audit-chain-seal: required` on every channel; all events include `tenant_class` dimension per ADR-0263.
- Internal RPC: `contracts/marketing-automation-v1.proto` — proto3 over HTTP/3 with mandatory `tenant_id + principal_id + tenant_class + trace_id + idempotency_key + audit_chain_ref` headers.
- Local development variants under `contracts/local-*.yaml` mirror the production surface with relaxed transport (HTTP/1.1 allowed).
- Naming: BNF v4.1.
- Layers: ADR-0105 13-layer enum (api, rest, application, usecase, domain, kernel, adapter, worker, governance, cli, sdk, test).

## H. Per-microservice ADR ownership

The microservice owns its in-µservice ADRs under `decisions/`:

- `decisions/ADR-MS-MA-001-engagement-mutation-envelope.md` — the canonical mutation envelope for engagement-side state (tenant_id + principal_id + purpose + data_class + pack_overlay + idempotency_key + trace_context + audit_chain_target + tenant_class + cedar_decision_id); settles Wave-4 audit Q-001 (campaign ownership boundary between marketing-automation and crm).
- `decisions/ADR-MS-MA-002-landing-page-vs-sites-boundary.md` — settles Wave-4 audit Q-003 (Landing Page ownership) with the marketing-attached-page boundary.
- `decisions/ADR-MS-MA-003-form-vs-forms-microservice-boundary.md` — settles Wave-4 audit Q-004 (Form ownership) with marketing-form composition on forms substrate.

## I. Open boundary decisions

Wave-4 audit §5 enumerates 25 open boundary questions (Q-001..Q-025). The first three are settled by ADR-MS-MA-001..003 (this microservice). Q-005..Q-025 are settled cross-microservice in `docs/decisions/` ADRs (lifecycle-stage ownership, marketing asset ownership, customer-facing analytics ownership, mail integration contract, messenger integration contract, social integration contract, contact-center integration contract, calendar integration contract, intelligence AI primitives contract, multi-source migration path, OCI Always Free decomposition, tenant-class transition flow, GDPR right-to-be-forgotten cross-aggregate orchestrator, HIPAA marketing-engagement governance, UTM/link-parameter primitive, ad-network primitive, campaign boundary with crm, trigger primitive registry, exit/goal primitive registry).
