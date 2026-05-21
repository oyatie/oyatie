---
microservice: connect
doc_class: README
date: 2026-05-20
owner_team: axis-integration
status: Accepted
related_adrs: [ADR-0145, ADR-0245, ADR-0246, ADR-0249, ADR-0255, ADR-0296, ADR-0297]
companion_docs:
  - microservices/connect/PRD.md
  - microservices/connect/ARCHITECTURE.md
  - microservices/connect/CHANGELOG.md
inbound_citations: [microservices/connect/PRD.md, docs/DOC-CATALOG.md]
doc_status: published
---

# connect — Integration Substrate

oyatie's tenant-scoped **integration substrate**: a connector directory (≥500 adapters), OAuth broker, webhook receiver, signature verification, payload canonicalization, retry/DLQ machinery, and data-mapping engine — the substrate every product surface uses to talk to external SaaS APIs.

## What this µservice is

- A directory of ≥500 pre-built connector adapters (Slack, Salesforce, Stripe, Shopify, GitHub, AWS, Toss Payments, etc.).
- An OAuth 2.0 / OIDC / JWT-bearer / client-credentials broker with per-tenant provider-credential BYOK client provisioning (ADR-0255 §D-4).
- A per-tenant webhook receiver with HMAC verification, replay-window ≤5min, idempotency-key dedup, backpressure.
- A retry-and-DLQ engine with exponential backoff, circuit-breakers (ADR-0145 §invariant-1), per-tenant token-bucket rate limiting.
- A visual data-mapper with schema-drift detection.

## What this µservice is NOT

- It is NOT workflow-engine. Connect is the substrate; workflow-engine consumes it.
- It is NOT api-gateway. api-gateway handles north-south for oyatie's own surfaces; connect handles outbound to third-party SaaS.
- It is NOT a credential store. OpenBao (cloud-secrets µservice) holds raw credentials; connect holds `SecretReference` strings.

## Hyperscaler precedents

Zapier (App Directory + OAuth + webhooks); n8n (open-source workflow with connector library); Workato (enterprise integration); Boomi (data mapping + connectors); MuleSoft (connectors + auth); Tray.io (low-code integration); Pipedream (event-driven workflows + connectors); AWS EventBridge (webhook/event-bus substrate).

## Quick links

- [PRD.md](PRD.md) — product requirements, user stories, NFRs
- [ARCHITECTURE.md](ARCHITECTURE.md) — layer-by-layer walkthrough
- [threat-model.md](threat-model.md) — STRIDE for each BC
- [dpia.md](dpia.md) — DPIA (GDPR Art. 35)
- [compliance.md](compliance.md) — pack overlays + cert readiness
- [policy/](policy/) — Cedar fragments
- [runbooks/](runbooks/) — operational procedures
- [contracts/](contracts/) — OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3
- [catalog/](catalog/) — crate records + connector seed
- [slos/](slos/) — OpenSLO manifests
- [iac/](iac/) — Helm/Terraform/Kustomize

## Status

Substrate; M01-foundation milestone; consumed by workflow-engine + marketplace + foundry + ops-dashboard + intelligence.

## Tenant class model

Connect follows the `tenant_class` model from [ADR-0330](../../docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md). The service no longer carries customer capability ladder vocabulary; commercial eligibility is expressed as `tenant_class = demo_trial | paid`, and paid commercial shape is expressed through `billing_components` (`revenue_share`, `per_seat`, `per_usage`). Product-quality controls remain uniform; regulated behavior is gated by compliance packs and cell topology where required.

## Retirement coordination note

The pre-2026-05-20 `connect` umbrella retirement plan (now in `RETIREMENT-PLAN.md` + `IP-001-connect-retirement-design-readiness.md`) covered the dissolved consumer-suite umbrella (mail/messenger/calendar/community/etc.). The integration-substrate scope described here is **a new µservice scope**, not the umbrella; the retirement plan remains canonical for the *umbrella* artifacts but is orthogonal to this substrate.

## Owner

axis-integration (per `manifest.json`).
