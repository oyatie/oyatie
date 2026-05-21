---
doc_class: Reference
shape: Explanation
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0245, ADR-0273]
companion_docs:
  - microservices/mail/PRD.md
  - microservices/mail/ARCHITECTURE.md
  - microservices/mail/manifest.json
inbound_citations:
  - docs/README.md
  - docs/DOC-CATALOG.md
---

# Mail µservice — README

## What this µservice does

The Mail µservice is oyatie's personal + B2B email product. JMAP RFC 8620 primary; IMAP/POP3 secondary; iCalendar + Sieve for filters; per-tenant DKIM/SPF/DMARC per ADR-0273. Hyperscaler precedent: Gmail / Outlook / Apple Mail / Fastmail / ProtonMail / Hey.com.

## Quick links

- Product requirements: `PRD.md`
- Architecture walkthrough: `ARCHITECTURE.md`
- Threat model: `threat-model.md`
- DPIA: `dpia.md`
- Compliance: `compliance.md`
- Capacity model: `capacity-model.md`
- Cost budget: `cost-budget.md`
- Failure modes: `failure-modes.md`
- Multi-region: `multi-region.md`
- Incident response: `incident-response.md`
- Backfill replay: `backfill-replay.md`
- Competitor parity: `competitor-parity-matrix.md`
- SDK plan: `sdk-plan.md`
- Contracts: `contracts/openapi/mail.yaml`, `contracts/asyncapi/mail-events.yaml`, `contracts/proto/mail.proto`
- Cedar fragments: `policy/*.cedar`
- Runbooks: `runbooks/*.md`
- IPs: `IP-*.md`
- Dashboards: `dashboards/*.{json,md}`
- SLOs: `slos/*.openslo.yaml`
- Catalog: `catalog/*.yaml`
- IaC: `iac/**`

## How to consume

- Web / mobile clients: use JMAP over HTTP/3 via `contracts/openapi/mail.yaml`.
- Legacy clients: IMAP/POP3 over TLS 1.3.
- Calendar integration: iCalendar via JMAP-calendars extension.
- Server-side filters: Sieve (RFC 5228) per-tenant.

## Tenant Class Model

Mail follows ADR-0330. Customer access is modeled with `tenant_class`
(`demo_trial`, `paid`) and paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Demo-trial behavior is bounded by
usage and OCI Always Free constraints; paid tenants receive the same product
quality bar with billing components composed by contract.

## Status

Product, ga. Eligible for cell topology placement. HIPAA pack overlay available for B2B PHI mailboxes.
