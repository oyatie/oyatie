---
doc_class: Reference
shape: Reference
microservice: comms-email
companion_docs:
  - microservices/comms-email/ARCHITECTURE.md
  - microservices/comms-email/PRD.md
related_adrs: [ADR-0201, ADR-0273]
---

# comms-email

Transactional + marketing email substrate. Hyperscaler precedents: SendGrid, Mailgun,
Postmark, Amazon SES, Resend, Mailchimp, Klaviyo. Per ADR-0273 per-tenant
DKIM/SPF/DMARC. Postal as self-hosted relay for sovereign packs.

## Bounded contexts

`outbound-delivery` / `inbound-receiving` / `template-rendering` / `list-management` /
`unsubscribe-handling` / `deliverability-tracking` / `dkim-spf-dmarc-management` /
`bounce-handling` / `reputation-monitoring`.

## Entry points

PRD.md, ARCHITECTURE.md, threat-model.md, dpia.md, compliance.md, runbooks/.

## Tenant Class Model

comms-email follows ADR-0330: tenant eligibility is expressed as
`tenant_class` (`demo_trial` or `paid`) plus paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Customer-facing capability
ladders are retired; deliverability, DKIM custody, warmup, compliance-pack
routing, and send-volume behavior are governed by tenant class,
compliance packs, and cell topology.
