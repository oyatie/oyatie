---
doc_class: DPIA
title: "Data Protection Impact Assessment"
microservice: plugin-app-store
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Data Protection Impact Assessment


## Scope

EU GDPR Article 35 DPIA covering the µservice's processing of personal data + developer / tenant data flows.

## Data flows

1. **Tenant operator → plugin-app-store**: tenant_id, principal_id, plugin selection, grant decisions. Lawful basis: legitimate interest + contract.
2. **Developer → developer-sdk**: legal_name, email, ID document, liveness video, bank account, tax_id. Lawful basis: contract + legal obligation (KYC / AML).
3. **Plugin runtime → tenant data**: scoped by Cedar policy + declared data classes. Lawful basis: tenant-granted consent.
4. **plugin-app-store / developer-sdk → audit-chain**: seal events. Lawful basis: legitimate interest + legal obligation.
5. **developer-sdk → bank**: payout settlement messages. Lawful basis: contract.

## Risks + mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Cross-tenant data leak via plugin | High | Cedar policy tenant-scope BLOCKER |
| Developer PII leak via OpenBao compromise | High | OpenBao auto-unseal HA + audit chain |
| KYC document retention exceeds purpose | Medium | 7-year retention per BSA; deletion job nightly |
| Plugin manifest leaks tenant-identifiable info | Low | Manifest review in vetting pipeline |

## Data subject rights

- Access: tenant operator + developer can export their data via REST.
- Rectification: REST + portal.
- Erasure: developer revocation cascades; tenant uninstall removes per-installation data after 90d retention.
- Portability: data export endpoints return JSON.
- Objection: opt-out of marketing-only data uses.

## Cross-border transfers

- KR data stays in ap-northeast-2 (PIPA).
- EU data stays in eu-central-1 (GDPR).
- US-public-sector data stays in us-gov-east-1 (ITAR / EAR-eligible).

