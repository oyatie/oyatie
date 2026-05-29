---
doc_class: ProductRequirementsDocument
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: axis-forms + council-product
deciders: council-product, council-architecture, axis-forms, ops-security, council-privacy, council-legal-compliance, council-design-system
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145), ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/microservices/forms.json]
competitor_benchmark:
  - Google Forms (Google Workspace)
  - Microsoft Forms (Microsoft 365)
  - Typeform
  - Jotform
  - Tally
  - Airtable Forms
  - SurveyMonkey
  - Wufoo
  - Formstack
  - Survicate
  - Qualtrics (XM Platform)
  - HubSpot Forms
  - Mailchimp Forms (Mailchimp / Intuit)
  - Hotjar Surveys (Contentsquare)
doc_status: published
---

# Forms — Product Requirements Document

## 1. Vision

Forms is the canonical oyatie µservice for **typed form definitions, response capture, and survey distribution**. It is the entry point for the `Form → Response → downstream action` lifecycle across every pack, every channel, and every workload class — from one-question polls to GDPR Art. 9 special-category consent capture to HIPAA-eligible patient intake.

Forms is **net-new** per ADR-0135 — no `oya-forms-*` crates exist. Forms ships flat (per ADR-0131), single-concern (per ADR-0132), SLO-gated (per ADR-0139).

Forms is the **Workspace-tier sibling** to `sheets` (responses bridge → sheets), `drive` (file uploads bridge → drive), `mail` (bulk distribution → mail), `messenger` (link share → messenger), and `workflow-engine` (form-submission triggers a workflow). Forms NEVER calls a foreign µservice directly; all inter-product flows are mediated by the Workflow + Ontology adapter pattern.

Forms competes head-to-head with the 14 industry leaders listed above and MUST match or exceed each on response fidelity, accessibility (WCAG 2.2 AA), regulatory pack coverage, and AI-form-build quality.

## 2. Functional Requirements

| FR | Description | AC | Tier |
|---|---|---|---|
| FR-01 | Tenant operator authors a typed Form (`form.v1`) via a Leptos-WASM form-builder; field types include text, number, date, datetime, single-choice, multi-choice, scale (Likert), grid (matrix), file-upload, signature, payment, and conditional. | AC-01, AC-02 | Must-GA |
| FR-02 | Form sections + page-flow with declarative branching (conditional-logic engine per ADR-FORMS-0004). | AC-03 | Must-GA |
| FR-03 | Per-field validation (JSON Schema-derived) AND cross-field validation (declarative DAG). | AC-04 | Must-GA |
| FR-04 | Form definition versioning via the ChangeSet state machine (ADR-0110). Form schema evolution is non-breaking by default; breaking changes require a new major version + sunset window. | AC-05 | Must-GA |
| FR-05 | Response capture: anonymous, OIDC-authenticated, or pre-filled-link (HMAC-signed) submission. Responses sealed via audit-chain (ADR-0028) and versioned to the Form major version they target. | AC-06, AC-07 | Must-GA |
| FR-06 | PII-aware response store: per-field `data_class` declared at authoring time; storage applies per-tenant DEK column-level envelope encryption (ADR-FORMS-0003); pack-routed (per `data-residency.md`). | AC-08, AC-09 | Must-GA |
| FR-07 | Form share: anonymous link, authenticated link, pre-filled link (HMAC + TTL), embed (iframe + JS widget + REST). CSP-strict; per `policy/embed-csp.md`. | AC-10 | Must-GA |
| FR-08 | Submission rate-limit + captcha selection (hCaptcha / Cloudflare Turnstile / Friendly Captcha) per ADR-FORMS-0002. Privacy-preserving by default; reCAPTCHA explicitly forbidden in pack-eu, pack-kr, pack-us-healthcare. | AC-11 | Must-GA |
| FR-09 | E-signature per eIDAS 910/2014 (XAdES / PAdES / CAdES) with per-tier signature class (AES vs QES) per ADR-FORMS-0006. | AC-12 | Should-GA |
| FR-10 | Payment field: cross-µservice bridge to `fintech` (Tier-G); Forms NEVER stores PCI-scope data. | AC-13 | Should-GA |
| FR-11 | File-upload field: cross-µservice bridge to `drive` (Tier-D); ClamAV/OPSWAT scan inline; size + MIME-type allowlist per pack. | AC-14 | Must-GA |
| FR-12 | Conditional-logic engine: declarative DAG of branch predicates + skip-logic + show-if/hide-if; evaluated server-side at submit and client-side for UX. | AC-15 | Must-GA |
| FR-13 | Form export to PDF + DOCX (server-rendered, accessibility-preserving). Response export to CSV + XLSX + JSON + Google-Sheets-bridge (via the `sheets` µservice). | AC-16 | Must-GA |
| FR-14 | Analytics: response count, drop-off funnel, average completion time, conversion rate, per-field abandonment, A/B variant performance. | AC-17 | Must-GA |
| FR-15 | A/B testing: per-form variant set with deterministic submitter-hash routing; statistical-significance gating before declaring a winner. | AC-18 | Should-GA |
| FR-16 | Multi-language (i18n) per field; pack-defaults for locale; submitter sees pack-pinned locale unless explicit override; right-to-left scripts supported (Arabic in pack-ae / pack-ksa). | AC-19 | Must-GA |
| FR-17 | WCAG 2.2 AA accessibility: every form passes axe-core + manual screen-reader + keyboard-only + reduced-motion tests before publish. CI gate `oya-governance-wcag22-conformance` blocks publish on any AA failure. | AC-20 | Must-GA |
| FR-18 | Webhooks on submission: per-form webhook target with mTLS + HMAC-SHA-256 signature; retries with exponential backoff; dead-letter queue. | AC-21 | Must-GA |
| FR-19 | Workflow-trigger: form-submission triggers a workflow in `workflow-engine` via the Workflow + Ontology adapter. Trigger is fail-closed (workflow start failure → audit row, no silent drop). | AC-22 | Must-GA |
| FR-20 | Bulk distribution to ≤ 10k recipients per send (cross-µservice to `mail` + `messenger` + SMS-via-Tier-G). Per-recipient pre-filled link generated; unsubscribe honoured per pack regulation. | AC-23 | Should-GA |
| FR-21 | Templates + template-marketplace: per-pack-signed template bundles (signing-key per pack per `data-residency.md`); template-quarantine on signature-drift. | AC-24 | Should-GA |
| FR-22 | AI-form-build (T2 capability): tenant prose → candidate form definition; bounded by ADR-FORMS-0005 (EU AI Act 2024/1689 high-risk if used for employment / credit / insurance screening per Annex III §4). Tenant explicitly reviews + accepts; ChangeSet reversibility. | AC-25 | Should-GA |
| FR-23 | HIPAA-compliant mode for pack-us-healthcare: PHI fields declarable; BAA-required tenant; PHI never traverses pack boundary; HIPAA §164.308/310/312/316 controls applied. | AC-26 | Should-GA |
| FR-24 | Submitter-side DSR: data-subject access + rectification + erasure honoured via `oya-dsr-cascade-runner`; per-pack SLA (KR 30d / BR 15d / EU 30d). | AC-27 | Must-GA |
| FR-25 | Tenant-scoped Cedar policy default-deny across read/list/save/delete/publish/respond/export/distribute (ADR-0140). | AC-28 | Must-GA |

## 3. Acceptance Criteria

| AC | Test | Lane |
|---|---|---|
| AC-01 | Author a 50-field form via builder; persist; reload; byte-identical. | `oya-governance-round-trip-byte-equality` |
| AC-02 | RFC 8785-canonicalized form.v1 spec; `dsl-loader(emit(x)) == x`. | `oya-governance-canonical-form-byte-equality` |
| AC-03 | Conditional-logic DAG resolves identically server + client over 1000-case corpus. | `oya-forms-conditional-logic-parity` |
| AC-04 | Cross-field validation: e.g., `end_date >= start_date`; fails respond with precise per-field diagnostic. | `oya-forms-cross-field-validation` |
| AC-05 | Form major-version bump: old responses still load against old version's schema; no silent migration. | `oya-forms-version-isolation` |
| AC-06 | 1000-submission corpus; every response Ed25519-sealed; chain reconstructable. | `oya-governance-audit-chain-coverage` |
| AC-07 | Pre-filled-link HMAC + TTL: tampered link rejected with 401; expired link rejected with 410. | `oya-forms-prefill-link-integrity` |
| AC-08 | Per-tenant DEK envelope encryption: every PII column encrypted at rest; DEK rotation tested quarterly. | `oya-forms-pii-column-encryption-correctness` |
| AC-09 | Pack-routing: pack-eu tenant's responses never land in pack-us cluster; verified via residency probe. | `oya-governance-pack-routing-conformance` |
| AC-10 | Embed iframe: CSP `frame-ancestors` per tenant allow-list; non-listed parent gets blank iframe + parent console error. | `oya-forms-embed-csp-conformance` |
| AC-11 | Captcha: hCaptcha challenge served at 100% of anonymous submits; bypass token rejected; reCAPTCHA never loaded in pack-eu/pack-kr/pack-us-healthcare. | `oya-forms-captcha-conformance` |
| AC-12 | E-signature: PAdES-LTA generated for tenant-tier-G+ signatures; signature verifiable against pack-trusted CA. | `oya-forms-esignature-conformance` |
| AC-13 | Payment field: Stripe / Toss Payments client-side tokenisation; no PAN traverses Forms; PCI-DSS v4 scope-reduction proven. | `oya-forms-payment-scope-reduction` |
| AC-14 | File upload: 100MB max per file; 1GB total per form; ClamAV scan inline; infected file rejected with precise diagnostic. | `oya-forms-upload-scan-conformance` |
| AC-15 | Skip-logic: hidden-by-condition field has no `data_class=PII_*` value persisted. | `oya-forms-skip-logic-pii-correctness` |
| AC-16 | CSV export 100k responses ≤ 5s (p95); columns ordered per form spec; PII columns redacted unless explicit unredact entitlement. | `oya-forms-export-latency` + `oya-forms-export-pii-redaction` |
| AC-17 | Drop-off funnel: every drop-off recorded; histograms accurate over 10k-submission corpus. | `oya-forms-analytics-fidelity` |
| AC-18 | A/B variant: deterministic submitter-hash routing; ≥ 95% statistical-significance gate before declaring winner. | `oya-forms-ab-statistical-significance` |
| AC-19 | i18n: 14 locales (EN, KO, JA, ZH-CN, ZH-TW, AR, DE, FR, ES, PT-BR, HI, ID, MS, TH); RTL renders identically. | `oya-forms-i18n-rtl-conformance` |
| AC-20 | axe-core 0 violations + screen-reader nav order matches DOM order + keyboard-only completion possible + reduced-motion respects `prefers-reduced-motion`. | `oya-governance-wcag22-conformance` |
| AC-21 | Webhook: 1000-submission corpus; each delivered ≤ 5s p95 with mTLS + HMAC; failures retried with exponential backoff to dead-letter. | `oya-forms-webhook-delivery` |
| AC-22 | Workflow-trigger: 1000-form-submissions / 1000-workflow-starts; fail-closed verified by induced workflow-engine 500. | `oya-forms-workflow-trigger-fail-closed` |
| AC-23 | Bulk distribute: 10k-recipient corpus; per-recipient HMAC link; unsubscribe ≥ 99.9% honoured; CAN-SPAM / GDPR / KISA conformance. | `oya-forms-bulk-distribute-conformance` |
| AC-24 | Template-marketplace: signature-drift bundle quarantined; tenant can't install. | `oya-forms-template-signature-conformance` |
| AC-25 | AI-form-build: 1000-prompt corpus; schema-valid output rate ≥ 80%; tenant-accept rate ≥ 50%; high-risk classification triggers DPIA prompt. | `oya-forms-ai-build-quality` |
| AC-26 | HIPAA mode: PHI field tagged; never traverses pack; BAA-required tenant; audit-chain seal on every PHI read. | `oya-forms-hipaa-conformance` |
| AC-27 | DSR: submitter request → form identification → erasure within pack-SLA. | `oya-governance-dsr-cascade-conformance` |
| AC-28 | Cedar default-deny: 50-case adversarial corpus; every non-permit attempt returns deny + audit row. | `oya-governance-cedar-default-deny-conformance` |

## 4. Non-Functional Requirements

### Performance Requirements

| Metric | Budget (GA) | Notes |
|---|---|---|
| Form render (TTI, p95) | ≤ 200ms | from CDN edge to first interactive field |
| Field validate (p99) | ≤ 50ms | client-side fast path + server-side authoritative |
| Submission (p95) | ≤ 150ms | server round-trip including Cedar + audit-chain seal |
| Analytics dashboard render (p95) | ≤ 500ms | cached aggregate; <60s freshness |
| Bulk distribute 10k recipients (p95) | ≤ 30s | from start-distribute to last-recipient enqueued |
| Export CSV (100k responses, p95) | ≤ 5s | streaming; no full materialisation |
| Export XLSX (100k responses, p95) | ≤ 10s | streaming via openpyxl-equivalent |
| AI-form-build (p95) | ≤ 8s | T2 invocation including PII redactor + LLM + schema validate |
| File upload scan (100MB file, p95) | ≤ 5s | ClamAV streaming |

### Horizontal Scalability

- Form-rest: stateless; HPA 4–80 replicas per region; per-tenant rate-limit at L7.
- Response-collector-rest: stateless; HPA 4–80 replicas per region; sticky to Citus shard via tenant_id.
- Response-store (Postgres + Citus 12.x): tenant_id shard key; 32 shards baseline; replication-factor 2.
- Response-cache (Valkey 8.1 (RESP3 wire-compatible)): per-cell HA; ephemeral; regenerable.
- Form-builder-wasm: served via CDN; tenant-agnostic; cached at edge.
- Bulk-distribute-worker: async; back-pressured queue (Kafka); ≤ 1k recipients/sec per pack.
- Export-worker: async; streaming to object storage; 100k-response export ≤ 5s.

### DR posture (ADR-0343)

- RTO/RPO target: manifest-declared RTO p99 900s and RPO p99 60s for response-store and published form definitions, staying stricter than the HIPAA-2024 floor of RTO 3600s/RPO 300s, SOC2-T2 floor of 14400s/900s, and KR-PIPA floor of 14400s/900s. Effective floor driver: HIPAA-2024 for PHI intake.
- Failover reference: manifest `failover_runbook` is `runbooks/dr-failover.md`; supporting corruption drills remain `runbooks/response-store-corruption.md` and export backlog recovery remains `runbooks/export-pipeline-failure.md`.
- Multi-region active-active posture: true per manifest; replication shape is `active-active-multi-az-cross-region-warm` across `postgres_wal_g`, versioned object storage, Valkey, and audit-chain Merkle seals.
- Tenant-visible behavior: submitted forms either commit once with an audit-chain seal or fail closed, so patient intake, e-signature, and paid form-submission workflows do not silently lose responses during a regional event.

### Capacity model (ADR-0340)

- Per-tenant baseline: manifest-declared 0.08 vCPU, 256 MiB RAM, 5 GB storage, two Postgres connections, two Valkey connections, and five outbound HTTP connections reserved before burst credits.
- Scaling dimension: `per_request` for render/submit, `per_form_response` for storage, and `per_export_job` for CSV/XLSX/PDF/DOCX workloads.
- Cell placement class: Tier-3 per manifest because Forms is a tenant-facing Workspace app with request/submission scaling, while PHI/PII controls are enforced by pack admission and audit-chain seals.
- Autoscaling boundaries: form-rest and response-collector HPA 4-80 replicas per region; ClamAV 4-20; export, bulk-distribute, webhook, and AI-form-build workers 2-20, with per-tenant L7 rate limits and Citus tenant_id shard placement.
- Tenant load profile: supports 50k form-render RPS, 5k sustained submissions, 20k peak submissions, and 100k-response exports without letting one survey tenant starve healthcare intake tenants.

### Sustainability and cost attribution (ADR-0344)

- Per-call emission claim: every `FormResponseSubmitted`, export, webhook, bulk-distribute, and AI-form-build audit row carries `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, capability, provider, cell, and compliance_pack axes.
- Carbon-aware provider routing: yes for AI-form-build, exports, and bulk distribution when the pack and latency budget allow; no for HIPAA emergency intake, payment field tokenization, and submit/PHI paths that must stay in the pinned region.
- Tenant transparency surface: finops-portal exposes per-form submission, export, webhook, and AI-build line items, with pack and region attribution for customer chargeback.
- Regulatory driver: the same audit-side dimensions satisfy CSRD ESRS E1, SB-253, and SEC climate disclosure rollups without reconstructing emissions after the fact.

### API versioning posture (ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet across the version header, URL prefix, and proto3 field for REST, embed widget, webhook, and form-definition contracts.
- SDK semver model: generated SDKs use `major.minor.patch`; breaking form-definition and response API changes require a major SDK bump and a new public date version.
- Support window: last 3 public API versions are supported for at least 180 days.
- Per-tenant pinning: yes for embedded forms, webhooks, and pre-filled-link submitters so a tenant can complete long-running campaigns on the version they published.
- Internal-mesh exemption: yes; direct gRPC within the Forms mesh remains exempt under ADR-0145 while public carriers stay date-versioned.

### Protocols

- **OpenAPI 3.2.0** — main REST surface for form + response APIs.
- **OAuth 2.1 + OIDC** — authenticated forms; submitter identity claim.
- **JSON Schema (draft 2020-12)** — per-field type + validation.
- **OpenSchema** — form-definition portability across vendors (export contract).
- **eIDAS 910/2014** — e-signature classes (SES / AES / QES) with XAdES / PAdES / CAdES profiles.
- **W3C WCAG 2.2 AA** — accessibility conformance.
- **hCaptcha / Cloudflare Turnstile / Friendly Captcha JS SDK** — privacy-preserving spam protection (per ADR-FORMS-0002).
- **AsyncAPI 3.0** — webhook + Kafka submission-event contracts.
- **gRPC + Protocol Buffers** — internal SDK between Forms components.
- **WebDAV (PROPFIND/PUT)** — file-upload bridge to drive.

### Layer-A Substrate

| Component | Pin | Purpose |
|---|---|---|
| Postgres 16 LTS + Citus 12.x | 16.3 / Citus 12.1 | Form-definition + response-store; tenant_id shard key; RLS + column-level envelope encryption |
| Valkey 8.1 (RESP3 wire-compatible) | 7.2.5 | Rate-limit + session + WAF cache |
| Meilisearch 0.10.0 | 0.10.0 | Response search (full-text + facet) |
| Leptos 0.7.x | 0.7.3 | Form-builder + form-renderer WASM |
| ClamAV 1.3 LTS | 1.3.1 | Upload scan (free path) |
| OPSWAT MetaDefender | latest | Upload scan (enterprise path; multi-engine) |
| hCaptcha SDK | 1.x | Spam protection (pack-eu default) |
| Cloudflare Turnstile SDK | 0.x | Spam protection (pack-us default) |
| Friendly Captcha SDK | 1.x | Spam protection (pack-kr fallback) |
| Cedar v4 | 4.2.x | Policy engine (default-deny) |
| OpenBao 2.x | 2.0.x | Secret references |

## 8. Cross-µservice Boundaries (Workflow + Ontology adapter mandatory)

| Direction | Counterpart µservice | Contract | Purpose |
|---|---|---|---|
| Forms → tenancy | tenancy | OAuth 2.1 + Cedar entity | tenant identity + per-form seat entitlement |
| Forms → audit-chain | audit-chain | Ed25519 seal SDK | submission + edit + DSR seal |
| Forms → cell | cell | per-form cell SDK | tenant cell pinning |
| Forms → ontology | ontology | OpenAPI + entity binding | Form/Response/Submitter entities |
| Forms → foundry-runtime | foundry-runtime | T1 field-suggest, T2 AI-form-build, T2 response-clustering | per-tier capability gates |
| Forms → workflow-engine | workflow-engine | submit-event → workflow-start | trigger workflows on submission |
| Forms → sheets | sheets | bridge contract | response-export to live spreadsheet |
| Forms → drive | drive | WebDAV / OpenAPI | file-upload backend |
| Forms → mail | mail | bulk-distribute SDK | email blast |
| Forms → messenger | messenger | link-share SDK | messenger blast |
| Forms → social | social | embed-on-profile | form embedding |
| Forms → workflow-studio | workflow-studio | form-as-input SDK | forms feed workflow-studio nodes |
| Forms → observability | observability | OpenSLO + Prometheus | SLO publishing |
| Forms → foundry-providers | foundry-providers | LLM-assist SDK | AI-form-build LLM routing |

## 9. Workflow Events Produced

- `FormPublished{tenant_id, form_id, form_version, pack, sealed_at, audit_chain_seal}`
- `FormResponseSubmitted{tenant_id, form_id, form_version, response_id, submitter_hash, submitted_at, pack, audit_chain_seal}`
- `FormResponseSubmitFailed{tenant_id, form_id, reason, attempted_at}` (informational SLI)
- `FormAiBuildRequested{tenant_id, prompt_hash, tier, cross_microservice_destinations}`
- `FormAiBuildAccepted{tenant_id, draft_id, accepted_at}`
- `FormAiBuildRejected{tenant_id, draft_id, reason}`
- `FormDsrExecuted{tenant_id, subject_hash, removed_response_ids_count, executed_at}`
- `FormWebhookDelivered{tenant_id, form_id, target, http_status, delivered_at}`
- `FormBulkDistributeCompleted{tenant_id, form_id, recipient_count, succeeded, failed, completed_at}`

## 10. Ontology Writes

- `Form{tenant_id, form_id, form_version, pack, created_at, published_at, schema_hash, ai_build_origin?}`
- `Response{tenant_id, response_id, form_id, form_version, submitter_hash, submitted_at, pii_encrypted_bool, audit_chain_seal}`
- `Submitter{tenant_id, submitter_hash, identifier_class, first_seen, last_seen}`  (joint-controllership-aware)
- `FormTemplate{template_id, marketplace_pack, signature, signed_by_authority}`
- `FormAiBuildDraft{tenant_id, draft_id, prompt_hash, completion_hash, accepted_at, tier, cross_microservice_destinations[]}`

## 11. Audit + Compliance

- 90-day archive of AI-form-build prompts + completions.
- Per-pack retention table per `policy/data-residency.md`.
- DSR cascade per `policy/data-residency.md` §"DSR Cascade".
- DPIA at `dpia.md` (special-category fields trigger GDPR Art. 35 mandatory).
- AI Act conformity assessment at `legal/ai-act-conformity.md` for T2 AI-form-build.

## 12. Open Questions

| # | Question | Status |
|---|---|---|
| 1 | Form-definition canonicalisation: RFC 8785 vs custom DSL — resolved in ADR-FORMS-0001 | **Closed** |
| 2 | Captcha choice — resolved in ADR-FORMS-0002 | **Closed** |
| 3 | PII column-encryption KMS root — resolved in ADR-FORMS-0003 | **Closed** |
| 4 | Conditional-logic engine — resolved in ADR-FORMS-0004 | **Closed** |
| 5 | AI-form-build scope bounds — resolved in ADR-FORMS-0005 | **Closed** |
| 6 | E-signature conformance class per tenant tier — resolved in ADR-FORMS-0006 | **Closed** |
| 7 | Response-search cold-start: do we pre-index all responses or lazy-index on first search? | Open; recommend lazy-with-pre-warm |
| 8 | Multi-page-form auto-save vs final-submit-only: per-pack default? | Open; recommend per-tenant configurable, default final-submit |

## 13. References

- `/specs/microservices/forms.json`.
- ADR-0056 (clean architecture), ADR-0105 (layer enum), ADR-0106 (12-layer), ADR-0135 (microservice naming), ADR-0139 (SLO-gated promotion), ADR-0131 (per-microservice flat layout), ADR-0132 (single-concern microservices), ADR-0133 (compliance review cadence), ADR-0140 (Cedar policy).
- ADR-FORMS-0001 through ADR-FORMS-0006.
- Regulation (EU) 2016/679 (GDPR) — Arts. 7, 9, 17, 22, 25, 32, 35, 44-50.
- KR PIPA — Arts. 15, 17, 22-2, 23, 24, 28, Enforcement Decree Art. 30.
- HIPAA — 45 CFR §§164.308, 164.310, 164.312, 164.316, 164.530.
- APPI (Japan), PDPA (Singapore), DPDPA 2023 (India), LGPD (Brazil), UAE PDPL, KSA PDPL.
- Regulation (EU) 2024/1689 (AI Act) — Arts. 9-15, 50, 72; Annex III §4.
- Regulation (EU) 910/2014 (eIDAS).
- PCI DSS v4 (PCI Security Standards Council).
- W3C WCAG 2.2 AA (W3C Recommendation, 5 October 2023).
- NIST SSDF SP 800-218.
- SLSA Level 3 framework.
- ISO 27001:2022.
- SOC 2 Type 2 Trust Services Criteria (AICPA).
- OWASP ASVS v4.
- CIS Kubernetes Benchmark.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `forms` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `forms` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_request` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
