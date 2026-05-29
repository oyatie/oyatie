---
ip_id: IP-021
microservice: finops-portal
bounded_context: showback-chargeback
layer: domain
related_adrs: [ADR-0199, ADR-0244]
---

# IP-021 — showback-chargeback domain

## Goal

Tenant-internal showback (visibility) + chargeback (actual transfer). Hyperscaler precedent:
Vantage Showback + Apptio Cloudability chargeback.

## Crate

`oya-finops-portal-showback-chargeback-domain`.

## Acceptance

- Per-sub-scope cost attribution.
- Configurable allocation method (proportional / equal / metered).
- Audit event `ShowbackChargebackEmitted`.
