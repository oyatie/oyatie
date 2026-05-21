---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-marketing-automation
microservice: marketing-automation
status: wave-15a-big8-remediation
date: 2026-05-21
owner_team: axis-marketing-automation + council-product
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
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
retired_adr_bindings:
  - ADR-0315
  - ADR-0316
companion_docs:
  - microservices/marketing-automation/ARCHITECTURE.md
  - microservices/marketing-automation/compliance.md
  - microservices/marketing-automation/manifest.json
  - microservices/marketing-automation/coherence-audit-2026-05-20.md
  - microservices/marketing-automation/REMEDIATION-NOTES-2026-05-21.md
planned_enforcement_ref: oya-governance-marketing-automation-doc-suite
---

# PRD-marketing-automation: Marketing Automation

## A. Problem

Marketing Automation is the Phase-4A.5 Big-8 customer-engagement substrate. The microservice closes B2B-leader coverage for the engagement lifecycle (segment, journey, attribution, suppression, deliverability, lead-scoring, lifecycle-stage, ABM) under a HubSpot Marketing Hub primary anchor with Adobe Marketo Engage and Mailchimp as flanking counterparts. The operational reason for a dedicated microservice is that segmentation, journeys, suppression, attribution, and frequency capping need one tenant-owned outbound-engagement owner — scattering campaign state across mail, messenger, social, contact-center, and crm yields cross-channel fatigue, undetectable attribution leakage, broken right-of-erasure, and per-vendor lock-in.

The microservice owns 25 bounded contexts plus 5 differentiator capability slices (IP-026..IP-030) that exceed counterpart depth on auditable evidence chain. The microservice does not own tenant identity (iam), Cedar policy internals (policy-engine), workflow runtime internals (workflow-engine), ontology storage (ontology), email send execution (mail), messaging delivery (messenger), payments rails (commerce + cloud-billing), marketplace settlement (marketplace), sales-record state (crm), tenant website root (sites), social publishing (social), live agent handoff destination (contact-center), or meeting scheduler (calendar).

ADR-0316 (capability-tier activation doctrine) is retired per the Wave-4 audit; the µservice now adopts ADR-0244 tenant-class primitive (`{demo_trial, paid}`) with composable paid billing components `{revenue_share, per_seat, per_usage}`. ADR-0315 (SAP-parity doctrine) is removed from this µservice's binding ADRs — the Big-8 marketing-automation counterpart family is HubSpot/Marketo/Mailchimp, not SAP.

## B. Target Users

- **Marcus Chen**, operations owner at his 600-person B2B SaaS company: needs sub-second buying-committee segment freshness to launch ABM nurture programs without recreating Marketo Smart List refresh delays; runs paid tenancy with per_seat marketing-ops seats + per_usage email_sends meter; deploys on oyatie-public-cloud.
- **Yejin Park**, owner of a side-business that must stay compliant while she works another job: needs demo_trial onboarding to validate Oyatie marketing-automation against her HubSpot free tier before committing budget; needs OCI Always Free deployment profile to keep evaluation cost zero; converts to paid revenue_share when ready to consume marketplace audience-license dealsets.
- **Diana Alvarez**, principal at an agency serving several tenant clients: needs per-tenant isolation across her clients' workspaces (some on AWS-guest, some on on-prem); needs migration playbooks from HubSpot Marketing Hub and Mailchimp because her clients arrive from those vendors; needs the µservice to honor per-tenant pack overlays (GDPR for EU clients, KR-PIPA for KR clients, CASL for Canadian clients).
- **Nadia Singh**, enterprise administrator responsible for pack activation: needs to activate HIPAA + EU-AI-Act-Marketing-Personalization packs on her healthcare-adjacent paid tenancy; needs Cedar policy evaluation to fail-closed on PHI in marketing communications until explicit operator review.
- **Omar Watkins**, SRE accountable for incident evidence and rollback: needs deliverability-warmup fail-closed on DMARC failure with audit-chain sealed evidence; needs runbook coverage for every operational scenario; needs cross-region failover via per-deployment-context multi-region runbooks (AWS via Route 53 + RDS read-replica; OCI via Traffic Management + Autonomous DB DR; on-prem via customer DR plan).
- **Hana Mori**, auditor tracing policy decisions across vendors: needs append-only HLC-stamped consent-suppression ledger to discharge GDPR Article 15/17/21 obligations; needs deterministic attribution replay (same touches + revenue events + model version → same credit allocation) to defend revenue-attribution claims to her audit committee.
- **Pat Lee**, RevOps director at a mid-market manufacturing tenant: needs multi-touch attribution that links marketing touches to crm.opportunity revenue events with cryptographic seal — not vendor heuristic; needs first-touch, last-touch, linear, position-based (40/20/40), and time-decay models out of the box; needs custom model with explicit formula.
- **Sam Okafor**, marketing operations engineer at a B2C e-commerce tenant: needs cross-channel frequency cap that prevents email + SMS + push fatigue on the same subject × purpose window — counterparts cap per-campaign or per-list, allowing cross-product fatigue.

## C. User Stories

User stories are organized by bounded context and persona. Each story is bespoke — acceptance criteria reference specific contracts, audit events, SLO targets, and migration evidence rather than the prior verbatim template.

### C.1 Segment & list

- **US-001 (Marcus, segment)**: As a marketing-ops operator launching ABM nurture, I want to define a buying-committee segment whose membership refreshes in under one second from product events so that nurture sends fire while intent is fresh.  
  Acceptance: `POST /v1/marketing-automation/segments/{segment_id}:materialize` accepts a predicate tree validated against ontology; `EVT-MARKETING-SEGMENT-MATERIALIZED` is sealed within 250 ms p95 per IP-026 SLO; downstream journey-trigger workers consume `EVT-MARKETING-SEGMENT-DELTA-APPLIED` with `freshness_floor_ms ≤ 750`.

- **US-002 (Diana, static-list)**: As an agency principal migrating a HubSpot Static List from a client, I want explicit-membership import with row-level provenance so that I can defend audit questions about how each contact entered the list.  
  Acceptance: `marketing_static_list` import dry-run produces rejection report with `source_row_id + transform_id + reason + retry_plan`; on success each membership row carries `source_vendor + source_object_id + source_timestamp + import_batch_id`.

- **US-003 (Yejin, segment demo_trial cap)**: As a demo_trial tenant evaluating Oyatie, I want clear guard-rails so that I can explore segmentation without surprise charges, and a frictionless upgrade flow when I'm ready to convert.  
  Acceptance: `marketingAutomation::MaterializeSegment` Cedar policy denies with 429 + `x-demo-trial-cap-hit: active_segments` when `active_segments_count >= 5`; the response body links to `/v1/billing/conversion/from-demo-trial`; conversion preserves all defined segments.

### C.2 Email & landing & form

- **US-004 (Marcus, email)**: As a marketing-ops operator composing a product-launch email, I want dynamic personalization tokens + smart content rules + A/B variants + send-time-optimization to align with HubSpot Marketing Email parity.  
  Acceptance: `marketing_email` aggregate validates token resolution against ontology traits; smart content rules compile against subject predicate tree; A/B variant set sums to 100% allocation; send_window resolves via send-time-optimization bounded context.

- **US-005 (Marcus, landing-page)**: As a marketing-ops operator building a campaign landing page, I want template + form attachment + conversion goal + A/B variant + SEO metadata to match HubSpot Landing Pages without recreating CMS Hub.  
  Acceptance: `marketing_landing_page` references a `marketing_form.form_id`; conversion goal binds to attribution touch-kind; SEO metadata delegates to sites SEO seam per ADR-MS-MA-002.

- **US-006 (Sam, form)**: As a marketing operations engineer capturing leads from a third-party site, I want collected-form variant + signed-token submission + GDPR consent block + post-submit redirect to match HubSpot Form parity.  
  Acceptance: `marketing_form.collected_variant` accepts submissions from JS SDK; submission idempotency-key prevents duplicates; GDPR consent block resolves per residency pack overlay; post-submit redirect URL validated against tenant allow-list.

### C.3 Workflow & journey

- **US-007 (Diana, workflow-canvas)**: As an agency principal authoring nurture workflows, I want a drag-and-drop visual canvas that produces immutable published snapshots so that running journeys cannot be silently mutated by canvas edits.  
  Acceptance: `marketing_workflow_canvas` is DAG-validated (no cycles); `publish` creates an immutable snapshot referenced by `marketing_journey_run.canvas_snapshot_id`; canvas edits create a new draft version without affecting running journeys.

- **US-008 (Marcus, journey runtime)**: As a marketing-ops operator running multi-step engagement, I want deterministic replay so that re-running the same journey from the same trigger event produces the same step sequence (same Cedar decisions, same suppression checks, same frequency reservations).  
  Acceptance: `marketing_journey_run.replay` consumes the original trigger event + canvas snapshot id + HLC stamp and produces an identical step sequence; differences are audited as `EVT-MARKETING-JOURNEY-REPLAY-DRIFT`.

### C.4 Consent & suppression (IP-027 differentiator)

- **US-009 (Hana, consent-audience)**: As an auditor responding to a GDPR Article 15 (right of access) request, I want an append-only HLC-stamped consent ledger so that I can produce a complete consent history without vendor cooperation.  
  Acceptance: `consent-audience.generate_dsr_report` returns all `consent_appended + revocation_appended` rows for the subject across all channels × purposes with HLC stamps and source provenance; the report is sealed by audit-chain as `EVT-DSR-REPORT-GENERATED`.

- **US-010 (Hana, right to erasure)**: As an auditor responding to a GDPR Article 17 (right to erasure) request, I want tombstone projection that hides erased subjects from all read paths while keeping the underlying ledger append-only for regulator reconstruction.  
  Acceptance: erasure applies a tombstone row that suppresses subject_hash from all `marketing_segment + marketing_journey_run + marketing_email_tracking_event + marketing_behavioral_profile` reads; the underlying `marketing_consent_audience` row remains for regulator inspection; erasure is sealed as `EVT-MARKETING-ERASURE-APPLIED`.

### C.5 Attribution (IP-028 differentiator)

- **US-011 (Pat, attribution)**: As a RevOps director defending attribution claims to my audit committee, I want deterministic attribution replay sourced from crm.opportunity revenue events with cryptographic seal.  
  Acceptance: `attribution.reconcile` accepts a model parameter set + window + revenue-events sourced from crm; same inputs yield same outputs; `EVT-MARKETING-ATTRIBUTION-RECONCILED` is sealed with credit allocation per touch; replay against the same model version reproduces credit deterministically.

### C.6 Deliverability (IP-029 differentiator)

- **US-012 (Omar, deliverability)**: As an SRE accountable for marketing-mail blast radius, I want DMARC failure to automatically pause all marketing mail for the affected domain.  
  Acceptance: `marketingAutomation::AdmitSend` Cedar policy denies on DMARC failure regardless of warmup state; tenant admin override requires Cedar step-up authentication + audit; `EVT-MARKETING-DELIVERABILITY-PAUSED` is emitted with `pause_reason: 'dmarc_failure'`.

### C.7 Frequency cap (IP-030 differentiator)

- **US-013 (Sam, frequency-cap)**: As a marketing operations engineer preventing cross-channel fatigue, I want a single frequency window per (subject, purpose, channel) that atomically reserves touches across email + SMS + push + in-app.  
  Acceptance: `frequency-cap.reserve_touch` is atomic via CAS retry up to 3x; denial returns `defer_until` with HLC; legal-notice purpose bypasses cap via Cedar; `EVT-MARKETING-FREQUENCY-TOUCH-RESERVED` and `EVT-MARKETING-FREQUENCY-CAP-DENIED` are sealed.

### C.8 Tenant-class conversion stories

- **US-014 (Yejin, demo_trial → paid conversion)**: As a side-business owner converting from demo_trial to paid, I want a non-destructive upgrade path that preserves my defined segments, journeys, attribution models, and consent ledger.  
  Acceptance: `tenancy.upgrade_to_paid` removes demo_trial caps; preserves all aggregates; binds the chosen billing_components (revenue_share + per_seat + per_usage in any combination); `EVT-TENANT-CLASS-TRANSITION` is sealed with `from: demo_trial, to: paid, billing_components: [...]`.

- **US-015 (Diana, per-tenant pack overlay)**: As an agency principal serving multi-jurisdiction clients, I want per-tenant pack overlays so that one EU client honors GDPR + ePrivacy + DSA while a Canadian client honors CASL + PIPEDA.  
  Acceptance: gateway resolves `pack_overlays` per tenant principal claim; Cedar policies evaluate higher-restriction-wins; pack-conflict is surfaced via `EVT-PACK-CONFLICT` not silently merged.

### C.9 Lead scoring, lifecycle, ABM

- **US-016 (Marcus, lead-scoring)**: As a marketing-ops operator scoring inbound leads, I want demographic + behavioral + predictive score components that decay over time so that scoring reflects current intent not historical engagement.  
  Acceptance: `lead-scoring.score_subject` composes demographic + behavioral + predictive components; decay applies per `decay_half_life_days`; manual adjustments are auditable; `EVT-MARKETING-LEAD-SCORED` is sealed.

- **US-017 (Pat, lifecycle-stage)**: As a RevOps director enforcing the funnel discipline, I want monotonic lifecycle progression with auditable downgrade evidence.  
  Acceptance: `lifecycle-stage.progress` is monotonic by default; `downgrade` requires principal role `revops.manager` + reason; `EVT-MARKETING-LIFECYCLE-PROGRESSED` and `EVT-MARKETING-LIFECYCLE-DOWNGRADED` are sealed.

- **US-018 (Diana, abm)**: As an agency principal running ABM for a B2B client, I want target accounts bound to crm.account-master with account-level workflow trigger references and intent-data ingestion.  
  Acceptance: `abm.target_account` references `crm.account_id`; account score composes demographic + behavioral + intent components; account-level workflow-canvas triggers fire when account score crosses threshold; `EVT-MARKETING-ABM-ACCOUNT-TARGETED` is sealed.

### C.10 A/B test, send-time-optimization, email tracking, webhook

- **US-019 (Marcus, a-b-test)**: As a marketing-ops operator running A/B tests, I want statistical significance threshold + auto winner selection + audit-chain sealed conclusion.  
  Acceptance: `a-b-test.configure` accepts variant_set with allocation summing to 100; `significance_threshold` default 0.95; auto winner selection at significance; `EVT-MARKETING-AB-TEST-CONCLUDED` is sealed with winner + p_value + sample size.

- **US-020 (Marcus, send-time-optimization)**: As a marketing-ops operator scheduling sends, I want per-recipient optimal-send-time prediction that respects frequency-cap reservations and deliverability admit decisions.  
  Acceptance: `send-time-optimization.predict_window` sources prediction from intelligence µservice; window honors frequency-cap reservation (does not schedule into denial windows) and deliverability admit budget; fallback window applies when prediction confidence is low.

- **US-021 (Sam, email-tracking)**: As a marketing operations engineer respecting privacy signals, I want email tracking to honor GPC / DNT signals and Apple Mail Privacy Protection aggregation.  
  Acceptance: tracking-pixel only emits aggregated counts for Apple MPP-confirmed opens; GPC / DNT signals suppress click tracking per pack overlay; `EVT-MARKETING-TRACKING-RECORDED` includes `privacy_signal_applied` dimension.

- **US-022 (Diana, webhook-subscription)**: As an agency principal integrating Oyatie marketing-automation with my agency's project-management tool, I want signed HMAC-SHA-256 webhook delivery with retry policy.  
  Acceptance: `webhook-subscription.subscribe` requires URL + event filter + signing secret; deliveries include signed timestamp + payload signature; retry policy is exponential backoff with max 6 attempts over 24 hours; HTTP/3 delivery by default.

### C.11 Calendar, behavioral-profile, marketing-asset, customer-analytics, chatflow

- **US-023 (Marcus, marketing-calendar)**: As a marketing-ops operator coordinating multi-channel campaigns, I want a calendar view that detects conflicts when two campaigns target overlapping audiences in the same week.  
  Acceptance: `marketing-calendar.conflict_detection` flags overlapping audience × week conflicts; conflict resolution requires explicit acknowledgement or reschedule; `EVT-MARKETING-CALENDAR-CONFLICT-DETECTED` is sealed.

- **US-024 (Pat, behavioral-profile)**: As a RevOps director understanding contact intent, I want per-contact behavioral profile that ingests custom behavioral events with HLC stamps and derives traits for segmentation.  
  Acceptance: `behavioral-profile.ingest_event` accepts custom event schemas; `derive_trait` rules compile against event aggregates; derived traits feed segment predicate trees.

- **US-025 (Diana, marketing-asset)**: As an agency principal managing client brand assets, I want per-tenant template library with per-locale variants so that one GDPR-pack client's templates carry the required disclosure block while another client's templates carry CAN-SPAM disclosure.  
  Acceptance: `marketing-asset.locale_variants` resolve per pack overlay; brand kit per tenant; publish requires accessibility audit pass.

## D. Functional Requirements

Functional requirements are organized per bounded context × verb. Each FR encodes the specific acceptance criteria for that verb on that aggregate.

### D.1 segment

- **FR-001 `segment.define`**: tenant scope required; predicate tree validates against ontology trait/event registry; freshness_floor_ms in [100, 86400000]; Cedar gates on tenant_class (demo_trial cap=5 segments); audit event `EVT-MARKETING-SEGMENT-DEFINED`; rollback evidence.
- **FR-002 `segment.amend`**: tenant scope; predicate tree re-validates; member-count delta recomputed; freshness invariant preserved; Cedar gates on principal role; audit event `EVT-MARKETING-SEGMENT-AMENDED`.
- **FR-003 `segment.materialize`**: idempotency key required; freshness floor honored (p95 ≤ 250 ms per IP-026 SLO); event-cursor monotonic; per_usage meter `segment_materializations` increments; audit event `EVT-MARKETING-SEGMENT-MATERIALIZED`.
- **FR-004 `segment.replay`**: replay against historical event-store cursor; deterministic given (predicate_tree, cursor, ontology_snapshot); audit event `EVT-MARKETING-SEGMENT-REPLAYED`.

### D.2 campaign

- **FR-005 `campaign.create`**: tenant scope; engagement-side only (revenue lineage via crm); goal + channel + asset_refs[] + budget_metadata; UTM scheme honored; Cedar gates on principal role; audit event `EVT-MARKETING-CAMPAIGN-CREATED`.
- **FR-006 `campaign.attach-asset`**: asset must be marketing_email | marketing_landing_page | marketing_form | social_asset_ref (via social contract); attachment is auditable; audit event `EVT-MARKETING-CAMPAIGN-ASSET-ATTACHED`.
- **FR-007 `campaign.publish`**: validates all attached assets are published; emits `EVT-MARKETING-CAMPAIGN-PUBLISHED`; engages workflow-canvas runtimes for attached workflows.
- **FR-008 `campaign.retire`**: stops new journey triggers; preserves audit lineage; emits `EVT-MARKETING-CAMPAIGN-RETIRED`.

### D.3 journey & workflow-canvas

- **FR-009 `workflow-canvas.publish`**: DAG-validated; step types in canonical registry; published snapshot is immutable; emits `EVT-MARKETING-WORKFLOW-CANVAS-PUBLISHED`.
- **FR-010 `journey.trigger`**: requires canvas_snapshot_id + trigger_event_ref; Cedar gates on tenant_class (demo_trial cap=2 active journeys); suppression check before trigger; emits `EVT-MARKETING-JOURNEY-TRIGGERED`.
- **FR-011 `journey.advance-step`**: idempotent per (journey_run_id, step_index); HLC-stamped; emits `EVT-MARKETING-JOURNEY-STEP-ADVANCED`.
- **FR-012 `journey.replay`**: deterministic given (trigger_event, canvas_snapshot, HLC); differences emit `EVT-MARKETING-JOURNEY-REPLAY-DRIFT`.

### D.4 email & landing & form

- **FR-013 `email.compose`**: tenant scope; subject + content + tokens validated against ontology; smart content rules compile; A/B variant_set sums to 100; accessibility audit pass; emits `EVT-MARKETING-EMAIL-COMPOSED`.
- **FR-014 `email.publish`**: validates accessibility + content + tokens; immutable once published; emits `EVT-MARKETING-EMAIL-PUBLISHED`.
- **FR-015 `email.schedule-send`**: requires deliverability admit decision + frequency-cap reservation + suppression check; per_usage meter `email_sends` increments; emits `EVT-MARKETING-EMAIL-SCHEDULED-SEND`.
- **FR-016 `landing-page.publish`**: validates form attachment if present + SEO metadata + accessibility; immutable once published; emits `EVT-MARKETING-LANDING-PAGE-PUBLISHED`.
- **FR-017 `form.submit`**: idempotency key required; signed-token verification for collected variant; GDPR consent block resolved per pack; submission triggers downstream subscriber-creation + journey-trigger; emits `EVT-MARKETING-FORM-SUBMITTED`.

### D.5 consent-audience & subscription-type

- **FR-018 `consent-audience.append-consent`**: append-only HLC-stamped; per (subject_hash, channel, purpose); evidence chain via consent-graph; emits `EVT-MARKETING-CONSENT-APPENDED`.
- **FR-019 `consent-audience.append-revocation`**: subject-initiated always allowed by Cedar (Article 21); HLC-stamped; emits `EVT-MARKETING-SUPPRESSION-APPENDED`.
- **FR-020 `consent-audience.check-suppression`**: deterministic given (subject_hash, channel, purpose, HLC); fail-closed on ledger unavailability; p99 ≤ 60 ms per IP-027 SLO; emits `EVT-MARKETING-SEND-SUPPRESSED` on deny.
- **FR-021 `subscription-type.subject-subscribe`**: requires opt-in evidence signed by consent-graph; per pack overlay disclosure block resolved; emits `EVT-MARKETING-SUBSCRIPTION-SUBSCRIBED`.
- **FR-022 `subscription-type.subject-unsubscribe`**: subject-initiated always allowed; cascades to consent-audience.append-revocation; emits `EVT-MARKETING-SUBSCRIPTION-UNSUBSCRIBED`.

### D.6 attribution

- **FR-023 `attribution.configure-model`**: model_version monotonic; formula explicit for custom models; Cedar gates on principal role + tenant_class (demo_trial cap=1 model); emits `EVT-MARKETING-ATTRIBUTION-MODEL-CONFIGURED`.
- **FR-024 `attribution.reconcile`**: requires (model_version, window_start, window_end, touches_source, revenue_events_source); deterministic given inputs; per_usage meter `attribution_runs` increments; emits `EVT-MARKETING-ATTRIBUTION-RECONCILED`.
- **FR-025 `attribution.replay`**: deterministic; differences from prior run emit `EVT-MARKETING-ATTRIBUTION-REPLAY-DRIFT`.

### D.7 deliverability & frequency-cap

- **FR-026 `deliverability.admit-send`**: Cedar gates on DMARC status + complaint_rate + warmup_state + tenant_class (demo_trial cap=5000 monthly_email_sends); fail-closed on DMARC failure; per_usage meter `deliverability_admit_decisions` increments; emits `EVT-MARKETING-DELIVERABILITY-ADMITTED` or `EVT-MARKETING-DELIVERABILITY-PAUSED`.
- **FR-027 `deliverability.override`**: requires Cedar step-up + tenant admin role + reason; auditable; emits `EVT-MARKETING-DELIVERABILITY-OVERRIDE-APPLIED`.
- **FR-028 `frequency-cap.reserve-touch`**: atomic CAS retry up to 3x then defer; legal_notice purpose bypass via Cedar; per_usage meter `frequency_reservations` increments; emits `EVT-MARKETING-FREQUENCY-TOUCH-RESERVED` or `EVT-MARKETING-FREQUENCY-CAP-DENIED`.

### D.8 lead-scoring, lifecycle-stage, ABM, A/B test, STO, email-tracking, webhook, calendar, behavioral-profile, marketing-asset, customer-analytics, chatflow

- **FR-029 `lead-scoring.score-subject`**: composes demographic + behavioral + predictive components; decay applied; manual adjustments auditable; emits `EVT-MARKETING-LEAD-SCORED`.
- **FR-030 `lifecycle-stage.progress`**: monotonic by default; downgrade requires role + reason; emits `EVT-MARKETING-LIFECYCLE-PROGRESSED` or `EVT-MARKETING-LIFECYCLE-DOWNGRADED`.
- **FR-031 `abm.target-account`**: binds to crm.account_id via crm contract; tenant_class cap (demo_trial cap=25 target accounts); emits `EVT-MARKETING-ABM-ACCOUNT-TARGETED`.
- **FR-032 `a-b-test.configure`**: variant_set sums to 100; significance_threshold default 0.95; tenant_class cap (demo_trial cap=1 a/b test); emits `EVT-MARKETING-AB-TEST-CONFIGURED`.
- **FR-033 `a-b-test.conclude`**: auto winner at significance; emits `EVT-MARKETING-AB-TEST-CONCLUDED` with winner + p_value + sample size.
- **FR-034 `send-time-optimization.predict-window`**: sources prediction from intelligence; honors frequency-cap + deliverability; emits `EVT-MARKETING-STO-WINDOW-PREDICTED`.
- **FR-035 `email-tracking.record-open`**: GPC / DNT / Apple MPP applied; aggregated-only where required; emits `EVT-MARKETING-EMAIL-TRACKING-OPEN`.
- **FR-036 `webhook-subscription.deliver`**: HMAC-SHA-256 signing; signed timestamp; HTTP/3 default; per_usage meter `webhook_deliveries` increments; emits `EVT-MARKETING-WEBHOOK-DELIVERY-ATTEMPTED`.
- **FR-037 `marketing-calendar.schedule`**: conflict-detection per (channel, audience_overlap, week); emits `EVT-MARKETING-CALENDAR-SCHEDULED` or `EVT-MARKETING-CALENDAR-CONFLICT-DETECTED`.
- **FR-038 `behavioral-profile.ingest-event`**: HLC-stamped; per-tenant event-schema registry; emits `EVT-MARKETING-BEHAVIOR-EVENT-INGESTED`.
- **FR-039 `marketing-asset.publish`**: per-locale variants resolved; accessibility audit; brand kit consistency; emits `EVT-MARKETING-ASSET-PUBLISHED`.
- **FR-040 `customer-analytics.run-report`**: scope-bound; export honors data-class boundaries; emits `EVT-MARKETING-CUSTOMER-REPORT-RAN`.
- **FR-041 `chatflow.handoff`**: PII redaction at boundary; routes to messenger / contact-center per chatflow rule; emits `EVT-MARKETING-CHATFLOW-HANDOFF-TRIGGERED`.

### D.9 Tenant-class & billing components

- **FR-042 `tenant_class.read`**: principal claim resolution at gateway; never client-supplied; per ADR-0244.
- **FR-043 `tenant_class.transition`**: demo_trial → paid is non-destructive; preserves all aggregates; binds billing_components; emits `EVT-TENANT-CLASS-TRANSITION`.
- **FR-044 `per_usage.meter`**: meters per (event_class, tenant_id, billing_period); audit-chain sealed; finops contract delivery.
- **FR-045 `revenue_share.bind`**: triggered by marketplace DealSet settlement per ADR-0314; revenue lineage to crm.

## E. Non-Functional Requirements

- **Maintainability**: Operational-concern doctrine per ADR-0245 keeps product labels out of microservice boundaries; new services exist only for distinct operational concerns. For marketing-automation, every commit names tenant scope, principal role, audit-chain target, and rollback evidence.
- **Observability**: Every aggregate emits audit-chain events with tenant_class + tenant_id + principal_id + cedar_decision_id + trace_id dimensions per ADR-0263. Operator dashboards under `dashboards/` and customer-facing reports via customer-analytics bounded context.
- **Scalability**: Tenant + cell + data-class + workload partition prevents cross-tenant impact. Per-cell capacity per ADR-0248 cellular topology.
- **Performance**: Interactive operations carry p50/p95/p99 budgets per `slos/`. Long-running operations (segment materialization, attribution reconcile, backfill replay) are async with progress projections.
- **Optimization**: Cost dimensions include tenant + tenant_class + deployment_context + cell + data_class + per_usage_meter_class + workflow_template + migration_batch (ADR-0263).
- **Availability**: paid tenant target 99.95% on tier-1 cells; demo_trial best-effort (no contractual SLO).
- **Latency**: simple tenant-scoped command p95 target 300 ms; bulk operations async.
- **Capacity**: per-tenant_class × per-deployment-context capacity grid in `capacity-model.md`.
- **Quality**: unit + property + migration + replay + authorization + contract tests gated by CI per ADR-0297.

### DR posture (ADR-0343)

- Target: RTO <= 1800 s and RPO <= 300 s for consent, suppression, journey, segment, attribution, and deliverability state, matching `manifest.json#dr`.
- Compliance floors considered: EU-AI-ACT-2024-HIGH-RISK requires 1800 s / 300 s with multi-region DR; HIPAA-2024 requires 3600 s / 300 s; KR-PIPA sensitive floors require 7200 s / 600 s and KR resident-registration-number floors require 3600 s / 300 s; SOC2-T2 requires 14400 s / 900 s; ISO27001-2022 requires 14400 s / 3600 s. The effective target is 1800 s / 300 s.
- Failover runbook reference: `runbooks/provider-migration-rollback.md`, `multi-region.md`, `iac/dr-failover.yaml`, and `IP-022-chaos-drill-pack.md`. The manifest substrate is `postgres_wal_g`, `valkey_cluster`, and `object_storage_versioned`; failover verification must prove suppression checks fail closed and journey cursors replay without duplicate sends.
- Multi-region active-active posture: `true` in `manifest.json`; active-active applies to segment reads, deliverability health, journey-admission replicas, and attribution summaries, while consent, suppression, frequency reservations, and send decisions still require idempotent single-owner commit semantics.
- WHY: campaigns can pause, replay, or continue safely after a regional outage while consent and suppression evidence remains stronger than throughput pressure.

### Capacity model (ADR-0340)

- Manifest source: `manifest.json#capacity_model` declares the PRD capacity baseline.
- Per-tenant baseline: reserve 0.12 vCPU, 256 MiB RAM, 4 GB segment/consent/attribution working storage, 5 Postgres connections, 8 Valkey/cache connections, and 24 outbound HTTP slots for mail, messenger, CRM, ontology, analytics, and consent-graph calls.
- Scaling dimension: `per_message`, because journey events, segment materialization, provider callbacks, and consent filters drive load more than seat count.
- Cell placement class: Tier-3 product cell. Rationale: marketing state is tenant-owned product data; mail delivery, consent graph, identity, and analytics remain separate owners.
- Autoscaling boundaries: segment materializer and journey workers floor at 3 replicas and scale to 80 per cell; suppression checks floor at 3 and scale to 100 for the 50k checks/s/cell target; attribution workers floor at 2 and scale to 40 for 200 jobs/hour/cell; deliverability/frequency workers shed background work before interactive sends.
- WHY: the model serves sub-second segment freshness, high-volume suppression checks, and deterministic attribution without letting background reconciliation delay consent or send-admission gates.

### Sustainability + cost attribution (ADR-0344)

- Per-call emission claim: every segment materialization, journey step, suppression check, attribution reconcile, deliverability decision, frequency reservation, and audit export row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours`.
- Provider routing affected by carbon: yes for send-time optimization, attribution reconciliation, segment backfills, and report rebuilds when legal windows allow; no for suppression checks, consent revocations, DMARC fail-closed pauses, or PHI/operator-review decisions.
- Per-tenant cost transparency surface: customer-analytics and tenant billing show cost by campaign, segment, journey, channel, provider, cell, compliance pack, and per_usage meter.
- WHY: marketing teams can defend campaign cost, carbon, and regulatory reporting under CSRD, SB-253, and SEC climate disclosure without delaying consent-sensitive actions.

### API versioning posture (ADR-0342)

- Public API version model: date carrier triplet using `Oyatie-Version: YYYY-MM-DD`, URL prefix `/v/<YYYY-MM-DD>/marketing-automation/...`, and proto3 field `oyatie_version`.
- SDK semver model: marketing SDKs use `major.minor.patch` for segment, journey, suppression, attribution, and deliverability clients.
- Support window: last N=3 public API dates are supported for at least 180 days.
- Per-tenant pinning supported: yes, especially for HubSpot, Marketo, Mailchimp, Iterable, and Braze migration windows.
- Internal-mesh exemption: yes. ADR-0145 direct gRPC remains valid for mail, messenger, crm, consent, ontology, and workflow-engine internal calls.

## F. UX Flows

- **segment**: discover ontology traits → preview predicate → request Cedar permit → materialize → inspect freshness + member count → seal audit event.
- **journey**: author canvas → validate DAG → publish snapshot → trigger from event → execute steps (with Cedar + suppression + frequency at each step) → seal step audit events.
- **attribution**: configure model → load touches + revenue events → reconcile credit → seal reconciliation → export to customer-analytics report.
- **consent**: receive subject-initiated revocation → append HLC-stamped ledger row → project to all read paths → seal `EVT-MARKETING-SUPPRESSION-APPENDED`.
- **deliverability**: monitor DMARC + complaint + bounce → admit-or-pause send → emit warmup state event → cascade to journey + campaign send admission.

## G. Success Metrics

- **Counterpart coverage**: HubSpot Marketing Hub UNION-coverage ≥ 90% per `feature-parity-matrix-2026-05-20.md`; Marketo and Mailchimp UNION-coverage ≥ 85%.
- **Differentiator depth**: IP-026..IP-030 maintain DIFFERENTIATOR status vs counterpart depth (sub-second freshness, append-only consent, deterministic attribution replay, DMARC fail-closed, cross-channel atomic frequency cap).
- **Authorization**: 100% of mutations pass through Cedar default-deny.
- **Observability**: 100% of state transitions emit metric + trace + structured log + audit-chain event with tenant_class dimension.
- **Tenant-class adoption**: 11/11 surfaces present (manifest, PRD, Cedar, OpenAPI, audit events, SLOs, per_usage meters, capacity, cost, migration, demo_trial_caps).
- **Migration**: at least 3 counterpart migration playbooks (HubSpot + Marketo + Mailchimp) with field-level mapping.

## H. Compliance Impact

- **SOC-2, ISO-27001**: audit-chain seal + Cedar default-deny + observability emission satisfy.
- **GDPR**: append-only consent-suppression ledger (IP-027) satisfies Article 15 (right of access), Article 17 (right to erasure via tombstone), Article 21 (right to object).
- **LGPD**: GDPR-equivalent satisfaction.
- **KR-PIPA**: KR-PIPA-pack overlay adds residency + retention controls; pack-overlay-disclosure block in subscription-type + form.
- **CPRA**: GPC signal respect in email-tracking; right-to-opt-out cascade through consent-audience.
- **CAN-SPAM**: unsubscribe link mandatory in subscription-type; physical address in email composition.
- **CASL**: explicit opt-in evidence required; subscription-type.opt_in_evidence signed by consent-graph.
- **HIPAA**: marketing communications touching PHI require explicit operator review per Q-020; Cedar gates fail-closed on PHI traits in predicate trees for non-HIPAA tenants.
- **ePrivacy Directive**: cookie-consent in form + landing-page; tracking-pixel resolution per pack.
- **TCPA**: SMS frequency-cap and quiet-hours enforcement via frequency-cap + send-time-optimization.
- **EU-AI-Act-Marketing-Personalization**: transparency block on AI-personalized content; opt-out per recipient via subscription-type.

## I. Open Questions

Settled in this remediation pass:
- Q-001 (Campaign boundary between marketing-automation and crm): settled by ADR-MS-MA-001 (engagement-side here; revenue-side in crm).
- Q-003 (Landing Page ownership): settled by ADR-MS-MA-002.
- Q-004 (Form ownership boundary): settled by ADR-MS-MA-003.

Open (tracked in companion ADRs):
- Q-002 (Email ownership boundary with mail): contract specified in IP-006; ADR-MS-MA-004 to be authored.
- Q-005 (Lead ownership): handoff specified to crm.lead.
- Q-006 (Lifecycle Stage progression seam with crm): handoff via lifecycle-stage bounded context.
- Q-007..Q-014 (cross-microservice questions): tracked in `docs/decisions/`.
- Q-015 (multi-source migration from HubSpot + Marketo + Mailchimp combined): authored in migration playbooks.
- Q-018 (demo_trial caps registry): bound in `manifest.json` `demo_trial_caps`.
- Q-019 (GDPR right-to-be-forgotten cross-aggregate orchestrator): consent-audience.tombstone projection covers; full orchestrator IP planned.
- Q-020 (HIPAA marketing-engagement governance): operator-review gate authored.
- Q-021 (UTM / link-parameter primitive): ADR-MS-MA-004 to be authored.
- Q-022 (ad-network primitive): delegated to advertising-platform µservice; seam declared.
- Q-023 (campaign boundary with crm): settled by ADR-MS-MA-001.
- Q-024 (journey trigger registry): authored in workflow-canvas step-registry.
- Q-025 (journey exit/goal registry): authored in workflow-canvas step-registry.

## J. Out of Scope

- Recreating a vendor suite boundary (Marketing Cloud / Hub-suite).
- Sharing database tables with adjacent microservices.
- Treating vendor labels as canonical object names.
- Bypassing marketplace DealSet settlement for commercial obligations.
- Owning email send execution (delegated to mail).
- Owning SMS / push / in-app delivery (delegated to messenger).
- Owning tenant website root (delegated to sites).
- Owning live agent live-chat (delegated to contact-center via chatflow handoff).
- Owning meeting scheduler (delegated to calendar).
- Owning sales-record state (delegated to crm.contact-master + crm.account-master + crm.opportunity + crm.lead).
- Owning Cedar policy engine internals (delegated to policy-engine).
- Owning ontology storage (delegated to ontology).
- Owning workflow runtime internals (delegated to workflow-engine; this µservice owns the workflow-canvas authoring surface but runtime execution is workflow-engine).

## K. Hyperscaler and Industry Precedents

- **HubSpot Marketing Hub** (primary counterpart per ADR-0328 §D-2.18-19): import lesson is unified marketing object model under one workspace; Oyatie improvement is auditable evidence chain via audit-chain seal.
- **Adobe Marketo Engage** (flanking counterpart): import lesson is Engagement Program nurture stream + Smart Campaign trigger + filter + flow steps; Oyatie improvement is deterministic journey replay.
- **Mailchimp** (flanking counterpart): import lesson is SMB-friendly Customer Journey Builder with multiple starting points; Oyatie improvement is per-pack disclosure block and HLC stamping.
- **AWS Pinpoint** (substrate precedent): import lesson is cellular campaign orchestration at hyperscaler scale; Oyatie improvement is per-tenant home cell with shuffle sharding per ADR-0248.
- **Google Cloud Marketing Platform** (substrate precedent): import lesson is per-tenant data-class boundaries; Oyatie improvement is Cedar-evaluated default-deny on every mutation.
- **Salesforce Marketing Cloud Engagement** (reference): import lesson is Journey Builder visual primitives; Oyatie improvement is workflow-canvas immutable snapshots + deterministic replay.

## L. Pack Overlay Applicability

The default overlay roster is SOC-2 + ISO-27001 + GDPR + LGPD + KR-PIPA + CPRA + CAN-SPAM + CASL + HIPAA + ePrivacy-Directive + TCPA + EU-AI-Act-Marketing-Personalization. Each pack must declare permit deltas, data-class deltas, retention deltas, export deltas, regulator-evidence deltas, and UI-disclosure deltas via the pack-overlay contract.

## M. Follow-Up Buildout

- **IP-031..IP-055** (25 new slices) land in this remediation at the IP-026 substance bar covering Email, Landing Page, Form, Workflow visual canvas, Lead Scoring, ABM, Lifecycle Stage, Subscription Type, A/B Test, Send-Time Optimization, Email Tracking, Webhook, Marketing Calendar, Behavioral Profile, Marketing Asset, Static List, Chatflow, Ad Network seam, Social seam, SEO seam, CMS seam, Customer-facing Analytics, Survey, Postcard, Mobile SDK.
- **Per-microservice ADRs**: ADR-MS-MA-001 (engagement mutation envelope), ADR-MS-MA-002 (landing-page vs sites), ADR-MS-MA-003 (form vs forms microservice).
- **Migration playbooks**: from-hubspot-marketing-hub.md (primary), from-marketo.md (flanking), from-mailchimp.md (flanking).
- **OpenTofu modules**: per-deployment-context modules under `iac/<context>/` including OCI Always Free for demo_trial.
- **Cedar policies**: tenant_class gates on all bounded-context mutations; per-pack overlay-aware policies.
- **Per-tenant_class SLO overlay**: demo_trial best-effort vs paid contractual SLO under `slos/`.

This remediation closes 96 P0 BIG-8 findings from `coherence-audit-2026-05-20.md` and 25 tenant-class adoption gaps (C-001..C-011) plus 11 of 25 open boundary questions (Q-001, Q-003, Q-004, Q-018, Q-019, Q-020, Q-022, Q-023, Q-024, Q-025).
