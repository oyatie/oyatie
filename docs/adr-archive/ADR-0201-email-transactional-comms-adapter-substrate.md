---
id: ADR-0201
status: Superseded
superseded_by: [ADR-701]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0201 — Email + transactional comms adapter substrate

- Status: Accepted
- Date: 2026-05-18
- Deciders: Substrate architecture authority (oya-architecture-authority)
- Tags: substrate, communications, adapters, deliverability, multi-tenant
- Supersedes: none
- Superseded by: none
- Related: ADR-0064 (canonical base + localization packs — DKIM
  domain set is per-pack),
  ADR-0145 (inter-microservice communication reform — webhook
  events flow into the audit chain),
  ADR-0166 (event-schema versioning — email delivery events are
  versioned events),
  ADR-0173 (vendor lock-in avoidance — no single-provider lock-in
  allowed for comms),
  ADR-0174 (FinOps cost attribution — per-tenant email cost
  attributed via send metering).

## Context

Transactional email is a substrate concern, not a per-µservice
concern. The current footprint already needs it from at least:

- Identity (verification, password reset, MFA enrollment)
- Tenancy (invitation, role-change notification, deactivation)
- Workflow Studio (run-completed digest, alerting)
- Foundry (long-running run completion + cost-cap warnings)
- Audit chain (regulatory disclosure mailings)
- Billing (planned, per ADR-0174 FinOps lane)

A single per-µservice SES-only integration violates ADR-0173
(vendor lock-in) and forces every µservice to re-implement DKIM,
suppression-list management, bounce/complaint feedback loops,
per-tenant rate limits, and localized templating.

## Decision

Introduce a substrate-level email-comms adapter pattern owned by
the new `microservices/comms-email/` µservice and exposed via the
`crates/oya-shared-email-comms-kernel` trait + real adapter set.

### Adapter set (no Noop fallback)

- **`SesEmailComms`** — AWS SES (default for cloud-hosted clusters).
- **`PostalEmailComms`** — Postal self-hosted (AGPL), Ruby on Rails
  + RabbitMQ + MariaDB. Default for sovereign-tier and air-gapped
  deployments. Ships as a Helm chart under
  `microservices/comms-email/iac/helm/postal/`.
- **`MailgunEmailComms`** — Alternative SaaS path; commercial but
  retained as a second-source so SES is never the only path.
- **`SmtpEmailComms`** — Generic RFC 5321 fallback. Last-resort
  adapter for unknown infrastructures (relay-only smarthost,
  partner-managed MTAs, isolated lab environments).

### Per-tenant deliverability primitives

- **DKIM**: per-tenant from-domain with DKIM signing key stored in
  OpenBao (per ADR-0173 secrets storage). Rotation is annual or
  on-revocation. **DKIM signing is mandatory on every outbound
  message** — adapters that cannot sign reject the send at
  pre-flight, never silently emit unsigned mail.
- **SPF**: each tenant from-domain publishes an SPF record that
  authorizes the configured provider's send sources (SES, Postal
  MTA pool, Mailgun, SMTP relay). The µservice exposes a tooling
  endpoint that emits the canonical SPF record string per tenant
  pack so DNS automation (Tier-B OpenTofu DNS module per
  ADR-0202) can publish it.
- **DMARC**: every tenant pack publishes a DMARC record. Default
  policy is `p=quarantine` for new tenants and `p=reject` for
  tenants past their warm-up window. Reports (RUA/RUF) are
  ingested back into the audit chain (ADR-0145) for tamper
  detection.
- **BIMI** (out of scope this ADR — follow-up).
- **Reply-To**: per-tenant override; default points at the tenant's
  configured support inbox.
- **Bounce / complaint feedback**: each adapter delivers normalized
  events into the audit chain (ADR-0145) and the schema registry
  (ADR-0166). Event kinds: `sent`, `delivered`, `opened`,
  `clicked`, `bounced`, `complained`, `suppressed`.

### Templating

- **MJML** as the canonical responsive-email markup. Compiled via
  the `mrml` Rust crate (footnoted floor: 5.x LTS line as of
  2026-05-18). MJML compiles to cross-client HTML.
- **Liquid** as the variable-substitution language inside compiled
  MJML output. Liquid (the `liquid` Rust crate, footnoted floor:
  0.26.x line as of 2026-05-18) is preferred over Handlebars
  because of its sandbox guarantees and i18n track record.
- **Per-locale templates**: each template ships per locale per
  pack (ADR-0064). Fallback chain: tenant-override → pack-locale →
  pack-default → canonical-base.

### Rate limits + suppression

- Per-tenant send-rate ceiling configured in the manifest delta
  (see manifest delta section below). The kernel exposes a
  capability-token-checked `send` API so violations surface as
  pre-send rejection, not provider error.
- Suppression list is canonical across adapters; entries are
  written from any bounce/complaint event with the same key.

### Push notifications

Out of scope. A follow-up ADR will define the push (APNs / FCM /
WebPush) substrate using the same adapter pattern.

## Alternatives considered

- **SendGrid / Resend / Postmark** — All commercial-only,
  rejected for canonical-default per ADR-0173. SendGrid is
  acceptable as an externally-pluggable adapter at the tenant's
  request, not as a default.
- **SES exclusive** — Forces every customer onto AWS. Violates
  ADR-0173 vendor lock-in avoidance. Rejected.
- **Postal exclusive** — Self-hosted-only stance shuts out
  cloud-hosted tenants that don't want operational burden.
  Rejected.
- **Roll-your-own MTA** — Outside oyatie's substrate scope.
  Rejected.

## Consequences

- New µservice `microservices/comms-email/` is created with full
  audit-grade pack (PRD, IPs, manifest, compliance, dpia,
  threat-model, slos, cost-budget, multi-region, etc.).
- All µservices that send email migrate to the kernel within the
  90-day window. The migration ADR addendum (T+30d) enumerates
  call sites.
- Existing one-off SES integrations are refactored behind the
  `EmailComms` trait. Direct `aws-sdk-sesv2` use outside the
  adapter is a discipline violation enforced by the existing
  layered-architecture lane.
- Helm chart for Postal is committed under
  `microservices/comms-email/iac/helm/postal/`.
- AWS SES is config-only (no chart needed); manifest delta carries
  the configuration knob.
- Webhook event flow joins ADR-0145 audit chain and ADR-0166
  schema-registry-versioned events.

## Standards anchor

- `crates/oya-shared-email-comms-kernel/src/lib.rs` — trait
  surface + four real adapter impls (no Noop).
- `microservices/comms-email/PRD.md` — pack-owner contract.
- `microservices/comms-email/iac/helm/postal/` — sovereign-tier
  chart.

## Manifest delta (parent wires)

- `comms.email.provider` ∈ {`ses`, `postal`, `mailgun`, `smtp`}.
- `comms.email.per_tenant_rate_limit_per_min`.
- `comms.email.dkim_rotation_days` (default 365).
- `comms.email.default_from_domain` (per tenant pack overlay).

## Migration

- T+0 (this ADR): ADR + kernel crate + Helm chart + µservice
  scaffold.
- T+30d: All existing senders migrated to the kernel.
- T+60d: Lane discipline gate flips to BLOCKER on direct provider
  SDK imports outside the adapter set.

## In-house roadmap

This ADR is the clearest case in the batch for in-house Phase-2
build because email is split between open standards (keep) and
vendor adapters (replace).

### Keep as community/open standards

- **SMTP (RFC 5321 / 5322)**: open standard. Used as the wire
  protocol for the in-house relay (see below) and the `Smtp`
  fallback adapter.
- **DKIM / SPF / DMARC**: open standards. Owned by the kernel;
  enforced at preflight; never adapter-specific.
- **MJML** (open source, Apache-2.0): canonical templating
  source. Stays.
- **Liquid** (open source, dual MIT/Apache-2.0): canonical
  variable substitution. Stays.

### Phase 0 vendor adapters (gated, never canonical)

- **SES adapter** — AWS-only path. Gated behind the adapter
  trait; allowed only when the deployment is already on AWS.
  Tenant manifest overrides default to a non-SES adapter when
  the cluster lives outside AWS. Per ADR-0173 vendor lock-in
  avoidance, no µservice may assume SES.
- **Mailgun adapter** — commercial second-source. Gated; allowed
  when the customer explicitly chooses it. Never the default.
- **Postal adapter** — open source AGPL Phase-0 vendor for
  sovereign / air-gapped tier. Self-hosted via Helm chart
  shipped in this batch.

### Phase 2 in-house build (planned)

`oya-comms-email-server` — in-house Rust-native SMTP relay with
the following surface, planned to reach parity-with-SES at scale
and replace Postal for any deployment that wants a single
Rust-native operator footprint:

- DKIM key generation + rotation (annual + on-revocation), keys
  in OpenBao.
- Per-tenant from-domain + Reply-To handling.
- MJML compile + Liquid substitution at send time (kernel-side,
  not adapter-side — already true today).
- Webhook delivery events normalized into ADR-0145 audit chain +
  ADR-0166 schema-registry events.
- Per-tenant rate ceilings + suppression-list as first-class
  state.
- Outbound SMTP over TLS 1.3; STARTTLS for any peer that
  requires it; explicit deliverability dashboards.

**Trigger conditions** (either fires Phase 2). Numeric, not
aspirational:

1. Parity with SES at scale (any single tenant sustains
   ≥ 1,000,000 sends/day for ≥ 7 consecutive days OR
   cluster aggregate ≥ 10,000,000 sends/day for ≥ 7
   consecutive days) AND at least one of:
   - per-send cost > $0.15 / 1k (75% above 2026-05-18 SES
     baseline of $0.10 / 1k),
   - SES regional quota cap reached repeatedly in any 24h
     window,
   - audit-trail requirement that SES SendEmail API
     cannot furnish (e.g. per-event sub-second correlation
     back to ADR-0145 chain).
2. Sovereign / air-gapped deploy: no inbound or outbound
   internet to AWS AND customer compliance posture (ITAR,
   CMMC L3, KSA PDPL Tier 1, UAE NESA) rejects Postal's
   Ruby + RabbitMQ + MariaDB stack OR mandates Rust-only
   operator footprint.

### SES adapter — AWS coupling disclosure

The SES adapter is structurally coupled to AWS — SDK, IAM
role, regional endpoint set. When the deployment is on-prem
(no AWS account at all) the SES adapter is unreachable;
tenants must use Postal / Mailgun / SMTP. This is acceptable
in Phase 1 because (a) the adapter is gated, (b) the kernel
rejects sends that point at an unreachable provider, (c)
sovereign packs already force Postal-only at config-load
(IP-014). On-prem deployments today route through Postal;
Phase 2 in-house `oya-comms-email-server` is the path forward
for on-prem deployments wanting a Rust-native single-process
footprint.

When Phase 2 ships, the kernel `EmailComms` trait gains a fifth
adapter (`OyaCommsEmailServerAdapter`) and existing adapters stay
operational for transition.

### In-house contribution path

For Postal / MJML / Liquid / SES SDK we contribute upstream when
fixes land in our adapter that belong upstream. Per ADR-0173
contribution-back policy.

## Open questions

- Inbound email (replies, support inbox ingestion) is out of
  scope for this ADR and slotted for a follow-up.
- Provider-specific extensions (e.g. SES configuration sets) are
  exposed via a typed extension struct on the trait — exact
  shape ratified per adapter at first integration.
