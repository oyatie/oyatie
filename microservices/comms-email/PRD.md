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

## 11. Rollout

- T+0: ADR-0201 + kernel crate + Helm chart + µservice scaffold
  (this batch).
- T+30d: First migration call site (Identity verification
  emails).
- T+60d: All existing email-sending µservices migrated.
- T+90d: CI lane flips to BLOCKER on direct provider SDK
  imports.

## 12. Open questions

- Inbound email ADR — slot reserved.
- BIMI logo policy — slot reserved.
- Phase-2 in-house `oya-comms-email-server` triggers (parity
  with SES + sovereign Rust-native footprint) — ADR-0201
  §"In-house roadmap" tracks.
