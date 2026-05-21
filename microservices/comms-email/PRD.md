---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-comms-email
microservice: comms-email
status: Draft
milestone_first_ship: PHASE-01-COMMS-EMAIL-SUBSTRATE
related_adrs: [ADR-0064, ADR-0144, ADR-0145, ADR-0149, ADR-0166, ADR-0173, ADR-0201, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/compliance-pack-floors.json, /specs/finops-dimensional-model.json]
date: 2026-05-18
last_amended: 2026-05-21
owner_team: oya-substrate-comms
doc_status: draft
---

# PRD — `comms-email` µservice

> Status: Draft v0.1.0
> Authored: 2026-05-18
> Owner: `oya-substrate-comms` (substrate authority)
> ADR anchors: ADR-0201, ADR-0145, ADR-0166, ADR-0173, ADR-0149, ADR-0064.

## 1. Problem statement

Every oyatie µservice that needs to email a human today reaches
for its own SES integration. The result is six bespoke SES
clients, six inconsistent DKIM postures, six different webhook
ingest paths, zero coordinated suppression list, and a vendor
lock-in posture that ADR-0173 forbids. `comms-email` resolves
this by providing the canonical transactional-email substrate
that every µservice consumes through one trait
(`oya-shared-email-comms-kernel`).

## 2. Vision

A single substrate where:

- Every send is DKIM signed, SPF authorized, DMARC enforced —
  at preflight, never silently emitted.
- Every send routes through a pluggable provider adapter (SES,
  Postal, Mailgun, SMTP) with the µservice unaware of which.
- Every webhook delivery event normalizes into ADR-0145 audit
  chain + ADR-0166 schema-registry events on a single shape.
- Every tenant gets per-locale templating, per-domain DKIM
  keys, per-tenant rate ceilings, and a per-tenant suppression
  list.
- Sovereign / air-gapped deployments operate identically to
  cloud-hosted ones — just with the `PostalEmailComms` adapter
  in place of `SesEmailComms`.

## 3. Scope

### In scope (Phase 1 substrate)

- Trait-level integration through `oya-shared-email-comms-kernel`.
- Four real adapters with no Noop fallback: `SesEmailComms`,
  `PostalEmailComms`, `MailgunEmailComms`, `SmtpEmailComms`.
- DKIM key generation + rotation pipeline (per-tenant, OpenBao).
- SPF authorization check at preflight (the µservice publishes
  the canonical SPF record string per tenant pack; OpenTofu DNS
  module emits the record).
- DMARC posture enforcement (`p=quarantine` new tenant,
  `p=reject` post-warm-up; report ingestion).
- MJML compile + Liquid substitution.
- Webhook delivery event ingest + audit-chain emission.
- Per-tenant rate ceilings + suppression list (canonical
  across adapters).
- Per-locale template overlays (ADR-0064 packs).
- Helm chart for Postal (sovereign tier).
- AWS SES configuration only (no chart needed).

### Out of scope (deferred)

- Inbound email (replies / support inbox ingestion). Follow-up
  ADR.
- BIMI logos. Follow-up ADR.
- Push notifications (APNs / FCM / WebPush). Separate µservice
  + ADR.
- In-house Rust-native MTA (`oya-comms-email-server`). ADR-0201
  §"In-house roadmap" Phase 2 — trigger-gated.

## 4. Personas

1. **µservice author** — wants `send_email(to, subject, mjml,
   locale)` and that's it. The substrate handles DKIM, SPF,
   DMARC, suppression, rate ceiling, audit emission.
2. **Tenant administrator** — onboards the tenant's from-domain,
   reviews bounce / complaint dashboards, can request DKIM key
   rotation.
3. **Substrate operator** — monitors deliverability, rotates
   DKIM keys, handles blacklist remediation, swaps providers
   per tenant pack.
4. **Auditor / Compliance** — consumes the audit-chain (ADR-0145)
   stream to prove who sent what to whom when.

## 5. Goals + non-goals

### Goals

- Zero µservice contains a direct provider SDK import.
- DKIM signed delivery rate ≥ 99.99%.
- DMARC alignment rate ≥ 99% for tenants past warm-up.
- p99 enqueue-to-provider-accept latency ≤ 500 ms.
- Webhook event delivery to audit chain p99 ≤ 5 s end-to-end.
- Per-tenant suppression list lookup p99 ≤ 5 ms.

### Non-goals

- Marketing-class campaign management (lists, A/B tests,
  segments). Those belong in a separate µservice if oyatie ever
  ships them.
- HTML editor / WYSIWYG. Tenants supply MJML.

## 6. Success metrics

- Direct provider SDK imports outside `oya-shared-email-comms-kernel`:
  **0** (CI lane enforced).
- DKIM signed send rate: **≥ 99.99%**.
- DMARC alignment past warm-up: **≥ 99%**.
- p99 send latency: **≤ 500 ms**.
- Audit-chain emission lag p99: **≤ 5 s**.
- Sovereign tier deploys using Postal: **100%** of opt-in tenants.

## 7. High-level architecture

```
        ┌─────────────────────────────────────────────────────┐
        │   any µservice (Identity, Tenancy, Workflow, …)     │
        │   ──> oya-shared-email-comms-kernel ::EmailComms    │
        └────────────────────┬────────────────────────────────┘
                             │
        ┌────────────────────┴────────────────────────────────┐
        │              microservices/comms-email              │
        │                                                     │
        │   preflight: DKIM key check, SPF authz, DMARC,      │
        │              suppression, rate ceiling,             │
        │              MJML→HTML, Liquid sub, locale overlay  │
        │                                                     │
        │   send: dispatch to active adapter                  │
        │                                                     │
        │     ┌───────┐ ┌────────┐ ┌─────────┐ ┌──────────┐  │
        │     │  Ses  │ │ Postal │ │ Mailgun │ │   Smtp   │  │
        │     └───────┘ └────────┘ └─────────┘ └──────────┘  │
        │                                                     │
        │   webhook ingest: normalize → audit chain (ADR-0145)│
        └─────────────────────────────────────────────────────┘
```

## 8. Provider matrix (Phase 1)

| Provider | Tier | License | Default for | Lock-in posture |
| -------- | ---- | ------- | ----------- | --------------- |
| SES      | SaaS | AWS-only| AWS clusters| Gated adapter   |
| Postal   | Self | AGPL    | Sovereign   | None (self-host)|
| Mailgun  | SaaS | Comm.   | Customer-opt| Gated adapter   |
| SMTP     | Self | Open    | Fallback    | None            |

## 9. Localization packs

ADR-0064 canonical-base + overlay. Each pack ships a tenant-pack
overlay under `iac/packs/<pack>/`:

- `eu/` — GDPR-aligned defaults (Postal-first or SES-eu-* with
  explicit data-residency).
- `kr/` — KR pack (canonical first pack, ADR-0064 reference).
- `ksa/`, `uae/` — sovereign-tier; Postal-only is forced.
- `us-healthcare/` — HIPAA-aligned BAA-supported providers
  only; SES with BAA OR Postal.

## 10. Compliance posture

See `compliance.md`. Specifically:

- CAN-SPAM (US): unsubscribe footer mandatory; header
  enforcement enforced at preflight.
- GDPR (EU): Art. 6 + Art. 7 — consent-class + legitimate-interest
  thread carried in the audit chain per-send.
- CCPA (CA): opt-out honored via suppression list.
- HIPAA (US-healthcare pack): BAA-only providers; PHI in
  attachments encrypted per ADR-0184 storage tier.

## 11. Non-Functional Requirements

### DR posture (per ADR-0343)

- Manifest target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, `replication_shape=active-active-multi-az-cross-region-warm`.
- Applicable pack floors from `specs/compliance-pack-floors.json`: EU-AI-ACT-2024-HIGH-RISK `1800s/300s` with multi-region required; HIPAA-2024 `3600s/300s` with multi-region required; KR-PIPA-2023 default `14400s/900s`; SOC2-T2 `14400s/900s`; ISO27001-2022 `14400s/3600s`; PCI-DSS-L1-v4 `86400s/3600s`. The effective maximum pack floor is PCI-DSS `86400s/3600s`; comms-email keeps `1800s/300s` because notification delivery is often safety or compliance critical.
- `failover_runbook=runbooks/dr-failover.md`, resolved at `microservices/comms-email/runbooks/dr-failover.md`; backup substrates are `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, and `audit_chain_merkle_seal`.
- `multi_region_active_active=true` for enqueue, provider selection, suppression lookup, and webhook/audit normalization; provider choice remains pack-pinned so sovereign tenants stay on Postal where required.
- Why: tenants and sibling µservices treat accepted transactional email as notice delivery evidence; regional failover must preserve DKIM identity, suppression state, and audit-chain lag rather than simply retrying later.

### Capacity model (per ADR-0340)

- Per-tenant baseline: `0.08 vCPU`, `192 MiB RAM`, `2 GiB storage`, `connections_per_tenant={valkey:2, postgres:2, outbound_http:10}`.
- Scaling dimension: `per_message` for transactional send, DKIM rotation, suppression lookup, webhook replay, provider routing, and Postal/SES failover.
- Cell placement class: `Tier-1` with manifest `pod_runtime_tier=1`; comms-email is a T0 substrate for notices, tenant-domain custody, and signing-key paths while provider adapters fail over independently.
- Autoscaling boundaries: min `3` enqueue/provider-router replicas per tenant-cell, max `48` before tenant/provider queue split; webhook workers scale separately on delivery-event lag and audit-chain p99.
- Why: outbound email mixes low-latency notices with provider-imposed rate limits; this model keeps accepted-send and audit paths stable while throttling provider-specific queues.

### Sustainability + cost attribution (per ADR-0344)

- Every send, template render, suppression lookup, DKIM rotation, provider API call, webhook event, bounce/complaint update, and audit-chain emission emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider routing is carbon-aware for DKIM rotation batches, DMARC report aggregation, bounce analytics, and non-urgent replay; it is not carbon-routed for transactional sends, HIPAA notifications, EU-AI high-risk notices, passwordless/login mail, or provider failover during outage.
- Tenant cost transparency surface: deliverability dashboard shows per-domain send volume, provider spend, webhook/audit lag cost, suppression-list activity, and Postal-vs-SES routing; finops-portal supplies tenant and compliance-pack rollups.
- Why: email provider choice has direct financial and emissions variance, so CSRD, SB-253, and SEC climate-disclosure outputs need per-provider cost/emission rows tied to tenant notice traffic.

### API versioning posture (per ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet using `Oyatie-Version` header, `/v/YYYY-MM-DD/` send/webhook URL prefix, and proto3 field `string oyatie_version = 8001` for public email events/contracts.
- SDK semver model: email SDKs and the shared `EmailComms` trait publish `major.minor.patch`; provider adapter versions do not replace the public date carrier.
- Support window: last `N=3` public versions for at least `180` days after deprecation.
- Per-tenant pinning: yes for regulated tenants, provider migrations, and webhook consumers that must certify payload shape before rollout.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC over HTTP/3 remains tag-compatible and exempt from public carrier routing.

## 12. Rollout

- T+0: ADR-0201 + kernel crate + Helm chart + µservice scaffold
  (this batch).
- T+30d: First migration call site (Identity verification
  emails).
- T+60d: All existing email-sending µservices migrated.
- T+90d: CI lane flips to BLOCKER on direct provider SDK
  imports.

## 13. Open questions

- Inbound email ADR — slot reserved.
- BIMI logo policy — slot reserved.
- Phase-2 in-house `oya-comms-email-server` triggers (parity
  with SES + sovereign Rust-native footprint) — ADR-0201
  §"In-house roadmap" tracks.
