# Marketing Automation

Service: marketing-automation
Date: 2026-05-21
Doc class: README
Phase: 4A.5 Big-8 customer-engagement substrate
Primary counterpart: HubSpot Marketing Hub
Flanking counterparts: Adobe Marketo Engage, Mailchimp
Binding authorities: ADR-0105 (13-layer enum), ADR-0131 (per-microservice flat layout), ADR-0244 (tenant-class primitive), ADR-0245 (substrate vs product), ADR-0248 (cellular shape), ADR-0251 (compliance pack primitive), ADR-0253-amendment (HTTP/3 + QUIC + ECH + PQC), ADR-0263 (observability emission), ADR-0314 (marketplace DealSet), ADR-0321 (per-microservice doc-set anchor), ADR-0328 (substance bar + Big-8 sequence), ADR-0331 (tenant-class adoption surfaces — in flight).

## 1. What this microservice owns

Marketing Automation is the tenant-owned customer-engagement spine that owns 25 bounded contexts and 5 differentiator slices across HubSpot Marketing Hub, Adobe Marketo Engage, and Mailchimp coverage. Twenty-five contexts span segmentation, journey orchestration, lifecycle progression, suppression, attribution, deliverability, frequency capping, marketing assets, lead scoring, ABM, A/B testing, send-time optimization, social, SEO, CMS overlap, customer-facing analytics, behavioral profiling, chatflow seam, ad-network seam, calendar seam, webhook subscriptions, marketing email composition, landing page, form, and workflow visual canvas. Five differentiator slices (IP-026 real-time segment materializer; IP-027 consent-suppression ledger; IP-028 multi-touch attribution reconciler; IP-029 deliverability warmup governor; IP-030 cross-channel frequency cap) carry hyperscaler-grade substance that exceeds counterpart depth on auditable evidence.

The microservice is the customer-engagement on-ramp for revenue, retention, and pipeline-attribution motion. It does not own tenant identity (delegated to iam), Cedar policy engine internals (delegated to policy-engine), workflow runtime internals (delegated to workflow-engine), ontology storage (delegated to ontology), payments rails (delegated to commerce + cloud-billing), marketplace settlement state (delegated to marketplace), or sales-record state (delegated to crm). It also does not recreate the Salesforce Marketing Cloud or HubSpot Hub-suite boundaries — products and product labels remain UX/marketing concerns, not microservice boundaries.

## 2. HubSpot Marketing Hub anchor

HubSpot Marketing Hub is the primary counterpart anchor per ADR-0328 §D-2.18-19. The bounded-context set in §3 is rendered against HubSpot's canonical object model: Marketing Email, Workflow, Active List, Static List, Form, Landing Page, CTA, Campaign, Marketing Event, Custom Behavioral Event, Smart Content, Sequence, Snippet, Template, Subscription Type, Lead Scoring, Marketing Calendar, A/B Test, Send Time Optimization, Email Tracking, Webhook Subscription, Marketing Analytics, Behavioral Event, Lifecycle Stage, Communication Preference, Email Health.

HubSpot parity coverage as of this README: Email primitive (B-005) + Landing Page (B-006) + Form (B-007) + Workflow visual canvas (B-008) + Lifecycle Stage (B-011) + A/B Test (B-013) + Send-Time Optimization (B-014) + Lead Scoring (B-015) + ABM (B-016) + Email Tracking (B-021) + Webhook (B-022) + Marketing Calendar (B-023) + Behavioral Profile (B-024) + Chatflow seam (B-025) are now scoped as bounded contexts with capability YAMLs in `capabilities/`. IP-031..IP-055 deliver depth at the IP-026 substance bar.

## 3. Bounded contexts

The microservice owns 25 bounded contexts:

1. **segment** (dynamic membership) — predicate-based, real-time materialization; counterpart HubSpot Active List + Marketo Smart List + Mailchimp Segment.
2. **static-list** — explicit-membership list; counterpart HubSpot Static List + Marketo Static List + Mailchimp Tag.
3. **campaign** — engagement-side wrapper grouping email/landing/form/social assets under one tracked initiative; counterpart HubSpot Marketing Campaign + Marketo Program + Mailchimp Campaign.
4. **journey** (runtime) — execution of triggered multi-step engagement; counterpart HubSpot Workflow runtime + Marketo Smart Campaign + Mailchimp Customer Journey runtime.
5. **workflow-canvas** (authoring) — drag-and-drop visual builder; counterpart HubSpot Workflow Editor + Marketo Smart Campaign Designer + Mailchimp Customer Journey Builder.
6. **email** (composition) — marketing email object with subject + content + dynamic tokens + smart content + A/B variant; counterpart HubSpot Marketing Email + Marketo Email + Mailchimp Regular Campaign.
7. **landing-page** — hosted page with form attachment, conversion goal, A/B variant, SEO metadata; counterpart HubSpot Landing Page + Marketo Landing Page + Mailchimp Landing Page.
8. **form** — field-config, validation, GDPR consent, progressive profiling; counterpart HubSpot Form + Marketo Form + Mailchimp Signup Form.
9. **lead-scoring** — score formula + decay + behavioral triggers; counterpart HubSpot Lead Scoring + Marketo Lead Scoring + Mailchimp Premium Predicted Demographics.
10. **lifecycle-stage** — Subscriber → Lead → MQL → SQL → Opportunity → Customer progression; counterpart HubSpot Lifecycle Stage + Marketo Engagement Score Buckets + Mailchimp CLV bands.
11. **subscription-type** — publication-channel category (newsletter / product update / event invite); counterpart HubSpot Subscription Type + Marketo Communication Limits + Mailchimp Groups.
12. **consent-audience** — lawful-basis suppression and consent audit per channel × purpose; counterpart HubSpot Subscription Preferences + Marketo Unsubscribe Tracking + Mailchimp Unsubscribed.
13. **attribution** — multi-touch credit allocation linking touches to revenue events; counterpart HubSpot Campaign Attribution + Marketo Revenue Cycle Modeler + Mailchimp Premium Revenue Reports.
14. **deliverability** — DKIM/SPF/DMARC monitoring + warmup throttling; counterpart HubSpot Email Health + Marketo Deliverability Program + Mailchimp Premium Delivery Optimization.
15. **frequency-cap** — cross-channel touch cap per subject × purpose × channel; counterpart HubSpot Frequency Safeguard + Marketo Communication Limit + Mailchimp Premium Contact Rating.
16. **abm** (account-based marketing) — target accounts, account scoring, account-level workflows; counterpart HubSpot ABM + Marketo Account-Based Marketing.
17. **a-b-test** — variant test framework for email + landing page + workflow; counterpart HubSpot A/B Test + Marketo A/B Test + Mailchimp A/B Test.
18. **send-time-optimization** — per-recipient optimal-send-time prediction; counterpart HubSpot STO + Marketo Optimal Send Time + Mailchimp Send Time Optimization.
19. **email-tracking** — open + click + reply telemetry distinct from attribution credit; counterpart HubSpot Email Tracking + Marketo Email Insights + Mailchimp Click/Open Reports.
20. **webhook-subscription** — outbound HTTP/3 delivery + retry + signing for tenant-side integrations; counterpart HubSpot Webhooks + Marketo Webhooks + Mailchimp Webhooks.
21. **marketing-calendar** — visualization of scheduled engagement across channels; counterpart HubSpot Marketing Calendar + Marketo Calendar + Mailchimp Content Calendar.
22. **behavioral-profile** — per-contact behavior aggregation distinct from segment predicate state; counterpart HubSpot Behavioral Event + Marketo Activity Log + Mailchimp Audience Insights.
23. **marketing-asset** — templates + files + design blocks + snippets; counterpart HubSpot Design Manager + Marketo Design Studio + Mailchimp Content Studio.
24. **customer-analytics** — tenant-visible marketing report surface distinct from operator dashboards; counterpart HubSpot Marketing Analytics + Marketo Performance Insights + Mailchimp Reports.
25. **chatflow** — bot + live-chat seam to messenger / contact-center; counterpart HubSpot Chatflows + Conversational Bots.

The boundary between marketing-automation.campaign and crm.campaign is settled by the per-microservice ADR-MS-MA-001 (engagement-side ownership here; revenue-side ownership in crm). The boundary between marketing-automation.email composition and mail send execution is settled by IP-006 contract (compose here; deliver via mail substrate). The boundary between marketing-automation.landing-page and sites is settled by ADR-MS-MA-002 (marketing-attached pages here; tenant website root in sites).

## 4. Tenant-class behavior (ADR-0244)

Every request carries a tenant principal claim that the gateway resolves into a `tenant_class ∈ {demo_trial, paid}`. The microservice never trusts a client-supplied tenant_class — it reads the gateway-stamped principal claim. Cedar policies under `policy/` and `policies/` gate operations by tenant_class.

**Demo-trial caps** (registered in `manifest.json` `demo_trial_caps`):

- contacts: 500
- monthly_email_sends: 5,000
- active_journeys: 2
- active_segments: 5
- attribution_models: 1
- deliverability_warmups: 1
- frequency_windows: 3
- landing_pages: 3
- forms: 5
- a_b_tests: 1
- custom_properties: 50

Demo-trial tenants get best-effort SLO (no contractual guarantee) and OCI Always Free deployment context as the default substrate. Conversion to paid removes caps; paid SLO targets are listed under `slos/`.

**Paid billing components** are composable: `revenue_share` applies when marketing-automation drives marketplace DealSet settlement per ADR-0314; `per_seat` applies to licensed marketing-ops principals; `per_usage` meters are tracked per (event_class, tenant_id) tuple — email_sends, attribution_runs, segment_materializations, journey_executions, form_submissions, webhook_deliveries, deliverability_admit_decisions, frequency_reservations.

## 5. Deployment contexts (ADR-0244 multi-context)

The microservice runs in six deployment contexts:

- **oyatie-public-cloud** — Oyatie-managed multi-tenant SaaS; default home for paid tenants.
- **aws-guest** — single-tenant guest install on customer AWS account; per-tenant K8s + RDS + S3.
- **oci-guest** — single-tenant guest install on customer OCI account; demo-trial tenants land on Always Free profile (4 OCPU + 24 GB RAM + 200 GB block + 2× Autonomous DB).
- **on-prem** — customer-controlled bare-metal or VM; Talos + Cilium + Cloud Hypervisor + Kata Containers.
- **colo** — customer-controlled colocation with Oyatie-supplied control plane.
- **oyatie-as-cloud-provider** — Oyatie running as IaaS; cloud-* microservices supply substrate.

Per-context OpenTofu modules live under `iac/<context>/` and ship signed (sigstore). Tenant onboarding for any context is a single `tofu apply -var tenant_id=<id> -var tenant_class=<class> -var deployment_context=<ctx> -var pack_overlays=<list>`.

## 6. Transport, cryptography, identity

HTTP/3 + QUIC is the default transport (ADR-0253-amendment). TLS 1.3 is the floor with ECH (Encrypted Client Hello) where terminated and PQC hybrid (X25519MLKEM768 + ML-KEM-768) where negotiated. gRPC runs over HTTP/3 internally. Workload identity uses SPIFFE/SPIRE; provider credentials never live in service config — they resolve at request time from OpenBao with ≤60s TTL leases under `${openbao:secret/<tenant_id>/marketing-automation/<credential>}`.

## 7. Compliance packs

Tenant pack overlays attach at request time via the gateway. The microservice honors: SOC-2, ISO-27001, GDPR, LGPD, KR-PIPA, CPRA, CAN-SPAM, CASL, HIPAA. Higher-restriction-wins applies to residency, retention, breach timing, regulator export, and appeal/notice rules. HIPAA tenants are gated behind explicit operational review because marketing communication that touches PHI is a sharp surface — Q-020 of the Wave-4 audit captured the governance question.

## 8. Rust-strict source layout

The microservice ships as a workspace member under `src/`. Cargo strict-lint posture forbids `unwrap_used`, `expect_used`, `panic`, and `unsafe_code`. Layer modules align to ADR-0105: `api/`, `rest/`, `application/`, `usecase/`, `domain/`, `kernel/`, `adapter/`, `worker/`, `governance/`. The `cli/`, `sdk/`, and `test/` layers are present as appropriate. The integration test surface under `tests/` follows one fixture per IP slice.

## 9. Differentiator capabilities

IP-026..IP-030 carry depth that the counterparts treat as platform-only:

- **IP-026 Real-time segment materializer** — sub-second buying-committee segment freshness; exceeds Marketo Smart List default refresh cadence.
- **IP-027 Consent-suppression ledger** — append-only HLC-stamped ledger with per-channel × per-purpose granularity; auditable to GDPR Article 15 (right of access) + Article 17 (right of erasure) + Article 21 (right to object) without vendor cooperation.
- **IP-028 Multi-touch attribution reconciler** — auditable evidence chain linking touches to revenue events; exceeds HubSpot Campaign Attribution because revenue events come from crm with cryptographic seal not vendor heuristic.
- **IP-029 Deliverability warmup governor** — DMARC-failure fail-closed posture; tenant admin override requires Cedar-evaluated step-up.
- **IP-030 Cross-channel frequency cap** — single subject × purpose × channel window across email + SMS + push + in-app; counterparts cap per-campaign or per-list, allowing cross-product fatigue.

## 10. Companion documents

- `manifest.json` — machine-readable spec; bounded contexts, deployment contexts, demo-trial caps, supported OSes, ADR bindings.
- `PRD.md` — bespoke per-context user stories + per-verb-per-aggregate functional requirements + tenant-class conversion stories.
- `ARCHITECTURE.md` — bounded-context architecture, layer map, integration topology, failure modes, per-anchor depth.
- `compliance.md` — per-pack regulatory mapping.
- `dpia.md` — DPIA per GDPR Article 35.
- `threat-model.md` — STRIDE threat model.
- `multi-region.md` — multi-region substrate per deployment context.
- `incident-response.md` — incident playbook.
- `failure-modes.md` — failure-mode register.
- `capacity-model.md` — tenant-class × deployment-context capacity grid.
- `cost-budget.md` — tenant-class × deployment-context cost grid.
- `backfill-replay.md` — backfill and replay strategy.
- `sdk-plan.md` — generated SDK manifest.
- `competitor-parity-matrix.md` — bespoke per-capability matrix with HubSpot/Marketo/Mailchimp coverage.
- `feature-parity-matrix-2026-05-20.md` — companion UNION-coverage matrix landed by the Wave-4 audit.
- `performance-benchmark-numbers-2026-05-20.md` — bespoke performance numbers.
- `coherence-audit-2026-05-20.md` — Wave-4 audit findings consumed by this remediation pass.
- `REMEDIATION-NOTES-2026-05-21.md` — this remediation's change log.
- `decisions/ADR-MS-MA-001-engagement-mutation-envelope.md` — per-microservice ADR for the engagement mutation envelope.
- `migration-playbooks/` — per-counterpart migration playbooks (from-hubspot-marketing-hub.md + from-marketo.md + from-mailchimp.md).
- `IP-001..IP-055` — implementation plan slices.
- `runbooks/` — 20 operational runbooks.
- `slos/` — 12 OpenSLO 1.0 SLO definitions.
- `dashboards/` — 10 Grafana dashboards.
- `policy/` + `policies/` — Cedar default-deny gates.
- `contracts/` — OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 surfaces.
- `iac/<context>/` — OpenTofu modules per deployment context.

## 11. Open questions consumed from Wave-4 audit

Q-001..Q-025 in the audit's §5 enumerate the open boundary decisions. ADR-MS-MA-001 settles Q-001 (Campaign ownership), ADR-MS-MA-002 settles Q-003 (Landing Page ownership), ADR-MS-MA-003 settles Q-004 (Form ownership). Cross-microservice questions Q-009..Q-014 are tracked in the engagement-fabric inter-microservice contract ADRs filed under `docs/decisions/`.

## 12. Status

Wave-15A Big-8 remediation as of 2026-05-21:

- Tier-retirement: 24 distinct call-sites scrubbed + 25 IP frontmatter scrubs (T-001..T-026). Cell-tier eligible_tiers preserved per ADR-0248.
- Tenant-class adoption: 11 surfaces implemented (C-001..C-011) — manifest tenant_class principal claim, PRD conversion stories, Cedar tenant_class gates, OpenAPI extension doc, audit-event tenant_class dimension, per-class SLO overlays, per_usage meter binding, capacity/cost tenant_class × context grid, migration playbook conversion path, demo_trial_caps registry.
- Big-8 family completeness: 20 new bounded-context capability YAMLs added (B-005..B-025); HubSpot Marketing Hub promoted to primary counterpart across README + manifest + PRD + parity matrix + ARCHITECTURE.
- IP-031..IP-055 (25 new slices) land at the IP-026 substance bar.
- ARCHITECTURE.md §F rewritten as 5 bespoke aggregate traces (replacing 210 stamped expansion bullets).
- PRD.md §C+§D rewritten as bespoke per-aggregate stories + per-verb-per-aggregate functional requirements.
- competitor-parity-matrix.md rewritten with bespoke per-capability comparison.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
