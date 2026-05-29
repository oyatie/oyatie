---
doc_class: DeprecationPlan
title: "Deprecation plan"
microservice: plugin-app-store
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Deprecation plan


## API deprecation policy

- ≥ 12-month sunset window per ADR (per `feedback_no_silent_regression.md`).
- Deprecated endpoints return 200 with `Deprecation: <RFC9745-date>` + `Sunset: <RFC8594-date>` headers.
- SDK families ship with the deprecation header surfaced in client warnings.

## Plugin deprecation flow

- Publisher marks plugin version Deprecated → continue serving installations but block new installs.
- Tenants notified 30 days before Retired transition.
- Retired transition: existing installations continue running until tenant uninstalls; no new installs.
- Revoked transition: kill-switch; all installations stop within 30s p99.

## ADR-0213 sunset clause

- ADR-0213 itself has no sunset window; this is the architectural authority.
- Each sub-ADR ships with its own sunset evaluation cycle (every 18 months).

